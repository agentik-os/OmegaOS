//! OAuth flow for OmegaOS — re-implements the AISB Python `_request_reauth` /
//! `_handle_code` pair in Rust against the rmux SDK.
//!
//! The flow:
//!   1. `request_reauth` spawns a dedicated rmux session `aisb-reauth` running
//!      `claude`, sends `/login`, and captures the OAuth URL from the pane.
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

use crate::session::SessionManager;

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
    // Emitted when the authorize step itself is refused and when a pasted code
    // is rejected. Without them the poll loop below has nothing to short-circuit
    // on, so a login that has ALREADY failed still burns its full 15s and then
    // reports a timeout — hiding the real answer behind the wrong error.
    "OAuth error",
    "Invalid code",
];

/// Modal dialogs that can hold the reauth pane BEFORE its input box exists,
/// as `(human name, signatures that must ALL be on screen)`.
///
/// Nothing here DRIVES a modal — the pane is launched so that none of them can
/// appear (see the command in `request_reauth`). This exists so that when one
/// shows up anyway, the failure NAMES it instead of reporting a bare timeout.
/// The Bypass Permissions dialog can still be raised by an operator's own
/// `~/.claude/settings.json` (`permissions.defaultMode: bypassPermissions`),
/// which no flag of ours controls.
///
/// Signatures are required as a SET, never singly: "Bypass Permissions mode" on
/// its own is a phrase ordinary prose carries, and the option strings are shared
/// between dialogs (folder-trust and Bypass Permissions both offer "No, exit").
/// Pairing a subject with that dialog's own accept option is what makes this a
/// modal detector and not a keyword grep.
const BLOCKING_MODALS: &[(&str, &[&str])] = &[
    (
        "Bypass Permissions confirmation (default answer is \"No, exit\")",
        &["Bypass Permissions mode", "Yes, I accept"],
    ),
    // A rendered menu option, not a phrase prose produces on its own, so it
    // needs no second signature to be unambiguous.
    ("folder-trust confirmation", &["Yes, I trust this folder"]),
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
    //
    // Deliberately WITHOUT `--dangerously-skip-permissions`. This pane types
    // `/login` and nothing else — it never calls a tool, never edits a file,
    // never runs a command — so the flag bought it no capability it uses, while
    // costing it a blocking modal: on any machine where the operator has not
    // already accepted that dialog (a FRESH INSTALL, which is exactly what the
    // symptom reports), `claude --dangerously-skip-permissions` stops on
    // "WARNING: Claude Code running in Bypass Permissions mode" whose
    // pre-selected option is "No, exit". The single Enter this flow sends
    // therefore ANSWERED NO and killed the pane, `/login` was never interpreted,
    // no URL was ever painted, and the operator got the 15s timeout below
    // instead of a link. Verified at runtime on 2.1.232 against a fresh HOME:
    // with the flag the pane holds on that modal; without it the pane lands on
    // the input box reading "Not logged in · Run /login", which is precisely the
    // state `/login` needs. The folder-trust gate is unchanged by this (it fires
    // on the directory, not the mode) and is still dismissed below.
    let cmd = "claude";
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
                pane_tail(&pane)
            ));
        }
    }

    if url.is_empty() {
        let _ = mgr.kill_session(REAUTH_SESSION).await;
        PendingReauth::clear().context("clearing reauth authority after URL timeout")?;
        // Say what actually blocked. "could not extract OAuth URL" describes our
        // parser, not the operator's problem: when a modal is still holding the
        // pane, `/login` was never interpreted at all, and naming that dialog is
        // the difference between an actionable report and a dead end.
        let cause = match detect_blocking_modal(&pane) {
            Some(modal) => format!(
                "the pane is still held by the {modal}, so /login was never \
                 interpreted — accept that dialog once in an interactive \
                 `claude` session, then retry"
            ),
            None => "no known modal was on screen and /login painted no \
                     authorize URL"
                .to_string(),
        };
        return Err(anyhow::anyhow!(
            "could not extract OAuth URL from /login output (15s timeout): {}. Pane tail: {}",
            cause,
            pane_tail(&pane)
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
    if success {
        if let Err(e) = sync_credentials_to_omega() {
            tracing::warn!(error = %e, "failed to sync credentials to omega store");
        } else {
            tracing::info!("synced fresh credentials to ~/.omega/credentials/claude.json");
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

/// Name the modal dialog currently holding the pane, if any.
///
/// A modal is reported only when EVERY signature `BLOCKING_MODALS` lists for it
/// is on screen, which is what keeps ordinary prose that happens to mention
/// permissions or bypassing from reading as a dialog.
pub fn detect_blocking_modal(pane_text: &str) -> Option<&'static str> {
    let lower = pane_text.to_lowercase();
    BLOCKING_MODALS
        .iter()
        .find(|(_, signatures)| {
            signatures
                .iter()
                .all(|sig| lower.contains(&sig.to_lowercase()))
        })
        .map(|(name, _)| *name)
}

/// The last 300 chars of a pane capture, IN READING ORDER.
///
/// `chars().rev().take(n).collect()` yields the tail BACKWARDS — it reverses to
/// reach the end and never reverses back, so the operator is handed a mirrored
/// wall of text at the exact moment they most need to read it. The second `rev`
/// is the whole point; `handle_code` already carries this pattern.
fn pane_tail(pane_text: &str) -> String {
    pane_text
        .chars()
        .rev()
        .take(300)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
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

    /// Captured verbatim from `claude 2.1.232` launched with
    /// `--dangerously-skip-permissions` against a HOME that had never accepted
    /// the dialog. Note the pre-selected option: `❯ 1. No, exit`.
    const BYPASS_MODAL_PANE: &str = "\
────────────────────────────────────────────────────────────────────────────────
  WARNING: Claude Code running in Bypass Permissions mode

  In Bypass Permissions mode, Claude Code will not ask for your approval
  before running potentially dangerous commands.
  This mode should only be used in a sandboxed container/VM that has
  restricted internet access and can easily be restored if damaged.

  By proceeding, you accept all responsibility for actions taken while running
  in Bypass Permissions mode.

  https://code.claude.com/docs/en/security

  ❯ 1. No, exit
    2. Yes, I accept

  Enter to confirm · Esc to cancel
";

    /// Captured verbatim from the SAME binary launched in an untrusted folder.
    /// It is the dialog this flow already dismisses, and the one the Bypass
    /// detector must never be confused by: both are modals, both offer
    /// "No, exit", both link the security guide.
    const TRUST_MODAL_PANE: &str = "\
────────────────────────────────────────────────────────────────────────────────
 Accessing workspace:

 /home/vibe

 Quick safety check: Is this a project you created or one you trust? (Like your
 own code, a well-known open source project, or work from your team). If not,
 take a moment to review what's in this folder first.

 Claude Code'll be able to read, edit, and execute files here.

 Security guide

 ❯ 1. Yes, I trust this folder
   2. No, exit

 Enter to confirm · Esc to cancel
";

    /// The modal that used to end this flow in silence. It is raised whenever
    /// the session runs in bypassPermissions mode without the dialog having been
    /// accepted, its default answer is "No, exit", and the single Enter this
    /// flow sends therefore ANSWERED NO — so `/login` was never interpreted and
    /// the operator got a 15s timeout instead of a link.
    #[test]
    fn detect_blocking_modal_names_the_bypass_dialog() {
        let name = detect_blocking_modal(BYPASS_MODAL_PANE)
            .expect("the real Bypass Permissions pane must be recognised");
        assert!(name.contains("Bypass Permissions"), "got {name:?}");
    }

    /// Two modals, one flow. Telling the operator to accept a folder-trust
    /// prompt when the pane is actually held by the Bypass dialog sends them
    /// after the wrong dialog, so the detector must discriminate, not merely
    /// fire.
    #[test]
    fn detect_blocking_modal_does_not_confuse_trust_with_bypass() {
        let name = detect_blocking_modal(TRUST_MODAL_PANE)
            .expect("the real folder-trust pane must be recognised");
        assert!(
            name.contains("folder-trust"),
            "the trust dialog was reported as {name:?}"
        );
        assert!(
            !name.contains("Bypass"),
            "the trust dialog was mistaken for the Bypass modal: {name:?}"
        );
    }

    /// A detector that fires on prose is worse than no detector: it would name a
    /// modal that is not there and send the operator to accept a dialog nobody
    /// is showing. Each fixture carries the words but not the dialog.
    #[test]
    fn detect_blocking_modal_ignores_ordinary_prose() {
        for prose in [
            "We had to bypass permissions on the socket before the daemon started.",
            "Added a --dangerously-skip-permissions flag that will bypass permissions prompts.",
            "Yes, I accept that the retry budget is spent.",
            // The modal's own body, caught mid-paint before its options exist:
            // a subject with no accept option is not yet a dialog to answer.
            "In Bypass Permissions mode, Claude Code will not ask for your approval \
             before running potentially dangerous commands.",
            "Not logged in · Run /login",
        ] {
            assert_eq!(
                detect_blocking_modal(prose),
                None,
                "prose read as a modal: {prose:?}"
            );
        }
    }

    /// Both markers are failures the flow could previously only discover by
    /// timing out: the pane already said the login was refused, and the poll
    /// loop kept asking for 15s anyway.
    #[test]
    fn detect_auth_failure_catches_oauth_error_and_invalid_code() {
        assert_eq!(
            detect_auth_failure("OAuth error: access_denied"),
            Some("OAuth error")
        );
        assert_eq!(
            detect_auth_failure("Invalid code. Please try again."),
            Some("Invalid code")
        );
        assert!(AUTH_FAILURE_MARKERS.contains(&"OAuth error"));
        assert!(AUTH_FAILURE_MARKERS.contains(&"Invalid code"));
    }

    /// The tail is what the operator actually reads when the flow fails, and it
    /// was being handed to them REVERSED: `chars().rev().take(n)` walks back
    /// from the end and never turns around again.
    #[test]
    fn pane_tail_reads_forwards_not_mirrored() {
        let pane = format!("{}\nPaste code here:", "x".repeat(400));
        let tail = pane_tail(&pane);
        assert!(tail.ends_with("Paste code here:"), "got {tail:?}");
        assert!(
            !tail.starts_with(':'),
            "the tail came back mirrored: {tail:?}"
        );
        assert_eq!(tail.chars().count(), 300);
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
