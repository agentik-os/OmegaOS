# Orchestration {OS}: Operating Specification

## 1. Purpose

Compose many agents and systems into one mission that finishes, with a shape
chosen deliberately, a ledger that survives an interruption, verification on the
edges, and a closure that leaves nothing running.

A fan out without a synthesis is not a mission. A plan that lives only in a
conversation is gone the moment the conversation is compacted.

## 2. Boundary

- **Owns:** the mission shape (which steps run in parallel, which are genuinely
  sequential, where a barrier is justified); the durable mission ledger; task
  state and its transitions; budget across the whole mission; scope claims that
  keep two writers off one file; independent verification of each task before it
  is closed; synthesis of every child output; and closure, including accounting
  for every worker it started.
- **Does not own:** the design of any individual agent (Agent {OS}); the rubric
  used to judge output (Evaluation {OS}); the contract of any tool a step calls
  (Tool & Integration {OS}); whether a step should be a model call at all (AI
  Logic {OS}); the canonical storage of what was learned (Context & Memory
  {OS}); and the installation or running of an OS unit (Agentik Runtime).
- **Hands off to:** Evaluation {OS} for scoring the mission's outputs; Context &
  Memory {OS} for the ledger, the outcomes and the postmortem; the mission's
  requester for the synthesis.
- **Consumes from:** Agent {OS} approved briefs it may compose; Tool &
  Integration {OS} the failure semantics its topology must tolerate; AI Logic
  {OS} the arbitration behind each node; Context & Memory {OS} the compiled
  context each step starts from.

**The near neighbour it is confused with: Agent {OS}.** Agent designs, briefs
and supervises **one** worker. Orchestration composes **many** into a mission
with a topology, a budget, a ledger and a closure. Orchestration never rewrites
a brief mid mission: if the brief is wrong, the mission stops and the change is
made in Agent {OS} with its own review. Agent never decides the shape of a fan
out, and never owns the mission's budget or its closure.

It is also not Agentik Runtime, which installs, configures and runs OS units.
Orchestration composes work at runtime; it installs nothing.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SHAPE` | a mission with more than two steps | the topology, with each edge justified | every edge passes the data test |
| `PLAN` | a shape exists | the durable ledger, one entry per ask | the ledger is persisted, not narrated |
| `DISPATCH` | the ledger has a ready set | running steps, with scope claims held | every dispatched step has an owner and a budget |
| `WATCH` | steps are running | state per step and the matching action | each step reaches a terminal state |
| `VERIFY` | a step claims completion | independent evidence | the verification command was run by the coordinator |
| `SYNTHESISE` | children have returned | one coherent result | every child output is represented or explicitly discarded |
| `CLOSE` | every ledger entry is done or honestly not | a signal and a clean shutdown | no worker of this mission is still running |
| `POSTMORTEM` | a mission failed or overran | the cause and the shape change | the finding is a change to the topology or the ledger |

`PLAN` is not optional and it is not prose. A ledger that exists only in the
transcript disappears at the first compaction, which is precisely when the tail
of the mission gets dropped.

## 4. Inputs

- The mission as the requester stated it, in their order, including the asks
  that look secondary.
- Approved agent briefs from Agent {OS}, each with a mechanically checkable done
  test.
- The compiled context pack per step from Context & Memory {OS}.
- The failure semantics of every external system the mission touches.
- The budget ceiling: tokens, time, cost.
- The file and resource scope of each step, so overlapping writers can be
  serialised or isolated.

## 5. Outputs

| Artifact | Shape | Goes to |
|---|---|---|
| Topology | nodes, edges, and the justification for each edge and barrier | the mission record |
| Ledger | one entry per ask, with state, owner and evidence | persisted, and readable after a restart |
| Verification record | per task: the command run, by whom, and its result | the ledger |
| Synthesis | one coherent result assembled from every child output | the requester |
| Closure signal | clean, pending with what remains, or failed with evidence | the requester |
| Postmortem | cause, and the change to the shape or the ledger | Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the ledger, verification records, closure signals, postmortems | Context & Memory {OS} via `memory.record.staged`, and a persisted mission file |
| projection | the live view of what is running | recomputed from the ledger and the running set |
| cache | child outputs before synthesis | discarded once the synthesis is recorded |
| temporary | one poll of a running step | the turn |

The ledger is written to disk before each dispatch, so an interrupted mission
resumes at the first entry that is not done rather than from somebody's memory
of what was happening.

## 7. Rules and invariants

1. **Enumerate every ask first, in the requester's own order.** Missions
   routinely carry several asks, and the ones that vanish are always the last.
2. **Persist the plan, do not narrate it.** The ledger file is the mission state.
   Prose is not state.
3. **One task in progress per coordinator lane.** Transitions are explicit and
   happen when they actually happen, never batched at the end.
4. **Pipeline by default; a barrier must be justified.** A barrier makes every
   item wait for the slowest, and that latency is real. The legal reasons are: a
   cross set operation, an early exit on the total, or a step that genuinely
   compares an item against all the others.
5. **An edge exists only where data moves.** If the next step does not read the
   previous step's output, there is no edge, and the wait is pure delay.
6. **Coordination is code, not an agent.** Flatten, dedupe, filter, rank and sort
   between stages cost nothing when they are code and cost a model call when they
   are an agent.
7. **A delegate's claim of done is an input, never the verdict.** The
   coordinator runs the verification named in the brief before an entry moves to
   done.
8. **Failure is contained per node.** A failing step resolves to a null result
   that downstream steps tolerate, rather than sinking the mission. Steps
   unreachable because of it are reported as unreachable, not left queued
   forever.
9. **Two writers never share a file.** Overlapping scope is serialised or
   isolated, and a claim is released when the step ends, including when it ends
   badly.
10. **Cycles converge or they do not exist.** A loop stops after a bounded number
    of rounds that produce nothing new, and it deduplicates against everything
    seen rather than only against what was accepted.
11. **Closure accounts for every worker.** A mission does not signal clean while
    anything it started is still running, and closure is safe to run twice.
12. **The signal is honest.** Clean only when every entry is done and
    independently verified; otherwise pending with what remains, or failed with
    the evidence. An incomplete mission reported as clean ends the work for
    everyone downstream.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a step fails | contain it, resolve to null, report which downstream steps became unreachable |
| a step claims done | run the verification before believing it, record both facts |
| a step is blocked | escalate, never nudge, and never re-dispatch into the same block |
| the budget is approaching its ceiling | escalate before overrunning, never silently continue |
| a loop is not converging | stop it at its bound and report what it kept rediscovering |
| two steps want the same file | serialise them or isolate them, never run both |
| the session is interrupted | resume from the persisted ledger at the first entry that is not done |
| a worker is still running at closure | refuse the clean signal, account for it, then close |
| an ask cannot be completed | keep it in the ledger as not done and say so in the report, never quietly drop it |

## 9. Human approval boundary

This OS asks before:

- dispatching a step that performs an irreversible action, unless a gate is
  already recorded for it
- exceeding the mission budget ceiling
- changing an agent's brief mid mission, which belongs to Agent {OS} and is not
  an orchestration decision
- closing a mission as clean when any entry lacks independent verification
- killing a worker that holds uncommitted work
- re-running a step whose external effects are not idempotent

It never answers a question a running step escalated to a human, and it never
marks an ask done on a delegate's word.

## 10. Completion criteria

Every ask the requester stated appears in a persisted ledger; each is done and
independently verified, or honestly reported as not done; the shape is one
somebody chose rather than one that emerged; every child output is represented
in a single synthesis; nothing the mission started is still running; and the
closure signal says exactly what happened.
