# Workflow: change authorisation

Produces an approve, reject or defer decision on a consequential change, taken
by somebody other than the proposer, with conditions, a reversal path and a
verification test.

## Trigger

A domain OS proposes a change to its own boundary, policy or controls, or any
change is proposed whose failure would be expensive, visible or hard to undo.
Examples that always arrive here: Operations {OS} wanting to remove a control,
Client {OS} wanting a new exception class, KPI & Analytics {OS} retiring a
metric others depend on, Team & Delegation {OS} raising an authority level past
a policy limit, Execution {OS} changing what counts as proof.

## Inputs

- The proposal: what changes, why now, and what problem it solves.
- The evidence that the problem is real.
- What is at risk if it goes wrong, and who is exposed.
- The reversal path: how it would be undone, by whom, and what would be lost.
- The current policy set and decision rights map.
- Any previous decision on the same question.

## Steps

1. **Check the separation.** The proposer may not approve. If the same person
   holds both roles, separate them in time and in writing: the proposal is
   recorded first, the decision second, and the record names the conflict.
2. **Check the decision rights.** Who has the right to decide this, at this
   value, under the current map. If nobody does, that gap is fixed before the
   change is decided.
3. **Test the problem.** Is there evidence that the problem is real and
   recurring, or is this a change proposed from preference.
4. **Ask what is lost if it goes wrong,** in concrete terms: money, a client, a
   control, data, trust, time.
5. **Require the reversal path.** A change nobody can undo faces a higher bar,
   not a quieter one.
6. **Look for the cheaper experiment.** Can this be tried on one process, one
   client or one week before it becomes the rule. A bounded trial is usually the
   right approval.
7. **Check it against existing policy.** Either it fits, or the policy changes
   too, deliberately and in the same decision.
8. **Decide: approve, approve with conditions, defer with what is missing, or
   reject with the reason.** All four are legitimate outcomes.
9. **Write the verification test** with the approval: what would show it worked,
   measured how, by when. No test, no approval.
10. **Record it in the append-only audit trail** with the evidence, the decider
    and the date.
11. **Hand it back to the owning OS** with its conditions, and schedule the
    verification.

## Completion test

- The decider is not the proposer, or the temporal separation is recorded with
  the conflict named.
- The decision rights for this change are established before the decision.
- The proposal states what is lost if it fails and how it would be reversed.
- The outcome is one of approve, approve with conditions, defer, or reject, with
  the reason recorded.
- An approved change has a verification test with a date.
- The record is in the append-only audit trail, citing its evidence.

## Failure paths

| Situation | Response |
|---|---|
| the change has already been made | record it as unauthorised, decide keep or revert on its merits, and fix the decision rights gap that allowed it |
| the proposer is the only qualified approver | separate in time, record the conflict, and require an independent verification of the result rather than of the decision |
| the change is urgent and there is no time | approve provisionally with an explicit expiry date, and run the full authorisation before that date |
| there is no reversal path at all | raise the bar: require a bounded trial, a stronger verification, or reject |
| the same change was rejected before | require the new information; without it, the previous decision stands and the record says so |
