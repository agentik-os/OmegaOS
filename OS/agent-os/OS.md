# Agent {OS}: Operating Specification

## 1. Purpose

Design, brief and supervise individual agents that do real work, so that an
agent is a specified worker with a job, a boundary, a done test and an owner,
rather than a prompt someone got lucky with once.

An agent that cannot be graded is not an agent. This OS makes each one
gradeable before it runs, watchable while it runs, and honestly closed when it
finishes.

## 2. Boundary

- **Owns:** the design of one agent (its job, its inputs, its tool grant, its
  boundary, its escalation path); the executable brief; the supervision of a
  running agent and the classification of its state; the debrief that turns a
  run into a change to the brief; the roster of agents that exist, who owns each
  one, and when each was last useful; and the retirement of agents that no
  longer earn their keep.
- **Does not own:** whether the job needs an agent at all (AI Logic {OS}
  arbitrates that); the composition of several agents into a mission
  (Orchestration {OS}); the rubric that scores the agent's output (Evaluation
  {OS} defines it, this OS embeds it in the brief); the contract of any tool the
  agent calls (Tool & Integration {OS}); the durable storage of what the agent
  learned (Context & Memory {OS}).
- **Hands off to:** Orchestration {OS} with an approved brief that can be
  composed; Evaluation {OS} with completed runs to score; Tool & Integration
  {OS} with the minimum tool grant a brief requires; Context & Memory {OS} with
  outcomes and debriefs.
- **Consumes from:** AI Logic {OS} the verdict that this job genuinely needs
  judgment; Evaluation {OS} the rubric and the score history; Tool &
  Integration {OS} the typed contracts an agent may call; Context & Memory {OS}
  the compiled context an agent starts from.

**The near neighbour it is confused with: Orchestration {OS}.** Agent {OS}
designs and supervises **one** worker. Orchestration {OS} composes **many** into
a mission with a topology, a budget and a closure. Agent never owns the mission
and never decides the shape of a fan out. Orchestration never rewrites an
agent's brief mid mission; if the brief is wrong, that is an Agent {OS} change
with its own review.

It is also not Automation {OS}: an automation is a governed deterministic
process, an agent is a supervised judgment worker. If a candidate can be
expressed as rules, AI Logic {OS} sends it to Automation and no agent is
designed at all.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `DESIGN` | a job exists that needs judgment | the agent's job statement, boundary and escalation path | the job is one sentence and the boundary names what it must not touch |
| `BRIEF` | a design is accepted | an executable brief with four blocks | objective, constraints, verifiable done test and do not touch are all filled |
| `GRANT` | a brief names external capabilities | the minimum tool grant | every granted tool is used by a named step of the brief |
| `SUPERVISE` | an agent is running | a state classification and, when warranted, an intervention | the run reaches a terminal state |
| `DEBRIEF` | a run finished, well or badly | what it did, what it cost, what changes in the brief | the brief is amended or explicitly left unchanged |
| `ROSTER` | someone asks what agents exist | the roster with owner, last run and score trend | every agent has a named owner |
| `RETIRE` | an agent stopped earning its keep | a retirement with its reason and its residue removed | no live path still dispatches to it |

`BRIEF` is the mode that decides whether anything else works. A brief whose four
blocks are not filled is refused, because an underspecified brief is precisely
what produces a large diff nobody can use.

## 4. Inputs

- The job the agent is supposed to do, stated as an outcome rather than an
  activity.
- The AI Logic {OS} verdict that this job genuinely needs judgment.
- The rubric from Evaluation {OS} that will score the output.
- The tool contracts from Tool & Integration {OS} the agent may be granted.
- The compiled context pack from Context & Memory {OS} the agent starts with.
- The named human owner. An agent without one is not designed.
- The failure history of previous runs, which is the only reliable source of
  what the brief is still missing.

## 5. Outputs

| Artifact | Shape | Goes to |
|---|---|---|
| Agent design | job, boundary, escalation path, owner | the requester, staged to Context & Memory {OS} |
| Executable brief | objective, constraints, verifiable done test, do not touch | Orchestration {OS}, or direct dispatch |
| Tool grant | the minimum set, each tied to a step | Tool & Integration {OS} |
| Supervision report | the state, the evidence for it, the action taken | the agent's owner |
| Debrief | what was produced, what it cost, what changes | the brief, and Evaluation {OS} |
| Roster | every agent with owner, last run, score trend | the user |
| Retirement | reason, residue removed, dispatch paths closed | Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | agent designs, approved briefs, tool grants, debriefs, retirements | Context & Memory {OS} via `memory.record.staged` |
| projection | the roster, and score trends per agent | recomputed from briefs plus Evaluation {OS} scores |
| cache | a running agent's captured output while it is being classified | discarded once the run reaches a terminal state |
| temporary | the current supervision poll | the turn |

An agent's own working notes during a run are temporary by default. They become
records only when the debrief promotes them, which is what stops a roster of ten
agents from producing ten competing memories.

## 7. Rules and invariants

1. **One agent, one job.** An agent that does two jobs cannot be graded on
   either, and its failures cannot be attributed.
2. **The four block brief is mandatory.** Objective, constraints, a
   mechanically verifiable done test, and do not touch. A brief missing any
   block is red and blocking, and the fourth block is the one that is always
   omitted and always the one that prevents an unusable diff.
3. **The done test must be checkable without judgment.** A command that exits
   zero, a file that exists, an assertion that passes. "Looks correct" is not a
   done test.
4. **The tool grant is minimum and justified per step.** Every granted
   capability maps to a step of the brief. An unused grant is removed at debrief.
5. **An agent's own claim of success is an input, never the verdict.** The
   verification named in the brief is run by the supervisor, not by the agent
   reporting on itself.
6. **Every agent has a named human owner.** Ownership is a person, not a team
   and not the system.
7. **Supervision distinguishes four states, not two:** working (say nothing),
   stalled (nudge mechanically), blocked (never nudge, escalate), and asking a
   question that needs judgment (escalate, never answer on the human's behalf).
8. **Escalation is written down before the run,** not improvised during it: who
   is escalated to, through which channel, and what they are being asked.
9. **A run always produces a debrief,** including a run that succeeded. A
   success nobody examined is where the next failure is hiding.
10. **Retirement is a real mode.** An agent that has not earned its keep is
    retired, its dispatch paths closed and its residue removed. A roster that
    only grows is a roster nobody reads.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the job is not actually a judgment job | hand to AI Logic {OS}, do not design an agent |
| the brief has an unfillable done test | refuse the brief, state which block cannot be filled |
| no named owner | refuse to design, this is not negotiable |
| the agent asks a question mid run | escalate to the owner, never answer for them |
| the agent claims done | run the verification from the brief before believing it |
| the agent is stalled with work available | nudge, and bound the nudges on the absence of progress rather than on a flat count |
| the agent is blocked with nothing runnable | escalate, do not nudge, a nudge here manufactures thrash |
| the agent requests a tool it was not granted | deny, log the request, review the grant at debrief |
| the run failed with no usable output | debrief anyway, name the block that was missing |

## 9. Human approval boundary

This OS asks before:

- granting an agent a tool that can write, send, pay, publish or delete
- widening a tool grant beyond what the brief's steps justify
- dispatching an agent whose done test cannot be checked mechanically
- letting an agent run unattended when its brief contains an irreversible action
- retiring an agent another mission still dispatches to
- accepting an agent's self reported success in place of the named verification

An agent never approves its own escalation, and never widens its own grant.

## 10. Completion criteria

A user names a job and leaves with: an agent whose job is one sentence, a brief
whose four blocks are filled and whose done test is a command, a tool grant that
is exactly what the brief needs, a named owner, a written escalation path, and,
after the run, a debrief that either changed the brief or explicitly said why it
did not.
