#!/usr/bin/env python3
"""Normalise every manifest's `dependencies` onto ONE canonical schema.

Three different shapes were authored across the fleet, because the brief pinned
the field names but not the element type:

  A  strings that are EVENT names      "wealth.networth.updated"
  B  objects                            {"os": "x-os", "event": "a.b.c"}
                                        {"artifact": "...", "to": ["x-os"]}
  C  strings that are OS SLUGS          "health-energy-os"

All three carry real information. None is wrong on its own terms; they are just
different types, and the Runtime cannot resolve three schemas.

CANONICAL SCHEMA (lossless superset of all three):

  requires      [slug]                  hard dependency: must be installed
  consumes      [event]                 events this OS listens for
  emits         [event]                 events this OS publishes
  consumes_from [slug]                  OSes it takes input from
  emits_to      [slug]                  OSes its output reaches
  handoffs      [{to: slug, artifact}]  named artifact handoffs, no event

Rules: nothing is invented, nothing is dropped. A slug found in `consumes`
moves to `consumes_from` (it was always a "who", never a "what"). An object's
`event` goes to consumes/emits and its `os`/`to` to consumes_from/emits_to.
An `artifact` object becomes a handoff. Idempotent.

Usage:
    python3 normalize.py --check          report what would change
    python3 normalize.py --write          apply
    python3 normalize.py --write <slug>   apply to some units only
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REG = json.load(open(os.path.join(OS_DIR, "_registry.json"), encoding="utf-8"))
SLUGS = {o["slug"] for o in REG["os"]}
EVENT_RE = re.compile(r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$")

CANON = ["requires", "consumes", "emits", "consumes_from", "emits_to", "handoffs"]

# Retired directory names that agents still referenced, because the legacy dirs
# are on disk and readable. Each maps to the canonical unit that replaced it.
# The registry's own alias table, plus the three units that were SPLIT (where
# the legacy name resolves to the half that kept the primary concern).
ALIAS_RESOLVE = dict(REG.get("aliases") or {})
ALIAS_RESOLVE.update({
    "quality-evaluation-release-os": "quality-evaluation-os",
    "operations-automation-os": "operations-os",
    "wealth-capital-os": "wealth-os",
    "seductive-os": "social-intelligence-os",
    "books-os": "librarian-os",
    "relationship-network-os": "network-os",
    "delivery-customer-success-os": "delivery-cs-os",
})


def resolve(slug):
    """Map a retired directory name onto its canonical unit."""
    return ALIAS_RESOLVE.get(slug, slug)


def normalize(deps):
    """Return (canonical_deps, list_of_moves). Pure, no I/O."""
    out = {k: [] for k in CANON}
    moves = []

    def add(key, val):
        if key in ("requires", "consumes_from", "emits_to") and isinstance(val, str):
            r = resolve(val)
            if r != val:
                moves.append(f"{key}: retired alias {val!r} -> {r!r}")
                val = r
        if val and val not in out[key]:
            out[key].append(val)

    # requires stays as slugs
    for d in deps.get("requires") or []:
        if isinstance(d, str) and d in SLUGS:
            add("requires", d)
        elif isinstance(d, str):
            moves.append(f"requires {d!r} is not a known slug, kept as is")
            add("requires", d)

    # already-canonical side channels carry through
    for key in ("consumes_from", "emits_to"):
        for d in deps.get(key) or []:
            if isinstance(d, str):
                add(key, d)
    for h in deps.get("handoffs") or []:
        if isinstance(h, dict) and h not in out["handoffs"]:
            out["handoffs"].append(h)

    for kind, who_key in (("consumes", "consumes_from"), ("emits", "emits_to")):
        for e in deps.get(kind) or []:
            if isinstance(e, str):
                if e in SLUGS or e.endswith("-os"):
                    add(who_key, e)
                    moves.append(f"{kind}: slug {e!r} -> {who_key}")
                elif EVENT_RE.match(e):
                    add(kind, e)
                else:
                    moves.append(f"{kind}: {e!r} is neither slug nor event, kept in {kind}")
                    add(kind, e)
            elif isinstance(e, dict):
                ev = e.get("event")
                if isinstance(ev, str) and EVENT_RE.match(ev):
                    add(kind, ev)
                art = e.get("artifact")
                targets = []
                if isinstance(e.get("os"), str):
                    targets.append(e["os"])
                for t in e.get("to") or []:
                    if isinstance(t, str):
                        targets.append(t)
                for t in targets:
                    add(who_key, t)
                if art and not ev:
                    for t in targets or [None]:
                        h = {"to": resolve(t), "artifact": art} if t else {"artifact": art}
                        if h not in out["handoffs"]:
                            out["handoffs"].append(h)
                    moves.append(f"{kind}: artifact object -> handoffs")
                elif ev:
                    moves.append(f"{kind}: object -> event {ev!r} + {who_key}")
    # drop empty keys so a manifest stays readable
    return {k: v for k, v in out.items() if v}, moves


def main():
    write = "--write" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    units = [o for o in REG["os"] if not args or o["slug"] in args]

    changed = total_moves = 0
    for u in units:
        p = os.path.join(OS_DIR, u["slug"], "manifest.json")
        if not os.path.isfile(p):
            continue
        try:
            man = json.load(open(p, encoding="utf-8"))
        except Exception as e:
            print(f"  {u['slug']:<28} INVALID JSON, skipped: {e}")
            continue
        deps = man.get("dependencies") or {}
        new, moves = normalize(deps)
        if new == deps:
            continue
        changed += 1
        total_moves += len(moves)
        print(f"  {u['slug']:<28} {len(moves)} change(s)")
        for m in moves[:4]:
            print(f"       {m}")
        if len(moves) > 4:
            print(f"       ... and {len(moves) - 4} more")
        if write:
            man["dependencies"] = new
            with open(p, "w", encoding="utf-8") as fh:
                json.dump(man, fh, indent=2, ensure_ascii=False)
                fh.write("\n")

    print(f"\nunits needing normalisation : {changed}/{len(units)}")
    print(f"total field moves           : {total_moves}")
    print("APPLIED" if write else "(dry run, pass --write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
