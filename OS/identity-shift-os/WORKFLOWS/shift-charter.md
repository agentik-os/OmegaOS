# Identity Shift {OS}: Shift charter

**Produces:** the charter for one bounded shift: named current identity, named target identity, entry baseline, falsifiable exit test, review cadence, close-by date, and the single behaviour that carries it.
**Trigger:** the user names a from and a to and a reason the change is bounded in time, or `SCOPE` concludes that the request is a real shift.
**Runs in:** `CHARTER`.
**Takes:** the current identity model and belief ledger from Mindset {OS}, the value set from Alignment {OS}, the user's own words for the target identity, and the event or deadline that bounds it.

## Steps

1. Read the current identity model from Mindset {OS} and quote the starting
   identity into the charter verbatim. If Mindset holds no model, stop here and
   send the user there; a shift from an unnamed starting point cannot be
   reviewed.
2. Write the target identity as the user says it, then rewrite it as behaviours
   under conditions. "A founder who sells" becomes "when a prospect asks for a
   discount, I hold the price and explain the value" plus one or two more.
   Charter the rewritten form.
3. Check every target statement against the value set from Alignment {OS}.
   Escalate any conflict to the user as a conflict. Do not rewrite the value and
   do not silently drop the statement.
4. Write the exit test before anything else is agreed. It must be checkable by
   an observer who is not the user: an action taken, a role held, a request
   refused, a thing shipped, a number reached by behaviour rather than by mood.
5. Set the close-by date. It is a real calendar date, and it is the date the
   exit test will be applied whether or not the shift is going well.
6. Record the entry baseline: what is true today, dated, expressed in the exact
   terms the exit test uses, so the close has something to compare against.
7. Name the ONE behaviour that carries the shift, with a floor version that
   survives a bad week. Hand it to Habit Tracker {OS} as a contract tagged with
   the shift id. Everything else the user wants to add is parked.
8. Set the review cadence, weekly by default, and put the close-by date into it
   as the final review.
9. Get explicit approval to open the shift, then persist the charter through
   Context & Memory {OS} with a stable shift id and status `open`.

## Completion test

A charter exists with a shift id and status `open`; it quotes a starting
identity read from Mindset {OS}; every target statement names a condition and an
observable action; the exit test is stated in terms a third party could check;
there is a calendar close-by date; the entry baseline is dated; exactly one
behaviour contract exists in Habit Tracker {OS} tagged with the shift id; and no
other shift is open.

## Failure

If Mindset {OS} has no current identity model, the workflow refuses to charter
and routes the user there. If the exit test cannot be made checkable by a third
party, it refuses and says so plainly rather than accepting a softer test. If
another shift is already open, it stops and asks the user to close or explicitly
replace it. If the value set from Alignment {OS} is unavailable, step 3 is
skipped and recorded as skipped in the charter, so the first review knows the
check was never run.
