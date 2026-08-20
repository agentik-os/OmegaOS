# Codex installation prompt

Copy this prompt into the Agentik OS Forge repository with this package available:

```text
Upgrade Builder {OS} to v4.1.0 using this package as the normative specification.

Read, in order:
1. AGENTS.md
2. MASTER_PROMPT.md
3. OS_DEFINITION.md
4. docs/CANONICAL_PIPELINE.md
5. docs/RESEARCH_AND_CORPUS_PROTOCOL.md
6. docs/KNOWLEDGE_SYNTHESIS_PROTOCOL.md
7. docs/OS_ARCHITECTURE_STANDARD.md
8. docs/QUALITY_GATES.md
9. workflows/os-build-ultimate.yaml
10. schemas and registries

First inspect the existing Forge repository and map current Builder, Research, Librarian, Quality, Documentation and Release components.

Then:
- create a migration plan;
- preserve compatible existing behavior;
- replace direct name-to-OS generation with the research-first pipeline;
- implement durable build state and source-of-truth artifacts;
- implement book discovery, retained-corpus curation and per-retained-book deep-analysis fanout;
- implement non-book evidence expansion;
- implement claim, contradiction and synthesis artifacts;
- implement source-to-rule traceability;
- implement architecture, eval, red-team, repair, documentation and packaging gates;
- update commands, schemas, manifests, registries, docs and tests;
- run validations after every milestone;
- fix failures immediately;
- produce a migration report, eval report, audit report and versioned Builder {OS} package.

Do not ask basic questions. Infer sensible defaults and record assumptions. Do not stop at a plan. Complete the repository changes and tests.
```
