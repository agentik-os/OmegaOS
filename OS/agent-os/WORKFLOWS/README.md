# Agent {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`design-and-brief-an-agent.md`](design-and-brief-an-agent.md) | a recurring job needs judgment and someone is about to write a prompt | an agent design and a four block brief whose done test is a command |
| [`supervise-and-debrief-a-run.md`](supervise-and-debrief-a-run.md) | an agent is dispatched, or has just claimed it is finished | a four state supervision record, an independent verification, and a brief amendment |
| [`review-and-retire-the-roster.md`](review-and-retire-the-roster.md) | periodic review, or an agent nobody remembers using | a roster with owners and score trends, and clean retirements |

Two invariants cut across all three: a brief with an unfillable block is refused
rather than softened, and an agent's own claim of success is an input, never the
verdict.
