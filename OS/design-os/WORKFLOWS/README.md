# Design {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [compile-design-handoff.md](compile-design-handoff.md) | a frozen Blueprint handoff on a product with a UX surface | the design pack plus a validated `design-handoff.json` |
| [challenge-a-flow.md](challenge-a-flow.md) | a journey is expensive, risky or nobody has questioned it | a before and after path with every edge state named |
| [audit-an-existing-interface.md](audit-an-existing-interface.md) | an interface exists and its soundness is in question | ranked findings plus a repair handoff Stepper can plan |

A workflow is finished when its completion test passes, not when its last step
has been performed.
