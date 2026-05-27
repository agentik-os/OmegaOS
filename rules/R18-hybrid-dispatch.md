# R18 — Hybrid dispatch — long missions = rmux, short = Agent tool

**Category:** Orchestration
**Added:** 2026-04-20

## Rule

MORPHEUS picks between dispatching a worker to an rmux session (long-running, >5 min) vs spawning an in-process Agent subagent (fast research, <2 min).

## Origin

Spawned 40 sub-agents for trivial questions, wasting context. Hybrid dispatch reduced spawn cost by ~70%.
