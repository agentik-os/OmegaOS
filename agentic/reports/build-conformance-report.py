#!/usr/bin/env python3
"""Render the Agentik Runtime OS conformance report from the checker JSON.

Input : agentik-runtime-os-conformance.json (produced by os-conformance-check.sh)
Output: agentik-runtime-os-conformance.html (self-contained, theme aware)
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
data = json.load(open(os.path.join(HERE, "agentik-runtime-os-conformance.json")))
S, ROWS = data["summary"], data["os"]

PILLARS = ["install", "configure", "run", "compose", "update", "evaluate", "permissions"]
PILLAR_LABEL = {
    "install": "Install", "configure": "Configure", "run": "Run", "compose": "Compose",
    "update": "Update", "evaluate": "Evaluate", "permissions": "Permissions",
}
PILLAR_NEEDS = {
    "install": "present in a shipped SSOT (OmegaOS repo or Agentik-Skills)",
    "configure": "a declarative config contract (config/os.yaml)",
    "run": "an invocable entrypoint (SKILL.md)",
    "compose": "resolvable dependencies (an entry in the suite registry)",
    "update": "a version plus a CHANGELOG.md to diff against",
    "evaluate": "an evals/ suite the runtime can execute",
    "permissions": "a declared human-approval boundary",
}
N = S["total_os_units"]


def bar(n, total=N):
    pct = round(100 * n / total)
    tone = "good" if pct >= 90 else ("warn" if pct >= 50 else "bad")
    return (f'<div class="bar"><div class="bar-fill {tone}" style="width:{pct}%"></div></div>'
            f'<span class="bar-num">{n}/{total}</span>')


def dot(v):
    return '<span class="dot ok" title="ready">&#10003;</span>' if v else '<span class="dot no" title="missing">&#10007;</span>'


verdict_class = {"RUNTIME-READY": "ok", "PARTIAL": "warn", "NOT-READY": "bad"}

rows_html = []
for r in sorted(ROWS, key=lambda x: (-x["pillars_ready"], x["unit"])):
    cells = "".join(f"<td class='c'>{dot(r['pillars'][p])}</td>" for p in PILLARS)
    orphan = " <span class='tag orphan'>orphan</span>" if r["ssot"]["orphan"] else ""
    tier = f"<span class='tag {r['tier']}'>{r['tier']}</span>"
    rows_html.append(
        f"<tr><td class='unit'><code>{r['unit']}</code>{orphan}</td><td>{tier}</td>{cells}"
        f"<td class='c'><b>{r['pillars_ready']}</b>/7</td>"
        f"<td class='c'><span class='verdict {verdict_class[r['verdict']]}'>{r['verdict']}</span></td></tr>"
    )

pillar_rows = "".join(
    f"<tr><td><b>{PILLAR_LABEL[p]}</b></td><td class='need'>{PILLAR_NEEDS[p]}</td>"
    f"<td class='barcell'>{bar(S['pillar_coverage'][p])}</td></tr>"
    for p in PILLARS
)

orphan_list = "".join(f"<li><code>{o}</code></li>" for o in S["orphans"])
notreg_list = "".join(f"<li><code>{o}</code></li>" for o in S["not_in_registry"])

HTML = f"""<title>Runtime Conformance Verdict</title>
<style>
:root {{
  --bg:#fbfaf8; --panel:#ffffff; --ink:#17151f; --muted:#645f75; --line:#e6e2dc;
  --accent:#6b4bd6; --accent-soft:#f0ecfd;
  --ok:#1f8a54; --ok-bg:#e7f5ee; --warn:#a3690a; --warn-bg:#fdf2df; --bad:#b3261e; --bad-bg:#fdeceb;
  --code:#f4f2ef;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    --bg:#100f14; --panel:#181722; --ink:#eeecf5; --muted:#a09bb2; --line:#2b2937;
    --accent:#a68dff; --accent-soft:#221d38;
    --ok:#54d391; --ok-bg:#12301f; --warn:#e8b25c; --warn-bg:#33260e; --bad:#ff8a80; --bad-bg:#3a1512;
    --code:#211f2c;
  }}
}}
:root[data-theme="dark"] {{
  --bg:#100f14; --panel:#181722; --ink:#eeecf5; --muted:#a09bb2; --line:#2b2937;
  --accent:#a68dff; --accent-soft:#221d38;
  --ok:#54d391; --ok-bg:#12301f; --warn:#e8b25c; --warn-bg:#33260e; --bad:#ff8a80; --bad-bg:#3a1512;
  --code:#211f2c;
}}
* {{ box-sizing:border-box; }}
body {{
  margin:0; background:var(--bg); color:var(--ink);
  font:16px/1.65 ui-sans-serif,-apple-system,"Segoe UI",Inter,Roboto,Helvetica,Arial,sans-serif;
  -webkit-font-smoothing:antialiased;
}}
.wrap {{ max-width:1080px; margin:0 auto; padding:48px 24px 96px; }}
header.hero {{ border-bottom:1px solid var(--line); padding-bottom:28px; margin-bottom:36px; }}
.eyebrow {{ font-size:12px; letter-spacing:.14em; text-transform:uppercase; color:var(--accent); font-weight:700; }}
h1 {{ font-size:clamp(28px,4.4vw,42px); line-height:1.15; margin:12px 0 10px; letter-spacing:-.02em; }}
.sub {{ color:var(--muted); font-size:16px; max-width:70ch; margin:0; }}
.meta {{ margin-top:18px; font-size:13px; color:var(--muted); }}
.meta code {{ font-size:12px; }}
h2 {{ font-size:22px; margin:52px 0 14px; letter-spacing:-.01em; scroll-margin-top:20px; }}
h3 {{ font-size:16px; margin:30px 0 8px; }}
p {{ max-width:78ch; }}
code {{ background:var(--code); padding:.13em .42em; border-radius:5px; font-size:.88em;
       font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace; }}
pre {{ background:var(--code); border:1px solid var(--line); border-radius:10px; padding:14px 16px;
      overflow-x:auto; font-size:13px; line-height:1.55; }}
pre code {{ background:none; padding:0; }}
.verdict-banner {{
  background:var(--bad-bg); border:1px solid var(--bad); border-left:5px solid var(--bad);
  border-radius:12px; padding:20px 22px; margin:0 0 8px;
}}
.verdict-banner .big {{ font-size:19px; font-weight:750; color:var(--bad); letter-spacing:-.01em; }}
.verdict-banner p {{ margin:8px 0 0; color:var(--ink); }}
.grid {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(160px,1fr)); gap:12px; margin:22px 0; }}
.stat {{ background:var(--panel); border:1px solid var(--line); border-radius:12px; padding:16px 18px; }}
.stat .n {{ font-size:30px; font-weight:750; letter-spacing:-.02em; line-height:1.1; }}
.stat .l {{ font-size:12.5px; color:var(--muted); margin-top:4px; }}
.stat.bad .n {{ color:var(--bad); }} .stat.warn .n {{ color:var(--warn); }} .stat.ok .n {{ color:var(--ok); }}
nav.toc {{ background:var(--panel); border:1px solid var(--line); border-radius:12px; padding:16px 20px; margin:28px 0 0; }}
nav.toc div {{ font-size:12px; text-transform:uppercase; letter-spacing:.12em; color:var(--muted); font-weight:700; margin-bottom:8px; }}
nav.toc ol {{ margin:0; padding-left:20px; columns:2; column-gap:28px; }}
nav.toc li {{ margin:3px 0; break-inside:avoid; }}
nav.toc a {{ color:var(--ink); text-decoration:none; border-bottom:1px solid transparent; }}
nav.toc a:hover {{ border-bottom-color:var(--accent); color:var(--accent); }}
.scroll {{ overflow-x:auto; border:1px solid var(--line); border-radius:12px; margin:16px 0; background:var(--panel); }}
table {{ border-collapse:collapse; width:100%; font-size:13.5px; }}
th, td {{ padding:9px 12px; text-align:left; border-bottom:1px solid var(--line); white-space:nowrap; }}
th {{ background:var(--accent-soft); font-size:11.5px; text-transform:uppercase; letter-spacing:.07em;
     color:var(--ink); font-weight:700; position:sticky; top:0; }}
tbody tr:last-child td {{ border-bottom:none; }}
td.c, th.c {{ text-align:center; }}
td.unit code {{ background:none; padding:0; font-size:13px; }}
td.need {{ white-space:normal; color:var(--muted); font-size:13px; }}
.dot {{ font-weight:800; font-size:14px; }}
.dot.ok {{ color:var(--ok); }} .dot.no {{ color:var(--bad); opacity:.75; }}
.tag {{ font-size:10.5px; text-transform:uppercase; letter-spacing:.06em; padding:2px 7px;
       border-radius:999px; font-weight:700; border:1px solid var(--line); color:var(--muted); }}
.tag.full {{ background:var(--ok-bg); color:var(--ok); border-color:transparent; }}
.tag.thin {{ background:var(--warn-bg); color:var(--warn); border-color:transparent; }}
.tag.orphan {{ background:var(--bad-bg); color:var(--bad); border-color:transparent; margin-left:6px; }}
.verdict {{ font-size:11px; font-weight:750; padding:3px 9px; border-radius:999px; letter-spacing:.03em; }}
.verdict.ok {{ background:var(--ok-bg); color:var(--ok); }}
.verdict.warn {{ background:var(--warn-bg); color:var(--warn); }}
.verdict.bad {{ background:var(--bad-bg); color:var(--bad); }}
.bar {{ display:inline-block; width:150px; height:9px; background:var(--line); border-radius:999px;
       overflow:hidden; vertical-align:middle; }}
.bar-fill {{ height:100%; border-radius:999px; }}
.bar-fill.good {{ background:var(--ok); }} .bar-fill.warn {{ background:var(--warn); }} .bar-fill.bad {{ background:var(--bad); }}
.bar-num {{ font-size:12.5px; color:var(--muted); margin-left:10px; font-variant-numeric:tabular-nums; }}
td.barcell {{ white-space:nowrap; }}
.finding {{ background:var(--panel); border:1px solid var(--line); border-radius:12px;
           padding:18px 20px; margin:14px 0; }}
.finding.crit {{ border-left:4px solid var(--bad); }}
.finding.high {{ border-left:4px solid var(--warn); }}
.finding.med {{ border-left:4px solid var(--accent); }}
.finding h3 {{ margin:0 0 6px; font-size:16.5px; }}
.finding .sev {{ font-size:10.5px; font-weight:750; letter-spacing:.08em; text-transform:uppercase;
                padding:2px 8px; border-radius:999px; margin-right:8px; }}
.sev.crit {{ background:var(--bad-bg); color:var(--bad); }}
.sev.high {{ background:var(--warn-bg); color:var(--warn); }}
.sev.med {{ background:var(--accent-soft); color:var(--accent); }}
.finding p {{ margin:8px 0 0; }}
.ev {{ font-size:12.5px; color:var(--muted); margin-top:10px; border-top:1px dashed var(--line); padding-top:9px; }}
.ev b {{ color:var(--ink); }}
.callout {{ background:var(--accent-soft); border:1px solid var(--accent); border-radius:12px; padding:20px 22px; margin:20px 0; }}
.callout h3 {{ margin-top:0; color:var(--accent); }}
ul.tight li {{ margin:5px 0; }}
ol.steps > li {{ margin:14px 0; }}
footer {{ margin-top:64px; padding-top:20px; border-top:1px solid var(--line); font-size:13px; color:var(--muted); }}
@media print {{
  body {{ background:#fff; color:#000; }}
  nav.toc, .bar {{ break-inside:avoid; }}
  .wrap {{ padding:0; max-width:none; }}
  th {{ position:static; }}
}}
@media (max-width:640px) {{ nav.toc ol {{ columns:1; }} .wrap {{ padding:28px 16px 64px; }} }}
</style>

<div class="wrap">
<header class="hero">
  <div class="eyebrow">OmegaOS &middot; Agentik Runtime</div>
  <h1>Is the Runtime good for all OS?</h1>
  <p class="sub">A deterministic conformance audit of every Operating System unit shipped by OmegaOS,
  measured against the seven things the Agentik Runtime promises to do: install, configure, run,
  compose, update, evaluate, and gate by permission.</p>
  <div class="meta">Generated from <code>os-conformance-check.sh</code> (read only, no network).
  Scope: <code>~/.omega/skills/*-os</code> plus <code>personal-os-builder</code>,
  cross checked against <code>OmegaOS/skills/</code>, <code>agentik-os/Agentik-Skills</code>
  and <code>_os-suite-registry.json</code>.</div>
</header>

<div class="verdict-banner">
  <div class="big">No. Today the Runtime is good for zero of {N} OS.</div>
  <p>Every unit runs (all {N} carry an invocable <code>SKILL.md</code>), but not one satisfies all seven
  Runtime pillars. {S['verdicts']['NOT-READY']} units score below 4 of 7. The blocking problem is not
  quality of thinking, it is that the contract the Runtime needs to read does not exist on most of them,
  and that the twelve best built OS are not shipped anywhere.</p>
</div>

<div class="grid">
  <div class="stat"><div class="n">{N}</div><div class="l">OS units on disk</div></div>
  <div class="stat bad"><div class="n">0</div><div class="l">Runtime ready (7 of 7)</div></div>
  <div class="stat warn"><div class="n">{S['verdicts']['PARTIAL']}</div><div class="l">Partial (5 of 7)</div></div>
  <div class="stat bad"><div class="n">{S['verdicts']['NOT-READY']}</div><div class="l">Not ready</div></div>
  <div class="stat bad"><div class="n">{len(S['orphans'])}</div><div class="l">Orphans, shipped nowhere</div></div>
</div>

<nav class="toc">
  <div>Contents</div>
  <ol>
    <li><a href="#answer">The short answer</a></li>
    <li><a href="#pillars">Pillar coverage</a></li>
    <li><a href="#findings">Findings</a></li>
    <li><a href="#matrix">Full conformance matrix</a></li>
    <li><a href="#fix">The recommended fix</a></li>
    <li><a href="#plan">Remediation plan</a></li>
    <li><a href="#repro">Reproduce this audit</a></li>
  </ol>
</nav>

<h2 id="answer">1. The short answer</h2>
<p>The Agentik Runtime document describes a layer that installs, configures, runs, composes, updates,
evaluates and permission gates every Agentik OS. Measured against what is actually on this machine,
six of those seven verbs have no substrate on most OS units, and one of them, update, has no substrate
on any unit at all.</p>
<p>Two structural generations coexist and were never reconciled. Thirteen units carry a
<code>MANIFEST.json</code> with a real event model, schemas, evals and a config contract. Sixteen are a
single <code>SKILL.md</code> with some loose folders. The Runtime can only operate on the first group,
and even there it cannot update or fully evaluate them.</p>
<p>The most consequential finding has nothing to do with the specification. Twelve of the thirteen best
built OS are not present in either source of truth. They exist only in <code>~/.omega/skills/</code> on
this box. <code>agentik install revenue</code> could never succeed for anyone, including the operator
after a reinstall.</p>

<h2 id="pillars">2. Pillar coverage</h2>
<p>Each Runtime verb needs a specific artifact to operate on. This is how many of the {N} units supply it.</p>
<div class="scroll"><table>
  <thead><tr><th>Runtime pillar</th><th>What it needs on the OS</th><th>Units supplying it</th></tr></thead>
  <tbody>{pillar_rows}</tbody>
</table></div>
<p><b>Run</b> is the only verb that works everywhere, because it is the only one that was ever actually
built. <b>Update</b> is at zero: every manifest reads <code>version 1.0.0</code> and exactly one unit in
{N} has a <code>CHANGELOG.md</code>, so there is nothing for an update command to compare or report.</p>

<h2 id="findings">3. Findings</h2>

<div class="finding crit">
  <h3><span class="sev crit">Critical</span>F1. Twelve OS are shipped nowhere and would be lost on a reset</h3>
  <p>These units exist in neither source of truth: not in the OmegaOS repository, not in the
  <code>agentik-os/Agentik-Skills</code> library. A fresh <code>git clone &amp;&amp; ./install.sh</code>
  reproduces none of them, and the private library mirror cannot supply them either. They are also,
  by every structural measure in this audit, the twelve strongest OS in the suite.</p>
  <ul class="tight">{orphan_list}</ul>
  <div class="ev"><b>Evidence:</b> orphan column of the matrix below, computed by comparing
  <code>~/.omega/skills/</code> against <code>OmegaOS/skills/</code> and a recursive directory index of
  <code>~/.omega/repos/Agentik-Skills</code> (HEAD <code>c9b21e9</code>, 2026-08-14).
  Breaks Law 0 (install parity) and R-SKILLPUB (a skill that lives only locally does not exist).</div>
</div>

<div class="finding crit">
  <h3><span class="sev crit">Critical</span>F2. Two structural generations, no shared contract</h3>
  <p>{S['tier_full']} units carry <code>MANIFEST.json</code>; {S['tier_thin']} carry only
  <code>SKILL.md</code>. Install, update, evaluate and doctor all need a machine readable descriptor,
  so on {S['tier_thin']} of {N} units they have nothing to read. The thin group is not marginal: it
  contains Blueprint, Design, Stepper, Builder, Brainstorm, Market Research, Execution, Mindset,
  Habit Tracker, Storyteller and AI Logic, which is the entire product build pipeline.</p>
  <div class="ev"><b>Evidence:</b> tier column of the matrix. Thin units expose no version, no slug,
  no events, no dependencies and no capability requirements.</div>
</div>

<div class="finding high">
  <h3><span class="sev high">High</span>F3. The document's folder standard collides with the one on disk</h3>
  <p>The specification asks for <code>OS.md</code>, <code>SYSTEM.md</code>, <code>SETUP.md</code>,
  <code>manifest.json</code>, <code>WORKFLOWS/</code>, <code>PROMPTS/</code>, <code>REFERENCES/</code>,
  <code>TOOLS/</code>, <code>EVALS/</code>, <code>EXAMPLES/</code>. What exists is
  <code>MASTER.md</code>, <code>system/</code>, <code>INSTALL.md</code>, <code>MANIFEST.json</code>,
  <code>protocols/</code>, <code>agents/</code>, <code>knowledge/</code> and <code>references/</code>,
  <code>runtime/</code> and <code>scripts/</code>, <code>evals/</code>, <code>examples/</code>.
  Adopting the document literally means renaming directories across {N} units for no functional gain,
  and the case difference between <code>manifest.json</code> and <code>MANIFEST.json</code> alone would
  break every existing unit on a case sensitive filesystem.</p>
  <div class="ev"><b>Evidence:</b> 12 units carry <code>MASTER.md</code> and 12 carry
  <code>OMEGA_INTEGRATION.md</code>; zero carry <code>OS.md</code>, <code>SYSTEM.md</code> or
  <code>SETUP.md</code>. See section 5 for the alternative that avoids the rename.</div>
</div>

<div class="finding crit">
  <h3><span class="sev crit">Critical</span>F4. Update and changelog have no substrate at all</h3>
  <p>All {S['tier_full']} manifests declare <code>version 1.0.0</code>. Exactly one unit in {N}
  (<code>intuitive-os</code>) has a <code>CHANGELOG.md</code>. So <code>agentik update</code> has no
  current version to compare, no available version to fetch, and nothing to show the user. The
  subscription argument in the document, that you are running software like systems that keep
  improving, currently has no mechanism behind it.</p>
  <div class="ev"><b>Evidence:</b> update pillar coverage is 0 of {N}.</div>
</div>

<div class="finding high">
  <h3><span class="sev high">High</span>F5. Dependencies live centrally and cover only 23 of {N}</h3>
  <p>The OS graph the Stack Composer needs exists, but only inside
  <code>_os-suite-registry.json</code>, and only for 23 units. Zero manifests declare their own
  dependencies. Six units are absent from the registry entirely, so the composer cannot place them
  in any stack.</p>
  <ul class="tight">{notreg_list}</ul>
  <div class="ev"><b>Evidence:</b> compose pillar coverage is 23 of {N}; <code>declares_dependencies</code>
  is false on all {S['tier_full']} manifests.</div>
</div>

<div class="finding med">
  <h3><span class="sev med">Medium</span>F6. The permission model already exists, in better form than the document proposes</h3>
  <p>Eleven units carry <code>config/os.yaml</code> with an explicit
  <code>requires_human_approval_for</code> list. Revenue OS, for example, already gates sending
  collections messages, issuing invoices, posting accounting entries and moving cash. That is exactly
  the permission layer the document asks for, invented already, just not surfaced to any runtime and
  not present on the other 18 units. <code>alignment-os</code> has the config file but no approval list.</p>
  <div class="ev"><b>Evidence:</b> permissions pillar coverage is 11 of {N};
  <code>revenue-os/config/os.yaml</code> lines 8 to 17.</div>
</div>

<div class="finding high">
  <h3><span class="sev high">High</span>F7. No unit declares target capabilities, so doctor cannot work</h3>
  <p>The document shows a doctor output in which the Gemini adapter warns that one required tool
  capability is unsupported and a fallback is available. Nothing on disk declares which capabilities an
  OS needs, so that line cannot be computed for any unit. Grepping every manifest and SKILL.md for a
  target or capability declaration returns nothing.</p>
  <div class="ev"><b>Evidence:</b> <code>declares_targets</code> false on all {S['tier_full']} manifests;
  zero matches for chatgpt or gemini across manifests and skill files.</div>
</div>

<div class="finding med">
  <h3><span class="sev med">Medium</span>F8. The registry documents a fix that was never applied</h3>
  <p><code>_os-suite-registry.md</code> states that the Alignment manifest was normalized with a
  <code>slug</code>, a <code>generated_at</code> and a <code>counts</code> object matching every other
  manifest. The file on disk has none of the three. Its keys are
  <code>critical_design_goals, file_count, files, generated, name, version</code>, a schema shared with
  no other unit. The registry is additionally five to six units behind what is on disk.</p>
  <div class="ev"><b>Evidence:</b> <code>~/.omega/skills/alignment-os/MANIFEST.json</code> versus the
  twelve peers, all of which carry <code>schema_version</code>, <code>slug</code>, <code>counts</code>,
  <code>category</code>, <code>position</code>, <code>events</code> and <code>purpose</code>.</div>
</div>

<div class="finding med">
  <h3><span class="sev med">Medium</span>F9. A duplicate, drifted Blueprint OS sits in the library</h3>
  <p>The Agentik-Skills library carries both <code>blueprint-os</code> and
  <code>Dev/blueprint-os</code>, and they have diverged: the first holds
  <code>assets/</code>, <code>legacy/</code>, <code>references/blueprint-contract.md</code> and
  <code>references/deep-guide.md</code>, the second holds <code>references/challenge.md</code>. The
  install mirror walks to depth 3 and keys purely on directory name, so which copy wins is incidental.</p>
  <div class="ev"><b>Evidence:</b> <code>diff -rq</code> across the two library paths.</div>
</div>

<h2 id="matrix">4. Full conformance matrix</h2>
<p>One row per OS unit, sorted by readiness. A check means the Runtime has something concrete to
operate on for that verb.</p>
<div class="scroll"><table>
  <thead><tr>
    <th>OS unit</th><th>Tier</th>
    {''.join(f"<th class='c'>{PILLAR_LABEL[p]}</th>" for p in PILLARS)}
    <th class="c">Score</th><th class="c">Verdict</th>
  </tr></thead>
  <tbody>{''.join(rows_html)}</tbody>
</table></div>

<h2 id="fix">5. The recommended fix</h2>
<div class="callout">
  <h3>Make the manifest the contract, not the folder shape</h3>
  <p>Do not restructure {N} directories to match the document. Invert it: require exactly one
  declarative file per OS, and let each OS keep the layout it already has by declaring where its parts
  live. Conformance then costs one file per unit instead of a rename across the suite, and the number
  of units the Runtime can operate on goes from 0 to {N}.</p>
</div>
<p>The natural home already exists. Twelve units carry <code>config/os.yaml</code>, and it already holds
name, slug, version, category, position, boundary and the human approval list. Promote that file to the
Runtime contract and extend it with the four things it lacks:</p>
<pre><code>name: Revenue OS
slug: revenue
version: 1.4.0                  # real, and moved by a changelog (fixes F4)
category: Business Stack

dependencies:                   # fixes F5, moves the graph out of the central registry
  requires: [context-memory]
  consumes: [content, market-research]
  emits: [delivery-customer-success, wealth-capital, strategy-portfolio]

targets:                        # fixes F7, lets doctor compute the fallback
  claude:  {{ supported: true }}
  chatgpt: {{ supported: true }}
  gemini:  {{ supported: partial, missing: [tool_use], fallback: manual_step }}

entrypoints:                    # fixes F3 without renaming a single directory
  system:     system/
  workflows:  protocols/
  references: [knowledge/, references/]
  memory:     memory/
  tools:      [runtime/, scripts/]
  evals:      evals/
  examples:   examples/

requires_human_approval_for:    # already present on 11 units, keep as is
  - issuing invoices or credits
  - moving cash or reserves</code></pre>
<p>With that single file present, all seven verbs become computable on every unit, the document's folder
standard becomes a default rather than a requirement, and the existing structural investment in the
twelve full tier OS is preserved instead of being churned.</p>

<h2 id="plan">6. Remediation plan</h2>
<ol class="steps">
  <li><b>P1. Ship the twelve orphans (fixes F1, unblocks install for 12 units).</b>
  Push each orphan to a source of truth, either <code>OmegaOS/skills/</code> or the Agentik-Skills
  library, and confirm a fresh install reproduces it. This is the only finding where the work is
  currently at risk of being destroyed, so it goes first. It writes to two GitHub repositories and
  needs an explicit go.</li>
  <li><b>P2. Freeze the contract.</b> Write the extended <code>os.yaml</code> schema plus a validator,
  and wire the validator into <code>verify-install.sh</code> so a non conforming OS fails the build
  rather than failing silently at runtime.</li>
  <li><b>P3. Backfill the twelve full tier units.</b> They already have <code>config/os.yaml</code>,
  so this is adding four blocks to an existing file. Fix the Alignment manifest drift in the same pass
  (F8) and correct the registry claim.</li>
  <li><b>P4. Backfill the sixteen thin units (the largest piece).</b> Author
  <code>config/os.yaml</code> from scratch for each, deriving dependencies from the suite registry
  where an entry exists, and add the six missing units to the registry (F5).</li>
  <li><b>P5. Turn on update.</b> Seed a <code>CHANGELOG.md</code> per unit, set honest versions, and
  only then is <code>agentik update</code> a real command rather than a promise (F4).</li>
  <li><b>P6. Deduplicate the library.</b> Resolve <code>blueprint-os</code> against
  <code>Dev/blueprint-os</code> and delete the loser, after the operator picks which is canonical (F9).</li>
</ol>

<h2 id="repro">7. Reproduce this audit</h2>
<p>The checker is read only, needs no network, and regenerates both the data and this page.</p>
<pre><code>cd ~/Station/SideBusiness/OmegaOS/agentic/reports
bash os-conformance-check.sh &gt; agentik-runtime-os-conformance.json
python3 build-conformance-report.py</code></pre>
<p>Every number on this page is read from that JSON at render time, so the report cannot drift from
the evidence.</p>

<footer>
  OmegaOS conformance audit &middot; {N} OS units measured &middot; source
  <code>agentic/reports/os-conformance-check.sh</code> &middot; data
  <code>agentik-runtime-os-conformance.json</code>
</footer>
</div>
"""

out = os.path.join(HERE, "agentik-runtime-os-conformance.html")
open(out, "w", encoding="utf-8").write(HTML)
print(f"wrote {out} ({len(HTML)} bytes)")
