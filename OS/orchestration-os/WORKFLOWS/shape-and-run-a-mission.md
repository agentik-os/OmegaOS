# Workflow: Shape and run a mission

Take a mission with several parts and finish all of them, in a shape somebody
chose.

## Trigger

- A mission carries three or more parts that touch different files.
- A request contains several asks in one message.
- A breadth first sweep is needed across many sources or many files.

## Steps

1. **Enumerate every ask, in the requester's own order,** before anything else.
   Include the asks that look secondary; those are the ones that disappear.
2. **Persist the ledger to a file,** one entry per ask, with state and owner. If
   the plan lives only in the conversation, the mission has no state and cannot
   be resumed.
3. **Draw the nodes.** Each is one agent with one bounded job, one input and one
   output. A node whose output only a human can read cannot be wired into
   anything.
4. **Apply the data test to every edge.** Does the next step read the previous
   step's output? If not, the edge is a wait and it is deleted. Most sequential
   looking missions collapse into something much wider at this step.
5. **Default to a pipeline.** Add a barrier only for a cross set operation, an
   early exit on the total, or a comparison of each item against all the others.
   "The stages feel separate" is not a reason: separate is not synchronised.
6. **Put the reductions in code.** Flatten, dedupe, filter, rank and sort are
   plain code between stages, not agents. An agent burned on a dedupe is an
   expensive way to write three lines.
7. **Declare every step's file and resource scope** and resolve overlaps by
   serialising or isolating them. Two writers on one file is the failure that
   silently destroys work.
8. **Attach the verification command** from each brief to its ledger entry now,
   not after the output arrives.
9. **Set the budget** for the mission and per step, and name the point at which
   you escalate rather than overrun.
10. **Dispatch the ready set concurrently,** each with its owner, budget and
    claim.
11. **Watch and classify:** working, stalled, blocked, asking. Silence, nudge,
    escalate, escalate. Never answer an escalated question for the human.
12. **Verify each claimed completion yourself** before moving its ledger entry,
    and record the claim and the verification separately.
13. **Contain failures.** A failed node resolves to null, downstream steps
    tolerate the gap, and steps that became unreachable are reported as such
    rather than left queued.
14. **Synthesise every child output** into one result, discarding nothing
    silently.
15. **Close** through the closure workflow.

## Completion test

- Every ask appears as an entry in a persisted ledger file, in the requester's
  order.
- Every edge in the topology carries data, and every barrier names its
  justification.
- No reduction between stages was performed by an agent.
- No two steps hold overlapping write scope.
- Every ledger entry that is done has an independent verification record naming
  the command and its output.
- Failed nodes are contained, and unreachable downstream steps are reported.
- A single synthesis exists in which every child output is represented or
  explicitly discarded with a reason.
- The budget was respected, or an escalation happened before the ceiling.
- The mission ends through the closure workflow with an honest signal.

A mission that produced excellent work on four asks and never touched the fifth
is not finished, and reporting it as finished is the failure this workflow is
built against.
