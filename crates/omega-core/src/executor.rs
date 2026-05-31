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
            let _done: DoneSignal = runtime.wait_done(&session, opts.worker_timeout).await?;
            let step = tracker.get_step(&sid).context("step vanished")?.clone();
            let attempt = step.attempt + 1;
            match guardian.verify(&step, project_dir, attempt).await {
                Verdict::Pass => {
                    tracker.mark_done(&sid)?;
                    report.completed.push(sid.clone());
                }
                Verdict::Retry { feedback } => {
                    tracing::warn!(step = %sid, %feedback, "guardian: retry");
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                }
                Verdict::Fail { reason } => {
                    tracing::error!(step = %sid, %reason, "guardian: fail");
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
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
        // Claim file scope upfront — fail fast on conflict (same gate as dispatch_task).
        if !step.files_to_touch.is_empty() {
            scope::claim_or_reject(&self.state_dir, &session, step.files_to_touch.clone())
                .with_context(|| format!("scope claim for {session}"))?;
        }
        let cwd = cwd.to_string_lossy();
        self.mgr
            .create_session_with_agent(&session, Some(&cwd), self.agent, Some(brief))
            .await
            .with_context(|| format!("spawning {session}"))?;
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
}
