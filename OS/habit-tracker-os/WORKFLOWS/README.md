# Habit Tracker {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`daily-check-in.md`](daily-check-in.md) | The user reports on the day ("done", "half of it", "missed"), or the evening cadence fires | One dated, provenance-labelled log record per contract due today, plus the next action |
| [`weekly-habit-review.md`](weekly-habit-review.md) | The weekly cadence fires, the user asks how they are doing, or three consecutive misses accumulate on one contract | A computed review with confidence and named data gaps, and one keep, change or stop decision per contract |

The daily loop feeds the weekly one: the review is computed from the records
the check-in wrote, and a review with no records underneath it is reported as
zero coverage rather than produced.
