# Revenue {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Invoice to cash](invoice-to-cash.md) | a deliverable is accepted, a milestone completes, or a billing period ends | an approved invoice, issued, then matched to received cash |
| [Receivables ageing and collections](receivables-ageing-and-collections.md) | the collections cadence fires, or an invoice passes its terms | an ageing view with exactly one recorded action per open invoice |
| [Renewal decision](renewal-decision.md) | a contract term approaches its end, or Delivery publishes a recommendation | the renewal decision, citing the delivery signals it accepted or overrode |
