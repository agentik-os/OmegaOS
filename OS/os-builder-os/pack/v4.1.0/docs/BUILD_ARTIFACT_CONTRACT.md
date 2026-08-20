# Build artifact contract

Every artifact created by Builder {OS} must have one clear owner, purpose and lifecycle stage.

## Artifact classes

1. **Control**: build state, assumptions, logs, decisions and manifests.
2. **Evidence**: source discovery, book analyses, research notes and ledgers.
3. **Knowledge**: claims, conflicts, synthesis, principles and mechanisms.
4. **System model**: ontology, states, rules and policies.
5. **Architecture**: capabilities, dependencies, commands and workflows.
6. **Runtime**: prompts, agents, skills, tools, memory and executable assets.
7. **Verification**: validation, evals, audit, repairs and regression baselines.
8. **Release**: documentation, package manifests, registry entries and distribution ZIP.

## Required metadata

Machine-readable artifacts should expose, directly or through `00_control/ARTIFACT_INDEX.json`:

```yaml
artifact_id: unique-id
phase: 00..25
owner: role-id
status: template|draft|validated|released
inputs: []
outputs_to: []
source_trace: []
schema: optional-schema-id
updated_at: ISO-8601
```

## Mutation rule

Agents may write only artifacts owned by their current phase unless a repair task explicitly authorizes upstream changes. All cross-phase modifications are recorded in `22_repair/CHANGESET.jsonl` and `00_control/DECISION_LOG.md`.

## Release rule

Only artifacts referenced by `24_release/RELEASE_MANIFEST.json` are part of the official release. Research artifacts can be included for provenance but are not automatically runtime dependencies.
