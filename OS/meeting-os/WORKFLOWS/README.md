# Meeting {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`meeting-triage.md`](meeting-triage.md) | someone proposes a meeting, or an invite arrives | a verdict of hold, shrink, async or decline, with its person-hour cost and, when declined, the asynchronous alternative |
| [`decision-meeting.md`](decision-meeting.md) | a meeting has passed triage and is scheduled | an agenda with one decision per item, a circulated pre-read, decisions taken, and action items with one owner and one date |
| [`recurring-audit.md`](recurring-audit.md) | a recurring meeting reaches its review date, or produces no decisions for three occurrences | a keep, shrink, merge or kill verdict backed by the decision and cost record |
