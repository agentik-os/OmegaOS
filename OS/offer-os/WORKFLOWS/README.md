# Offer {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Offer definition](offer-definition.md) | a new thing is about to be sold, or a frozen offer must be redefined | a versioned offer definition with scope, exclusions, guarantee and proof |
| [Guarantee record](guarantee-record.md) | a guarantee is added or changed, or the fulfilment cost model moved | a guarantee with a modelled worst case cost and a withdrawal ceiling |
| [Offer retirement record](offer-retirement-record.md) | an offer is discontinued or replaced, or its economics failed | a retirement record with a named destination for every live customer |
