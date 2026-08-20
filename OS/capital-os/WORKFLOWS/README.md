# Capital {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Write the allocation policy](write-the-allocation-policy.md) | A new pool exists, a candidate arrived with no policy written, or a ceiling is being tested | A signed, versioned allocation policy in which every ceiling has a number and a consequence |
| [Approve a commitment amount](approve-a-commitment-amount.md) | A named commitment has a thesis, a diligence outcome and agreed terms, and needs a number | A signed allocation decision record: amount, reserve, ceilings tested, funding source, labelled assumptions |
| [Review pacing and concentration](review-pacing-and-concentration.md) | The period closes, a ceiling breach surfaces, a position is impaired, or the capital constraint moves | The period review with every policy line marked held, breached or amended, realised separated from unrealised |

The order they are usually run in: policy first, commitments against it, review
at period close, and the review feeds the next policy revision. An allocator who
starts at the second workflow is writing a justification, not a policy.
