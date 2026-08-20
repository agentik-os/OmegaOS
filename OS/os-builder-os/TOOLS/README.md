# OS Builder {OS}: Tools

Deterministic work is done by code. Judgment is done by the model. This
directory holds the code half, and the split is not stylistic: a threshold that
lives in prose gets rounded, a contract check that lives in prose gets skimmed,
and both failures show up as a package that shipped when it should not have.

Everything here runs with the Python 3 standard library. No install step, no
dependency, no network. A tool that needs a package the operator does not have
is a tool that silently stops being run.

## The tools

| Tool | Purpose | Permission needed | If unavailable |
|---|---|---|---|
| [`validate_os.py`](validate_os.py) | grade a candidate OS package against the live suite contract | read the candidate and `OS/_tools/` | fall back to `OS/_tools/verify.py` after registering, which is the wrong order and must be stated as such |
| [`score_os.py`](score_os.py) | apply the release threshold to a filled scorecard | read one JSON file | apply the threshold by hand and record who did the arithmetic |
| [`create_zip.py`](create_zip.py) | build a reproducible release archive with its SHA-256 | read the package, write one file | ship the directory and drop the "reproducible ZIP" gate item explicitly, never silently |
| [`schemas/`](schemas/) | the machine-readable shapes the above check against | read | validate by hand against `REFERENCES/` |

Two suite tools are used constantly and are **not** duplicated here, because a
second implementation drifts from the first and the drift is always discovered
in the wrong direction:

| Suite tool | Purpose |
|---|---|
| `OS/_tools/verify.py` | the authoritative grader for a REGISTERED unit |
| `OS/_tools/graph.py` | whether the suite's event graph actually joins up |
| `OS/_tools/normalize.py` | normalise `manifest.json` dependencies onto the canonical schema |
| `OS/_tools/suite.py` | the single source of truth for the registry |
| `OS/_tools/gen_readme.py`, `gen_os_products.py` | regenerate the derived human index and TUI roster |

## validate_os.py, and why it exists

`OS/_tools/verify.py` resolves every unit from `OS/_registry.json` and reads it
from `OS/<slug>/`. That is exactly right for a unit that is already registered,
and exactly wrong for the one thing OS Builder produces: a package that is not
registered yet. Registering first and grading second means a broken unit enters
the registry, the generators run over it, and the suite goes red before anyone
has looked at it.

`validate_os.py` inverts the order. It **imports** `verify` and calls
`verify.check()` against an arbitrary directory, so a candidate is graded with
the identical STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH and SUBSTANCE checks
the suite itself uses, before it is allowed anywhere near the registry.
Dependency slugs still resolve against the real registry, so a candidate cannot
declare a handoff to an OS that does not exist.

```bash
python3 OS/os-builder-os/TOOLS/validate_os.py <path>           # CORE tier, wave 1
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full    # all 23 contract files
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --json    # machine-readable, for a gate
python3 OS/os-builder-os/TOOLS/validate_os.py --registered <slug>   # defer to verify.py
```

Exit codes: `0` pass, `1` failures found, `2` usage or resolution error.

The upstream `scripts/validate_package.py` that shipped with the original OS
Builder material is **superseded** by this tool and is deliberately not carried
across. It checked six file names and a minimum byte count against a package
tree the suite no longer uses. Keeping it would mean two validators disagreeing
about what a valid OS is, which is worse than either of them alone.

## score_os.py

The sixteen rubric dimensions are judged, so the tool does not attempt to score
them. What it does is refuse to let the arithmetic be fudged.

```bash
python3 OS/os-builder-os/TOOLS/score_os.py --template > scorecard.json
# fill every dimension with a score AND its evidence
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json --json
```

It enforces four things: all sixteen dimensions present as integers 0 to 5, each
carrying written evidence; every dimension at 4 or above; the five critical
dimensions at 4 or above with no waiver possible; and the mean at 4.3 or above.
A missing score is `MALFORMED`, not a zero, because treating an unfilled field
as a zero lets an incomplete review masquerade as a harsh one.

Exit codes: `0` RELEASE, `1` BLOCKED, `2` malformed.

Worked input: [`../EVALS/scorecard.example.json`](../EVALS/scorecard.example.json).

## create_zip.py

Reproducibility is a gate item, so the archive has to actually be reproducible.
The naive approach stamps each entry with its mtime and copies its permission
bits, so zipping unchanged content twice yields two different hashes and the
gate item becomes uncheckable.

This one sorts entries, fixes every timestamp, fixes every permission to 0644,
excludes build noise and `outputs/`, and prints the SHA-256. Two runs over
unchanged content produce the same hash. If they do not, something changed, and
noticing that is the whole point.

```bash
python3 OS/os-builder-os/TOOLS/create_zip.py <path/to/os-dir>
python3 OS/os-builder-os/TOOLS/create_zip.py <path/to/os-dir> --list
python3 OS/os-builder-os/TOOLS/create_zip.py <path/to/os-dir> out.zip --json
```

## schemas/

The shapes live beside the tool that checks them, not in `REFERENCES/`, because
a schema is a contract rather than knowledge and it must not drift from its
validator.

| Schema | Shapes | Written by |
|---|---|---|
| [`schemas/os_spec.schema.json`](schemas/os_spec.schema.json) | the normalised intake and specification of a candidate | phase 0 and 1 |
| [`schemas/evidence.schema.json`](schemas/evidence.schema.json) | one evidence item with its state and provenance | phase 3 onward |
| [`schemas/scorecard.schema.json`](schemas/scorecard.schema.json) | the filled sixteen dimension rubric | phase 11 |

The narrative standards behind them are in
[`../REFERENCES/`](../REFERENCES/README.md): the schema says what shape is legal,
the reference says what makes it honest.

## Tool selection, for a generated OS

The same discipline applies to any OS this one builds. Reach for the lightest
mechanism that carries the semantics:

| The work is | The right mechanism |
|---|---|
| a deterministic calculation | a script or a calculator |
| strict interchange between systems | a schema |
| a document produced repeatedly | a template |
| adaptive questioning or synthesis | an LLM prompt |
| a judgment with consequences | a human skill plus an approval gate |
| repeated quality assurance | tests plus a rubric |
| movement across a system boundary | a handoff contract |
| a one off explanation | nothing, do not overbuild |

The ladder, in order, and you stop at the first rung that works:
`DO NOTHING -> REMOVE -> SIMPLIFY -> STANDARD SOFTWARE -> DETERMINISTIC
AUTOMATION -> AI ASSIST -> AGENT -> MULTI AGENT`.

An unnecessary agent is one of the ten adversarial test cases, and it is the one
builders fail most often, because reaching for the top rung feels like ambition
rather than what it is: cost, latency, variance and a silent failure mode bought
in exchange for nothing.

## The release ordering

```bash
# grade unregistered
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full
# score, with evidence
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json
# register only after both pass
python3 OS/_tools/suite.py check && python3 OS/_tools/suite.py registry
python3 OS/_tools/normalize.py --check
python3 OS/_tools/graph.py --strict
python3 OS/_tools/gen_readme.py
python3 OS/_tools/gen_os_products.py --check
# grade again, now registered
python3 OS/_tools/verify.py <slug> --full
# archive
python3 OS/os-builder-os/TOOLS/create_zip.py OS/<slug>
```
