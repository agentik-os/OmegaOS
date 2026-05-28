//! OmegaUsage — Claude token-budget monitor + Telegram threshold alerts.
//!
//! Reinterpreted from the legacy AISB `usage-monitor.sh` (bash) into a
//! native OmegaOS Rust subcommand. Same intent, OmegaOS-aligned plumbing:
//!
//! - Reads the OAuth usage endpoint (`/api/oauth/usage`) for the active
//!   account's 5-hour + 7-day utilization. This is the authoritative
//!   source when Anthropic serves it.
//! - The alert metric is `max(session_5h%, week_7d%)` — whichever cap is
//!   closest to being hit, exactly like the old monitor.
//! - At 80% → yellow proactive alert. At 90% → red alert. Each threshold
//!   has its own 30-minute cooldown file under `~/.omega/state/` so we
//!   don't spam.
//! - Alerts go out through the SAME Telegram bot the bridge uses
//!   (`~/.omega/telegram.toml`), formatted with a usage bar.
//!
//! Scheduled by cron (`omega usage --check`, every 10 min) — set up by
//! install.sh so a fresh clone gets the alert automatically.
//!
//! NOTE: account auto-switching (the old 90% shadow-rotate) is NOT ported
//! here — OmegaOS handles multi-account differently. This module is
//! strictly the ALERT half the user asked for.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const COOLDOWN_SECS: u64 = 1800; // 30 minutes per threshold

/// Utilization snapshot for the active account.
#[derive(Debug, Clone, Default)]
pub struct UsageSnapshot {
    pub session_pct: u32, // 5-hour window
    pub week_pct: u32,    // 7-day window
}

impl UsageSnapshot {
    /// The metric the alert fires on — closest cap to being hit.
    pub fn alert_pct(&self) -> u32 {
        self.session_pct.max(self.week_pct)
    }
}

fn state_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".omega/state")
}

/// Read the OAuth access token from Claude's credentials file.
fn oauth_token() -> Option<String> {
    let home = dirs::home_dir()?;
    let cred = home.join(".claude/.credentials.json");
    let content = std::fs::read_to_string(&cred).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    // Token may be at .claudeAiOauth.accessToken or .accessToken
    json.get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .or_else(|| json.get("accessToken"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Fetch utilization from the OAuth endpoint. Returns None if the API is
/// unavailable (429, network error, etc.) — caller decides whether to
/// stay silent.
pub async fn fetch_usage(client: &reqwest::Client) -> Option<UsageSnapshot> {
    let token = oauth_token()?;
    let resp = client
        .get(OAUTH_USAGE_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;

    // five_hour / seven_day may be an object {utilization} or a bare number.
    let extract = |key: &str| -> u32 {
        match json.get(key) {
            Some(serde_json::Value::Object(o)) => o
                .get("utilization")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as u32,
            Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0) as u32,
            _ => 0,
        }
    };

    Some(UsageSnapshot {
        session_pct: extract("five_hour"),
        week_pct: extract("seven_day"),
    })
}

/// Read the Telegram bot creds from ~/.omega/telegram.toml.
fn telegram_creds() -> Option<(String, i64)> {
    let home = dirs::home_dir()?;
    let path = home.join(".omega/telegram.toml");
    let content = std::fs::read_to_string(&path).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;
    let token = parsed.get("bot_token")?.as_str()?.to_string();
    // Alert goes to the first allow_user_id (the owner's DM), falling back
    // to chat_id.
    let chat_id = parsed
        .get("allow_user_ids")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_integer())
        .or_else(|| parsed.get("chat_id").and_then(|v| v.as_integer()))?;
    Some((token, chat_id))
}

fn cooldown_elapsed(flag: &str) -> u64 {
    let path = state_dir().join(flag);
    let Ok(meta) = std::fs::metadata(&path) else {
        return u64::MAX;
    };
    let Ok(modified) = meta.modified() else {
        return u64::MAX;
    };
    let modified_secs = modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now.saturating_sub(modified_secs)
}

fn touch_cooldown(flag: &str) {
    let dir = state_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join(flag), "");
}

fn usage_bar(pct: u32) -> String {
    let filled = (pct / 5).min(20) as usize;
    let empty = 20usize.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

async fn send_alert(client: &reqwest::Client, snap: &UsageSnapshot, red: bool) -> Result<()> {
    let (token, chat_id) = telegram_creds().context("no telegram creds")?;
    let pct = snap.alert_pct();
    let icon = if red { "🔴" } else { "🟡" };
    let level = if red { "USAGE 90%" } else { "USAGE 80%" };
    let bar = usage_bar(pct);
    let text = format!(
        "{icon} <b>{level}</b> · <code>{pct}%</code>\n\
         ────────────────────\n\
         <code>{bar}</code>\n\n\
         <b>5h:</b> {sess}%   <b>Week:</b> {week}%\n\n\
         <i>Approche de la limite. /clean ou switch de compte si besoin.</i>",
        icon = icon,
        level = level,
        pct = pct,
        bar = bar,
        sess = snap.session_pct,
        week = snap.week_pct,
    );
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
    });
    let _ = client.post(&url).json(&body).send().await;
    Ok(())
}

/// One monitor tick: fetch usage, evaluate thresholds, alert if crossed
/// (respecting cooldowns). Returns the snapshot for logging.
pub async fn check_and_alert() -> Result<Option<UsageSnapshot>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let Some(snap) = fetch_usage(&client).await else {
        // API unavailable — stay silent (the old monitor fell back to a
        // local estimator; that's a separate follow-up if Anthropic's
        // endpoint stays broken).
        return Ok(None);
    };

    let pct = snap.alert_pct();

    // 90% red (own cooldown)
    if pct >= 90 && cooldown_elapsed("usage-alert-90-sent") >= COOLDOWN_SECS {
        touch_cooldown("usage-alert-90-sent");
        send_alert(&client, &snap, true).await?;
    }
    // 80% yellow (own cooldown, only when not already in 90 band)
    else if pct >= 80 && pct < 90 && cooldown_elapsed("usage-alert-80-sent") >= COOLDOWN_SECS {
        touch_cooldown("usage-alert-80-sent");
        send_alert(&client, &snap, false).await?;
    }

    // Persist latest snapshot for `omega usage` (no --check) display.
    let dir = state_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(
        dir.join("usage.json"),
        serde_json::json!({
            "session_pct": snap.session_pct,
            "week_pct": snap.week_pct,
            "alert_pct": pct,
            "ts": chrono::Utc::now().to_rfc3339(),
        })
        .to_string(),
    );

    Ok(Some(snap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_pct_is_max() {
        let s = UsageSnapshot { session_pct: 72, week_pct: 81 };
        assert_eq!(s.alert_pct(), 81);
    }

    #[test]
    fn usage_bar_fills() {
        assert_eq!(usage_bar(50).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(usage_bar(100).chars().filter(|c| *c == '█').count(), 20);
        assert_eq!(usage_bar(0).chars().filter(|c| *c == '█').count(), 0);
    }

    #[test]
    fn usage_bar_clamps_over_100() {
        assert_eq!(usage_bar(150).chars().count(), 20);
    }
}
