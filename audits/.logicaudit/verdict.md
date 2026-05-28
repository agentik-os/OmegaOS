# /logicaudit verdict — Hermux right-panel responsiveness logic

**Ticket:** hermux-perf · **Score: 82/100 (B+)** · **Confidence: high**
**Scope:** LOGIC + architecture of the responsiveness path across 5 files.

> *Einstein test: "as simple as possible, but no simpler."* This subsystem
> passes it. The hot path is lean and correct; the deductions are optimization
> headroom, not defects.

## BLUF

The committed responsiveness work is **architecturally sound** and **logically
addresses all three user complaints**. The core correctness claim — keystrokes
delivered in FIFO order — holds under falsification. The deductions (82, not
100) are minor, mostly *documented-not-fixed* because changing them would add
complexity or risk a visible regression without runtime proof (First Law).

## How the user's three complaints are solved (logic trace)

| Complaint | Root-cause removed | Where |
|---|---|---|
| "typing is very slow" | per-keystroke blocking RPC (5–15ms) → pane cache + `connect_cached` singleton + off-loop ordered forwarder; loop now does a non-blocking channel `send` | session.rs:117,143,266 · main.rs:578,1050 · forwarder.rs |
| "scroll works badly" | column-aware mouse scroll routes to preview; follow_tail/scroll model + scrollback heavy-path give real history to scroll into | input.rs:111 · app.rs:860-888,1021 |
| "long paste bugs" | forwarded as ONE bracketed-paste block (no per-line Enter), body chunked at 4096B char boundaries | session.rs:497 · forwarder.rs Paste |

## Hinge logic — the load-bearing 10%

The **single ordered forwarder** is the hinge (it owns complaint #1). Verified:
single-threaded sender + order-preserving mpsc + one sequential consumer ⇒ no
reorder path; coalescing is same-session-only and flushes before any Key/Paste.
The bounded drain (8ms < 16ms tick, ZERO-poll exit) cannot starve the redraw.
**This is the right design.**

## Findings

| # | Sev | Finding | Action |
|---|---|---|---|
| F1 | LOW | Dead `fn shell_escape` (session.rs:575), 0 callers | **FIXED** — removed, build re-verified |
| F2 | MED | Space routed as Key 'Space' (main.rs:1039) flushes the forwarder, defeating Text coalescing for prose (~1 RPC/word vs 1/burst) — on the user's prime hot path. Justifying comment likely a misdiagnosis. | DOCUMENTED — verify `send_text(" ")` echo on a live session, then route space as Text. Not applied blind (regression risk, First Law). |
| F3 | LOW | `capture_pane_history(1000)` re-spawns `rmux` subprocess ~12×/s re-fetching identical lines while browsing history | DOCUMENTED — bounded, off the typing hot path; SDK has no scrollback API |
| F4 | LOW | `send_paste_raw` lacks the stale-pane retry its siblings have (mid-paste kill leaves no closer) | DOCUMENTED — rare; correct retry is non-trivial |
| F5 | LOW | `refresh_preview` triggered from 3 sites/loop — rare same-tick double capture | DOCUMENTED — each trigger has a distinct purpose |
| F6 | LOW | Cold action handlers use `connect()` (fresh socket) vs `connect_cached()` | DOCUMENTED — one-shot paths, imperceptible |

## Why not 100

Genuine optimization headroom remains (F2 especially, since it touches the
exact prose-typing path the user complained about). But the responsive
*feature* is not broken — it is solidly built. Reporting 82 over a gamed 100 is
the honest call (Second Law: no fake confidence). The single safe, verified fix
was applied; the rest are deliberate trade-offs with a recommended verify-first
path for F2.

## Scope-drift note
`hinge-points.json` was generated at HEAD~1 over a *different* file set
(telegram_bridge.rs, patrol.rs, ui.rs); only main.rs + app.rs overlap this
audit's 5 files. The 3 overlapping hinges were verified; forwarder.rs/
session.rs/input.rs had no hinge entries despite hosting the core logic. Same
drift was noted in the prior feature audit (obs 45250) — the hinge ref should
be regenerated against the actual scoped diff next time.
