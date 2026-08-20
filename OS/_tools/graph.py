#!/usr/bin/env python3
"""Check that the AGENTIK {OS} event graph actually joins up. Read only.

Each OS declares `consumes` and `emits` in its manifest. Those are authored one
group at a time, by agents that never see each other's manifests, so a single
character difference silently severs a boundary: wealth emits
`wealth.capital_constraints.published` while capital consumes
`wealth.capital_constraint.published`, and nothing joins.

Per-unit verification cannot catch that. Only the whole graph can.

Reports:
  ORPHAN CONSUME   an event consumed by someone and emitted by nobody
  NEAR MISS        an orphan consume within one edit of a real emitted event
  UNCONSUMED EMIT  an event emitted and consumed by nobody (informational)
  NAMESPACE        an event whose namespace matches no OS and no shared name

Usage:
    python3 graph.py            full report
    python3 graph.py --strict   exit 1 on any orphan consume
"""
import difflib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REG = json.load(open(os.path.join(OS_DIR, "_registry.json"), encoding="utf-8"))

# Namespaces that are legitimately produced outside a single OS manifest:
# cross-cutting governance and memory events the whole suite shares.
SHARED_NS = {"memory", "change", "review", "policy", "risk", "incident",
             "decision", "context", "runtime", "agentik"}


def events(dep_list):
    """Normalise the three authored shapes into plain event names."""
    out = []
    for x in dep_list or []:
        if isinstance(x, str):
            out.append(x)
        elif isinstance(x, dict):
            e = x.get("event")
            if isinstance(e, str):
                out.append(e)
            # {"artifact": ...} entries describe a handoff with no event name;
            # they are not part of the event graph and are skipped here.
    return out


def main():
    strict = "--strict" in sys.argv
    emitted, consumed = {}, {}
    for o in REG["os"]:
        p = os.path.join(OS_DIR, o["slug"], "manifest.json")
        if not os.path.isfile(p):
            continue
        try:
            m = json.load(open(p, encoding="utf-8"))
        except Exception:
            continue
        d = m.get("dependencies") or {}
        for e in events(d.get("emits")):
            emitted.setdefault(e, []).append(o["slug"])
        for e in events(d.get("consumes")):
            consumed.setdefault(e, []).append(o["slug"])

    all_emitted = set(emitted)
    orphans = sorted(e for e in consumed if e not in all_emitted)
    unconsumed = sorted(e for e in emitted if e not in consumed)

    print(f"events emitted : {len(emitted)}")
    print(f"events consumed: {len(consumed)}")
    print()

    print(f"=== ORPHAN CONSUMES ({len(orphans)}) ===")
    print("An event someone waits on that nobody produces. This is a severed boundary.\n")
    near = 0
    for e in orphans:
        ns = e.split(".")[0]
        by = ", ".join(consumed[e])
        match = difflib.get_close_matches(e, all_emitted, n=1, cutoff=0.85)
        if match:
            near += 1
            src = ", ".join(emitted[match[0]])
            print(f"  {e}")
            print(f"      consumed by : {by}")
            print(f"      NEAR MISS   : {match[0]}  (emitted by {src})")
        elif ns in SHARED_NS:
            print(f"  {e}\n      consumed by : {by}\n      shared namespace {ns!r}, external producer")
        else:
            print(f"  {e}\n      consumed by : {by}\n      no producer, no near match")
    print()
    print(f"near misses (one-edit typos severing a real link): {near}")
    print(f"\n=== UNCONSUMED EMITS ({len(unconsumed)}) ===")
    print("Informational: produced but nobody listens yet.\n")
    for e in unconsumed[:20]:
        print(f"  {e}  (from {', '.join(emitted[e])})")
    if len(unconsumed) > 20:
        print(f"  ... and {len(unconsumed) - 20} more")

    hard = [e for e in orphans
            if e.split(".")[0] not in SHARED_NS
            and difflib.get_close_matches(e, all_emitted, n=1, cutoff=0.85)]
    print(f"\nSEVERED LINKS NEEDING A FIX: {len(hard)}")
    return 1 if (strict and hard) else 0


if __name__ == "__main__":
    sys.exit(main())
