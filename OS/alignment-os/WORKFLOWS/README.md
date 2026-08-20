# Alignment {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`daily-alignment-pass.md`](daily-alignment-pass.md) | `/morning` or `/evening`, or opening the OS in the first or last hour of the working day | a dated pass record: state, virtue, obstacle, rehearsed response, first action, and at close one adjustment |
| [`weekly-values-audit.md`](weekly-values-audit.md) | `/weekly`, or a week closing with a declared value set and no audit for that week | a per-value verdict (matched, drifted, unmeasured) with cited evidence, one governing principle, and any drift alerts to neighbouring units |

Both workflows require a declared value set. If none exists, they stop and route
to `TRUE_NORTH` rather than grading behaviour against nothing.
