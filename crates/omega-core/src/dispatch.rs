use crate::config::OmegaConfig;
use crate::done::DoneSignal;
use crate::oracle_lifecycle::{
    OraclePromptGenerator, OracleRegistryEntry, OracleRegistryStatus, OracleRegistry,
    OracleState, WorkerEntry, WorkerEntryStatus,
};
use crate::routing;
use crate::session::{SessionManager, SessionRole};
use anyhow::{bail, Result};
use chrono::Utc;
use std::path::Path;
use std::time::Duration;

/// Structured context for worker dispatch — ensures every worker gets
/// the information it needs to be fully autonomous (Third Law compliant).
///
/// Mirrors the VPS Fresh Context Template:
/// Mission, Purpose, Context, What's Done, Current Task, Done Criteria,
/// Verify Command, Key Decisions, Files in Scope, Relevant Memories.
#[derive(Debug, Clone, Default)]
pub struct WorkerContext {
    pub mission: String,
    pub purpose: Option<String>,
    pub project: Option<String>,
    pub working_dir: Option<String>,
    pub done_criteria: String,
    pub verify_command: Option<String>,
    pub files_owned: Vec<String>,
    pub context_notes: Vec<String>,
    pub what_done: Vec<String>,
    pub key_decisions: Vec<String>,
    pub git_branch: Option<String>,
    pub git_recent_commits: Vec<String>,
}

impl WorkerContext {
    pub fn format_prompt(&self, worker_name: &str) -> String {
        let mut prompt = String::with_capacity(2048);
        prompt.push_str("[DISPATCHED] You are an autonomous worker. Third Law: decide and proceed, never wait.\n\n");

        prompt.push_str(&format!("## Mission\n{}\n\n", self.mission));

        if let Some(ref purpose) = self.purpose {
            prompt.push_str(&format!("## Purpose\n{}\n\n", purpose));
        }

        if let Some(ref project) = self.project {
            let dir_str = self.working_dir.as_deref().unwrap_or(".");
            prompt.push_str(&format!("## Context\nProject: {} ({})\n", project, dir_str));
            if let Some(ref branch) = self.git_branch {
                prompt.push_str(&format!("Branch: {}\n", branch));
            }
            if !self.git_recent_commits.is_empty() {
                prompt.push_str("Recent commits:\n");
                for c in &self.git_recent_commits {
                    prompt.push_str(&format!("  {}\n", c));
                }
            }
            prompt.push('\n');
        }

        if !self.what_done.is_empty() {
            prompt.push_str("## What's Done\n");
            for item in &self.what_done {
                prompt.push_str(&format!("- {}\n", item));
            }
            prompt.push('\n');
        }

        if !self.context_notes.is_empty() {
            prompt.push_str("## Current Task\n");
            for note in &self.context_notes {
                prompt.push_str(&format!("- {}\n", note));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!("## Done Criteria\n{}\n\n", self.done_criteria));

        if let Some(ref verify) = self.verify_command {
            prompt.push_str(&format!("## Verify Command\n```bash\n{}\n```\n\n", verify));
        }

        if !self.files_owned.is_empty() {
            prompt.push_str(&format!(
                "## Files in Scope\n{}\nOnly modify files in your scope.\n\n",
                self.files_owned.join(", ")
            ));
        }

        if !self.key_decisions.is_empty() {
            prompt.push_str("## Key Decisions\n");
            for d in &self.key_decisions {
                prompt.push_str(&format!("- {}\n", d));
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

    /// Collect git context from a working directory.
    pub fn with_git_context(mut self, working_dir: &Path) -> Self {
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                self.git_branch =
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        if let Ok(output) = std::process::Command::new("git")
            .args(["log", "--oneline", "-5"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                self.git_recent_commits = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|l| l.to_string())
                    .collect();
            }
        }

        self
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
        let work_path = std::path::PathBuf::from(&work_dir);

        // Use oracle registry for naming + reuse of idle oracles
        let mut registry = OracleRegistry::load(&self.config.state_dir);
        let oracle_name = if let Some(idle) = registry.find_available(project) {
            idle.oracle_name.clone()
        } else {
            registry.next_oracle_name(project)
        };

        let decision = routing::classify_mission(mission);
        let ship = OraclePromptGenerator::should_ship(mission);
        let god_mode = OraclePromptGenerator::is_god_mode(mission);

        // Generate structured oracle prompt
        let mut prompt = OraclePromptGenerator::generate(
            project,
            &work_path,
            &oracle_name,
            mission,
            ship,
            god_mode,
        );

        // Append detected audit skills
        if !decision.audit_skills.is_empty() {
            prompt.push_str(&format!(
                "\n## Detected Audit Skills\n{}\nDispatch each as a separate worker with `/skillname` on line 1.\n",
                decision.audit_skills.iter()
                    .map(|a| format!("- /{} (triggered by '{}')", a.skill, a.trigger))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Append complexity hint
        prompt.push_str(&format!("\n## Complexity: {:?}\n", decision.complexity));

        self.session_mgr
            .create_agent_session(
                &oracle_name,
                &work_dir,
                &self.config.agent_command,
                Some(&prompt),
            )
            .await?;

        // Register in oracle registry
        registry.register(OracleRegistryEntry {
            oracle_name: oracle_name.clone(),
            project: project.to_string(),
            session_name: oracle_name.clone(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        });
        let _ = registry.save(&self.config.state_dir);

        tracing::info!(
            oracle = %oracle_name,
            project = %project,
            complexity = ?decision.complexity,
            audits = decision.audit_skills.len(),
            ship = %ship,
            god_mode = %god_mode,
            "Oracle dispatched"
        );
        Ok(oracle_name)
    }

    /// Dispatch a worker with structured context (Fresh Context Template).
    /// Automatically registers the worker in the parent oracle's state.
    pub async fn dispatch_worker_with_context(
        &self,
        oracle_name: &str,
        task_id: &str,
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

        // Register worker in oracle state if it exists
        if let Ok(Some(mut oracle_state)) =
            OracleState::read(&self.config.state_dir, oracle_name)
        {
            oracle_state.register_worker(WorkerEntry {
                session_name: worker_name.clone(),
                task_id: task_id.to_string(),
                task_name: task_name.to_string(),
                files_owned: ctx.files_owned.clone(),
                dispatched_at: Utc::now(),
                status: WorkerEntryStatus::Running,
            });
            let _ = oracle_state.write(&self.config.state_dir);
        }

        tracing::info!(worker = %worker_name, oracle = %oracle_name, task = %task_name, "Worker dispatched with structured context");
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

    /// Find or generate an oracle name. Checks registry first (reuse idle),
    /// falls back to live session scan (compat), then generates a new name.
    async fn find_available_oracle(&self, project: &str) -> Result<String> {
        // Check registry first
        let registry = OracleRegistry::load(&self.config.state_dir);
        if let Some(idle) = registry.find_available(project) {
            return Ok(idle.oracle_name.clone());
        }

        // Fallback: scan live sessions
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
