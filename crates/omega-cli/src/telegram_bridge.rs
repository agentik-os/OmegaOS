//! Minimal Rust Telegram bridge — no Python dependency.
//!
//! Long-polls Telegram getUpdates → relays the user's text into the
//! configured rmux session (default: aisb-master) via the rmux SDK.
//! Periodically captures the pane and posts deltas back to the chat.
//!
//! This is a SEPARATE bot from the existing AISB Python bot — it lives
//! in OmegaOS and talks to the Master AISB session over the same SDK we
//! use everywhere else.

use anyhow::{Context, Result};
use omega_core::monitor::OmegaTelegramConfig;
use omega_core::session::SessionManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_BASE: &str = "https://api.telegram.org";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetUpdatesResp {
    ok: bool,
    #[serde(default)]
    result: Vec<Update>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Update {
    update_id: i64,
    #[serde(default)]
    message: Option<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    message_id: i64,
    #[serde(default)]
    text: Option<String>,
    chat: Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageReq<'a> {
    chat_id: i64,
    text: &'a str,
}

pub async fn run(cfg: OmegaTelegramConfig) -> Result<()> {
    println!("◆ Omega Telegram bridge starting");
    println!("  Relay session: {}", cfg.relay_session);
    println!("  Chat ID:       {}", cfg.chat_id);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let mgr = SessionManager::connect().await?;

    // Ensure the relay session exists (boot AISB Master if it's the target)
    if cfg.relay_session == omega_core::aisb::MASTER_SESSION_NAME {
        let agent = omega_core::agents::Agent::Claude;
        let cwd = std::env::current_dir()?
            .to_str()
            .unwrap_or("/home")
            .to_string();
        let _ = omega_core::aisb::ensure_master(&mgr, agent, &cwd).await;
    }

    let _ = send_telegram(
        &client,
        &cfg,
        "◆ Omega Telegram bridge online. Type a message and I'll relay it to AISB Master.",
    )
    .await;

    let mut offset: i64 = 0;
    let mut last_capture = String::new();

    loop {
        // Long-poll updates
        let url = format!(
            "{}/bot{}/getUpdates?timeout=25&offset={}",
            API_BASE, cfg.bot_token, offset
        );
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "getUpdates request failed");
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        let updates: GetUpdatesResp = match resp.json().await {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!(error = %e, "getUpdates parse failed");
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        if !updates.ok {
            tracing::warn!("getUpdates returned ok=false");
            tokio::time::sleep(Duration::from_secs(3)).await;
            continue;
        }

        for upd in updates.result {
            offset = upd.update_id + 1;
            let Some(msg) = upd.message else { continue };
            let Some(text) = msg.text.as_deref() else { continue };

            // Only accept messages from the configured chat
            if msg.chat.id != cfg.chat_id {
                continue;
            }

            tracing::info!(text = %text, "Received Telegram message");

            // Handle Omega-specific commands
            if text.starts_with('/') {
                if let Some(reply) = handle_command(text, &mgr, &cfg).await {
                    let _ = send_telegram(&client, &cfg, &reply).await;
                    continue;
                }
            }

            // Default: relay to the configured session
            if let Err(e) = mgr.send_text(&cfg.relay_session, text).await {
                let _ = send_telegram(
                    &client,
                    &cfg,
                    &format!("✗ Could not relay to {}: {}", cfg.relay_session, e),
                )
                .await;
                continue;
            }

            // Capture the response after a small delay
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Ok(content) = mgr.capture_pane(&cfg.relay_session).await {
                let delta = compute_delta(&last_capture, &content);
                last_capture = content;
                if !delta.is_empty() {
                    let trimmed = if delta.len() > 3900 {
                        format!("…{}", &delta[delta.len() - 3900..])
                    } else {
                        delta
                    };
                    let _ = send_telegram(&client, &cfg, &trimmed).await;
                }
            }
        }
    }
}

fn compute_delta(prev: &str, current: &str) -> String {
    if prev.is_empty() {
        return current.lines().rev().take(15).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
    }
    // Naive delta: take lines that appear in current but not at end of prev
    let prev_lines: Vec<&str> = prev.lines().collect();
    let cur_lines: Vec<&str> = current.lines().collect();
    let common_tail_len = prev_lines
        .iter()
        .rev()
        .zip(cur_lines.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let new_tail_start = cur_lines.len().saturating_sub(common_tail_len);
    if new_tail_start >= cur_lines.len() {
        return String::new();
    }
    cur_lines[new_tail_start..]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

async fn handle_command(
    text: &str,
    mgr: &SessionManager,
    cfg: &OmegaTelegramConfig,
) -> Option<String> {
    let mut parts = text.splitn(2, ' ');
    let cmd = parts.next()?;
    let rest = parts.next().unwrap_or("");
    match cmd {
        "/start" | "/help" => Some(
            "Omega Telegram bridge commands:\n\
             /start /help — this message\n\
             /list — show all rmux sessions\n\
             /status <session> — capture last 20 lines\n\
             /billing — current Claude usage\n\
             /aisb <text> — send to AISB Master (default if no command)\n\
             /relay <session> <text> — send to a specific session"
                .to_string(),
        ),
        "/list" => {
            let sessions = mgr.list_sessions().await.ok()?;
            let mut s = String::from("Sessions:\n");
            for sess in sessions {
                s.push_str(&format!("  • {:?} {}\n", sess.role, sess.name));
            }
            Some(s)
        }
        "/billing" => {
            let snap = omega_core::monitor::UsageSnapshot::read()
                .ok()
                .flatten()?;
            Some(format!(
                "Billing:\n  5h:    {:.1}%\n  Week:  {:.1}%\n  Account: {} ({})",
                snap.precise_5h(),
                snap.precise_week(),
                snap.active_account,
                snap.email
            ))
        }
        "/status" => {
            let session = if rest.is_empty() { &cfg.relay_session } else { rest };
            let content = mgr.capture_pane(session).await.ok()?;
            let tail: Vec<&str> = content.lines().rev().take(20).collect();
            Some(tail.into_iter().rev().collect::<Vec<_>>().join("\n"))
        }
        "/aisb" => {
            let _ = mgr.send_text(&cfg.relay_session, rest).await;
            Some(format!("→ {}", cfg.relay_session))
        }
        "/relay" => {
            let mut rp = rest.splitn(2, ' ');
            let session = rp.next()?;
            let payload = rp.next().unwrap_or("");
            let _ = mgr.send_text(session, payload).await;
            Some(format!("→ {}", session))
        }
        _ => None,
    }
}

async fn send_telegram(
    client: &reqwest::Client,
    cfg: &OmegaTelegramConfig,
    text: &str,
) -> Result<()> {
    let url = format!("{}/bot{}/sendMessage", API_BASE, cfg.bot_token);
    let body = SendMessageReq {
        chat_id: cfg.chat_id,
        text,
    };
    client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("sendMessage")?;
    Ok(())
}
