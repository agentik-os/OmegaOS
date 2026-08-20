# Workflow: scope to plan

Produces a scope statement that can be finished and a milestone plan that can
be tracked.

## Trigger

A project is proposed, or work has already started on something nobody has
scoped. The second case is the more common and the more urgent.

## Inputs

- The requester, and what changes for them when this exists.
- Hard constraints: dates, budget, named people, external dependencies.
- Real capacity from Team & Delegation {OS}, not headcount.
- Actual durations of comparable past work from Context & Memory {OS}.

## Steps

1. **Name the requester.** If nobody wants the outcome, stop and say so.
2. **Write the done test.** One observable condition, checkable by a person who
   did no work on it. If it cannot be written, the project is not ready to plan
   and this workflow ends here with that finding.
3. **Write the out-of-scope list.** Everything a reasonable person might assume
   is included and is not. Longer than the scope list, usually.
4. **Record the constraints and separate them from the preferences.** A fixed
   launch date is a constraint. A preferred technology is not.
5. **List the assumptions.** Numbered, falsifiable, each with what happens if it
   is wrong.
6. **Decompose to milestones.** Five to nine for most projects. Each milestone
   is a state of the deliverable, not a phase of activity.
7. **Give every milestone one owner, one date and one acceptance test.**
8. **Map the dependencies**, including the ones that belong to other people and
   other organisations. Name the critical path explicitly.
9. **Label every date with its basis:** comparable past work, decomposition, or
   a guess declared as a guess.
10. **Test the plan against capacity.** If it does not fit, present the cut-scope
    option now. Presenting it later costs the whole difference.
11. **Emit the work.** Next actions to Execution {OS}, work packages to Team &
    Delegation {OS}, the scope statement to Client {OS} if there is a client.

## Completion test

- A done test exists and a stranger could apply it.
- An out-of-scope list exists and is not empty.
- Every milestone has exactly one owner, one date and one acceptance test.
- The critical path is named.
- Every date states its basis.
- The plan fits the real available capacity, or the mismatch is written down and
  has been shown to the requester.

## Failure paths

| Situation | Response |
|---|---|
| the done test keeps changing during scoping | that is the finding: run a decision session, do not plan over an unstable target |
| the deadline is fixed and the scope does not fit | present cut-scope options before work starts, and record who decided |
| no comparable history exists | say the estimate is a guess, widen the range, and set an early checkpoint to correct it |
| the requester will not commit to an out-of-scope list | proceed, and record the refusal as the top risk in the register |
