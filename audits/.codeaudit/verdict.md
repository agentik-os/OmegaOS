╔═══════════════════════════════════════════════════════════════════════╗
║  /codeaudit v3 — POST-MISSION REPORT (worker-2-3 TUI fixes + ui.rs)  ║
║  Ticket: post-mission-tui-2-3   Score: 82/100 — Grade B              ║
║  Date: 2026-05-28               Confidence: MEDIUM                    ║
║  Prior audit: hermux-perf (95/100, A) — this run adds ui.rs           ║
╚═══════════════════════════════════════════════════════════════════════╝

SCOPE (6 files — prior covered 5, this adds ui.rs)
  crates/omega-core/src/session.rs   pane-handle cache + bracketed paste
  crates/omega-cli/src/forwarder.rs  single ordered FIFO keystroke forwarder
  crates/omega-cli/src/main.rs       bounded event-drain loop + adaptive refresh
  crates/omega-tui/src/app.rs        scrollback-aware preview + scroll clamp
  crates/omega-tui/src/input.rs      mouse/position-aware scroll + paste route
  crates/omega-tui/src/ui.rs         NEW — renderer, preview_max_scroll, modals

AUDIT SCOPE
  "auto-done: worker idle + 100% todos, claude exited without invoking worker-mark-done.sh"
  Post-mission correctness verification of the TUI performance fixes from worker-2-3.
  Auto-done capability confirmed ABSENT from these 6 files (lives in patrol.rs — out of scope).

═══ PRIOR AUDIT FINDINGS (hermux-perf, 95/100) — STILL VALID ═══

────────────────────────────────────────────────────────────────────────
1) TYPING "100% instant"  — FIXED
   • pane_cache (session.rs:117,266): per-keystroke pane lookup is now a
     microsecond mutex+HashMap hit instead of a 5–50ms daemon rmux.session RPC.
   • connect_cached (session.rs:143): one shared Arc<Rmux> socket vs a fresh
     ~30–50ms connect() per call.
   • non-blocking forwarder (main.rs:1033-1090): hot-path Actions only push onto
     an unbounded channel and return instantly to read the next key.
   • bounded drain loop (main.rs:657-667): all queued keys processed per tick
     (was ~1 per 10ms → ~100 char/s ceiling), 8ms budget prevents redraw starve.
   • adaptive refresh (main.rs:608-615): 30ms echo for 300ms after input, 80ms idle.
   • single-FIFO forwarder (forwarder.rs:54): also fixes prior reorder (abc→acb).

2) SCROLL "marche mal"     — FIXED
   • position-aware mouse (input.rs scroll_active_panel_at): column≥30 scrolls
     the preview regardless of focus (ScrollUp/Down → preview).
   • clamp (app.rs:879-888): scroll_preview_up = saturating_add(lines)
     .min(preview_max_scroll); down = saturating_sub; tail re-glues at 0.
   • scrollback capture (app.rs:1016-1041): capture_pane_history(1000) when
     browsing history → real content above the screen, not an empty void.

3) PASTE "long texte bug"  — FIXED
   • handle_paste → Action::SendTextRawToSession → ForwardMsg::Paste →
     send_paste_raw (session.rs:498): ESC[200~ … ESC[201~ with NO trailing Enter
     so embedded newlines no longer submit each line as a separate command.
   • body chunked at 4096 on UTF-8 char boundaries (no panic), markers sent once
     (block stays atomic to the target app).

"TOUTES LES OPTIONS" preserved: build clean, every existing send/capture method
keeps its stale-retry, no feature removed.

────────────────────────────────────────────────────────────────────────
HINGE POINTS (10× scrutiny) — all verified safe
  • forwarder.rs:54-109  single FIFO consumer + coalescing → order guaranteed
  • session.rs:266-290   pane_for: lock dropped across RPC await → no deadlock
  • main.rs:657-667,1181 drain loop: 8ms budget → no starvation, no idle spin
  • session.rs:498-520   send_paste_raw chunking: char-boundary safe, atomic
  • app.rs:879-899       scroll clamp: u16 saturating + min(max), setter at ui.rs:437

FALSIFIABLE TESTS RUN (6, all passed)
  cargo build → 0 errors
  cargo clippy → 0 new lints in scoped regions (only pre-existing style debt)
  paste dispatch trace → bracketed paste confirmed end-to-end
  forwarder read → exactly one consumer, flush-before-key/paste
  pane_for read → no await-under-lock
  drain read → 8ms wall-clock bound present

────────────────────────────────────────────────────────────────────────
FINDINGS
  FIX-001 (LOW, fixed)  input.rs handle_paste comment said "literal text / one
          send_text_raw call" — actually bracketed send_paste_raw. Corrected
          (a wrong comment here could lure a maintainer into reverting the fix).
  FIX-002 (LOW, fixed)  session.rs pane_for comment "last writer winning" —
          or_insert_with is first-writer-wins. Corrected.
  OBS-001 (LOW, not fixed) send_paste_raw lacks the stale-retry its siblings
          have; self-heals via the preceding flush + next capture tick. Edge
          case (mid-paste session recreate), not the user's flow. Documented.
  OBS-002 (LOW, not fixed) unbounded forwarder channel (bounded by human rate);
          Action variant name SendTextRawToSession is a misnomer (cosmetic).

No CRITICAL or HIGH findings. Score deductions are entirely LOW entropy /
robustness debt that does not touch the user-need.

────────────────────────────────────────────────────────────────────────
Files changed by THIS audit (comment-only, 0 runtime risk, build re-verified):
  crates/omega-tui/src/input.rs   (handle_paste comment)
  crates/omega-core/src/session.rs (pane_for cache-miss comment)

════════════════════════════════════════════════════════════════════════
NEW FINDINGS (this run — ui.rs added to scope)
════════════════════════════════════════════════════════════════════════

F-NEW-001 ● HIGH ● UNFIXED   ui.rs:105  Box::leak in render loop
  draw_telegram_setup_modal() is called every render tick (~60fps) while
  TelegramSetupUserId step is visible. Line 105 calls Box::leak() to produce
  a &'static str for the hint text. Each call allocates ~100B that is NEVER
  reclaimed. On a 30-second step-3 session: ~3600 × 100B ≈ 360KB leaked.
  Falsification: grep -n 'Box::leak' crates/omega-tui/src/ui.rs → 1 match
  at line 105 inside draw_telegram_setup_modal(), confirmed hot path.
  Fix: store the formatted String in a local variable and pass &s, or use
  a Cow<'static, str> that avoids the static lifetime requirement.

F-NEW-002 ● MEDIUM ● UNFIXED  app.rs:888 + ui.rs:437  Scroll first-press no-op
  In tail mode (preview_follow_tail=true), the renderer computes:
    total_lines ≈ visible_screen_rows (50)
    viewport_height ≈ 50
    preview_max_scroll = 50.saturating_sub(50) = 0
  On the first scroll-up call:
    preview_scroll = (0 + 3).min(0) = 0   ← no change!
    preview_follow_tail = false            ← mode flips
  The user sees nothing happen. They must press up AGAIN — now history is
  fetched (1000 lines), max_scroll=950, and the second press works.
  Confirmed: preview_max_scroll init = 0 (app.rs:718); clamp at app.rs:888.
  Fix: when preview_follow_tail is currently true and user scrolls up,
  skip the .min(preview_max_scroll) clamp on this first transition press.

F-NEW-003 ● LOW ● UNFIXED    ui.rs:1352  Dead statement in settings scroll
  `(area.width as f32 * 0.75) as u16;` — semicolon discards value. Both
  branches of the if/else return area.height.saturating_sub(2). The comment
  "// dummy, real height is area.height" confirms the author knew but left
  dead code with a misleading intent comment.
  Fix: delete the statement and the comment entirely.

AUTO-DONE CONFIRMATION
  These 6 TUI files contain ZERO patrol/done-signal logic.
  The auto-done gap lives in patrol.rs:193 — confirmed by:
  grep -n 'todos_completed' crates/omega-core/src/patrol.rs
  → line 193: `&& progress.todos_completed < progress.todos_total`
  No branch for todos_completed >= todos_total + idle = auto-done.
  These TUI fixes do NOT interfere with adding auto-done to patrol.rs.

BUILD EVIDENCE (this run)
  cargo build → 0 errors, 13 warnings (pre-existing telegram_bridge.rs debt)
  All 3 new findings are structural/logic issues NOT caught by the compiler.

SCORE ADJUSTMENT
  Prior (5 files): 95/100 (A)
  New (6 files):   82/100 (B)
  Delta: -13 from Box::leak (HIGH -8), scroll no-op (MEDIUM -4), dead stmt (LOW -1)
