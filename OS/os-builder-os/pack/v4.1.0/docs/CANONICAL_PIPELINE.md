# Canonical OS build pipeline

## Pipeline overview

```text
00 CONTRACT
01 FRAME
02 JOBS AND OUTCOMES
03 RESEARCH PROTOCOL
04 DISCOVER BOOKS
05 CURATE CORPUS
06 DEEP ANALYZE BOOKS
07 EXPAND EVIDENCE
08 NORMALIZE CLAIMS
09 MAP CONTRADICTIONS
10 SYNTHESIZE KNOWLEDGE
11 BUILD ONTOLOGY
12 COMPILE LOGIC
13 ARCHITECT SYSTEM
14 IMPLEMENT PACKAGE
15 DESIGN AND RUN EVALS
16 RED TEAM
17 REPAIR
18 DOCUMENT AND RELEASE
19 CONTINUOUS UPDATE
```

## 00. Contract gate

**Inputs:** OS name or desired outcome.

**Outputs:** `BUILD_CONTRACT.yaml`, initial `BUILD_STATE.json`.

**Pass condition:** Scope, target user, outcomes, non-goals, risk and deliverables are explicit.

## 01. Domain framing gate

**Outputs:** domain map, glossary draft, boundary map.

**Pass condition:** Major subdomains, actors, contexts, risks and ambiguous terms are represented.

## 02. Jobs and outcomes gate

**Outputs:** job map and measurable success model.

**Pass condition:** Every planned capability supports at least one defined job and outcome.

## 03. Research protocol gate

**Outputs:** `RESEARCH_PROTOCOL.yaml`.

**Pass condition:** Search questions, source classes, inclusion rules, recency and stop rules are explicit.

## 04. Discovery gate

**Outputs:** candidate source inventory.

**Pass condition:** Bestseller, foundational, evidence-led, specialist, current and contrarian categories have been searched.

## 05. Corpus curation gate

**Outputs:** `CORPUS_MATRIX.csv`, retained and rejected lists.

**Pass condition:** Coverage is adequate, redundancy is controlled and every major school has fair representation.

## 06. Book-deep gate

**Outputs:** one validated analysis per retained book.

**Pass condition:** No retained title lacks a schema-complete deep analysis.

## 07. Evidence expansion gate

**Outputs:** non-book source records and extracted claims.

**Pass condition:** Currentness, primary evidence, official authority and field reality are covered as appropriate.

## 08. Claim normalization gate

**Outputs:** `SOURCE_LEDGER.jsonl`, `CLAIM_LEDGER.jsonl`.

**Pass condition:** Material claims have provenance, type, confidence, scope and implementation status.

## 09. Contradiction gate

**Outputs:** school map and contradiction register.

**Pass condition:** Disagreements are classified and unresolved conflicts are visible.

## 10. Synthesis gate

**Outputs:** `SYNTHESIS_MAP.yaml`.

**Pass condition:** Claims have been compiled into mechanisms, principles, conditions and candidate rules.

## 11. Ontology gate

**Outputs:** entity, state, event and relationship definitions.

**Pass condition:** Workflow objects use consistent domain language and state definitions.

## 12. Logic compilation gate

**Outputs:** diagnostics, decision tables, rules, loops, exceptions and escalation policies.

**Pass condition:** Every material principle has an operational consequence or is intentionally informational.

## 13. Architecture gate

**Outputs:** `OS_MANIFEST.yaml`, command and workflow design.

**Pass condition:** Capabilities, commands, workflows, state, memory and handoffs are coherent and traceable.

## 14. Implementation gate

**Outputs:** runtime package and docs skeleton.

**Pass condition:** Every declared command is implemented or explicitly marked interface-only with reason.

## 15. Evaluation gate

**Outputs:** test suite and `EVAL_REPORT.md`.

**Pass condition:** All critical tests pass and quality thresholds are met.

## 16. Red-team gate

**Outputs:** adversarial findings.

**Pass condition:** No unmitigated critical or high-severity finding remains.

## 17. Repair gate

**Outputs:** patches, regression tests and updated reports.

**Pass condition:** Failed gates are rerun and pass.

## 18. Release gate

**Outputs:** HOW_TO_USE, presentation, reports, ZIP, registry update.

**Pass condition:** Package validates, documentation agrees with implementation and version is immutable.

## 19. Update gate

**Outputs:** monitored source map and migration protocol.

**Pass condition:** The OS can identify what evidence or logic needs reevaluation when the domain changes.
