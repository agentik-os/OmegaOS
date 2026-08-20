# Review pacing and concentration

Produces the period review: pacing against plan, concentration against every
ceiling, and realised outcomes measured against the assumptions the policy was
built on, with each policy line marked held, breached or amended.

## Trigger

The period closes, or an unplanned event forces an early review: a ceiling
breach was discovered, a position was impaired
(`portfolio.position.impaired`), or the Wealth {OS} capital constraint moved.

## Inputs

- The allocation policy in force during the period, plus every amendment made
  inside it.
- The period budget and pacing plan.
- Every allocation decision record for the period, approved and declined.
- The reserve ledger.
- Current marks from Portfolio Management {OS}, event `portfolio.mark.updated`,
  with the method and evidence each mark carries.
- The current Wealth {OS} capital constraint.

## Steps

1. Reconcile deployed capital against the pacing plan: planned, deployed,
   remaining, and the shape of the deployment across the period.
2. Reconcile the reserve ledger: reserve committed, reserve drawn, reserve
   released, and reserve still held against live positions. Every entry points
   at a position or it is an error.
3. Recompute concentration on every axis in policy: position, sector, stage and
   vintage, with reserves included, using current marks.
4. Mark every ceiling held, breached or amended. A ceiling that was breached
   and later amended is recorded as both, in that order, with the dates. The
   review does not rewrite the sequence to make the period look disciplined.
5. Recompute the illiquid fraction of the pool and compare it to the
   illiquidity ceiling. State the maximum lock period now sitting in the
   portfolio.
6. Compare realised outcomes to the assumptions in the policy: loss rate,
   follow-on rate, hold period. Where an assumption is now falsified by
   evidence, say so and name the evidence, keeping the E label visible.
7. Read the declines. Group them by the policy line that killed them. A line
   that kills a large share of what the allocator later wishes they had done is
   a candidate for amendment, and this is the only place that pattern is
   visible.
8. Check the Wealth {OS} capital constraint against committed and reserved
   capital. If the constraint has tightened below current commitments plus
   reserves, raise it as a blocking finding, not a note.
9. Separate realised from unrealised in every figure presented. A period that
   looks strong only on unrealised marks is reported as exactly that.
10. Produce the review artifact and emit `capital.pacing.reported`.
11. **Human approval gate.** Any policy amendment the review recommends is not
    applied here. It is routed through Review & Governance {OS} and requires the
    allocator's signature under the `write-the-allocation-policy` workflow.
12. Set the next period budget only after the review is signed off, so the new
    pacing is built on reconciled numbers rather than on the previous plan.

## Completion test

Every line of the policy appears in the review with one of exactly three marks:
held, breached, amended. Deployed capital plus remaining budget reconciles to
the period budget. Every reserve ledger entry points at a named position.
Realised and unrealised figures are shown separately in every view. The review
carries a date and the marks it relied on carry their method.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| breach erased by later amendment | the review shows the ceiling as held because the rule changed afterwards | record both events in sequence with dates, never only the end state |
| pacing measured on commitments, not cash | deployed figure counts signed but unfunded commitments | separate signed from funded, report both, pace against funded |
| stale marks | concentration computed on marks from two periods ago | request current marks from Portfolio Management {OS}, and report concentration as provisional until they arrive |
| unrealised gains presented as performance | one strong mark carries the whole period | separate realised from unrealised in every figure |
| declines never reviewed | only approvals are examined | pull the decline log and group by governing policy line |
| constraint breach treated as a note | Wealth {OS} constraint now below commitments plus reserves | raise as blocking, halt new allocation until the constraint or the commitments are resolved |
