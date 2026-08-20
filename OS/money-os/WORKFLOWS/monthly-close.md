# Workflow: Monthly close

Turn a finished month into a fact that Wealth {OS} and an accountant can both
build on.

**Trigger:** the month has ended and `/money-intake` has been run for every
account that had activity. Also triggered by `/money-close <month>` on an older
month that was never closed.

**Owner:** the operator. The OS reconciles and reports; it never absorbs a
difference to make a close succeed.

## Steps

1. **Drain intake.** List every personal account. For each, confirm a statement
   or export covering the full month has been staged. An account with no
   document is named, and the close does not start.
2. **Work the staged queue.** Run classification until nothing older than the
   month end is unclassified. Lines the operator cannot identify stay
   unclassified on purpose; they are not distributed across categories.
3. **Detect duplicates.** Compare staged lines against already verified records
   by account, date, amount and running balance. Suspected duplicates are listed
   and the operator rules on each. Nothing is dropped silently.
4. **Promote to verified.** Each staged line the operator confirms becomes a
   verified transaction carrying its source document and the date it was seen.
   This is an approval step, not an automatic one.
5. **Reconcile each account.** Compare the computed closing balance against the
   statement closing balance. Any gap is reported per account, in currency and
   direction. A gap stops the close until it is explained or explicitly recorded
   as an accepted discrepancy with a reason.
6. **Accept the owner distribution.** If Revenue {OS} has emitted
   `revenue.owner_distribution.verified` for the period, record it as personal
   income. Refuse every other business fact offered.
7. **Compute the month.** In, out, left, surplus or deficit, by category, with
   the unclassified residue shown as its own line and its own amount.
8. **Test the reserve.** Compare the surplus against `wealth.reserve_target.set`
   from Wealth {OS}. State whether the month funded the reserve, in currency,
   and by how much it missed if it did not.
9. **Publish.** Emit `money.month.closed` and `money.surplus.verified`. Write
   the verified records to Context & Memory {OS}.
10. **Refresh forward view.** Recompute runway and the 90 day obligation
    calendar off the new closed month, and raise anything dated inside 30 days
    into Execution {OS}.
11. **Park the questions.** Anything touching deductibility, tax treatment or a
    filing goes into the accountant pack with its records attached, unanswered.

## Completion test

The month is closed when all of the following hold:

- every personal account reconciles to its statement closing balance, or carries
  a recorded discrepancy with a stated reason and amount
- every verified transaction in the month has a source document
- the unclassified residue is a named line with an amount, not zero by
  distribution
- `money.month.closed` and `money.surplus.verified` have been emitted
- the close states, in currency, whether the reserve target was funded

If any one of these fails, the month is not closed and is not reported as
closed.
