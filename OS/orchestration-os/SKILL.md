---
name: orchestration-os
description: Compose many agents and systems into one reliable mission. Orchestration {OS}, unit 71 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about orchestration or invokes /orchestration-os.
---

# Orchestration {OS}

Compose many agents and systems into one mission that finishes: a deliberate
shape, a durable ledger, verification on the edges, and a closure that leaves
nothing running.

## When to use this

Use it when:

- A mission has several file disjoint parts that could run at once.
- A mission carries several asks and the last ones keep getting dropped.
- A long run gets interrupted and nobody can say where it was.
- Steps are waiting on each other with no data actually passing between them.
- Workers finish and nobody assembles their outputs into one answer.
- A mission was reported complete and half of it was not.

**Near neighbours, and why this is not them.** Agent {OS} designs, briefs and
supervises one worker; this OS composes many and owns the mission's budget,
ledger and closure. It never rewrites a brief mid mission: a wrong brief stops
the mission and is fixed in Agent {OS}. AI Logic {OS} decides whether a node
should be a model call at all. Evaluation {OS} scores the outputs. Tool &
Integration {OS} owns the failure semantics this topology must tolerate. Agentik
Runtime installs and runs OS units; this OS installs nothing.

## Capabilities

- Choose a topology deliberately: pipeline by default, barriers only when
  justified.
- Apply the data test to every edge, and delete the ones where nothing moves.
- Persist a ledger with one entry per ask, in the requester's own order.
- Resume from the ledger after an interruption instead of from memory.
- Hold scope claims so two steps never write the same file.
- Dispatch a ready set concurrently with a budget per step.
- Classify running steps and act on the classification without babysitting them.
- Run the verification a brief names, rather than accepting a delegate's claim.
- Contain a node failure and report which downstream steps became unreachable.
- Bound a loop and deduplicate against everything seen, not only what was kept.
- Synthesise every child output into one coherent result.
- Close honestly: clean, pending with what remains, or failed with evidence.

## Procedure

1. **Enumerate every ask in the requester's own order,** including the ones that
   look secondary. This happens before the first dispatch, because the asks that
   vanish are always the last.
2. **Persist the ledger.** One entry per ask, written to a file. A plan that
   lives only in prose is gone at the first compaction.
3. **Draw the shape.** Nodes are single bounded jobs. Draw an edge only where the
   next step reads the previous step's output; if it does not, the edge is a
   wait, and the chain collapses into something wider.
4. **Default to a pipeline.** Justify any barrier by naming which of the three
   legal reasons applies: a cross set operation, an early exit on the total, or a
   comparison of each item against all the others.
5. **Put coordination in code.** Flattening, deduplicating, filtering, ranking
   and sorting between stages are not agents.
6. **Declare each step's file and resource scope,** and serialise or isolate
   overlaps before dispatching anything.
7. **Set the budget** for the mission and per step, and decide the escalation
   point before it is reached.
8. **Dispatch the ready set concurrently,** each step with an owner, a budget and
   the verification command from its brief.
9. **Watch cheaply.** Classify each step as working, stalled, blocked or asking,
   and act accordingly: silence, a nudge, an escalation, an escalation. Never
   answer an escalated question on the human's behalf.
10. **Verify before closing any entry.** Run the named command yourself. The
    delegate's claim and your verification are recorded as two separate facts.
11. **Contain failures per node.** A failed step resolves to null; downstream
    steps tolerate missing inputs; steps that became unreachable are reported as
    unreachable rather than left queued.
12. **Synthesise.** Every child output is represented or explicitly discarded
    with a reason. A fan out with no synthesis is unfinished work.
13. **Close.** Account for every worker started, release every scope claim, and
    signal clean only when every entry is done and verified. Otherwise pending
    with what remains, or failed with the evidence.
14. **Postmortem an overrun or a failure** into a change to the shape or the
    ledger, not into a resolution to be more careful.

## Handoffs

| Direction | Counterpart | Contract |
|---|---|---|
| in | Agent {OS} | approved briefs with mechanically checkable done tests |
| in | AI Logic {OS} | the arbitration behind each node |
| in | Tool & Integration {OS} | failure semantics the topology must tolerate |
| in | Context & Memory {OS} | the compiled context pack per step |
| out | Evaluation {OS} | completed missions to score end to end |
| out | Context & Memory {OS} | the ledger, verifications, outcomes and postmortems |
| out | the requester | one synthesis and one honest closure signal |

The one thing this OS never does is report a mission clean when part of it is
not. That failure ends the work for everyone downstream, which is why the
closure rules are the strictest in the specification.
