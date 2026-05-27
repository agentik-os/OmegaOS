use crate::config::OmegaConfig;
use crate::done::DoneSignal;
use crate::routing;
use crate::session::{SessionManager, SessionRole};
use anyhow::{bail, Result};
use std::time::Duration;

/// Structured context for worker dispatch — ensures every worker gets
/// the information it needs to be fully autonomous (Third Law compliant).
#[derive(Debug, Clone, Default)]
pub struct WorkerContext {
    pub mission: String,
    pub purpose: Option<String>,
    pub done_criteria: String,
    pub verify_command: Option<String>,
    pub files_owned: Vec<String>,
    pub context_notes: Vec<String>,
}

impl WorkerContext {
    pub fn format_prompt(&self, worker_name: &str) -> String {
        let mut prompt = String::new();
        prompt.push_str("[DISPATCHED] You are an autonomous worker. Third Law: decide and proceed, never wait.\n\n");

        prompt.push_str(&format!("## Mission\n{}\n\n", self.mission));

        if let Some(ref purpose) = self.purpose {
            prompt.push_str(&format!("## Purpose\n{}\n\n", purpose));
        }

        prompt.push_str(&format!("## Done Criteria\n{}\n\n", self.done_criteria));

        if let Some(ref verify) = self.verify_command {
            prompt.push_str(&format!("## Verify Command\n```bash\n{}\n```\n\n", verify));
        }

        if !self.files_owned.is_empty() {
            prompt.push_str(&format!(
                "## Files Owned\n{}\nOnly modify files in your scope.\n\n",
                self.files_owned.join(", ")
            ));
        }

        if !self.context_notes.is_empty() {
            prompt.push_str("## Context\n");
            for note in &self.context_notes {
                prompt.push_str(&format!("- {}\n", note));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!(
            "## Completion\nWhen done: `omega done {} done_clean \"<summary>\"`\n\
             If blocked: `omega done {} blocked \"<what's blocking>\"`\n\
             If failed: `omega done {} failed \"<what went wrong>\"`\n",
            worker_name, worker_name, worker_name
        ));

        prompt
    }
}

pub struct Dispatcher {
    session_mgr: SessionManager,
    config: OmegaConfig,
}

impl Dispatcher {
    pub fn new(session_mgr: SessionManager, config: OmegaConfig) -> Self {
        Self {
            session_mgr,
            config,
        }
    }

    pub async fn dispatch_oracle(
        &self,
        project: &str,
        mission: &str,
    ) -> Result<String> {
        let work_dir = match self.config.find_project(project) {
            Some(pc) => pc.path.to_string_lossy().to_string(),
            None => std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
        };

        let oracle_name = self.find_available_oracle(project).await?;

        let decision = routing::classify_mission(mission);
        let audit_note = if !decision.audit_skills.is_empty() {
            format!(
                "\n## Detected Audit Skills\n{}\nDispatch each as a separate worker with `/skillname` on line 1.\n",
                decision.audit_skills.iter()
                    .map(|a| format!("- /{} (triggered by '{}')", a.skill, a.trigger))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        let prompt = format!(
            "## Mission: {}\n## Project: {} ({})\n## Role: ORACLE\n## Complexity: {:?}\n\
             \nYou are the Oracle for this project. Analyze the mission, decompose into tasks, \
             and dispatch workers via `omega spawn-worker <task>`. Monitor progress and verify \
             quality before reporting done.\n\
             \n## Three Laws\n\
             1. Code lies. Only runtime tells the truth.\n\
             2. Be a researcher, not a sycophant.\n\
             3. Decide and proceed, never wait.\n\
             \n## Quality Gate\n\
             Before reporting done: all workers complete, build passes, no runtime errors, \
             `omega gate {}` criteria satisfied.\n\
             {}\n\
             When all verified: `omega done {} done_clean \"<summary>\"`",
            mission, project, work_dir, decision.complexity,
            oracle_name, audit_note, oracle_name,
        );

        self.session_mgr
            .create_agent_session(
                &oracle_name,
                &work_dir,
                &self.config.agent_command,
                Some(&prompt),
            )
            .await?;

        tracing::info!(
            oracle = %oracle_name,
            project = %project,
            complexity = ?decision.complexity,
            audits = decision.audit_skills.len(),
            "Oracle dispatched"
        );
        Ok(oracle_name)
    }

    /// Dispatch a worker with structured context (Fresh Context Template).
    pub async fn dispatch_worker_with_context(
        &self,
        oracle_name: &str,
        task_name: &str,
        ctx: &WorkerContext,
        working_dir: &str,
    ) -> Result<String> {
        let worker_name = format!("{}-worker-{}", oracle_name.replace("oracle-", ""), task_name);
        let prompt = ctx.format_prompt(&worker_name);

        if !ctx.files_owned.is_empty() {
            crate::scope::claim_or_reject(
                &self.config.state_dir,
                &worker_name,
                ctx.files_owned.clone(),
            )?;
        }

        self.session_mgr
            .create_agent_session(
                &worker_name,
                working_dir,
                &self.config.agent_command,
                Some(&prompt),
            )
            .await?;

        tracing::info!(worker = %worker_name, oracle = %oracle_name, "Worker dispatched with structured context");
        Ok(worker_name)
    }

    pub async fn dispatch_worker(
        &self,
        oracle_name: &str,
        task_name: &str,
        prompt: &str,
        working_dir: &str,
    ) -> Result<String> {
        let worker_name = format!("{}-worker-{}", oracle_name.replace("oracle-", ""), task_name);

        let worker_prompt = format!(
            "[DISPATCHED] You are an autonomous worker. Third Law: decide and proceed, never wait.\n\n\
             {}\n\n\
             ## Completion\n\
             When done: `omega done {} done_clean \"<summary>\"`\n\
             If blocked: `omega done {} blocked \"<what's blocking>\"`\n\
             If failed: `omega done {} failed \"<what went wrong>\"`",
            prompt, worker_name, worker_name, worker_name
        );

        self.session_mgr
            .create_agent_session(
                &worker_name,
                working_dir,
                &self.config.agent_command,
                Some(&worker_prompt),
            )
            .await?;

        tracing::info!(worker = %worker_name, oracle = %oracle_name, "Worker dispatched");
        Ok(worker_name)
    }

    async fn find_available_oracle(&self, project: &str) -> Result<String> {
        let sessions = self.session_mgr.list_sessions().await?;
        let existing_oracles: Vec<_> = sessions
            .iter()
            .filter(|s| {
                s.role == SessionRole::Oracle
                    && s.project.as_deref() == Some(project)
            })
            .collect();

        if existing_oracles.is_empty() {
            return Ok(format!("oracle-{}", project));
        }

        let max_index = existing_oracles
            .iter()
            .filter_map(|s| s.oracle_index)
            .max()
            .unwrap_or(1);

        Ok(format!("oracle-{}-{}", project, max_index + 1))
    }

    pub async fn wait_for_done(
        &self,
        session_name: &str,
        timeout: Duration,
    ) -> Result<DoneSignal> {
        let done_path = self
            .config
            .state_dir
            .join(format!("worker-{}.done.json", session_name));

        let start = std::time::Instant::now();
        loop {
            if done_path.exists() {
                let content = std::fs::read_to_string(&done_path)?;
                return Ok(serde_json::from_str(&content)?);
            }
            if start.elapsed() > timeout {
                bail!("Timeout waiting for done signal from {}", session_name);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    pub fn session_manager(&self) -> &SessionManager {
        &self.session_mgr
    }
}
