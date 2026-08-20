# Operations {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`process-diagnosis.md`](process-diagnosis.md) | a repeating process is expensive, slow, error-prone, or about to be automated | a current-state map recognised by the people who run it, with a number or an explicit unknown on every step |
| [`simplification-ladder.md`](simplification-ladder.md) | a process has been mapped and measured | a decision per step in ladder order, and a target operating model reachable from today |
| [`automation-readiness.md`](automation-readiness.md) | a simplified target model exists and someone wants it automated | a ready, not ready, or ready-for-part verdict with its reasons, and the handoff packet for Automation {OS} |
