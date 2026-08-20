# Ownership {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`entity-map-build.md`](entity-map-build.md) | first run, or an entity is created, acquired, dissolved or discovered | the entity map, the cap tables with their denominators, and the document request list |
| [`cap-table-reconcile.md`](cap-table-reconcile.md) | a round or transfer closes, due diligence opens, or the quarterly cadence | a line-level diff against the source and statutory registers, with every difference resolved or raised |
| [`transaction-consent-map.md`](transaction-consent-map.md) | `exit.structure.proposed`, or any contemplated issuance, transfer or partner admission | who must sign or waive, the clause behind each requirement, and a counsel pack to confirm it |

Every one of these ends at an approval gate. None of them writes to the
canonical register, emits to another OS, or contacts a third party before the
user decides, and none of them performs an act reserved to a lawyer, corporate
secretary, accountant or tax professional.
