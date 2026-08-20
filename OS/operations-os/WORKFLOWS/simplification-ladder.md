# Workflow: the simplification ladder

Produces a decision on every step of a measured process, taken in ladder order,
and a target operating model that is reachable from today.

## Trigger

A process has been mapped and measured. This workflow never runs on an unmeasured
process, because without numbers every argument is a preference.

## Inputs

- The current-state map with waits and rework loops.
- The measurement sheet, including the unknowns.
- The exception list and its rate.
- The control list: steps that exist for compliance, financial or safety reasons.
- What the process is supposed to produce, and for whom.

## Steps

1. **Start from the output.** What does the process produce, who consumes it, and
   what would they actually notice if it stopped. Sometimes the answer is
   nothing, and that ends the analysis.
2. **Eliminate.** For each step ask what would break if it disappeared. A step
   with no answer is removed. Steps that exist to check another step's known
   defect are usually removed together with that defect.
3. **Attack the waits before the touches.** Waiting is normally the majority of
   elapsed time and the minority of the attention. Remove approvals that never
   reject, batches that exist for convenience, and handoffs that exist because
   of an org chart.
4. **Simplify.** Fewer inputs, fewer decisions, fewer systems, fewer people per
   run. Merge steps that always happen together, and remove decisions whose
   answer is always the same.
5. **Reduce the exceptions.** Many exceptions are caused upstream by a
   permissive input. Fixing the input removes the exception branch entirely.
6. **Protect the controls.** A step that exists for compliance, money or safety
   is never removed here. Route it to Review & Governance {OS} with the question
   and the evidence.
7. **Standardise what survives.** The remaining process goes to Process & SOP
   {OS} to become steps anyone can follow.
8. **Decide who does it.** If it should move to another person, hand to Team &
   Delegation {OS} with the outcome and the definition of done.
9. **Design the target operating model:** the steps that survive, in order, with
   their owners, their controls and their exception paths. Check it is reachable
   from today without stopping the business.
10. **Record every decision with its reason**, including the steps kept
    unchanged, so the next diagnosis does not re-litigate them.

## Completion test

- Every step has a recorded decision: removed, merged, reordered, simplified,
  or kept, each with a reason.
- The waits were addressed explicitly, not only the touch steps.
- No control was removed inside this workflow; each was routed for review.
- The target operating model exists, with owners, controls and exception paths.
- The target is reachable from today, and the transition is described.
- The expected effect is stated in the same units as the measurement sheet.

## Failure paths

| Situation | Response |
|---|---|
| the requester wants to jump to automation | run the ladder anyway, then show how much of what they wanted to automate is now gone |
| a step's owner defends it without an answer to what would break | record the disagreement, keep the step, and set a trial removal with a defined observation window |
| the exception rate is unknown | stop before standardisation and measure exceptions for one cycle |
| the target model requires a system that does not exist | split it: ship the simplification that needs no new system now, and route the rest to Automation {OS} as a separate decision |
