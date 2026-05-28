# Auto-Done Implementation Plan
**Priority: CRITICAL** — workers completing 100% of todos silently stall the oracle

## Task 1 — Make `detect_idle_prompt` pub
**File:** `crates/omega-core/src/oracle_lifecycle.rs`
**Change:** `fn detect_idle_prompt` → `pub fn detect_idle_prompt`
**Risk:** Low — pure visibility change, no logic change.

## Task 2 — Add auto-done branch in patrol.rs
**File:** `crates/omega-core/src/patrol.rs`
**Location:** After the stall detection block (after line ~221), still inside `for session in &sessions { if session.role == SessionRole::Worker {`

```rust
// ── AUTO-DONE: 100% todos + idle pane (Claude exited without writing done.json) ──
if !has_done
    && progress.todos_total > 0
    && progress.todos_completed >= progress.todos_total
    && !progress.blocked
{
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

**Required imports already present:** `DoneSignal`, `DoneStatus`, `WorkerStallDetector` all already imported at top of patrol.rs.

## Task 3 — close-gate.sh compatibility (optional for Rust auto-done, required for shell gate)
**File:** `~/.aisb/lib/close-gate.sh`
**Change:** Add `patrol` to the valid `written_by` values at line 427.

**Current:**
```bash
if [ "$DC_WRITER" = "worker-mark-done.sh" ] && ...
```
**Change to:**
```bash
if { [ "$DC_WRITER" = "worker-mark-done.sh" ] || [ "$DC_WRITER" = "patrol" ]; } && ...
```

Or add a `written_by` field to `DoneSignal` (optional — `close-gate.sh` may not be reading Rust-written done.json at all).

## Verification
After implementing Task 1+2:
1. `cargo check --workspace` must pass with 0 errors
2. Create a test state: write a `{session}.progress.json` with `todos_completed=3, todos_total=3`
3. Start `omega patrol --once`
4. Confirm `worker-{session}.done.json` is written with `status: done_clean`
5. Confirm patrol log shows `Auto-done: {session} (3/3 todos, Claude idle)`
