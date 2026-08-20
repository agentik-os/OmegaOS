# Orchestration {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`shape-and-run-a-mission.md`](shape-and-run-a-mission.md) | a mission with several parts that could run at once | a justified topology, a persisted ledger, verified tasks and one synthesis |
| [`recover-a-failed-mission.md`](recover-a-failed-mission.md) | a step failed, a budget overran, or the session was interrupted | a resumed or honestly closed mission, and the shape change the failure earned |
| [`close-a-mission-cleanly.md`](close-a-mission-cleanly.md) | every ledger entry is done, or the mission must end without that | an honest signal, released claims, and nothing left running |

One invariant cuts across all three: a delegate's claim of done is an input, and
the closure signal is honest even when honesty is inconvenient.
