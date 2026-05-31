//! Executor — the autonomous Driver. Selects work ONLY via
//! PlanTracker::ready_steps (the LLM is never asked "what next?"), spawns one
//! worker per step through a WorkerRuntime, and gates every completion through
//! the Guardian before marking a step Done.

use crate::done::DoneSignal;
use crate::planner::PlanStep;
use anyhow::Result;
use std::path::Path;
use std::time::Duration;

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
}
