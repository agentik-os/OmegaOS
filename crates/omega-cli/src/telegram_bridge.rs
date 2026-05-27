//! Minimal Rust Telegram bridge — no Python dependency.
//!
//! Long-polls Telegram getUpdates → relays the user's text into the
//! configured rmux session (default: aisb-master) via the rmux SDK.
//! Periodically captures the pane and posts deltas back to the chat.
//!
//! Message style mirrors the AISB Python bot: HTML parse_mode,
//! blockquotes for AISB voice, <code> for terminal output, emojis for
//! status.

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
    #[serde(default)]
    from: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i64,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageReq<'a> {
    chat_id: i64,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
}

pub async fn run(cfg: OmegaTelegramConfig) -> Result<()> {
    println!("◆ Omega Telegram bridge starting");
    println!("  Relay session: {}", cfg.relay_session);
    println!("  Chat ID:       {}", cfg.chat_id);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let mgr = SessionManager::connect().await?;

    if cfg.relay_session == omega_core::aisb::MASTER_SESSION_NAME {
        let agent = omega_core::agents::Agent::Claude;
        let cwd = std::env::current_dir()?
            .to_str()
            .unwrap_or("/home")
            .to_string();
        let _ = omega_core::aisb::ensure_master(&mgr, agent, &cwd).await;
    }

    // For DMs, chat_id == user_id. Config may have the bot's own ID.
    let mut reply_chat_id = if !cfg.allow_user_ids.is_empty() {
        cfg.allow_user_ids[0]
    } else {
        cfg.chat_id
    };

    let _ = send_html(
        &client,
        &cfg.bot_token,
        reply_chat_id,
        "🟢 <b>Ω OmegaOS Bridge</b> — online\n\n\
         <i>Messages are relayed to AISB Master.\n\
         Type normally to chat, or use /help for commands.</i>",
    )
    .await;

    let mut offset: i64 = 0;
    let mut last_capture = String::new();

    loop {
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

            let sender_id = msg.from.as_ref().map(|u| u.id);
            if !cfg.is_authorized(msg.chat.id, sender_id) {
                tracing::warn!(
                    chat_id = msg.chat.id,
                    sender_id = ?sender_id,
                    sender_username = ?msg.from.as_ref().and_then(|u| u.username.as_deref()),
                    "Rejected unauthorized Telegram message"
                );
                continue;
            }

            tracing::info!(
                text = %text,
                chat_id = msg.chat.id,
                sender_id = ?sender_id,
                "Received Telegram message"
            );

            reply_chat_id = msg.chat.id;

            // Handle Omega-specific commands
            if text.starts_with('/') {
                if let Some(reply) = handle_command(text, &mgr, &cfg).await {
                    let _ = send_html(&client, &cfg.bot_token, reply_chat_id, &reply).await;
                    continue;
                }
            }

            // Relay to the configured session
            if let Err(e) = mgr.send_text(&cfg.relay_session, text).await {
                let _ = send_html(
                    &client,
                    &cfg.bot_token,
                    reply_chat_id,
                    &format!(
                        "🔴 <b>Relay failed</b>\n<code>{}</code>",
                        escape_html(&e.to_string())
                    ),
                )
                .await;
                continue;
            }

            // Send a quick "thinking" indicator
            let _ = send_html(
                &client,
                &cfg.bot_token,
                reply_chat_id,
                &format!("⏳ → <code>{}</code>", escape_html(&cfg.relay_session)),
            )
            .await;

            // Wait for the agent to process, then capture the response
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Poll up to 30s for a response delta
            let mut attempts = 0;
            loop {
                if let Ok(content) = mgr.capture_pane(&cfg.relay_session).await {
                    let delta = compute_delta(&last_capture, &content);
                    last_capture = content;
                    if !delta.is_empty() {
                        let cleaned = clean_terminal_output(&delta);
                        if !cleaned.is_empty() {
                            let formatted = format_agent_response(&cleaned);
                            let _ =
                                send_html(&client, &cfg.bot_token, reply_chat_id, &formatted)
                                    .await;
                        }
                        break;
                    }
                }
                attempts += 1;
                if attempts >= 6 {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Remove ANSI escape codes and terminal artifacts from pane capture.
fn clean_terminal_output(text: &str) -> String {
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07").unwrap_or_else(|_| {
        regex::Regex::new(r"$^").unwrap()
    });
    let stripped = ansi_re.replace_all(text, "");
    // Strip entire <system-reminder>...</system-reminder> blocks
    let no_reminders = regex::Regex::new(r"(?s)<system-reminder>.*?</system-reminder>")
        .map(|re| re.replace_all(&stripped, "").to_string())
        .unwrap_or_else(|_| stripped.to_string());
    let lines: Vec<&str> = no_reminders
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| {
            if l.is_empty() { return false; }
            let t = l.trim();
            // Claude prompt / status chrome
            if t.starts_with("❯") { return false; }
            if t.starts_with("⎿") { return false; }
            if t.starts_with("●") { return false; }
            if t.starts_with("·") { return false; }
            if t.starts_with("✻") { return false; }
            // Claude activity indicators
            if t.contains("Cultivating") { return false; }
            if t.contains("Brewing") || t.contains("Brewed") { return false; }
            if t.contains("Crunched") || t.contains("Crunching") { return false; }
            if t.contains("Pontificating") { return false; }
            if t.contains("Thinking") && t.len() < 30 { return false; }
            if t.contains("skills available") { return false; }
            // Terminal chrome
            if t.contains("bypass permissions") { return false; }
            if t.contains("shift+tab to cycle") { return false; }
            if t.contains("← for agents") { return false; }
            if t.contains("esc to interrupt") { return false; }
            if t.contains("Press up to edit") { return false; }
            if t.chars().all(|c| c == '─' || c == '━' || c == ' ') { return false; }
            // Claude-mem / system-reminder / observation metadata — internal agent context
            if t.contains("system-reminder") { return false; }
            if t.contains("claude-mem") { return false; }
            if t.contains("observation") && t.contains("token") { return false; }
            if t.contains("get_observations") { return false; }
            if t.contains("mem-search") { return false; }
            if t.contains("smart_outline") { return false; }
            if t.contains("memory_search") { return false; }
            if t.contains("observation_") { return false; }
            if t.starts_with("S1") && t.contains("AISB") { return false; }
            if t.starts_with("#") && t.contains("obs") { return false; }
            if t.contains("savings") && t.contains("tokens") { return false; }
            if t.contains("Need details on a past") { return false; }
            if t.contains("supplementary context") { return false; }
            if t.contains("prior observations") { return false; }
            true
        })
        .collect();
    lines.join("\n").trim().to_string()
}

/// Format the agent's response for Telegram with HTML.
/// No prefix — the user knows they're talking to the agent.
fn format_agent_response(text: &str) -> String {
    let escaped = escape_html(text);
    let code_lines = escaped
        .lines()
        .filter(|l| l.starts_with("  ") || l.starts_with("\t") || l.contains("fn ") || l.contains("{}"))
        .count();
    let total_lines = escaped.lines().count().max(1);

    if code_lines as f32 / total_lines as f32 > 0.5 {
        format!("<pre>{}</pre>", escaped)
    } else {
        escaped
    }
}

/// Escape HTML special chars for Telegram.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn compute_delta(prev: &str, current: &str) -> String {
    if prev.is_empty() {
        return current
            .lines()
            .rev()
            .take(15)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
    }
    let prev_lines: Vec<&str> = prev.lines().collect();
    let cur_lines: Vec<&str> = current.lines().collect();

    // Find the longest common suffix
    let common_tail_len = prev_lines
        .iter()
        .rev()
        .zip(cur_lines.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let new_start = if common_tail_len >= cur_lines.len() {
        return String::new();
    } else {
        cur_lines.len() - common_tail_len
    };

    // Find the first line that differs from the beginning
    let prefix_match = prev_lines
        .iter()
        .zip(cur_lines.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let start = prefix_match.min(new_start);

    cur_lines[start..]
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
            "🟢 <b>Ω OmegaOS Telegram Bridge</b>\n\
             ━━━━━━━━━━\n\n\
             <b>Commands:</b>\n\
             /help — this message\n\
             /list — show all rmux sessions\n\
             /status <code>[session]</code> — capture last 20 lines\n\
             /billing — current Claude usage\n\
             /aisb <code>text</code> — send to AISB Master\n\
             /relay <code>session text</code> — send to a specific session\n\n\
             <i>Any other message is relayed directly to AISB Master.</i>"
                .to_string(),
        ),
        "/list" => {
            let sessions = mgr.list_sessions().await.ok()?;
            let mut lines = vec!["📋 <b>Sessions</b>\n━━━━━━━━━━".to_string()];
            for sess in sessions {
                let icon = match sess.role {
                    omega_core::session::SessionRole::Oracle => "🔮",
                    omega_core::session::SessionRole::Worker => "⚙️",
                    omega_core::session::SessionRole::Home => "🏠",
                    omega_core::session::SessionRole::System => "🧠",
                };
                lines.push(format!(
                    "{} <code>{}</code>",
                    icon,
                    escape_html(&sess.name)
                ));
            }
            Some(lines.join("\n"))
        }
        "/billing" => {
            let snap = omega_core::monitor::UsageSnapshot::read()
                .ok()
                .flatten()?;
            Some(format!(
                "💰 <b>Billing</b>\n━━━━━━━━━━\n\
                 <b>5h:</b>    <code>{:.1}%</code>\n\
                 <b>Week:</b>  <code>{:.1}%</code>\n\
                 <b>Account:</b> {} ({})",
                snap.precise_5h(),
                snap.precise_week(),
                escape_html(&snap.active_account),
                escape_html(&snap.email),
            ))
        }
        "/status" => {
            let session = if rest.is_empty() {
                &cfg.relay_session
            } else {
                rest
            };
            let content = mgr.capture_pane(session).await.ok()?;
            let tail: Vec<&str> = content.lines().rev().take(20).collect();
            let output = tail
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            let cleaned = clean_terminal_output(&output);
            Some(format!(
                "📺 <b>{}</b>\n<pre>{}</pre>",
                escape_html(session),
                escape_html(&cleaned)
            ))
        }
        "/aisb" => {
            if rest.is_empty() {
                return Some("Usage: /aisb <code>your message</code>".to_string());
            }
            let _ = mgr.send_text(&cfg.relay_session, rest).await;
            Some(format!(
                "⚡ → <code>{}</code>",
                escape_html(&cfg.relay_session)
            ))
        }
        "/relay" => {
            let mut rp = rest.splitn(2, ' ');
            let session = rp.next()?;
            let payload = rp.next().unwrap_or("");
            if payload.is_empty() {
                return Some(
                    "Usage: /relay <code>session text</code>".to_string(),
                );
            }
            let _ = mgr.send_text(session, payload).await;
            Some(format!("⚡ → <code>{}</code>", escape_html(session)))
        }
        _ => None,
    }
}

async fn send_html(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<()> {
    let url = format!("{}/bot{}/sendMessage", API_BASE, bot_token);
    let body = SendMessageReq {
        chat_id,
        text,
        parse_mode: Some("HTML"),
    };
    client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("sendMessage")?;
    Ok(())
}
