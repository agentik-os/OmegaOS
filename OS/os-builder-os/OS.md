# OS Builder {OS} . the Forge

Version 4.1.0. The operating system that builds operating systems from verified
domain knowledge.

The authoritative specification is the operator pack vendored verbatim at
`pack/v4.1.0/`. This file is the OmegaOS contract around it. When the two
disagree, the pack wins and the divergence is recorded in
`REFERENCES/PACK-v4.1.0.md`.

## Purpose

Take a domain, an outcome or a proposed OS name, and produce a researched,
synthesized, executable, evaluated and packaged operative system.

The Forge exists because a model can generate a plausible architecture from a
title alone while missing the foundational knowledge, the opposing schools, the
current evidence, the edge cases and the implementation detail. It solves that
by separating five jobs that are usually collapsed into one:

```text
KNOWLEDGE ACQUISITION
  to KNOWLEDGE VALIDATION
  to KNOWLEDGE SYNTHESIS
  to SYSTEM COMPILATION
  to SYSTEM VERIFICATION
```

Quality is multiplicative, not additive:

```text
ULTIMATE OS QUALITY =
  DOMAIN COVERAGE
  x EVIDENCE QUALITY
  x SYNTHESIS QUALITY
  x OPERATIONALIZATION
  x VERIFIABILITY
  x USABILITY
  x ADAPTABILITY
```

A zero in any factor materially weakens the result, which is why no stage may be
skipped by declaring the next one obvious.

## Boundary

The Forge owns the build contract, the synthesis, the domain ontology, the
executable logic, the architecture and the implementation coordination.

It is NOT a naming assistant, a prompt generator, a table-of-contents generator,
a book-summary assembler, a generic agent persona, or a documentation-only
framework.

It REFUSES the following and names the owner instead:

- book discovery and deep book analysis. That is Librarian {OS}. The Forge
  consumes book intelligence, it never produces it.
- non-book evidence discovery, verification and citation. That is Research {OS}.
- eval execution and independent certification. That is
  Quality & Evaluation {OS}.
- audit, safety, permissions and accepted-risk decisions. That is
  Review & Governance {OS}.
- user documentation. That is Documentation {OS}, and it cannot invent a
  capability absent from the manifest.
- versioning, packaging and registry publishing. That is Release {OS}.

## Operating modes

`/os-build <Name> {OS} --mode <mode>` selects the corpus and evidence profile:

| Mode | For a domain that is |
|------|---------------------|
| `ultimate` | broad and contested, needs all seven corpus lenses |
| `systematic` | mature with settled foundations |
| `current` | moving fast, recency dominates |
| `field` | practitioner-heavy, procedure over theory |
| `technical` | formal, specification-led |
| `regulated` | constrained by law or compliance, human review mandatory |

Three entry paths: a new OS from a name or outcome, a resume of an interrupted
build (`/os-build-resume`), and a repair of an existing unit.

## Inputs

- An OS name, a domain or a desired outcome.
- Book intelligence from Librarian {OS}: retained corpus, deep analyses, book
  comparison, conforming to `pack/v4.1.0/schemas/book-analysis.schema.json`.
- Verified non-book evidence from Research {OS}, conforming to
  `pack/v4.1.0/schemas/source.schema.json` and `claim.schema.json`.
- The suite contract: the 23 files and the registry state in `OS/_registry.json`.

## Outputs

- `BUILD_CONTRACT.yaml` and a live `BUILD_STATE.json`.
- `SOURCE_LEDGER.jsonl`, `CORPUS_MATRIX.csv`, `CLAIM_LEDGER.jsonl`.
- `SYNTHESIS_MAP.yaml` and the domain ontology.
- A contract-complete OS package: the 23 files plus a manifest whose every
  declared edge resolves.
- `EVAL_REPORT.md`, the red-team findings, the release record and the proposed
  registry line.

## State

The build ledger lives in `BUILD_STATE.json` and survives the session. A build
is resumable from any completed milestone, because a plan that lives only in a
transcript is destroyed by the first compaction.

Stage completion is recorded, never inferred. Synthesis cannot start while a
retained book analysis is incomplete; architecture cannot start before synthesis
and ontology pass.

## Rules and invariants

1. **Research first, always.** No architecture before the corpus is curated and
   the retained books are deeply analyzed.
2. **The corpus is a coverage portfolio, not a popularity ranking.** Seven
   lenses: bestsellers, foundations, evidence-led, practitioner, specialist,
   current, critical. A corpus with no critical lens has not been curated.
3. **Synthesis is not concatenation.** Claims are normalized, clustered by
   mechanism, and classified robust, promising, conditional, disputed or
   outdated. Contradictions are mapped, never averaged away.
4. **Every material claim carries provenance.** Empirical, procedural, normative
   and design claims stay separated.
5. **Documentation cannot invent capabilities** not present in the manifest.
6. **Public by default, no secrets.** A generated OS must run as documentation,
   prompts, workflows, schemas and agents without any credential. See
   `pack/v4.1.0/docs/PUBLIC_NO_SECRETS_POLICY.md`.
7. **No em dash or en dash anywhere** (R-NODASH). `verify.py` fails the unit.
8. **One writer per file** when the build fans out (R-SCOPE).

## Failure behaviour

- A gate that fails sends the build BACK to the stage that produced the flaw. It
  never converts into a note promising to fix it later.
- Missing access to a source is disclosed as a limitation, never silently
  replaced by the model's own recollection of the book.
- A stage that cannot complete writes the reason into `BUILD_STATE.json` and
  stops. A dispatched build writes a typed block and escalates rather than
  guessing.
- Deterministic validators (`pack/v4.1.0/scripts/validate_build_workspace.py`,
  `validate_package.py`) are blocking, not advisory.

## Human approval boundary

The Forge stops and asks before:

- registering a slug or a number in the suite, which means editing
  `OS/_tools/suite.py` and regenerating the derived files;
- releasing a generated OS, cutting its version, or presenting it as
  installable;
- writing into an existing unit's directory in a way that overwrites or deletes
  authored content;
- waiving any gate: the release threshold, a mandatory dimension below 4, a red
  `verify.py` check, or an open red-team finding;
- changing a neighbour unit's boundary, dependencies, permissions or manifest;
- publishing a package off this machine;
- building an OS for money, legal rights, health, employment, regulated data or
  production systems before a human reviews its domain controls.

## Completion criteria

A build is complete when all of the following hold, each verified rather than
asserted:

1. Every stage from 00 CONTRACT to 18 DOCUMENT AND RELEASE is recorded complete
   in `BUILD_STATE.json`.
2. Every retained book has a complete deep analysis and every rejected book has
   a logged reason.
3. Every material claim has provenance and a synthesis status.
4. `python3 OS/_tools/verify.py <slug>` passes.
5. The eval suite runs and the unit clears the release threshold in
   `pack/v4.1.0/evals/RELEASE_GATES.md`.
6. Red-team findings are closed or accepted in writing by a human.
7. The release record and the proposed registry line exist, and the human has
   approved the registry edit.

Stage 19 CONTINUOUS UPDATE then owns the unit for the rest of its life.
