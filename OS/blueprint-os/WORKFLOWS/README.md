# Blueprint {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [compile-definition-pack.md](compile-definition-pack.md) | an idea plus context, no prior pack | a validated pack and a frozen handoff |
| [recover-canonical-truth.md](recover-canonical-truth.md) | a project exists, its definition is scattered or stale | one canonical pack rebuilt from prior sources |
| [revise-and-propagate.md](revise-and-propagate.md) | a decision changed after a handoff was frozen | a new version, a delta, and an updated impact set |

A workflow is finished when its completion test passes, not when its last step
has been performed.
