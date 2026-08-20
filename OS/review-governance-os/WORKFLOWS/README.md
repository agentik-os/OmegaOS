# Review & Governance {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`review-cadence.md`](review-cadence.md) | the daily, weekly, monthly or quarterly cycle closes | a review record whose output is decisions with owners and dates, not a summary |
| [`postmortem.md`](postmortem.md) | an incident or a material failure | a blameless account of sequence and contributing conditions, ending in one change with an owner and a verification test |
| [`change-authorisation.md`](change-authorisation.md) | a domain OS proposes a change to its own boundary, policy or controls | an approve, reject or defer decision with conditions, a reversal path and a verification test, taken by someone other than the proposer |
| [`change-verification.md`](change-verification.md) | the verification date of an approved change arrives | evidence of whether the change did what it claimed, and a standardise, adjust or revert verdict |
