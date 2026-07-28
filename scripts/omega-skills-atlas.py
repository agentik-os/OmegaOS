#!/usr/bin/env python3
"""OmegaOS Skill Atlas: index every skill (native OmegaOS + Power-Up library) with
its invocation command(s), emit skills-atlas.json + a searchable HTML catalog."""
import os, re, json, html, datetime, glob

HOME = os.path.expanduser("~")
OMEGA = os.environ.get("OMEGA_DIR") or os.path.join(HOME, ".omega")
NATIVE = os.path.join(OMEGA, "skills")
POWERUP_MANIFEST = os.path.join(OMEGA, "skills-library/youraipowerup/MANIFEST.json")

def parse_frontmatter(path):
    """Return (name, description) handling single-line and folded (>|) scalars."""
    try:
        txt = open(path, encoding="utf-8", errors="replace").read()
    except Exception:
        return None, ""
    m = re.match(r"^---\s*\n(.*?)\n---", txt, re.S)
    if not m:
        return os.path.basename(os.path.dirname(path)), ""
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
        name = os.path.basename(os.path.dirname(path))
    return name, desc

def derive_commands(name, path):
    """Primary command = /<name> (Skill tool). Add /omg-<name> if that skill dir exists."""
    cmds = [f"/{name}"]
    if os.path.isdir(os.path.join(NATIVE, f"omg-{name}")) and f"omg-{name}" != name:
        cmds.append(f"/omg-{name}")
    return cmds

# ---- 1. native OmegaOS skills ----
native = []
for d in sorted(glob.glob(os.path.join(NATIVE, "*"))):
    if not os.path.isdir(d):
        continue
    slug = os.path.basename(d)
    sk = os.path.join(d, "SKILL.md")
    if slug == "audits" or not os.path.isfile(sk):
        continue
    name, desc = parse_frontmatter(sk)
    group = "Audits" if name.endswith("audit") or name.endswith("audits") else "Skills"
    native.append({"name": name, "slug": slug, "description": desc,
                   "commands": derive_commands(slug, sk), "group": group, "source": "omegaos"})

# audits subdir
for d in sorted(glob.glob(os.path.join(NATIVE, "audits", "*"))):
    if not os.path.isdir(d):
        continue
    sk = os.path.join(d, "SKILL.md")
    if not os.path.isfile(sk):
        continue
    slug = os.path.basename(d)
    name, desc = parse_frontmatter(sk)
    native.append({"name": name, "slug": slug, "description": desc,
                   "commands": [f"/{slug}", f"/omg-{slug}"], "group": "Audits", "source": "omegaos"})

# dedup native by name (installed dir may duplicate audits)
seen = set(); native_u = []
for r in native:
    if r["name"] in seen:
        continue
    seen.add(r["name"]); native_u.append(r)
native = native_u

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

atlas = {
    "generated": datetime.date.today().isoformat(),
    "native_count": len(native),
    "powerup_count": len(powerups),
    "total": len(native) + len(powerups),
    "native": native,
    "powerups": powerups,
}
json.dump(atlas, open(os.path.join(OMEGA, "skills-atlas.json"), "w"), indent=2, ensure_ascii=False)

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

native_html = []
for g in sorted(native_groups, key=lambda x: (x != "Skills", x)):
    items = sorted(native_groups[g], key=lambda r: r["name"].lower())
    native_html.append(f'<section class="cat" data-tab="native"><h2>{esc(g)} <span class="ct">{len(items)}</span></h2><div class="grid">{render_cards(items)}</div></section>')

pu_html = []
for (src, cat), items in sorted(pu_groups.items()):
    items = sorted(items, key=lambda r: r["name"].lower())
    badge = "bundle" if src == "bundle-501" else "power-up"
    pu_html.append(f'<section class="cat pu" data-tab="powerup"><h2>{esc(cat)} <span class="src {badge}">{esc(src)}</span> <span class="ct">{len(items)}</span></h2><div class="grid">{render_cards(items, show_cmd=False)}</div></section>')

nc, pc, tot, gen = atlas["native_count"], atlas["powerup_count"], atlas["total"], atlas["generated"]

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
</div>
<div class="tabs">
<button class="tab on" data-t="native">OmegaOS native &middot; {nc}</button>
<button class="tab" data-t="powerup">Power-Up library &middot; {pc}</button>
</div>
<input id="q" placeholder="Search skills, commands, descriptions…" autocomplete="off">
<div id="native-wrap">{''.join(native_html)}</div>
<div id="powerup-wrap" class="hidden">{''.join(pu_html)}</div>
<div class="foot">
<b>Run a native skill:</b> type its command (e.g. <code>/uiuxaudit</code>, <code>/higgsfield-generate</code>) or ask for it by name; Claude invokes it via the Skill tool. Most also answer to <code>/omg-&lt;name&gt;</code>.<br>
<b>CLI:</b> <code>omega-skills</code> lists all &middot; <code>omega-skills &lt;term&gt;</code> searches &middot; <code>omega-skills --powerups &lt;term&gt;</code> searches the library.<br>
<b>Power-Up library</b> (paid, private): activate one by copying its folder from <code>~/.omega/skills-library/youraipowerup/</code> into <code>~/.claude/skills/</code>, or upload a <code>.plugin</code> via Claude &rsaquo; Customize &rsaquo; Plugins.
</div></div>
<script>
const q=document.getElementById('q'),nw=document.getElementById('native-wrap'),pw=document.getElementById('powerup-wrap');
document.querySelectorAll('.tab').forEach(b=>b.addEventListener('click',()=>{{
 document.querySelectorAll('.tab').forEach(x=>x.classList.remove('on'));b.classList.add('on');
 const t=b.dataset.t;nw.classList.toggle('hidden',t!=='native');pw.classList.toggle('hidden',t!=='powerup');filt();
}}));
function filt(){{const t=q.value.trim().toLowerCase();
 document.querySelectorAll('.card').forEach(c=>{{c.classList.toggle('hidden',!!t&&!c.dataset.s.includes(t))}});
 document.querySelectorAll('.cat').forEach(s=>{{if(s.offsetParent===null&&!t)return;const any=[...s.querySelectorAll('.card')].some(c=>!c.classList.contains('hidden'));s.style.display=any?'':'none'}});
}}
q.addEventListener('input',filt);
</script></body></html>"""

os.makedirs(os.path.join(OMEGA, "artifacts"), exist_ok=True)
open(os.path.join(OMEGA, "artifacts/omega-skill-atlas.html"), "w", encoding="utf-8").write(doc)
print(f"native={nc} powerup={pc} total={tot}")
print("wrote ~/.omega/skills-atlas.json + ~/.omega/artifacts/omega-skill-atlas.html")
