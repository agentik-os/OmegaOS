# Reference Policy

What a generated OS is allowed to cite, how each source is captured, and how far
that source may be pushed. This policy governs phase 3 (Research) and is scored
by rubric dimension 3 (domain depth) and dimension 6 (evidence discipline).

## The capture record

Every source is captured in full before it is used. Eight fields, none optional,
none inferred:

```
TITLE                 exact title as published
AUTHOR OR ORG         the entity accountable for the claim
TYPE                  foundational | official | regulatory | academic |
                      technical | practical | book | case_study
YEAR                  publication year, or "undated" (never guessed)
WHY IT MATTERS        one sentence: what this source lets the OS decide
KEY IDEAS USED        the specific ideas taken, not a summary of the work
LIMITATIONS           what this source cannot support, in its own terms
WHERE USED            file and decision, e.g. "OS.md section 7, invariant 3"
```

A record missing `WHERE USED` is not a reference, it is a reading note. Reading
notes do not ship in the package.

A record missing `LIMITATIONS` is incomplete by construction: every source has
limitations, and an agent that cannot name one has not read the source closely
enough to cite it.

## Capture rules

1. **Capture before use, never after.** Writing the operating logic first and
   hunting for a citation afterwards produces post hoc justification, and it is
   visible in the output because the citation supports the sentence rather than
   the sentence following the source.
2. **Quote the load bearing line.** When a source carries a specific number, a
   threshold or a definition the OS depends on, the record holds the quoted line
   verbatim. Paraphrase drifts, and the drift is always in the direction the
   author wanted.
3. **Year is a fact, not a guess.** An undated source is recorded as `undated`
   and may never support a claim about the current state of anything.
4. **The accountable entity, not the aggregator.** Cite the standards body, not
   the blog that summarised it. If the primary source could not be reached, the
   record says so and the class drops to `practical`.
5. **A source that was not read is not cited.** Not by title, not from memory,
   not "as widely reported". Absence of a source is a reportable state and is
   recorded as `UNKNOWN` in the evidence register.

## What a class may and may not support

The trust table in [`README.md`](README.md) is normative and is repeated here as
the operative constraint set. Violating any of these is a scored defect:

- `case_study` supports existence and failure modes. It never supports a base
  rate, a typical value, or "most teams".
- `practical` supports how work is done. It never supports a measured number.
- `official` supports the vendor's own spec, limits and prices. It never
  supports a comparison against a competitor, and never supports a claim about
  the vendor's reliability.
- `academic` supports a measured effect **under the conditions of the study**.
  Extending it past those conditions is inference, and inference is labelled.
- `foundational` and `book` support vocabulary and mental models. Neither
  supports a current number.
- `regulatory` supports what is required or forbidden. It never supports how
  strictly that requirement is enforced in practice.
- `technical` supports mechanism and interface. It never supports intent,
  roadmap or "this is the recommended approach" unless the document says so in
  those words.

## Source sufficiency by capability risk

Research depth scales with what the generated OS is allowed to touch, not with
how interesting the topic is.

| Capability risk | Minimum sources | Class requirement |
|---|---|---|
| Low: internal drafting, formatting, ideation | 3 | any, at least one not `practical` |
| Medium: decisions with reversible cost | 5 | at least one `official` or `technical` |
| High: money, production systems, employment | 8 | at least one `regulatory` or `official`, at least one `academic` or `case_study` naming a failure |
| Regulated: health, legal rights, compliance, finance | 10 | `regulatory` mandatory, and the OS declares the jurisdiction it was researched against |

Falling short of the minimum is a block, not a warning. The build stops and the
gate item "purposeful references" fails.

## Conflict

When two sources disagree on a load bearing point, **both are kept**. The
conflict is recorded in the evidence register with state `CONFLICTING`, the
generated OS states which side its logic follows and why, and the losing source
stays in the package with its `LIMITATIONS` field explaining the disagreement.

Averaging two disagreeing sources into a middle number is forbidden. It produces
a figure that no source supports and that nobody can defend when challenged. See
[`TRACEABILITY.md`](TRACEABILITY.md).

## Licensing and redistribution

A generated OS may cite a paid, licensed or proprietary corpus. It may not
**contain** it. The package carries the capture record, the quoted load bearing
line, and a pointer to where the operator's own copy lives. It never carries the
source text wholesale, and it never carries a corpus the operator paid for into
a package intended to be shared.

## Anti patterns, each a scored defect

- A bibliography where no entry has a `WHERE USED`.
- A source cited in the README and used nowhere in the operating logic.
- A count of sources presented as a measure of rigour.
- A URL with no capture record behind it.
- "Studies show" with no study named.
- A number whose source is the model's own prior.
