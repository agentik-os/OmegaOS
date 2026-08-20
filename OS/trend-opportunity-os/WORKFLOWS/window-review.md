# Workflow: Window review

**Produces:** a dated review record for every open watch and opportunity, and a
retirement record for each window that has closed.

## Trigger

A watch reaches its cadence, an opportunity reaches its review date, an expiry
date passes, or a closing condition is observed firing.

## Steps

1. **List what is due.** Every watch past its cadence and every opportunity at
   or past its review date. Anything overdue is reported as overdue with the
   number of days, not silently swept today and dated today.
2. **Sweep the sources.** Capture new signals with their real observation dates.
   Where a source returned nothing, record the null with the date it was swept.
   A null is a result, and a watch with three consecutive nulls is a candidate
   for either new sources or retirement.
3. **Re-run the confirmation test** on every candidate that gained signals. A
   candidate that has been "almost confirmed" for three reviews is usually a
   watch with the wrong sources, and that is the finding to report.
4. **Check each open opportunity's closing condition** against the new signals.
   Did the dominant entrant appear, the platform close, the price floor land,
   the behaviour become default. Record the answer with its dated evidence,
   including the answer "no change observed".
5. **Check reversal.** Look specifically for signals that the confirmed movement
   has stalled or turned. If it has, write a dated reversal record and notify
   everyone who consumed `trend.movement.confirmed`. A reversal is reported as
   loudly as the confirmation was.
6. **Restate, narrow, widen or close each opportunity.** Restating requires
   fresh dated evidence: an opportunity re-affirmed on the same evidence as last
   quarter has not been reviewed, it has been copied. Narrowing and widening
   both require a stated reason and a changed expiry.
7. **Retire what has closed.** For each closed window write: what closed, on
   what date, which closing condition fired (or that the expiry simply passed),
   what was learned, and which units were betting on it. Retiring an opportunity
   another OS is already betting on requires approval before the record is
   written.
8. **Emit `opportunity.window.closed`** for each retirement, to Strategy &
   Portfolio {OS}, so a funded bet resting on a closed window reaches a kill
   review instead of quietly continuing.
9. **Reset the calendar.** Set the next cadence date for every watch and the
   next review date for every surviving opportunity before the session ends. A
   review that does not schedule the next one is how a watchlist dies.
10. **Report the board.** What is confirmed, what is still one observation, what
    is retired this cycle, and what expires before the next review.

## Completion test

- Every watch and opportunity that was due has a dated review record.
- Overdue items are reported as overdue, with the number of days.
- Null sweeps are recorded with their dates rather than omitted.
- Every open opportunity's closing condition was checked against new evidence,
  and the answer is dated.
- A reversal check was run explicitly, and its result recorded.
- Any opportunity restated as still open cites evidence newer than the previous
  review.
- Every closed window has a retirement record naming the condition that fired or
  the expiry that passed, and `opportunity.window.closed` was emitted.
- Retirements affecting a bet another OS holds carry an approval record.
- Every surviving watch and opportunity has a next date on the calendar.
