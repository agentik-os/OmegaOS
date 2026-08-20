# Context & Memory {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`capture-and-verify-a-record.md`](capture-and-verify-a-record.md) | a fact, decision or file arrives that should outlive the session | a verified canonical record with full provenance, or a staged record with the gap named |
| [`compile-a-context-pack.md`](compile-a-context-pack.md) | another OS starts work and states a purpose | the minimum sufficient context pack for that purpose, with what was withheld |
| [`resolve-a-contradiction.md`](resolve-a-contradiction.md) | two records disagree, or an entity is ambiguous | an adjudicated contradiction with the superseded record kept |

Two invariants cut across all three: nothing becomes canonical without
provenance, and no credential or secret enters any tier at any point.
