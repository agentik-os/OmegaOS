# Deal Flow {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Screen an inbound opportunity](screen-an-inbound-opportunity.md) | anything arrives that could become a deal | a recorded qualify, pass or abstain inside a time budget, with the reason attached to its source |
| [Run the weekly funnel sweep](run-the-weekly-funnel-sweep.md) | a fixed weekly slot | an honest funnel: live items with next actions, stalled items named, channels with dated next touches |
