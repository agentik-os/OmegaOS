#!/usr/bin/env python3
"""Materialise the AGENTIK {OS} file contract for every unit in the registry.

Reads OS/_registry.json and creates, under OS/<slug>/, exactly the structure
the operator specified. Additive and idempotent by construction:

  - it NEVER deletes anything
  - it NEVER overwrites a file that already exists

so running it over the 32 units that inherit real content is safe: it fills
the gaps in their structure and leaves every authored file untouched.

Generated files carry a marker line so a later pass can tell a scaffold
placeholder from authored content, and `status` reports exactly that.

Usage:
    python3 scaffold.py status        what exists vs what the contract needs
    python3 scaffold.py build         create every missing file and directory
    python3 scaffold.py build <slug>  only that unit
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REG = json.load(open(os.path.join(OS_DIR, "_registry.json"), encoding="utf-8"))
C = REG["contract"]
GROUPS = {g["key"]: g for g in REG["groups"]}

MARK = "<!-- agentik:scaffold -->"   # present = not yet authored


def is_scaffold(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return MARK in fh.read(4096)
    except Exception:
        return False


# ── content templates ─────────────────────────────────────────────────────

def readme(u, g):
    return f"""# {u['display']}

{MARK}

> {u['tagline']}

**Suite position:** `{u['num']:02d}` in **{g['label']}** ({g['purpose']}).

## What this OS is for

{u['name']} {{OS}} is one operative system of the AGENTIK {{OS}} suite. It is not a
prompt and not a document: it is a system with its own operating specification,
workflows, commands, memory rules, evaluations and adapters, installed and run
by the Agentik Runtime.

Use it when your problem is: {u['tagline'][:-1].lower()}.

## Start here

| You want to | Read |
|---|---|
| Understand what it does | this file |
| Understand how it operates | [`OS.md`](OS.md) |
| See the AI behaviour contract | [`SYSTEM.md`](SYSTEM.md) |
| See what it can do | [`SKILL.md`](SKILL.md) |
| Configure it for yourself | [`SETUP.md`](SETUP.md) |
| See every command | [`COMMANDS/`](COMMANDS/) |
| See it in use | [`EXAMPLES/`](EXAMPLES/) |

## Install and run

```bash
agentik install {u['slug']}          # install this OS
agentik configure {u['slug']}        # answer the minimum setup questions
agentik run {u['slug']}              # start using it
```

Inside OmegaOS it is also reachable from `omega menu`, **OS** tab, entry
`{u['num']:02d}. {u['name']} {{OS}}`.

## Structure

```
{u['slug']}/
├── README.md      this file, the human entry point
├── OS.md          the complete operating specification
├── SYSTEM.md      AI and system instructions
├── SKILL.md       capabilities and procedures
├── SETUP.md       initial configuration
├── manifest.json  machine-readable metadata
├── CHANGELOG.md   what changed between versions
├── WORKFLOWS/     repeatable processes
├── COMMANDS/      every command, explained
├── PROMPTS/       reusable prompt units
├── REFERENCES/    knowledge this OS needs
├── MEMORY/        what may be remembered, updated, forgotten
├── TOOLS/         external capabilities it may use
├── EVALS/         tests that prove it behaves correctly
├── EXAMPLES/      worked examples
├── INTERFACES/    chat, artifact, dashboard, generative UI
└── ADAPTERS/      ChatGPT, Claude, Gemini, Codex
```

## Status

Scaffolded against the AGENTIK {{OS}} contract. Sections still carrying the
scaffold marker are structure, not authored content.
"""


def os_md(u, g):
    return f"""# {u['display']}: Operating Specification

{MARK}

## 1. Purpose

{u['tagline']}

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** to be authored.
- **Does not own:** to be authored.
- **Hands off to:** to be authored.
- **Consumes from:** to be authored.

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| to be authored | | | |

## 4. Inputs

What this OS needs to know before it can be useful, and where each input
legitimately comes from.

## 5. Outputs

The artifacts it produces, each with an owner and a place to live.

## 6. State

What is canonical here, what is a projection of another OS's truth, what is a
cache and what is temporary. Canonical state routes through Context & Memory
{{OS}}.

## 7. Rules and invariants

Statements that must remain true no matter which mode is running.

## 8. Failure behaviour

What it does on missing input, contradiction, and refusal. Abstention is a
valid output; a confident guess is not.

## 9. Human approval boundary

Actions this OS must never take without an explicit human decision.

## 10. Completion criteria

How a user knows the OS did its job.
"""


def system_md(u, g):
    return f"""# {u['display']}: System Instructions

{MARK}

You are {u['display']}, one operative system of the AGENTIK {{OS}} suite.

## Role

{u['tagline']}

## Operating contract

1. Stay inside the boundary declared in `OS.md`. When a request belongs to
   another OS, name that OS and hand off rather than improvising its job.
2. Separate evidence from inference. Label what is observed, what is reported
   by the user, what you inferred and what you assumed.
3. Never fabricate a fact, a source, a number or a citation. Absence of
   information is a reportable state.
4. Ask only what you cannot derive. One question at a time, and only when a
   wrong answer would change the output.
5. Respect the human approval boundary in `OS.md` section 9 without exception.
6. Produce the artifact the mode promises, or say plainly why you cannot.

## Refusal and abstention

Abstain when the evidence does not support a conclusion. Say what is missing
and what would resolve it. An honest "not enough to say" outranks a confident
answer that is wrong.

## Memory

Read `MEMORY/policy.md` before writing anything durable. Session context is not
memory. Only confirmed, durable facts are persisted, and the user can inspect
and remove them.
"""


def skill_md(u, g):
    return f"""---
name: {u['slug']}
description: {u['tagline']} {u['display']}, unit {u['num']:02d} of the AGENTIK {{OS}} suite ({g['label']}). Use when the user asks about {u['name'].lower()} or invokes /{u['slug']}.
---

# {u['display']}

{MARK}

{u['tagline']}

## When to use this

To be authored: the concrete situations where this OS is the right tool, and
the near-neighbour OSes it is commonly confused with.

## Capabilities

To be authored: what it can actually do, one line each.

## Procedure

To be authored: the steps it follows, in order.

## Handoffs

To be authored: which OS receives its output, and what that OS expects.
"""


def setup_md(u, g):
    return f"""# {u['display']}: Setup

{MARK}

The Runtime asks for the minimum context needed to be useful now, not
everything this OS could ever use. You can deepen configuration later.

## Required

To be authored: the few inputs without which this OS cannot work.

## Optional

To be authored: what improves the output when supplied.

## Configure

```bash
agentik configure {u['slug']}
```

## Verify

```bash
agentik doctor {u['slug']}
```

Reports which required inputs are present, which adapters support this OS on
your current AI environment, and what falls back.
"""


def manifest(u, g):
    return json.dumps({
        "schema_version": "2.0.0",
        "id": u["slug"],
        "num": u["num"],
        "name": u["display"],
        "version": "0.1.0",
        "status": "scaffold",
        "group": {"key": u["group"], "label": g["label"], "purpose": g["purpose"]},
        "tagline": u["tagline"],
        "inherits_from": u["inherits_from"],
        "commands": [],
        "dependencies": {"requires": [], "consumes": [], "emits": []},
        "targets": {
            "claude": {"supported": True},
            "chatgpt": {"supported": True},
            "gemini": {"supported": True},
            "codex": {"supported": True},
        },
        "entrypoints": {
            "spec": "OS.md", "system": "SYSTEM.md", "skill": "SKILL.md",
            "setup": "SETUP.md", "workflows": "WORKFLOWS/",
            "commands": "COMMANDS/", "prompts": "PROMPTS/",
            "references": "REFERENCES/", "memory": "MEMORY/",
            "tools": "TOOLS/", "evals": "EVALS/", "examples": "EXAMPLES/",
            "interfaces": "INTERFACES/", "adapters": "ADAPTERS/",
        },
        "requires_human_approval_for": [],
    }, indent=2) + "\n"


def changelog(u, g):
    return f"""# Changelog: {u['display']}

{MARK}

All notable changes to this OS are recorded here. The Runtime reads this file
for `agentik update {u['slug']}`, so every version bump needs an entry.

## [0.1.0] (unreleased)

### Added
- Scaffolded against the AGENTIK {{OS}} file contract (unit {u['num']:02d},
  {g['label']}).
"""


def commands_readme(u, g):
    return f"""# {u['display']}: Commands

{MARK}

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install {u['slug']}` | Installs this OS into your environment | Once, first |
| `agentik configure {u['slug']}` | Collects the minimum context it needs | After install |
| `agentik run {u['slug']}` | Starts the OS | Every session |
| `agentik doctor {u['slug']}` | Checks config, adapters and dependencies | When something is off |
| `agentik update {u['slug']}` | Updates to the latest version | When a release lands |
| `agentik eval {u['slug']}` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| to be authored | | | |
"""


def memory_policy(u, g):
    return f"""# {u['display']}: Memory Policy

{MARK}

Not everything said deserves to become permanent context. This file declares
what this OS may remember, for how long, and what the user may remove.

| Tier | Example | Lifetime | User can delete |
|---|---|---|---|
| Temporary | a value used once in this answer | the turn | n/a |
| Session | what we are working on right now | the session | yes |
| Project | decisions scoped to one project | the project | yes |
| Preference | how the user wants things done | until changed | yes |
| Confirmed | a fact the user explicitly confirmed | durable | yes |
| Outcome | what happened and what it taught | durable | yes |

## Never stored

Credentials, secrets, and anything the user marks private. Canonical durable
state routes through Context & Memory {{OS}} rather than living here.

## Retrieval

Only what is relevant to the current mission is loaded. Loading everything is
the failure mode this policy exists to prevent.
"""


def evals_readme(u, g):
    return f"""# {u['display']}: Evaluations

{MARK}

A prompt can sound impressive and still be wrong. These evaluations decide
whether this OS actually behaves correctly.

Run them with:

```bash
agentik eval {u['slug']}
```

## Suites

| Suite | Asserts | Pass condition |
|---|---|---|
| system-integrity | every contract file present and parseable | all present |
| output-contract | outputs match the shapes `OS.md` promises | all match |
| boundary | out-of-scope requests hand off, never improvise | no boundary breach |
| abstention | insufficient evidence produces abstention | no fabrication |
| to be authored | | |
"""


def interface_md(u, g, kind):
    blurb = {
        "chat": "Conversation is the primary interface. This file defines the "
                "opening move, how the OS asks for missing context, and how it "
                "closes a mode.",
        "artifact": "A rendered, self-contained deliverable the user can keep, "
                    "share or print.",
        "dashboard": "A persistent view of this OS's state, for a user who "
                     "returns to it repeatedly.",
        "generative-ui": "Interface generated per situation rather than fixed, "
                         "for when the right control depends on the data.",
    }[kind]
    return f"""# {u['display']}: {kind.replace('-', ' ').title()} Interface

{MARK}

{blurb}

## Applies when

To be authored.

## Contract

To be authored: what this surface shows, what the user can do from it, and what
it must never do.

## Degradation

What this surface falls back to when the environment cannot render it.
"""


def adapter_md(u, g, target):
    label = {"chatgpt": "ChatGPT", "claude": "Claude",
             "gemini": "Gemini", "codex": "Codex"}[target]
    return f"""# {u['display']}: {label} Adapter

{MARK}

The operating logic in `OS.md` stays constant. This file records only how it is
implemented on {label}, and what {label} cannot do.

## Capabilities used

To be authored: projects, persistent memory, tools, code execution, agents,
file handling.

## Installation

To be authored: exactly where the files go for {label}.

## Unsupported capabilities

To be authored: what this environment lacks and what the OS falls back to.
Absence must be stated, never silently worked around.
"""


PLACEHOLDER = {
    "WORKFLOWS": ("README.md", lambda u, g: f"""# {u['display']}: Workflows

{MARK}

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| to be authored | | |
"""),
    "PROMPTS": ("README.md", lambda u, g: f"""# {u['display']}: Prompts

{MARK}

Reusable prompt units this OS composes. Each is a named unit with a stated
input contract and output shape, never a loose paragraph.
"""),
    "REFERENCES": ("README.md", lambda u, g: f"""# {u['display']}: References

{MARK}

Knowledge this OS needs to do its job: frameworks, checklists, domain facts.
Each reference states its source and how far it can be trusted.
"""),
    "TOOLS": ("README.md", lambda u, g: f"""# {u['display']}: Tools

{MARK}

External capabilities this OS may use, each with the permission it requires and
what happens when it is unavailable.

| Tool | Purpose | Permission | Fallback |
|---|---|---|---|
| to be authored | | | |
"""),
    "EXAMPLES": ("README.md", lambda u, g: f"""# {u['display']}: Examples

{MARK}

Worked examples showing this OS on a real situation, from opening move to
finished artifact.
"""),
}


def build_unit(u, refresh=False):
    """Create missing contract files.

    With refresh=True, ALSO rewrite files that still carry the scaffold marker
    (template fixes propagate). A file without the marker has been authored by
    a human or an agent and is NEVER touched, in either mode.
    """
    g = GROUPS[u["group"]]
    root = os.path.join(OS_DIR, u["slug"])
    created = []

    def put(rel, content):
        path = os.path.join(root, rel)
        if os.path.exists(path):
            if not (refresh and is_scaffold(path)):
                return
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(content)
        created.append(rel)

    os.makedirs(root, exist_ok=True)
    put("README.md", readme(u, g))
    put("OS.md", os_md(u, g))
    put("SYSTEM.md", system_md(u, g))
    put("SKILL.md", skill_md(u, g))
    put("SETUP.md", setup_md(u, g))
    put("manifest.json", manifest(u, g))
    put("CHANGELOG.md", changelog(u, g))
    put("COMMANDS/README.md", commands_readme(u, g))
    put("MEMORY/policy.md", memory_policy(u, g))
    put("EVALS/README.md", evals_readme(u, g))
    for d, (fname, fn) in PLACEHOLDER.items():
        put(f"{d}/{fname}", fn(u, g))
    for i in C["INTERFACES"]:
        put(f"INTERFACES/{i}", interface_md(u, g, i[:-3]))
    for a in C["ADAPTERS"]:
        put(f"ADAPTERS/{a}", adapter_md(u, g, a[:-3]))
    return created


def audit_unit(u):
    root = os.path.join(OS_DIR, u["slug"])
    need = list(C["files"]) + \
        [f"COMMANDS/README.md", "MEMORY/policy.md", "EVALS/README.md"] + \
        [f"{d}/README.md" for d in PLACEHOLDER] + \
        [f"INTERFACES/{i}" for i in C["INTERFACES"]] + \
        [f"ADAPTERS/{a}" for a in C["ADAPTERS"]]
    present = [f for f in need if os.path.isfile(os.path.join(root, f))]
    authored = [f for f in present if not is_scaffold(os.path.join(root, f))]
    return len(need), len(present), len(authored)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    cmd = args[0] if args else "status"
    only = args[1] if len(args) > 1 else None
    units = [u for u in REG["os"] if only is None or u["slug"] == only]
    if only and not units:
        print(f"unknown slug {only!r}", file=sys.stderr)
        return 2

    if cmd == "build":
        refresh = "--refresh" in sys.argv
        total = 0
        for u in units:
            made = build_unit(u, refresh=refresh)
            total += len(made)
            if made:
                print(f"  {u['slug']:<28} {'~' if refresh else '+'}{len(made)} files")
        note = ("authored files untouched, nothing deleted" if refresh
                else "no file overwritten, nothing deleted")
        print(f"\n{'refreshed' if refresh else 'created'} {total} files across "
              f"{len(units)} units ({note})")
        return 0

    if cmd == "status":
        print(f"{'unit':<28} {'have':>5} {'/need':>6} {'authored':>9}")
        print("-" * 52)
        tn = tp = ta = 0
        for u in units:
            need, present, authored = audit_unit(u)
            tn += need; tp += present; ta += authored
            flag = "" if present == need else "  INCOMPLETE"
            print(f"{u['slug']:<28} {present:>5} {'/'+str(need):>6} {authored:>9}{flag}")
        print("-" * 52)
        print(f"{'TOTAL':<28} {tp:>5} {'/'+str(tn):>6} {ta:>9}")
        print(f"\nstructural completeness: {100*tp//tn if tn else 0}%")
        print(f"authored (non-scaffold) : {100*ta//tn if tn else 0}%")
        return 0

    print(f"unknown command {cmd!r}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
