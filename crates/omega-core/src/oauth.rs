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
//! State is tracked in-memory AND persisted to `/tmp/omega-pending-reauth.json`
//! so a bridge restart doesn't lose the pending flag.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::session::SessionManager;

pub const REAUTH_SESSION: &str = "aisb-reauth";
pub const PENDING_STATE_PATH: &str = "/tmp/omega-pending-reauth.json";
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

/// Module-level cooldown timestamp (epoch seconds of last reauth attempt).
/// Prevents double-tap re-trigger when the user retries quickly.
static REAUTH_COOLDOWN_TS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub fn load() -> Self {
        std::fs::read_to_string(PENDING_STATE_PATH)
            .ok()
            .and_then(|s| serde_json::from_str::<Self>(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(PENDING_STATE_PATH, json)
            .with_context(|| format!("writing {}", PENDING_STATE_PATH))?;
        Ok(())
    }

    pub fn clear() {
        let cleared = Self {
            pending: false,
            ts: 0.0,
            target_account: String::new(),
            reason: String::new(),
        };
        let _ = cleared.save();
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
) -> Result<Option<ReauthRequest>> {
    let now = now_epoch_secs();

    // Cooldown — 30s between attempts.
    let last = REAUTH_COOLDOWN_TS.load(Ordering::Relaxed);
    if last > 0 && now.saturating_sub(last) < COOLDOWN_SEC {
        tracing::debug!("reauth skipped: cooldown active ({}s ago)", now - last);
        return Ok(None);
    }

    // Stale check — clear if pending > TTL.
    let mut state = PendingReauth::load();
    if state.is_stale() {
        tracing::warn!("reauth: stale pending flag — auto-clearing");
        state.pending = false;
    }
    if state.pending {
        tracing::debug!("reauth skipped: already pending");
        return Ok(None);
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
    new_state.save().ok();

    // Spawn the reauth session.
    let cmd = "claude --dangerously-skip-permissions";
    let cwd = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    if let Err(e) = mgr
        .create_session(REAUTH_SESSION, Some(&cwd), Some(cmd))
        .await
    {
        PendingReauth::clear();
        return Err(anyhow::anyhow!("failed to spawn reauth session: {}", e));
    }

    // Wait for claude to boot.
    tokio::time::sleep(Duration::from_secs(8)).await;

    // Send /login.
    if let Err(e) = mgr.send_text(REAUTH_SESSION, "/login").await {
        let _ = mgr.kill_session(REAUTH_SESSION).await;
        PendingReauth::clear();
        return Err(anyhow::anyhow!("send_text /login failed: {}", e));
    }
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Confirm option 1 (Claude account with subscription) — just press Enter.
    if let Err(e) = mgr.send_text(REAUTH_SESSION, "").await {
        tracing::warn!("reauth: confirm Enter failed: {}", e);
    }
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Capture pane and extract URL.
    let pane = mgr.capture_pane(REAUTH_SESSION).await.unwrap_or_default();
    let url = extract_auth_url(&pane);

    if url.is_empty() {
        let _ = mgr.kill_session(REAUTH_SESSION).await;
        PendingReauth::clear();
        return Err(anyhow::anyhow!(
            "could not extract OAuth URL from /login output. Pane tail: {}",
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
    let waiting_for_code = pre_capture.contains("Paste code here")
        || pre_capture.contains("Paste your code");
    if !waiting_for_code {
        tracing::warn!(
            "Claude does not appear to be waiting for code. Last 300 chars: {}",
            &pre_capture.chars().rev().take(300).collect::<String>().chars().rev().collect::<String>()
        );
    }

    // CRITICAL: paste code WITHOUT Enter, sleep 1s, then send Enter separately.
    // VPS Python pattern: load-buffer + paste-buffer + sleep 1 + send-keys Enter.
    // Without this gap, Claude /login input field rejects the paste.
    let pane = mgr.get_active_pane(REAUTH_SESSION)
        .await
        .context("get reauth pane failed")?;
    pane.send_text(code)
        .await
        .context("paste code failed")?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    pane.send_key("Enter")
        .await
        .context("send Enter failed")?;

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
    let pane_tail = mgr
        .capture_pane(REAUTH_SESSION)
        .await
        .unwrap_or_default();
    let pane_tail: String = pane_tail
        .lines()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");

    // Success criteria:
    // - mtime bumped + we have a token (preferred path)
    // - OR pane explicitly confirms "Logged in as" (Claude already had valid token)
    // - OR a token now exists at the native path and differs from before
    let success = (updated && !after_token.is_empty())
        || pane_tail.contains("Logged in as")
        || (!after_token.is_empty() && after_token != before_token);

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
    PendingReauth::clear();
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
    for marker in AUTH_FAILURE_MARKERS {
        if lower.contains(&marker.to_lowercase()) {
            return Some(marker);
        }
    }
    None
}

/// Extract the OAuth URL from the captured pane.
///
/// Strategy:
///   1. Scan lines for one starting with `https://claude.com/cai/oauth/authorize`.
///   2. Continue concatenating subsequent non-empty lines that look like a URL
///      continuation (no whitespace, no prompt markers).
///   3. Trim trailing punctuation/whitespace.
pub fn extract_auth_url(pane_text: &str) -> String {
    let mut parts = Vec::new();
    let mut in_url = false;

    for line in pane_text.lines() {
        let stripped = line.trim();
        if !in_url {
            if stripped.contains("https://claude.com/cai/oauth/authorize") {
                in_url = true;
                if let Some(idx) = stripped.find("https://claude.com/cai/oauth/authorize") {
                    parts.push(stripped[idx..].to_string());
                }
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
        if !started {
            if candidate.starts_with("https://claude.com/cai/oauth/authorize") {
                started = true;
            }
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
    let canonical = home.join(".omega").join("credentials").join("claude.json");
    if canonical.exists() {
        return canonical;
    }
    let legacy = home.join(".claude").join(".credentials.json");
    if legacy.exists() && !canonical.exists() {
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
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let canonical = home.join(".omega").join("credentials").join("claude.json");

    // If native is a real file (Claude's atomic write broke the symlink),
    // copy it into omega and re-link.
    let meta = std::fs::symlink_metadata(&native)?;
    if meta.file_type().is_symlink() {
        // Still a symlink — nothing to do, both already point at omega.
        return Ok(());
    }

    // Real file at native path → copy to omega canonical, then re-symlink.
    if let Some(parent) = canonical.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&native, &canonical)?;
    let _ = std::fs::set_permissions(
        &canonical,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    );
    // Replace native with a symlink back to omega.
    std::fs::remove_file(&native)?;
    std::os::unix::fs::symlink(&canonical, &native)?;
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
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    let oauth = v
        .get("claudeAiOauth")
        .ok_or_else(|| anyhow::anyhow!("missing claudeAiOauth"))?;
    Ok(CredentialsInfo {
        email: oauth
            .get("email")
            .and_then(|e| e.as_str())
            .map(String::from),
        refresh_token: oauth
            .get("refreshToken")
            .and_then(|e| e.as_str())
            .map(String::from),
        access_token: oauth
            .get("accessToken")
            .and_then(|e| e.as_str())
            .map(String::from),
        expires_at_ms: oauth
            .get("expiresAt")
            .and_then(|e| e.as_i64())
            .unwrap_or(0),
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
        assert!(looks_like_oauth_code("abc123_DEF-456_xyz789012345#state_value"));
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
            https://claude.com/cai/oauth/authorize?code=true&client_id=abc&state=xyz\n\
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

    #[test]
    fn detect_auth_failure_catches_401() {
        assert_eq!(detect_auth_failure("Got 401 Unauthorized"), Some("401"));
        assert_eq!(
            detect_auth_failure("rate_limit_error: too many"),
            Some("rate_limit_error")
        );
        assert!(detect_auth_failure("everything is fine").is_none());
    }

    #[test]
    fn pending_reauth_stale_detection() {
        let mut p = PendingReauth::default();
        p.pending = true;
        p.ts = (now_epoch_secs() - PENDING_TTL_SEC - 10) as f64;
        assert!(p.is_stale());

        p.ts = now_epoch_secs() as f64;
        assert!(!p.is_stale());
    }
}
