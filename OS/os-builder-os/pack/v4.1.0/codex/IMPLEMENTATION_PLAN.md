# Reference implementation plan

## Milestone 1: Repository discovery

- Locate existing Forge, Builder, Research and Librarian contracts.
- Identify duplicate or superseded Books {OS} naming and migrate to Librarian {OS}.
- Map current commands, schemas, workflows, tests and packaging.
- Write migration inventory.

## Milestone 2: Build contract and state

- Add `BUILD_CONTRACT.yaml` generation.
- Add resumable `BUILD_STATE.json`.
- Add decision log and milestone events.
- Test interruption and resume.

## Milestone 3: Corpus pipeline

- Implement bestseller and canonical discovery adapters.
- Implement coverage matrix and curation scoring.
- Implement retained and rejected corpus artifacts.
- Implement parallel `/book --deep` fanout.
- Block synthesis on incomplete retained-book analyses.

## Milestone 4: Evidence and claims

- Implement Research {OS} expansion handoff.
- Add source and claim ledgers.
- Add confidence, recency, applicability and contradiction fields.
- Add access-integrity checks.

## Milestone 5: Synthesis compiler

- Add concept normalization.
- Add school and contradiction map.
- Add synthesis statuses.
- Add principles and decision-rule compiler.
- Add source-to-rule traceability graph.

## Milestone 6: Architecture compiler

- Generate ontology and state model.
- Generate capability, command and workflow contracts.
- Generate optional typed inter-OS handoffs.
- Add graceful degradation.

## Milestone 7: Evaluation and repair

- Add deterministic validators.
- Add scenario and rubric evals.
- Add adversarial gauntlet.
- Add repair and regression loop.

## Milestone 8: Documentation and release

- Generate HOW_TO_USE.
- Generate full `/presentation-os` command reference.
- Validate documentation against manifest.
- Package versioned ZIP.
- Update registries and changelog.

## Milestone validation rule

After every milestone:

1. run relevant tests;
2. repair failures;
3. update `BUILD_STATE.json`;
4. update `DECISION_LOG.md`;
5. commit a scoped change or preserve a reviewable diff.
