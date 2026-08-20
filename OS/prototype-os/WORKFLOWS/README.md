# Prototype {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [answer-the-riskiest-question.md](answer-the-riskiest-question.md) | several open questions block a decision | one falsifiable question, its evidence and its verdict |
| [technical-spike.md](technical-spike.md) | the plan depends on an unproven technical approach | a feasibility answer with evidence, and the artifact removed |
| [teardown-and-record.md](teardown-and-record.md) | a prototype's question is answered, or its expiry passed | the verdict written and the artifact gone |

A workflow is finished when its completion test passes, not when its last step
has been performed.
