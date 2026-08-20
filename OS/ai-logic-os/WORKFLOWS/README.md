# AI Logic {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`arbitrate-one-step.md`](arbitrate-one-step.md) | someone proposes a model call, or an existing model call looks like a rule | one step in one bin, with its falsifier and its cost comparison |
| [`challenge-an-agentic-system.md`](challenge-an-agentic-system.md) | a pipeline, agent or OS is expensive, unreliable or drifting | five cited findings, ranked by cost, with the first fix specified |
| [`triage-a-measured-process.md`](triage-a-measured-process.md) | a process has been mapped and measured, and someone wants it automated | every step binned, deletions first, one move specified |

A workflow that cannot state its completion test is not a workflow, it is a
habit. Each of these ends in a checkable artifact.
