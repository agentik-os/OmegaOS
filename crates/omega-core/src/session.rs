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
        // contention the worst case is N callers each doing one RPC, the
        // first to re-lock winning the cache slot (or_insert_with), and
        // every caller returning its own equivalent pane for the same name.
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

    /// Resize a session's active pane to (cols, rows). The TUI calls this
    /// to match the rmux pane to the preview panel's real width — without
    /// it, Claude renders to its spawn-time 200-col width and the preview
    /// shows a clipped slice, making the agent UI look "phone-width".
    pub async fn resize_pane(&self, session_name: &str, cols: u16, rows: u16) -> Result<()> {
        // IMPORTANT (verified empirically): a pane is bounded by its WINDOW.
        // The SDK `pane.resize()` is CLAMPED to the current window size, so it
        // cannot grow the content past the window — calling it to widen a
        // session is a silent no-op. That was the real "Claude isn't
        // responsive to the panel width" bug: every resize call did nothing.
        // Growing the content requires resizing the WINDOW. The SDK exposes no
        // public window-resize, so we shell out to the proven
        // `rmux resize-window`, which sets the absolute size and makes the
        // inner app redraw via SIGWINCH (confirmed live: 200→120→150→100).
        //
        // Cheap in practice: callers guard this behind (session, cols, rows)
        // change-detection, so it fires only when the geometry actually
        // changes, not every frame.
        let out = tokio::process::Command::new("rmux")
            .args([
                "resize-window",
                "-t",
                session_name,
                "-x",
                &cols.to_string(),
                "-y",
                &rows.to_string(),
            ])
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!(
                "rmux resize-window failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
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

    /// Capture pane text AND the REAL cursor position from the pane
    /// snapshot. The TUI uses (row, col) to paint the caret exactly where
    /// the agent's input cursor is — not a guessed "last non-empty line".
    /// Returns (visible_text, cursor_row, cursor_col, cursor_visible).
    pub async fn capture_pane_with_cursor(
        &self,
        session_name: &str,
    ) -> Result<(String, u16, u16, bool)> {
        let pane = self.pane_for(session_name).await?;
        let snapshot = match pane.snapshot().await {
            Ok(s) => s,
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.snapshot().await?
            }
            Err(e) => return Err(e.into()),
        };
        let c = snapshot.cursor;
        Ok((snapshot.visible_text(), c.row, c.col, c.visible))
    }

    /// Capture the pane WITH per-cell styling (fg/bg/bold/reverse), so the
    /// TUI preview can render Claude's colored UI — most importantly the
    /// `/` command-menu selection highlight, which `visible_text()` drops
    /// entirely (the user "can't see what they're selecting"). Also makes
    /// diffs, syntax highlighting, and status colors visible.
    ///
    /// Capture the visible pane as styled rows, GATED on the pane's revision.
    ///
    /// `since_revision` = the revision the caller last rendered. The pane's
    /// `revision` (in every snapshot) bumps on any observable change — output,
    /// resize, clear, cursor move. If it hasn't moved, we return
    /// [`StyledCapture::Unchanged`] WITHOUT running `styled_rows_from_snapshot`
    /// over ~10k cells or flattening to text — the dominant per-frame cost.
    /// During the (frequent) "agent is thinking" pauses the preview redraws for
    /// free off the cached rows. Pass `0` to force a fresh capture (e.g. on a
    /// session switch). Revision `0` (stale/empty snapshot) never gates.
    pub async fn capture_pane_styled(
        &self,
        session_name: &str,
        since_revision: u64,
    ) -> Result<StyledCapture> {
        let pane = self.pane_for(session_name).await?;
        let snapshot = match pane.snapshot().await {
            Ok(s) => s,
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                pane.snapshot().await?
            }
            Err(e) => return Err(e.into()),
        };
        if snapshot.revision != 0 && snapshot.revision == since_revision {
            return Ok(StyledCapture::Unchanged);
        }
        let c = snapshot.cursor;
        let rows = styled_rows_from_snapshot(&snapshot);
        Ok(StyledCapture::Changed {
            rows,
            cursor_row: c.row,
            cursor_col: c.col,
            cursor_visible: c.visible,
            revision: snapshot.revision,
        })
    }

    /// Capture the pane INCLUDING scrollback history (last `history_lines`
    /// lines, counted up from the live tail). `snapshot().visible_text()`
    /// only renders the current visible screen — so the TUI preview can't
    /// scroll up into history with it. This shells out to the rmux CLI which
    /// supports `-S -<N>` (start N lines into the scrollback buffer).
    ///
    /// SLOW PATH: only used while the user is actively browsing history
    /// (`preview_follow_tail == false`); the live tail keeps the fast
    /// `capture_pane` snapshot path.
    pub async fn capture_pane_history(
        &self,
        session_name: &str,
        history_lines: u32,
    ) -> Result<String> {
        let out = tokio::process::Command::new("rmux")
            .args([
                "capture-pane",
                "-p",
                "-t",
                session_name,
                "-S",
                &format!("-{}", history_lines),
            ])
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!(
                "rmux capture-pane -S failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Forward a paste as ONE bracketed-paste block (`\e[200~ … \e[201~`)
    /// with NO trailing Enter. The target app (Claude Code / a REPL) buffers
    /// the whole block as a single paste instead of treating each embedded
    /// `\n` as a submit — so a 30-line paste lands as 30 lines of one turn,
    /// not 30 separate commands. The user presses Enter themselves.
    ///
    /// Distinct from `send_paste_then_submit` (which auto-submits) — used for
    /// forwarding a *user* paste into the interactive preview.
    pub async fn send_paste_raw(&self, session_name: &str, text: &str) -> Result<()> {
        // Whole bracketed-paste block (markers + chunked body) as one unit, so
        // the stale-pane retry replays the ENTIRE paste atomically — never a
        // half-sent block. Mirrors the single-retry strategy of send_text_raw.
        async fn paste_block(pane: &Pane, text: &str) -> std::result::Result<(), rmux_sdk::RmuxError> {
            pane.send_text("\u{1b}[200~").await?;
            // Chunk the body so a very large paste isn't sent as one oversized
            // PTY write. Markers are sent once; only the body is split, so the
            // block stays atomic from the target app's perspective.
            const CHUNK: usize = 4096;
            if text.len() <= CHUNK {
                pane.send_text(text).await?;
            } else {
                let mut start = 0;
                let bytes = text.as_bytes();
                while start < bytes.len() {
                    // Advance to a char boundary at/under the chunk limit.
                    let mut end = (start + CHUNK).min(bytes.len());
                    while end < bytes.len() && !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    pane.send_text(&text[start..end]).await?;
                    start = end;
                }
            }
            pane.send_text("\u{1b}[201~").await?;
            Ok(())
        }

        let pane = self.pane_for(session_name).await?;
        match paste_block(&pane, text).await {
            Ok(()) => Ok(()),
            Err(e) if is_pane_stale(&e) => {
                self.invalidate_pane(session_name).await;
                let pane = self.pane_for(session_name).await?;
                paste_block(&pane, text).await?;
                Ok(())
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

/// A preview color, PRESERVING its original depth. Critical: converting an
/// ANSI 16-color (what Claude/most CLIs emit) into 24-bit RGB makes the TUI
/// emit truecolor `38;2;…` escapes — which a terminal reached over mosh / a
/// `TERM=xterm` session renders as DEFAULT (grey). Keeping the original index
/// lets the renderer emit the matching 16-color (`9x`) / 256 (`38;5`) escape,
/// which renders everywhere the chrome already does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewColor {
    /// ANSI/xterm palette index (0–15 = the 16 base colors, 16–255 = 256-cube).
    Indexed(u8),
    /// True 24-bit color (only when the source cell was genuinely RGB).
    Rgb(u8, u8, u8),
}

/// One styled run of text within a preview row. `None` = terminal default.
#[derive(Debug, Clone)]
pub struct PreviewSpan {
    pub text: String,
    pub fg: Option<PreviewColor>,
    pub bg: Option<PreviewColor>,
    pub bold: bool,
}

/// A styled preview row = a sequence of spans.
pub type PreviewLine = Vec<PreviewSpan>;

/// Result of a revision-gated styled capture. `Unchanged` means the pane's
/// revision matched what the caller already rendered — no restyle was done and
/// the caller should keep its cached preview.
pub enum StyledCapture {
    Unchanged,
    Changed {
        rows: Vec<PreviewLine>,
        cursor_row: u16,
        cursor_col: u16,
        cursor_visible: bool,
        revision: u64,
    },
}

/// Convert a pane snapshot into styled rows, merging adjacent same-style
/// cells into spans. Honors REVERSE by swapping fg/bg (that's how the
/// `/` selector + many TUI highlights are drawn).
fn styled_rows_from_snapshot(snapshot: &rmux_sdk::PaneSnapshot) -> Vec<PreviewLine> {
    let cols = snapshot.cols;
    let rows = snapshot.rows;
    let mut out: Vec<PreviewLine> = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut line: PreviewLine = Vec::new();
        let mut cur_text = String::new();
        let mut cur_fg: Option<PreviewColor> = None;
        let mut cur_bg: Option<PreviewColor> = None;
        let mut cur_bold = false;
        let mut started = false;
        for col in 0..cols {
            let Some(cell) = snapshot.cell(r, col) else { continue };
            if cell.glyph.is_padding() { continue; }
            let ch = cell.glyph.text.clone();
            let attr_bits = cell.attributes.bits;
            let reverse = attr_bits & rmux_sdk::PaneAttributes::REVERSE.bits != 0;
            let bold = attr_bits & rmux_sdk::PaneAttributes::BOLD.bits != 0;
            let mut fg = pane_color_to_preview(&cell.foreground);
            let mut bg = pane_color_to_preview(&cell.background);
            if reverse {
                std::mem::swap(&mut fg, &mut bg);
                // ensure a visible swap even when one side was default
                if fg.is_none() { fg = Some(PreviewColor::Indexed(0)); }
                if bg.is_none() { bg = Some(PreviewColor::Indexed(7)); }
            }
            let glyph = if ch.is_empty() { " ".to_string() } else { ch };
            if started && fg == cur_fg && bg == cur_bg && bold == cur_bold {
                cur_text.push_str(&glyph);
            } else {
                if started {
                    line.push(PreviewSpan { text: std::mem::take(&mut cur_text), fg: cur_fg, bg: cur_bg, bold: cur_bold });
                }
                cur_fg = fg;
                cur_bg = bg;
                cur_bold = bold;
                cur_text = glyph;
                started = true;
            }
        }
        if started && !cur_text.is_empty() {
            line.push(PreviewSpan { text: cur_text, fg: cur_fg, bg: cur_bg, bold: cur_bold });
        }
        // Trim trailing all-blank spans to keep lines tight.
        while line.last().map_or(false, |s| s.text.trim().is_empty() && s.bg.is_none()) {
            line.pop();
        }
        out.push(line);
    }
    // Drop trailing blank rows.
    while out.last().map_or(false, |l| l.is_empty()) {
        out.pop();
    }
    out
}

/// Resolve a rmux PaneColor into a PreviewColor, PRESERVING its depth so the
/// renderer can emit a 16-color / 256 / truecolor escape that matches what the
/// source actually used. `None` = terminal default. Upconverting ANSI→RGB here
/// was the bug that rendered Claude's 16-color output as grey truecolor.
fn pane_color_to_preview(c: &rmux_sdk::PaneColor) -> Option<PreviewColor> {
    use rmux_sdk::PaneColor;
    match c {
        PaneColor::Default | PaneColor::None | PaneColor::Terminal => None,
        PaneColor::Rgb { red, green, blue } => Some(PreviewColor::Rgb(*red, *green, *blue)),
        PaneColor::Ansi { index } => Some(PreviewColor::Indexed(index & 0x0f)),
        PaneColor::BrightAnsi { index } => Some(PreviewColor::Indexed((index & 0x07) + 8)),
        PaneColor::Indexed { index } => Some(PreviewColor::Indexed(*index)),
        _ => None,
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
