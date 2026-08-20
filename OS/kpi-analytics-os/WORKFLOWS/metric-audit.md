# Workflow: the metric audit

Produces a decision on every metric in the set, on the evidence of the decisions
it actually changed.

## Trigger

The review cycle for the metric set fires, or the set has grown past one page.

## Inputs

- The metric set with definitions, thresholds and owners.
- The readings and the breaches over the review period.
- The decisions actually taken, from Review & Governance {OS}, Project {OS},
  Operations {OS} and Client {OS}.
- The time cost of collecting and reporting each metric.

## Steps

1. **For each metric, count the decisions it changed** during the period. Not
   the times it was reported. The times somebody did something different because
   of it.
2. **Count the breaches and what followed each one.** A threshold crossed with
   no decision taken means either the threshold is wrong or the decision was
   never real.
3. **Cost each metric.** Collection time, reporting time, and the attention it
   takes in every review where it appears.
4. **Classify:**
   - Keep: it changed a decision, or it is a leading indicator whose value is
     precisely that it has not moved.
   - Redefine: it changed decisions but produces disputes, or its counting rules
     no longer match the business.
   - Retire: it changed nothing, and would change nothing at any value.
   - Externally required: keep, but out of the decision set, marked as reported.
5. **Check definition stability.** A metric redefined more than once in the
   period has no usable history, and that is worth saying.
6. **Check for a blind spot.** Which risks had no metric during the period, and
   did any of them materialise. That is how a set is genuinely improved.
7. **Retire on the record.** Name the metric, the reason, and who was reporting
   it. Silent retirement leaves people compiling numbers nobody reads.
8. **Rebalance if the set exceeds one page.** Selection is the point; growth is
   the default.
9. **Send the audit to Review & Governance {OS},** which approves retirement of
   anything other people depend on.

## Completion test

- Every metric has a count of decisions changed during the period.
- Every metric is classified: keep, redefine, retire, or externally required.
- Retirements are recorded with a reason and the people who were reporting them.
- The blind-spot check is done, naming risks that had no metric.
- The surviving set fits on one page.
- Retirements affecting other people were approved rather than assumed.

## Failure paths

| Situation | Response |
|---|---|
| no metric changed any decision | that is a finding about the review process, not only about the metrics; route it to Review & Governance {OS} |
| a metric owner defends a metric with no decisions | ask for the value at which they would act; if none exists, it moves to reported or retired |
| the set keeps growing every cycle | cap it explicitly: nothing new enters without something leaving |
| retiring a metric would hide a bad number | that is a reason to keep it; record the objection and the resolution |
