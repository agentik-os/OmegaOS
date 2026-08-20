# Stepper {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [compile-the-graph.md](compile-the-graph.md) | a frozen Blueprint, and a Design handoff where there is UX | a validated step graph frozen at `BUILD READY` |
| [run-a-wave.md](run-a-wave.md) | a validated graph with a non-empty ready set | one safe wave of steps closed through the verifier |
| [repair-a-failing-step.md](repair-a-failing-step.md) | `done` failed on a claimed step | a passing step, or a bounded escalation to a human |

A workflow is finished when its completion test passes, not when its last step
has been performed.
