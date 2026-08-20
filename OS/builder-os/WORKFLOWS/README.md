# Builder {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [step-transaction.md](step-transaction.md) | a READY step exists and there is capacity | one closed step with deterministic evidence |
| [resume-after-interruption.md](resume-after-interruption.md) | a session died, compacted or changed hands | reconciled state, and work continued without loss |
| [finalize-the-build.md](finalize-the-build.md) | every step at the target priority is DONE | BG01 to BG20 evaluated and a frozen engineering handoff |

A workflow is finished when its completion test passes, not when its last step
has been performed.
