# Pre-registered experiment

Run one experiment whose verdict was decided before the data arrived, and which
was applied by the unit that owns the thing it changed.

## Trigger

A backlog entry reaches the top of the ranking and the operator selects it.

## Steps

1. **The OS restates the hypothesis**: the loop step it touches, the mechanism,
   and the expected effect size. An entry without an expected effect size goes
   back to `/hypothesis`.
2. **The OS fixes the design** (`/experiment-design`): target metric, guardrail
   metric, minimum sample, duration, success threshold, and stopping rule. All
   metric definitions are pulled from KPI & Analytics {OS}.
3. **The OS checks for surface contention.** If another experiment is running on
   the same surface or the same population, this one does not start, and the
   contention is named.
4. **The OS checks the approval boundary.** Spend, any customer-facing price
   change, any experiment on live customers who are not told, and any guardrail
   that is a trust, safety or support metric stop here for an explicit human
   decision on the specific design.
5. **The design is frozen.** From this point the threshold, the sample and the
   stopping rule are immutable. An edit attempt is refused and logged.
6. **The OS hands the change proposal to the owning OS** (`/experiment-run`):
   Content {OS} for an editorial surface, Pricing {OS} for a price, Offer {OS}
   for scope or guarantee, Sales {OS} for a pipeline change, Affiliate {OS} for
   a partner promotion. Growth applies nothing itself.
7. **The owning OS applies the change under its own approval gate**, and Growth
   records that the gate was passed, with the owner's approval reference.
8. **The experiment runs** until the stopping rule fires or the duration ends.
   No interim read is compared to a threshold.
9. **The OS reads the result** (`/experiment-read`) against the frozen
   threshold, and checks the guardrail with the same seriousness as the target.
10. **The OS issues the verdict**: win, loss, or underpowered. A win on the
    target with a broken guardrail is recorded as a loss.
11. **The OS routes the outcome**: a win goes to `/scale` as a proposal, a loss
    goes back to the backlog with what was learned, an underpowered result names
    the sample that would have been needed.
12. **The verdict goes to KPI & Analytics {OS} and Business Strategy {OS}**,
    carrying the pre-registered threshold, the achieved sample and both metric
    movements.

## Completion test

The experiment record contains, all fixed before the run started and unchanged
since: a target metric, a guardrail metric, a minimum sample, a duration, a
success threshold and a stopping rule. It contains the owning OS, that unit's
approval reference, the achieved sample, both metric movements, and a verdict
of win, loss or underpowered.

The frozen fields are checked against their pre-run values. Any difference fails
the test and voids the verdict, which is then reported as unregistered rather
than as a result.

## Failure and abort

- **Achieved sample below the minimum:** report underpowered, name the sample
  that would have been needed, issue no verdict, and keep the raw data.
- **Threshold missing or edited during the run:** void the verdict, report the
  experiment as unregistered, and do not publish a directional read.
- **The owning OS declines the change:** record the decline and its reason
  against the backlog entry. Do not route the change to a different unit.
- **Human approval refused:** the experiment does not start. Record the refusal
  against the backlog entry so it is not silently re-proposed next cycle.
- **Surface contention:** the second experiment does not start. Queue it and
  name the contending experiment and its end date.
- **Guardrail breaks mid-run:** the stopping rule fires immediately, the change
  is handed back to its owner to revert, and the verdict is a loss.
