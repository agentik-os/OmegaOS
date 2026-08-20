# Builder {OS} v4 Build Lifecycle Roadmap

This document is the canonical roadmap executed for every `/os-build <Name> {OS}` launch.

## Core rule

An OS is not generated from a title. The title initializes a build program. Knowledge is researched first, normalized second, compiled into executable system logic third, and released only after evaluation and repair.

```text
REQUEST
  ↓
00 INTAKE & BUILD CONTRACT
  ↓
01 DOMAIN FRAMING
  ↓
02 OUTCOME & JOB MODEL
  ↓
03 RESEARCH DESIGN
  ↓
04 BESTSELLER & CANONICAL SOURCE DISCOVERY
  ↓
05 CORPUS CURATION
  ↓
06 BOOK DEEP FANOUT
  ↓
07 NON-BOOK EVIDENCE FANOUT
  ↓
08 SOURCE NORMALIZATION
  ↓
09 CLAIM EXTRACTION & CLAIM LEDGER
  ↓
10 CONTRADICTIONS, SCHOOLS & UNCERTAINTY
  ↓
11 KNOWLEDGE SYNTHESIS
  ↓
12 DOMAIN ONTOLOGY & STATE MODEL
  ↓
13 PRINCIPLES & DECISION RULE COMPILATION
  ↓
14 CAPABILITY ARCHITECTURE
  ↓
15 COMMAND & WORKFLOW DESIGN
  ↓
16 AGENT, SKILL, TOOL, PROMPT & MEMORY DESIGN
  ↓
17 INTER-OS HANDOFF DESIGN
  ↓
18 IMPLEMENTATION
  ↓
19 STATIC VALIDATION
  ↓
20 SCENARIO EVALS
  ↓
21 ADVERSARIAL / GAUNTLET / OMEGA AUDIT
  ↓
22 REPAIR & REGRESSION LOOP
  ↓
23 DOCUMENTATION & PRESENTATION
  ↓
24 RELEASE PACKAGING & REGISTRY
  ↓
25 CONTINUOUS UPDATE BASELINE
```

## Phase 00: Intake and build contract

Purpose: turn a minimal user request into an explicit executable contract without blocking on unnecessary questions.

Created or updated:

```text
00_control/BUILD_REQUEST.md
00_control/BUILD_CONTRACT.yaml
00_control/BUILD_STATE.json
00_control/ASSUMPTIONS.md
00_control/DECISION_LOG.md
00_control/BUILD_EVENT_LOG.jsonl
00_control/ARTIFACT_INDEX.json
```

Exit gate: scope, outcomes, exclusions, risk level, modes and assumptions are explicit.

## Phase 01: Domain framing

Purpose: understand what the domain actually contains before looking for solutions.

```text
01_domain/DOMAIN_MAP.md
01_domain/GLOSSARY.yaml
01_domain/SUBDOMAIN_MAP.yaml
01_domain/BOUNDARIES.md
01_domain/STAKEHOLDERS.yaml
01_domain/RISK_MAP.yaml
```

Exit gate: no critical subdomain is omitted without a recorded reason.

## Phase 02: Outcome and job model

```text
02_outcomes/JOB_OUTCOME_MAP.yaml
02_outcomes/USER_STAGES.yaml
02_outcomes/SUCCESS_METRICS.yaml
02_outcomes/FAILURE_MODES.yaml
02_outcomes/USE_CASES.yaml
```

Exit gate: every major capability can later be traced to a real user job or outcome.

## Phase 03: Research design

```text
03_research/RESEARCH_PROTOCOL.yaml
03_research/SEARCH_PLAN.md
03_research/SOURCE_STRATEGY.yaml
03_research/QUERY_LOG.jsonl
03_research/RECENCY_POLICY.yaml
03_research/EVIDENCE_QUALITY_RUBRIC.yaml
```

Exit gate: source classes, search lenses, recency rules, stopping conditions and evidence grading are defined.

## Phase 04: Bestseller and canonical discovery

Librarian {OS} and Research {OS} fan out across books, schools, standards, papers, practitioner sources and failure evidence.

```text
04_discovery/BOOK_CANDIDATES.jsonl
04_discovery/CANONICAL_SOURCES.jsonl
04_discovery/SCHOOL_CANDIDATES.yaml
04_discovery/EXPERT_CANDIDATES.yaml
04_discovery/GAP_LOG.yaml
```

Exit gate: discovery covers foundational, evidence-led, practitioner, current and critical perspectives.

## Phase 05: Corpus curation

```text
05_corpus/CORPUS_MATRIX.csv
05_corpus/RETAINED_BOOKS.yaml
05_corpus/REJECTED_BOOKS.yaml
05_corpus/RETAINED_NONBOOK_SOURCES.yaml
05_corpus/COVERAGE_MATRIX.yaml
05_corpus/CORPUS_AUDIT.md
```

Exit gate: retained sources collectively satisfy coverage and diversity requirements.

## Phase 06: Book deep fanout

Every retained book receives an independent deep extraction.

```text
06_books/<book-slug>/SOURCE_CARD.yaml
06_books/<book-slug>/ANALYSIS.md
06_books/<book-slug>/ANALYSIS.json
06_books/<book-slug>/CLAIMS.jsonl
06_books/<book-slug>/FRAMEWORKS.yaml
06_books/<book-slug>/PROCEDURES.yaml
06_books/<book-slug>/FAILURE_MODES.yaml
06_books/<book-slug>/OS_CANDIDATES.yaml
06_books/BOOK_SYNTHESIS_INDEX.yaml
```

Exit gate: every retained title has a schema-valid extraction and no book is silently skipped.

## Phase 07: Non-book evidence fanout

```text
07_evidence/<subdomain>/SOURCE_NOTES.md
07_evidence/<subdomain>/CLAIMS.jsonl
07_evidence/<subdomain>/EVIDENCE_TABLE.csv
07_evidence/PRIMARY_RESEARCH_INDEX.yaml
07_evidence/STANDARDS_INDEX.yaml
07_evidence/CASE_STUDY_INDEX.yaml
07_evidence/POSTMORTEM_INDEX.yaml
```

Exit gate: books are corroborated, updated or challenged with appropriate external evidence.

## Phase 08: Source normalization

```text
08_ledgers/SOURCE_LEDGER.jsonl
08_ledgers/SOURCE_GRAPH.json
08_ledgers/PROVENANCE_AUDIT.md
08_ledgers/CITATION_MAP.yaml
```

Exit gate: every material source has provenance, quality, recency, access level and applicability metadata.

## Phase 09: Claim extraction

```text
09_claims/CLAIM_LEDGER.jsonl
09_claims/CLAIM_SUPPORT_GRAPH.json
09_claims/CLAIM_GAPS.yaml
09_claims/CLAIM_AUDIT.md
```

Exit gate: material recommendations are not allowed to exist without traceable claims.

## Phase 10: Contradictions, schools and uncertainty

```text
10_conflicts/SCHOOL_MAP.yaml
10_conflicts/CONTRADICTION_REGISTER.yaml
10_conflicts/UNCERTAINTY_REGISTER.yaml
10_conflicts/CONTEXT_CONDITIONS.yaml
10_conflicts/RESOLUTION_LOG.md
```

Exit gate: disagreements are surfaced and transformed into contextual rules where possible.

## Phase 11: Knowledge synthesis

```text
11_synthesis/SYNTHESIS_MAP.yaml
11_synthesis/MECHANISM_MAP.yaml
11_synthesis/PRINCIPLES.yaml
11_synthesis/PATTERNS.yaml
11_synthesis/ANTI_PATTERNS.yaml
11_synthesis/KNOWLEDGE_BASE.md
11_synthesis/SYNTHESIS_AUDIT.md
```

Exit gate: the corpus has become a coherent domain model rather than a pile of summaries.

## Phase 12: Domain ontology and state model

```text
12_model/ONTOLOGY.yaml
12_model/ENTITY_MODEL.yaml
12_model/RELATION_MODEL.yaml
12_model/STATE_MODEL.yaml
12_model/EVENT_MODEL.yaml
12_model/SIGNAL_MODEL.yaml
12_model/ARTIFACT_MODEL.yaml
```

Exit gate: the OS can represent the domain, user state and meaningful transitions.

## Phase 13: Principles and decision rules

```text
13_logic/DECISION_RULES.yaml
13_logic/DIAGNOSTICS.yaml
13_logic/POLICIES.yaml
13_logic/ESCALATION_RULES.yaml
13_logic/STOPPING_RULES.yaml
13_logic/PRIORITIZATION_RULES.yaml
13_logic/FEEDBACK_LOOPS.yaml
```

Exit gate: important knowledge is compiled into condition-aware executable logic.

## Phase 14: Capability architecture

```text
14_architecture/OS_MANIFEST.yaml
14_architecture/CAPABILITIES.yaml
14_architecture/CAPABILITY_GRAPH.yaml
14_architecture/MODULE_MAP.yaml
14_architecture/DEPENDENCY_MAP.yaml
14_architecture/ARCHITECTURE.md
```

Exit gate: capabilities completely cover target jobs without unjustified duplication.

## Phase 15: Commands and workflows

```text
15_interfaces/COMMANDS.yaml
15_interfaces/COMMAND_REFERENCE.md
15_interfaces/WORKFLOW_MAP.yaml
15_interfaces/WORKFLOWS/*.yaml
15_interfaces/INPUT_OUTPUT_CONTRACTS.yaml
15_interfaces/ERROR_MODEL.yaml
```

Exit gate: every user-facing command maps to real runtime behavior and every workflow has explicit termination and error handling.

## Phase 16: Agents, skills, tools, prompts and memory

```text
16_runtime_design/AGENT_ROSTER.yaml
16_runtime_design/AGENTS/*.md
16_runtime_design/SKILLS/*.md
16_runtime_design/TOOLS.yaml
16_runtime_design/PROMPTS/*.md
16_runtime_design/MEMORY_MODEL.yaml
16_runtime_design/CONTEXT_POLICY.yaml
16_runtime_design/ROUTING_POLICY.yaml
```

Exit gate: responsibilities are typed, bounded and non-overlapping where possible.

## Phase 17: Inter-OS handoffs

```text
17_handoffs/HANDOFF_REGISTRY.yaml
17_handoffs/HANDOFF_CONTRACTS/*.yaml
17_handoffs/ROUTING_GRAPH.yaml
17_handoffs/FAILURE_AND_FALLBACK.md
```

Exit gate: collaborations are optional, user-controlled, traceable and disableable.

## Phase 18: Implementation

```text
18_implementation/runtime/
18_implementation/commands/
18_implementation/workflows/
18_implementation/agents/
18_implementation/skills/
18_implementation/prompts/
18_implementation/tools/
18_implementation/memory/
18_implementation/schemas/
18_implementation/registry/
18_implementation/tests/
```

Exit gate: architecture exists as runnable or machine-consumable implementation artifacts, not documentation only.

## Phase 19: Static validation

```text
19_validation/SCHEMA_REPORT.md
19_validation/LINKAGE_REPORT.md
19_validation/TRACEABILITY_REPORT.md
19_validation/COMPLETENESS_REPORT.md
19_validation/VALIDATION_FINDINGS.yaml
```

Exit gate: schemas, references, IDs and traceability chains validate.

## Phase 20: Scenario evals

```text
20_evals/EVAL_PLAN.yaml
20_evals/SCENARIOS/*.yaml
20_evals/EVAL_RESULTS.jsonl
20_evals/EVAL_REPORT.md
20_evals/REGRESSION_BASELINE.json
```

Exit gate: all critical scenarios pass the minimum score defined by the build contract.

## Phase 21: Adversarial, Gauntlet and Omega audit

```text
21_audit/GAUNTLET_PLAN.yaml
21_audit/RED_TEAM_CASES.yaml
21_audit/OMEGA_AUDIT.md
21_audit/AUDIT_FINDINGS.yaml
21_audit/RISK_ACCEPTANCE.md
```

Exit gate: no unresolved blocker or critical risk is allowed into release.

## Phase 22: Repair and regression

```text
22_repair/REPAIR_PLAN.yaml
22_repair/REPAIR_LOG.md
22_repair/CHANGESET.jsonl
22_repair/REGRESSION_REPORT.md
22_repair/OPEN_FINDINGS.yaml
```

This phase loops back to the affected upstream stage. It repeats until gates pass or the build is explicitly blocked.

## Phase 23: Documentation and presentation

```text
23_docs/README.md
23_docs/HOW_TO_USE.md
23_docs/PRESENTATION_OS.md
23_docs/COMMAND_REFERENCE.md
23_docs/WORKFLOW_COOKBOOK.md
23_docs/EXAMPLES.md
23_docs/CHANGELOG.md
```

`PRESENTATION_OS.md` must contain the exhaustive command reference, purpose, syntax, when to use, examples, workflows, modes and end-to-end sequences.

## Phase 24: Release packaging and registry

```text
24_release/RELEASE_MANIFEST.json
24_release/RELEASE_REPORT.md
24_release/CHECKSUMS.txt
24_release/REGISTRY_PATCH.yaml
24_release/MIGRATION_NOTES.md
24_release/dist/<os-slug>-v<version>.zip
```

Exit gate: package reproduces the validated workspace and contains all required runtime and documentation artifacts.

## Phase 25: Continuous update baseline

```text
25_update/UPDATE_POLICY.yaml
25_update/WATCHLIST.yaml
25_update/BASELINE_SNAPSHOT.json
25_update/REGRESSION_POLICY.yaml
25_update/SOURCE_REFRESH_PLAN.yaml
```

This establishes the data required by `/os-update` to detect changes later.

# Global invariant

Every important released rule should support this trace:

```text
USER JOB
→ SOURCE(S)
→ CLAIM(S)
→ SYNTHESIS
→ PRINCIPLE / MECHANISM
→ DECISION RULE
→ CAPABILITY
→ COMMAND / WORKFLOW
→ EVAL
→ RELEASE ARTIFACT
```

If that chain cannot be reconstructed, the system is not considered fully traceable.
