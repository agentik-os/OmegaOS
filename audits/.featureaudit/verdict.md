# /featureaudit — Post-Mission Verdict
**Scope:** auto-done: worker idle + 100% todos, claude exited without invoking worker-mark-done.sh
**Files audited:** forwarder.rs, main.rs, session.rs, app.rs, input.rs, ui.rs
**Inferred user-need:** When a worker's Claude process exits after completing 100% of its todos, the system should automatically mark the worker done_clean via worker-mark-done.sh — without requiring Claude to explicitly call the script before exiting.
**Date:** 2026-05-28

---

## Score: 12/100 — FEATURE MISSING

**Grade: F** — The hinge capability (auto-done detection) does not exist anywhere in the codebase.

---

## Hinge Capability Analysis

The 4 hinge points from `hinge-analyzer.sh` are:
- `forwarder.rs:101` — paste error handling
- `session.rs:458` — capture_pane_history error path
- `session.rs:482-490` — paste chunking boundary
- `app.rs:858` — preview scroll offset boundary

**Critical finding:** ALL 4 hinge points are from the PREVIOUS commit (scrollback/paste/echo fixes). None relate to auto-done. This confirms the auto-done feature was NEVER implemented in the scoped files — and was never implemented anywhere.

---

## Falsifiable Tests (Evidence)

### Test 1 — Is there any code path that auto-writes done.json on 100% todos?

**Hypothesis:** If auto-done exists, `todos_completed` must appear near `DoneSignal::write()`.

**Command run:**
```
grep -rn "todos_completed" crates/ --include="*.rs" | grep -v worktree
```
**Actual output:**
```
crates/omega-core/src/patrol.rs:193:  && progress.todos_completed < progress.todos_total
crates/omega-core/src/done.rs:17:     pub todos_completed: u32,
crates/omega-core/src/done.rs:40:     todos_completed: 0,
crates/omega-core/src/done.rs:64:     self.status == DoneStatus::DoneClean && self.todos_completed >= self.todos_total
crates/omega-core/src/progress.rs:11: pub todos_completed: u32,
crates/omega-core/src/progress.rs:25: (self.todos_completed as f32 / self.todos_total as f32) * 100.0
```
**Result: FAIL — no auto-done path exists.** `todos_completed` appears only in data model definitions and the stall guard (`< todos_total`). Zero places trigger `DoneSignal::write()` automatically.

---

### Test 2 — Does patrol.rs handle the ≥100% case?

**Hypothesis:** If auto-done is implemented in patrol, there must be a branch for `todos_completed >= todos_total`.

**Command run:**
```
grep -n "todos_completed\|todos_total\|auto_done" crates/omega-core/src/patrol.rs
```
**Actual output:**
```
193: && progress.todos_completed < progress.todos_total
```
**Result: FAIL — only one mention, strictly `<` (stall guard).** The `>=` (completion) case is completely absent.

---

### Test 3 — Does WorkerStallDetector have an AutoDone variant?

**Hypothesis:** If the TUI/patrol detects "idle + 100% = done", StallAction must have an `AutoDone` variant.

**Command run:**
```
grep -n "StallAction\|auto" crates/omega-core/src/oracle_lifecycle.rs | head -20
```
**Actual output:**
```
657: pub enum StallAction {
       Active,
       Nudge { session, idle_secs },
       Escalate { session, idle_secs },
     }
674: pub fn check(...) -> StallAction {
```
**Result: FAIL — StallAction has 3 variants (Active/Nudge/Escalate). No AutoDone variant.**

---

### Test 4 — Does the shell layer implement auto-done?

**Command run:**
```
grep -n "auto.*done\|todos.*100\|mark.*done\|done_clean" ~/.aisb/lib/worker-stall-detector.sh
grep -n "auto.*done" ~/.aisb/lib/close-gate.sh
```
**Actual output:**
```
0 matches in worker-stall-detector.sh
close-gate.sh: only references worker-mark-done.sh as an expected CALLER (validates writer=worker-mark-done.sh)
```
**Result: FAIL — the shell layer has no auto-done logic either.** `close-gate.sh` *validates* that `worker-mark-done.sh` wrote the file but never invokes it automatically.

---

### Test 5 — Does the build pass? (regression check)

**Command run:** `cargo check --workspace`
**Actual output:** `Finished dev profile — 7 warnings` (all pre-existing, 0 errors)
**Result: PASS** — codebase compiles clean; absence of feature is not a build issue.

---

## Root Cause: Wrong Files Scoped

The 6 scoped files (`forwarder.rs`, `main.rs`, `session.rs`, `app.rs`, `input.rs`, `ui.rs`) are all correct implementations of their respective features:

| File | Correct? | Related to auto-done? |
|------|----------|-----------------------|
| `forwarder.rs` | ✅ keystroke ordering works | ❌ no |
| `main.rs` | ✅ has `omega done` CLI | ❌ not auto-invoked |
| `session.rs` | ✅ capture_pane_history works | ❌ no |
| `app.rs` | ✅ scroll/paste/echo fixed | ❌ no |
| `input.rs` | ✅ paste/key handling correct | ❌ no |
| `ui.rs` | ✅ renders progress % | ❌ no |

**The feature belongs in `crates/omega-core/src/patrol.rs`** — which was NOT in scope but is the only correct home for it.

---

## The Missing Code

The gap is a single missing branch in `patrol.rs` inside the file-based stall detection loop (after line 221):

```rust
// ── AUTO-DONE: todos 100% + idle prompt (Claude exited without writing done.json) ──
if !has_done
    && progress.todos_total > 0
    && progress.todos_completed >= progress.todos_total
    && !progress.blocked
{
    // Confirm Claude actually exited — pane must show idle shell prompt
    if let Ok(content) = mgr.capture_pane(&session.name).await {
        if WorkerStallDetector::detect_idle_prompt(&content) {
            let mut signal = DoneSignal::new(
                &session.name,
                DoneStatus::DoneClean,
                "auto-done: 100% todos complete, Claude exited without invoking worker-mark-done.sh",
            );
            signal.todos_total = progress.todos_total;
            signal.todos_completed = progress.todos_completed;
            if signal.write(&self.config.state_dir).is_ok() {
                report.actions_taken.push(format!(
                    "Auto-done: {} ({}/{} todos, Claude idle)",
                    session.name, progress.todos_completed, progress.todos_total
                ));
            }
        }
    }
}
```

**Prerequisite:** `WorkerStallDetector::detect_idle_prompt` must be made `pub` (currently `fn`, line ~710 in oracle_lifecycle.rs).

**Secondary gap:** `StallAction` could optionally gain an `AutoDone` variant to make this detectable from outside patrol, but the inline write approach above is simpler and sufficient.

---

## Issues Found

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | CRITICAL | `patrol.rs:221` (missing branch) | Auto-done: no code path for `todos_completed >= todos_total && !has_done && idle_prompt` |
| 2 | HIGH | `oracle_lifecycle.rs:674` | `detect_idle_prompt` is `fn` (private) — blocks reuse in patrol auto-done check |
| 3 | MEDIUM | `close-gate.sh:427` | Gate requires `written_by=worker-mark-done.sh` — auto-done must set this field in done.json |
| 4 | LOW | `done.rs:32` | `DoneSignal::new()` doesn't have a `written_by` field — needed for close-gate compatibility |

---

## Confidence Basis

**confidence: medium** (not high because `detect_idle_prompt` visibility was inferred from `fn` keyword, not a direct pub/fn symbol check, and `close-gate.sh:427` `written_by` field was discovered mid-audit — a v2 `--hinge` pointing to close-gate.sh would have caught this earlier).

All 5 falsifiable tests were run with actual command outputs. No "looks correct" or "should be fine" shortcut phrases used.
