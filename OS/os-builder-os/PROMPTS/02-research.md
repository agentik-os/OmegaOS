# Prompt: Research

**Runs at:** phase 3 of `WORKFLOWS/FULL_BUILD.md`.
**Takes:** the capability definition and its domain.
**Returns:** the source plan, the evidence register, the conflict register, and
the synthesis label on every claim the unit will make.
**Skipped by:** `FAST_BUILD.md`, which is exactly why a fast unit may not be
depended on until it is promoted.

## Instruction

Find out whether this capability is actually understood, by whom, and how well.
Then decide which of the unit's claims rest on that understanding and which are
your own synthesis. Both are legitimate. Presenting synthesis as established
practice is not.

Research is the phase that gets skipped, and it is the phase that decides
whether the unit is true. Every other phase can be repaired by rewriting.
This one cannot.

## Output shape

### 1. Source plan

Cover the classes that are relevant to this domain, and say explicitly which
classes are not relevant and why. An empty class with a reason is information;
an empty class with no reason is an omission.

| Class | What it supplies |
|---|---|
| Foundational | the models the field actually thinks in |
| Official | standards bodies, regulators, vendor documentation of record |
| Academic | peer reviewed work, with its sample and its date |
| Practitioner | what people who do this every day report |
| Regulatory | what is legally required, per jurisdiction |
| Technical | specifications, protocols, reference implementations |
| Book | the long form treatment, where one exists |
| Case study | what happened when someone tried it |

### 2. Evidence register

One entry per source, eight fields, all present.

```
Title:
Author or organization:
Type:                    (foundational | official | academic | practitioner |
                          regulatory | technical | book | case study)
Year:
Why it matters:
Key ideas used:
Limitations:
Where used:              (the file and section of the unit it supports)
```

`Where used` is the field that keeps the register honest. A source that
supports no logic in the unit is deleted, not kept for weight. Report how many
you deleted: a register that only ever grows is a bibliography, and a
bibliography is decoration.

`Limitations` is mandatory and never empty. Every source has a scope beyond
which it does not apply: a date, a sample, a jurisdiction, a company size, a
vendor interest. Recording that boundary is what makes the source usable rather
than merely cited.

### 3. Conflict register

Where two sources disagree on a material point, record both, in full, with
their evidence.

```
Point in dispute:
Position A:              (source, claim, evidence, limitation)
Position B:              (source, claim, evidence, limitation)
Why it matters here:
Resolution:              (unresolved | resolved by <source> | escalated to human)
```

Never average two disagreeing authorities into a third position nobody holds.
Never pick the more recent one by default: recency is one input, not a
tiebreak. An unresolved conflict is carried into the unit as an unresolved
conflict, and the unit's operating logic is written so that both positions
remain workable until a human decides.

### 4. Synthesis ledger

Every substantive claim the unit will make, classified.

```
Claim:
Basis:                   (source derived | original synthesis)
Source:                  (the register entry, when source derived)
Confidence:              (high | medium | low)
What would change it:
```

Original synthesis is allowed and often necessary. It is labelled, so a reader
can weigh it as your reasoning rather than as the field's consensus. A unit
whose claims are all source derived is probably not adding anything. A unit
whose claims are all original synthesis is probably not researched.

### 5. Gate report

State plainly, in one line each:

- How many pieces of major operating logic are source backed.
- How many are labelled original synthesis.
- How many are currently neither, which is the number that must reach zero.

**Gate:** the third number is zero. Every piece of major operating logic is
either source backed with a citation or explicitly labelled original synthesis.

## Refusals

- Do not invent a source, an author, a year, a study, a sample size or a
  statistic. Not as an illustration, not as a placeholder, not with a caveat.
- Do not cite a source you have not read enough of to fill its `Limitations`
  field.
- Do not report a source class as covered when what you found was a blog post
  summarising it. Name what you actually read.
- When the domain has no evidence base, say so. That finding blocks the release
  and it is the correct output: a unit whose core logic is unsupported ships as
  draft with the unsupported claims marked, or the capability is narrowed until
  it is supportable.
