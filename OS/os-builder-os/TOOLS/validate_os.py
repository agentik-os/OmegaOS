#!/usr/bin/env python3
"""Grade a CANDIDATE OS package against the live AGENTIK {OS} contract.

Why this exists, and why it is not a second copy of `verify.py`:

`OS/_tools/verify.py` is the suite grader and it is authoritative. It resolves
every unit from `OS/_registry.json` and reads it from `OS/<slug>/`. That is
exactly right for a unit that has already been registered, and exactly wrong
for the one thing OS Builder produces: a package that is NOT registered yet.
Registering first and grading second means a broken unit enters the registry,
the generators run over it, and the suite goes red before anyone looks at it.

So this tool inverts the order. It points the SAME checks at an arbitrary
directory, so a candidate can be graded, repaired and only then registered.
It imports `verify` and calls `verify.check()`. It does not reimplement
STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH or SUBSTANCE, because a second
implementation would drift from the first and the drift would always be
discovered in the wrong direction.

Dependency slugs are still resolved against the REAL registry, so a candidate
cannot declare a handoff to an OS that does not exist.

Usage:
    validate_os.py <path/to/candidate>            grade at CORE tier (wave 1)
    validate_os.py <path/to/candidate> --full     grade all 23 contract files
    validate_os.py <path/to/candidate> --json     machine-readable, for a gate
    validate_os.py --registered <slug>            defer to verify.py as-is

Exit codes: 0 pass, 1 failures found, 2 usage or resolution error.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
# OS/os-builder-os/TOOLS -> OS/os-builder-os -> OS -> OS/_tools
OS_DIR = os.path.dirname(os.path.dirname(HERE))
TOOLS = os.path.join(OS_DIR, "_tools")

if not os.path.isdir(TOOLS):
    sys.exit(f"cannot find the suite tooling at {TOOLS}; run this from inside the repo")
sys.path.insert(0, TOOLS)

try:
    import verify  # the authoritative grader
except Exception as exc:  # pragma: no cover
    sys.exit(f"cannot import the suite grader from {TOOLS}: {exc}")


def grade_candidate(path, tier="core"):
    """Run the suite's own checks against an arbitrary directory.

    `verify.check()` reads `os.path.join(verify.OS_DIR, unit["slug"])`. We
    repoint that module global at the candidate's PARENT for the duration of
    the call, so the identical code path reads the candidate instead. The
    registry (and therefore slug resolution for DEPS) is untouched.
    """
    path = os.path.abspath(path.rstrip(os.sep))
    if not os.path.isdir(path):
        return None, [f"not a directory: {path}"]

    slug = os.path.basename(path)
    parent = os.path.dirname(path)

    manifest_path = os.path.join(path, "manifest.json")
    num = None
    if os.path.isfile(manifest_path):
        try:
            num = json.load(open(manifest_path, encoding="utf-8")).get("num")
        except (OSError, ValueError) as exc:
            return None, [f"MANIFEST unreadable: {exc}"]
    if num is None:
        # verify.check() compares manifest num against the registry entry. A
        # candidate has no registry entry, so we take its own claim and only
        # check internal consistency.
        num = -1

    unit = {"slug": slug, "num": num}

    original = verify.OS_DIR
    try:
        verify.OS_DIR = parent
        fails, stats = verify.check(unit, tier=tier)
    finally:
        verify.OS_DIR = original

    # A candidate is allowed to be absent from the registry; that is the whole
    # point. Any other failure stands.
    fails = [f for f in fails if "!= registry" not in f]
    return stats, fails


def main():
    argv = [a for a in sys.argv[1:] if not a.startswith("--")]
    tier = "full" if "--full" in sys.argv else "core"
    as_json = "--json" in sys.argv

    if "--registered" in sys.argv:
        if not argv:
            print("--registered needs a slug", file=sys.stderr)
            return 2
        os.execv(sys.executable,
                 [sys.executable, os.path.join(TOOLS, "verify.py")] + argv
                 + (["--full"] if tier == "full" else []))

    if len(argv) != 1:
        print(__doc__.strip().split("Usage:")[1].strip(), file=sys.stderr)
        return 2

    stats, fails = grade_candidate(argv[0], tier=tier)
    slug = os.path.basename(os.path.abspath(argv[0].rstrip(os.sep)))

    if as_json:
        print(json.dumps({
            "slug": slug,
            "tier": tier,
            "passed": not fails,
            "failures": fails,
            "stats": {k: (sorted(v) if isinstance(v, set) else v)
                      for k, v in (stats or {}).items()},
        }, indent=2))
        return 0 if not fails else 1

    print(f"candidate : {slug}")
    print(f"tier      : {tier.upper()} "
          f"({'all 23 contract files' if tier == 'full' else 'the 5 defining files'})")
    if stats:
        print(f"present   : {stats['present']}/{stats['total']}")
        print(f"authored  : {stats['authored']}/{stats['total']}"
              f"   core {stats['core_authored']}/{stats['core_total']}")
    print()
    if not fails:
        print("PASS  ready to register in OS/_tools/suite.py, then run the generators")
        return 0
    print(f"FAIL  {len(fails)} problem(s)")
    for f in fails:
        print(f"        {f}")
    print()
    print("Fix these before registering. Registering first puts a broken unit")
    print("into the registry and turns the whole suite red.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
