# Workflow: Major purchase decision

Put a spending decision against the month, the runway and the reserve, and hand
the tradeoff back to the operator in currency.

**Trigger:** `/money-decide "<decision>"`, or any spend at or above the
threshold set at configure time. A recurring commitment (a new instalment, a
higher rent) triggers it regardless of its first month size.

**Owner:** the operator decides. This workflow produces no verdict.

## Steps

1. **State the decision in numbers.** One-off amount, or amount and cadence for
   a recurring commitment, plus the account it would leave from and the date.
   If the operator has a range, use the top of the range.
2. **Refuse to run on air.** If no month has been closed, say so and run
   `/money-close` first. A decision measured against an unreconciled month is
   measured against nothing.
3. **Effect on the month.** Recompute the current month's left, including
   obligations still to leave, with and without the decision.
4. **Effect on the runway.** Recompute months of cover, with and without, and
   show the burn and the balances used in both.
5. **Effect on the reserve.** Compare the resulting surplus against
   `wealth.reserve_target.set`. State how many months of reserve contribution
   the decision consumes, or whether it breaks the contribution entirely.
6. **Name the commitment tail.** For a recurring commitment, state the total
   over the next 12 months and whether it is cancellable, because a monthly
   figure understates what is being agreed to.
7. **Check the calendar.** Show what else is already dated in the 60 days around
   it, so two independent decisions are not made against the same money.
8. **Hand back the tradeoff.** Present the three deltas (month, runway, reserve)
   side by side. Recommend nothing. If the decision would take the runway below
   the reserve floor Wealth {OS} published, say that plainly as a fact, not as
   advice.
9. **Record the outcome.** If the operator proceeds, add it to the obligation
   register when recurring, and to the forecast when one-off. The OS records; it
   does not pay.

## Completion test

The read is complete when:

- the month, runway and reserve deltas are each stated in currency, with the
  inputs they used
- the 12 month total and cancellability of any recurring commitment are stated
- no verdict, recommendation or encouragement appears anywhere in the output
- if the operator proceeded, the commitment is in the register or the forecast
  and no payment was initiated by this OS
