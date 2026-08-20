# Habit Tracker {OS}: Weekly habit review

**Produces:** a computed review of the week (adherence, cue stability, recovery latency, trend), with confidence, named data gaps, and exactly one keep, change or stop decision per contract reviewed.
**Trigger:** the weekly cadence set at setup fires, or the user asks how they are doing, or three or more consecutive misses accumulate on one contract.
**Runs in:** `REVIEW`, handing to `ADAPT` or `RECOVER` when the decision is change or stop.
**Takes:** every log record in the review window, the contract versions in force during that window, the season, the capacity envelope from Health & Energy {OS}, and the previous review.

## Steps

1. Fix the window and list the contracts that were `ACTIVE` or `RECOVERING`
   inside it. A contract created mid-window is reviewed only over the days it
   existed.
2. Count the records. If a contract has too few to support a trend, say the
   count, report the metric with its confidence, and do not assert a pattern.
3. Compute from records, never from impression: adherence against target,
   adherence against the minimum viable version, cue stability, recovery
   latency after a miss, and direction of travel against the previous review.
4. Name the data gaps explicitly: days with no record, contracts with only
   `inferred` entries, device imports that disagree with the user's statement.
5. For each contract that is not being met, diagnose one barrier (capability,
   opportunity, reflective motivation, automatic motivation, overload,
   ambivalence, unknown). When the barrier is unknown, name the one observation
   that would discriminate rather than giving generic advice.
6. Record exactly one decision per contract: keep, change, or stop. A change
   goes to `ADAPT` as an experiment with a stopping rule. A stop moves the
   contract to `PAUSED` or `RETIRED`, with the user's agreement.
7. If the load exceeded the capacity envelope for most of the week, propose
   `RECOVER` and say which contracts survive the season.
8. Render one compact visual only when it makes the trend clearer than the
   numbers do.
9. Emit `habit.review.completed` to Mindset {OS} as evidence against the stated
   identity, and stage the review canonically through Context & Memory {OS}.
   Do not restate the identity or edit the goal.

## Completion test

Every reviewed contract carries a metric traceable to dated records, a stated
confidence, and exactly one recorded decision. Every day in the window is
either covered by a record or listed as a gap. The review event has been emitted
to Mindset {OS}.

## Failure

No records in the window: report zero coverage, do not produce a review, and
ask whether check-ins stopped or the habits did. Contract version changed
mid-window: report the two segments separately rather than averaging across a
changed target. A log record was corrected after a previous review: mark that
review invalidated and recompute. Health & Energy {OS} unavailable: run the
review, state that the capacity envelope is unknown, and defer any load
increase.
