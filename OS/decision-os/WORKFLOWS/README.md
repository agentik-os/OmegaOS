# Decision {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`decision-record.md`](decision-record.md) | `/decide` or `/decision-record`, or a framed call reaching its deadline or evidence threshold | the decision record: question, choice, discarded options, evidence, signal weight, reversibility class, review trigger |
| [`decision-review.md`](decision-review.md) | the review trigger fires, the outcome lands early, or `/decision-review <record>` | one appended verdict (held, wrong as predicted, wrong for an unpredicted reason, still open) with its lesson |

The two are one loop. A record written without a review trigger cannot be
reviewed, so `decision-record.md` refuses to close without one, and
`decision-review.md` appends to that record and never rewrites it.
