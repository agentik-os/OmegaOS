use crate::agents::Agent;
use anyhow::{Context, Result};
use rmux_sdk::{
    EnsureSession, EnsureSessionPolicy, Pane, ProcessSpec, Rmux, Session, SessionName,
    SplitDirection, TerminalSizeSpec,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRole {
    Oracle,
    Worker,
    Home,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaSession {
    pub name: String,
    pub role: SessionRole,
    pub project: Option<String>,
    pub oracle_index: Option<u32>,
    pub working_dir: Option<PathBuf>,
}

impl OmegaSession {
    pub fn classify(name: &str) -> Self {
        let (role, project, oracle_index) = Self::parse_session_name(name);
        Self {
            name: name.to_string(),
            role,
            project,
            oracle_index,
            working_dir: None,
        }
    }

    fn parse_session_name(name: &str) -> (SessionRole, Option<String>, Option<u32>) {
        // Oracle pattern first — most specific
        if let Some(rest) = name.strip_prefix("oracle-") {
            let (project, idx) = Self::extract_project_and_index(rest);
            return (SessionRole::Oracle, Some(project), idx);
        }

        // Worker pattern: <Project>-(worker|fix|dev|dispatch|...)-
        // Check this BEFORE the system-prefix check so that e.g. AISB-worker-X
        // is correctly identified as a Worker under project AISB.
        let worker_suffixes = [
            "-worker-", "-fix-", "-dev-", "-dispatch-", "-work-", "-linear", "-task-", "-audit-",
            "-challenger-", "-report-", "-verify-", "-build-", "-deploy-", "-team-",
        ];
        for suffix in &worker_suffixes {
            if let Some(pos) = name.find(suffix) {
                let project = name[..pos].to_string();
                return (SessionRole::Worker, Some(project), None);
            }
        }

        // Team session: Team-<Project>
        if let Some(rest) = name.strip_prefix("Team-") {
            return (SessionRole::Worker, Some(rest.to_string()), None);
        }

        // Home sessions
        if name.starts_with("Home") || name.starts_with("c-") {
            return (SessionRole::Home, None, None);
        }

        // System daemons (only true daemons, not project-prefixed sessions)
        let system_exact = ["AISB-monitor", "AISB-daemon", "AISB-master"];
        for sys in &system_exact {
            if name == *sys {
                return (SessionRole::System, None, None);
            }
        }
        if name.starts_with("earthbit-") {
            return (SessionRole::System, None, None);
        }

        (SessionRole::Home, None, None)
    }

    fn extract_project_and_index(rest: &str) -> (String, Option<u32>) {
        if let Some(last_dash) = rest.rfind('-') {
            if let Ok(idx) = rest[last_dash + 1..].parse::<u32>() {
                return (rest[..last_dash].to_string(), Some(idx));
            }
        }
        (rest.to_string(), None)
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }
}

pub struct SessionManager {
    rmux: Rmux,
}

impl SessionManager {
    pub async fn connect() -> Result<Self> {
        let rmux = Rmux::builder()
            .default_timeout(Duration::from_secs(10))
            .connect_or_start()
            .await
            .context("Failed to connect to rmux daemon")?;
        Ok(Self { rmux })
    }

    pub async fn create_session(
        &self,
        name: &str,
        working_dir: Option<&str>,
        command: Option<&str>,
    ) -> Result<Session> {
        let session_name = SessionName::new(name)?;
        let mut builder = EnsureSession::named(session_name)
            .policy(EnsureSessionPolicy::CreateOrReuse)
            .detached(true)
            .size(TerminalSizeSpec::new(200, 50));

        if let Some(cmd) = command {
            builder = builder.process(ProcessSpec::shell(cmd));
        }

        if let Some(dir) = working_dir {
            builder = builder.working_directory(dir);
        }

        let session = self
            .rmux
            .ensure_session(builder)
            .await
            .context("Failed to create session")?;
        Ok(session)
    }

    pub async fn create_agent_session(
        &self,
        name: &str,
        working_dir: &str,
        agent_command: &str,
        prompt: Option<&str>,
    ) -> Result<Session> {
        // Resolve the agent type from its name (defaults to Claude for backwards-compat)
        let agent = Agent::from_name(agent_command).unwrap_or(Agent::Claude);
        let cmd = agent.launch_command(prompt);
        self.create_session(name, Some(working_dir), Some(&cmd))
            .await
    }

    pub async fn create_session_with_agent(
        &self,
        name: &str,
        working_dir: Option<&str>,
        agent: Agent,
        prompt: Option<&str>,
    ) -> Result<Session> {
        let cmd = agent.launch_command(prompt);
        self.create_session(name, working_dir, Some(&cmd)).await
    }

    pub async fn list_sessions(&self) -> Result<Vec<OmegaSession>> {
        let session_names = self.rmux.list_sessions().await?;
        let mut sessions: Vec<OmegaSession> = session_names
            .iter()
            .map(|name| OmegaSession::classify(name.as_ref()))
            .collect();

        sessions.sort_by(|a, b| {
            let sa = section_order(&a.role);
            let sb = section_order(&b.role);
            sa.cmp(&sb)
                .then_with(|| a.project.cmp(&b.project))
                .then_with(|| a.oracle_index.cmp(&b.oracle_index))
                .then_with(|| role_order(&a.role).cmp(&role_order(&b.role)))
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(sessions)
    }

    pub async fn get_session(&self, name: &str) -> Result<Session> {
        let session_name = SessionName::new(name)?;
        self.rmux
            .session(session_name)
            .await
            .context("Session not found")
    }

    pub async fn kill_session(&self, name: &str) -> Result<()> {
        let session = self.get_session(name).await?;
        session.kill().await?;
        Ok(())
    }

    /// Rename a session via the rmux CLI (the SDK doesn't expose rename yet).
    /// Equivalent to: rmux rename-session -t <old> <new>
    pub async fn rename_session(&self, old_name: &str, new_name: &str) -> Result<()> {
        let _ = SessionName::new(new_name)
            .context("invalid new session name")?;
        let status = tokio::process::Command::new("rmux")
            .args(["rename-session", "-t", old_name, new_name])
            .status()
            .await
            .context("spawning rmux rename-session")?;
        if !status.success() {
            anyhow::bail!("rmux rename-session failed (exit {:?})", status.code());
        }
        Ok(())
    }

    pub async fn get_active_pane(&self, name: &str) -> Result<Pane> {
        let session = self.get_session(name).await?;
        Ok(session.pane(0, 0))
    }

    pub async fn send_text(&self, session_name: &str, text: &str) -> Result<()> {
        let pane = self.get_active_pane(session_name).await?;
        pane.send_text(text).await?;
        pane.send_key("Enter").await?;
        Ok(())
    }

    /// Raw text send — no auto-Enter. Used by the TUI interactive preview
    /// to forward single chars without injecting a newline the user did not type.
    pub async fn send_text_raw(&self, session_name: &str, text: &str) -> Result<()> {
        let pane = self.get_active_pane(session_name).await?;
        pane.send_text(text).await?;
        Ok(())
    }

    /// Send a named key event (e.g. "Enter", "BackSpace", "Up", "Escape").
    /// Mirrors the rmux key naming.
    pub async fn send_key(&self, session_name: &str, key: &str) -> Result<()> {
        let pane = self.get_active_pane(session_name).await?;
        pane.send_key(key).await?;
        Ok(())
    }

    pub async fn capture_pane(&self, session_name: &str) -> Result<String> {
        let pane = self.get_active_pane(session_name).await?;
        let snapshot = pane.snapshot().await?;
        Ok(snapshot.visible_text())
    }

    pub async fn wait_for_text(
        &self,
        session_name: &str,
        text: &str,
        timeout: Duration,
    ) -> Result<()> {
        let pane = self.get_active_pane(session_name).await?;
        pane.expect_visible_text()
            .to_contain(text)
            .timeout(timeout)
            .await?;
        Ok(())
    }

    pub async fn split_pane(
        &self,
        session_name: &str,
        command: Option<&str>,
    ) -> Result<Pane> {
        let pane = self.get_active_pane(session_name).await?;

        if let Some(cmd) = command {
            let new_pane = pane.split_with(SplitDirection::Right).shell(cmd).await?;
            Ok(new_pane)
        } else {
            let new_pane = pane.split(SplitDirection::Right).await?;
            Ok(new_pane)
        }
    }

    pub fn rmux(&self) -> &Rmux {
        &self.rmux
    }
}

fn section_order(role: &SessionRole) -> u8 {
    match role {
        SessionRole::Home => 0,
        SessionRole::Oracle | SessionRole::Worker => 1,
        SessionRole::System => 2,
    }
}

fn role_order(role: &SessionRole) -> u8 {
    match role {
        SessionRole::Oracle => 0,
        SessionRole::Worker => 1,
        SessionRole::Home => 2,
        SessionRole::System => 3,
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
