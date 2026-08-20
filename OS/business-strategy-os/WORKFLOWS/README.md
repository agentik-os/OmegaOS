# Business Strategy {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`asset-thesis-drafting.md`](asset-thesis-drafting.md) | first run, or the business changed materially, or the thesis is over a year old | the asset thesis, or the finding that the operator owns a job and which dependencies make it one |
| [`owner-dependence-audit.md`](owner-dependence-audit.md) | the thesis was just published, the cadence elapsed, or the operator is the bottleneck | the owner-dependence assessment by decision class, with named remediation |
| [`value-driver-review.md`](value-driver-review.md) | the cadence elapsed, an upstream metric was re-measured, or a buyer conversation is coming | the value driver table and the readiness gap, with every unverified input named |

Run them in that order on a first pass. The thesis defines what is being
measured, the dependence audit finds what is stuck to the owner, and the driver
review turns both into a gap against a standard the operator states.
