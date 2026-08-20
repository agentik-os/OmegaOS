# Workflow: Review and retire the roster

Keep the set of agents small enough that someone can still say what each one is
for.

## Trigger

- A periodic roster review.
- Nobody can remember why an agent exists.
- Two agents look like they do the same job.
- An owner leaves or changes role.

## Steps

1. **List every agent** with its owner, its last run, its score trend from
   Evaluation {OS}, and its current tool grant.
2. **Flag every agent with no named owner.** These are retired or re-owned. An
   unowned agent is a running process with no accountability.
3. **Flag every agent whose last run predates the stated review window,** and ask
   its owner one question: would you notice if this stopped existing?
4. **Compare overlapping agents pairwise.** Two agents whose jobs overlap are
   graded against each other on the same rubric; the loser is retired and its
   useful constraints are merged into the winner's brief.
5. **Read the score trend, not the last score.** An agent that is slowly getting
   worse against a stable rubric has a brief that is drifting away from the job.
6. **Audit grants against use.** Any capability that has not been exercised in
   the review window is removed. Grants only ever grow unless something removes
   them.
7. **Decide per agent: keep, amend, or retire.** Keeping is a decision and it is
   written down with a reason, exactly like retiring.
8. **Before retiring, find the live dispatch paths.** Orchestration missions,
   scheduled jobs, other agents that hand off to it. Retirement is refused while
   a live path still routes to it.
9. **Retire cleanly:** close the dispatch paths, revoke the tool grants, remove
   the residue, and record the reason and the date.
10. **Stage the review** to Context & Memory {OS} so the next review can see what
    was decided last time and whether it held.

## Completion test

- Every agent on the roster has a named owner, or has been retired.
- Every agent has an explicit decision recorded: keep, amend, or retire.
- Overlapping pairs have been graded against one rubric and resolved.
- Unused tool grants have been revoked.
- No retirement happened while a live dispatch path still routed to the agent.
- Each retirement records its reason, closed paths, revoked grants and date.
- The review is staged to Context & Memory {OS}.

A roster that only grows is a roster nobody reads, and an agent nobody reads
about is an agent nobody can be accountable for.
