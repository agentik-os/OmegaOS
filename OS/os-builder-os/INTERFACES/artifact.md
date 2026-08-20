# OS Builder {OS}: Artifact Interface

A rendered, self contained deliverable the operator keeps, shares or prints. OS
Builder produces five of them, and none is a transcript of the conversation that
made it.

The package itself is the primary artifact and it is a directory, not a
document. The four artifacts below are the **views** onto that package that a
person actually reads: the specification before it is built, the score after it
is built, the gate verdict that decides release, and the diff that shows what a
repair changed.

## The five artifacts

| Artifact | Rendered when | Read by | Lives at |
|---|---|---|---|
| OS Spec | end of phase 1, revised through phase 7 | the operator approving the boundary | the candidate's `OS.md`, plus a rendered view |
| Scorecard | phase 11, once per version | whoever decides whether to trust it | `scorecard.json` plus a rendered view |
| Gate verdict | phase 14 | the operator, and the suite | recorded in the scorecard's `gate` object |
| Repair diff | phase 12, once per cycle | the reviewer who scored it | rendered per cycle, not persisted |
| The package | phase 8 onward | everyone downstream | `OS/<slug>/`, and a reproducible ZIP |

## 1. The OS Spec view

The most important artifact in the build, because it is the last cheap place to
be wrong. It renders the normalised intake plus the boundary, in the shape of
[`../TOOLS/schemas/os_spec.schema.json`](../TOOLS/schemas/os_spec.schema.json).

Renders, in this order:

1. **The one line capability**, and immediately under it the primary artifact.
2. **The boundary as four columns**: owns, does not own, hands off to, consumes
   from. Rendered side by side rather than as four lists, because the reader is
   checking them against each other.
3. **The near neighbour**, named, with the sentence that distinguishes them.
4. **The mode table**: name, entry condition, produces, done when. One row per
   mode, and a mode with an empty completion test renders in an error state
   rather than blank.
5. **The approval boundary and the stop conditions**, together, because they are
   the same question asked from two sides.
6. **Non scope**, given equal visual weight to scope. A spec view that renders
   scope large and non scope small teaches the reader to skip the half that
   matters.
7. **The decision**: `build_os`, `split_capability`, `use_lighter_artifact`,
   `extend_existing_os` or `refuse`, with its reason. Rendered even when it is
   `build_os`, so a later reader knows the question was asked.

## 2. The scorecard view

Sixteen dimensions with their scores, evidence and threshold status. The
rendering rules exist because a scorecard is the artifact most vulnerable to
being read as better than it is.

- **Every score renders next to its evidence sentence.** A number with its
  justification hidden behind a click is a number that gets skimmed and
  believed. If the evidence does not fit, the evidence is too long, not the
  layout too small.
- **The five CRITICAL dimensions are marked as such** wherever they appear, and
  never sorted away from the others. Sorting by score puts the failures at the
  bottom, which is exactly where nobody reads.
- **The average renders with its margin**, not alone: `4.3125, threshold 4.3,
  margin 0.0125`. An average of 4.31 and an average of 4.8 are not the same
  result and must not render the same.
- **A waiver renders in the row it waives**, with its approver and reason
  inline. A waiver in a footnote is a waiver nobody sees.
- **`BLOCKED` renders before the scores, not after.** The verdict is the answer;
  the sixteen rows are the working.

## 3. The gate verdict view

Eighteen rows, each `YES`, `NO` or `NA`, each with its kind (`MECH`, `JUDGED`,
`BOTH`) and its evidence. Mechanical rows carry the command and its exit code.
Judged rows carry the sentence and the name of the reader who wrote it.

One `NO` renders the whole verdict as `BLOCKED`, at the top, before the table.
There is no partial pass to render, so there is no partial pass to show.

## 4. The repair diff view

Rendered once per repair cycle, and it answers one question: did the change move
the score, and was it the change we thought it was.

| Column | Content |
|---|---|
| dimension | which dimension the repair targeted |
| before | the score and its evidence sentence |
| change | the files touched, and what changed in them |
| after | the new score and its new evidence sentence |
| moved | yes, no, or "not re-scored" |

A repair whose score did not move is rendered prominently rather than quietly.
It is the more informative outcome, because it means the diagnosis was wrong and
the next cycle should attack something else.

A re-score with **no files changed** renders as an error, not as a result. That
is the mechanism by which a scorecard gets talked upward, and the artifact
surface is where it has to be caught.

## 5. The package view

A tree, annotated per file with three facts: present, authored, and graded.

```
example-candidate-os/
  OS.md              authored   CORE   10/10 sections
  SKILL.md           authored   CORE
  manifest.json      authored   CORE   13/13 keys, 3 handoffs resolve
  COMMANDS/README.md authored   CORE
  WORKFLOWS/README.md scaffold  CORE   BLOCKS: AUTHORED
  ...
  CORE 4/5   FULL 19/23
```

The tree renders the two tiers separately, matching `verify.py`. A unit at
`CORE 5/5, FULL 12/23` is a usable OS mid build, and rendering that as "12/23,
red" would be a false report of failing work.

## Rendering contract

- **Self contained.** One file, inline styles, no external asset, opens offline.
  An artifact that needs a network to render is not an artifact the operator
  keeps.
- **Print sane.** These get printed and pasted into reviews. Tables do not
  scroll off, and the verdict survives a black and white printer, which means
  status is never carried by colour alone.
- **The answer first.** Verdict, then margin, then working. A reader who stops
  after fifteen seconds must leave with the correct conclusion.
- **Its own limitations stated.** Every artifact ends with what it does not
  cover: which gate items were `NA`, which assumptions are still open, which
  dimensions were judged by the builder rather than by an independent reader.

## What is never rendered

- A quality claim with no scorecard behind it.
- A score without its evidence.
- A gate `YES` without the command or the sentence that produced it.
- Real names, real figures or real organisations from a requester's domain.
  Every rendered example is anonymised at intake, per
  [`../REFERENCES/SECURITY.md`](../REFERENCES/SECURITY.md).

## Degradation

Where no rendering surface exists, every artifact degrades to plain markdown
with the same section order and the same ordering rule: verdict first, working
second. The scorecard degrades to the table `score_os.py` already prints, which
is deliberately readable in a terminal. Nothing degrades to a summary paragraph:
a summary is where the margin, the waivers and the `NA` rows go to disappear.
