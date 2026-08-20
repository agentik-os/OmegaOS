# Health & Energy {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`daily-readiness.md`](daily-readiness.md) | The morning cadence fires, the user asks what they can carry today, or another OS requests a current envelope | Today's capacity assessment (level, limiting factor, constraints, validity window, confidence) and the constraint delivered to the units that schedule work |
| [`weekly-capacity-review.md`](weekly-capacity-review.md) | The weekly cadence fires, readiness stays below threshold for several days, or a plan hits its review trigger | The coming week's capacity envelope as `health.capacity.assessed`, and one keep, change or stop decision per active plan and experiment |

The daily pass feeds the weekly one: the review is computed from the readiness
assessments the daily pass recorded, and a week with fewer than three
assessments carries the previous envelope forward rather than revising it.

The safety gate runs first in both. A red flag ends the workflow, produces the
routing to a qualified human professional, and produces nothing else.
