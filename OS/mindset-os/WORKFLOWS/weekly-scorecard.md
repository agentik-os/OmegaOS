# Mindset {OS}: Weekly scorecard

**Produces:** a validated summary of one closed week, plus the single keystone adjustment it justifies.
**Trigger:** the week boundary the user chose has passed, or the user says "score my week", or a filled `weekly/WEEK-<n>.json` appears in a challenge workspace.
**Runs in:** `SCORE`, then `COACH` for the adjustment.
**Takes:** the weekly scorecard JSON (`04_WEEKLY_SCORECARD.json`, or `weekly/WEEK-<n>.json`), the identity ledger, the `NOT NOW` list, and behaviour evidence from Habit Tracker {OS} when it is present.

## Steps

1. Fill the scorecard for the closed week: a 0 to 10 score per state domain,
   the execution counts, the promise counts (kept, repaired, avoided), and the
   Rohn metrics. Leave nothing blank; an unfilled field is a validation error,
   not a zero.
2. Run `omega-mindset score <path>`. Read the printed summary. If it errors,
   fix the named field and run it again. Do not summarize a week by hand.
3. Read the lowest state domain out loud. That domain, not the lowest promise
   count, is what the week is about.
4. Compare the promise kept-or-repaired rate with the same rate from the
   previous week. A repaired promise counts; a repair is the system working, not
   a failure hidden.
5. For every avoided promise, write the system reason in the identity ledger:
   the cue that was missing, the friction that was present, or the load that was
   too high. A character reason ("I was lazy") is rejected and rewritten.
6. Check the standing identity statements against the week. Each statement gets
   one dated line: evidence for, evidence against, or no evidence this week.
7. Choose exactly one keystone adjustment for the coming week. Everything else
   that came up goes to the `NOT NOW` list, with the date it was parked.
8. Record the adjustment and the first action inside 24 hours in the daily card.

## Completion test

`omega-mindset score <path>` returned a summary with no validation error; the
identity ledger has one dated line per standing identity statement for this
week; exactly one keystone adjustment is recorded; and every item that was
considered and not chosen appears in `NOT NOW` with a date.

## Failure

If the scorecard file is missing, the workflow stops and says which week is
unscored. It does not reconstruct a week from memory. If the file exists but
fails validation, it reports the exact failing field and its expected range and
produces no summary. If Habit Tracker {OS} evidence is unavailable, it runs on
the self-reported counts alone and marks the summary as self-reported, which is
weaker evidence and is labelled as such.
