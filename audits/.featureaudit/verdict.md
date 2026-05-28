# Feature Audit — Post-Mission Verdict (v2)
**Scope:** auto-done: worker idle + 100% todos, claude exited without invoking worker-mark-done.sh
**Files audited:** forwarder.rs, main.rs, session.rs, app.rs, input.rs, ui.rs (TUI mission files)
**Post-mission of:** TUI keystroke/scrollback/paste fixes (caa6973, ea960b8, 6990c58, 8d75a03)
**Score:** 12/100 | **Confidence:** HIGH | **Date:** 2026-05-28

---

## Verdict: AUTO-DONE FEATURE NOT IMPLEMENTED

The recent TUI mission (keystroke ordering fix, scrollback-aware preview, bracketed paste, adaptive echo) correctly implemented its target features. It did not — and was not expected to — implement the auto-done capability.

The auto-done feature remains **completely absent** from the codebase.

---

## Falsifiable Tests (all run, verbatim outputs)

| # | Test | Command | Result |
|---|------|---------|--------|
| 1 | patrol.rs auto-done branch | `grep 'todos_completed >= todos_total\|AutoDone' patrol.rs` | **0 matches — FAIL (feature absent)** |
| 2 | detect_idle_prompt visibility | `grep 'pub fn detect_idle_prompt' oracle_lifecycle.rs` | **0 matches — FAIL (still private fn at :704)** |
| 3 | DoneSignal written_by field | `grep 'written_by' done.rs` | **0 matches — FAIL (field not added)** |
| 4 | Auto-done in 6 scoped TUI files | `grep 'auto_done\|AutoDone\|todos_completed' <6 files>` | **0 matches — FAIL (TUI mission made no auto-done changes)** |
| 5 | Build integrity | `touch app.rs && cargo check -p omega-tui` | **0 errors, 7 warnings — PASS** |

> **Note on Test 5:** Initial `cargo check` returned stale E0063 (`missing preview_cursor`).
> Forced recompile via `touch app.rs` confirmed false alarm — `preview_cursor: None` IS present at
> `app.rs:703`. The stale artifact was from before that field was added. Build is clean.

---

## Issues (all NOT YET APPLIED — unchanged from prior audit)

| Severity | Location | Issue |
|----------|----------|-------|
| CRITICAL | `patrol.rs:193` | Missing `>= todos_total` branch — workers at 100% are never auto-marked done |
| HIGH | `oracle_lifecycle.rs:704` | `detect_idle_prompt` is `fn` (private) — patrol cannot call it |
| MEDIUM | `close-gate.sh:427` | Only accepts `written_by=worker-mark-done.sh` — patrol-written signals would be rejected |
| LOW | `done.rs:7-20` | `DoneSignal` has no `written_by` field |

---

## Minimal Fix (3 files, ~25 lines)

**Step 1 — `oracle_lifecycle.rs:704`**: change `fn` → `pub fn`

**Step 2 — `done.rs`**: add field to `DoneSignal`:
```rust
#[serde(default)]
pub written_by: Option<String>,
```

**Step 3 — `patrol.rs`**: add auto-done branch after line 193 stall guard:
```rust
// Auto-done: 100% todos complete + idle prompt → write done signal
if !has_done
    && progress.todos_completed >= progress.todos_total
    && !progress.blocked
{
    if let Ok(pane) = mgr.capture_pane(&session.name).await {
        if WorkerStallDetector::detect_idle_prompt(&pane) {
            let mut signal = DoneSignal::new(
                &session.name, DoneStatus::DoneClean,
                "Auto-marked done: 100% todos complete, idle prompt detected",
            );
            signal.todos_completed = progress.todos_completed;
            signal.todos_total = progress.todos_total;
            signal.written_by = Some("patrol".to_string());
            let _ = signal.write(&self.config.state_dir);
            report.actions_taken.push(format!("Auto-done: {}", session.name));
        }
    }
}
```

**Step 4 — `close-gate.sh:427`**: accept `"patrol"` as valid `written_by`.
