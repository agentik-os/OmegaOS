# OS Builder {OS}: Gemini Adapter

The operating logic in `OS.md` is constant. This file records only how it is
implemented on Gemini, and what Gemini cannot do.

Gemini is the **review** target. Its very large context window means an entire
candidate package, all twenty three files, fits in one prompt at once, and that
is a genuinely different capability from reading files one at a time. Whole
package review is where this target earns its place: cross file contradictions,
a boundary that section 2 draws and section 5 crosses, a mode with no command,
a reference cited nowhere. Those defects are invisible per file and obvious in
aggregate.

## Capabilities used

| Capability | Used for | Phase |
|---|---|---|
| Very large context | the whole candidate package graded in one pass, not summarised file by file | 9, 11 |
| Gems, with the system prompt below | the OS persists as a named configuration | all |
| File upload, many files at once | the candidate and its references go in together | 9, 11 |
| Grounded search, where enabled | source capture during research | 3 |
| Structured output | the scorecard emitted directly in the shape of `TOOLS/schemas/scorecard.schema.json` | 11 |

## Installation

**As a Gem.** Create the Gem, attach `OS.md`, `SYSTEM.md`, `SKILL.md`, the
contents of `REFERENCES/` and `TOOLS/schemas/`, and paste the system prompt as
the Gem's instructions.

The system prompt:

> You are OS Builder. For any requested professional capability, define the
> problem and the value, the scope and the non scope, the research base, the
> human skill, the workflow, the evidence rules, the artifacts, the package
> components that are actually useful, the tests, the red team cases, the score,
> the repairs, and the release. Never equate an OS with one giant prompt. Every
> material conclusion carries an evidence state: VERIFIED, SUPPORTED, INFERRED,
> ASSUMED, CONFLICTING or UNKNOWN, and confidence is tracked separately from
> state. You cannot run the suite validators here; when a gate item depends on
> one, say it is unanswered and name the command the operator must run. When
> asked to score, emit the scorecard as JSON matching the attached
> scorecard.schema.json, with an evidence sentence citing a file for every one
> of the sixteen dimensions.

For a single review pass without a Gem, upload the candidate plus
`EVALS/OS-QUALITY-RUBRIC.md` and `EVALS/RELEASE-GATE.md`, and ask for the
scorecard as JSON. The rubric travels with the request, so the review does not
depend on the model recalling sixteen dimensions correctly.

## The whole package review, phase 9 and 11

The pass this target exists for. Upload every file of the candidate at once and
ask five cross file questions that no single file review can answer:

1. Does anything in `OS.md` sections 3 to 10 cross the boundary drawn in
   section 2?
2. Is every mode in section 3 reachable by a command in `COMMANDS/README.md`,
   and does every command belong to a mode?
3. Does every reference carry a `WHERE USED` that points at a decision that
   actually exists in the file it names?
4. Does every artifact promised in section 5 appear finished in `EXAMPLES/`?
5. Does any adapter contain a sentence that would be equally true in another
   adapter, and therefore says nothing about its own target?

Question 5 is the adapter dimension's own test, and it is the one a per file
review structurally cannot run, because the defect only exists in the comparison.

## Operating contract on Gemini

1. Read the attached `SYSTEM.md` before answering. Long context makes it cheap
   to actually read it rather than rely on the Gem summary.
2. Score with the rubric attached, never from recall. Sixteen dimensions
   recalled from memory become thirteen, and the three that vanish are the ones
   the package is weakest on.
3. Emit the scorecard as JSON against the schema, so
   [`../TOOLS/score_os.py`](../TOOLS/score_os.py) can apply the threshold. Do
   not state the verdict in prose: the arithmetic is the tool's job precisely
   because prose rounds.
4. Every one of the sixteen evidence sentences cites a file. A whole package
   review that cites nothing is an impression, and an impression from a large
   context is still an impression.
5. Say plainly which gate items could not be answered here.

## Unsupported capabilities

- **No filesystem and no shell.** Nothing is written into `OS/<slug>/` and no
  validator runs. Gate items 8, 11, 15, 16, 17 and 18 are unanswered here. Item
  15 is nearly answered: the scorecard JSON is produced here, the threshold is
  applied by the operator running `score_os.py`.
- **No access to the suite tooling or the live registry.** Slug resolution uses
  an uploaded `_registry.json` snapshot; state its date whenever a slug is
  asserted to exist. A handoff validated against a stale snapshot is a handoff
  that may not join.
- **No parallel independent subagents.** Phase 10 runs sequentially, recorded in
  the ledger as not independent.
- **Long context is not attention.** A package that fits in the window is not
  automatically read closely. The five cross file questions above are asked
  explicitly, one at a time, because a single request to "review this package"
  returns the same generic paragraph regardless of what is in it. This is the
  specific failure mode of this target and the reason its review is structured
  as questions rather than as an instruction to review.

## Fallbacks, declared not silent

| Cannot | Falls back to | Recorded as |
|---|---|---|
| run `validate_os.py` | the contract checklist read against the uploaded tree | gate 11, 16 unanswered |
| run `score_os.py` | scorecard emitted as JSON, threshold applied by the operator | gate 15 pending |
| run `graph.py` | slugs checked against an uploaded registry snapshot, with its date | gate 8 unanswered |
| produce a reproducible archive | the operator runs `create_zip.py` twice | gate 18 unanswered |
| write the package to disk | complete file bodies, in contract order | delivery is manual |

## What is false here that is true elsewhere

The whole package pass described above is not available on Claude or Codex in
the same form: both read files in sequence and hold summaries, which is why
their review catches per file defects and misses cross file ones. Conversely,
every mechanical gate item that Claude answers with an exit code is unanswered
here. The two targets are complementary and neither substitutes for the other:
review on Gemini, validate on Claude or Codex, and never let a strong review
stand in for an unrun validator.
