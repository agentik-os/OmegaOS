# Workflow: definition and threshold

Produces a metric definition that two people compute identically, and a
threshold whose decision was agreed before anybody saw the number.

## Trigger

A metric has been selected and is about to be reported for the first time, or an
existing metric has produced a dispute about what it means.

## Inputs

- The metric and the decision it serves.
- The candidate source systems, and which is authoritative for this quantity.
- The population: who is counted and who is not.
- History, if any, and the definition that produced it.

## Steps

1. **Write the question the metric answers,** in one sentence, before the
   formula. Formula-first definitions measure what is easy rather than what was
   asked.
2. **Write the formula,** with the numerator and denominator fully specified.
3. **Name the authoritative source.** One system is the truth for this quantity.
   If two exist, pick one and record the other as secondary.
4. **Write the counting rules explicitly:** refunds, cancellations, trials,
   internal users, tests, duplicates, partial periods, and time zone. This is
   where nearly every metric dispute actually originates.
5. **Set the period and the cadence.** The cadence at which the number can
   genuinely move, which is often slower than the cadence somebody wants a
   report.
6. **Establish normal variation** from history where it exists: the range this
   metric moves in when nothing meaningful has happened. Without it, every
   fluctuation becomes a story.
7. **Set the threshold and the decision** it triggers, and name the person who
   owns that decision. Agreed now, before the first reading.
8. **Test the definition.** Have two people compute it independently from the
   written definition. If they differ, the definition is not finished.
9. **Version it,** date it, and publish it through Documentation {OS} with an
   owner and a review date.
10. **If the data cannot yet support it,** mark the metric not yet computable and
    issue the instrumentation requirement to Automation {OS}. Do not estimate.

## Completion test

- The definition states the question, the formula, the source, the filters, the
  period and the cadence.
- Counting rules cover refunds, cancellations, trials, internal users,
  duplicates and partial periods.
- Two people computed the same number independently from the written definition.
- The threshold, its decision and the decision owner are recorded before the
  first reading.
- Normal variation is stated, or explicitly unknown pending history.
- The definition is versioned, dated, owned and published.

## Failure paths

| Situation | Response |
|---|---|
| the two independent computations differ | find the rule they disagreed on and write it down; that rule was the actual definition all along |
| history exists under an older definition | keep both, mark the changeover date, and never silently recompute the past |
| nobody will commit to a threshold | the metric is not decision-driving yet; record it as informational and revisit |
| the authoritative source contradicts the accounts | the accounts win; suspend the metric until it is reconciled |
