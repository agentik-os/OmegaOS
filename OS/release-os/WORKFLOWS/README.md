# Release {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [ship-a-release.md](ship-a-release.md) | a certified and cleared build is candidate for production | a recorded decision, a staged rollout and production verification |
| [roll-back.md](roll-back.md) | an abort criterion fired, or production verification failed | the previous known good state restored, with the data implications stated |
| [run-an-incident.md](run-an-incident.md) | production is degraded | containment, an incident record and a routed postmortem |

A workflow is finished when its completion test passes, not when its last step
has been performed.
