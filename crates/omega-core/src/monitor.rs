//! Monitor — read-only view into Claude Code billing, accounts, and AISB bot
//! activity. This module NEVER modifies the AISB system; it only reads cache
//! files and credential files that the AISB system already maintains.
//!
//! Files we read (read-only):
//! - `/tmp/aisb-usage.json`            — live billing percentages (updated by
//!                                       `~/.aisb/lib/usage-monitor.sh` cron)
//! - `~/.claude/.credentials.json`     — current OAuth credentials
//! - `~/.claude/accounts/*.json`       — saved account profiles
//! - `~/.omega/telegram.toml`          — Omega's own Telegram bot config
//!
//! Files we WRITE (Omega-owned only):
//! - `~/.omega/telegram.toml`          — when user sets up Omega Telegram bot
//! - `~/.omega/monitor.last.json`      — cache of last-seen snapshot for diffs

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const USAGE_PATH: &str = "/tmp/aisb-usage.json";

/// Live billing snapshot read from the AISB usage cache.
/// Values mirror what the AISB bot's `/billing` command shows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSnapshot {
    #[serde(default)]
    pub session_pct: u32,
    #[serde(default)]
    pub week_pct: u32,
    #[serde(default)]
    pub sonnet_pct: u32,
    #[serde(default)]
    pub extra_pct: u32,
    #[serde(default)]
    pub pct_5h_precise: f32,
    #[serde(default)]
    pub pct_week_precise: f32,
    #[serde(default)]
    pub tokens_5h: u64,
    #[serde(default)]
    pub tokens_7d: u64,
    #[serde(default)]
    pub sessions_5h: u32,
    #[serde(default)]
    pub sessions_7d: u32,
    #[serde(default)]
    pub budget_5h: u64,
    #[serde(default)]
    pub budget_week: u64,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub active_account: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub source: String,
    /// ISO-8601 timestamp from the cache file
    #[serde(default)]
    pub ts: String,
}

impl UsageSnapshot {
    /// Read /tmp/aisb-usage.json — returns Ok(None) if file missing.
    pub fn read() -> Result<Option<Self>> {
        let path = std::path::Path::new(USAGE_PATH);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", USAGE_PATH))?;
        let snap: Self = serde_json::from_str(&content)
            .with_context(|| format!("parsing {}", USAGE_PATH))?;
        Ok(Some(snap))
    }

    /// Age of the cache file in seconds (None if file missing).
    pub fn cache_age_secs() -> Option<u64> {
        let meta = std::fs::metadata(USAGE_PATH).ok()?;
        let modified = meta.modified().ok()?;
        modified.elapsed().ok().map(|d| d.as_secs())
    }

    pub fn parsed_ts(&self) -> Option<DateTime<Utc>> {
        chrono::DateTime::parse_from_rfc3339(&self.ts)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    pub fn precise_5h(&self) -> f32 {
        if self.pct_5h_precise > 0.0 {
            self.pct_5h_precise
        } else {
            self.session_pct as f32
        }
    }

    pub fn precise_week(&self) -> f32 {
        if self.pct_week_precise > 0.0 {
            self.pct_week_precise
        } else {
            self.week_pct as f32
        }
    }
}

/// A saved Claude account profile (one per file in ~/.claude/accounts/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub label: String,
    pub email: Option<String>,
    pub is_active: bool,
}

/// The currently connected Claude account: email + subscription plan.
/// Read live from `claude auth status --json`. Returns None if not logged
/// in or the CLI isn't available.
#[derive(Debug, Clone)]
pub struct ConnectedAccount {
    pub email: String,
    /// e.g. "max", "pro", "free", "team".
    pub plan: String,
    pub auth_method: String,
}

pub fn connected_account() -> Option<ConnectedAccount> {
    let out = std::process::Command::new("claude")
        .args(["auth", "status"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    if !json.get("loggedIn").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }
    Some(ConnectedAccount {
        email: json.get("email").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
        plan: json
            .get("subscriptionType")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        auth_method: json
            .get("authMethod")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
    })
}

/// Discover all Claude account profiles + identify the active one.
pub fn list_accounts() -> Vec<AccountProfile> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let creds_path = home.join(".claude/.credentials.json");
    let accounts_dir = home.join(".claude/accounts");

    // Read current refreshToken to identify the active account
    let current_token: Option<String> = std::fs::read_to_string(&creds_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| {
            v.get("claudeAiOauth")
                .and_then(|o| o.get("refreshToken"))
                .and_then(|t| t.as_str().map(String::from))
        });

    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&accounts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let label = name.strip_prefix("account-").unwrap_or(name).to_string();

            let (email, account_token) = std::fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .map(|v| {
                    let oauth = v.get("claudeAiOauth");
                    let email = oauth
                        .and_then(|o| o.get("email"))
                        .and_then(|e| e.as_str())
                        .map(String::from);
                    let token = oauth
                        .and_then(|o| o.get("refreshToken"))
                        .and_then(|t| t.as_str())
                        .map(String::from);
                    (email, token)
                })
                .unwrap_or((None, None));

            let is_active = match (&current_token, &account_token) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            profiles.push(AccountProfile {
                label,
                email,
                is_active,
            });
        }
    }
    profiles.sort_by(|a, b| a.label.cmp(&b.label));
    profiles
}

/// Detect whether the AISB Telegram bot is running on this machine.
/// Looks at /tmp/aisb-usage.json freshness + any process matching aisb*python.
pub fn aisb_bot_status() -> AisbBotStatus {
    let cache_age = UsageSnapshot::cache_age_secs();
    let bot_alive = is_aisb_bot_running();

    let cache_status = match cache_age {
        None => CacheStatus::Missing,
        Some(s) if s < 900 => CacheStatus::Fresh(s),
        Some(s) => CacheStatus::Stale(s),
    };

    AisbBotStatus {
        bot_alive,
        cache_status,
    }
}

fn is_aisb_bot_running() -> bool {
    // Best-effort: pgrep -f "aisb.*python" or similar.
    let Ok(output) = std::process::Command::new("pgrep")
        .args(["-f", "aisb.app|telegram-bot|agentik-monitor"])
        .output()
    else {
        return false;
    };
    output.status.success() && !output.stdout.is_empty()
}

#[derive(Debug, Clone)]
pub struct AisbBotStatus {
    pub bot_alive: bool,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone)]
pub enum CacheStatus {
    /// Cache file exists and is < 15 minutes old
    Fresh(u64),
    /// Cache file exists but is older than 15 minutes
    Stale(u64),
    /// Cache file does not exist
    Missing,
}

/// Trigger a Claude OAuth login by opening `claude /login` in a new pane.
/// Returns the command the user should run if we cannot spawn it ourselves.
pub fn login_command_hint() -> String {
    "claude /login".to_string()
}

/// Omega's own Telegram bot configuration (separate from AISB's).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaTelegramConfig {
    pub bot_token: String,
    /// The ONLY chat_id allowed to talk to the bot. Messages from any other
    /// chat are silently dropped.
    pub chat_id: i64,
    /// Optional allow-list of Telegram user IDs (sender). When empty, only
    /// the chat_id check applies. When non-empty, the message author MUST
    /// match one of these IDs OR the chat_id check is bypassed.
    #[serde(default)]
    pub allow_user_ids: Vec<i64>,
    /// Session to relay messages to (default: aisb-master)
    #[serde(default = "default_relay_session")]
    pub relay_session: String,
    /// Optional human label so the user can identify which profile is active.
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub enabled: bool,
}

fn default_relay_session() -> String {
    crate::aisb::MASTER_SESSION_NAME.to_string()
}

impl OmegaTelegramConfig {
    fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("telegram.toml")
    }

    pub fn read() -> Option<Self> {
        let path = Self::path();
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn write(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn exists() -> bool {
        Self::path().exists()
    }

    /// Remove the active config file. Returns Ok(true) if a file was removed.
    pub fn disconnect() -> Result<bool> {
        let path = Self::path();
        if !path.exists() {
            return Ok(false);
        }
        std::fs::remove_file(&path)
            .with_context(|| format!("removing {}", path.display()))?;
        Ok(true)
    }

    /// True if a Telegram message is allowed to interact with this bot.
    ///
    /// Security model (2-level, either path grants access):
    ///   1. sender_id is in allow_user_ids → authorized (works for DMs and groups)
    ///   2. chat_id matches config AND allow_user_ids is empty → authorized (group-only legacy mode)
    ///
    /// For DMs: chat_id == sender's user_id (not the bot's ID), so the user_id
    /// check is the primary gate. The chat_id field in config is mainly for
    /// restricting to a specific group chat when allow_user_ids is empty.
    pub fn is_authorized(&self, chat_id: i64, sender_id: Option<i64>) -> bool {
        if let Some(uid) = sender_id {
            if !self.allow_user_ids.is_empty() && self.allow_user_ids.contains(&uid) {
                return true;
            }
        }
        if chat_id == self.chat_id && self.allow_user_ids.is_empty() {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_snapshot_handles_missing_file() {
        // /tmp/aisb-usage.json may or may not exist on the test host
        let result = UsageSnapshot::read();
        assert!(result.is_ok());
    }

    #[test]
    fn list_accounts_does_not_panic() {
        let _ = list_accounts();
    }
}
