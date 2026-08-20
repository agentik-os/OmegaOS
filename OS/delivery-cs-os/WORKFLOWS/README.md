# Delivery & Customer Success {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Post-payment handoff](post-payment-handoff.md) | Sales {OS} reports a closed won commitment | a promise register with a scope verdict per promise, and `handoff.accepted` after the gate clears |
| [Acceptance record](acceptance-record.md) | a deliverable or milestone is put forward for acceptance | an acceptance record carrying customer-produced evidence |
| [Renewal recommendation](renewal-recommendation.md) | a term approaches its end, or health drops before it does | a recommendation with its full signal set, sent to Revenue {OS} to decide |
