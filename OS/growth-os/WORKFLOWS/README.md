# Growth {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [loop-map](loop-map.md) | the operator asks how to grow, or the offer, price or delivery model changed | every candidate classified as a loop or a funnel, with cycle time, coefficient, cost and a retained-cohort score |
| [pre-registered-experiment](pre-registered-experiment.md) | a backlog entry reaches the top and is selected | a frozen design, a change applied by its owning OS, and a verdict read against the threshold fixed beforehand |
| [channel-verdict](channel-verdict.md) | a retention window closes, a spend ceiling is hit, or the cadence review reaches the channel | scale, hold or kill, with cost per retained cohort and a spend ceiling or a revisit condition |
