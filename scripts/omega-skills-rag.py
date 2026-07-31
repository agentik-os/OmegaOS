#!/usr/bin/env python3
"""Skill RAG — semantic retrieval over the full skill corpus (native + Power-Up
library) so a session finds the right skill by MEANING, not just keywords.

Backends (auto):
  - EMBEDDINGS (preferred): OpenAI text-embedding-3-small, cached vectors on disk.
    Needs OPENAI_API_KEY (~/.omega/secrets/integrations.env). Best quality.
  - BM25 (fallback): pure-Python lexical ranking. No key, always works, offline.

Usage:
  omega-skills-rag.py build            # (re)build the index from skills-atlas.json
  omega-skills-rag.py query "<text>"   # top-K skills for a natural-language need
  omega-skills-rag.py query "<text>" --k 8 --json
"""
import os, sys, json, re, math, urllib.request, hashlib, subprocess

OMEGA = os.environ.get("OMEGA_DIR") or os.path.join(os.path.expanduser("~"), ".omega")
ATLAS = os.path.join(OMEGA, "skills-atlas.json")
CATALOG = os.environ.get("OMEGA_SKILL_CATALOG") or os.path.join(OMEGA, "skill-catalog-v1.json")
RAGDIR = os.path.join(OMEGA, "skills-rag")
VEC = os.path.join(RAGDIR, "vectors.npy")
META = os.path.join(RAGDIR, "meta.json")
MODEL = "text-embedding-3-small"

def _key():
    k = os.environ.get("OPENAI_API_KEY")
    if k:
        return k.strip()
    env = os.path.join(OMEGA, "secrets/integrations.env")
    if os.path.isfile(env):
        for line in open(env):
            m = re.match(r"\s*(?:export\s+)?OPENAI_API_KEY\s*=\s*(.+)", line)
            if m:
                return m.group(1).strip().strip('"\'')
    return None

def _canonical_hash():
    if not os.path.isfile(CATALOG):
        return None
    try:
        catalog = json.load(open(CATALOG, encoding="utf-8"))
        if catalog.get("schema_version") != 1:
            return None
        digest = catalog.get("content_digest")
        return digest if isinstance(digest, str) else None
    except (OSError, ValueError, TypeError):
        return None

def _ensure_atlas_current():
    """Rebuild Atlas before RAG if its canonical catalog projection drifted."""
    expected = _canonical_hash()
    atlas = None
    if os.path.isfile(ATLAS):
        try:
            atlas = json.load(open(ATLAS, encoding="utf-8"))
        except (OSError, ValueError, TypeError):
            atlas = None
    stale = atlas is None or (expected is not None and atlas.get("catalog_hash") != expected)
    if stale:
        generator = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 "omega-skills-atlas.py")
        if not os.path.isfile(generator):
            raise RuntimeError(
                f"skill Atlas is stale and generator is unavailable: {generator}")
        subprocess.run([sys.executable, generator], check=True)
        atlas = json.load(open(ATLAS, encoding="utf-8"))
        if expected is not None and atlas.get("catalog_hash") != expected:
            raise RuntimeError("Atlas rebuild did not consume the current canonical catalog")
    return atlas

def _corpus(atlas=None):
    a = atlas or _ensure_atlas_current()
    rows = []
    for r in a.get("native", []):
        rows.append({"name": r["name"], "text": f'{r["name"]}. {r.get("description","")}',
                     "commands": r.get("commands", []), "source": r.get("source", "omegaos"),
                     "group": r.get("group", ""), "path": r.get("slug", "")})
    for r in a.get("powerups", []):
        rows.append({"name": r["name"], "text": f'{r["name"]}. {r.get("description","")}',
                     "commands": [], "source": r.get("source", "powerup"),
                     "group": r.get("group", ""), "path": r.get("path", "")})
    return rows

def _rows_hash(rows):
    payload = json.dumps(rows, ensure_ascii=False, sort_keys=True,
                         separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()

OPENAI_URL = "https://api.openai.com/v1/embeddings"

def _omniroute_embed_url():
    """OmniRoute /v1/embeddings if the gateway is up AND actually serves an
    embedding provider; else None. Prefer the OmegaOS API gateway so embedding
    calls get its routing + fallback; degrade cleanly to OpenAI-direct otherwise.
    Override with OMEGA_EMBED_BASE_URL (e.g. http://127.0.0.1:20128/v1)."""
    base = os.environ.get("OMEGA_EMBED_BASE_URL")
    if base:
        return base.rstrip("/") + "/embeddings"
    port = os.environ.get("OMNIROUTE_PORT", "20128")
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/v1/models", timeout=3) as r:
            d = json.load(r); models = d.get("data", d) if isinstance(d, dict) else d
            if any("embed" in (m.get("id") or "").lower() for m in models):
                return f"http://127.0.0.1:{port}/v1/embeddings"
    except Exception:
        pass
    return None

def _embed_batch(url, chunk, key):
    body = json.dumps({"model": MODEL, "input": chunk}).encode()
    headers = {"Content-Type": "application/json"}
    if key:
        headers["Authorization"] = f"Bearer {key}"
    req = urllib.request.Request(url, data=body, headers=headers)
    with urllib.request.urlopen(req, timeout=60) as resp:
        data = json.load(resp)
    return [d["embedding"] for d in sorted(data["data"], key=lambda x: x["index"])]

def _embed(texts, key):
    """Batch-embed. Route via OmniRoute (/v1) when it can serve embeddings, else
    OpenAI-direct; on an OmniRoute error, fall back to OpenAI-direct per batch."""
    primary = _omniroute_embed_url()
    out = []
    B = 256
    for i in range(0, len(texts), B):
        chunk = texts[i:i+B]
        if primary:
            try:
                out.extend(_embed_batch(primary, chunk, key)); continue
            except Exception as e:
                print(f"[rag] OmniRoute embeddings failed ({str(e)[:60]}); OpenAI-direct fallback",
                      file=sys.stderr)
                primary = None  # stop retrying the gateway this run
        out.extend(_embed_batch(OPENAI_URL, chunk, key))
    return out

def build():
    os.makedirs(RAGDIR, exist_ok=True)
    atlas = _ensure_atlas_current()
    rows = _corpus(atlas)
    key = _key()
    meta = {"model": None, "backend": "bm25", "rows": rows,
            "catalog_hash": atlas.get("catalog_hash"),
            "atlas_hash": atlas.get("atlas_hash"),
            "corpus_hash": _rows_hash(rows)}
    if key:
        try:
            import numpy as np
            vecs = _embed([r["text"] for r in rows], key)
            arr = np.array(vecs, dtype="float32")
            arr /= (np.linalg.norm(arr, axis=1, keepdims=True) + 1e-9)
            np.save(VEC, arr)
            meta["model"] = MODEL
            meta["backend"] = "embeddings"
            print(f"[rag] embeddings index built: {len(rows)} skills ({MODEL})")
        except Exception as e:
            print(f"[rag] embedding build failed ({e}); using BM25 fallback", file=sys.stderr)
    else:
        print(f"[rag] no OPENAI_API_KEY; built BM25 lexical index ({len(rows)} skills)")
    temp = META + ".tmp"
    with open(temp, "w", encoding="utf-8") as handle:
        json.dump(meta, handle, ensure_ascii=False, sort_keys=True)
        handle.write("\n")
    os.replace(temp, META)
    return meta

def _tok(s): return re.findall(r"[a-z0-9]+", s.lower())

def _bm25(query, rows, k):
    docs = [_tok(r["text"]) for r in rows]
    N = len(docs); avgdl = sum(len(d) for d in docs) / max(N, 1)
    df = {}
    for d in docs:
        for t in set(d):
            df[t] = df.get(t, 0) + 1
    q = _tok(query); k1, b = 1.5, 0.75
    scores = []
    for i, d in enumerate(docs):
        tf = {}
        for t in d: tf[t] = tf.get(t, 0) + 1
        s = 0.0
        for t in q:
            if t not in tf: continue
            idf = math.log(1 + (N - df.get(t, 0) + 0.5) / (df.get(t, 0) + 0.5))
            s += idf * (tf[t] * (k1 + 1)) / (tf[t] + k1 * (1 - b + b * len(d) / avgdl))
        scores.append((s, i))
    scores.sort(reverse=True)
    return [(rows[i], sc) for sc, i in scores[:k] if sc > 0]

def query(text, k=6, as_json=False):
    atlas = _ensure_atlas_current()
    meta = None
    if os.path.isfile(META):
        try:
            candidate = json.load(open(META, encoding="utf-8"))
            current_rows = _corpus(atlas)
            if (
                candidate.get("catalog_hash") == atlas.get("catalog_hash")
                and candidate.get("atlas_hash") == atlas.get("atlas_hash")
                and candidate.get("corpus_hash") == _rows_hash(current_rows)
            ):
                meta = candidate
        except (OSError, ValueError, TypeError):
            meta = None
    if meta is None:
        print("[rag] index drift detected; rebuilding", file=sys.stderr)
        meta = build()
    rows = meta["rows"]
    results = []
    if meta.get("backend") == "embeddings" and os.path.isfile(VEC):
        key = _key()
        if key:
            try:
                import numpy as np
                arr = np.load(VEC)
                qv = np.array(_embed([text], key)[0], dtype="float32")
                qv /= (np.linalg.norm(qv) + 1e-9)
                sims = arr @ qv
                idx = sims.argsort()[::-1][:k]
                results = [(rows[i], float(sims[i])) for i in idx]
            except Exception as e:
                print(f"[rag] embedding query failed ({e}); BM25 fallback", file=sys.stderr)
    if not results:
        results = _bm25(text, rows, k)

    if as_json:
        print(json.dumps([{"name": r["name"], "score": round(s, 4), "source": r["source"],
                           "commands": r["commands"], "path": r["path"],
                           "description": r["text"].split(". ", 1)[-1][:200]}
                          for r, s in results], ensure_ascii=False, indent=2))
        return
    be = meta.get("backend", "bm25")
    print(f"\033[2m[rag: {be}]  query: {text}\033[0m")
    for r, s in results:
        cmd = ("  " + " ".join(r["commands"])) if r["commands"] else "  (library)"
        d = re.sub(r"\s+", " ", r["text"].split(". ", 1)[-1])[:100]
        print(f"  \033[1m{r['name']}\033[0m\033[36m{cmd}\033[0m  \033[2m{s:.3f}\033[0m")
        print(f"      {d}")

def main():
    args = sys.argv[1:]
    if not args or args[0] in ("-h", "--help"):
        print(__doc__); return
    if args[0] == "build":
        build(); return
    if args[0] == "query":
        rest = args[1:]
        k = 6; as_json = False; terms = []
        i = 0
        while i < len(rest):
            if rest[i] == "--k" and i + 1 < len(rest): k = int(rest[i+1]); i += 2
            elif rest[i] == "--json": as_json = True; i += 1
            else: terms.append(rest[i]); i += 1
        query(" ".join(terms), k=k, as_json=as_json); return
    print(f"unknown command: {args[0]}", file=sys.stderr); sys.exit(2)

if __name__ == "__main__":
    main()
