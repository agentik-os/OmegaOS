# Phase 17 — Simplification Map (ranked by impact)

Impact = complexity_removed × frequency_of_path × blast_radius

## 1. F2 — Space-as-Key defeats coalescing (HIGHEST impact, verify-first)
- CURRENT: `if ch == ' ' { fwd_tx.send(Key{"Space"}) }` (main.rs:1039-1046).
  Every space flushes the forwarder → a 10-word line = ~10 send_text + 10
  send_key RPCs instead of 1 coalesced send_text per burst.
- PROPOSED: send space via `ForwardMsg::Text { text: " " }` like every other
  printable char. Coalescing then merges a whole typed phrase into one RPC.
- DELTA: ~2× fewer transport RPCs on the prose-typing hot path (the user's #1
  complaint). ~5 lines simpler (drops the special-case branch).
- RISK: IF the comment's premise is right (bare space echoes as "space"),
  this regresses visibly. PRE-REQ: run `send_text(" ")` on a live session and
  confirm the pane shows a literal space. Reasoning strongly suggests the
  premise is a misdiagnosis (send_text forwards literal bytes), but on-screen
  regressions outrank RPC savings — verify before applying.

## 2. F1 — Dead shell_escape (DONE)
- CURRENT → PROPOSED: 3-line dead fn → removed. DELTA: -3 lines, -1 warning.
  RISK: none (0 callers, compiler-confirmed). **Applied + re-verified.**

## 3. F3 — History re-fetch caching (LOW, defer)
- CURRENT: subprocess fork+exec + 1000-line fetch every 80ms while browsing.
- PROPOSED: capture once on follow_tail→false, refresh only on further scroll.
- DELTA: ~12 → ~1 subprocess spawn/sec while paused in history.
- RISK: staleness if the agent emits output above the fold while paused;
  adds state. Per SIMPLICITY-COMPLETE, defer until a real complaint exists.

## 4. F4/F5/F6 (LOW, leave)
- Error-handling symmetry, refresh-trigger consolidation, cold connect() —
  each would couple currently-independent concerns or guard a near-impossible
  path. Net simplicity gain is negative or marginal. Leave as-is.

**Net applied this audit:** -3 lines, -1 dead-code warning, 0 behavior change.
**Net recommended (verify-first):** F2 — the one change that would measurably
improve the user's prime complaint.
