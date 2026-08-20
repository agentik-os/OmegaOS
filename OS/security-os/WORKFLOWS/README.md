# Security {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [clear-a-release.md](clear-a-release.md) | Quality & Evaluation {OS} issued its verdict on a pinned build | the security clearance with its conditions and untested surface |
| [threat-model-a-system.md](threat-model-a-system.md) | a new system, or an architecture change to an existing one | assets, actors, entry points, trust boundaries and abuse cases |
| [handle-a-live-secret.md](handle-a-live-secret.md) | a credential is found in code, logs, history or an artifact | rotation, blast-radius assessment and a restricted record |

A workflow is finished when its completion test passes, not when its last step
has been performed.
