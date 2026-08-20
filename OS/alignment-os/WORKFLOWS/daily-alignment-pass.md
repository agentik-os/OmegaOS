# Alignment {OS}: Daily alignment pass

**Produces:** one dated pass record: state, chosen virtue, expected obstacle, rehearsed response, first action, and at day close the lesson and the release.
**Trigger:** the user runs `/morning` or `/evening`, or opens the OS within the first or last hour of their working day.
**Runs in:** `DAILY_PASS`.
**Takes:** the declared value set and its priority order (Context & Memory {OS}), yesterday's pass record if one exists, today's calendar or stated plan from the user, and habit evidence from Habit Tracker {OS} when installed.

## Steps

1. Read the declared value set. If none exists, stop and run `TRUE_NORTH`
   instead: a daily pass with no values is a mood check.
2. Ask for physical and emotional state in one line, and record it verbatim.
3. Ask for one specific thing that is actually true and good from the last
   twenty four hours. Reject a generic answer once, then accept what comes.
4. Read back the identity sentence for today in the form "today I practise being
   someone who ...", derived from the top declared value, and let the user
   correct it.
5. Have the user pick one virtue from wisdom, courage, justice, temperance.
6. Capture one meaningful outcome for the day, phrased as a result and not as an
   activity.
7. Name the obstacle most likely to interfere, then write the if-then response
   for it in the user's own words.
8. Name the first move and the execution window it starts in.
9. At day close, run the evening half: what held, where action fell below the
   user's own standard, what was never theirs to control, where they forced,
   where they avoided necessary effort, the lesson, the release.
10. Write exactly one adjustment for tomorrow, and ask before persisting the
    record.

## Completion test

The record contains a named virtue, a named obstacle, an if-then response, a
first action with its execution window, and at close exactly one written
adjustment for the next day. Zero or more than one adjustment fails the test.

## Failure

If the value set is missing, the pass does not run: the OS states that there is
nothing to align against and offers `TRUE_NORTH`. If the previous day's record
is absent, the evening half runs on the user's recollection and marks the
comparison as unverified. If the user refuses a step, that field is recorded as
declined and the pass continues; a declined first action ends the pass with the
refusal recorded, not with an invented action.
