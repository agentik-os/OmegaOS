# KPI & Analytics {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`metric-selection.md`](metric-selection.md) | there are too many numbers, or none, or a new dashboard has been requested | a one-page metric set where every metric names the decision it can change, plus the stop-tracking list |
| [`definition-and-threshold.md`](definition-and-threshold.md) | a metric has been selected and is about to be reported | a definition two people compute identically, with its counting rules, its threshold and the decision that threshold triggers |
| [`reading-cycle.md`](reading-cycle.md) | the measurement cadence fires | values with movement judged against normal variation, breaches routed to their decision owners, and gaps reported as gaps |
| [`metric-audit.md`](metric-audit.md) | the review cycle for the metric set, or a set that has grown past one page | each metric kept, redefined or retired, on the evidence of the decisions it actually changed |
