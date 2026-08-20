# Money {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`monthly-close.md`](monthly-close.md) | the month has ended and intake is drained, or `/money-close <month>` | a closed month: in, out, left, surplus or deficit, reconciled per account, with `money.month.closed` and `money.surplus.verified` emitted |
| [`document-intake.md`](document-intake.md) | `/money-intake <path>`, or the close finds an account with no document | staged records carrying their source, confidence and period, plus a duplicate and gap report |
| [`major-purchase-decision.md`](major-purchase-decision.md) | `/money-decide "<decision>"`, or any spend at or above the operator's threshold | the effect on the month, the runway and the reserve contribution, stated in currency, with no verdict |

Intake feeds the close; the close feeds every forward-looking number. A decision
read run before a close is measured against an unreconciled month, which is why
that workflow refuses to run until one exists.
