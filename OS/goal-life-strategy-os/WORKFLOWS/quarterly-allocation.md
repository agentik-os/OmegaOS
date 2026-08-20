# Goal & Life Strategy {OS}: Quarterly allocation

**Produces:** the allocation ledger for the coming quarter (planned share of
time, attention, money and energy per life domain), the closing review of the
quarter that ended, and the not-doing list for the next ninety days.

**Trigger:** the last week of a calendar quarter, or the user says "the quarter
is over", "where did my time actually go", "I need to re-plan the next three
months", or a capacity change from Health & Energy {OS} invalidates the
current ledger.

**Runs in:** `ALLOCATION_REVIEW`, then `TRADEOFF` when the new plan overflows,
then back to `ALLOCATION_REVIEW` to write the ledger.

**Takes:** the previous quarter's allocation ledger, the active goal set and
horizon map (Context & Memory {OS}); actual spend evidence from Execution {OS}
and Habit Tracker {OS}; the capacity ceiling and any standing load veto from
Health & Energy {OS}; the value order from Alignment {OS}; decision records
from the quarter (Decision {OS}).

## Steps

1. Load the closing quarter's ledger and the active goal set. Print both. If
   either is missing, say which and stop before producing numbers.
2. Collect actual spend per life domain: hours per week and money per month.
   Take it from Execution {OS} and Habit Tracker {OS} where evidence exists,
   from the user otherwise, and label each figure as evidenced or reported.
3. Build the planned versus actual table, one row per domain, with the delta in
   hours and money and the sign of the delta.
4. Mark every row whose delta exceeds the declared tolerance. For each marked
   row, name one cause. A cause is an event or a claim, not a mood.
5. For each marked row, choose exactly one correction: change the plan, or
   change the behaviour. Write which. A divergence carried forward with no
   correction is recorded as an accepted overrun with an expiry date.
6. List every goal with no progress evidence at all this quarter. Flag each as
   unmeasured and name the evidence that would resolve it.
7. Ask, per unmeasured or stalled goal, whether it is retired. Route any yes to
   `/goal-retire`, which asks for human approval before writing.
8. Read the new capacity ceiling from Health & Energy {OS}. If it moved,
   discard the cached allocation percentages and recompute from the new
   ceiling.
9. Draft the next quarter's ledger: each active goal on the `now` horizon gets
   hours per week and money per month. Sum it.
10. If the sum exceeds the ceiling, stop and run `/tradeoff` for each
    overflowing claim, using the Alignment {OS} value order as the ranking
    rule. Record the loser. Do not scale every claim down proportionally.
11. Write the not-doing list: everything considered and refused this cycle,
    each with its reason. If the list is empty, say so and state that the plan
    is therefore likely to overrun.
12. Emit the review packet to Review & Governance {OS} and the updated
    allocation to Execution {OS} and Habit Tracker {OS} as the capacity each
    may consume.
13. Persist the new ledger, the tradeoff records and any retirement records
    through Context & Memory {OS}, after human approval where the boundary
    requires it.

## Completion test

The next quarter's ledger exists, its summed cost is at or under the capacity
ceiling, every domain has a planned figure for hours and money, every
divergence over tolerance from the closing quarter carries a named cause and
one correction, and the not-doing list has at least one entry or an explicit
statement that nothing was refused.

## Failure

- No previous ledger: skip the review half, say the quarter cannot be reviewed
  because there is no baseline, and run only the planning half. Label the
  output as a first cycle.
- No capacity ceiling from Health & Energy {OS}: ask the user for hours per
  week and money per month, label the ceiling as user-stated rather than
  measured, and stamp that label on the ledger.
- No actual spend evidence: report the quarter as unmeasured rather than
  estimating it. Name Execution {OS} and Habit Tracker {OS} as the missing
  sources and note that the next review will have the same gap unless they are
  connected.
- No value order from Alignment {OS}: run the tradeoffs on explicit user
  preference and record every one of them as unranked preference.
- The user refuses to cut anything when the plan overflows: write the ledger as
  over-ceiling, mark it over-ceiling on its face, and name which claims are
  expected to fail first.
