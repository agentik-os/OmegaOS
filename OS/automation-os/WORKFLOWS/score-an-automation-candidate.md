# Workflow: Score an automation candidate

Decide whether a piece of already simplified work is worth automating, with the
arithmetic visible.

## Trigger

- Operations {OS} emits an approved simplified process map.
- Someone asks for an automation of work that recurs.
- An existing manual step has become the bottleneck of a stable process.

## Steps

1. **Require the approved simplified map.** If Operations {OS} has not approved
   one, refuse here and hand back. Do not soften this into "we can start while
   they finish": automating a broken process makes it permanent, and this is the
   only moment where it is cheap to stop.
2. **Require the baseline:** frequency, time per unit, error rate, cost, and the
   date they were measured. Without it there will be nothing to reconcile against
   after deployment, and the automation's value will be permanently unprovable.
3. **Require the AI Logic {OS} arbitration** for every step that is not obviously
   a rule. A step that needs judgment is not scored as automation work here; it
   is split out to Agent {OS}.
4. **Enumerate the exceptions from observed work.** Ask what happened the last
   ten times, not what is supposed to happen. Count them, and record the share
   of volume they represent.
5. **Assess suitability:** are the inputs stable, are the rules stable, is the
   outcome checkable? Unstable inputs make an automation a maintenance
   subscription rather than a saving.
6. **Score value:** frequency times time, plus error reduction, plus any service
   gain that is real and measured rather than hoped for.
7. **Score cost:** risk of a wrong effect, exception handling effort,
   maintenance, and the change cost when the process next moves. Maintenance is
   never zero and is never omitted.
8. **Compute the verdict** and show every input. If value does not exceed cost,
   the verdict is no, and it is stated plainly with the number that would have to
   change.
9. **Check the ownership condition.** No named human owner, no automation, at any
   score.
10. **Rank surviving candidates** and recommend exactly one to design first.
11. **Stage the score and its inputs** to Context & Memory {OS}, so the next
    review can check whether the prediction held.

## Completion test

- An approved simplified map from Operations {OS} exists and is cited.
- A dated baseline exists with all four numbers.
- Every step is either a rule, or arbitrated by AI Logic {OS} and split out.
- Exceptions are counted from observed work, with their share of volume.
- Value and cost both have visible inputs, and maintenance appears in cost.
- The verdict follows the arithmetic, including when the verdict is no.
- Each surviving candidate has a named human owner.
- Exactly one candidate is recommended for design.
- The score is staged to Context & Memory {OS} with its baseline.

A candidate that scores well but has no owner does not proceed. That combination
is the most common way an automation reaches production and quietly dies.
