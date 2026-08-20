# Social Intelligence {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`interaction-prep.md`](interaction-prep.md) | `/prep`, or a named conversation that has not happened yet | a one-page brief: aim, likely counter-aim, opening line, what to listen for, walk-away line |
| [`weekly-relational-debrief.md`](weekly-relational-debrief.md) | a fixed weekly slot, or `/debrief --week` | a debrief record per significant interaction, each naming what the prior read got wrong, sent to Journal {OS} on approval |

The pair is the learning loop. Prep produces a read before the interaction, the
debrief falsifies it afterwards, and only the second half makes the first half
better over time. Running prep without ever debriefing produces confident reads
that nothing ever corrects.
