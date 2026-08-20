# Pricing {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Price book](price-book.md) | a published offer has no price, or the book has drifted from evidence | a versioned price book where every line carries evidence |
| [Discount policy](discount-policy.md) | a price book is live with no policy, or sellers routinely operate outside the one that exists | the permitted range, the approval ladder, the floor and its named owner |
| [Price change plan](price-change-plan.md) | evidence indicates a price is wrong and a change is being executed | old and new price, revenue impact, grandfathering decision and approved customer text |
