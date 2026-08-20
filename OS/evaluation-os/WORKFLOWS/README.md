# Evaluation {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`build-a-rubric-and-eval-set.md`](build-a-rubric-and-eval-set.md) | quality is being argued about with no written standard | criteria two graders apply the same way, and a fixed versioned case set |
| [`run-a-regression-check.md`](run-a-regression-check.md) | a prompt, agent, index or pipeline changed | scores against the baseline, with regressions attributed to named cases |
| [`calibrate-a-grader.md`](calibrate-a-grader.md) | an automated grader is about to be trusted at scale | measured agreement with human labels, per criterion |

One invariant cuts across all three: no rubric, no score, and no score without
the four versions it is only meaningful against.
