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
import os, sys, json, re, math, urllib.request, hashlib

OMEGA = os.environ.get("OMEGA_DIR") or os.path.join(os.path.expanduser("~"), ".omega")
ATLAS = os.path.join(OMEGA, "skills-atlas.json")
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

def _corpus():
    a = json.load(open(ATLAS))
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

def _embed(texts, key):
    """Batch-embed via OpenAI REST (stdlib only)."""
    out = []
    B = 256
    for i in range(0, len(texts), B):
        chunk = texts[i:i+B]
        body = json.dumps({"model": MODEL, "input": chunk}).encode()
        req = urllib.request.Request(
            "https://api.openai.com/v1/embeddings", data=body,
            headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = json.load(resp)
        out.extend(d["embedding"] for d in sorted(data["data"], key=lambda x: x["index"]))
    return out

def build():
    os.makedirs(RAGDIR, exist_ok=True)
    rows = _corpus()
    key = _key()
    meta = {"model": None, "backend": "bm25", "rows": rows,
            "corpus_hash": hashlib.md5("".join(r["text"] for r in rows).encode()).hexdigest()}
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
    json.dump(meta, open(META, "w"))
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
    if not os.path.isfile(META):
        build()
    meta = json.load(open(META))
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
