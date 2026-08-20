# Workflow: Post-close obligations register

Record everything that survives the close, with a date, an owner and the
condition that ends it, and keep tracking until the last one expires.

**Mode:** `OBLIGATIONS`
**Produces:** the post-close obligations register and `exit.obligation.tracked`
**Typical duration:** opened on the day of close, runs for years

## Trigger

Any of:

- a transaction has closed, in any shape
- a partial sale or secondary has completed and the operator retains a position
- an obligation falls due within the reporting window
- a milestone, an escrow release or a covenant expiry date has passed without
  being marked

A closed transaction with an untracked earn-out is an unfinished transaction.
This workflow opens on the day of close, not later.

## Steps

1. **Read the executed agreement for surviving terms.** Read it for what it
   obliges, not for what was negotiated. Where a term's meaning is unclear, that
   is a question for the lawyer who owns the agreement, and it is recorded as an
   open question rather than resolved here.

2. **Enter every earn-out milestone.** The metric, the measurement period, who
   measures it, the payment that follows, and the date the measurement is
   determined. Record the operator's own tracking of the metric alongside the
   buyer's, because a discrepancy discovered at determination is worth less than
   one discovered during the period.

3. **Enter every escrow tranche.** Amount, release date, and what may be claimed
   against it before that date. Set the reminder ahead of the release date, not
   on it.

4. **Enter every transition service commitment.** What the operator owes, the
   hours or availability implied, the end date, and what happens on overrun.
   These are the obligations most often underestimated at signing.

5. **Enter every restrictive covenant.** Scope, geography, duration and expiry
   date. A non-compete or non-solicit constrains what the operator may build
   next, so its expiry date belongs in the register the same way a payment does.

6. **Enter every indemnity and warranty period.** What is covered, the cap, and
   the date the exposure ends.

7. **Assign an owner and a release condition to every line.** An obligation with
   no owner is an obligation nobody performs. An obligation with no release
   condition never ends, and the operator carries it indefinitely because nobody
   recorded what closes it.

8. **Emit `exit.obligation.tracked`** with the register contents, and re-emit
   whenever a line is added, satisfied or disputed.

9. **Report the window.** On the standing cadence, and on demand via
   `/exit-obligations --due 90d`, report what falls due, what is overdue, and
   what is disputed. An overdue milestone is escalated, not carried forward
   quietly.

10. **Close a line only on evidence.** A milestone is satisfied when the payment
    landed or the determination was issued, an escrow tranche when the funds
    released, a covenant when its expiry date passed. Mark the evidence. The OS
    does not mark a line satisfied on the strength of an expectation, and it
    never receives or moves the money itself.

11. **Route the money outward, not inward.** Proceeds and releases are facts for
    Wealth {OS} to place against reserves and goals, and for Capital {OS} to
    allocate. This OS records that they arrived; it does not hold them and does
    not decide where they go.

12. **Escalate disputes to counsel immediately.** A contested earn-out
    determination, a claim against escrow, or an alleged covenant breach is a
    legal matter with deadlines. The OS assembles the timeline and the evidence
    for the lawyer; the lawyer runs it.

## Completion test

The register is correct when all of the following hold:

- every surviving term in the executed agreement appears as a line
- every line carries a date, an owner and a release condition
- earn-out lines carry the metric, the measurer and the determination date
- escrow lines carry the amount, the release date and the claim conditions
- reminders are set ahead of each date, not on it
- satisfied lines carry the evidence that satisfied them
- open questions about a term's meaning sit with the lawyer, not resolved here
- `exit.obligation.tracked` has been emitted and reflects the current register

The register is complete, and only then, when every line has been satisfied or
has expired against its recorded condition.
