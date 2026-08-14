//! OAuth flow for OmegaOS — re-implements the AISB Python `_request_reauth` /
//! `_handle_code` pair in Rust against the rmux SDK.
//!
//! The flow:
//!   1. `request_reauth` spawns a dedicated rmux session `aisb-reauth` running
//!      `claude --dangerously-skip-permissions`, sends `/login`, and captures
//!      the OAuth URL from the pane.
//!   2. The bridge sends the URL to the user via Telegram.
//!   3. The user clicks → authorizes → pastes the code back into Telegram.
//!   4. `handle_code` pastes the code into the waiting rmux session, watches
//!      `~/.claude/.credentials.json` until its mtime changes (or 20s timeout),
//!      then kills the reauth session.
//!
//! State is tracked in-memory AND persisted as private OmegaOS authority under
//! `$OMEGA_DIR/state/` so a bridge restart doesn't lose the pending flag.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::session::{sanitize_session_name, SessionManager};

pub const REAUTH_SESSION: &str = "aisb-reauth";
pub const COOLDOWN_SEC: u64 = 30;
pub const PENDING_TTL_SEC: u64 = 300;

/// Markers in pane output that indicate an auth failure / rate limit.
pub const AUTH_FAILURE_MARKERS: &[&str] = &[
    "401",
    "Unauthorized",
    "rate_limit_error",
    "Rate limit reached",
    "Please run /login",
    "Invalid bearer token",
    "Token expired",
    "authentication failed",
];

/// Authorization-URL prefixes Claude can present on its `/login` screen.
///
/// The binary ships MORE THAN ONE: 2.1.232 carries both the historical
/// `claude.com/cai/…` form and a `platform.claude.com/…` variant, and which one
/// a given login paints is not ours to predict. Recognising only the first made
/// `extract_auth_url` return "" for the second, so the poll loop in
/// `start_reauth` ran out its 15s and the operator got "Link not generated"
/// instead of a link. Every prefix test in this module reads THIS list, so a
/// third form is one line here, never three edits scattered across the parser.
pub const AUTHORIZE_URL_PREFIXES: &[&str] = &[
    "https://claude.com/cai/oauth/authorize",
    "https://platform.claude.com/oauth/authorize",
];

/// Module-level cooldown timestamp (epoch seconds of last reauth attempt).
/// Prevents double-tap re-trigger when the user retries quickly.
static REAUTH_COOLDOWN_TS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PendingReauth {
    pub pending: bool,
    #[serde(default)]
    pub ts: f64,
    #[serde(default)]
    pub target_account: String,
    #[serde(default)]
    pub reason: String,
}

impl PendingReauth {
    fn path() -> PathBuf {
        crate::config::omega_dir().join("state/pending-reauth.json")
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.ts.is_finite() && self.ts >= 0.0,
            "invalid reauth timestamp"
        );
        anyhow::ensure!(
            !self.pending || self.ts > 0.0,
            "pending reauth authority requires a timestamp"
        );
        anyhow::ensure!(
            self.target_account.len() <= 512 && self.reason.len() <= 4096,
            "reauth authority fields exceed their safety bounds"
        );
        Ok(())
    }

    fn load_at(path: &std::path::Path) -> Result<Self> {
        let Some(bytes) = crate::config::read_private_optional(path)? else {
            return Ok(Self::default());
        };
        let state: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing pending reauth authority {}", path.display()))?;
        state.validate()?;
        Ok(state)
    }

    pub fn load() -> Result<Self> {
        Self::load_at(&Self::path())
    }

    fn save_at(&self, path: &std::path::Path) -> Result<()> {
        self.validate()?;
        let lock = path.with_extension("json.lock");
        let _guard = crate::config::acquire_private_lock_path(&lock)?;
        crate::config::atomic_write_private(path, &serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("writing pending reauth authority {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    fn clear_at(path: &std::path::Path) -> Result<()> {
        let cleared = Self {
            pending: false,
            ts: 0.0,
            target_account: String::new(),
            reason: String::new(),
        };
        cleared.save_at(path)
    }

    pub fn clear() -> Result<()> {
        Self::clear_at(&Self::path())
    }

    /// Stale = pending but set more than PENDING_TTL_SEC ago.
    pub fn is_stale(&self) -> bool {
        if !self.pending {
            return false;
        }
        let now = now_epoch_secs() as f64;
        (now - self.ts) > PENDING_TTL_SEC as f64
    }
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Result of a reauth request.
#[derive(Debug, Clone)]
pub struct ReauthRequest {
    pub auth_url: String,
}

/// Result of a code-paste exchange.
#[derive(Debug, Clone)]
pub struct ReauthResult {
    pub success: bool,
    pub email: String,
    pub expires_min: i64,
    pub pane_tail: String,
    /// Live sessions that were already running when the fresh credentials
    /// landed, and therefore still hold the PREVIOUS authentication. Empty on
    /// a failed login (nothing was rotated, so nothing went stale).
    pub stale_sessions: Vec<StaleSession>,
}

/// Spawn the reauth session, send `/login`, and return the captured auth URL.
///
/// If a reauth is already pending and not stale → returns Ok(None) (silent skip).
/// If cooldown is active → returns Ok(None).
/// On any extraction failure → kills the session and returns an Err.
pub async fn request_reauth(
    mgr: &SessionManager,
    reason: &str,
    target_account: Option<&str>,
    force: bool,
) -> Result<Option<ReauthRequest>> {
    let now = now_epoch_secs();

    // Cooldown + pending guards apply to AUTOMATIC reauth only (so a detected
    // auth failure can't spam logins). An operator-initiated login (force=true)
    // ALWAYS spawns a fresh session and returns a fresh URL — otherwise a stuck
    // `pending` flag or the 30s cooldown makes the TUI/bridge keep showing the
    // same stale link (the "le lien est générique / identique à chaque fois"
    // symptom). The kill_session below then guarantees a clean fresh session.
    if !force {
        // Cooldown — 30s between attempts.
        let last = REAUTH_COOLDOWN_TS.load(Ordering::Relaxed);
        if last > 0 && now.saturating_sub(last) < COOLDOWN_SEC {
            tracing::debug!("reauth skipped: cooldown active ({}s ago)", now - last);
            return Ok(None);
        }

        // Stale check — clear if pending > TTL.
        let mut state = PendingReauth::load()
            .context("reading pending reauth authority before automatic login")?;
        if state.is_stale() {
            tracing::warn!("reauth: stale pending flag — auto-clearing");
            state.pending = false;
            state
                .save()
                .context("clearing stale pending reauth authority")?;
        }
        if state.pending {
            tracing::debug!("reauth skipped: already pending");
            return Ok(None);
        }
    }

    // Kill any pre-existing reauth session before starting fresh.
    let _ = mgr.kill_session(REAUTH_SESSION).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Record cooldown + pending state BEFORE spawning so a crash leaves a trail.
    REAUTH_COOLDOWN_TS.store(now, Ordering::Relaxed);
    let new_state = PendingReauth {
        pending: true,
        ts: now as f64,
        target_account: target_account.unwrap_or("").to_string(),
        reason: reason.to_string(),
    };
    new_state
        .save()
        .context("persisting pending reauth authority before session spawn")?;

    // Spawn the reauth session.
    let cmd = "claude --dangerously-skip-permissions";
    let cwd = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    if let Err(e) = mgr
        .create_session(REAUTH_SESSION, Some(&cwd), Some(cmd))
        .await
    {
        PendingReauth::clear().context("clearing reauth authority after spawn failure")?;
        return Err(anyhow::anyhow!("failed to spawn reauth session: {}", e));
    }

    // Wait for claude to boot.
    tokio::time::sleep(Duration::from_secs(8)).await;

    // Dismiss the "trust this folder" gate. claude shows it on first launch in an
    // untrusted dir (e.g. $HOME) BEFORE the input box exists; if we send `/login`
    // into that menu it lands on the wrong control and the URL never appears.
    // Default-selected option is "Yes, I trust this folder", so a single Enter
    // confirms it. Best-effort: only fires when the prompt is detected.
    let boot_pane = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
    if boot_pane.contains("trust this folder") || boot_pane.contains("Do you trust") {
        if let Ok(p) = mgr.get_active_pane(REAUTH_SESSION).await {
            let _ = p.send_key("Enter").await;
            tracing::info!("reauth: dismissed 'trust this folder' prompt");
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    // Send /login.
    if let Err(e) = mgr.send_text(REAUTH_SESSION, "/login").await {
        let _ = mgr.kill_session(REAUTH_SESSION).await;
        PendingReauth::clear().context("clearing reauth authority after login send failure")?;
        return Err(anyhow::anyhow!("send_text /login failed: {}", e));
    }
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Confirm option 1 (Claude account with subscription) — just press Enter.
    if let Err(e) = mgr.send_text(REAUTH_SESSION, "").await {
        tracing::warn!("reauth: confirm Enter failed: {}", e);
    }

    // Poll the pane for the OAuth URL instead of a single fixed sleep. claude can
    // boot slowly (cold cache, busy host) and miss a one-shot capture; poll every
    // 500ms for up to 15s and return as soon as a real authorize URL appears.
    // Also short-circuit to an error if the pane shows a known auth failure.
    let mut pane = String::new();
    let mut url = String::new();
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        pane = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
        url = extract_auth_url(&pane);
        if !url.is_empty() {
            break;
        }
        if let Some(marker) = detect_auth_failure(&pane) {
            let _ = mgr.kill_session(REAUTH_SESSION).await;
            PendingReauth::clear()
                .context("clearing reauth authority after authentication failure")?;
            return Err(anyhow::anyhow!(
                "auth failure during /login (marker: {}). Pane tail: {}",
                marker,
                pane.chars().rev().take(300).collect::<String>()
            ));
        }
    }

    if url.is_empty() {
        let _ = mgr.kill_session(REAUTH_SESSION).await;
        PendingReauth::clear().context("clearing reauth authority after URL timeout")?;
        return Err(anyhow::anyhow!(
            "could not extract OAuth URL from /login output (15s timeout). Pane tail: {}",
            pane.chars().rev().take(300).collect::<String>()
        ));
    }

    Ok(Some(ReauthRequest { auth_url: url }))
}

/// Paste an OAuth code into the waiting reauth session and wait for credentials
/// to update.
pub async fn handle_code(mgr: &SessionManager, code: &str) -> Result<ReauthResult> {
    // Verify the reauth session is alive.
    if mgr.get_session(REAUTH_SESSION).await.is_err() {
        return Err(anyhow::anyhow!(
            "no reauth session active — run /login first"
        ));
    }

    // Watch the path Claude ACTUALLY writes to (atomic write breaks symlinks,
    // so the freshest creds always land at ~/.claude/.credentials.json).
    let creds_path = claude_native_path();
    let before_mtime = creds_path
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let before_token = read_refresh_token(&creds_path);

    tracing::info!(
        code_len = code.len(),
        has_hash = code.contains('#'),
        "pasting OAuth code into reauth session"
    );

    // Verify Claude is actually waiting for the code (VPS pattern).
    let pre_capture = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
    let waiting_for_code =
        pre_capture.contains("Paste code here") || pre_capture.contains("Paste your code");
    if !waiting_for_code {
        tracing::warn!(
            "Claude does not appear to be waiting for code. Last 300 chars: {}",
            &pre_capture
                .chars()
                .rev()
                .take(300)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()
        );
    }

    // CRITICAL: paste code WITHOUT Enter, sleep 1s, then send Enter separately.
    // VPS Python pattern: load-buffer + paste-buffer + sleep 1 + send-keys Enter.
    // Without this gap, Claude /login input field rejects the paste.
    let pane = mgr
        .get_active_pane(REAUTH_SESSION)
        .await
        .context("get reauth pane failed")?;
    pane.send_text(code).await.context("paste code failed")?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    pane.send_key("Enter").await.context("send Enter failed")?;

    // Poll credentials.json for mtime change — up to 20s.
    // Also: Claude shows "Press Enter to continue..." after a successful login
    // BEFORE writing the credentials file. We detect that prompt and auto-Enter
    // so the user doesn't have to do anything else.
    let mut updated = false;
    let mut enter_sent = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Auto-confirm the "Press Enter to continue" prompt that Claude shows
        // after a successful login (one shot).
        if !enter_sent {
            let pane = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
            if pane.contains("Login successful")
                || pane.contains("Press Enter to continue")
                || pane.contains("Logged in as")
            {
                if let Ok(p) = mgr.get_active_pane(REAUTH_SESSION).await {
                    let _ = p.send_key("Enter").await;
                    enter_sent = true;
                    tracing::info!("OAuth: detected 'Login successful', sent Enter to confirm");
                }
            }
        }

        let cur_mtime = creds_path
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if cur_mtime > before_mtime {
            tokio::time::sleep(Duration::from_secs(1)).await; // let Claude finish writing
            updated = true;
            break;
        }
    }

    // Final fallback: if pane says success but mtime didn't bump, consider it
    // successful (Claude may have updated the file in place without changing mtime).
    if !updated {
        let pane = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
        if pane.contains("Logged in as") && pane.contains("Login successful") {
            tracing::info!("OAuth: pane confirms success even though mtime check didn't fire");
            updated = true;
        }
    }

    let after_token = read_refresh_token(&creds_path);
    let pane_tail = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
    let pane_tail: String = pane_tail
        .lines()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");

    // Success criteria — a real refresh_token MUST exist on disk. The pane
    // message ("Logged in as") is shown before/independently of the file write
    // and is not proof on its own: trusting it lets callers assume a valid token
    // when none was persisted. So every success path requires after_token.
    let pane_says_ok = pane_tail.contains("Logged in as");
    let success =
        !after_token.is_empty() && (updated || pane_says_ok || after_token != before_token);
    if pane_says_ok && after_token.is_empty() {
        tracing::warn!(
            "OAuth: pane reported 'Logged in as' but no refresh_token landed on disk — \
             treating as failure"
        );
    }

    // On success, sync the fresh creds Claude wrote at its native path back
    // into the omega canonical store + re-establish the symlink.
    let mut stale_sessions = Vec::new();
    if success {
        if let Err(e) = sync_credentials_to_omega() {
            tracing::warn!(error = %e, "failed to sync credentials to omega store");
        } else {
            tracing::info!("synced fresh credentials to ~/.omega/credentials/claude.json");
        }

        // The fresh token is on disk, and every session started BEFORE this
        // moment is still holding the old one: a `claude` session reads its
        // authentication once at launch and is never told the file changed.
        // Report them (see `sessions_with_stale_auth`) so the operator learns
        // it here instead of from a 401 hours later. Detection only: nothing
        // below ever kills, restarts, or interrupts a session.
        stale_sessions = sessions_with_stale_auth(mgr).await;
        if !stale_sessions.is_empty() {
            tracing::warn!(
                count = stale_sessions.len(),
                "live sessions are still running on the previous authentication"
            );
        }
    }

    // Always clean up.
    PendingReauth::clear().context("clearing completed reauth authority")?;
    let _ = mgr.kill_session(REAUTH_SESSION).await;

    if !success {
        tracing::warn!(
            updated,
            token_changed = after_token != before_token,
            "OAuth code paste did not refresh credentials"
        );
    }

    let (email, expires_min) = if success {
        let creds = read_credentials(&creds_path).unwrap_or_default();
        let expires_min = creds.expires_min();
        // Email is NOT in credentials.json — get it from `claude auth status`.
        let email = crate::account::email_from_claude_auth_status();
        (email, expires_min)
    } else {
        ("?".to_string(), 0)
    };

    Ok(ReauthResult {
        success,
        email,
        expires_min,
        pane_tail,
        stale_sessions,
    })
}

/// Scan a pane snapshot for known auth-failure markers.
pub fn detect_auth_failure(pane_text: &str) -> Option<&'static str> {
    let lower = pane_text.to_lowercase();
    AUTH_FAILURE_MARKERS
        .iter()
        .find(|&&marker| lower.contains(&marker.to_lowercase()))
        .copied()
}

/// Does this look like a COMPLETE authorize URL, not a half-painted one?
///
/// `redirect_uri` is the deciding parameter: it sits in the tail of the query
/// string and is what Anthropic's OAuth server rejects the request without
/// ("Invalid OAuth Request — Missing redirect_uri parameter"). Anything that
/// stops before it is a screen still being drawn, not a link.
fn auth_url_is_complete(url: &str) -> bool {
    match url.split_once("redirect_uri=") {
        // A bare "redirect_uri=" with nothing after it is the same half-render,
        // caught one paint later.
        Some((_, tail)) => !tail.is_empty(),
        None => false,
    }
}

/// Byte offset of the EARLIEST `AUTHORIZE_URL_PREFIXES` match in `line`.
///
/// Earliest, not first-in-the-list: the pane may prefix the link with a label,
/// and the leftmost match is the start of the URL whichever form it takes.
fn find_authorize_prefix(line: &str) -> Option<usize> {
    AUTHORIZE_URL_PREFIXES
        .iter()
        .filter_map(|p| line.find(p))
        .min()
}

/// Does `s` BEGIN with one of the recognised authorization-URL prefixes?
fn starts_with_authorize_prefix(s: &str) -> bool {
    AUTHORIZE_URL_PREFIXES.iter().any(|p| s.starts_with(p))
}

/// Extract the OAuth URL from the captured pane.
///
/// Strategy:
///   1. Scan lines for one carrying an `AUTHORIZE_URL_PREFIXES` prefix.
///   2. Continue concatenating subsequent non-empty lines that look like a URL
///      continuation (no whitespace, no prompt markers).
///   3. Trim trailing punctuation/whitespace.
///   4. Return "" unless the result is COMPLETE — see `auth_url_is_complete`.
///
/// Step 4 is load-bearing for the caller: `start_reauth` polls this every 500ms
/// and breaks on the first non-empty answer. Claude paints its /login screen
/// progressively, so returning a truncated URL there would hand the operator a
/// dead link. Reporting "not found yet" instead makes the poll wait one more
/// tick for the screen to finish.
pub fn extract_auth_url(pane_text: &str) -> String {
    let mut parts = Vec::new();
    let mut in_url = false;

    for line in pane_text.lines() {
        let stripped = line.trim();
        if !in_url {
            if let Some(idx) = find_authorize_prefix(stripped) {
                in_url = true;
                parts.push(stripped[idx..].to_string());
            }
        } else {
            if stripped.is_empty()
                || stripped.contains(' ')
                || stripped.starts_with("Paste")
                || stripped.starts_with("Esc")
                || stripped.starts_with('❯')
                || stripped.starts_with("Browser")
            {
                break;
            }
            parts.push(stripped.to_string());
        }
    }

    let candidate: String = parts.join("");

    // Keep only the valid URL char set.
    let mut out = String::with_capacity(candidate.len());
    let mut started = false;
    for c in candidate.chars() {
        if !started && starts_with_authorize_prefix(&candidate) {
            started = true;
        }
        if !started {
            continue;
        }
        if c.is_ascii_alphanumeric()
            || matches!(
                c,
                '.' | '_'
                    | '~'
                    | ':'
                    | '/'
                    | '?'
                    | '#'
                    | '['
                    | ']'
                    | '@'
                    | '!'
                    | '$'
                    | '&'
                    | '\''
                    | '('
                    | ')'
                    | '*'
                    | '+'
                    | ','
                    | ';'
                    | '='
                    | '%'
                    | '-'
            )
        {
            out.push(c);
        } else {
            break;
        }
    }
    if !auth_url_is_complete(&out) {
        return String::new();
    }
    out
}

/// Check if a string looks like an OAuth code paste.
///
/// OAuth callback codes are typically 20+ chars of `[A-Za-z0-9_-]`, optionally
/// followed by `#<state>`. We strip the `#...` suffix before testing.
pub fn looks_like_oauth_code(s: &str) -> bool {
    let core = s.split('#').next().unwrap_or(s);
    if core.len() < 20 || core.len() > 512 {
        return false;
    }
    core.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

// ───────────────────────── credentials helpers ─────────────────────────

/// Canonical Claude credentials path: `~/.omega/credentials/claude.json`.
///
/// The legacy `~/.claude/.credentials.json` is a symlink to this file
/// (set up by `install.sh` and `CredentialStore::ensure_legacy_symlink`).
/// During the transition window, if the canonical file does not yet exist
/// but the legacy file does, we fall back to the legacy path so first-launch
/// after upgrade still works before migration runs.
pub fn credentials_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let canonical = crate::config::omega_dir()
        .join("credentials")
        .join("claude.json");
    let legacy = home.join(".claude").join(".credentials.json");
    // Prefer canonical ONLY if it actually holds a usable refresh_token. A
    // present-but-stale/empty canonical (e.g. a truncated write) must not shadow
    // a fresh legacy file Claude just wrote — otherwise reads return dead creds.
    if canonical.exists() && !read_refresh_token(&canonical).is_empty() {
        return canonical;
    }
    if legacy.exists() && !read_refresh_token(&legacy).is_empty() {
        return legacy;
    }
    // Neither has a token — fall back to whichever path exists so the caller's
    // error surfaces against a real path; default to canonical.
    if canonical.exists() {
        return canonical;
    }
    if legacy.exists() {
        return legacy;
    }
    canonical
}

/// The path Claude Code ACTUALLY writes to during `/login`.
/// Claude does an atomic write (temp + rename) which replaces any symlink
/// at this path with a fresh regular file — so this is where the freshest
/// credentials always land. We watch THIS for the login mtime check.
pub fn claude_native_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
        .join(".credentials.json")
}

/// After a successful login, sync the fresh credentials Claude wrote at its
/// native path into the omega canonical store, then re-establish the symlink
/// so future reads go through omega. Idempotent.
pub fn sync_credentials_to_omega() -> std::io::Result<()> {
    let native = claude_native_path();
    // Use the SAME resolver credentials_path() reads from — honoring $OMEGA_DIR /
    // the consolidated ~/OmegaOS/System layout. A hardcoded ~/.omega here would
    // write fresh OAuth creds where the reader never looks under a relocated root.
    let canonical = crate::config::omega_dir()
        .join("credentials")
        .join("claude.json");

    // If native is a real file (Claude's atomic write broke the symlink),
    // copy it into omega and re-link.
    let meta = std::fs::symlink_metadata(&native)?;
    if meta.file_type().is_symlink() {
        // Still a symlink — nothing to do, both already point at omega.
        return Ok(());
    }

    // Native is a real file (Claude's atomic write replaced the symlink) holding
    // the freshest creds. Resolve the canonical store to the REAL file it points
    // at: `~/.omega/credentials/claude.json` is itself a symlink to the shared
    // `/Shared/claude/credentials.json` on multi-user hosts. We must write THAT
    // shared file — never clobber the omega→shared symlink.
    if let Some(parent) = canonical.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let shared_target = std::fs::canonicalize(&canonical).unwrap_or_else(|_| canonical.clone());

    // Atomic, EPERM-proof shared write. The old code did `std::fs::copy` straight
    // onto the shared file, which (1) TRUNCATED it in place — a concurrent reader
    // during the write saw a 0-byte file and every session 401'd — and (2) then
    // tried to fchmod the (often root-owned) shared file, hitting EPERM, which
    // aborted the whole sync and left native a stray regular file so the symlink
    // chain was never restored. Instead: stage a temp we OWN in the target's dir,
    // chmod the TEMP only, then rename(2) over the target. rename is atomic, so
    // readers always see the old-or-new complete file — never a truncated one.
    let bytes = std::fs::read(&native)?;
    let staged = shared_target.with_extension("omega-sync.tmp");
    let _ = std::fs::remove_file(&staged); // clear any stale temp
    std::fs::write(&staged, &bytes)?;
    let _ = std::fs::set_permissions(&staged, std::os::unix::fs::PermissionsExt::from_mode(0o660));
    if let Err(e) = std::fs::rename(&staged, &shared_target) {
        let _ = std::fs::remove_file(&staged); // don't leak the temp
        return Err(e);
    }

    // Shared store now holds the fresh creds. Re-establish native → canonical so
    // future reads route through omega again. Atomic (temp symlink + rename) so a
    // failure never leaves native deleted (which would let Claude recreate an
    // unprotected regular file there).
    let tmp_link = native.with_extension("omega-relink.tmp");
    let _ = std::fs::remove_file(&tmp_link); // clear any stale temp link
    std::os::unix::fs::symlink(&canonical, &tmp_link)?;
    if let Err(e) = std::fs::rename(&tmp_link, &native) {
        let _ = std::fs::remove_file(&tmp_link); // don't leak the temp link
        return Err(e);
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct CredentialsInfo {
    pub email: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at_ms: i64,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

impl CredentialsInfo {
    /// Minutes until expiry (negative if expired).
    pub fn expires_min(&self) -> i64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        (self.expires_at_ms - now_ms) / 60_000
    }

    pub fn is_valid(&self) -> bool {
        self.expires_min() > 0
    }
}

pub fn read_credentials(path: &std::path::Path) -> Result<CredentialsInfo> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let v: serde_json::Value =
        serde_json::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;
    let oauth = v
        .get("claudeAiOauth")
        .ok_or_else(|| anyhow::anyhow!("missing claudeAiOauth"))?;

    let refresh_token = oauth.get("refreshToken").and_then(|e| e.as_str());
    let access_token = oauth.get("accessToken").and_then(|e| e.as_str());
    let expires_at = oauth.get("expiresAt").and_then(|e| e.as_i64());
    // claudeAiOauth is present but the auth-bearing fields are missing/null →
    // the file is malformed (truncated/corrupt write), NOT merely legacy. Don't
    // silently default to empty/0, which is indistinguishable from "no creds";
    // surface it so the caller can react instead of trusting dead credentials.
    if refresh_token.is_none() || access_token.is_none() || expires_at.is_none() {
        tracing::warn!(
            path = %path.display(),
            has_refresh = refresh_token.is_some(),
            has_access = access_token.is_some(),
            has_expires = expires_at.is_some(),
            "credentials file is malformed: claudeAiOauth present but auth fields missing"
        );
        return Err(anyhow::anyhow!(
            "credentials file is malformed (claudeAiOauth present but missing auth fields): {}",
            path.display()
        ));
    }

    Ok(CredentialsInfo {
        email: oauth
            .get("email")
            .and_then(|e| e.as_str())
            .map(String::from),
        refresh_token: refresh_token.map(String::from),
        access_token: access_token.map(String::from),
        expires_at_ms: expires_at.unwrap_or(0),
        subscription_type: oauth
            .get("subscriptionType")
            .and_then(|e| e.as_str())
            .map(String::from),
        rate_limit_tier: oauth
            .get("rateLimitTier")
            .and_then(|e| e.as_str())
            .map(String::from),
    })
}

fn read_refresh_token(path: &std::path::Path) -> String {
    read_credentials(path)
        .ok()
        .and_then(|c| c.refresh_token)
        .unwrap_or_default()
}

// ─────────────────── sessions left on the previous auth ───────────────────
//
// A successful login rewrites the credentials file, and NOTHING tells the
// sessions already running about it. Atlas survives this because it re-execs a
// fresh `claude -p` per message and therefore re-reads the file every time; an
// rmux session does not, because it is one long-lived `claude` process that
// read its authentication once at launch and closed the file. Measured on the
// operator's box: live `claude` processes 10 to 19 days old against a
// credentials file rewritten the same day. The 30-minute token-refresh cron
// makes it worse, since it ROTATES the refresh token and follows the rotation
// into exactly one consumer.
//
// So the login knows something the operator does not, and the whole point of
// this section is to say it out loud. DETECT AND REPORT ONLY: nothing here
// kills, restarts, signals, or interrupts a session. Killing one would destroy
// whatever work it is holding, and the decision belongs to the operator.

/// A live session still running on the authentication that preceded the login
/// that just completed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleSession {
    /// rmux session name, exactly as `omega sessions` lists it.
    pub name: String,
    /// Epoch seconds at which this session's agent process was launched.
    pub started_at: i64,
    /// How long it has been running, in seconds.
    pub running_for_secs: i64,
}

impl StaleSession {
    /// Coarse human age ("19d", "6h", "12m") for operator-facing text.
    pub fn running_for_human(&self) -> String {
        let secs = self.running_for_secs.max(0);
        match secs {
            s if s >= 86_400 => format!("{}d", s / 86_400),
            s if s >= 3_600 => format!("{}h", s / 3_600),
            s if s >= 60 => format!("{}m", s / 60),
            s => format!("{s}s"),
        }
    }
}

/// The pure core: of the live Claude-bearing sessions and the instant each one
/// started, which ones started BEFORE the credentials file was written?
///
/// Split out from the rmux/filesystem lookups precisely so this decision is
/// testable without a live daemon or a real process.
///
/// A session that started at exactly `credentials_written_at` is NOT reported.
/// One-second mtime granularity cannot order those two events, and an
/// unprovable claim about the operator's running work is worse than a missed
/// one: the report has to be trustworthy to be worth printing at all.
/// Oldest first, then by name, so the same input always renders the same way.
pub fn select_stale_auth_sessions(
    sessions: &[(String, i64)],
    credentials_written_at: i64,
    now: i64,
) -> Vec<StaleSession> {
    let mut stale: Vec<StaleSession> = sessions
        .iter()
        .filter(|(_, started_at)| *started_at < credentials_written_at)
        .map(|(name, started_at)| StaleSession {
            name: name.clone(),
            started_at: *started_at,
            running_for_secs: (now - *started_at).max(0),
        })
        .collect();
    stale.sort_by(|a, b| a.started_at.cmp(&b.started_at).then(a.name.cmp(&b.name)));
    stale
}

/// Epoch seconds at which OmegaOS launched `session`'s agent process.
///
/// Read from the provider record `SessionManager` writes beside every typed
/// agent session — the same file `list_sessions` already reads the provider
/// out of, so this is one more field of a record we are holding anyway rather
/// than a second mechanism for tracking sessions. It is also the RIGHT
/// instant: `relaunch_agent_session_with_opts` rewrites the record when it
/// replaces a pane's agent, so the timestamp follows the process that actually
/// holds the token, not merely the session's creation.
///
/// `None` when the record is absent, unreadable, or names a different session
/// (mirroring `read_session_provider`'s own guard). An unknown age cannot be
/// compared to anything, so such a session is left out of the report.
fn session_agent_started_at(name: &str) -> Option<i64> {
    let path = crate::config::omega_dir().join("state").join(format!(
        "session-provider-{}.json",
        sanitize_session_name(name)
    ));
    agent_start_from_record(&path, name)
}

/// The parsing half of `session_agent_started_at`, split off the `$OMEGA_DIR`
/// lookup so it can be tested against a real record on disk without an
/// environment override.
fn agent_start_from_record(path: &std::path::Path, name: &str) -> Option<i64> {
    let payload: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    if payload.get("session")?.as_str()? != sanitize_session_name(name) {
        return None;
    }
    let recorded_at = payload.get("recorded_at")?.as_str()?;
    Some(
        chrono::DateTime::parse_from_rfc3339(recorded_at)
            .ok()?
            .timestamp(),
    )
}

/// Every live Claude session that is still running on the authentication from
/// before the current credentials were written.
///
/// Enumerates through `SessionManager::list_sessions` — the one mechanism the
/// patrol and the TUI already use — and keeps the sessions whose recorded
/// provider is Claude, since those are the ones that read this credentials
/// file. Codex, GLM and plain shells authenticate elsewhere and are not
/// affected by a Claude login.
///
/// Never fails the caller: a login that genuinely succeeded must not be
/// reported as broken because a report about it could not be assembled, so
/// every unreadable input degrades to "nothing to report" plus a log line.
pub async fn sessions_with_stale_auth(mgr: &SessionManager) -> Vec<StaleSession> {
    let creds_path = credentials_path();
    let Some(written_at) = creds_path
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
    else {
        tracing::warn!(
            path = %creds_path.display(),
            "cannot read credentials mtime — skipping the stale-session report"
        );
        return Vec::new();
    };

    let sessions = match mgr.list_sessions().await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::warn!(error = %e, "cannot list sessions — skipping the stale-session report");
            return Vec::new();
        }
    };

    let started: Vec<(String, i64)> = sessions
        .into_iter()
        // The reauth session is this login's own plumbing, killed moments from
        // now by `handle_code`. Reporting it as the operator's stale work would
        // be a false positive on the very first line of the report.
        .filter(|s| s.name != REAUTH_SESSION)
        .filter(|s| s.provider.as_deref() == Some("claude"))
        .filter_map(|s| session_agent_started_at(&s.name).map(|started| (s.name, started)))
        .collect();

    select_stale_auth_sessions(&started, written_at, now_epoch_secs() as i64)
}

/// The operator-facing sentence for a stale-session report, or `None` when
/// every live Claude session is already on the fresh token.
///
/// Says how many, says which, and says the only remedy that exists: open a new
/// session. It deliberately does NOT offer to restart or kill anything —
/// OmegaOS will not end a session that is holding the operator's work, so
/// suggesting it here would promise something the system must never do.
pub fn stale_auth_notice(stale: &[StaleSession]) -> Option<String> {
    if stale.is_empty() {
        return None;
    }
    const SHOWN: usize = 8;
    let mut named: Vec<String> = stale
        .iter()
        .take(SHOWN)
        .map(|s| format!("{} (running {})", s.name, s.running_for_human()))
        .collect();
    if stale.len() > SHOWN {
        named.push(format!("and {} more", stale.len() - SHOWN));
    }
    let (subject, they) = if stale.len() == 1 {
        ("1 live session is".to_string(), "it")
    } else {
        (format!("{} live sessions are", stale.len()), "they")
    };
    Some(format!(
        "{subject} still running on the PREVIOUS authentication: {}. Each read its token once at \
         launch and is never told the file changed, so {they} keep using the old one. Open a NEW \
         session to work on the fresh login; OmegaOS will not restart or interrupt a running \
         session.",
        named.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_oauth_code_accepts_realistic() {
        assert!(looks_like_oauth_code("abc123_DEF-456_xyz789012345"));
        assert!(looks_like_oauth_code(
            "abc123_DEF-456_xyz789012345#state_value"
        ));
    }

    #[test]
    fn looks_like_oauth_code_rejects() {
        assert!(!looks_like_oauth_code("short"));
        assert!(!looks_like_oauth_code("has spaces in it abcdef0123456789"));
        assert!(!looks_like_oauth_code("has/slash/in/it_abcdef0123456789"));
        assert!(!looks_like_oauth_code(""));
    }

    #[test]
    fn extract_auth_url_finds_url() {
        let pane = "\
            Some pre-text\n\
            https://claude.com/cai/oauth/authorize?code=true&client_id=abc\
&redirect_uri=https%3A%2F%2Fconsole.anthropic.com%2Foauth%2Fcode%2Fcallback&state=xyz\n\
            Paste code here:\n\
        ";
        let url = extract_auth_url(pane);
        assert!(url.starts_with("https://claude.com/cai/oauth/authorize"));
        assert!(url.contains("client_id=abc"));
    }

    #[test]
    fn extract_auth_url_handles_no_url() {
        assert_eq!(extract_auth_url("nothing here"), "");
    }

    /// The poll loop in `start_reauth` breaks on the FIRST non-empty extraction.
    /// Claude paints its /login screen progressively, so a 500ms tick can catch
    /// the pane mid-render and see only the head of the URL. `redirect_uri` sits
    /// in the TAIL of the query string, so that truncated URL is accepted and the
    /// operator lands on "Invalid OAuth Request — Missing redirect_uri parameter".
    /// An incomplete URL must therefore read as NOT FOUND, so the loop keeps
    /// polling until the screen has finished painting.
    #[test]
    fn extract_auth_url_rejects_a_half_painted_url() {
        let half = "\
            https://claude.com/cai/oauth/authorize?code=true&client_id=abc\n\
        ";
        assert_eq!(
            extract_auth_url(half),
            "",
            "a URL without redirect_uri is half-rendered, not a usable link"
        );

        let complete = "\
            https://claude.com/cai/oauth/authorize?code=true&client_id=abc\
&response_type=code&redirect_uri=https%3A%2F%2Fconsole.anthropic.com%2Foauth%2Fcode%2Fcallback&state=xyz\n\
        ";
        let url = extract_auth_url(complete);
        assert!(url.contains("redirect_uri=https%3A"), "got {url:?}");
        assert!(url.contains("state=xyz"), "got {url:?}");
    }

    /// A wrapped URL (terminal folded it across physical lines) must be rejoined
    /// into one link — that is the whole reason the extractor concatenates.
    #[test]
    fn extract_auth_url_rejoins_a_wrapped_url() {
        let pane = "\
            https://claude.com/cai/oauth/authorize?code=true&client_id=abc&response_type=code\n\
            &redirect_uri=https%3A%2F%2Fconsole.anthropic.com%2Foauth%2Fcode%2Fcallback\n\
            &state=xyz\n\
            Paste code here:\n\
        ";
        let url = extract_auth_url(pane);
        assert!(url.contains("redirect_uri=https%3A"), "got {url:?}");
        assert!(url.ends_with("state=xyz"), "got {url:?}");
    }

    /// The 2.1.232 binary carries a SECOND authorize host beside the historical
    /// one (`platform.claude.com/oauth/authorize`, verified with a strings grep
    /// over the binary). The extractor recognised only the historical form, so a
    /// login that painted the platform variant returned "" on all 30 poll ticks
    /// and the operator got "Link not generated" instead of a link.
    #[test]
    fn extract_auth_url_accepts_the_platform_form() {
        let pane = "\
            Browser didn't open? Use the url below to sign in:\n\
            https://platform.claude.com/oauth/authorize?code=true&client_id=abc\
&response_type=code&redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback&state=xyz\n\
            Paste code here:\n\
        ";
        let url = extract_auth_url(pane);
        assert!(
            url.starts_with("https://platform.claude.com/oauth/authorize"),
            "got {url:?}"
        );
        assert!(url.contains("client_id=abc"), "got {url:?}");
        assert!(url.contains("redirect_uri=https%3A"), "got {url:?}");
        assert!(url.ends_with("state=xyz"), "got {url:?}");
    }

    /// Every form flows through the SAME constant, and gaining the new host must
    /// not cost the old one: an operator on an older Claude still gets the
    /// `claude.com/cai/…` screen, so dropping it would only move the outage.
    /// A prefix listed and not honoured is the exact defect the list prevents.
    #[test]
    fn extract_auth_url_accepts_every_listed_prefix() {
        assert!(AUTHORIZE_URL_PREFIXES.contains(&"https://claude.com/cai/oauth/authorize"));
        assert!(AUTHORIZE_URL_PREFIXES.contains(&"https://platform.claude.com/oauth/authorize"));

        for prefix in AUTHORIZE_URL_PREFIXES {
            let pane = format!(
                "Some pre-text\n\
                 {prefix}?code=true&client_id=abc&response_type=code\
&redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback&state=xyz\n\
                 Paste code here:\n"
            );
            let url = extract_auth_url(&pane);
            assert!(url.starts_with(*prefix), "prefix {prefix} yielded {url:?}");
            assert!(
                url.ends_with("state=xyz"),
                "prefix {prefix} yielded {url:?}"
            );
        }
    }

    /// The truncation guard covers the platform form too. `start_reauth` breaks
    /// on the first non-empty extraction, so a platform URL caught mid-paint has
    /// to read as NOT FOUND and let the poll wait one more repaint — otherwise
    /// widening the prefix list would trade a missing link for a dead one.
    /// Both shapes of incomplete are checked: no `redirect_uri` at all, and the
    /// bare `redirect_uri=` that `auth_url_is_complete` rejects on an empty tail.
    #[test]
    fn extract_auth_url_rejects_a_half_painted_platform_url() {
        let half = "\
            https://platform.claude.com/oauth/authorize?code=true&client_id=abc\n\
        ";
        assert_eq!(
            extract_auth_url(half),
            "",
            "a platform URL without redirect_uri is half-rendered, not a link"
        );

        let bare_param = "\
            https://platform.claude.com/oauth/authorize?code=true&client_id=abc&redirect_uri=\n\
        ";
        assert_eq!(
            extract_auth_url(bare_param),
            "",
            "a bare redirect_uri= is the same half-render, one paint later"
        );

        // Positive control: the SAME pane once the tail lands. Without it both
        // assertions above stay green on a build that simply never recognised
        // the platform prefix, which is the very bug under test.
        let complete = "\
            https://platform.claude.com/oauth/authorize?code=true&client_id=abc\
&redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback\n\
        ";
        assert!(
            extract_auth_url(complete).contains("redirect_uri=https%3A"),
            "the same URL must extract once redirect_uri has painted"
        );
    }

    /// The platform URL is LONGER than the historical one, so the terminal folds
    /// it at least as often. Rejoining must work on it identically.
    #[test]
    fn extract_auth_url_rejoins_a_wrapped_platform_url() {
        let pane = "\
            https://platform.claude.com/oauth/authorize?code=true&client_id=abc&response_type=code\n\
            &redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback\n\
            &state=xyz\n\
            Paste code here:\n\
        ";
        let url = extract_auth_url(pane);
        assert!(
            url.starts_with("https://platform.claude.com/oauth/authorize"),
            "got {url:?}"
        );
        assert!(url.contains("redirect_uri=https%3A"), "got {url:?}");
        assert!(url.ends_with("state=xyz"), "got {url:?}");
    }

    #[test]
    fn detect_auth_failure_catches_401() {
        assert_eq!(detect_auth_failure("Got 401 Unauthorized"), Some("401"));
        assert_eq!(
            detect_auth_failure("rate_limit_error: too many"),
            Some("rate_limit_error")
        );
        assert!(detect_auth_failure("everything is fine").is_none());
    }

    // ─────────── sessions left on the previous auth (pure core) ───────────
    //
    // The selector is split out from rmux and the filesystem so these run with
    // no daemon, no real process and no credentials file: the whole decision
    // is (session start instants, credential write instant) in, names out.

    /// Names + start instants, in the shape `select_stale_auth_sessions` takes.
    fn starts(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
        pairs
            .iter()
            .map(|(name, at)| ((*name).to_string(), *at))
            .collect()
    }

    fn names(stale: &[StaleSession]) -> Vec<&str> {
        stale.iter().map(|s| s.name.as_str()).collect()
    }

    /// The load-bearing case: a login at T=1000 leaves everything started
    /// before T=1000 on the old token, and only those. The age is measured
    /// against `now`, not against the login, because what the operator needs to
    /// recognise a session is how long it has been running.
    #[test]
    fn select_stale_auth_reports_exactly_the_sessions_that_predate_the_login() {
        let sessions = starts(&[
            ("verba-claude", 100),
            ("oracle-loumna", 900),
            ("OmegaOS-claude", 1_500),
        ]);
        let stale = select_stale_auth_sessions(&sessions, 1_000, 2_000);

        assert_eq!(names(&stale), vec!["verba-claude", "oracle-loumna"]);
        assert_eq!(stale[0].started_at, 100);
        assert_eq!(stale[0].running_for_secs, 1_900);
        assert_eq!(stale[1].running_for_secs, 1_100);
    }

    /// The zero case. Every live session started after the credentials landed,
    /// so there is nothing to report and the login says nothing extra. An empty
    /// list is the answer, never a defensive "could not tell".
    #[test]
    fn select_stale_auth_is_empty_when_every_session_is_fresh() {
        let sessions = starts(&[("a", 1_200), ("b", 1_800)]);
        assert!(select_stale_auth_sessions(&sessions, 1_000, 2_000).is_empty());
        assert!(select_stale_auth_sessions(&[], 1_000, 2_000).is_empty());
    }

    /// No false positive, which is the defect that would kill this feature's
    /// credibility fastest: a session opened AFTER the login already holds the
    /// fresh token, and telling the operator to reopen it is a lie that trains
    /// them to ignore the whole report. The equal-instant case counts as fresh
    /// too: one-second mtime granularity cannot order those two events, so the
    /// claim is unprovable and is not made.
    #[test]
    fn select_stale_auth_never_reports_a_session_started_after_the_login() {
        let after = starts(&[("opened-after", 1_001)]);
        assert!(select_stale_auth_sessions(&after, 1_000, 2_000).is_empty());

        let same_second = starts(&[("opened-during", 1_000)]);
        assert!(
            select_stale_auth_sessions(&same_second, 1_000, 2_000).is_empty(),
            "a session started in the same second the file was written is not provably stale"
        );

        // And the boundary one tick earlier IS reported, so the guard above is
        // a real boundary rather than a filter that never fires.
        let just_before = starts(&[("opened-just-before", 999)]);
        assert_eq!(
            names(&select_stale_auth_sessions(&just_before, 1_000, 2_000)),
            vec!["opened-just-before"]
        );
    }

    /// Oldest first, then by name. The operator reads this list to decide what
    /// to reopen, and the session that has been drifting for 19 days is the one
    /// that has to be on the first line.
    #[test]
    fn select_stale_auth_orders_oldest_first_then_by_name() {
        let sessions = starts(&[("zebra", 500), ("alpha", 500), ("ancient", 10)]);
        assert_eq!(
            names(&select_stale_auth_sessions(&sessions, 1_000, 2_000)),
            vec!["ancient", "alpha", "zebra"]
        );
    }

    #[test]
    fn running_for_human_is_coarse_and_readable() {
        let age = |secs| StaleSession {
            name: "s".into(),
            started_at: 0,
            running_for_secs: secs,
        }
        .running_for_human();

        assert_eq!(age(19 * 86_400), "19d");
        assert_eq!(age(6 * 3_600), "6h");
        assert_eq!(age(12 * 60), "12m");
        assert_eq!(age(5), "5s");
    }

    /// Silence when there is nothing to say. A login that left no session
    /// behind must not print a reassuring paragraph nobody needs to read.
    #[test]
    fn stale_auth_notice_is_silent_when_nothing_is_stale() {
        assert_eq!(stale_auth_notice(&[]), None);
    }

    /// The message the operator actually acts on: how many, which ones, and the
    /// only remedy that exists. It must NEVER offer to restart or kill a
    /// session — OmegaOS will not end a session holding live work, so promising
    /// it here would be promising something the system must refuse to do.
    #[test]
    fn stale_auth_notice_names_the_sessions_and_never_offers_a_kill() {
        let stale = select_stale_auth_sessions(
            &starts(&[("verba-claude", 0), ("oracle-loumna", 100)]),
            1_000,
            19 * 86_400,
        );
        let notice = stale_auth_notice(&stale).expect("two stale sessions must produce a notice");

        assert!(notice.contains("2 live sessions are"), "got {notice:?}");
        assert!(notice.contains("verba-claude (running 19d)"), "got {notice:?}");
        assert!(notice.contains("oracle-loumna"), "got {notice:?}");
        assert!(notice.contains("Open a NEW session"), "got {notice:?}");
        for banned in ["kill", "restart it", "terminate", "/kill"] {
            assert!(
                !notice.to_lowercase().contains(banned),
                "the notice must not suggest ending a session ({banned}): {notice:?}"
            );
        }

        // Singular reads as a sentence too, not "1 live sessions are".
        let one = select_stale_auth_sessions(&starts(&[("solo", 0)]), 1_000, 3_600);
        let notice = stale_auth_notice(&one).expect("one stale session must produce a notice");
        assert!(notice.starts_with("1 live session is"), "got {notice:?}");
    }

    /// A box with dozens of live sessions must not emit an unbounded wall of
    /// names into a Telegram card: the list is capped and the remainder is
    /// counted, so the total stays honest either way.
    #[test]
    fn stale_auth_notice_caps_the_list_but_keeps_the_count_honest() {
        let many: Vec<(String, i64)> = (0..12)
            .map(|i| (format!("session-{i:02}"), i as i64))
            .collect();
        let stale = select_stale_auth_sessions(&many, 1_000, 2_000);
        assert_eq!(stale.len(), 12);

        let notice = stale_auth_notice(&stale).expect("12 stale sessions must produce a notice");
        assert!(notice.contains("12 live sessions are"), "got {notice:?}");
        assert!(notice.contains("and 4 more"), "got {notice:?}");
        assert!(!notice.contains("session-08"), "got {notice:?}");
    }

    /// The start instant is read from the provider record `SessionManager`
    /// writes at launch, in the exact shape it writes it (this fixture is a
    /// verbatim copy of a live `session-provider-*.json` from the operator's
    /// box, whose `recorded_at` matched rmux's own "created" line to the
    /// second). A record that names a DIFFERENT session is refused rather than
    /// trusted, mirroring `read_session_provider`'s own guard: a stale file
    /// left by a killed-and-recreated session must not date a live one.
    #[test]
    fn agent_start_is_read_from_the_provider_record_and_rejects_a_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("session-provider-verba-claude.json");
        std::fs::write(
            &path,
            r#"{"session":"verba-claude","provider":"claude",
                "recorded_at":"2026-08-11T20:45:12.567119067+00:00"}"#,
        )
        .unwrap();

        assert_eq!(
            agent_start_from_record(&path, "verba-claude"),
            Some(1_786_481_112),
        );
        assert_eq!(
            agent_start_from_record(&path, "some-other-session"),
            None,
            "a record naming another session cannot date this one"
        );

        assert_eq!(
            agent_start_from_record(&tmp.path().join("absent.json"), "verba-claude"),
            None
        );

        let broken = tmp.path().join("session-provider-broken.json");
        std::fs::write(&broken, r#"{"session":"broken","provider":"claude"}"#).unwrap();
        assert_eq!(
            agent_start_from_record(&broken, "broken"),
            None,
            "a record with no recorded_at yields no age, never a fabricated one"
        );
    }

    #[test]
    fn pending_reauth_stale_detection() {
        let mut p = PendingReauth {
            pending: true,
            ts: (now_epoch_secs() - PENDING_TTL_SEC - 10) as f64,
            ..PendingReauth::default()
        };
        assert!(p.is_stale());

        p.ts = now_epoch_secs() as f64;
        assert!(!p.is_stale());
    }

    #[cfg(unix)]
    #[test]
    fn pending_reauth_authority_is_private_strict_and_alias_safe() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state/pending-reauth.json");
        let state = PendingReauth {
            pending: true,
            ts: 42.0,
            target_account: "operator".to_string(),
            reason: "expired".to_string(),
        };
        state.save_at(&path).unwrap();
        assert_eq!(
            PendingReauth::load_at(&path).unwrap().target_account,
            "operator"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        std::fs::write(
            &path,
            r#"{"pending":false,"ts":0,"target_account":"","reason":"","extra":true}"#,
        )
        .unwrap();
        assert!(PendingReauth::load_at(&path).is_err());

        let external = tmp.path().join("external.json");
        std::fs::write(&external, serde_json::to_vec(&state).unwrap()).unwrap();
        std::fs::set_permissions(&external, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::remove_file(&path).unwrap();
        symlink(&external, &path).unwrap();
        assert!(PendingReauth::load_at(&path).is_err());

        std::fs::remove_file(&path).unwrap();
        std::fs::hard_link(&external, &path).unwrap();
        assert!(PendingReauth::load_at(&path).is_err());
    }
}
