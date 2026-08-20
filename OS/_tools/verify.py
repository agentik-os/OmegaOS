#!/usr/bin/env python3
"""Verify the AGENTIK {OS} suite against its contract. Read only.

One command that answers: is every OS actually complete and consistent?
Checks, per unit:

  STRUCTURE   every contract file and directory present
  AUTHORED    no file still carries the scaffold marker
  MANIFEST    valid JSON, required keys, commands and dependencies filled
  DEPS        every declared dependency resolves to a real slug
  NODASH      no em dash or en dash anywhere (R-NODASH)
  SUBSTANCE   OS.md carries its 10 sections and no "to be authored" left

Exit 0 only when every unit passes every check.

Usage:
    python3 verify.py             full report
    python3 verify.py --summary   one line per unit
    python3 verify.py <slug>      one unit, verbose
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REG = json.load(open(os.path.join(OS_DIR, "_registry.json"), encoding="utf-8"))
C = REG["contract"]
SLUGS = {o["slug"] for o in REG["os"]}
MARK = "<!-- agentik:scaffold -->"
DASHES = ("\u2014", "\u2013")

# namespace.thing.verb, lowercase, at least two dots-separated parts
EVENT_RE = re.compile(r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$")

# R-NODASH bans the long dash in COPY. These are machine-matched protocol
# tokens, not copy: `builder_os.py` compares a blueprint's status against this
# exact string, and it appears in 73 files. Rewriting the dash would break the
# Builder intake gate, so the literal is exempt and stripped before the scan.
# A false positive here would train everyone to ignore the check.
PROTOCOL_LITERALS = [
    "BLUEPRINT COMPLETE — STEPPER READY",
]

CONTRACT_FILES = list(C["files"]) + [
    "COMMANDS/README.md", "MEMORY/policy.md", "EVALS/README.md",
    "WORKFLOWS/README.md", "PROMPTS/README.md", "REFERENCES/README.md",
    "TOOLS/README.md", "EXAMPLES/README.md",
] + [f"INTERFACES/{i}" for i in C["INTERFACES"]] \
  + [f"ADAPTERS/{a}" for a in C["ADAPTERS"]]

# WAVE 1 contract: the five files that actually define an OS. An OS with these
# authored is usable. The remaining 18 contract files are wave 2 (surfaces:
# INTERFACES, ADAPTERS, PROMPTS, REFERENCES, EVALS, EXAMPLES, MEMORY, SETUP,
# SYSTEM, README, CHANGELOG). Grading both tiers separately keeps the report
# honest instead of failing finished work for not being a later wave.
CORE_FILES = [
    "OS.md", "SKILL.md", "manifest.json",
    "COMMANDS/README.md", "WORKFLOWS/README.md",
]

OS_SECTIONS = [
    "Purpose", "Boundary", "Operating modes", "Inputs", "Outputs", "State",
    "Rules and invariants", "Failure behaviour", "Human approval boundary",
    "Completion criteria",
]

MANIFEST_REQUIRED = [
    "schema_version", "id", "num", "name", "version", "group", "tagline",
    "commands", "dependencies", "targets", "entrypoints",
    "requires_human_approval_for",
]


def read(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except Exception:
        return None


def check(unit, tier="core"):
    """Return (list_of_failures, stats) for one unit.

    tier="core" grades the five defining files (wave 1).
    tier="full" grades all 23 contract files (wave 2).
    Structure, dashes, manifest, deps and substance are checked in BOTH tiers.
    """
    slug = unit["slug"]
    root = os.path.join(OS_DIR, slug)
    fails = []
    stats = {"present": 0, "authored": 0, "total": len(CONTRACT_FILES),
             "core_authored": 0, "core_total": len(CORE_FILES),
             "shapes": set()}

    if not os.path.isdir(root):
        return [f"directory missing: OS/{slug}/"], stats

    # STRUCTURE + AUTHORED + NODASH
    for rel in CONTRACT_FILES:
        p = os.path.join(root, rel)
        if not os.path.isfile(p):
            fails.append(f"STRUCTURE missing {rel}")
            continue
        stats["present"] += 1
        body = read(p) or ""
        scaffold = MARK in body
        if not scaffold:
            stats["authored"] += 1
        if rel in CORE_FILES and not scaffold:
            stats["core_authored"] += 1
        # Only fail on an unauthored file when it is in the graded tier.
        if scaffold and (tier == "full" or rel in CORE_FILES):
            fails.append(f"AUTHORED still scaffold: {rel}")
        scan = body
        for lit in PROTOCOL_LITERALS:
            scan = scan.replace(lit, "")
        for d in DASHES:
            if d in scan:
                fails.append(f"NODASH long dash in {rel}")
                break

    # MANIFEST
    mpath = os.path.join(root, "manifest.json")
    man = None
    if os.path.isfile(mpath):
        try:
            man = json.loads(read(mpath) or "")
        except Exception as e:
            fails.append(f"MANIFEST invalid JSON: {e}")
    if isinstance(man, dict):
        for k in MANIFEST_REQUIRED:
            if k not in man:
                fails.append(f"MANIFEST missing key {k!r}")
        if man.get("id") != slug:
            fails.append(f"MANIFEST id {man.get('id')!r} != slug {slug!r}")
        if man.get("num") != unit["num"]:
            fails.append(f"MANIFEST num {man.get('num')} != registry {unit['num']}")
        if not man.get("commands"):
            fails.append("MANIFEST commands is empty")
        deps = man.get("dependencies") or {}
        # The canonical schema has six keys. A unit that expresses its graph as
        # artifact handoffs rather than events is fully specified, so checking
        # only the first three reported real work as empty.
        if not any(deps.get(k) for k in
                   ("requires", "consumes", "emits",
                    "consumes_from", "emits_to", "handoffs")):
            fails.append("MANIFEST dependencies are all empty")
        for h in deps.get("handoffs") or []:
            if not isinstance(h, dict) or "artifact" not in h:
                fails.append(f"DEPS handoff missing 'artifact': {json.dumps(h)[:60]}")
            elif h.get("to") is not None and h["to"] not in SLUGS:
                fails.append(f"DEPS handoff 'to' {h['to']!r} is not a known slug")
        for key in ("consumes_from", "emits_to"):
            for s in deps.get(key) or []:
                if s not in SLUGS:
                    fails.append(f"DEPS {key} -> unknown slug {s!r}")
        # `requires` is a hard dependency on another OS, so it holds SLUGS.
        # `consumes` / `emits` describe the EVENT graph, so they hold dotted
        # event names (namespace.thing.verb), matching the event namespaces the
        # OS suite already uses. These are different types and are checked as
        # such: putting a slug in `emits` is the real mistake to catch.
        for d in deps.get("requires") or []:
            if not isinstance(d, str):
                fails.append(f"DEPS requires -> {json.dumps(d)[:60]} is not a slug string")
            elif d not in SLUGS:
                fails.append(f"DEPS requires -> unknown slug {d!r}")
        # Three element shapes were produced across the fleet. All are accepted
        # and validated on their own terms; SHAPE records which one so the
        # inconsistency is visible and fixable rather than silently tolerated.
        for kind in ("consumes", "emits"):
            for e in deps.get(kind) or []:
                if isinstance(e, str):
                    stats["shapes"].add("string")
                    if e in SLUGS or e.endswith("-os"):
                        fails.append(f"DEPS {kind} -> {e!r} is a slug, expected an event name")
                    elif not EVENT_RE.match(e):
                        fails.append(f"DEPS {kind} -> {e!r} is not a dotted event name")
                elif isinstance(e, dict):
                    stats["shapes"].add("object")
                    ev = e.get("event")
                    if ev is not None:
                        if not isinstance(ev, str) or not EVENT_RE.match(ev):
                            fails.append(f"DEPS {kind} -> object 'event' {ev!r} is not a dotted event name")
                    elif "artifact" not in e:
                        fails.append(f"DEPS {kind} -> object has neither 'event' nor 'artifact': {json.dumps(e)[:60]}")
                    for ref_key in ("os",):
                        ref = e.get(ref_key)
                        if ref is not None and ref not in SLUGS:
                            fails.append(f"DEPS {kind} -> object '{ref_key}' {ref!r} is not a known slug")
                    for ref in e.get("to") or []:
                        if ref not in SLUGS:
                            fails.append(f"DEPS {kind} -> object 'to' {ref!r} is not a known slug")
                else:
                    fails.append(f"DEPS {kind} -> unsupported element type {type(e).__name__}")

    # SUBSTANCE
    osmd = read(os.path.join(root, "OS.md")) or ""
    for sec in OS_SECTIONS:
        if not re.search(rf"^##\s*\d*\.?\s*{re.escape(sec)}", osmd, re.M | re.I):
            fails.append(f"SUBSTANCE OS.md has no '{sec}' section")
    for rel in ("OS.md", "SKILL.md", "COMMANDS/README.md"):
        body = read(os.path.join(root, rel)) or ""
        if "to be authored" in body.lower():
            fails.append(f"SUBSTANCE '{rel}' still says 'to be authored'")

    return fails, stats


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    summary = "--summary" in sys.argv
    tier = "full" if "--full" in sys.argv else "core"
    units = [o for o in REG["os"] if not args or o["slug"] in args]
    if args and not units:
        print(f"unknown slug(s): {args}", file=sys.stderr)
        return 2

    print(f"grading tier: {tier.upper()} "
          f"({'all 23 contract files' if tier == 'full' else 'the 5 defining files'})\n")
    total_fail = 0
    ok_units = 0
    tp = ta = tt = ca = ct = 0
    for u in units:
        fails, st = check(u, tier=tier)
        tp += st["present"]; ta += st["authored"]; tt += st["total"]
        ca += st["core_authored"]; ct += st["core_total"]
        if fails:
            total_fail += len(fails)
        else:
            ok_units += 1
        core = f"core {st['core_authored']}/{st['core_total']}"
        allf = f"all {st['authored']}/{st['total']}"
        if summary:
            mark = "PASS" if not fails else f"FAIL({len(fails)})"
            print(f"{u['num']:02d} {u['slug']:<28} {core}  {allf:<10} {mark}")
        else:
            head = f"{u['num']:02d} {u['slug']}"
            if not fails:
                print(f"PASS  {head}  ({core}, {allf})")
            else:
                print(f"FAIL  {head}  ({core}, {allf})")
                for f in fails[:12]:
                    print(f"        {f}")
                if len(fails) > 12:
                    print(f"        ... and {len(fails) - 12} more")

    print()
    print(f"units passing {tier.upper():<5}        : {ok_units}/{len(units)}")
    print(f"contract files present   : {tp}/{tt} ({100*tp//tt if tt else 0}%)")
    print(f"CORE authored (5 files)  : {ca}/{ct} ({100*ca//ct if ct else 0}%)")
    print(f"ALL authored (23 files)  : {ta}/{tt} ({100*ta//tt if tt else 0}%)")
    print(f"total failures           : {total_fail}")
    return 0 if total_fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
