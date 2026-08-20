# Wealth {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`balance-sheet-baseline.md`](balance-sheet-baseline.md) | `/wealth-baseline` on first run, or after a structural change (inheritance, separation, move, restructure, closed exit) | a dated balance sheet with a valuation basis per line, the unvalued kept at zero on their own list, and `wealth.networth.updated` emitted |
| [`reserve-sizing-and-funding.md`](reserve-sizing-and-funding.md) | `/wealth-reserve`, no reserve policy, a drawdown below target, or a change in the risk picture | a reserve target in months and currency, the liquidity verdict per holding, the gap, the refill rule, and `wealth.reserve_target.set` emitted |
| [`goal-funding-path.md`](goal-funding-path.md) | `/wealth-goal "<goal>"`, or a change in a goal's target, horizon or the verified surplus | required monthly contribution, a funded or short verdict, the quantified levers, the dated liquidity need, and `wealth.goal.funded_path` emitted |

They run in that order on a first pass. The baseline gives the position, the
reserve claims the money that must never be at risk, and only what survives both
is available to fund a goal. Running the goal workflow first produces a funding
path built on money the reserve was going to need.
