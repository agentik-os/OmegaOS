# Release Gate

Eighteen items. Every one must be `YES`. There is no partial pass, no "mostly",
and no item that can be answered by intending to fix it later.

The gate is deliberately separate from the rubric. The rubric asks *how good is
this*, and answers with a number that a determined builder can argue upward. The
gate asks *is this thing true*, and the answer is yes or no. A package can score
4.4 on the rubric and still fail the gate on one missing changelog entry, and
that is the correct outcome.

## How each item is answered

Three kinds of check, and mixing them up is how a gate becomes theatre:

- **MECH** a command produces the answer. Cite the command and its exit code.
- **JUDGED** a reader decides, and writes the sentence that decided it.
- **BOTH** a command establishes the floor, a reader establishes the rest.

| # | Item | Kind | How it is answered |
|---|---|---|---|
| 1 | bounded capability | JUDGED | `OS.md` section 2 names owns, does not own, hands off to, consumes from, and the near neighbour it is confused with |
| 2 | specific value proposition | JUDGED | the promise fails if you swap in another capability's name |
| 3 | primary artifact defined | BOTH | `OS.md` section 5 names it; `EXAMPLES/` shows one finished |
| 4 | human skill defined | JUDGED | `SKILL.md` teaches the capability without the model, and carries a proficiency ladder |
| 5 | evidence states defined | JUDGED | the six states appear in the operating logic, not only in a glossary |
| 6 | decision gates defined | JUDGED | each gate names its criterion, its branches and who can overrule it |
| 7 | stop conditions defined | JUDGED | at least one path exists where the correct output is a refusal, and it is tested |
| 8 | handoffs defined | MECH | `python3 OS/_tools/graph.py --strict` exits 0 and `verify.py` reports no `DEPS` failure |
| 9 | appropriate security controls | BOTH | MECH: no credential pattern in any package file. JUDGED: controls match the declared sensitivity class and the domain table |
| 10 | tests beyond happy path | BOTH | MECH: `EVALS/TEST-PLAN.md` covers all seven families. JUDGED: name the change that would turn a test red |
| 11 | substantive files | MECH | `validate_os.py <path> --full` reports no `AUTHORED` failure and no `SUBSTANCE` failure |
| 12 | realistic examples | JUDGED | `EXAMPLES/` runs a real situation end to end, anonymised, opening move to finished artifact |
| 13 | purposeful references | JUDGED | every reference carries a filled `WHERE USED`; unused references are deleted, not kept |
| 14 | no unsupported major claims | JUDGED | every ROI figure, guarantee, benchmark or "production ready" traces to evidence, or is removed |
| 15 | quality threshold passed | MECH | `python3 TOOLS/score_os.py scorecard.json` exits 0 |
| 16 | package validated | MECH | `python3 TOOLS/validate_os.py <path> --full` exits 0 |
| 17 | changelog updated | MECH | `CHANGELOG.md` carries an entry for the exact version in `manifest.json` |
| 18 | reproducible ZIP | MECH | `python3 TOOLS/create_zip.py <path>` run twice yields the identical SHA-256 |

## The mechanical run, in one block

```bash
SLUG=<slug>; CAND=<path/to/candidate>

python3 OS/os-builder-os/TOOLS/validate_os.py "$CAND" --full   ; echo "16,11 -> $?"
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json      ; echo "15 -> $?"
python3 OS/_tools/graph.py --strict                            ; echo "8  -> $?"
python3 -c "import json,sys,pathlib;m=json.load(open(pathlib.Path('$CAND','manifest.json')));\
c=pathlib.Path('$CAND','CHANGELOG.md').read_text();\
sys.exit(0 if m['version'] in c else 1)"                       ; echo "17 -> $?"
A=$(python3 OS/os-builder-os/TOOLS/create_zip.py "$CAND" /tmp/a.zip --json | grep sha256)
B=$(python3 OS/os-builder-os/TOOLS/create_zip.py "$CAND" /tmp/b.zip --json | grep sha256)
[ "$A" = "$B" ]                                                ; echo "18 -> $?"
```

Every `echo` must print `0`. An exit code that is not zero is a `NO`, and a `NO`
is not a note for the release summary, it is a stop.

## The judged run

The eleven judged items are answered by a reader who did not write the package.
Self review passes items that a stranger fails, every time, and the items it
passes are exactly the ones that were hardest to write.

Each judged `YES` carries one sentence of evidence citing a file. A `YES` with
no sentence is recorded as `NO`, because an unjustified yes is indistinguishable
from an unread item.

## Recording the verdict

The gate result is recorded in the scorecard's `gate` object, one key per item,
each `YES`, `NO` or `NA`. `NA` is legal for exactly one case: item 9's domain
specific controls, when the capability touches none of the listed domains. Every
other `NA` is a `NO` that someone did not want to write down.

The released package states its gate verdict and its scorecard together. A
quality claim published without both is the unsupported major claim that item 14
exists to block, made by the builder about their own work.

## What happens on a NO

1. The build returns to phase 12 (Repair). It does not proceed to release with
   an exception, and it does not proceed with a plan to fix it after release.
2. The repair changes the package. Re-running the gate against an unchanged
   package is not a repair.
3. On a `NO` against a CRITICAL rubric dimension (evidence discipline, operating
   logic, artifact quality, security, testability), the repair also adds the
   test that would have caught it. A defect fixed without a regression test is a
   defect scheduled to return.
4. If three repair cycles do not clear the same item, stop and escalate to the
   operator with what was tried. Grinding a fourth cycle on an unchanged
   diagnosis is how a build burns its budget and ships anyway.
