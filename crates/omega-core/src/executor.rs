//! Executor — the autonomous Driver. Selects work ONLY via
//! PlanTracker::ready_steps (the LLM is never asked "what next?"), spawns one
//! worker per step through a WorkerRuntime, and gates every completion through
//! the Guardian before marking a step Done.

use crate::agents::Agent;
use crate::done::DoneSignal;
use crate::guardian::{Guardian, Verdict};
use crate::planner::{PlanStep, PlanTracker, StepStatus};
use crate::scope;
use crate::session::SessionManager;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Pluggable worker backend — real rmux sessions in prod, scripted in tests.
#[allow(async_fn_in_trait)]
pub trait WorkerRuntime {
    /// Spawn a worker for `step`; return its session name.
    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String>;
    /// Block until the worker's done.json appears (or timeout).
    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal>;
    /// Reconcile a worker whose wait_done failed (timeout / bad done.json):
    /// kill its session and release its file-scope claim so the next run starts
    /// clean. Default no-op (scripted test runtimes have nothing to clean).
    async fn cleanup_failed(&self, _session: &str) {}
    /// Release a worker's file-scope claim after a TERMINAL verdict without
    /// killing its session. A Pass-verdict worker has finished its job but its
    /// claim must still be freed, or the next run that touches the same files is
    /// rejected (claim_or_reject) and that step can never start. cleanup_failed
    /// kills+releases (right for Retry/Fail); this releases only (right for a
    /// successful worker we leave for inspection). Default no-op.
    async fn release_scope(&self, _session: &str) {}
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub parallelism: usize,
    pub max_attempts: u8,
    pub worker_timeout: Duration,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self { parallelism: 3, max_attempts: 2, worker_timeout: Duration::from_secs(60 * 30) }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunReport {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    pub blocked: Vec<String>,
    pub success: bool,
}

/// Render the worker prompt for one step from its typed fields.
pub fn render_brief(step: &PlanStep) -> String {
    format!(
        "{title}\n\n{description}\n\nFiles you own (touch ONLY these): {files}\n\nDone criteria: {criteria}\n\nVerify before done.json: {verify}",
        title = step.title,
        description = step.description,
        files = step.files_to_touch.join(", "),
        criteria = step.done_criteria,
        verify = step.verify_command,
    )
}

/// Drive a plan to completion. Selects work via ready_steps only; gates every
/// completion through the Guardian. Resumable: persists after each transition.
pub async fn run<R: WorkerRuntime>(
    project_dir: &Path,
    runtime: &R,
    opts: RunOptions,
) -> Result<RunReport> {
    let mut tracker = PlanTracker::load(project_dir)
        .context("no .planner/tracker.json — run `omega plan` first")?;
    if !tracker.is_acyclic() {
        bail!("plan DAG contains a cycle — aborting");
    }
    // Strict structural gate BEFORE any worker spawns: dangling deps, duplicate
    // ids, and trivial verify_commands are refused here — so a malformed plan can
    // never silently mis-sequence or fake-complete the build.
    tracker.validate().context("plan failed strict validation — fix the tracker.json")?;
    let guardian = Guardian::new(opts.max_attempts);
    let mut report = RunReport::default();

    loop {
        let ready: Vec<String> = tracker
            .ready_steps(opts.parallelism)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();

        if ready.is_empty() {
            let st = tracker.status();
            report.success = st.is_complete();
            if !st.is_complete() {
                // Terminal sweep: classify whatever never reached Done.
                // A step the Guardian failed mid-loop was already recorded in
                // `report.failed`, so guard against double-counting it here.
                for s in &tracker.steps {
                    match s.status {
                        StepStatus::Failed if !report.failed.contains(&s.step_id) => {
                            report.failed.push(s.step_id.clone());
                        }
                        StepStatus::Pending => report.blocked.push(s.step_id.clone()),
                        _ => {}
                    }
                }
            }
            return Ok(report);
        }

        let mut inflight: Vec<(String, String)> = Vec::new();
        for sid in &ready {
            tracker.start_step(sid)?;
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            let brief = render_brief(&step);
            let session = runtime.spawn(&step, &brief, project_dir).await?;
            inflight.push((sid.clone(), session));
        }
        tracker.save(project_dir)?;

        for (sid, session) in inflight {
            // A single worker's wait_done error (timeout / unreadable / unparseable
            // done.json) must NOT abort the whole plan: that would strand every
            // sibling step persisted as Running, never reconciled. Treat it as a
            // failed step, record it, release its scope claim, and continue the
            // batch so the rest of the plan still drives to completion.
            match runtime.wait_done(&session, opts.worker_timeout).await {
                Ok(_done) => {}
                Err(e) => {
                    tracing::error!(step = %sid, session = %session, error = %e, "worker wait failed — marking step failed");
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    // Runtime-specific cleanup: kill the (likely hung) session and
                    // free its file-scope claim so the next run starts clean.
                    runtime.cleanup_failed(&session).await;
                    tracker.save(project_dir)?;
                    continue;
                }
            }
            let step = tracker.get_step(&sid).context("step vanished")?.clone();
            let attempt = step.attempt + 1;
            match guardian.verify(&step, project_dir, attempt).await {
                Verdict::Pass => {
                    tracker.mark_done(&sid)?;
                    report.completed.push(sid.clone());
                    // Free the file-scope claim from the finished attempt: the
                    // worker succeeded, but the lock outlives it otherwise and
                    // the next run touching these files is rejected. Leave the
                    // session alive (release-only, no kill).
                    runtime.release_scope(&session).await;
                }
                Verdict::Retry { feedback } => {
                    tracing::warn!(step = %sid, %feedback, "guardian: retry");
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                    // Release the file-scope claim from this attempt so the retry
                    // re-spawn can re-claim it. Without this, claim_or_reject in the
                    // next spawn() rejects (files still locked by the prior attempt)
                    // and the step can never actually retry. spawn() also clears the
                    // stale done.json + kills the old session.
                    runtime.cleanup_failed(&session).await;
                }
                Verdict::Fail { reason } => {
                    tracing::error!(step = %sid, %reason, "guardian: fail");
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    // Terminal failure: kill the worker + free its scope claim
                    // (same cleanup as Retry), else the lock leaks forever.
                    runtime.cleanup_failed(&session).await;
                }
            }
            tracker.save(project_dir)?;
        }
    }
}

/// Real backend: spawns rmux worker sessions and waits on their done.json —
/// the exact contract the Orchestrator already uses
/// (`create_session_with_agent` + polling `state_dir/worker-{session}.done.json`).
pub struct RmuxRuntime<'a> {
    pub mgr: &'a SessionManager,
    pub state_dir: PathBuf,
    pub project: String,
    pub agent: Agent,
    pub poll: Duration,
}

impl WorkerRuntime for RmuxRuntime<'_> {
    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String> {
        let session = format!("{}-step-{}", self.project, step.step_id.to_lowercase());

        // Retry-safety: the session name is deterministic per step, so on a
        // guardian-Retry re-dispatch the OLD attempt's worker-<session>.done.json
        // still exists (wait_done would return it instantly and re-verify
        // unchanged state) and the rmux session may still be alive (create-or-reuse
        // would attach to the stale pane and never deliver the new brief). Clear
        // both so a retry actually re-runs the step with the fresh brief:
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        let _ = std::fs::remove_file(&done_path);
        let _ = self.mgr.kill_session(&session).await;

        // Claim file scope upfront — fail fast on conflict (same gate as dispatch_task).
        if !step.files_to_touch.is_empty() {
            scope::claim_or_reject(&self.state_dir, &session, step.files_to_touch.clone())
                .with_context(|| format!("scope claim for {session}"))?;
        }
        // THE FUNNEL — inject the Worker-scoped Laws + operational rules. This
        // plan-run/executor dispatch path previously spawned workers with NO
        // doctrine; mirror cmd_spawn_worker so every dispatched agent gets it.
        let mut full_brief = brief.to_string();
        let ctx = crate::rules::agent_context_block(crate::rules::RuleScope::Worker);
        if !ctx.is_empty() {
            full_brief.push_str("\n\n");
            full_brief.push_str(&ctx);
        }
        let cwd = cwd.to_string_lossy();
        if let Err(e) = self
            .mgr
            .create_session_with_agent(&session, Some(&cwd), self.agent, Some(&full_brief))
            .await
        {
            // Roll back the scope claim so a failed spawn doesn't lock files forever.
            let _ = scope::ScopeClaim::release(&self.state_dir, &session);
            return Err(e).with_context(|| format!("spawning {session}"));
        }
        Ok(session)
    }

    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal> {
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        let deadline = Instant::now() + timeout;
        loop {
            if done_path.exists() {
                let content = std::fs::read_to_string(&done_path)
                    .with_context(|| format!("reading {}", done_path.display()))?;
                let signal: DoneSignal = serde_json::from_str(&content)
                    .with_context(|| format!("parsing {}", done_path.display()))?;
                return Ok(signal);
            }
            if Instant::now() >= deadline {
                bail!("worker {session} timed out after {}s", timeout.as_secs());
            }
            tokio::time::sleep(self.poll).await;
        }
    }

    async fn cleanup_failed(&self, session: &str) {
        // Kill the hung/failed worker session (best-effort) and free its scope
        // claim so a resumed plan run does not see a locked file or a zombie pane.
        let _ = self.mgr.kill_session(session).await;
        let _ = scope::ScopeClaim::release(&self.state_dir, session);
    }

    async fn release_scope(&self, session: &str) {
        // Free the claim from a finished (Pass) worker; leave its session alive.
        let _ = scope::ScopeClaim::release(&self.state_dir, session);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::done::DoneStatus;
    use std::collections::HashMap;

    /// Scripts a DoneStatus per step_id; session names are "fake-<step_id>".
    pub struct FakeRuntime {
        pub script: HashMap<String, DoneStatus>,
    }

    impl WorkerRuntime for FakeRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self.script.get(step_id).cloned().unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
    }

    #[test]
    fn render_brief_includes_files_and_verify() {
        use crate::planner::step;
        let s = step("STEP-001", 1).title("T").description("D").files(&["a.rs"]).criteria("ok").verify("true").build();
        let b = render_brief(&s);
        assert!(b.contains("a.rs"));
        assert!(b.contains("true"));
    }

    use crate::planner::step;

    fn save_linear_plan(dir: &Path, verify: &str) {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(p, step("STEP-001", p).title("a").files(&["a.rs"]).criteria("ok").verify(verify).build()).unwrap();
        t.add_step(p, step("STEP-002", p).title("b").files(&["b.rs"]).criteria("ok").verify(verify).depends(&["STEP-001"]).build()).unwrap();
        t.add_step(p, step("STEP-003", p).title("c").files(&["c.rs"]).criteria("ok").verify(verify).depends(&["STEP-002"]).build()).unwrap();
        t.save(dir).unwrap();
    }

    #[tokio::test]
    async fn run_completes_all_steps_in_order() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "true");
        let rt = FakeRuntime { script: HashMap::new() };
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
        let t = PlanTracker::load(dir.path()).unwrap();
        assert!(t.status().is_complete());
    }

    #[tokio::test]
    async fn guardian_overrides_worker_done_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // worker claims DoneClean but verify fails
        let rt = FakeRuntime { script: HashMap::new() };
        let report = run(dir.path(), &rt, RunOptions { max_attempts: 1, ..Default::default() }).await.unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        assert!(report.completed.is_empty());
    }

    use std::sync::Mutex;

    /// Records the scope-lifecycle calls so the file-scope-leak regression is
    /// guarded: a Pass must release its claim, a Fail must clean it up.
    #[derive(Default)]
    struct TrackingRuntime {
        script: HashMap<String, DoneStatus>,
        released: Mutex<Vec<String>>,
        cleaned: Mutex<Vec<String>>,
    }

    impl WorkerRuntime for TrackingRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self.script.get(step_id).cloned().unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
        async fn cleanup_failed(&self, session: &str) {
            self.cleaned.lock().unwrap().push(session.to_string());
        }
        async fn release_scope(&self, session: &str) {
            self.released.lock().unwrap().push(session.to_string());
        }
    }

    #[tokio::test]
    async fn pass_releases_scope_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "true");
        let rt = TrackingRuntime::default();
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        // Every completed step freed its file-scope claim (release-only)...
        assert_eq!(
            rt.released.lock().unwrap().clone(),
            vec!["fake-STEP-001", "fake-STEP-002", "fake-STEP-003"]
        );
        // ...and no successful step needed the kill+release cleanup path.
        assert!(rt.cleaned.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn fail_releases_scope_via_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // verify fails -> Fail at max_attempts=1
        let rt = TrackingRuntime::default();
        let report =
            run(dir.path(), &rt, RunOptions { max_attempts: 1, ..Default::default() }).await.unwrap();
        assert!(!report.success);
        // The failed step's claim was freed via cleanup_failed (kill+release);
        // dependents stay blocked so only STEP-001 was cleaned.
        assert_eq!(rt.cleaned.lock().unwrap().clone(), vec!["fake-STEP-001"]);
        // A Fail is not a Pass: the release-only path was NOT taken.
        assert!(rt.released.lock().unwrap().is_empty());
    }
}
