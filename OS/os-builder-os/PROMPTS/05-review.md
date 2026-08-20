# Prompt: Review

**Runs at:** phases 11 and 14 of `WORKFLOWS/FULL_BUILD.md`.
**Takes:** the built unit, its test results and its red team attack log.
**Returns:** the sixteen-dimension score card, the strengths and weaknesses, the
mandatory repairs, and a `RELEASE` or `NO RELEASE` verdict.

## Instruction

Grade this unit. Score against the rubric, not against how much work it clearly
took and not against how good the writing is. A well written unit that skipped
research scores badly on evidence discipline, and that is the correct outcome.

Every score carries one sentence of evidence naming the file, the line or the
observed behaviour that justifies it. A score with no evidence is not counted,
and a card with uncounted scores does not clear the gate.

## The sixteen dimensions

| # | Dimension | Asks |
|---|---|---|
| 1 | Value proposition | Is the promise specific and falsifiable, with an observable before and after? |
| 2 | Scope | Is the boundary real, with a named owner for everything in non-scope? |
| 3 | Domain depth | Does it know the field, or does it know the vocabulary of the field? |
| 4 | Human skill | Could a person learn the capability from it with the AI switched off? |
| 5 | Operating logic | Does every mode have an entry condition, an artifact and a completion test? |
| 6 | Evidence discipline | Is every major claim source backed or labelled synthesis, with conflicts preserved? |
| 7 | Decision quality | Are decision points, approval gates and stop conditions declared and reachable? |
| 8 | Artifact quality | Does the primary artifact have an owner, a home and a consumer? |
| 9 | Executive usability | Can a decision maker act on it in two minutes? |
| 10 | Security | Is sensitivity classified, is collection minimised, are controls proportionate? |
| 11 | Testability | Do the tests go beyond the happy path, and did they run? |
| 12 | Traceability | Does every recommendation trace to a finding, evidence and a source? |
| 13 | Reusability | Does it work for a second user in a second context without a rewrite? |
| 14 | Installability | Does it register, scaffold, verify and configure without hand editing a generated file? |
| 15 | Handoffs | Are the upstream and downstream contracts typed, and do the events join up? |
| 16 | Adapters | Does each target record what it cannot do, and the fallback? |

## The scale

| Score | Meaning |
|---|---|
| 5 | Exceptional. Better than the field's normal standard. |
| 4 | Strong, professional release quality. |
| 3 | Usable but incomplete. Someone will have to finish it. |
| 2 | Weak. Present in form, not in substance. |
| 1 | Superficial. It names the thing without doing it. |
| 0 | Absent. |

## Output shape

### 1. Score card

```
Dimension:               (name)
Score:                   (0 to 5)
Evidence:                (one sentence naming the file, line or behaviour)
```

Sixteen blocks, then the average to one decimal place. The dimension anchors
are in [`EVALS/OS-QUALITY-RUBRIC.md`](../EVALS/OS-QUALITY-RUBRIC.md), and the
card is filled from `TOOLS/score_os.py --template` so the threshold is applied
by the tool rather than by you.

### 2. Findings

```
Strengths:               (what is genuinely good, specific, at most five)
Weaknesses:              (specific, each tied to a dimension)
Missing:                 (what the contract or the spec requires and is absent)
Unsupported:             (claims with no evidence and no synthesis label)
Overbuilt:               (components that should be deleted, and why)
Underbuilt:              (components that are present but hollow)
```

Overbuilt is the section reviewers skip, and it is the one that keeps the suite
maintainable. A package that is 20 percent larger than the capability needs
costs that 20 percent again at every future change.

### 3. Mandatory repairs

Every mandatory dimension below 4, and every critical or high red team finding.
Each with the file, the specific change, and the dimension it moves.

### 4. Machine verdict

Pasted, not summarised:

```
python3 OS/_tools/verify.py --full <slug>
python3 OS/_tools/graph.py --strict
python3 OS/_tools/suite.py check
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json
```

The tool output outranks your judgement on everything it covers. If your card
says installability 5 and `verify.py` exits 1, the card is wrong.

### 5. Release gate

Eighteen conditions, from [`EVALS/RELEASE-GATE.md`](../EVALS/RELEASE-GATE.md).
Every one must be yes.

```
[ ] bounded capability
[ ] specific value proposition
[ ] primary artifact defined
[ ] human skill defined
[ ] evidence states defined
[ ] decision gates defined
[ ] stop conditions defined
[ ] handoffs defined
[ ] appropriate security controls
[ ] tests beyond happy path
[ ] substantive files
[ ] realistic examples
[ ] purposeful references
[ ] no unsupported major claims
[ ] quality threshold passed
[ ] package validated
[ ] changelog updated
[ ] reproducible package
```

### 6. Verdict

`RELEASE` requires all four: no dimension below 4, the five critical dimensions
(evidence discipline, operating logic, artifact quality, security, testability)
at 4 or above with no waiver available, a mean of at least 4.3, and eighteen
yeses. Anything else is `NO RELEASE`, and the verdict names the shortest path
back: which dimensions, which repairs, which phase.

## Rejection criteria

Return `NO RELEASE` immediately, without completing the card, when the unit is
any of: generic (it would read the same for a different capability), unbounded
(no real non-scope), chat only (it produces conversation rather than an
artifact), unsupported (its core logic rests on nothing), insecure (it collects
what it does not need or skips a control the domain requires), untested (the
matrix was reasoned about rather than run), or mostly placeholders.

Say which, name the evidence, and stop. A full score card over a unit that
fails one of these seven is wasted work that also legitimises the unit by
implying it was close.
