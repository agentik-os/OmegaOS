---
name: agent-os
description: Design, brief and supervise agents that do real work. Agent {OS}, unit 66 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about agent or invokes /agent-os.
---

# Agent {OS}

Design, brief and supervise individual agents that do real work. One agent, one
job, one done test, one named owner.

## When to use this

Use it when:

- A recurring job needs judgment and you are about to write a prompt for it.
- An agent keeps returning output nobody can use, and you suspect the brief
  rather than the model.
- An agent is running and you need to know whether it is working, stalled,
  blocked, or waiting on a human.
- An agent finished and you need to know what it actually cost and what to
  change.
- Nobody can say which agents exist, who owns them, or when each last helped.
- An agent has been running for months and nobody has asked whether it should.

**Near neighbours, and why this is not them.** Orchestration {OS} composes many
agents into one mission with a topology and a budget; Agent {OS} specifies and
supervises exactly one. AI Logic {OS} decides whether the job needs an agent at
all, and frequently the answer is no. Automation {OS} governs a deterministic
process; an agent is a judgment worker, and a job expressible as rules should
never become an agent. Evaluation {OS} owns the rubric that scores the output;
this OS embeds that rubric in the brief but does not write it.

## Capabilities

- Turn a job into an agent design: one sentence job, an explicit boundary, an
  escalation path, and a named owner.
- Write the four block executable brief: objective, constraints, mechanically
  verifiable done test, do not touch.
- Refuse a brief whose done test cannot be checked without judgment.
- Compute the minimum tool grant and tie every granted capability to a step.
- Classify a running agent into working, stalled, blocked, or asking.
- Nudge a stalled agent, escalate a blocked one, and never answer a question on
  the owner's behalf.
- Bound nudges on the absence of progress rather than on a flat count.
- Run the brief's verification instead of believing a self reported success.
- Debrief a run into a concrete brief amendment, or an explicit decision to
  change nothing.
- Maintain a roster with owner, last run and score trend.
- Retire an agent, close its dispatch paths and remove its residue.

## Procedure

1. **Check the job needs an agent.** Ask AI Logic {OS}. If it is expressible as
   rules, it goes to Automation {OS} and you stop here.
2. **State the job in one sentence** as an outcome. If it takes two sentences,
   it is two agents.
3. **Draw the boundary.** What the agent may touch, and explicitly what it must
   not. The second half is what prevents the unusable diff.
4. **Name the owner.** A person. Without one, do not proceed.
5. **Get the rubric** from Evaluation {OS}, so the brief can carry the standard
   the output will be scored against.
6. **Write the four blocks.**
   - *Objective:* the outcome, not the activity.
   - *Constraints:* budget, scope, forbidden approaches, required conventions.
   - *Done test:* a command, an assertion, a file that must exist. Mechanically
     checkable, no judgment.
   - *Do not touch:* the files, systems and decisions outside the boundary.
7. **Compute the tool grant.** Walk each step of the brief and grant only what
   that step needs. Anything that writes, sends, pays, publishes or deletes goes
   through a human approval before it is granted.
8. **Write the escalation path before dispatch:** who, through which channel,
   asked what.
9. **Dispatch and supervise.** Poll cheaply. Classify into working (say
   nothing), stalled (nudge), blocked (escalate, never nudge), asking (escalate,
   never answer). Reset the nudge budget whenever real progress advances.
10. **Verify before accepting done.** Run the brief's done test yourself. The
    agent's claim is an input.
11. **Debrief every run,** including successful ones: what was produced, what it
    cost, which grant went unused, which block was missing. Amend the brief or
    state why it stands.
12. **Review the roster periodically** and retire what has stopped earning its
    keep.

## Handoffs

| Direction | Counterpart | Contract |
|---|---|---|
| in | AI Logic {OS} | the verdict that this job needs judgment, with its falsifier |
| in | Evaluation {OS} | the rubric the output will be scored against |
| in | Tool & Integration {OS} | typed tool contracts, with their failure semantics |
| in | Context & Memory {OS} | the compiled context pack the agent starts from |
| out | Orchestration {OS} | an approved brief that can be composed into a mission |
| out | Evaluation {OS} | completed runs to score, with their rubric attached |
| out | Tool & Integration {OS} | the minimum grant the brief requires |
| out | Context & Memory {OS} | designs, briefs, debriefs, retirements, staged as records |

This OS hands over a specified worker. It never hands over a mission; that is
Orchestration's, and the difference is the whole reason both exist.
