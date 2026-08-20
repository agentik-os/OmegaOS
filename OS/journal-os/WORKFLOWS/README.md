# Journal {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`daily-entry.md`](daily-entry.md) | `/journal`, `/entry`, or an end-of-day nudge the user asked for | one stored entry for today with its cross-OS context attached |
| [`monthly-pattern-extract.md`](monthly-pattern-extract.md) | first working day of a month, or `/pattern --since <date>` | candidate patterns with supporting entries, contradicting entries, n, and the OS that would own each |

The two form the loop this OS exists for: capture costs nothing daily, and once
a month the accumulated entries are turned into proposals that Mindset {OS},
Alignment {OS} or Goal & Life Strategy {OS} decide on. Journal runs both halves
and adopts nothing itself.
