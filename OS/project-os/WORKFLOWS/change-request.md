# Workflow: the change request

Produces a priced change record and a decision, so that scope grows on purpose
or not at all.

## Trigger

Anything is requested that the scope statement does not contain, including a
request that sounds small, including a request from the person paying, and
including a good idea generated inside the team.

## Inputs

- The agreed scope statement and its out-of-scope list.
- The current plan, critical path and remaining capacity.
- The requester of the change and their reason.

## Steps

1. **Check it against the scope statement.** If it is inside scope, it is not a
   change, and the plan already covers it. Say so and stop.
2. **State the change in one sentence** that the requester would recognise.
3. **Price it in time.** Work, plus the rework it causes to anything already
   built or approved.
4. **Price it in money** where money applies, including third-party costs.
5. **State the effect on the landing date.** If it touches the critical path,
   say the new date. If it does not, say the float it consumes.
6. **Give the options.** Accept and move the date, accept and cut something
   named, reject, or defer to a later phase. Every option carries its
   consequence.
7. **Name the decider.** The person who owns the date and the budget, not the
   person who asked.
8. **Escalate if it crosses a threshold.** A change beyond the agreed threshold
   goes to Review & Governance {OS}, which decides. The project does not
   approve its own boundary change.
9. **Record the decision and the decider.** Then update the plan, or do not
   touch the plan at all.

## Completion test

- A change record exists with the change, its time cost, its money cost, and its
  effect on the landing date.
- At least one option to keep the date is on the table, and it names what would
  be cut.
- The decider is named, the decision is recorded, and the date is stamped.
- If accepted, the plan and the scope statement are both updated. If rejected,
  the request and the rejection are still kept.

## Failure paths

| Situation | Response |
|---|---|
| the change is small and everyone wants to just do it | still write the record; the record takes minutes and untracked small changes are how scope dies |
| the requester refuses to accept a date move | present the cut-scope option with named cuts, and let them choose the trade |
| the change arrives as an assumption in someone else's plan | surface it as a change anyway, and price it before it becomes a commitment |
| the change crosses a policy or contractual boundary | stop, and route to Review & Governance {OS} and Client {OS} before any pricing is communicated |
