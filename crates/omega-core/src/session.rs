use crate::agents::Agent;
use anyhow::{Context, Result};
use rmux_sdk::{
    EnsureSession, EnsureSessionPolicy, Pane, ProcessSpec, Rmux, Session, SessionName,
    SplitDirection, TerminalSizeSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
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

#[derive(Clone)]
pub struct SessionManager {
    rmux: Arc<Rmux>,
    /// Process-wide Pane handle cache keyed by session name.
    ///
    /// Each keystroke in the TUI chat-focused right panel ultimately calls
    /// `send_text` / `send_text_raw` / `send_key`. Without this cache, every
    /// call resolves the active pane by issuing a fresh `rmux.session(name)`
    /// RPC to the daemon (5–15ms over the local Unix socket). At 60 FPS
    /// typing that's ~120ms of stacked blocking per second — visible as
    /// "typing feels laggy in Hermux".
    ///
    /// The cache stores cloned `Pane` handles (Pane is `#[derive(Clone)]` —
    /// just an Arc<endpoint> + Arc<transport>) so hot-path lookups become a
    /// single mutex acquisition + HashMap get, with zero daemon RPCs. The
    /// cache is invalidated on `kill_session` and on send errors.
    pane_cache: Arc<tokio::sync::Mutex<HashMap<String, Pane>>>,
}

// Process-wide singleton — reused across every Action handler in the TUI
// and Telegram bridge. The original SessionManager::connect() was opening
// a fresh rmux daemon socket every call (~30-50ms latency per call), which
// stacked up to >100ms perceived latency per keystroke in interactive
// passthrough. The cached path serves the same Arc<Rmux> to every caller.
static CACHED_MANAGER: tokio::sync::OnceCell<SessionManager> = tokio::sync::OnceCell::const_new();

impl SessionManager {
    pub async fn connect() -> Result<Self> {
        let rmux = Rmux::builder()
            .default_timeout(Duration::from_secs(10))
            .connect_or_start()
            .await
            .context("Failed to connect to rmux daemon")?;
        Ok(Self {
            rmux: Arc::new(rmux),
            pane_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Process-wide cached SessionManager. First call connects, every
    /// subsequent call hands back a clone (Arc<Rmux> share). Use this
    /// in hot paths (per-keystroke forwarding, capture refresh).
    pub async fn connect_cached() -> Result<Self> {
        CACHED_MANAGER
            .get_or_try_init(|| async { Self::connect().await })
            .await
            .cloned()
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

    /// Spawn an agent session with full LaunchOptions — for Claude this
    /// enables /goal injection, --effort, --max-turns, --max-budget-usd.
    /// Other providers ignore the Claude-only fields.
    pub async fn create_agent_session_with_opts(
        &self,
        name: &str,
        working_dir: &str,
        agent: Agent,
        prompt: Option<&str>,
        opts: crate::agents::LaunchOptions,
    ) -> Result<Session> {
        let cmd = agent.launch_command_with(prompt, opts);
        self.create_session(name, Some(working_dir), Some(&cmd))
            .await
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
        // The pane handle we held is now dangling — drop it so any future
        // send_text/send_key for the same name re-resolves cleanly.
        self.invalidate_pane(name).await;
        Ok(())
    }

    /// Cached lookup of the active pane for `session_name`.
    ///
    /// On a cache hit this is a single mutex acquisition (microseconds).
    /// On a miss it issues one `rmux.session(name)` RPC, stores the result,
    /// and returns the new Pane. The Pane handle itself is cheap to clone
    /// (Arc-wrapped endpoint), so we hand back a clone and keep one in the
    /// cache for the next caller.
    ///
    /// Use this in any hot path where the same session is touched repeatedly
    /// (TUI keystroke forwarding, preview capture). For one-shot operations
    /// `get_active_pane` is still fine but offers no win.
    pub async fn pane_for(&self, session_name: &str) -> Result<Pane> {
        {
            let cache = self.pane_cache.lock().await;
            if let Some(pane) = cache.get(session_name) {
                return Ok(pane.clone());
            }
        }
        // Miss — resolve and cache. We deliberately drop the lock across
        // the get_session await to avoid serialising callers; under
        // contention the worst case is N callers each doing one RPC and
        // the last writer winning, all equivalent panes for the same name.
        let session = self.get_session(session_name).await?;
        let pane = session.pane(0, 0);
        let mut cache = self.pane_cache.lock().await;
        cache
            .entry(session_name.to_string())
            .or_insert_with(|| pane.clone());
        Ok(pane)
    }

    /// Drop the cached Pane for a session — call after kill/recreate or
    /// after a send error that suggests the daemon-side pane is gone.
    pub async fn invalidate_pane(&self, session_name: &str) {
        self.pane_cache.lock().await.remove(session_name);
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
        // Old name is no longer addressable — drop its cached pane.
        self.invalidate_pane(old_name).await;
        Ok(())
    }

    pub async fn get_active_pane(&self, name: &str) -> Result<Pane> {
        let session = self.get_session(name).await?;
        Ok(session.pane(0, 0))
    }

    pub async fn send_text(&self, session_name: &str, text: &str) -> Result<()> {
        // Two hot RPCs (send_text + send_key Enter). Use the cached pane and
        // a single retry on stale-cache errors so a kill+recreate of the
        // same name self-heals on the next send.
        let pane = self.pane_for(session_name).await?;
        match pane.send_text(text).await {
            Ok(()) => {}
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_text(text).await?;
                pane.send_key("Enter").await?;
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        }
        pane.send_key("Enter").await?;
        Ok(())
    }

    /// Raw text send — no auto-Enter. Used by the TUI interactive preview
    /// to forward single chars without injecting a newline the user did not type.
    ///
    /// HOT PATH: invoked per keystroke when the right panel is chat-focused.
    /// Uses the cached pane lookup so the only daemon RPC is the actual
    /// send_text; on a stale-pane error we invalidate + retry once.
    pub async fn send_text_raw(&self, session_name: &str, text: &str) -> Result<()> {
        let pane = self.pane_for(session_name).await?;
        match pane.send_text(text).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_text(text).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Send a named key event (e.g. "Enter", "BackSpace", "Up", "Escape").
    /// Mirrors the rmux key naming.
    ///
    /// HOT PATH: invoked per arrow/enter/space in chat-focused mode. Same
    /// cached-pane + single-retry strategy as `send_text_raw`.
    pub async fn send_key(&self, session_name: &str, key: &str) -> Result<()> {
        let pane = self.pane_for(session_name).await?;
        match pane.send_key(key).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.send_key(key).await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Multi-line aware send: wraps `text` in bracketed-paste escape
    /// sequences (`\e[200~ ... \e[201~`) so that an interactive TUI
    /// (e.g. Claude Code) buffers the whole block as one paste rather
    /// than submitting on every embedded newline. After the closing
    /// marker, sends Enter to submit the buffered input.
    ///
    /// Use this whenever the prompt may contain `\n` AND you want a
    /// single coherent user turn (e.g., reply-context + new message).
    pub async fn send_paste_then_submit(
        &self,
        session_name: &str,
        text: &str,
    ) -> Result<()> {
        let pane = self.pane_for(session_name).await?;
        // Bracketed paste opener
        pane.send_text("\u{1b}[200~").await?;
        pane.send_text(text).await?;
        // Closer
        pane.send_text("\u{1b}[201~").await?;
        // Submit
        pane.send_key("Enter").await?;
        Ok(())
    }

    /// HOT PATH (~12 FPS during chat focus): the right-panel preview tick
    /// uses this on every refresh. Cached pane → one daemon RPC per tick
    /// (the snapshot itself), not three.
    pub async fn capture_pane(&self, session_name: &str) -> Result<String> {
        let pane = self.pane_for(session_name).await?;
        match pane.snapshot().await {
            Ok(snapshot) => Ok(snapshot.visible_text()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                let snapshot = pane.snapshot().await?;
                Ok(snapshot.visible_text())
            }
            Err(e) => Err(e.into()),
        }
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

/// Cached `Pane` handles can outlive the underlying daemon-side pane when a
/// session is killed-and-recreated under the same name (rename-then-recreate,
/// crash-restart, etc.). The hot-path send/capture methods rely on this
/// predicate to recognise such errors, drop the stale handle, and retry once.
fn is_pane_stale(err: &rmux_sdk::RmuxError) -> bool {
    matches!(
        err,
        rmux_sdk::RmuxError::PaneNotFound { .. }
            | rmux_sdk::RmuxError::OwnedSessionLeaseLost { .. }
    )
}
