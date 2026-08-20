# Performance council

Judge a cycle of published content against the job each asset was given, and
route the learning to the units that own the next decision.

## Trigger

A cycle closes, the operator runs `/content-review`, or a pillar's measurement
window ends.

## Steps

1. **The OS assembles the cycle's published assets** from their publication
   records, each with its stated job, its surface and its publication time.
2. **The OS pulls platform metrics** as a cache entry with a retrieval
   timestamp. Missing metrics are named; nothing is inferred to fill a gap.
3. **The OS pulls the canonical metric definitions** from KPI & Analytics {OS}
   and states which it is using.
4. **For each asset the OS judges against its stated job**, not against
   whichever metric moved. An asset that performed well at a job it was not
   given is recorded as a finding, with both the intended job and the observed
   effect.
5. **The OS separates packaging from substance.** An asset with high reach and
   no downstream signal indicates packaging that earned attention the substance
   did not hold, and is labelled as such rather than as a success.
6. **The OS checks the cascade hypothesis**: did the native packages behave
   differently from each other, and did any behave identically, which would
   indicate they were crops rather than adaptations.
7. **The OS produces the council verdict per asset**: worked, did not work, or
   unreadable, each with the evidence and what would change the reading.
8. **The OS routes the learning**: `content.performance.feedback` to Storyteller
   {OS} for story-object learning only, per-asset performance to Growth {OS}
   and KPI & Analytics {OS}, `content.intent.qualified` to Sales {OS} for reader
   signals worth a commercial follow-up.
9. **The OS updates the content GPS** where the cycle contradicts a pillar's
   premise, and flags the contradiction rather than quietly editing the pillar.

## Completion test

Every published asset in the cycle has a verdict of worked, did not work, or
unreadable. Each verdict names the asset's stated job, the metric definition
used with its source in KPI & Analytics {OS}, and what evidence would change
the reading.

Every routed handoff exists: a `content.performance.feedback` record for each
asset carrying a story object, a performance record for Growth {OS}, and a
`content.intent.qualified` record for each reader signal passed to Sales {OS}.

A cycle in which any asset is scored without reference to its stated job fails
this test.

## Failure and abort

- **Platform metrics unavailable:** mark those assets unreadable, judge the
  rest, and never infer performance from a neighbouring asset.
- **A local number disagrees with the KPI & Analytics definition:** report both
  and the definition, escalate, do not reconcile silently.
- **An asset has no stated job:** it cannot be judged. Record it as unjudgeable
  and fix the calendar process that let an unjobbed asset publish.
- **Measurement window still open:** mark the verdict provisional with the date
  it becomes final.
- **Storyteller feedback would read as narrative direction:** strip it back to
  performance facts. Content never tells Storyteller what the story should be.
- **The cycle contradicts a content GPS pillar:** flag the contradiction to the
  operator and to Positioning {OS} where the claim is implicated. Do not edit
  the pillar silently to match the result.
