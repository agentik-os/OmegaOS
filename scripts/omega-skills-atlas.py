#!/usr/bin/env python3
"""OmegaOS Skill Atlas.

The canonical input is SkillCatalogV1 exported by omega-core. During the staged
migration, a recursive, bounded legacy reader remains available when that
artifact is absent. Both paths exclude vendor/build trees and fail on duplicate
skill identities instead of silently dropping one.
"""
import os, re, json, html, datetime, hashlib, sys, unicodedata

HOME = os.path.expanduser("~")
OMEGA = os.environ.get("OMEGA_DIR") or os.path.join(HOME, ".omega")
NATIVE = os.path.join(OMEGA, "skills")
CATALOG = os.environ.get("OMEGA_SKILL_CATALOG") or os.path.join(OMEGA, "skill-catalog-v1.json")
POWERUP_MANIFEST = os.path.join(OMEGA, "skills-library/youraipowerup/MANIFEST.json")
COOKBOOK_INDEX = os.path.join(OMEGA, "cookbooks-index.json")
COOKBOOK_CORPUS = os.path.join(OMEGA, "cookbooks")
EXCLUDED_DIRS = {".git", ".venv", "build", "dist", "node_modules", "target", "vendor"}
MAX_SKILL_BYTES = 2 * 1024 * 1024
MAX_SKILLS = 10_000

def parse_frontmatter(path):
    """Legacy fallback parser for required name and description fields."""
    if os.path.getsize(path) > MAX_SKILL_BYTES:
        raise ValueError(f"skill exceeds {MAX_SKILL_BYTES} bytes: {path}")
    txt = open(path, encoding="utf-8", errors="strict").read()
    m = re.match(r"^---\s*\n(.*?)\n---", txt, re.S)
    if not m:
        raise ValueError(f"missing YAML frontmatter: {path}")
    block = m.group(1)
    lines = block.split("\n")
    name = None; desc = ""
    i = 0
    while i < len(lines):
        line = lines[i]
        key = re.match(r"^(\w[\w-]*):\s*(.*)$", line)
        if key:
            k, v = key.group(1).lower(), key.group(2).strip()
            if k == "name" and name is None:
                name = v.strip('"\'')
            elif k == "description":
                if v in (">", "|", ">-", "|-", ">+", "|+"):
                    # folded: gather subsequent more-indented lines
                    buf = []; i += 1
                    while i < len(lines) and (lines[i].startswith((" ", "\t")) or lines[i].strip() == ""):
                        buf.append(lines[i].strip()); i += 1
                    desc = " ".join(x for x in buf if x)
                    continue
                else:
                    desc = v.strip('"\'')
        i += 1
    if not name:
        raise ValueError(f"missing required name: {path}")
    if not desc.strip():
        raise ValueError(f"missing required description: {path}")
    return name, desc

def derive_commands(name, group):
    """Keep the legacy slash surface while exposing the provider-neutral name."""
    cmds = [f"/{name}"]
    if group == "Audits" and not name.startswith("omg-"):
        cmds.append(f"/omg-{name}")
    return cmds

def identity(value):
    return unicodedata.normalize("NFKC", value).casefold()

def validate_unique(rows):
    seen = {}
    for row in rows:
        key = identity(row["name"])
        if key in seen:
            raise ValueError(
                f"duplicate skill identity: {seen[key]['name']} and {row['name']}")
        seen[key] = row

def canonical_native():
    if not os.path.isfile(CATALOG):
        return None, None
    try:
        data = json.load(open(CATALOG, encoding="utf-8"))
        if data.get("schema_version") != 1:
            raise ValueError(f"unsupported schema_version {data.get('schema_version')!r}")
        digest = data.get("content_digest")
        skills = data.get("skills")
        if not isinstance(digest, str) or len(digest) < 32 or not isinstance(skills, list):
            raise ValueError("canonical catalog is missing digest or skills")
        rows = []
        for skill in skills:
            name = skill.get("name")
            desc = skill.get("description")
            rel = skill.get("relative_path")
            if not all(isinstance(value, str) and value.strip() for value in (name, desc, rel)):
                raise ValueError("canonical skill is missing name, description, or relative_path")
            category = str(skill.get("category", "Custom"))
            group = "Audits" if category.lower() == "audit" else category
            if group.lower() in ("custom", "utility"):
                group = "Skills"
            rows.append({
                "name": name,
                "slug": os.path.dirname(rel).replace(os.sep, "/"),
                "description": desc,
                "commands": derive_commands(name, group),
                "group": group,
                "source": "omegaos",
                "provider_states": skill.get("provider_states", {}),
                "content_digest": skill.get("content_digest", ""),
            })
        rows.sort(key=lambda row: (identity(row["name"]), row["slug"]))
        validate_unique(rows)
        return rows, digest
    except Exception as exc:
        print(f"[atlas] canonical catalog invalid ({exc}); using legacy fallback",
              file=sys.stderr)
        return None, None

def legacy_native():
    rows = []
    if not os.path.isdir(NATIVE):
        return rows
    for current, dirs, files in os.walk(NATIVE, topdown=True, followlinks=False):
        dirs[:] = sorted(
            name for name in dirs
            if name not in EXCLUDED_DIRS and
            not os.path.islink(os.path.join(current, name)))
        if "SKILL.md" not in files:
            continue
        path = os.path.join(current, "SKILL.md")
        if os.path.islink(path):
            continue
        name, desc = parse_frontmatter(path)
        rel = os.path.relpath(path, NATIVE).replace(os.sep, "/")
        group = "Audits" if "/audits/" in f"/{rel}" or name.endswith("audit") else "Skills"
        rows.append({
            "name": name,
            "slug": os.path.dirname(rel),
            "description": desc,
            "commands": derive_commands(name, group),
            "group": group,
            "source": "omegaos-legacy",
            "provider_states": {},
            "content_digest": hashlib.sha256(
                open(path, "rb").read().replace(b"\r\n", b"\n")).hexdigest(),
        })
        if len(rows) > MAX_SKILLS:
            raise ValueError(f"legacy catalog exceeds {MAX_SKILLS} skills")
    rows.sort(key=lambda row: (identity(row["name"]), row["slug"]))
    validate_unique(rows)
    return rows

def cookbook_recipes():
    """Anthropic's own reference recipes (anthropics/claude-cookbooks, MIT).

    The index ships in the OmegaOS repo and is installed unconditionally, so
    these rows exist on a bare clone. The notebook CORPUS is optional
    (tools/cookbooks/install-cookbooks.sh); when it is absent the row still
    carries the pinned upstream URL, so a recipe is never advertised as local
    when it is not — the same guard the Power-Up router uses.
    """
    if not os.path.isfile(COOKBOOK_INDEX):
        return []
    try:
        data = json.load(open(COOKBOOK_INDEX, encoding="utf-8"))
        if data.get("schema_version") != 1:
            raise ValueError(f"unsupported schema_version {data.get('schema_version')!r}")
        recipes = data.get("recipes")
        if not isinstance(recipes, list):
            raise ValueError("cookbook index has no recipes list")
        commit = str(data.get("commit", ""))[:7]
        have_corpus = os.path.isdir(COOKBOOK_CORPUS)
        rows = []
        for r in recipes:
            name, path = r.get("name"), r.get("path")
            title = r.get("title") or name
            if not all(isinstance(v, str) and v.strip() for v in (name, path, title)):
                raise ValueError("cookbook recipe is missing name, path, or title")
            local = os.path.join(COOKBOOK_CORPUS, path) if have_corpus else ""
            rows.append({
                "name": name,
                "title": title,
                "description": r.get("description", ""),
                # the RAG embeds `text`; the intent phrasing is what makes a
                # plain-language need retrieve the right recipe
                "intent": r.get("intent", ""),
                "commands": [],
                "group": r.get("group", "Cookbook"),
                "source": "cookbook",
                "path": path,
                "url": r.get("url", ""),
                "local": local if (local and os.path.isfile(local)) else "",
                "commit": commit,
            })
        rows.sort(key=lambda row: (identity(row["group"]), identity(row["name"])))
        return rows
    except Exception as exc:
        print(f"[atlas] cookbook index invalid ({exc}); skipping", file=sys.stderr)
        return []
native, catalog_hash = canonical_native()
if native is None:
    native = legacy_native()
    legacy_projection = json.dumps(native, ensure_ascii=False, sort_keys=True,
                                   separators=(",", ":")).encode()
    catalog_hash = "legacy-sha256:" + hashlib.sha256(legacy_projection).hexdigest()

# ---- 2. power-up library ----
powerups = []
if os.path.isfile(POWERUP_MANIFEST):
    man = json.load(open(POWERUP_MANIFEST))
    for r in man["skills"]:
        powerups.append({
            "name": r["name"], "description": r["description"],
            "commands": [], "group": r["category"],
            "source": "powerup" if r["source"] == "powerup" else "bundle-501",
            "path": r["path"],
        })
powerups.sort(key=lambda row: (identity(row["name"]), row.get("path", "")))

# ---- 2b. Anthropic cookbook recipes ----
cookbooks = cookbook_recipes()

atlas = {
    "generated": datetime.date.today().isoformat(),
    "schema_version": 2,
    "catalog_hash": catalog_hash,
    "native_count": len(native),
    "powerup_count": len(powerups),
    "cookbook_count": len(cookbooks),
    "total": len(native) + len(powerups) + len(cookbooks),
    "native": native,
    "powerups": powerups,
    "cookbooks": cookbooks,
}
hash_payload = {
    "catalog_hash": catalog_hash,
    "native": native,
    "powerups": powerups,
    "cookbooks": cookbooks,
}
atlas["atlas_hash"] = hashlib.sha256(json.dumps(
    hash_payload, ensure_ascii=False, sort_keys=True,
    separators=(",", ":")).encode()).hexdigest()
os.makedirs(OMEGA, exist_ok=True)
atlas_path = os.path.join(OMEGA, "skills-atlas.json")
atlas_tmp = atlas_path + ".tmp"
with open(atlas_tmp, "w", encoding="utf-8") as handle:
    json.dump(atlas, handle, indent=2, ensure_ascii=False, sort_keys=True)
    handle.write("\n")
os.replace(atlas_tmp, atlas_path)

# ---- 3. HTML ----
def esc(s): return html.escape(s or "")

def render_cards(items, show_cmd=True):
    out = []
    for r in items:
        cmds = ""
        if show_cmd and r["commands"]:
            cmds = "".join(f'<code class="cmd">{esc(c)}</code>' for c in r["commands"])
        out.append(
            f'<div class="card" data-s="{esc(r["name"]).lower()} {esc(r["description"]).lower()} {esc(" ".join(r["commands"]))}">'
            f'<div class="nm">{esc(r["name"])}</div>'
            f'{("<div class=cmds>"+cmds+"</div>") if cmds else ""}'
            f'<div class="ds">{esc(r["description"])[:280]}</div></div>')
    return "\n".join(out)

# group native
native_groups = {}
for r in native:
    native_groups.setdefault(r["group"], []).append(r)
# group powerups by category
pu_groups = {}
for r in powerups:
    pu_groups.setdefault((r["source"], r["group"]), []).append(r)
# group cookbook recipes by their registry category
cb_groups = {}
for r in cookbooks:
    cb_groups.setdefault(r["group"], []).append(r)

native_html = []
for g in sorted(native_groups, key=lambda x: (x != "Skills", x)):
    items = sorted(native_groups[g], key=lambda r: r["name"].lower())
    native_html.append(f'<section class="cat" data-tab="native"><h2>{esc(g)} <span class="ct">{len(items)}</span></h2><div class="grid">{render_cards(items)}</div></section>')

pu_html = []
for (src, cat), items in sorted(pu_groups.items()):
    items = sorted(items, key=lambda r: r["name"].lower())
    badge = "bundle" if src == "bundle-501" else "power-up"
    pu_html.append(f'<section class="cat pu" data-tab="powerup"><h2>{esc(cat)} <span class="src {badge}">{esc(src)}</span> <span class="ct">{len(items)}</span></h2><div class="grid">{render_cards(items, show_cmd=False)}</div></section>')

cb_html = []
for cat in sorted(cb_groups):
    items = sorted(cb_groups[cat], key=lambda r: r["name"].lower())
    cards = []
    for r in items:
        where = "local" if r.get("local") else "upstream"
        link = esc(r.get("url", ""))
        cards.append(
            f'<div class="card" data-s="{esc(r["name"]).lower()} {esc(r["title"]).lower()} '
            f'{esc(r["description"]).lower()} {esc(r["intent"]).lower()}">'
            f'<div class="nm">{esc(r["title"])}</div>'
            f'<div class="cmds"><code class="cmd">{esc(r["path"])}</code>'
            f'<span class="src {where}">{where}</span></div>'
            f'<div class="ds">{esc(r["description"])[:280]}</div>'
            f'{f"<div class=ds><a href={link} target=_blank rel=noopener>open upstream &rsaquo;</a></div>" if link else ""}'
            f'</div>')
    cb_html.append(
        f'<section class="cat" data-tab="cookbook"><h2>{esc(cat)} '
        f'<span class="ct">{len(items)}</span></h2>'
        f'<div class="grid">{"".join(cards)}</div></section>')

nc, pc, cc, tot, gen = (atlas["native_count"], atlas["powerup_count"],
                        atlas["cookbook_count"], atlas["total"], atlas["generated"])
cb_commit = (cookbooks[0]["commit"] if cookbooks else "")

doc = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>OmegaOS Skill Atlas</title>
<style>
:root{{--bg:#faf9f6;--card:#fff;--ink:#191919;--soft:#6b6b6b;--hair:#e6e3dc;--accent:#7c5cff;--lime:#2f9d63;--codebg:#f0eef7}}
@media(prefers-color-scheme:dark){{:root{{--bg:#100f0d;--card:#1a1917;--ink:#f3f0ea;--soft:#a39e94;--hair:#2b2925;--accent:#a48bff;--lime:#5fd39a;--codebg:#221f2e}}}}
:root[data-theme=dark]{{--bg:#100f0d;--card:#1a1917;--ink:#f3f0ea;--soft:#a39e94;--hair:#2b2925;--accent:#a48bff;--lime:#5fd39a;--codebg:#221f2e}}
:root[data-theme=light]{{--bg:#faf9f6;--card:#fff;--ink:#191919;--soft:#6b6b6b;--hair:#e6e3dc;--accent:#7c5cff;--lime:#2f9d63;--codebg:#f0eef7}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font:15px/1.5 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}}
.wrap{{max-width:1200px;margin:0 auto;padding:34px 22px 90px}}
h1{{font-size:30px;margin:0 0 4px;font-weight:680}}.sub{{color:var(--soft);margin:0 0 20px}}
.stats{{display:flex;gap:12px;flex-wrap:wrap;margin:0 0 20px}}
.stat{{background:var(--card);border:1px solid var(--hair);border-radius:12px;padding:12px 18px;min-width:110px}}
.stat b{{display:block;font-size:24px}}.stat span{{color:var(--soft);font-size:12.5px}}
.tabs{{display:flex;gap:8px;margin:0 0 16px}}
.tab{{padding:9px 16px;border-radius:9px;border:1px solid var(--hair);background:var(--card);color:var(--ink);cursor:pointer;font-size:14px;font-weight:600}}
.tab.on{{background:var(--accent);color:#fff;border-color:var(--accent)}}
#q{{width:100%;padding:13px 16px;border-radius:10px;border:1px solid var(--hair);background:var(--card);color:var(--ink);font-size:15px;margin:0 0 22px}}
#q:focus{{outline:none;border-color:var(--accent)}}
.cat{{margin:0 0 26px}}h2{{font-size:16px;border-bottom:1px solid var(--hair);padding-bottom:7px;display:flex;align-items:center;gap:10px;font-weight:650}}
.ct{{margin-left:auto;color:var(--soft);font-size:13px;font-weight:500}}
.src{{font-size:10.5px;font-weight:700;padding:2px 8px;border-radius:20px;text-transform:uppercase;letter-spacing:.04em}}
.src.bundle{{background:color-mix(in srgb,var(--accent) 16%,transparent);color:var(--accent)}}
.src.power-up{{background:color-mix(in srgb,var(--lime) 20%,transparent);color:var(--lime)}}
.src.local{{background:color-mix(in srgb,var(--lime) 20%,transparent);color:var(--lime)}}
.src.upstream{{background:color-mix(in srgb,var(--soft) 22%,transparent);color:var(--soft)}}
.card a{{color:var(--accent);text-decoration:none;font-weight:600}}
.card a:hover{{text-decoration:underline}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(290px,1fr));gap:12px;margin-top:13px}}
.card{{background:var(--card);border:1px solid var(--hair);border-radius:12px;padding:13px 15px}}
.nm{{font-weight:680;font-size:14px;margin-bottom:6px;word-break:break-word}}
.cmds{{display:flex;flex-wrap:wrap;gap:5px;margin-bottom:7px}}
code.cmd{{background:var(--codebg);color:var(--accent);font:12px/1.3 ui-monospace,SFMono-Regular,Menlo,monospace;padding:2px 8px;border-radius:6px;font-weight:600}}
.ds{{color:var(--soft);font-size:12.7px;line-height:1.45}}
.hidden{{display:none!important}}
.foot{{margin-top:38px;color:var(--soft);font-size:12.5px;border-top:1px solid var(--hair);padding-top:16px}}
.foot code{{background:var(--codebg);padding:1px 6px;border-radius:5px}}
</style></head><body><div class="wrap">
<h1>OmegaOS Skill Atlas</h1>
<p class="sub">Every skill in the system and how to run it &middot; generated {gen}</p>
<div class="stats">
<div class="stat"><b>{tot}</b><span>total skills</span></div>
<div class="stat"><b>{nc}</b><span>OmegaOS native</span></div>
<div class="stat"><b>{pc}</b><span>Power-Up library</span></div>
<div class="stat"><b>{cc}</b><span>Anthropic cookbooks</span></div>
</div>
<div class="tabs">
<button class="tab on" data-t="native">OmegaOS native &middot; {nc}</button>
<button class="tab" data-t="powerup">Power-Up library &middot; {pc}</button>
<button class="tab" data-t="cookbook">Anthropic cookbooks &middot; {cc}</button>
</div>
<input id="q" placeholder="Search skills, commands, descriptions…" autocomplete="off">
<div id="native-wrap">{''.join(native_html)}</div>
<div id="powerup-wrap" class="hidden">{''.join(pu_html)}</div>
<div id="cookbook-wrap" class="hidden">{''.join(cb_html)}</div>
<div class="foot">
<b>Run a native skill:</b> type its command (e.g. <code>/uiuxaudit</code>, <code>/higgsfield-generate</code>) or ask for it by name; Claude invokes it via the Skill tool. Most also answer to <code>/omg-&lt;name&gt;</code>.<br>
<b>CLI:</b> <code>omega-skills</code> lists all &middot; <code>omega-skills &lt;term&gt;</code> searches &middot; <code>omega-skills --powerups &lt;term&gt;</code> searches the library.<br>
<b>Power-Up library</b> (paid, private): activate one by copying its folder from <code>~/.omega/skills-library/youraipowerup/</code> into <code>~/.claude/skills/</code>, or upload a <code>.plugin</code> via Claude &rsaquo; Customize &rsaquo; Plugins.<br>
<b>Anthropic cookbooks</b> (MIT, pinned @ <code>{cb_commit}</code>): Anthropic's own reference recipes. Find one with <code>omega-skills --rag "&lt;your need&gt;"</code> or <code>/cookbook</code>; install the notebooks locally with <code>tools/cookbooks/install-cookbooks.sh</code>.
</div></div>
<script>
const q=document.getElementById('q'),nw=document.getElementById('native-wrap'),pw=document.getElementById('powerup-wrap'),cw=document.getElementById('cookbook-wrap');
document.querySelectorAll('.tab').forEach(b=>b.addEventListener('click',()=>{{
 document.querySelectorAll('.tab').forEach(x=>x.classList.remove('on'));b.classList.add('on');
 const t=b.dataset.t;nw.classList.toggle('hidden',t!=='native');pw.classList.toggle('hidden',t!=='powerup');cw.classList.toggle('hidden',t!=='cookbook');filt();
}}));
function filt(){{const t=q.value.trim().toLowerCase();
 document.querySelectorAll('.card').forEach(c=>{{c.classList.toggle('hidden',!!t&&!c.dataset.s.includes(t))}});
 document.querySelectorAll('.cat').forEach(s=>{{if(s.offsetParent===null&&!t)return;const any=[...s.querySelectorAll('.card')].some(c=>!c.classList.contains('hidden'));s.style.display=any?'':'none'}});
}}
q.addEventListener('input',filt);
</script></body></html>"""

os.makedirs(os.path.join(OMEGA, "artifacts"), exist_ok=True)
html_path = os.path.join(OMEGA, "artifacts/omega-skill-atlas.html")
html_tmp = html_path + ".tmp"
with open(html_tmp, "w", encoding="utf-8") as handle:
    handle.write(doc)
os.replace(html_tmp, html_path)
print(f"native={nc} powerup={pc} cookbook={cc} total={tot}")
print("wrote ~/.omega/skills-atlas.json + ~/.omega/artifacts/omega-skill-atlas.html")
