# Book intelligence contract . Librarian {OS} to OS Builder {OS}

Librarian {OS} owns stages **04 DISCOVER BOOKS**, **05 CURATE CORPUS** and
**06 DEEP ANALYZE BOOKS** of the Forge pipeline
(`OS/os-builder-os/WORKFLOWS/ULTIMATE_BUILD.md`). The Forge consumes book
intelligence and never produces it.

This extends the Librarian's existing purpose rather than replacing it. Its
first job stays the operator's own corpus and its retrieval problem. This
contract is its second job: supplying the Forge with the domain knowledge that
every new OS is built from.

## What the Forge sends

A corpus brief: the domain, the target user, the seven-lens targets, the
saturation goal, the language scope, and the schema every retained book analysis
must satisfy (`OS/os-builder-os/pack/v4.1.0/schemas/book-analysis.schema.json`).

## Stage 04 . discover, through seven lenses

The candidate list is built through all seven, and a discovery that returns only
lens 1 is incomplete, not fast:

1. **Bestsellers** . the influential ideas and the vocabulary practitioners use
2. **Foundations** . the classics that created the schools
3. **Evidence-led** . grounded in research or formal method
4. **Practitioner** . repeatable field procedure, operational detail
5. **Specialist** . the subdomain depth broad books omit
6. **Current** . recent developments, updated editions, changed conditions
7. **Critical** . counter-evidence, limits, misuse, failure analysis

## Stage 05 . curate into a coverage portfolio

Score each candidate 0 to 5 on domain relevance, source authority, evidence
transparency, practical actionability, unique coverage contribution, field
influence, currentness, counterpoint value, transferability and known limitation
severity. The score supports judgment, it does not replace it.

**Retain** a source that contributes at least one of: a unique mechanism, a major
school, a validated procedure, a missing subdomain, a useful counterargument, a
critical failure mode, a current update, or the canonical vocabulary of the
field.

**Reject** for substantial redundancy, access too weak to support reliable
extraction, or unverifiable claims with no unique operational value. **Every
rejection is logged with its reason.** An unlogged rejection is indistinguishable
from an oversight, and the Forge cannot tell the difference at stage 10.

The output is `CORPUS_MATRIX.csv`. It is a coverage portfolio, never a popularity
ranking.

## Stage 06 . deep analysis, per retained book

Every retained book produces a human-readable analysis AND a schema-valid
structured record. Never a summary. The required sections are fixed by
`OS/os-builder-os/pack/v4.1.0/docs/BOOK_DEEP_EXTRACTION_STANDARD.md`:

1. **Identity** . title, author, edition and date, category, access basis,
   analyst confidence
2. **Thesis and problem** . central thesis, the problem being solved, target
   reader, promised transformation, underlying worldview
3. **Conceptual system** . core concepts, causal mechanisms, models, sequence or
   lifecycle, definitions, prerequisites
4. **Operational intelligence** . diagnostics, decision rules, procedures,
   exercises, scripts, cadence, metrics, review loops, stopping rules,
   escalation rules
5. **Evidence and limitations** . the evidence the author uses, and where it runs
   out

Section 4 is the one that makes a book usable by an OS. A book analysis with a
rich section 3 and an empty section 4 has been read, not extracted.

**Access honesty is mandatory.** When the full text is not available, the access
basis says so and the analyst confidence drops. A remembered book presented as a
read one poisons every claim downstream, and the poison is invisible by stage 10.

## What the Forge receives back

- `CORPUS_MATRIX.csv` . retained and rejected, with reasons
- one book analysis per retained book, schema-valid
- a book comparison across the corpus: where the authors agree, where they
  genuinely disagree, and which disagreements are real conflicts rather than
  vocabulary differences

That last artifact is what lets stage 10 take the best of every book instead of
averaging them into a bland middle.

## Events

Emits `librarian.corpus.discovered`, `librarian.book.analyzed`,
`librarian.corpus.compared`. Consumes `osbuilder.contract.opened` and
`osbuilder.bookdeep.requested`.
