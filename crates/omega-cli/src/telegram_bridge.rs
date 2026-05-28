//! Full Telegram bot engine for OmegaOS.
//!
//! Implements the complete handler chain:
//! - Text messages → classify → route to oracle/AISB
//! - Callback queries → inline keyboard actions (stop workers, close oracle)
//! - Reply routing → message_id→project map for auto-dispatch
//! - Report pipeline → oracle result.md → format → send → track
//! - Voice, documents, photos → acknowledge with typed handlers
//!
//! HTML parse_mode throughout, using omega_core::formatting for rich output.

use anyhow::{Context, Result};
use omega_core::formatting;
use omega_core::monitor::OmegaTelegramConfig;
use omega_core::session::SessionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const API_BASE: &str = "https://api.telegram.org";
const TELEGRAM_MAX_MSG_LEN: usize = 4096;

// ── Telegram API types ──

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
    #[serde(default)]
    callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    message_id: i64,
    #[serde(default)]
    text: Option<String>,
    chat: Chat,
    #[serde(default)]
    from: Option<User>,
    #[serde(default)]
    reply_to_message: Option<Box<Message>>,
    #[serde(default)]
    voice: Option<Voice>,
    #[serde(default)]
    document: Option<Document>,
    #[serde(default)]
    photo: Option<Vec<PhotoSize>>,
    #[serde(default)]
    caption: Option<String>,
    #[serde(default)]
    message_thread_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chat {
    id: i64,
    #[serde(default, rename = "type")]
    chat_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i64,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    first_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Voice {
    file_id: String,
    duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    file_id: String,
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PhotoSize {
    file_id: String,
    width: i64,
    height: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CallbackQuery {
    id: String,
    #[serde(default)]
    from: Option<User>,
    #[serde(default)]
    message: Option<Box<Message>>,
    #[serde(default)]
    data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SendMessageResp {
    ok: bool,
    #[serde(default)]
    result: Option<SentMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SentMessage {
    message_id: i64,
}

// ── Inline Keyboard types ──

#[derive(Debug, Clone, Serialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Clone, Serialize)]
struct InlineKeyboardButton {
    text: String,
    callback_data: String,
}

// ── Reply Router ──

#[derive(Debug, Clone)]
struct ReplyRouter {
    message_to_project: HashMap<i64, String>,
}

impl ReplyRouter {
    fn new() -> Self {
        Self {
            message_to_project: HashMap::new(),
        }
    }

    fn track(&mut self, message_id: i64, project: &str) {
        self.message_to_project
            .insert(message_id, project.to_string());
        // Evict old entries to prevent unbounded growth
        if self.message_to_project.len() > 500 {
            let oldest: Vec<i64> = self
                .message_to_project
                .keys()
                .copied()
                .take(100)
                .collect();
            for key in oldest {
                self.message_to_project.remove(&key);
            }
        }
    }

    fn resolve(&self, reply_to_message_id: i64) -> Option<&str> {
        self.message_to_project
            .get(&reply_to_message_id)
            .map(|s| s.as_str())
    }
}

// ── Report Pipeline ──

struct ReportPipeline;

impl ReportPipeline {
    /// Check for oracle result files and return any new reports.
    fn check_for_reports() -> Vec<OracleReport> {
        let mut reports = Vec::new();
        let pattern = "/tmp/aisb-oracle-result-*.md";

        let entries = match glob_files(pattern) {
            Ok(e) => e,
            Err(_) => return reports,
        };

        for path in entries {
            match Self::parse_report(&path) {
                Ok(report) => {
                    reports.push(report);
                    // Remove the signal file after reading
                    let _ = std::fs::remove_file(&path);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to parse oracle report");
                }
            }
        }

        reports
    }

    fn parse_report(path: &std::path::Path) -> Result<OracleReport> {
        let content = std::fs::read_to_string(path)?;
        let mut project = String::new();
        let mut status = String::new();
        let mut build = String::new();
        let mut body = String::new();
        let mut in_body = false;

        for line in content.lines() {
            if line.starts_with("PROJECT:") {
                project = line.trim_start_matches("PROJECT:").trim().to_string();
            } else if line.starts_with("STATUS:") {
                status = line.trim_start_matches("STATUS:").trim().to_string();
            } else if line.starts_with("BUILD:") {
                build = line.trim_start_matches("BUILD:").trim().to_string();
            } else if line.starts_with("## ") || in_body {
                in_body = true;
                body.push_str(line);
                body.push('\n');
            }
        }

        if project.is_empty() {
            // Try to extract from filename: aisb-oracle-result-{project}.md
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(p) = name.strip_prefix("aisb-oracle-result-") {
                    project = p.to_string();
                }
            }
        }

        Ok(OracleReport {
            project,
            status,
            build,
            body,
        })
    }
}

#[derive(Debug, Clone)]
struct OracleReport {
    project: String,
    status: String,
    build: String,
    body: String,
}

impl OracleReport {
    fn to_telegram_html(&self) -> String {
        let status_icon = match self.status.to_uppercase().as_str() {
            "DONE" => "✅",
            "FAILED" => "🔴",
            _ => "📋",
        };
        let build_icon = match self.build.to_uppercase().as_str() {
            "PASS" => "🟢",
            "FAIL" => "🔴",
            _ => "⚪",
        };

        let mut out = format!(
            "{} <b>Oracle Report — {}</b>\n━━━━━━━━━━\n",
            status_icon,
            formatting::escape_html(&self.project)
        );

        if !self.build.is_empty() {
            out.push_str(&format!(
                "{} Build: <code>{}</code>\n\n",
                build_icon,
                formatting::escape_html(&self.build)
            ));
        }

        out.push_str(&formatting::markdown_to_telegram_html(&self.body));
        out
    }

    fn inline_keyboard(&self) -> InlineKeyboardMarkup {
        let project = &self.project;
        InlineKeyboardMarkup {
            inline_keyboard: vec![
                vec![
                    InlineKeyboardButton {
                        text: "🛑 Stop Workers".to_string(),
                        callback_data: format!("stop_workers:{}", project),
                    },
                    InlineKeyboardButton {
                        text: "🔒 Close Oracle".to_string(),
                        callback_data: format!("close_oracle:{}", project),
                    },
                ],
                vec![
                    InlineKeyboardButton {
                        text: "📋 Full Report".to_string(),
                        callback_data: format!("full_report:{}", project),
                    },
                    InlineKeyboardButton {
                        text: "🔄 Continue".to_string(),
                        callback_data: format!("continue:{}", project),
                    },
                ],
            ],
        }
    }
}

// ── Glob helper ──

fn glob_files(pattern: &str) -> Result<Vec<std::path::PathBuf>> {
    let dir = std::path::Path::new("/tmp");
    let prefix = "aisb-oracle-result-";
    let suffix = ".md";

    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(prefix) && name_str.ends_with(suffix) {
                paths.push(entry.path());
            }
        }
    }
    let _ = pattern; // used for documentation clarity
    Ok(paths)
}

// ── Bot Engine ──

pub struct TelegramBotEngine {
    client: reqwest::Client,
    cfg: OmegaTelegramConfig,
    mgr: SessionManager,
    reply_router: Arc<Mutex<ReplyRouter>>,
    reply_chat_id: i64,
}

impl TelegramBotEngine {
    pub async fn new(cfg: OmegaTelegramConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let mgr = SessionManager::connect().await?;

        let reply_chat_id = if !cfg.allow_user_ids.is_empty() {
            cfg.allow_user_ids[0]
        } else {
            cfg.chat_id
        };

        Ok(Self {
            client,
            cfg,
            mgr,
            reply_router: Arc::new(Mutex::new(ReplyRouter::new())),
            reply_chat_id,
        })
    }

    async fn ensure_master(&self) {
        if self.cfg.relay_session == omega_core::aisb::MASTER_SESSION_NAME {
            let agent = omega_core::agents::Agent::Claude;
            let cwd = std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let _ = omega_core::aisb::ensure_master(&self.mgr, agent, &cwd).await;
        }
    }

    /// Handle a text message through the full handler chain.
    async fn handle_text(&self, msg: &Message, text: &str) -> Result<()> {
        let chat_id = msg.chat.id;

        // 1. Check for reply-based routing
        if let Some(reply_msg) = &msg.reply_to_message {
            let router = self.reply_router.lock().await;
            if let Some(project) = router.resolve(reply_msg.message_id) {
                let oracle_session = format!("oracle-{}", project);
                tracing::info!(project = %project, "reply-routed to oracle");
                let _ = self.mgr.send_text(&oracle_session, text).await;
                let _ = self
                    .send_html(chat_id, &format!("⚡ → <code>{}</code>", formatting::escape_html(&oracle_session)))
                    .await;
                return Ok(());
            }
        }

        // 2. Handle commands
        if text.starts_with('/') {
            if let Some(reply) = self.handle_command(text).await {
                let _ = self.send_html(chat_id, &reply).await;
                return Ok(());
            }
        }

        // 3. Relay to AISB Master
        if let Err(_) = self.mgr.send_text(&self.cfg.relay_session, text).await {
            let _ = self
                .send_html(
                    chat_id,
                    "🔄 <i>AISB Master redémarrage — reprise de la conversation…</i>",
                )
                .await;

            let cwd = std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match omega_core::aisb::ensure_master(
                &self.mgr,
                omega_core::agents::Agent::Claude,
                &cwd,
            )
            .await
            {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Err(e) = self.mgr.send_text(&self.cfg.relay_session, text).await {
                        let _ = self
                            .send_html(
                                chat_id,
                                &format!(
                                    "🔴 <b>Relay failed after restart</b>\n<code>{}</code>",
                                    formatting::escape_html(&e.to_string())
                                ),
                            )
                            .await;
                        return Ok(());
                    }
                }
                Err(e) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "🔴 <b>Could not restart AISB Master</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                    return Ok(());
                }
            }
        }

        // 4. Show typing and wait for response
        let _ = self.send_chat_action(chat_id, "typing").await;
        let before = self
            .mgr
            .capture_pane(&self.cfg.relay_session)
            .await
            .unwrap_or_default();

        // Fast-response polling: 300ms ticks (vs 2s before).
        // Strategy: as soon as we detect a ● marker (agent response) followed
        // by 2 consecutive stable captures (response complete), send it.
        let mut response_text = String::new();
        let mut last_response = String::new();
        let mut stable_count = 0;
        let max_ticks = 200; // 200 × 300ms = 60s max

        for tick in 0..max_ticks {
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Refresh typing indicator every ~4s (expires at 5s)
            if tick % 13 == 12 {
                let _ = self.send_chat_action(chat_id, "typing").await;
            }

            let after = self
                .mgr
                .capture_pane(&self.cfg.relay_session)
                .await
                .unwrap_or_default();

            if after == before {
                continue;
            }

            let current = extract_response(&before, &after);

            // If we have a response and it hasn't changed for 2 ticks → ship it
            if !current.is_empty() && current == last_response {
                stable_count += 1;
                if stable_count >= 2 {
                    response_text = current;
                    break;
                }
            } else {
                stable_count = 0;
                last_response = current;
            }

            // Final fallback: idle prompt detection
            let last_lines: Vec<&str> = after.lines().rev().take(5).collect();
            let is_idle = last_lines
                .iter()
                .any(|l| l.trim().starts_with("❯") && l.trim().len() <= 2);
            if is_idle && !last_response.is_empty() {
                response_text = last_response.clone();
                break;
            }
        }

        // If timeout reached but we have a partial response, send it anyway
        if response_text.is_empty() && !last_response.is_empty() {
            response_text = last_response;
        }

        let cleaned = clean_terminal_output(&response_text);
        if !cleaned.is_empty() {
            let formatted = format_agent_response(&cleaned);
            let chunks = formatting::split_message(&formatted, TELEGRAM_MAX_MSG_LEN);
            for chunk in chunks {
                let _ = self.send_html(chat_id, &chunk).await;
            }
        }

        Ok(())
    }

    /// Handle voice messages.
    async fn handle_voice(&self, msg: &Message, voice: &Voice) -> Result<()> {
        tracing::info!(
            file_id = %voice.file_id,
            duration = voice.duration,
            "received voice message"
        );
        let _ = self
            .send_html(
                msg.chat.id,
                &format!(
                    "🎤 <i>Voice message received ({}s) — transcription not yet available in OmegaOS.\n\
                     Send as text for now.</i>",
                    voice.duration
                ),
            )
            .await;
        Ok(())
    }

    /// Handle document uploads.
    async fn handle_document(&self, msg: &Message, doc: &Document) -> Result<()> {
        let name = doc.file_name.as_deref().unwrap_or("unknown");
        let mime = doc.mime_type.as_deref().unwrap_or("application/octet-stream");
        tracing::info!(file_id = %doc.file_id, name = %name, mime = %mime, "received document");
        let _ = self
            .send_html(
                msg.chat.id,
                &format!(
                    "📄 <i>Document received: <code>{}</code> ({})\n\
                     Document processing not yet available — describe what you need in text.</i>",
                    formatting::escape_html(name),
                    formatting::escape_html(mime)
                ),
            )
            .await;
        Ok(())
    }

    /// Handle photo messages.
    async fn handle_photo(&self, msg: &Message, photos: &[PhotoSize]) -> Result<()> {
        let largest = photos.iter().max_by_key(|p| p.width * p.height);
        if let Some(photo) = largest {
            tracing::info!(
                file_id = %photo.file_id,
                size = format!("{}x{}", photo.width, photo.height),
                "received photo"
            );
        }
        let caption_note = msg
            .caption
            .as_deref()
            .map(|c| format!("\nCaption: <i>{}</i>", formatting::escape_html(c)))
            .unwrap_or_default();
        let _ = self
            .send_html(
                msg.chat.id,
                &format!(
                    "📷 <i>Photo received.{}\n\
                     Image analysis not yet available — describe what you need in text.</i>",
                    caption_note
                ),
            )
            .await;
        Ok(())
    }

    /// Handle callback queries from inline keyboards.
    async fn handle_callback(&self, cb: &CallbackQuery) -> Result<()> {
        let data = cb.data.as_deref().unwrap_or("");
        let chat_id = cb
            .message
            .as_ref()
            .map(|m| m.chat.id)
            .unwrap_or(self.reply_chat_id);

        tracing::info!(data = %data, "callback query received");

        let (action, project) = data.split_once(':').unwrap_or((data, ""));

        let reply = match action {
            "stop_workers" => {
                if project.is_empty() {
                    "⚠️ No project specified".to_string()
                } else {
                    self.stop_project_workers(project).await
                }
            }
            "close_oracle" => {
                if project.is_empty() {
                    "⚠️ No project specified".to_string()
                } else {
                    self.close_oracle(project).await
                }
            }
            "full_report" => {
                if project.is_empty() {
                    "⚠️ No project specified".to_string()
                } else {
                    self.get_full_report(project).await
                }
            }
            "continue" => {
                if project.is_empty() {
                    "⚠️ No project specified".to_string()
                } else {
                    let oracle_session = format!("oracle-{}", project);
                    let _ = self.mgr.send_text(&oracle_session, "continue").await;
                    format!("🔄 Continuing oracle for <b>{}</b>", formatting::escape_html(project))
                }
            }
            _ => format!("❓ Unknown action: <code>{}</code>", formatting::escape_html(action)),
        };

        let _ = self.send_html(chat_id, &reply).await;
        let _ = self.answer_callback_query(&cb.id, "").await;

        Ok(())
    }

    async fn stop_project_workers(&self, project: &str) -> String {
        let sessions = self.mgr.list_sessions().await.unwrap_or_default();
        let prefix = format!("{}-", project);
        let mut killed = 0;

        for sess in &sessions {
            if sess.name.starts_with(&prefix)
                && sess.role == omega_core::session::SessionRole::Worker
            {
                if let Ok(_) = self.mgr.kill_session(&sess.name).await {
                    killed += 1;
                }
            }
        }

        format!(
            "🛑 Stopped <b>{}</b> worker(s) for <b>{}</b>",
            killed,
            formatting::escape_html(project)
        )
    }

    async fn close_oracle(&self, project: &str) -> String {
        let oracle_session = format!("oracle-{}", project);
        match self.mgr.kill_session(&oracle_session).await {
            Ok(_) => format!(
                "🔒 Oracle <code>{}</code> closed",
                formatting::escape_html(&oracle_session)
            ),
            Err(e) => format!(
                "⚠️ Could not close oracle: <code>{}</code>",
                formatting::escape_html(&e.to_string())
            ),
        }
    }

    async fn get_full_report(&self, project: &str) -> String {
        let oracle_session = format!("oracle-{}", project);
        match self.mgr.capture_pane(&oracle_session).await {
            Ok(content) => {
                let tail: Vec<&str> = content.lines().rev().take(40).collect();
                let output: Vec<&str> = tail.into_iter().rev().collect();
                let cleaned = clean_terminal_output(&output.join("\n"));
                format!(
                    "📋 <b>Full Report — {}</b>\n━━━━━━━━━━\n<pre>{}</pre>",
                    formatting::escape_html(project),
                    formatting::escape_html(&cleaned)
                )
            }
            Err(_) => format!(
                "⚠️ Oracle <code>{}</code> not found",
                formatting::escape_html(&oracle_session)
            ),
        }
    }

    /// Handle slash commands.
    async fn handle_command(&self, text: &str) -> Option<String> {
        let mut parts = text.splitn(2, ' ');
        let cmd = parts.next()?;
        // Strip @botname suffix from commands
        let cmd = cmd.split('@').next().unwrap_or(cmd);
        let rest = parts.next().unwrap_or("");

        match cmd {
            "/start" | "/help" => Some(
                "🟢 <b>Ω OmegaOS Bot Engine</b>\n\
                 ━━━━━━━━━━\n\n\
                 <b>Commands:</b>\n\
                 /help — this message\n\
                 /list — show all rmux sessions\n\
                 /status <code>[session]</code> — capture last 20 lines\n\
                 /billing — current Claude usage\n\
                 /skills — list available skills\n\
                 /audits — show Quality Arsenal status\n\
                 /aisb <code>text</code> — send to AISB Master\n\
                 /relay <code>session text</code> — send to specific session\n\
                 /kill <code>session</code> — kill a session\n\n\
                 <i>Reply to any report to auto-route to that project.\n\
                 Any other message goes to AISB Master.</i>"
                    .to_string(),
            ),

            "/list" => {
                let sessions = self.mgr.list_sessions().await.ok()?;
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
                        formatting::escape_html(&sess.name)
                    ));
                }
                if lines.len() == 1 {
                    lines.push("  <i>No active sessions</i>".to_string());
                }
                Some(lines.join("\n"))
            }

            "/billing" => {
                let snap = omega_core::monitor::UsageSnapshot::read().ok().flatten()?;
                Some(format!(
                    "💰 <b>Billing</b>\n━━━━━━━━━━\n\
                     <b>5h:</b>    <code>{:.1}%</code>\n\
                     <b>Week:</b>  <code>{:.1}%</code>\n\
                     <b>Account:</b> {} ({})",
                    snap.precise_5h(),
                    snap.precise_week(),
                    formatting::escape_html(&snap.active_account),
                    formatting::escape_html(&snap.email),
                ))
            }

            "/status" => {
                let session = if rest.is_empty() {
                    &self.cfg.relay_session
                } else {
                    rest.trim()
                };
                let content = self.mgr.capture_pane(session).await.ok()?;
                let tail: Vec<&str> = content.lines().rev().take(20).collect();
                let output: Vec<&str> = tail.into_iter().rev().collect();
                let cleaned = clean_terminal_output(&output.join("\n"));
                Some(format!(
                    "📺 <b>{}</b>\n<pre>{}</pre>",
                    formatting::escape_html(session),
                    formatting::escape_html(&cleaned)
                ))
            }

            "/skills" => {
                let registry = omega_core::skill_registry::SkillRegistry::discover_default();
                match registry {
                    Ok(mut reg) => {
                        reg.register_audits();
                        let skills = reg.list();
                        let mut lines = vec![format!(
                            "🔧 <b>Skills</b> ({})\n━━━━━━━━━━",
                            skills.len()
                        )];
                        let mut by_cat: HashMap<&str, Vec<&omega_core::skill_registry::Skill>> =
                            HashMap::new();
                        for skill in &skills {
                            by_cat
                                .entry(skill.category.label())
                                .or_default()
                                .push(skill);
                        }
                        let mut cats: Vec<_> = by_cat.keys().copied().collect();
                        cats.sort();
                        for cat in cats {
                            lines.push(format!("\n<b>{}</b>", formatting::escape_html(cat)));
                            for skill in &by_cat[cat] {
                                lines.push(format!(
                                    "  <code>{}</code> — {}",
                                    formatting::escape_html(&skill.name),
                                    formatting::escape_html(&skill.description)
                                ));
                            }
                        }
                        Some(lines.join("\n"))
                    }
                    Err(_) => Some("⚠️ <i>Could not load skill registry</i>".to_string()),
                }
            }

            "/audits" => {
                let audits = omega_core::audit::all_audits();
                let mut lines = vec![format!(
                    "🛡️ <b>Quality Arsenal</b> ({} audits)\n━━━━━━━━━━",
                    audits.len()
                )];
                for audit in &audits {
                    let ro = if audit.read_only { " 📖" } else { "" };
                    lines.push(format!(
                        "  <code>/{}</code> — {} ({} phases, /{}){ro}",
                        audit.id, audit.description, audit.phases, audit.max_score
                    ));
                }
                Some(lines.join("\n"))
            }

            "/aisb" => {
                if rest.is_empty() {
                    return Some("Usage: /aisb <code>your message</code>".to_string());
                }
                let _ = self.mgr.send_text(&self.cfg.relay_session, rest).await;
                Some(format!(
                    "⚡ → <code>{}</code>",
                    formatting::escape_html(&self.cfg.relay_session)
                ))
            }

            "/relay" => {
                let mut rp = rest.splitn(2, ' ');
                let session = rp.next()?;
                let payload = rp.next().unwrap_or("");
                if payload.is_empty() {
                    return Some("Usage: /relay <code>session text</code>".to_string());
                }
                let _ = self.mgr.send_text(session, payload).await;
                Some(format!(
                    "⚡ → <code>{}</code>",
                    formatting::escape_html(session)
                ))
            }

            "/kill" => {
                if rest.is_empty() {
                    return Some("Usage: /kill <code>session</code>".to_string());
                }
                let session = rest.trim();
                match self.mgr.kill_session(session).await {
                    Ok(_) => Some(format!(
                        "🛑 Killed <code>{}</code>",
                        formatting::escape_html(session)
                    )),
                    Err(e) => Some(format!(
                        "⚠️ Could not kill <code>{}</code>: {}",
                        formatting::escape_html(session),
                        formatting::escape_html(&e.to_string())
                    )),
                }
            }

            _ => None,
        }
    }

    // ── Telegram API methods ──

    async fn send_html(&self, chat_id: i64, text: &str) -> Result<Option<i64>> {
        let chunks = formatting::split_message(text, TELEGRAM_MAX_MSG_LEN);
        let mut last_msg_id = None;

        for chunk in chunks {
            let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
            let body = serde_json::json!({
                "chat_id": chat_id,
                "text": chunk,
                "parse_mode": "HTML",
            });

            match self.client.post(&url).json(&body).send().await {
                Ok(resp) => {
                    if let Ok(parsed) = resp.json::<SendMessageResp>().await {
                        if let Some(msg) = parsed.result {
                            last_msg_id = Some(msg.message_id);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "sendMessage failed");
                }
            }
        }

        Ok(last_msg_id)
    }

    async fn send_html_with_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: &InlineKeyboardMarkup,
    ) -> Result<Option<i64>> {
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "reply_markup": keyboard,
        });

        match self.client.post(&url).json(&body).send().await {
            Ok(resp) => {
                if let Ok(parsed) = resp.json::<SendMessageResp>().await {
                    Ok(parsed.result.map(|m| m.message_id))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "sendMessage with keyboard failed");
                Ok(None)
            }
        }
    }

    async fn send_chat_action(&self, chat_id: i64, action: &str) -> Result<()> {
        let url = format!("{}/bot{}/sendChatAction", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "action": action,
        });
        let _ = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("sendChatAction")?;
        Ok(())
    }

    async fn answer_callback_query(&self, callback_id: &str, text: &str) -> Result<()> {
        let url = format!(
            "{}/bot{}/answerCallbackQuery",
            API_BASE, self.cfg.bot_token
        );
        let mut body = serde_json::json!({
            "callback_query_id": callback_id,
        });
        if !text.is_empty() {
            body["text"] = serde_json::Value::String(text.to_string());
        }
        let _ = self.client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Process pending oracle reports and deliver them.
    async fn deliver_reports(&self) {
        let reports = ReportPipeline::check_for_reports();
        for report in reports {
            let html = report.to_telegram_html();
            let keyboard = report.inline_keyboard();

            match self
                .send_html_with_keyboard(self.reply_chat_id, &html, &keyboard)
                .await
            {
                Ok(Some(msg_id)) => {
                    let mut router = self.reply_router.lock().await;
                    router.track(msg_id, &report.project);
                    tracing::info!(
                        project = %report.project,
                        msg_id = msg_id,
                        "delivered oracle report"
                    );
                }
                Ok(None) => {
                    tracing::warn!(project = %report.project, "report sent but no message_id");
                }
                Err(e) => {
                    tracing::error!(project = %report.project, error = %e, "failed to deliver report");
                }
            }
        }
    }
}

// ── Main entry point ──

pub async fn run(cfg: OmegaTelegramConfig) -> Result<()> {
    println!("◆ Omega Telegram bot engine starting");
    println!("  Relay session: {}", cfg.relay_session);
    println!("  Chat ID:       {}", cfg.chat_id);

    let engine = TelegramBotEngine::new(cfg.clone()).await?;
    engine.ensure_master().await;

    let _ = engine
        .send_html(
            engine.reply_chat_id,
            "🟢 <b>Ω OmegaOS Bot Engine</b> — online\n\n\
             <i>Full handler chain active: text, voice, docs, photos, callbacks.\n\
             Reply to any report to auto-route. /help for commands.</i>",
        )
        .await;

    let mut offset: i64 = 0;
    let mut last_healthcheck = std::time::Instant::now();
    let mut last_report_check = std::time::Instant::now();

    loop {
        // Periodic healthcheck every 60s
        if last_healthcheck.elapsed() > Duration::from_secs(60) {
            last_healthcheck = std::time::Instant::now();
            let sessions = engine.mgr.list_sessions().await.unwrap_or_default();
            if !sessions
                .iter()
                .any(|s| s.name == engine.cfg.relay_session)
            {
                tracing::info!("AISB Master not found — restarting");
                engine.ensure_master().await;
            }
        }

        // Check for oracle reports every 3s
        if last_report_check.elapsed() > Duration::from_secs(3) {
            last_report_check = std::time::Instant::now();
            engine.deliver_reports().await;
        }

        // Long-poll for updates
        let url = format!(
            "{}/bot{}/getUpdates?timeout=25&offset={}&allowed_updates=[\"message\",\"callback_query\"]",
            API_BASE, cfg.bot_token, offset
        );

        let resp = match engine.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "getUpdates failed");
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

            // Handle callback queries (inline keyboard presses)
            if let Some(cb) = upd.callback_query {
                let sender_id = cb.from.as_ref().map(|u| u.id);
                let chat_id = cb
                    .message
                    .as_ref()
                    .map(|m| m.chat.id)
                    .unwrap_or(engine.reply_chat_id);
                if cfg.is_authorized(chat_id, sender_id) {
                    let _ = engine.handle_callback(&cb).await;
                }
                continue;
            }

            // Handle messages
            let Some(msg) = upd.message else { continue };

            let sender_id = msg.from.as_ref().map(|u| u.id);
            if !cfg.is_authorized(msg.chat.id, sender_id) {
                tracing::warn!(
                    chat_id = msg.chat.id,
                    sender_id = ?sender_id,
                    "rejected unauthorized message"
                );
                continue;
            }

            // Route by message type
            if let Some(text) = msg.text.as_deref() {
                tracing::info!(text = %text, chat_id = msg.chat.id, "text message");
                let _ = engine.handle_text(&msg, text).await;
            } else if let Some(voice) = &msg.voice {
                let _ = engine.handle_voice(&msg, voice).await;
            } else if let Some(doc) = &msg.document {
                let _ = engine.handle_document(&msg, doc).await;
            } else if let Some(photos) = &msg.photo {
                if !photos.is_empty() {
                    let _ = engine.handle_photo(&msg, photos).await;
                }
            }
        }
    }
}

// ── Terminal output helpers (preserved from original) ──

fn clean_terminal_output(text: &str) -> String {
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07").unwrap_or_else(|_| {
        regex::Regex::new(r"$^").unwrap()
    });
    let stripped = ansi_re.replace_all(text, "");
    let no_reminders = regex::Regex::new(r"(?s)<system-reminder>.*?</system-reminder>")
        .map(|re| re.replace_all(&stripped, "").to_string())
        .unwrap_or_else(|_| stripped.to_string());
    let lines: Vec<&str> = no_reminders
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| {
            if l.is_empty() {
                return false;
            }
            let t = l.trim();
            if t.starts_with("❯") { return false; }
            if t.starts_with("⎿") { return false; }
            if t.starts_with("·") { return false; }
            if t.starts_with("✻") { return false; }
            if t.contains("Cultivating") { return false; }
            if t.contains("Brewing") || t.contains("Brewed") { return false; }
            if t.contains("Crunched") || t.contains("Crunching") { return false; }
            if t.contains("Pontificating") { return false; }
            if t.contains("Thinking") && t.len() < 30 { return false; }
            if t.contains("skills available") { return false; }
            if t.contains("bypass permissions") { return false; }
            if t.contains("shift+tab to cycle") { return false; }
            if t.contains("← for agents") { return false; }
            if t.contains("esc to interrupt") { return false; }
            if t.contains("Press up to edit") { return false; }
            if t.chars().all(|c| c == '─' || c == '━' || c == ' ') { return false; }
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

fn format_agent_response(text: &str) -> String {
    let escaped = formatting::escape_html(text);
    let code_lines = escaped
        .lines()
        .filter(|l| {
            l.starts_with("  ") || l.starts_with("\t") || l.contains("fn ") || l.contains("{}")
        })
        .count();
    let total_lines = escaped.lines().count().max(1);

    if code_lines as f32 / total_lines as f32 > 0.5 {
        format!("<pre>{}</pre>", escaped)
    } else {
        escaped
    }
}

fn extract_response(_before: &str, after: &str) -> String {
    // Strategy: find the LAST ● block (most recent agent response).
    // Don't diff line-by-line — pane scrolls break that approach.
    // The last ● is always the response to the user's most recent message.
    let lines: Vec<&str> = after.lines().collect();

    let last_bullet_idx = lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, l)| l.trim().starts_with("●"))
        .map(|(i, _)| i);

    let Some(start) = last_bullet_idx else {
        return String::new();
    };

    let mut response_lines: Vec<String> = Vec::new();
    let first = lines[start].trim().trim_start_matches("●").trim();
    if !first.is_empty() {
        response_lines.push(first.to_string());
    }

    // Collect continuation lines (until stop marker)
    for line in lines.iter().skip(start + 1) {
        let t = line.trim();
        if t.is_empty() { continue; }

        if t.starts_with("❯")
            || t.starts_with("✻")
            || t.starts_with("⎿")
            || t.starts_with("·")
            || t.starts_with("●")
            || t.contains("bypass permissions")
            || t.contains("esc to interrupt")
            || t.contains("shift+tab")
            || t.starts_with("───")
            || t.starts_with("━━━")
            || t.contains("Churned")
            || t.contains("Brewed")
            || t.contains("Cultivating")
            || t.contains("Crunched")
            || t.contains("Pontificating")
        {
            break;
        }

        response_lines.push(t.to_string());
    }

    response_lines.join("\n")
}
