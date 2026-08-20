# ULTIMATE BUILD . the canonical 20-stage pipeline

The full research-first build. Source of truth:
`pack/v4.1.0/docs/CANONICAL_PIPELINE.md`. This file is the OmegaOS operating
view of it: who owns each stage, what it emits, and what must be true to pass.

Trigger: `/os-build <Name> {OS} --mode ultimate`.

No stage is skipped by declaring the next one obvious. Quality is multiplicative,
so a zero anywhere is a zero overall.

| # | Stage | Owner | Output | Pass condition |
|---|-------|-------|--------|----------------|
| 00 | CONTRACT | Forge | `BUILD_CONTRACT.yaml`, `BUILD_STATE.json` | Scope, user, outcomes, non-goals, risk, deliverables explicit |
| 01 | FRAME | Forge | domain map, glossary, boundary map | Subdomains, actors, contexts, risks, ambiguous terms represented |
| 02 | JOBS AND OUTCOMES | Forge | job map, success model | Every planned capability supports a defined job and outcome |
| 03 | RESEARCH PROTOCOL | Research {OS} | `RESEARCH_PROTOCOL.yaml` | Questions, source classes, inclusion, recency and stop rules explicit |
| 04 | DISCOVER BOOKS | **Librarian {OS}** | candidate corpus | Bestseller discovery done across the seven lenses |
| 05 | CURATE CORPUS | **Librarian {OS}** | `CORPUS_MATRIX.csv` | Retained corpus diverse and non-redundant, every rejection logged |
| 06 | DEEP ANALYZE BOOKS | **Librarian {OS}** | one book analysis per retained book | Every retained book has a complete deep analysis |
| 07 | EXPAND EVIDENCE | Research {OS} | `SOURCE_LEDGER.jsonl` | Non-book evidence found, verified, triangulated, cited |
| 08 | NORMALIZE CLAIMS | Forge | `CLAIM_LEDGER.jsonl` | Empirical, procedural, normative and design claims separated, provenance on each |
| 09 | MAP CONTRADICTIONS | Forge | contradiction map | Schools that disagree are represented, not averaged away |
| 10 | SYNTHESIZE KNOWLEDGE | Forge | `SYNTHESIS_MAP.yaml` | Claims classified robust, promising, conditional, disputed, outdated |
| 11 | BUILD ONTOLOGY | Forge | domain ontology | Concepts normalized, synonyms mapped, distinctions preserved |
| 12 | COMPILE LOGIC | Forge | principles and decision rules | Rules carry trigger, action, exception and evidence |
| 13 | ARCHITECT SYSTEM | Forge | OS architecture | Capabilities map to jobs, commands to capabilities and workflows |
| 14 | IMPLEMENT PACKAGE | Forge | the 23 contract files | `verify.py <slug>` passes STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH, SUBSTANCE |
| 15 | DESIGN AND RUN EVALS | Quality & Evaluation {OS} | `EVAL_REPORT.md` | Critical scenarios tested, deterministic validators green |
| 16 | RED TEAM | Review & Governance {OS} | findings with severity | Attacks run, expected versus actual behaviour recorded |
| 17 | REPAIR | Forge | fixes and re-score | Every mandatory dimension at or above threshold, red checks closed |
| 18 | DOCUMENT AND RELEASE | Documentation {OS}, Release {OS} | release record, registry line | Documentation invents nothing absent from the manifest |
| 19 | CONTINUOUS UPDATE | Review & Governance {OS} | update log | The unit has a living owner and a review cadence |

## The three ordering invariants

These are not style. Violating one produces a plausible OS built on nothing.

1. **Architecture never precedes synthesis.** Stage 13 cannot start before 10
   and 11 pass.
2. **Synthesis never starts on an incomplete corpus.** Stage 10 cannot start
   while any retained book analysis from stage 06 is missing.
3. **Release never bypasses a critical gate.** Stage 18 cannot start with an
   open red-team finding unless a human accepted the risk in writing.

## Where the corpus comes from

Stages 04, 05 and 06 are delegated to **Librarian {OS}** and are the reason this
pipeline produces something a title-driven generator cannot. The corpus is built
through seven lenses and is a coverage portfolio, never a bestseller list:

1. **Bestsellers** . influential ideas and the language practitioners actually use
2. **Foundations** . the classics that created the schools
3. **Evidence-led** . works grounded in research or formal method
4. **Practitioner** . repeatable field procedure and operational detail
5. **Specialist** . subdomain depth the broad books omit
6. **Current** . recent developments and changed conditions
7. **Critical** . counter-evidence, limits, misuse, failure analysis

A corpus with no critical lens has not been curated, it has been collected.

## Taking the best of every book

Stage 10 is where the many source-specific models become one operating model.
It is a transformation, never a concatenation, and it runs in four moves
(`pack/v4.1.0/docs/KNOWLEDGE_SYNTHESIS_PROTOCOL.md`):

1. **Normalize concepts.** Canonical IDs, synonyms mapped, distinctions
   preserved BEFORE anything is merged.
2. **Normalize claims.** Each becomes one proposition with subject, mechanism,
   outcome, population, conditions, time horizon and source support.
3. **Cluster by mechanism.** Group claims describing the same process even when
   the authors use different vocabulary. This is where two books that look
   opposed turn out to describe one mechanism at different scales.
4. **Classify agreement.** Robust, promising, conditional, disputed, outdated.
   A disputed claim ships AS disputed, with both sides and the condition that
   separates them. Averaging two schools into a bland middle is the failure this
   stage exists to prevent.

## Related workflows

- `FAST_BUILD.md` . the reduced path, legitimate only for a narrow, settled domain
- `REPAIR_AN_EXISTING_OS.md` . entry at stage 17 for a unit that already ships
- `pack/v4.1.0/workflows/book-deep-fanout.yaml` . parallel deep analysis of a retained corpus
- `pack/v4.1.0/workflows/evidence-synthesis.yaml` . stages 08 to 10
- `pack/v4.1.0/workflows/eval-repair-loop.yaml` . the bounded 15 to 17 loop
