# Workflow: Design and brief an agent

Turn a job into a specified worker with a boundary, a done test that is a
command, and a named owner.

## Trigger

- A recurring job needs judgment and someone is about to write a prompt for it.
- An existing agent returns output nobody can use and the brief is suspected.
- Orchestration {OS} needs a node in a mission and no suitable agent exists.

## Steps

1. **Ask AI Logic {OS} first.** If the job is expressible as rules, it goes to
   Automation {OS} and no agent is designed. This step is skipped constantly and
   is the cheapest possible save.
2. **Check the roster.** An agent for this job may already exist. Amending one
   brief beats maintaining two agents that overlap.
3. **State the job in one sentence, as an outcome.** If it needs two sentences,
   split it into two agents and run this workflow twice.
4. **Name the owner.** A person, not a team. Stop here if there is none.
5. **Draw the boundary in both directions:** what the agent may read, write and
   call, and explicitly what it must not touch. Write the second list even when
   it feels obvious.
6. **Fetch the rubric** from Evaluation {OS}, so the standard the output will be
   judged against is known before the work is defined.
7. **Write block one, objective:** the outcome the run must produce, in the
   user's terms.
8. **Write block two, constraints:** budget, time, scope, forbidden approaches,
   conventions that must be followed.
9. **Write block three, the done test.** A command that exits zero, an assertion
   that passes, a file that must exist with stated properties. If you cannot
   write a mechanical test, stop: either the objective is too vague to be worked
   on, or the job is not yet an agent's job.
10. **Write block four, do not touch.** The files, systems, credentials and
    decisions outside the boundary. This is the block everyone omits and the one
    that prevents a nine hundred line diff nobody asked for.
11. **Compute the tool grant** step by step. Anything that writes, sends, pays,
    publishes or deletes is separated out for explicit human approval.
12. **Write the escalation path:** who is escalated to, through which channel,
    and what exactly they will be asked to decide.
13. **Stage the design and the brief** to Context & Memory {OS}, and register the
    agent on the roster with its owner.

## Completion test

- The job is one sentence and states an outcome.
- A named human owner exists.
- All four brief blocks are filled, with no placeholder text.
- The done test is a command or an assertion that a machine can run, and it has
  been run once against a known good input to prove it discriminates.
- Every granted tool maps to a numbered step of the brief.
- Capabilities that write, send, pay, publish or delete carry a recorded human
  approval.
- The escalation path names a person, a channel and a decision.
- The agent appears on the roster with its owner.

A brief that fails any of these is red and blocking. It is not dispatched, and
softening the done test to unblock it is the exact failure this workflow exists
to prevent.
