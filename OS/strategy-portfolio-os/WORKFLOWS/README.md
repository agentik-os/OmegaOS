# Strategy & Portfolio {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test. Each one runs one
of the six operating protocols shipped in the pack.

| Workflow | Trigger | Produces |
|---|---|---|
| [strategy-kernel.md](strategy-kernel.md) | an ambition is being treated as a strategy, with no stated obstacle behind it | the approved kernel: diagnosis of the critical challenge, a guiding policy that rules options out, coherent actions and their allocation |
| [ranked-portfolio.md](ranked-portfolio.md) | more candidates exist than the period can carry, or nobody can say how many things are actually running | the scored and ranked portfolio, every item with status, owner, cost and kill criteria, plus the not-doing list |
| [quarterly-allocation.md](quarterly-allocation.md) | a period is starting, or the constraint set changed enough that the current allocation no longer fits | the quarterly plan and the execution packet: outcomes, owners, allocation, signals and exclusions |
| [kill-review-verdict.md](kill-review-verdict.md) | a review trigger fired, a signpost was hit, a kill criterion was met, or a bet is limping | continue, narrow, pivot, pause or kill against the original thesis, with the learning captured and the resources released |

The two remaining protocols, scenario planning and the strategic decision memo,
are reached through the `/scenario` and `/strategic-decision` router commands and
feed these four: a triggered signpost opens a kill review, and a decision memo is
the record a funding or kill verdict is written into.
