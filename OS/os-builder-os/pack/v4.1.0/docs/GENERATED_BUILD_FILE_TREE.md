# Generated build file tree

Running:

```text
/os-build <Name> {OS}
```

or locally:

```bash
python scripts/create_build_workspace.py "<Name>" --output builds
```

creates the following build repository immediately. Files begin as structured templates and become the single source of truth for their phase.

```text
<name>-os/
├── README.md
├── 00_control/
│   ├── BUILD_REQUEST.md
│   ├── BUILD_CONTRACT.yaml
│   ├── BUILD_STATE.json
│   ├── ASSUMPTIONS.md
│   ├── DECISION_LOG.md
│   ├── BUILD_EVENT_LOG.jsonl
│   └── ARTIFACT_INDEX.json
├── 01_domain/
│   ├── DOMAIN_MAP.md
│   ├── GLOSSARY.yaml
│   ├── SUBDOMAIN_MAP.yaml
│   ├── BOUNDARIES.md
│   ├── STAKEHOLDERS.yaml
│   └── RISK_MAP.yaml
├── 02_outcomes/
│   ├── JOB_OUTCOME_MAP.yaml
│   ├── USER_STAGES.yaml
│   ├── SUCCESS_METRICS.yaml
│   ├── FAILURE_MODES.yaml
│   └── USE_CASES.yaml
├── 03_research/
│   ├── RESEARCH_PROTOCOL.yaml
│   ├── SEARCH_PLAN.md
│   ├── SOURCE_STRATEGY.yaml
│   ├── QUERY_LOG.jsonl
│   ├── RECENCY_POLICY.yaml
│   └── EVIDENCE_QUALITY_RUBRIC.yaml
├── 04_discovery/
│   ├── BOOK_CANDIDATES.jsonl
│   ├── CANONICAL_SOURCES.jsonl
│   ├── SCHOOL_CANDIDATES.yaml
│   ├── EXPERT_CANDIDATES.yaml
│   └── GAP_LOG.yaml
├── 05_corpus/
│   ├── CORPUS_MATRIX.csv
│   ├── RETAINED_BOOKS.yaml
│   ├── REJECTED_BOOKS.yaml
│   ├── RETAINED_NONBOOK_SOURCES.yaml
│   ├── COVERAGE_MATRIX.yaml
│   └── CORPUS_AUDIT.md
├── 06_books/
│   ├── BOOK_SYNTHESIS_INDEX.yaml
│   └── <book-slug>/
│       ├── SOURCE_CARD.yaml
│       ├── ANALYSIS.md
│       ├── ANALYSIS.json
│       ├── CLAIMS.jsonl
│       ├── FRAMEWORKS.yaml
│       ├── PROCEDURES.yaml
│       ├── FAILURE_MODES.yaml
│       └── OS_CANDIDATES.yaml
├── 07_evidence/
│   ├── PRIMARY_RESEARCH_INDEX.yaml
│   ├── STANDARDS_INDEX.yaml
│   ├── CASE_STUDY_INDEX.yaml
│   ├── POSTMORTEM_INDEX.yaml
│   └── <subdomain>/...
├── 08_ledgers/
│   ├── SOURCE_LEDGER.jsonl
│   ├── SOURCE_GRAPH.json
│   ├── PROVENANCE_AUDIT.md
│   └── CITATION_MAP.yaml
├── 09_claims/
│   ├── CLAIM_LEDGER.jsonl
│   ├── CLAIM_SUPPORT_GRAPH.json
│   ├── CLAIM_GAPS.yaml
│   └── CLAIM_AUDIT.md
├── 10_conflicts/
│   ├── SCHOOL_MAP.yaml
│   ├── CONTRADICTION_REGISTER.yaml
│   ├── UNCERTAINTY_REGISTER.yaml
│   ├── CONTEXT_CONDITIONS.yaml
│   └── RESOLUTION_LOG.md
├── 11_synthesis/
│   ├── SYNTHESIS_MAP.yaml
│   ├── MECHANISM_MAP.yaml
│   ├── PRINCIPLES.yaml
│   ├── PATTERNS.yaml
│   ├── ANTI_PATTERNS.yaml
│   ├── KNOWLEDGE_BASE.md
│   └── SYNTHESIS_AUDIT.md
├── 12_model/
│   ├── ONTOLOGY.yaml
│   ├── ENTITY_MODEL.yaml
│   ├── RELATION_MODEL.yaml
│   ├── STATE_MODEL.yaml
│   ├── EVENT_MODEL.yaml
│   ├── SIGNAL_MODEL.yaml
│   └── ARTIFACT_MODEL.yaml
├── 13_logic/
│   ├── DECISION_RULES.yaml
│   ├── DIAGNOSTICS.yaml
│   ├── POLICIES.yaml
│   ├── ESCALATION_RULES.yaml
│   ├── STOPPING_RULES.yaml
│   ├── PRIORITIZATION_RULES.yaml
│   └── FEEDBACK_LOOPS.yaml
├── 14_architecture/
│   ├── OS_MANIFEST.yaml
│   ├── CAPABILITIES.yaml
│   ├── CAPABILITY_GRAPH.yaml
│   ├── MODULE_MAP.yaml
│   ├── DEPENDENCY_MAP.yaml
│   └── ARCHITECTURE.md
├── 15_interfaces/
│   ├── COMMANDS.yaml
│   ├── COMMAND_REFERENCE.md
│   ├── WORKFLOW_MAP.yaml
│   ├── WORKFLOWS/
│   ├── INPUT_OUTPUT_CONTRACTS.yaml
│   └── ERROR_MODEL.yaml
├── 16_runtime_design/
│   ├── AGENT_ROSTER.yaml
│   ├── AGENTS/
│   ├── SKILLS/
│   ├── TOOLS.yaml
│   ├── PROMPTS/
│   ├── MEMORY_MODEL.yaml
│   ├── CONTEXT_POLICY.yaml
│   └── ROUTING_POLICY.yaml
├── 17_handoffs/
│   ├── HANDOFF_REGISTRY.yaml
│   ├── HANDOFF_CONTRACTS/
│   ├── ROUTING_GRAPH.yaml
│   └── FAILURE_AND_FALLBACK.md
├── 18_implementation/
│   ├── runtime/
│   ├── commands/
│   ├── workflows/
│   ├── agents/
│   ├── skills/
│   ├── prompts/
│   ├── tools/
│   ├── memory/
│   ├── schemas/
│   ├── registry/
│   └── tests/
├── 19_validation/
├── 20_evals/
├── 21_audit/
├── 22_repair/
├── 23_docs/
├── 24_release/
│   └── dist/
└── 25_update/
```

The tree intentionally separates raw evidence, normalized knowledge, compiled logic, implementation, evaluation and release artifacts. This prevents the final OS from becoming a single untraceable document.
