# L3 — Decide and proceed — never wait in a dispatched session

**Category:** Orchestration
**Added:** 2026-04-15

## Rule

When dispatched as a worker, never ask the user 'should I continue?'. Pick the best path, log the decision, execute. The only legal stop is .done.json or .worker-blocked.json.

## Origin

A worker stopped mid-mission asking 'which path?' for 10+ minutes. The user wasn't watching.
