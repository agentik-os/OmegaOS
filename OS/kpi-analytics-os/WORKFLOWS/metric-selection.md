# Workflow: metric selection

Produces a metric set that fits on one page, in which every metric names the
decision it can change.

## Trigger

There are too many numbers and nobody acts on them, there are none at all, or
somebody has asked for a new dashboard.

## Inputs

- The decisions people make repeatedly: pricing, hiring, stopping, doubling
  down, escalating, intervening.
- What the quarter is for, from Business Strategy {OS}.
- The existing metric list, however long.
- Which sources are authoritative for which quantities.

## Steps

1. **Start from decisions, not from data.** List the decisions that recur. If
   the list starts from what the tools can already report, the result will be a
   dashboard rather than a decision aid.
2. **For each decision, ask what number would change it,** and at what value.
   A decision with no such number is made on judgement, and that is a legitimate
   answer to record.
3. **Test each existing metric.** What decision would change at what value. Any
   metric with no answer goes on the stop-tracking list.
4. **Name the vanity ones explicitly.** Numbers that only go up, numbers that
   flatter, and numbers reported because they are easy to collect.
5. **Prefer few and stable.** A small set held across quarters is worth more
   than a rich set rebuilt each quarter, because comparability is most of the
   value.
6. **Check coverage against the real risks.** Money, delivery, clients, capacity.
   A set that measures only the easy domain will be blind exactly where it hurts.
7. **Assign an owner per metric,** the person who acts when it moves, not the
   person who compiles it.
8. **Write the stop-tracking list with reasons,** and say what attention it
   returns. Removing a report is a decision people will contest, so it needs its
   reason on the record.
9. **Publish the set on one page.** If it does not fit, it has not been selected.

## Completion test

- Every metric in the set names a decision and the value at which that decision
  becomes due.
- Every metric has an owner who acts, not an owner who compiles.
- The set fits on one page.
- The stop-tracking list exists, with a reason per removal.
- Vanity metrics are named as such rather than quietly dropped.
- The set covers money, delivery, clients and capacity, or the gap is stated.

## Failure paths

| Situation | Response |
|---|---|
| a metric is required by a board or a client | keep it, mark it as reported rather than decision-driving, and keep it out of the decision set |
| nobody can name a decision for a favourite metric | that is the answer; move it to the stop-tracking list and say why |
| the decision exists but the data does not | keep the metric, mark it not yet computable, and issue the instrumentation requirement |
| there are genuinely more than a page of decisions | select per audience: one page each, not one page of forty rows |
