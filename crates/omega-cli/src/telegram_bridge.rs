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
use omega_core::account::{self, CurrentAccount};
use omega_core::credentials::CredentialStore;
use omega_core::formatting;
use omega_core::monitor::OmegaTelegramConfig;
use omega_core::oauth;
use omega_core::providers::{ActiveModel, ProvidersConfig};
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
    /// Fires when the bot's membership in a chat changes — e.g. added or
    /// promoted to admin. This lets us auto-detect a project supergroup
    /// the moment the user makes the bot admin, so no manual `/setupgroup`
    /// is needed.
    #[serde(default)]
    my_chat_member: Option<ChatMemberUpdated>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMemberUpdated {
    chat: Chat,
    #[serde(default)]
    from: Option<User>,
    #[serde(default)]
    new_chat_member: Option<ChatMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMember {
    /// "creator" | "administrator" | "member" | "left" | "kicked"
    status: String,
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
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    is_forum: Option<bool>,
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
            "DONE" => "[DONE]",
            "FAILED" => "[FAILED]",
            _ => "[REPORT]",
        };
        let build_icon = match self.build.to_uppercase().as_str() {
            "PASS" => "PASS",
            "FAIL" => "FAIL",
            _ => "-",
        };

        let mut out = format!(
            "{} <b>Oracle Report — {}</b>\n\n",
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
                        text: " Stop Workers".to_string(),
                        callback_data: format!("stop_workers:{}", project),
                    },
                    InlineKeyboardButton {
                        text: " Close Oracle".to_string(),
                        callback_data: format!("close_oracle:{}", project),
                    },
                ],
                vec![
                    InlineKeyboardButton {
                        text: " Full Report".to_string(),
                        callback_data: format!("full_report:{}", project),
                    },
                    InlineKeyboardButton {
                        text: " Continue".to_string(),
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
    /// When set, the next plain-text message is relayed to THIS session
    /// instead of the default relay_session (set by tapping a session button).
    targeted_session: Arc<Mutex<Option<String>>>,
    /// Persistent claude subprocess for instant master-chat responses.
    /// Lazy-spawned on first message; respawns on crash.
    claude_stream: Arc<crate::claude_stream::ClaudeStreamHandle>,
    /// Bridge process start, surfaced as uptime in `/status`.
    start_time: std::time::Instant,
    /// Pending /dispatch confirmations: short token → (project, work_dir,
    /// amplified brief, raw mission, created_at). Cleared on Confirm/Cancel
    /// click. Lets the user preview the structured brief before the oracle
    /// is actually spawned (Pack INTERACTION — human-in-the-loop dispatch).
    pending_dispatches: Arc<Mutex<std::collections::HashMap<String, PendingDispatch>>>,
}

#[derive(Clone)]
struct PendingDispatch {
    project: String,
    work_dir: String,
    brief: String,
    raw_mission: String,
    created_at: chrono::DateTime<chrono::Utc>,
    requested_by_chat: i64,
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

        let bridge_cfg_dir = dirs::home_dir().map(|h| h.join(".omega/claude-bridge-config"));
        Ok(Self {
            client,
            cfg,
            mgr,
            reply_router: Arc::new(Mutex::new(ReplyRouter::new())),
            reply_chat_id,
            targeted_session: Arc::new(Mutex::new(None)),
            claude_stream: Arc::new(crate::claude_stream::ClaudeStreamHandle::new(bridge_cfg_dir)),
            start_time: std::time::Instant::now(),
            pending_dispatches: Arc::new(Mutex::new(std::collections::HashMap::new())),
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
    /// Inject a locally-typed line (from the aisb-master chat REPL) as if
    /// it had arrived from Telegram. Synthesizes a minimal Message with
    /// the owner's chat_id and routes it through the normal handler — so
    /// the reply goes to Telegram AND mirrors into the conversation log
    /// the REPL tails.
    async fn process_local_text(&self, chat_id: i64, text: &str) {
        let msg_json = serde_json::json!({
            "message_id": 0,
            "text": text,
            "chat": { "id": chat_id },
            "from": { "id": chat_id, "first_name": "You" }
        });
        if let Ok(msg) = serde_json::from_value::<Message>(msg_json) {
            let _ = self.handle_text(&msg, text).await;
        }
    }

    async fn handle_text(&self, msg: &Message, text: &str) -> Result<()> {
        let chat_id = msg.chat.id;

        // 1. Check for reply-based routing — if the replied message is one
        // we tracked as belonging to a project oracle, route there directly.
        if let Some(reply_msg) = &msg.reply_to_message {
            let router = self.reply_router.lock().await;
            if let Some(project) = router.resolve(reply_msg.message_id) {
                let oracle_session = format!("oracle-{}", project);
                tracing::info!(project = %project, "reply-routed to oracle");
                let _ = self.mgr.send_text(&oracle_session, text).await;
                let _ = self
                    .send_html(chat_id, &format!(" → <code>{}</code>", formatting::escape_html(&oracle_session)))
                    .await;
                return Ok(());
            }
        }

        // 1b. Reply context (Pack CONTEXT — full thread awareness):
        // If the user is replying to ANY other message (bot output, prior
        // user message, channel post), capture its text and pass it as
        // context to AISB Master. The LLM sees the full thread, not just
        // the bare reply.
        let reply_context: Option<String> = msg.reply_to_message.as_ref().and_then(|r| {
            // Prefer text, fall back to caption (photo/document)
            r.text.clone().or_else(|| r.caption.clone()).map(|body| {
                let author = r
                    .from
                    .as_ref()
                    .and_then(|u| u.first_name.clone().or_else(|| u.username.clone()))
                    .unwrap_or_else(|| "(unknown)".to_string());
                let ts = ""; // Telegram includes msg.date; we skip it here for brevity
                format!(
                    "<reply_context>\n\
                     The user is REPLYING to this earlier message.\n\
                     Author: {}\n{}\
                     Message content:\n---\n{}\n---\n\
                     </reply_context>\n\n\
                     User's new message:\n",
                    author, ts, body
                )
            })
        });

        // 2. OAuth code paste — only triggers if a reauth is pending.
        // Code pattern: 20+ chars of [A-Za-z0-9_-], optionally followed by #state.
        let trimmed = text.trim();
        if oauth::looks_like_oauth_code(trimmed) {
            let state = oauth::PendingReauth::load();
            if state.pending && !state.is_stale() {
                self.handle_oauth_code(chat_id, trimmed).await;
                return Ok(());
            }
        }

        // /cancel clears any targeted session.
        if text.trim() == "/cancel" {
            *self.targeted_session.lock().await = None;
            let _ = self.send_html(chat_id, "Cleared target. Messages now go to AISB Master.").await;
            return Ok(());
        }

        // /setupgroup <id>  — register a Telegram supergroup as the project
        // hub. Auto-creates a forum topic per known project, persists the
        // mapping in ~/.omega/telegram-group.toml. Re-runnable.
        if let Some(rest) = text.trim().strip_prefix("/setupgroup") {
            self.handle_setup_group(chat_id, rest.trim()).await;
            return Ok(());
        }

        // /dispatch <Project> <mission> — preview the amplified brief then
        // confirm-or-cancel via inline buttons (Pack INTERACTION). Accept
        // both `/dispatch` and `/dispatch@BotName` forms (the @suffix is
        // appended by Telegram when commands are sent in groups).
        if let Some(rest) = text
            .trim()
            .strip_prefix("/dispatch")
            .filter(|r| r.is_empty() || r.starts_with(' ') || r.starts_with('@'))
        {
            let after_at = match rest.strip_prefix('@') {
                Some(tail) => tail.split_once(' ').map(|(_, args)| args).unwrap_or(""),
                None => rest,
            };
            self.handle_dispatch_command(chat_id, msg.message_thread_id, after_at)
                .await;
            return Ok(());
        }

        // /clean — kill + respawn the AISB Master session FRESH (no
        // --continue → brand new conversation, clean slate). Lets the user
        // reset the master at any time from Telegram.
        if text.trim() == "/clean" {
            let placeholder = formatting::thinking_placeholder("AISB Master");
            let pid = self.send_html(chat_id, &placeholder).await?.unwrap_or(0);
            let result = self.clean_master().await;
            let body = match result {
                Ok(_) => "AISB Master cleaned and restarted fresh. New conversation, clean slate. Ping me with the mission.",
                Err(_) => "AISB Master kill succeeded but respawn had an issue — it will auto-respawn on the next message.",
            };
            let card = formatting::smart_wrap_response(
                "AISB Master", body, 0.0, "system", None, None, None,
                formatting::ResponseTier::Ok,
            );
            if pid != 0 {
                let _ = self.edit_message_html(chat_id, pid, &card).await;
            } else {
                let _ = self.send_html(chat_id, &card).await;
            }
            return Ok(());
        }

        // 3. Handle commands with inline keyboards (return early if handled).
        if text.starts_with('/') {
            if self.try_handle_keyboard_command(chat_id, text).await {
                return Ok(());
            }
            // Plain text commands return a string we send normally. If the
            // command was typed inside a forum topic, thread the reply back
            // into that topic so it doesn't bleed into the group's General.
            if let Some(reply) = self.handle_command(text).await {
                let _ = self
                    .send_html_smart(chat_id, msg.message_thread_id, &reply)
                    .await;
                return Ok(());
            }
        }

        // 3b. Topic → project oracle. A message typed inside a project's forum
        // topic IS a conversation with that project's oracle: spawn the oracle
        // if none is alive, continue the existing one otherwise. Plain
        // (non-command) text only — slash commands above still work in-topic.
        if self.try_route_topic_to_oracle(msg, text).await {
            return Ok(());
        }

        // Resolve the relay target: a tapped session/project overrides the default.
        // Special case: "__newproject__:<location>" means create a project with
        // this message as the name.
        let target = {
            let mut guard = self.targeted_session.lock().await;
            let t = guard.clone();
            // One-shot: clear after consuming (except for oracle/session targets
            // which the user may want to keep — but simplest is one-shot)
            *guard = None;
            t
        };

        if let Some(t) = &target {
            if let Some(location) = t.strip_prefix("__newproject__:") {
                // The message text is the project name
                self.handle_newproject(chat_id, &format!("/newproject {} {}", text.trim(), location)).await;
                return Ok(());
            }
        }

        let relay_target = target.clone().unwrap_or_else(|| self.cfg.relay_session.clone());

        // If targeting a specific rmux session (oracle/worker), use pane relay.
        // Otherwise (default = AISB Master chat), use direct `claude --print`
        // which is dramatically faster (streams stdout, no pane polling).
        let is_master_chat = relay_target == omega_core::aisb::MASTER_SESSION_NAME;

        if is_master_chat {
            // ── Minimal Engineer + Smart packs flow ────────────────────────
            // 1. React 👀 on user's message (instant ack)
            // 2. Send a "Thinking…" placeholder THREADED as reply_to user msg
            // 3. Typing-action ticker every 4s
            // 4. Edit placeholder with smart_wrap (expandable sections +
            //    mention by id + secret spoilers)
            // 5. If body too large → sendDocument with the tail
            let (provider, model) = read_active_provider_model();
            let agent_label = "AISB Master";
            let model_label = if model.is_empty() { provider.clone() } else { model.clone() };

            // Extract user identity for the mention pack
            let user_id = msg.from.as_ref().map(|u| u.id);
            let user_name = msg
                .from
                .as_ref()
                .and_then(|u| u.first_name.clone().or_else(|| u.username.clone()));
            let user_msg_id = msg.message_id;

            // Pack CONTEXT: thread the reply, react 👀 to ack receipt
            let _ = self.set_message_reaction(chat_id, user_msg_id, "🤔").await;

            let placeholder = formatting::thinking_placeholder(agent_label);
            let placeholder_id = self
                .send_html_reply(chat_id, &placeholder, Some(user_msg_id))
                .await?
                .unwrap_or(0);
            let started_for_progress = std::time::Instant::now();

            // Typing+progress ticker — fire-and-forget; aborts when this
            // scope ends. Every 3s it (a) refreshes the "typing" bubble
            // and (b) edits the placeholder with an updated progress bar
            // showing elapsed time (Pack PROGRESS).
            let ticker = {
                let client = self.client.clone();
                let token = self.cfg.bot_token.clone();
                let cid = chat_id;
                let agent_label_owned = agent_label.to_string();
                let pid = placeholder_id;
                tokio::spawn(async move {
                    loop {
                        // 1. Typing bubble
                        let url = format!("{}/bot{}/sendChatAction", API_BASE, token);
                        let body = serde_json::json!({"chat_id": cid, "action": "typing"});
                        let _ = client.post(&url).json(&body).send().await;
                        // 2. Live progress edit (only if we own a placeholder)
                        if pid != 0 {
                            let elapsed = started_for_progress.elapsed().as_secs_f32();
                            let text = formatting::thinking_progress(&agent_label_owned, elapsed);
                            let edit_url = format!("{}/bot{}/editMessageText", API_BASE, token);
                            let edit_body = serde_json::json!({
                                "chat_id": cid,
                                "message_id": pid,
                                "text": text,
                                "parse_mode": "HTML",
                            });
                            let _ = client.post(&edit_url).json(&edit_body).send().await;
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    }
                })
            };

            let started = std::time::Instant::now();

            // Combine reply context (if any) + new user text into a single
            // prompt so the LLM sees the full thread, not just the bare reply.
            let final_prompt: String = match &reply_context {
                Some(ctx) => format!("{}{}", ctx, text),
                None => text.to_string(),
            };

            // ─── OWN CLAUDE SDK (the bot's own brain) ─────────────────
            // The bot owns a persistent `claude --print --output-format
            // stream-json` subprocess (claude_stream.rs) running from $HOME
            // with --dangerously-skip-permissions and the AISB Master
            // system prompt. It has FULL VPS access (Bash/Read/Write/etc.)
            // and responds DIRECTLY to Telegram — no pane scraping, no
            // fragile extraction. This is the old VPS Omega model
            // (ClaudeSDKClient) reimplemented over the CLI stream.
            //
            // We also MIRROR the exchange into the rmux aisb-master pane
            // (display-only) so the user can WATCH the conversation live
            // in the TUI. The mirror is best-effort and never blocks the
            // Telegram response.
            let result: std::result::Result<String, String> = if provider == "claude"
                || provider.is_empty()
            {
                self.claude_stream.ask(&final_prompt).await.map_err(|e| e.to_string())
            } else {
                // Non-Claude providers (Gemini/Codex/GLM/Pi) still use the
                // one-shot CLI path.
                let prompt = final_prompt.clone();
                let p = provider.clone();
                let m = model.clone();
                tokio::task::spawn_blocking(move || run_llm_oneshot(&p, &m, &prompt))
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.map_err(|e| e.to_string()))
                    .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            };

            ticker.abort();
            let duration = started.elapsed().as_secs_f32();

            // Format + deliver. The user's original message is quoted
            // at the top of the bot's response (Pack QUOTE) so the chat
            // stays self-documenting when the user scrolls back.
            match result {
                Ok(body) if !body.is_empty() => {
                    // OmegaTrace — append this turn to today's trajectory
                    // JSONL. Fire-and-forget; never blocks the response.
                    {
                        use omega_core::trajectory::{Trajectory, TurnRole};
                        let mut traj = Trajectory::new(
                            format!("telegram-chat-{}", chat_id),
                            model_label.clone(),
                        );
                        traj.push(TurnRole::Human, text);
                        traj.push(TurnRole::Gpt, body.clone());
                        omega_core::trajectory::append_silent(&traj);
                    }
                    // MIRROR into the rmux aisb-master pane so the user can
                    // watch the Telegram conversation live in the TUI. The
                    // SDK subprocess owns the real conversation; this pane
                    // is a read-only echo. Best-effort, never blocks.
                    self.mirror_to_master_pane(text, &body).await;
                    let file_target = formatting::suggest_file_delivery(&body);
                    let wrapped = formatting::smart_wrap_response(
                        agent_label,
                        if file_target.is_some() {
                            "_Output too large — attached as file._"
                        } else {
                            &body
                        },
                        duration,
                        &model_label,
                        user_id,
                        user_name.as_deref(),
                        Some(text),
                        formatting::ResponseTier::Ok,
                    );

                    let chunks = formatting::split_message(&wrapped, TELEGRAM_MAX_MSG_LEN);
                    if let Some(first) = chunks.first() {
                        if placeholder_id != 0 {
                            let _ = self.edit_message_html(chat_id, placeholder_id, first).await;
                        } else {
                            let _ = self
                                .send_html_reply(chat_id, first, Some(user_msg_id))
                                .await;
                        }
                    }
                    for tail in chunks.iter().skip(1) {
                        let _ = self.send_html(chat_id, tail).await;
                    }
                    if let Some((filename, mime)) = file_target {
                        let _ = self
                            .send_document_bytes(chat_id, &filename, mime, body.as_bytes())
                            .await;
                    }
                }
                Ok(_) => {
                    let empty = formatting::smart_wrap_response(
                        agent_label,
                        "The model returned an empty response. Try rephrasing or ping me with more context.",
                        duration,
                        &model_label,
                        user_id,
                        user_name.as_deref(),
                        Some(text),
                        formatting::ResponseTier::Empty,
                    );
                    if placeholder_id != 0 {
                        let _ = self.edit_message_html(chat_id, placeholder_id, &empty).await;
                    } else {
                        let _ = self
                            .send_html_reply(chat_id, &empty, Some(user_msg_id))
                            .await;
                    }
                }
                Err(e) => {
                    let body = format!("`{} error`\n{}", provider, e);
                    let err_html = formatting::smart_wrap_response(
                        agent_label,
                        &body,
                        duration,
                        &model_label,
                        user_id,
                        user_name.as_deref(),
                        Some(text),
                        formatting::ResponseTier::Error,
                    );
                    if placeholder_id != 0 {
                        let _ = self.edit_message_html(chat_id, placeholder_id, &err_html).await;
                    } else {
                        let _ = self
                            .send_html_reply(chat_id, &err_html, Some(user_msg_id))
                            .await;
                    }
                }
            }
            return Ok(());
        }

        // Targeted session path — keep pane-based relay for now.
        let before = self
            .mgr
            .capture_pane(&relay_target)
            .await
            .unwrap_or_default();

        // 3b. Relay to the target session (default: AISB Master)
        if let Err(_) = self.mgr.send_text(&relay_target, text).await {
            let _ = self
                .send_html(
                    chat_id,
                    " <i>AISB Master redémarrage — reprise de la conversation…</i>",
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
                    if let Err(e) = self.mgr.send_text(&relay_target, text).await {
                        let _ = self
                            .send_html(
                                chat_id,
                                &format!(
                                    "<b>Relay failed after restart</b>\n<code>{}</code>",
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
                                "<b>Could not restart AISB Master</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                    return Ok(());
                }
            }
        }

        // 4. Show typing and wait for response (`before` was captured before send)
        let _ = self.send_chat_action(chat_id, "typing").await;

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
                .capture_pane(&relay_target)
                .await
                .unwrap_or_default();

            if after == before {
                continue;
            }

            let current = extract_response(&before, &after);
            if !current.is_empty() {
                tracing::debug!(tick, len = current.len(), "extract_response found content");
            }

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

            // Final fallback: idle prompt detection (bare ❯ prompt)
            let last_lines: Vec<&str> = after.lines().rev().take(5).collect();
            let is_idle = last_lines
                .iter()
                .any(|l| {
                    let t = l.trim();
                    t.starts_with('❯') && t.len() <= 3
                });
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
                    " <i>Voice message received ({}s) — transcription not yet available in OmegaOS.\n\
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
                    " <i>Document received: <code>{}</code> ({})\n\
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
                    " <i>Photo received.{}\n\
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

        // Account/billing callbacks
        if let Some(rest) = data.strip_prefix("acc:") {
            self.handle_account_callback(chat_id, rest).await;
            let _ = self.answer_callback_query(&cb.id, "").await;
            return Ok(());
        }

        // Project menu callbacks
        if let Some(rest) = data.strip_prefix("proj:") {
            self.handle_project_callback(chat_id, rest).await;
            let _ = self.answer_callback_query(&cb.id, "").await;
            return Ok(());
        }

        // Session targeting callbacks
        if let Some(rest) = data.strip_prefix("sess:") {
            self.handle_session_callback(chat_id, rest).await;
            let _ = self.answer_callback_query(&cb.id, "").await;
            return Ok(());
        }

        // Model selection callbacks
        if let Some(rest) = data.strip_prefix("model:") {
            self.handle_model_callback(chat_id, rest).await;
            let _ = self.answer_callback_query(&cb.id, "").await;
            return Ok(());
        }

        // /dispatch confirm/cancel (Pack INTERACTION — human-in-the-loop).
        if let Some(action_token) = data.strip_prefix("dispatch:") {
            let message_id = cb.message.as_ref().map(|m| m.message_id).unwrap_or(0);
            let _ = self.answer_callback_query(&cb.id, "").await;
            if message_id != 0 {
                self.handle_dispatch_callback(chat_id, message_id, action_token)
                    .await;
            }
            return Ok(());
        }

        // Group setup confirm button (sent in-group by /setupgroup).
        if let Some(id_str) = data.strip_prefix("setupgroup:") {
            let _ = self.answer_callback_query(&cb.id, "Setting up…").await;
            if let Ok(group_id) = id_str.parse::<i64>() {
                self.run_group_setup(group_id, chat_id).await;
            }
            return Ok(());
        }

        let (action, project) = data.split_once(':').unwrap_or((data, ""));

        let reply = match action {
            "stop_workers" => {
                if project.is_empty() {
                    " No project specified".to_string()
                } else {
                    self.stop_project_workers(project).await
                }
            }
            "close_oracle" => {
                if project.is_empty() {
                    " No project specified".to_string()
                } else {
                    self.close_oracle(project).await
                }
            }
            "full_report" => {
                if project.is_empty() {
                    " No project specified".to_string()
                } else {
                    self.get_full_report(project).await
                }
            }
            "continue" => {
                if project.is_empty() {
                    " No project specified".to_string()
                } else {
                    let oracle_session = format!("oracle-{}", project);
                    let _ = self.mgr.send_text(&oracle_session, "continue").await;
                    format!(" Continuing oracle for <b>{}</b>", formatting::escape_html(project))
                }
            }
            _ => format!(" Unknown action: <code>{}</code>", formatting::escape_html(action)),
        };

        let _ = self.send_html(chat_id, &reply).await;
        let _ = self.answer_callback_query(&cb.id, "").await;

        Ok(())
    }

    /// Handle keyboard commands that send their own message with inline buttons.
    /// Returns true if the command was handled (caller should NOT fall through
    /// to `handle_command`).
    async fn try_handle_keyboard_command(&self, chat_id: i64, text: &str) -> bool {
        let cmd = text
            .split_whitespace()
            .next()
            .unwrap_or("")
            .split('@')
            .next()
            .unwrap_or("");
        match cmd {
            "/account" => {
                // Centralized: account card has buttons for login/logout/billing/switch
                self.handle_account_command(chat_id, text).await;
                true
            }
            "/model" => {
                self.handle_model_command(chat_id, text).await;
                true
            }
            "/projects" | "/project" | "/newproject" => {
                // Unified project menu with buttons: list / new / add existing / scan
                self.send_projects_menu(chat_id).await;
                true
            }
            "/relay" | "/sessions" => {
                // Show active sessions as buttons; user clicks one to target it
                self.send_sessions_menu(chat_id).await;
                true
            }
            _ => false,
        }
    }

    /// `/model` — multi-provider model selector for this Telegram chat.
    ///
    /// Forms:
    ///   /model                       show current + list providers/models
    ///   /model <provider>            switch provider (uses default model)
    ///   /model <provider> <model>    switch provider + specific model
    async fn handle_model_command(&self, chat_id: i64, text: &str) {
        let parts: Vec<&str> = text.split_whitespace().skip(1).collect();
        if parts.is_empty() {
            let active = ActiveModel::load();
            let mut lines = vec![
                "<b>Model Selector</b>".to_string(),
                format!(
                    "Active: <code>{}</code> / <code>{}</code>",
                    formatting::escape_html(&active.active_provider),
                    formatting::escape_html(&active.active_model),
                ),
                String::new(),
                "<b>Providers</b>".to_string(),
            ];
            for prov in ProvidersConfig::all_providers() {
                let default_model = ProvidersConfig::default_model(prov);
                let auth = ProvidersConfig::auth_type(prov);
                let models = ProvidersConfig::models_for(prov).join(", ");
                lines.push(format!(
                    "  <code>{}</code> ({}) — default: <code>{}</code>\n    models: <i>{}</i>",
                    formatting::escape_html(prov),
                    formatting::escape_html(auth),
                    formatting::escape_html(default_model),
                    formatting::escape_html(&models),
                ));
            }
            lines.push(String::new());
            lines.push(
                "Switch: <code>/model &lt;provider&gt; [model]</code>".to_string(),
            );
            let _ = self.send_html(chat_id, &lines.join("\n")).await;
            return;
        }

        let provider = parts[0];
        if !ProvidersConfig::is_known(provider) {
            let known = ProvidersConfig::all_providers().join(", ");
            let _ = self
                .send_html(
                    chat_id,
                    &format!(
                        "<b>Unknown provider</b> <code>{}</code>\n\
                         Known: <code>{}</code>",
                        formatting::escape_html(provider),
                        formatting::escape_html(&known),
                    ),
                )
                .await;
            return;
        }
        let model = parts.get(1).copied();
        match ActiveModel::set(provider, model) {
            Ok(active) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            "<b>Active model updated</b>\n\
                             Provider: <code>{}</code>\n\
                             Model:    <code>{}</code>",
                            formatting::escape_html(&active.active_provider),
                            formatting::escape_html(&active.active_model),
                        ),
                    )
                    .await;
            }
            Err(e) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            "<b>Could not set model</b>\n<code>{}</code>",
                            formatting::escape_html(&e.to_string())
                        ),
                    )
                    .await;
            }
        }
    }

    /// Spawn a new oracle session for a project.
    async fn spawn_project_oracle(&self, chat_id: i64, project: &str, oracle_name: &str) {
        let registry = omega_core::project_manager::ProjectRegistry::load();
        let entry = registry.projects.iter().find(|p| p.name == project);
        let Some(entry) = entry else {
            self.send_html(
                chat_id,
                &format!("<i>Project <code>{}</code> not in registry.</i>", formatting::escape_html(project)),
            ).await.ok();
            return;
        };
        let cwd = entry.path.display().to_string();

        // Render the Oracle ROLE prompt (agents/oracle.md) with this
        // project's placeholders, write it to a per-oracle file, and
        // launch Claude with it as the system prompt. WITHOUT this the
        // session was a bare vanilla Claude that didn't know it was an
        // oracle — so it never planned or dispatched workers (and died
        // when the process exited, since there was no `exec bash`).
        let prompt_file = render_oracle_prompt(project, &cwd, oracle_name);
        let mut cmd = String::from("claude --dangerously-skip-permissions");
        if let Some(ref pf) = prompt_file {
            cmd.push_str(&format!(" --append-system-prompt-file '{}'", pf.replace('\'', r"'\''")));
        }
        // `exec bash` keeps the rmux session alive after Claude exits, so
        // the oracle pane stays attachable instead of vanishing.
        let wrapped = format!("bash -c '{}; exec bash'", cmd.replace('\'', r"'\''"));
        match self.mgr.create_session(oracle_name, Some(&cwd), Some(&wrapped)).await {
            Ok(_) => {
                *self.targeted_session.lock().await = Some(oracle_name.to_string());
                self.send_html(
                    chat_id,
                    &format!(
                        "Spawned <code>{}</code> in <code>{}</code>.\nNext message goes there. /cancel to clear.",
                        formatting::escape_html(oracle_name),
                        formatting::escape_html(&cwd),
                    ),
                ).await.ok();
            }
            Err(e) => {
                self.send_html(
                    chat_id,
                    &format!("<i>Spawn failed: <code>{}</code></i>", formatting::escape_html(&e.to_string())),
                ).await.ok();
            }
        }
    }

    /// Route a message typed inside a project's forum topic to that project's
    /// oracle. Spawn the oracle if none is alive; otherwise continue the
    /// conversation by sending into the existing pane. Returns true if the
    /// message belonged to a project topic and was handled.
    async fn try_route_topic_to_oracle(&self, msg: &Message, text: &str) -> bool {
        use omega_core::telegram_group::TelegramGroupConfig;
        let Some(thread_id) = msg.message_thread_id else {
            return false;
        };
        let Some(cfg) = TelegramGroupConfig::load() else {
            return false;
        };
        // Only inside the configured supergroup.
        if msg.chat.id != cfg.group_id {
            return false;
        }
        let Some(project) = cfg.project_for_topic(thread_id) else {
            // Message in the General topic or an unmapped topic → fall through
            // to the AISB Master brain.
            return false;
        };
        let group_id = cfg.group_id;

        // Find a live oracle for this project (oracle-<project> or
        // oracle-<project>-N).
        let sessions = self.mgr.list_sessions().await.unwrap_or_default();
        let exact = format!("oracle-{}", project);
        let prefix = format!("oracle-{}-", project);
        let live = sessions
            .iter()
            .find(|s| s.name == exact || s.name.starts_with(&prefix))
            .map(|s| s.name.clone());

        match live {
            Some(session) => {
                // Continue the conversation in the existing oracle pane.
                // paste-then-submit so multi-line input lands as one message.
                let _ = self.mgr.send_paste_then_submit(&session, text).await;
                let _ = self
                    .send_html_to_topic(
                        group_id,
                        thread_id,
                        &format!("→ <code>{}</code>", formatting::escape_html(&session)),
                    )
                    .await;
            }
            None => {
                let oracle_name = format!("oracle-{}-1", project);
                self.spawn_topic_oracle(&project, &oracle_name, text, group_id, thread_id)
                    .await;
            }
        }
        true
    }

    /// Spawn a fresh oracle for a project topic and hand it its first message.
    /// Unlike `spawn_project_oracle` (DM-centric, sets a one-shot target +
    /// "next message goes there" ack), this is topic-native: the topic IS the
    /// oracle's conversation, so we feed the first message straight in and ack
    /// in the topic. The mission is amplified into a structured brief first
    /// (skip-gated — short follow-ups pass through verbatim).
    async fn spawn_topic_oracle(
        &self,
        project: &str,
        oracle_name: &str,
        first_message: &str,
        group_id: i64,
        thread_id: i64,
    ) {
        let registry = omega_core::project_manager::ProjectRegistry::load();
        let Some(entry) = registry.projects.iter().find(|p| p.name == *project) else {
            let _ = self
                .send_html_to_topic(
                    group_id,
                    thread_id,
                    &format!(
                        "<i>Project <code>{}</code> not in registry — can't spawn an oracle.</i>",
                        formatting::escape_html(project)
                    ),
                )
                .await;
            return;
        };
        let cwd = entry.path.display().to_string();
        let prompt_file = render_oracle_prompt(project, &cwd, oracle_name);
        let mut cmd = String::from("claude --dangerously-skip-permissions");
        if let Some(ref pf) = prompt_file {
            cmd.push_str(&format!(
                " --append-system-prompt-file '{}'",
                pf.replace('\'', r"'\''")
            ));
        }
        let wrapped = format!("bash -c '{}; exec bash'", cmd.replace('\'', r"'\''"));
        if let Err(e) = self
            .mgr
            .create_session(oracle_name, Some(&cwd), Some(&wrapped))
            .await
        {
            let _ = self
                .send_html_to_topic(
                    group_id,
                    thread_id,
                    &format!(
                        "<i>Spawn failed: <code>{}</code></i>",
                        formatting::escape_html(&e.to_string())
                    ),
                )
                .await;
            return;
        }

        let _ = self
            .send_html_to_topic(
                group_id,
                thread_id,
                &format!(
                    "✦ Spawned <code>{}</code> in <code>{}</code>. Working on it…",
                    formatting::escape_html(oracle_name),
                    formatting::escape_html(&cwd)
                ),
            )
            .await;

        // Amplify the first message into a structured brief (skip-gated).
        // Runs a blocking subprocess → spawn_blocking so the async runtime
        // isn't stalled. Falls back to raw text if the pass fails.
        let brief = {
            let raw = first_message.to_string();
            let proj = project.to_string();
            let wd = cwd.clone();
            tokio::task::spawn_blocking(move || {
                omega_core::amplify::amplify_mission(&raw, &proj, &wd)
            })
            .await
            .unwrap_or_else(|_| first_message.to_string())
        };

        // The interactive `claude` REPL needs to boot before it can accept
        // piped input; pasting too early loses the keystrokes. Instead of a
        // fixed sleep (which races a slow boot and silently drops the mission),
        // poll the pane until it STABILIZES (two consecutive equal non-empty
        // captures = REPL ready). Never paste early; bounded ~30s; ~4s on a
        // fast boot. Best-effort: if it never settles in budget, paste anyway.
        {
            let mut prev = String::new();
            for _ in 0..15 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                match self.mgr.capture_pane(oracle_name).await {
                    Ok(cur) if !cur.trim().is_empty() && cur == prev => break,
                    Ok(cur) => prev = cur,
                    Err(_) => {}
                }
            }
        }
        let _ = self.mgr.send_paste_then_submit(oracle_name, &brief).await;
    }

    /// Handle `proj:*` callbacks (open / new / scan / delete).
    async fn handle_project_callback(&self, chat_id: i64, rest: &str) {
        let (sub, arg) = rest.split_once(':').unwrap_or((rest, ""));
        match sub {
            "menu" => {
                self.send_projects_menu(chat_id).await;
            }
            "open" => {
                // Project detail card — actions for this project.
                let project = arg;
                let keyboard = vec![
                    vec![InlineKeyboardButton {
                        text: "💬 Talk to oracle".to_string(),
                        callback_data: format!("proj:oracle:{}", project),
                    }],
                    vec![InlineKeyboardButton {
                        text: "🗑 Delete project".to_string(),
                        callback_data: format!("proj:del:{}", project),
                    }],
                    vec![InlineKeyboardButton {
                        text: "‹ Projects".to_string(),
                        callback_data: "proj:menu".to_string(),
                    }],
                ];
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": format!(
                        "<b>{}</b>\n\n<i>Choose an action:</i>",
                        formatting::escape_html(project)
                    ),
                    "parse_mode": "HTML",
                    "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
                });
                let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
                self.client.post(&url).json(&payload).send().await.ok();
            }
            "oracle" => {
                // Smart oracle routing:
                // - 0 oracles exist for this project → spawn oracle-{project}-1
                // - 1+ oracles exist → ask user: continue existing N or spawn new
                let project = arg;
                let sessions = self.mgr.list_sessions().await.unwrap_or_default();
                let prefix = format!("oracle-{}-", project);
                let mut existing: Vec<String> = sessions
                    .iter()
                    .filter(|s| s.name.starts_with(&prefix))
                    .map(|s| s.name.clone())
                    .collect();
                existing.sort();

                if existing.is_empty() {
                    // No oracle yet → spawn oracle-{project}-1
                    let oracle_name = format!("oracle-{}-1", project);
                    self.spawn_project_oracle(chat_id, project, &oracle_name).await;
                } else {
                    // Show buttons: each existing oracle + "new oracle" option
                    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                    for name in &existing {
                        keyboard.push(vec![InlineKeyboardButton {
                            text: format!("Continue {}", name),
                            callback_data: format!("proj:talkto:{}", name),
                        }]);
                    }
                    let next_idx = existing.len() + 1;
                    let new_oracle = format!("oracle-{}-{}", project, next_idx);
                    keyboard.push(vec![InlineKeyboardButton {
                        text: format!("+ Spawn new ({})", new_oracle),
                        callback_data: format!("proj:spawn:{}|{}", project, new_oracle),
                    }]);
                    let payload = serde_json::json!({
                        "chat_id": chat_id,
                        "text": format!(
                            "<b>{}</b> — {} oracle(s) running\n\n<i>Continue one of them or spawn a new one:</i>",
                            formatting::escape_html(project),
                            existing.len()
                        ),
                        "parse_mode": "HTML",
                        "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
                    });
                    let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
                    self.client.post(&url).json(&payload).send().await.ok();
                }
            }
            "del" => {
                let project = arg;
                let keyboard = vec![
                    vec![InlineKeyboardButton {
                        text: "✅ Confirm delete".to_string(),
                        callback_data: format!("proj:delyes:{}", project),
                    }],
                    vec![InlineKeyboardButton {
                        text: "Cancel".to_string(),
                        callback_data: format!("proj:open:{}", project),
                    }],
                ];
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": format!(
                        "🗑 <b>Delete {}?</b>\n\n\
                        This will remove the project from Omega's registry \
                        AND delete its Telegram topic.\n\n\
                        <i>The project's source files on disk are NOT deleted.</i>",
                        formatting::escape_html(project)
                    ),
                    "parse_mode": "HTML",
                    "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
                });
                let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
                self.client.post(&url).json(&payload).send().await.ok();
            }
            "delyes" => {
                use omega_core::project_manager::ProjectRegistry;
                use omega_core::telegram_group::TelegramGroupConfig;
                let project = arg;

                // 1. Remove the topic mapping + delete the forum topic (if any).
                let mut topic_deleted = false;
                if let Some(mut cfg) = TelegramGroupConfig::load() {
                    if let Some(tid) = cfg.remove_topic(project) {
                        topic_deleted = self.delete_forum_topic(cfg.group_id, tid).await;
                        let _ = cfg.save();
                    }
                }

                // 2. Remove from the project registry.
                let mut reg = ProjectRegistry::load();
                let removed = reg.remove(project);
                let _ = reg.save();

                // 3. Report.
                let msg = if !removed && !topic_deleted {
                    format!(
                        "⚠️ <b>{}</b> not found in registry and had no topic.",
                        formatting::escape_html(project)
                    )
                } else {
                    let mut parts: Vec<String> = Vec::new();
                    if removed {
                        parts.push("removed from Omega registry".to_string());
                    }
                    if topic_deleted {
                        parts.push("Telegram topic deleted".to_string());
                    }
                    format!(
                        "🗑 <b>{}</b> — {}.\n<i>Source files on disk were left untouched.</i>",
                        formatting::escape_html(project),
                        parts.join(" + ")
                    )
                };
                self.send_html(chat_id, &msg).await.ok();
                self.send_projects_menu(chat_id).await;
            }
            "talkto" => {
                // User chose to continue an existing oracle
                *self.targeted_session.lock().await = Some(arg.to_string());
                self.send_html(
                    chat_id,
                    &format!(
                        "Targeting <code>{}</code>.\nNext message goes there. /cancel to clear.",
                        formatting::escape_html(arg)
                    ),
                ).await.ok();
            }
            "spawn" => {
                // arg = "project|oracle_name"
                let (project, oracle_name) = arg.split_once('|').unwrap_or((arg, ""));
                if oracle_name.is_empty() {
                    return;
                }
                self.spawn_project_oracle(chat_id, project, oracle_name).await;
            }
            "new" => {
                // Show location buttons for new project
                let keyboard = vec![
                    vec![InlineKeyboardButton { text: "work/".to_string(), callback_data: "proj:newin:work".to_string() }],
                    vec![InlineKeyboardButton { text: "clients/".to_string(), callback_data: "proj:newin:clients".to_string() }],
                ];
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": "<b>New Project</b>\nWhere should it live?\n\n<i>After choosing, send the project name as your next message.</i>",
                    "parse_mode": "HTML",
                    "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
                });
                let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
                self.client.post(&url).json(&payload).send().await.ok();
            }
            "newin" => {
                // Remember the location, await the project name
                *self.targeted_session.lock().await = Some(format!("__newproject__:{}", arg));
                self.send_html(
                    chat_id,
                    &format!("Location: <code>{}/</code>\nSend the project name now.", formatting::escape_html(arg)),
                ).await.ok();
            }
            "scan" => {
                self.scan_and_propose_projects(chat_id).await;
            }
            "add" => {
                // Add a scanned project to the registry
                self.add_scanned_project(chat_id, arg).await;
            }
            _ => {}
        }
    }

    /// Scan VibeCoding dirs for git projects not yet in the registry.
    async fn scan_and_propose_projects(&self, chat_id: i64) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let registry = omega_core::project_manager::ProjectRegistry::load();
        let known: std::collections::HashSet<String> =
            registry.projects.iter().map(|p| p.path.display().to_string()).collect();

        let mut found: Vec<(String, String)> = Vec::new(); // (name, path)
        for sub in ["VibeCoding/work", "VibeCoding/clients"] {
            let base = home.join(sub);
            if let Ok(entries) = std::fs::read_dir(&base) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() && p.join(".git").exists() {
                        let path_str = p.display().to_string();
                        if !known.contains(&path_str) {
                            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                            found.push((name, path_str));
                        }
                    }
                }
            }
        }

        if found.is_empty() {
            self.send_html(chat_id, "<b>Scan complete</b>\nNo new git projects found (all are already registered).").await.ok();
            return;
        }

        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        for (name, _path) in found.iter().take(20) {
            keyboard.push(vec![InlineKeyboardButton {
                text: format!("+ {}", name),
                callback_data: format!("proj:add:{}", name),
            }]);
        }
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": format!("<b>Scan complete</b>\nFound {} unregistered project(s). Tap to add:", found.len()),
            "parse_mode": "HTML",
            "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
        });
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        self.client.post(&url).json(&payload).send().await.ok();
    }

    /// Register a scanned project into the registry.
    async fn add_scanned_project(&self, chat_id: i64, name: &str) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        // Find it in work or clients
        for sub in ["VibeCoding/work", "VibeCoding/clients"] {
            let p = home.join(sub).join(name);
            if p.is_dir() {
                match omega_core::project_manager::add_existing_project(&p) {
                    Ok(_) => {
                        self.send_html(
                            chat_id,
                            &format!("Added <b>{}</b> to the registry.\nUse /projects to see it.", formatting::escape_html(name)),
                        ).await.ok();
                    }
                    Err(e) => {
                        self.send_html(chat_id, &format!("Failed to add: <code>{}</code>", formatting::escape_html(&e.to_string()))).await.ok();
                    }
                }
                return;
            }
        }
        self.send_html(chat_id, &format!("Project <code>{}</code> not found on disk.", formatting::escape_html(name))).await.ok();
    }

    /// Handle `sess:*` callbacks (target a session for the next message).
    async fn handle_session_callback(&self, chat_id: i64, rest: &str) {
        if let Some(session) = rest.strip_prefix("target:") {
            *self.targeted_session.lock().await = Some(session.to_string());
            self.send_html(
                chat_id,
                &format!(
                    "Targeting <code>{}</code>.\nYour next message will be sent there. Send <code>/cancel</code> to clear.",
                    formatting::escape_html(session)
                ),
            ).await.ok();
        }
    }

    /// Handle `model:*` callbacks (switch provider/model).
    async fn handle_model_callback(&self, chat_id: i64, rest: &str) {
        // rest = "provider" or "provider:model"
        let (provider, model) = rest.split_once(':').unwrap_or((rest, ""));
        // Persist active selection to ~/.omega/state/telegram-active-model.json
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let state_path = home.join(".omega/state/telegram-active-model.json");
        if let Some(parent) = state_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let selection = serde_json::json!({
            "active_provider": provider,
            "active_model": model,
        });
        let _ = std::fs::write(&state_path, serde_json::to_string_pretty(&selection).unwrap_or_default());
        self.send_html(
            chat_id,
            &format!(
                "Active model set: <b>{}</b>{}",
                formatting::escape_html(provider),
                if model.is_empty() { String::new() } else { format!(" / <code>{}</code>", formatting::escape_html(model)) }
            ),
        ).await.ok();
    }

    /// `/projects` — interactive menu listing existing projects + actions.
    async fn send_projects_menu(&self, chat_id: i64) {
        let registry = omega_core::project_manager::ProjectRegistry::load();
        let mut text = String::from("<b>Projects</b>\n");

        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        if registry.projects.is_empty() {
            text.push_str("\n<i>No projects registered yet.</i>\n");
        } else {
            text.push_str(&format!("\n<i>{} project(s) registered:</i>\n", registry.projects.len()));
            for p in registry.projects.iter().take(20) {
                keyboard.push(vec![InlineKeyboardButton {
                    text: format!("{} {}", p.icon.as_deref().unwrap_or("·"), p.name),
                    callback_data: format!("proj:open:{}", p.name),
                }]);
            }
        }
        // Action row
        keyboard.push(vec![
            InlineKeyboardButton { text: "+ New project".to_string(), callback_data: "proj:new".to_string() },
            InlineKeyboardButton { text: "Scan & add existing".to_string(), callback_data: "proj:scan".to_string() },
        ]);

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
        });
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let _ = self.client.post(&url).json(&payload).send().await;
    }

    /// `/sessions` or `/relay` — interactive menu listing active rmux sessions.
    async fn send_sessions_menu(&self, chat_id: i64) {
        let all_sessions = self.mgr.list_sessions().await.unwrap_or_default();
        // Hide infra daemons — they're not for the user to interact with.
        let hidden_prefixes = ["omega-telegram-bridge", "aisb-reauth"];
        let sessions: Vec<_> = all_sessions
            .into_iter()
            .filter(|s| !hidden_prefixes.iter().any(|p| s.name.starts_with(p)))
            .collect();

        let mut text = String::from("<b>Active Sessions</b>\n");
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        if sessions.is_empty() {
            text.push_str("\n<i>No active sessions.</i>");
        } else {
            text.push_str(&format!("\n<i>{} session(s) — tap one to send a message:</i>\n", sessions.len()));
            for s in sessions.iter().take(20) {
                let label = match s.role {
                    omega_core::session::SessionRole::Oracle => format!("[oracle] {}", s.name),
                    omega_core::session::SessionRole::Worker => format!("[worker] {}", s.name),
                    omega_core::session::SessionRole::Home => format!("[home] {}", s.name),
                    omega_core::session::SessionRole::System => format!("[system] {}", s.name),
                };
                keyboard.push(vec![InlineKeyboardButton {
                    text: label,
                    callback_data: format!("sess:target:{}", s.name),
                }]);
            }
        }

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "reply_markup": InlineKeyboardMarkup { inline_keyboard: keyboard },
        });
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let _ = self.client.post(&url).json(&payload).send().await;
    }

    /// `/account` — bare form: show legacy Claude card (kept for compat).
    /// With args: multi-provider account management.
    ///   /account <provider>                list accounts for provider
    ///   /account <provider> <name>         switch to that account
    ///   /account add <provider> <name>     save current creds as named account
    async fn handle_account_command(&self, chat_id: i64, text: &str) {
        let parts: Vec<&str> = text.split_whitespace().skip(1).collect();
        if parts.is_empty() {
            self.send_account_card(chat_id).await;
            return;
        }

        // /account add <provider> <name>
        if parts[0].eq_ignore_ascii_case("add") {
            if parts.len() < 3 {
                let _ = self
                    .send_html(
                        chat_id,
                        "Usage: <code>/account add &lt;provider&gt; &lt;name&gt;</code>",
                    )
                    .await;
                return;
            }
            let provider = parts[1];
            let name = parts[2];
            if !ProvidersConfig::is_known(provider) {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            "Unknown provider <code>{}</code>",
                            formatting::escape_html(provider)
                        ),
                    )
                    .await;
                return;
            }
            let store = match CredentialStore::new() {
                Ok(s) => s,
                Err(e) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Credential store error</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                    return;
                }
            };
            match store.save_as_account(provider, name) {
                Ok(_) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Saved</b> current <code>{}</code> credentials as account <code>{}</code>",
                                formatting::escape_html(provider),
                                formatting::escape_html(name),
                            ),
                        )
                        .await;
                }
                Err(e) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Save failed</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                }
            }
            return;
        }

        let provider = parts[0];
        if !ProvidersConfig::is_known(provider) {
            let _ = self
                .send_html(
                    chat_id,
                    &format!(
                        "Unknown provider <code>{}</code>\nKnown: <code>{}</code>",
                        formatting::escape_html(provider),
                        formatting::escape_html(&ProvidersConfig::all_providers().join(", ")),
                    ),
                )
                .await;
            return;
        }

        let store = match CredentialStore::new() {
            Ok(s) => s,
            Err(e) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            "<b>Credential store error</b>\n<code>{}</code>",
                            formatting::escape_html(&e.to_string())
                        ),
                    )
                    .await;
                return;
            }
        };

        if let Some(name) = parts.get(1).copied() {
            // /account <provider> <name>  — switch
            match store.switch_account(provider, name) {
                Ok(_) => {
                    let _ = store.ensure_legacy_symlink(provider);
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Switched</b> <code>{}</code> → <code>{}</code>",
                                formatting::escape_html(provider),
                                formatting::escape_html(name),
                            ),
                        )
                        .await;
                }
                Err(e) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Switch failed</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                }
            }
            return;
        }

        // /account <provider>  — list
        let accounts = store.list_accounts(provider);
        let active_exists = store.active_path(provider).exists();
        let mut lines = vec![format!(
            "<b>{} accounts</b>",
            formatting::escape_html(provider)
        )];
        lines.push(format!(
            "Active credentials: <code>{}</code>",
            if active_exists { "present" } else { "missing" }
        ));
        if accounts.is_empty() {
            lines.push(
                "<i>No saved accounts.</i>\nSave current with: <code>/account add {provider} &lt;name&gt;</code>"
                    .to_string(),
            );
        } else {
            lines.push("<b>Saved:</b>".to_string());
            for a in &accounts {
                lines.push(format!("  <code>{}</code>", formatting::escape_html(a)));
            }
            lines.push(String::new());
            lines.push(format!(
                "Switch: <code>/account {} &lt;name&gt;</code>",
                formatting::escape_html(provider)
            ));
        }
        let _ = self.send_html(chat_id, &lines.join("\n")).await;
    }

    /// Handle /newproject <name> <work|clients> [emoji] — creates project + registers it
    async fn handle_newproject(&self, chat_id: i64, text: &str) {
        let parts: Vec<&str> = text.split_whitespace().skip(1).collect();
        if parts.len() < 2 {
            let _ = self.send_html(
                chat_id,
                "<b>New Project</b>\n\
                 Usage: <code>/newproject &lt;name&gt; &lt;work|clients&gt;</code>\n\n\
                 <b>Example:</b>\n  <code>/newproject MyApp work</code>\n\
                 <code>/newproject ClientX clients</code>",
            ).await;
            return;
        }
        let name = parts[0];
        let location = parts[1].to_lowercase();
        let icon = parts.get(2).copied().unwrap_or("");

        if location != "work" && location != "clients" {
            let _ = self.send_html(
                chat_id,
                &format!(
                    "Location must be <code>work</code> or <code>clients</code>, got <code>{}</code>",
                    formatting::escape_html(&location)
                ),
            ).await;
            return;
        }

        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let base = home.join("VibeCoding").join(&location);
        if let Err(e) = std::fs::create_dir_all(&base) {
            let _ = self.send_html(
                chat_id,
                &format!("Could not create base dir: <code>{}</code>", formatting::escape_html(&e.to_string()))
            ).await;
            return;
        }
        let icon_opt = if icon.is_empty() { None } else { Some(icon) };
        let result = omega_core::project_manager::create_project(name, &base, icon_opt);
        match result {
            Ok(project) => {
                let _ = self.send_html(
                    chat_id,
                    &format!(
                        "<b>Project Created</b>\n\
                         <b>Name:</b> {}\n\
                         <b>Path:</b> <code>{}</code>\n\
                         <b>Oracle:</b> <code>oracle-{}</code>\n\n\
                         <i>Next: send a message to start working on this project.</i>",
                        formatting::escape_html(&project.name),
                        formatting::escape_html(&project.path.display().to_string()),
                        formatting::escape_html(&project.name),
                    ),
                ).await;
            }
            Err(e) => {
                let _ = self.send_html(
                    chat_id,
                    &format!(
                        "<b>Project creation failed</b>\n<code>{}</code>",
                        formatting::escape_html(&e.to_string())
                    ),
                ).await;
            }
        }
    }

    /// Dispatch `acc:*` callback subactions.
    async fn handle_account_callback(&self, chat_id: i64, rest: &str) {
        let (sub, arg) = rest.split_once(':').unwrap_or((rest, ""));
        match sub {
            "show" => self.send_account_card(chat_id).await,
            "billing" => self.send_billing_card(chat_id).await,
            "login" => self.start_login_flow(chat_id, "User-initiated reauth").await,
            "logout_confirm" => self.send_logout_confirmation(chat_id).await,
            "cancel" => {
                let _ = self
                    .send_html(chat_id, " <i>Cancelled.</i>")
                    .await;
            }
            "logout" => match account::logout() {
                Ok(_) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            "<b>Logged out</b>\n<i>Credentials backed up to \
                             <code>.credentials.json.previous</code>. Use /login \
                             to re-authenticate.</i>",
                        )
                        .await;
                }
                Err(e) => {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                " <b>Logout failed</b>\n<code>{}</code>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                }
            },
            "switch" => {
                if arg.is_empty() {
                    let _ = self
                        .send_html(chat_id, " <i>No account name supplied.</i>")
                        .await;
                    return;
                }
                let result = account::switch_account(arg);
                if result.ok {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<b>Switched</b> → <code>{}</code>\n\
                                 <b>Email:</b> <code>{}</code>\n\
                                 <b>Expires:</b> <code>{} min</code>",
                                formatting::escape_html(&result.label),
                                formatting::escape_html(&result.email),
                                result.expires_min,
                            ),
                        )
                        .await;
                } else if result.method == "needs_reauth" {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                " <b>Refresh failed</b> for <code>{}</code>\n\
                                 <i>Need a full OAuth reauth — starting now…</i>",
                                formatting::escape_html(&result.label),
                            ),
                        )
                        .await;
                    self.start_login_flow(chat_id, &format!("Switch to {}", result.label))
                        .await;
                } else {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                " <b>Switch failed</b>\n<code>{}</code>",
                                formatting::escape_html(result.error.as_deref().unwrap_or("unknown"))
                            ),
                        )
                        .await;
                }
            }
            _ => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            " Unknown account action: <code>{}</code>",
                            formatting::escape_html(sub)
                        ),
                    )
                    .await;
            }
        }
    }

    /// Render the account card with [Login | Switch | Billing | Logout] inline
    /// buttons.
    async fn send_account_card(&self, chat_id: i64) {
        let current = CurrentAccount::read();
        let accounts = account::list_accounts();
        let billing = account::get_billing();

        let status_icon = if current.valid && !current.warning {
            ""
        } else if current.warning {
            ""
        } else {
            ""
        };
        let status_line = if current.valid {
            format!("{} min remaining", current.expires_min)
        } else {
            "expired".to_string()
        };

        let label = current
            .label
            .as_deref()
            .or(current.name.as_deref())
            .unwrap_or("?");

        let _ = label; // profile label dropped from the card — not useful
        let mut html = format!(
            "<b>Account</b>\n\n\
             <b>Email:</b>   <code>{}</code>\n\
             <b>Token:</b>   {} {}\n\
             <b>Tier:</b>    <code>{}</code>\n\
             <b>Plan:</b>    <code>{}</code>",
            formatting::escape_html(&current.email),
            status_icon,
            formatting::escape_html(&status_line),
            formatting::escape_html(&current.tier),
            formatting::escape_html(&current.subscription),
        );

        if let Some(b) = &billing {
            html.push_str(&format!(
                "\n\n<b>Usage</b>\n\
                 <b>5h:</b>   <code>{:.1}%</code>\n\
                 <b>Week:</b> <code>{:.1}%</code>",
                b.precise_5h(),
                b.precise_week(),
            ));
        }

        if !accounts.is_empty() {
            html.push_str("\n\n<b>Saved profiles:</b>\n");
            for a in &accounts {
                let marker = if a.is_active { "● " } else { "  " };
                html.push_str(&format!(
                    "{}<code>{}</code> — {}\n",
                    marker,
                    formatting::escape_html(&a.name),
                    formatting::escape_html(&a.email),
                ));
            }
        }

        let mut rows: Vec<Vec<InlineKeyboardButton>> = vec![vec![
            InlineKeyboardButton {
                text: "Login".to_string(),
                callback_data: "acc:login".to_string(),
            },
            InlineKeyboardButton {
                text: "Billing".to_string(),
                callback_data: "acc:billing".to_string(),
            },
        ]];

        // Add a switch row per saved profile (up to 4).
        for a in accounts.iter().take(4) {
            if a.is_active {
                continue;
            }
            rows.push(vec![InlineKeyboardButton {
                text: format!("↻ Switch → {}", a.label),
                callback_data: format!("acc:switch:{}", a.name),
            }]);
        }

        rows.push(vec![InlineKeyboardButton {
            text: "Logout".to_string(),
            callback_data: "acc:logout_confirm".to_string(),
        }]);

        let keyboard = InlineKeyboardMarkup {
            inline_keyboard: rows,
        };

        let _ = self.send_html_with_keyboard(chat_id, &html, &keyboard).await;
    }

    async fn send_logout_confirmation(&self, chat_id: i64) {
        let current = CurrentAccount::read();
        let label = current
            .label
            .as_deref()
            .or(current.name.as_deref())
            .unwrap_or("?");
        let html = format!(
            "<b>Confirm logout</b>\n\n\
             <b>Account:</b> <code>{}</code>\n<b>Email:</b>   <code>{}</code>\n\n\
             <i>Logging out backs up <code>.credentials.json.previous</code> \
             and removes the live credentials. New sessions will need /login.</i>",
            formatting::escape_html(label),
            formatting::escape_html(&current.email),
        );
        let keyboard = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![
                InlineKeyboardButton {
                    text: "Confirm logout".to_string(),
                    callback_data: "acc:logout".to_string(),
                },
                InlineKeyboardButton {
                    text: " Cancel".to_string(),
                    callback_data: "acc:cancel".to_string(),
                },
            ]],
        };
        let _ = self.send_html_with_keyboard(chat_id, &html, &keyboard).await;
    }

    async fn send_billing_card(&self, chat_id: i64) {
        let Some(snap) = account::get_billing() else {
            let _ = self
                .send_html(
                    chat_id,
                    " <i>No billing snapshot available (<code>/tmp/aisb-usage.json</code> missing).</i>",
                )
                .await;
            return;
        };
        // Email is not in usage.json — pull it from `claude auth status`.
        let email = if !snap.email.is_empty() && snap.email != "?" {
            snap.email.clone()
        } else {
            account::email_from_claude_auth_status()
        };
        let html = format!(
            "<b>Billing</b>\n\n\
             <b>5h:</b>    <code>{:.1}%</code>\n\
             <b>Week:</b>  <code>{:.1}%</code>\n\
             <b>Email:</b>   <code>{}</code>",
            snap.precise_5h(),
            snap.precise_week(),
            formatting::escape_html(&email),
        );
        // Suppress the "agentikos" internal account label — useless to user
        let _ = &snap.active_account;
        let keyboard = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![
                InlineKeyboardButton {
                    text: "↻ Refresh".to_string(),
                    callback_data: "acc:billing".to_string(),
                },
                InlineKeyboardButton {
                    text: "Account".to_string(),
                    callback_data: "acc:show".to_string(),
                },
            ]],
        };
        let _ = self.send_html_with_keyboard(chat_id, &html, &keyboard).await;
    }

    /// Start the OAuth reauth flow: spawn the rmux session, capture the URL,
    /// send it to the user.
    async fn start_login_flow(&self, chat_id: i64, reason: &str) {
        let _ = self
            .send_html(
                chat_id,
                " <i>Spawning reauth session… this takes ~15s.</i>",
            )
            .await;

        match oauth::request_reauth(&self.mgr, reason, None).await {
            Ok(Some(req)) => {
                let html = format!(
                    "<b>Auth Required</b>\n\n\
                     <b>Reason:</b> {}\n\n\
                     1. <a href=\"{}\">Open this URL</a> and authorize\n\
                     2. Copy the code from the callback page\n\
                     3. Paste it back here — auto-detected\n\n\
                     <i>The reauth session is waiting in <code>aisb-reauth</code>.</i>",
                    formatting::escape_html(reason),
                    req.auth_url,
                );
                let _ = self.send_html(chat_id, &html).await;
            }
            Ok(None) => {
                let _ = self
                    .send_html(
                        chat_id,
                        " <i>A reauth is already pending or just attempted. \
                         Wait 30s or check the <code>aisb-reauth</code> session.</i>",
                    )
                    .await;
            }
            Err(e) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            " <b>Login failed</b>\n<code>{}</code>",
                            formatting::escape_html(&e.to_string())
                        ),
                    )
                    .await;
            }
        }
    }

    /// Process a pasted OAuth code by handing it off to the reauth session.
    async fn handle_oauth_code(&self, chat_id: i64, code: &str) {
        let _ = self
            .send_html(chat_id, " <i>Exchanging code for fresh credentials…</i>")
            .await;

        match oauth::handle_code(&self.mgr, code).await {
            Ok(res) if res.success => {
                let html = format!(
                    "<b>Authenticated</b>\n\n\
                     <b>Email:</b>   <code>{}</code>\n\
                     <b>Expires:</b> <code>{} min</code>\n\n\
                     <i>Credentials updated.</i>",
                    formatting::escape_html(&res.email),
                    res.expires_min,
                );
                let _ = self.send_html(chat_id, &html).await;
            }
            Ok(res) => {
                let tail = res
                    .pane_tail
                    .chars()
                    .rev()
                    .take(400)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            " <b>Auth failed</b>\n\n\
                             Credentials did not update. Last pane:\n<pre>{}</pre>",
                            formatting::escape_html(&tail)
                        ),
                    )
                    .await;
            }
            Err(e) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!(
                            " <b>Code paste error</b>\n<code>{}</code>",
                            formatting::escape_html(&e.to_string())
                        ),
                    )
                    .await;
            }
        }
    }

    /// Register the bot's slash commands with Telegram so they appear
    /// in the autocomplete menu when typing "/" in the chat.
    async fn register_bot_commands(&self) {
        let commands = serde_json::json!({
            "commands": [
                {"command": "help",     "description": "Show available commands"},
                {"command": "account",  "description": "Account / billing / login (with buttons)"},
                {"command": "model",    "description": "Switch AI provider and model"},
                {"command": "projects", "description": "List projects + new / add existing"},
                {"command": "sessions", "description": "Active sessions (tap to target)"},
                {"command": "clean",    "description": "Restart AISB Master fresh (clean slate)"},
                {"command": "setupgroup","description": "Register a supergroup as the project hub (+ create per-project topics)"},
                {"command": "status",   "description": "Live system dashboard (oracles, workers, done signals)"},
                {"command": "dispatch", "description": "Preview a structured brief and dispatch to a project oracle (with confirm button)"}
            ]
        });

        let url = format!("{}/bot{}/setMyCommands", API_BASE, self.cfg.bot_token);
        let _ = self.client.post(&url).json(&commands).send().await;
    }

    /// One-shot back-online card sent on bridge (re)start.
    async fn send_back_online_card(&self) {
        let all_sessions = self.mgr.list_sessions().await.unwrap_or_default();
        let hidden = ["omega-telegram-bridge", "aisb-reauth"];
        let sessions: Vec<_> = all_sessions
            .into_iter()
            .filter(|s| !hidden.iter().any(|p| s.name.starts_with(p)))
            .collect();
        let mut session_lines = Vec::new();
        for sess in sessions.iter().take(8) {
            let icon = match sess.role {
                omega_core::session::SessionRole::Oracle => "",
                omega_core::session::SessionRole::Worker => "",
                omega_core::session::SessionRole::Home => "",
                omega_core::session::SessionRole::System => "",
            };
            session_lines.push(format!(
                "  {} <code>{}</code>",
                icon,
                formatting::escape_html(&sess.name)
            ));
        }
        if session_lines.is_empty() {
            session_lines.push("  <i>No active sessions</i>".to_string());
        }

        let billing = account::get_billing();
        let billing_block = if let Some(b) = &billing {
            format!(
                "\n\n<b>Billing:</b>\n  5h: <code>{:.1}%</code>  ·  Week: <code>{:.1}%</code>\n  Account: <code>{}</code>",
                b.precise_5h(),
                b.precise_week(),
                formatting::escape_html(&b.active_account),
            )
        } else {
            "\n\n<b>Billing:</b> <i>cache missing</i>".to_string()
        };

        let html = format!(
            "<b>OmegaOS — Back Online</b>\n\n\
             <b>Active Sessions:</b>\n{}{}\n\n\
             <i>Type /help for commands. Reply to oracle reports to route \
             messages back to that project. Paste an OAuth code anytime — \
             auto-detected.</i>",
            session_lines.join("\n"),
            billing_block,
        );

        let _ = self.send_html(self.reply_chat_id, &html).await;
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
            " Stopped <b>{}</b> worker(s) for <b>{}</b>",
            killed,
            formatting::escape_html(project)
        )
    }

    async fn close_oracle(&self, project: &str) -> String {
        let oracle_session = format!("oracle-{}", project);
        match self.mgr.kill_session(&oracle_session).await {
            Ok(_) => format!(
                " Oracle <code>{}</code> closed",
                formatting::escape_html(&oracle_session)
            ),
            Err(e) => format!(
                " Could not close oracle: <code>{}</code>",
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
                    " <b>Full Report — {}</b>\n\n<pre>{}</pre>",
                    formatting::escape_html(project),
                    formatting::escape_html(&cleaned)
                )
            }
            Err(_) => format!(
                " Oracle <code>{}</code> not found",
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
                "<b>OmegaOS Bot Engine</b>\n\
                 \n\n\
                 <b>Core:</b>\n\
                 /help — this message\n\
                 /list — show all rmux sessions\n\
                 /status — live system dashboard (oracles, workers, done signals, group)\n\
                 /status <code>&lt;session&gt;</code> — capture the last 20 lines of that pane\n\
                 /dispatch <code>&lt;Project&gt; &lt;mission&gt;</code> — preview brief + confirm button → oracle\n\
                 /aisb <code>text</code> — send to AISB Master\n\
                 /relay <code>session text</code> — send to specific session\n\
                 /kill <code>session</code> — kill a session\n\n\
                 <b>Account &amp; Billing:</b>\n\
                 /account — account card + login/switch/billing\n\
                 /account <code>&lt;provider&gt;</code> — list saved accounts for that provider\n\
                 /account <code>&lt;provider&gt; &lt;name&gt;</code> — switch to that account\n\
                 /account add <code>&lt;provider&gt; &lt;name&gt;</code> — save current creds as named account\n\
                 /model — show current provider/model + list available\n\
                 /model <code>&lt;provider&gt; [model]</code> — switch active provider/model\n\
                 /login — start OAuth reauth flow\n\
                 /logout — clear active credentials\n\
                 /billing — current Claude usage\n\n\
                 <b>Skills &amp; Audits:</b>\n\
                 /skills — list available skills\n\
                 /audits — show Quality Arsenal status\n\n\
                 <i>Reply to any report to auto-route to that project.\n\
                 Any other message goes to AISB Master.\n\
                 Paste an OAuth code directly — auto-detected.</i>"
                    .to_string(),
            ),

            "/list" => {
                let sessions = self.mgr.list_sessions().await.ok()?;
                let mut lines = vec![" <b>Sessions</b>\n".to_string()];
                for sess in sessions {
                    let icon = match sess.role {
                        omega_core::session::SessionRole::Oracle => "",
                        omega_core::session::SessionRole::Worker => "",
                        omega_core::session::SessionRole::Home => "",
                        omega_core::session::SessionRole::System => "",
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

            "/status" => {
                // Two modes:
                //   /status            → real system dashboard (oracles + workers + bridge + group)
                //   /status <session>  → tail the last 20 lines of that pane (legacy behavior)
                if !rest.trim().is_empty() {
                    let session = rest.trim();
                    let content = self.mgr.capture_pane(session).await.ok()?;
                    let tail: Vec<&str> = content.lines().rev().take(20).collect();
                    let output: Vec<&str> = tail.into_iter().rev().collect();
                    let cleaned = clean_terminal_output(&output.join("\n"));
                    return Some(format!(
                        " <b>{}</b>\n<pre>{}</pre>",
                        formatting::escape_html(session),
                        formatting::escape_html(&cleaned)
                    ));
                }
                Some(self.render_status_dashboard().await)
            }

            "/skills" => {
                let registry = omega_core::skill_registry::SkillRegistry::discover_default();
                match registry {
                    Ok(mut reg) => {
                        reg.register_audits();
                        let skills = reg.list();
                        let mut lines = vec![format!(
                            " <b>Skills</b> ({})\n",
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
                    Err(_) => Some(" <i>Could not load skill registry</i>".to_string()),
                }
            }

            "/audits" => {
                let audits = omega_core::audit::all_audits();
                let mut lines = vec![format!(
                    " <b>Quality Arsenal</b> ({} audits)\n",
                    audits.len()
                )];
                for audit in &audits {
                    let ro = if audit.read_only { " " } else { "" };
                    lines.push(format!(
                        "  <code>/{}</code> — {} ({} phases, /{}){ro}",
                        audit.id, audit.description, audit.phases, audit.max_score
                    ));
                }
                Some(lines.join("\n"))
            }

            // /aisb removed — plain text already routes to master.
            // /relay handled via interactive button menu (see handle_account_command path).

            "/kill" => {
                if rest.is_empty() {
                    return Some("Usage: /kill <code>session</code>".to_string());
                }
                let session = rest.trim();
                match self.mgr.kill_session(session).await {
                    Ok(_) => Some(format!(
                        " Killed <code>{}</code>",
                        formatting::escape_html(session)
                    )),
                    Err(e) => Some(format!(
                        " Could not kill <code>{}</code>: {}",
                        formatting::escape_html(session),
                        formatting::escape_html(&e.to_string())
                    )),
                }
            }

            _ => None,
        }
    }

    // ── /status: live system dashboard ────────────────────────────────────

    /// Build the `/status` (no args) dashboard: bridge uptime, session
    /// counts, live oracles + their model/tokens, recent done.json signals,
    /// and group/topic configuration. All data is live — pulled from the
    /// real session list, the persisted group config, and the state dir.
    async fn render_status_dashboard(&self) -> String {
        use omega_core::session::SessionRole;
        let sessions = self.mgr.list_sessions().await.unwrap_or_default();
        let mut oracles = 0usize;
        let mut workers = 0usize;
        let mut master = 0usize;
        let mut system = 0usize;
        let mut oracle_lines: Vec<String> = Vec::new();
        for s in &sessions {
            match s.role {
                SessionRole::Oracle => {
                    oracles += 1;
                    let meta = omega_core::claude_meta::read_meta_for_session(&s.name)
                        .map(|m| {
                            format!(
                                " <i>· {} · {} tok</i>",
                                m.model,
                                omega_core::claude_meta::fmt_tokens(m.tokens)
                            )
                        })
                        .unwrap_or_default();
                    // Current activity preview — what the oracle is doing
                    // right now. Best-effort: skip the line on capture error.
                    let activity = self
                        .mgr
                        .capture_pane(&s.name)
                        .await
                        .ok()
                        .as_deref()
                        .and_then(|t| last_activity_line(t, 80));
                    // Last completion result, if any (the file convention drops
                    // the "oracle-" prefix that's already in the session name).
                    let state_dir_for_done = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("/home/hacker"))
                        .join(".omega/state");
                    let oracle_id = s.name.strip_prefix("oracle-").unwrap_or(&s.name);
                    let last_done = omega_core::done::OracleDoneSignal::read(
                        &state_dir_for_done,
                        oracle_id,
                    )
                    .ok()
                    .flatten();

                    oracle_lines.push(format!(
                        "  ◆ <code>{}</code>{}",
                        formatting::escape_html(&s.name),
                        meta
                    ));
                    if let Some(line) = activity {
                        oracle_lines.push(format!(
                            "    <i>↳ {}</i>",
                            formatting::escape_html(&line)
                        ));
                    }
                    if let Some(d) = last_done {
                        use omega_core::done::DoneStatus;
                        let (badge, label) = match d.status {
                            DoneStatus::DoneClean => ("✓", "done_clean"),
                            DoneStatus::Pending => ("⏳", "pending"),
                            DoneStatus::Failed => ("✗", "failed"),
                            DoneStatus::Blocked => ("⊘", "blocked"),
                        };
                        let summary = if d.summary.len() > 70 {
                            let mut s: String = d.summary.chars().take(69).collect();
                            s.push('…');
                            s
                        } else {
                            d.summary.clone()
                        };
                        let suffix = if summary.is_empty() {
                            String::new()
                        } else {
                            format!(" — {}", formatting::escape_html(&summary))
                        };
                        oracle_lines.push(format!(
                            "    <i>{} <b>{}</b>{}</i>",
                            badge, label, suffix
                        ));
                    }
                }
                SessionRole::Worker => workers += 1,
                SessionRole::Home => master += 1,
                SessionRole::System => system += 1,
            }
        }

        let uptime = {
            let secs = self.start_time.elapsed().as_secs();
            if secs >= 86_400 {
                format!("{}d {}h", secs / 86_400, (secs % 86_400) / 3_600)
            } else if secs >= 3_600 {
                format!("{}h {}m", secs / 3_600, (secs % 3_600) / 60)
            } else if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            }
        };

        // Recent done signals — last 3 by mtime under ~/.omega/state/.
        let state_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/home/hacker"))
            .join(".omega/state");
        let mut done_files: Vec<(std::time::SystemTime, std::path::PathBuf)> = std::fs::read_dir(&state_dir)
            .ok()
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        let name = p.file_name()?.to_string_lossy().to_string();
                        if !name.ends_with(".done.json") {
                            return None;
                        }
                        let m = e.metadata().ok()?.modified().ok()?;
                        Some((m, p))
                    })
                    .collect()
            })
            .unwrap_or_default();
        done_files.sort_by(|a, b| b.0.cmp(&a.0));
        let recent_done: Vec<String> = done_files
            .iter()
            .take(3)
            .map(|(_, p)| {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let label = name
                    .trim_end_matches(".done.json")
                    .trim_start_matches("worker-")
                    .trim_start_matches("oracle-")
                    .to_string();
                format!("  • <code>{}</code>", formatting::escape_html(&label))
            })
            .collect();

        let group_line = match omega_core::telegram_group::TelegramGroupConfig::load() {
            Some(cfg) => format!(
                "<b>Group:</b> <code>{}</code> · {} topic(s)",
                formatting::escape_html(&cfg.group_name),
                cfg.topics.len()
            ),
            None => "<b>Group:</b> <i>not configured</i> — send /setupgroup".to_string(),
        };

        let mut out = String::new();
        out.push_str("◆ <b>OmegaOS · system status</b>\n");
        out.push_str(&format!("<i>bridge up {}</i>\n\n", uptime));
        out.push_str(&format!(
            "<b>Sessions:</b> {} oracle(s) · {} worker(s) · {} master · {} system\n",
            oracles, workers, master, system
        ));
        if !oracle_lines.is_empty() {
            out.push('\n');
            out.push_str("<b>Live oracles</b>\n");
            out.push_str(&oracle_lines.join("\n"));
            out.push('\n');
        }
        if !recent_done.is_empty() {
            out.push_str("\n<b>Recent done signals</b>\n");
            out.push_str(&recent_done.join("\n"));
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&group_line);
        out
    }

    // ── /dispatch + confirmation buttons (Pack INTERACTION) ───────────────

    /// `/dispatch <project> <mission>` — amplify the mission into a
    /// structured brief, preview it back to the user with [✅ Dispatch] /
    /// [❌ Cancel] inline buttons. The actual oracle spawn happens only on
    /// confirm — human-in-the-loop dispatch.
    async fn handle_dispatch_command(
        &self,
        chat_id: i64,
        thread_id: Option<i64>,
        args: &str,
    ) {
        let args = args.trim();
        let usage = "Usage: <code>/dispatch &lt;Project&gt; &lt;mission&gt;</code>\nExample: <code>/dispatch DentistryGPT fix the login redirect loop</code>";
        let Some((project, mission)) = args.split_once(char::is_whitespace) else {
            let _ = self.send_html_smart(chat_id, thread_id, usage).await;
            return;
        };
        let project = project.trim();
        let mission = mission.trim();
        if project.is_empty() || mission.is_empty() {
            let _ = self.send_html_smart(chat_id, thread_id, usage).await;
            return;
        }

        let registry = omega_core::project_manager::ProjectRegistry::load();
        let Some(entry) = registry.projects.iter().find(|p| p.name == *project) else {
            let known: Vec<String> = registry
                .projects
                .iter()
                .take(10)
                .map(|p| format!("<code>{}</code>", formatting::escape_html(&p.name)))
                .collect();
            let _ = self
                .send_html_smart(
                    chat_id,
                    thread_id,
                    &format!(
                        "<i>Project <code>{}</code> not in registry.</i>\n\n<b>Known:</b> {}",
                        formatting::escape_html(project),
                        if known.is_empty() {
                            "<i>(none)</i>".to_string()
                        } else {
                            known.join(", ")
                        }
                    ),
                )
                .await;
            return;
        };
        let cwd = entry.path.display().to_string();

        // Amplify in a blocking pool — same path as topic dispatch.
        let brief = {
            let raw = mission.to_string();
            let proj = project.to_string();
            let wd = cwd.clone();
            tokio::task::spawn_blocking(move || {
                omega_core::amplify::amplify_mission(&raw, &proj, &wd)
            })
            .await
            .unwrap_or_else(|_| mission.to_string())
        };

        // Short token. UTC-nanos + rand-ish disambiguation; we just need it
        // unique per concurrent request and short enough for callback_data
        // (Telegram caps at 64 bytes).
        let token: String = {
            let n = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u128;
            const A: &[u8; 32] = b"0123456789abcdefghijklmnopqrstuv";
            let mut t = String::with_capacity(8);
            let mut v = n;
            for _ in 0..8 {
                t.push(A[(v & 31) as usize] as char);
                v >>= 5;
            }
            t
        };

        let pending = PendingDispatch {
            project: project.to_string(),
            work_dir: cwd.clone(),
            brief: brief.clone(),
            raw_mission: mission.to_string(),
            created_at: chrono::Utc::now(),
            requested_by_chat: chat_id,
        };
        {
            let mut guard = self.pending_dispatches.lock().await;
            // Sweep stale entries: a user who ran /dispatch but never tapped
            // a button leaves the brief in memory forever. 15-min TTL bounds
            // it without a background task — O(N) sweep where N is the
            // in-flight dispatch count (single-digit in practice).
            let now = chrono::Utc::now();
            guard.retain(|_, p| (now - p.created_at).num_minutes() < 15);
            guard.insert(token.clone(), pending);
        }

        // Truncate brief preview so the Telegram message stays readable.
        const PREVIEW_MAX: usize = 1800;
        let brief_preview = if brief.len() > PREVIEW_MAX {
            let mut t: String = brief.chars().take(PREVIEW_MAX).collect();
            t.push_str(
                "\n…\n<i>(brief truncated for preview — full text dispatched on confirm)</i>",
            );
            t
        } else {
            brief.clone()
        };

        let body = format!(
            "✦ <b>Dispatch preview</b> · <code>{}</code>\n<i>cwd: {}</i>\n\n<pre>{}</pre>\n\n<i>Tap to confirm — the oracle will be spawned and given this brief.</i>",
            formatting::escape_html(project),
            formatting::escape_html(&cwd),
            formatting::escape_html(&brief_preview)
        );
        let kb = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![
                InlineKeyboardButton {
                    text: format!("✅ Dispatch to {}", project),
                    callback_data: format!("dispatch:go:{}", token),
                },
                InlineKeyboardButton {
                    text: "❌ Cancel".to_string(),
                    callback_data: format!("dispatch:cancel:{}", token),
                },
            ]],
        };
        let mut payload = serde_json::json!({
            "chat_id": chat_id,
            "text": body,
            "parse_mode": "HTML",
            "reply_markup": kb,
        });
        // Keep the preview INSIDE the topic when the command was typed there.
        if let Some(tid) = thread_id {
            payload["message_thread_id"] = serde_json::json!(tid);
        }
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let _ = self.client.post(&url).json(&payload).send().await;
    }

    /// Handle a `dispatch:` callback (go / cancel). The message_id is the
    /// preview message we sent — we edit it in place to reflect the outcome.
    async fn handle_dispatch_callback(
        &self,
        chat_id: i64,
        message_id: i64,
        action_token: &str,
    ) {
        let Some((action, token)) = action_token.split_once(':') else {
            return;
        };
        let entry = self.pending_dispatches.lock().await.remove(token);
        let Some(pending) = entry else {
            let _ = self
                .edit_message_html(
                    chat_id,
                    message_id,
                    "<i>This dispatch is stale (already confirmed, cancelled, or the bridge restarted).</i>",
                )
                .await;
            return;
        };
        if pending.requested_by_chat != chat_id {
            // Cross-chat replay — silently drop, only the requester acts on it.
            return;
        }
        // Keep raw_mission + created_at as audit trail — used in log line.
        tracing::info!(
            project = %pending.project,
            action = %action,
            age_secs = (chrono::Utc::now() - pending.created_at).num_seconds(),
            raw = %pending.raw_mission,
            "dispatch confirmation"
        );

        match action {
            "cancel" => {
                let _ = self
                    .edit_message_html(
                        chat_id,
                        message_id,
                        &format!(
                            "✗ <b>Cancelled</b> — <code>{}</code> mission not dispatched.",
                            formatting::escape_html(&pending.project)
                        ),
                    )
                    .await;
            }
            "go" => {
                // Pick the next free oracle slot for this project.
                let sessions = self.mgr.list_sessions().await.unwrap_or_default();
                let prefix = format!("oracle-{}-", pending.project);
                let next_idx = sessions
                    .iter()
                    .filter_map(|s| {
                        s.name
                            .strip_prefix(&prefix)
                            .and_then(|t| t.parse::<u32>().ok())
                    })
                    .max()
                    .map(|n| n + 1)
                    .unwrap_or(1);
                let oracle_name = format!("oracle-{}-{}", pending.project, next_idx);

                let prompt_file =
                    render_oracle_prompt(&pending.project, &pending.work_dir, &oracle_name);
                let mut cmd = String::from("claude --dangerously-skip-permissions");
                if let Some(ref pf) = prompt_file {
                    cmd.push_str(&format!(
                        " --append-system-prompt-file '{}'",
                        pf.replace('\'', r"'\''")
                    ));
                }
                let wrapped = format!("bash -c '{}; exec bash'", cmd.replace('\'', r"'\''"));
                let spawn_res = self
                    .mgr
                    .create_session(&oracle_name, Some(&pending.work_dir), Some(&wrapped))
                    .await;
                if let Err(e) = spawn_res {
                    let _ = self
                        .edit_message_html(
                            chat_id,
                            message_id,
                            &format!(
                                "<i>Spawn failed: <code>{}</code></i>",
                                formatting::escape_html(&e.to_string())
                            ),
                        )
                        .await;
                    return;
                }
                let _ = self
                    .edit_message_html(
                        chat_id,
                        message_id,
                        &format!(
                            "✓ <b>Dispatched</b> → <code>{}</code>\n<i>brief streamed in; output mirrored via the topic / reports.</i>",
                            formatting::escape_html(&oracle_name)
                        ),
                    )
                    .await;
                // Let the claude REPL boot before pasting the brief.
                let oracle = oracle_name.clone();
                let brief = pending.brief.clone();
                let mgr = self.mgr.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
                    let _ = mgr.send_paste_then_submit(&oracle, &brief).await;
                });
            }
            _ => {}
        }
    }

    // ── Telegram API methods ──

    /// Send plain text (no parse_mode) — for raw agent output that may
    /// contain unescaped < > & characters.
    async fn send_text_plain(&self, chat_id: i64, text: &str) -> Result<()> {
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        let _ = self.client.post(&url).json(&body).send().await;
        Ok(())
    }

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

    /// Send an HTML message threaded as a reply to `reply_to`. The bot's
    /// answer renders with a "↪ user's message" header in Telegram so the
    /// conversation stays visually anchored (Pack CONTEXT).
    async fn send_html_reply(
        &self,
        chat_id: i64,
        text: &str,
        reply_to: Option<i64>,
    ) -> Result<Option<i64>> {
        let chunks = formatting::split_message(text, TELEGRAM_MAX_MSG_LEN);
        let mut last_msg_id = None;
        for (idx, chunk) in chunks.iter().enumerate() {
            let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
            let mut body = serde_json::json!({
                "chat_id": chat_id,
                "text": chunk,
                "parse_mode": "HTML",
            });
            if idx == 0 {
                if let Some(rid) = reply_to {
                    body["reply_parameters"] = serde_json::json!({
                        "message_id": rid,
                        "allow_sending_without_reply": true,
                    });
                }
            }
            if let Ok(resp) = self.client.post(&url).json(&body).send().await {
                if let Ok(parsed) = resp.json::<SendMessageResp>().await {
                    if let Some(m) = parsed.result {
                        last_msg_id = Some(m.message_id);
                    }
                }
            }
        }
        Ok(last_msg_id)
    }

    /// Set a reaction emoji on a user message (Pack INTERACTION).
    /// Used as an instant ack — bot replies with 👀 right when receiving,
    /// so the user knows the message landed even before the placeholder
    /// shows.
    async fn set_message_reaction(
        &self,
        chat_id: i64,
        message_id: i64,
        emoji: &str,
    ) -> Result<()> {
        let url = format!("{}/bot{}/setMessageReaction", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "reaction": [{ "type": "emoji", "emoji": emoji }],
            "is_big": false,
        });
        let _ = self.client.post(&url).json(&body).send().await;
        Ok(())
    }

    /// Send a byte payload as a document (Pack FILES). Used when an
    /// agent response is too large to inline (>2KB or many lines).
    /// `filename` ends up as the Telegram caption + download name.
    async fn send_document_bytes(
        &self,
        chat_id: i64,
        filename: &str,
        mime: &str,
        bytes: &[u8],
    ) -> Result<()> {
        let url = format!("{}/bot{}/sendDocument", API_BASE, self.cfg.bot_token);
        let part = reqwest::multipart::Part::bytes(bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime)
            .unwrap_or_else(|_| {
                reqwest::multipart::Part::bytes(bytes.to_vec()).file_name(filename.to_string())
            });
        let form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", part);
        let _ = self.client.post(&url).multipart(form).send().await;
        Ok(())
    }

    /// Edit a previously-sent HTML message in place. Used by the
    /// "Thinking…" placeholder pattern: send placeholder → run LLM →
    /// edit placeholder with the formatted answer (single message
    /// morphs from thinking → answer, no chat clutter).
    /// Mirror a Telegram exchange into the live conversation log that the
    /// rmux aisb-master session tails. The bot's SDK subprocess owns the
    /// real conversation; this log is a read-only stream so the user can
    /// WATCH the Telegram chat live by attaching to aisb-master in the TUI.
    /// Best-effort, fire-and-forget — never blocks the Telegram response.
    // ── L4: Telegram group + forum topics ──────────────────────────────

    /// Handle `/setupgroup <id>` — register a supergroup, verify the bot
    /// has access, auto-create one topic per registered project. Idempotent
    /// and re-runnable (overwrites the stored config).
    async fn handle_setup_group(&self, chat_id: i64, arg: &str) {
        use omega_core::telegram_group::TelegramGroupConfig;
        // In a group the command often arrives as `/setupgroup@BotName` — the
        // strip_prefix leaves a leading "@token". Drop it so it doesn't get
        // mis-parsed as a group_id.
        let arg = arg.trim();
        let arg = arg.strip_prefix('@').map_or(arg, |rest| {
            rest.split_whitespace().skip(1).next().unwrap_or("")
        });
        // Invoked INSIDE a group/supergroup with no explicit id → the chat we
        // are in IS the target group. Offer a one-tap confirm button. This is
        // the robust path: it never depends on catching the fragile
        // my_chat_member promotion event (which is lost if the bot was
        // promoted before this build was running).
        if arg.is_empty() && chat_id < 0 {
            let kb = InlineKeyboardMarkup {
                inline_keyboard: vec![vec![InlineKeyboardButton {
                    text: "✅ Set up this group for Omega".to_string(),
                    callback_data: format!("setupgroup:{}", chat_id),
                }]],
            };
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "<b>Set up this group for Omega?</b>\nI'll create one Topic per project and route each Oracle's reports to its Topic.\nTap to confirm 👇",
                "parse_mode": "HTML",
                "reply_markup": kb,
            });
            let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
            let _ = self.client.post(&url).json(&payload).send().await;
            return;
        }
        if arg.is_empty() {
            // DM with no arg → show current state / instructions.
            let body = match TelegramGroupConfig::load() {
                Some(cfg) => format!(
                    "Group: <code>{}</code>\nTopics: {}\nSet up at: {}\n\n<i>To (re)configure: send <code>/setupgroup</code> INSIDE the supergroup and tap the confirm button.</i>",
                    cfg.group_id,
                    cfg.topics.len(),
                    cfg.setup_at
                ),
                None => "No group configured yet.\n\n<b>Easiest setup:</b> send <code>/setupgroup</code> <i>inside</i> the supergroup (bot must be admin, Topics enabled) and tap the confirm button.\n\nOr from here: <code>/setupgroup &lt;group_id&gt;</code> (negative integer, e.g. -1001234567890).".to_string(),
            };
            let _ = self.send_html(chat_id, &body).await;
            return;
        }
        let Ok(group_id) = arg.parse::<i64>() else {
            let _ = self
                .send_html(
                    chat_id,
                    "<i>Bad group_id. Expected a negative integer like -1001234567890.</i>",
                )
                .await;
            return;
        };
        self.run_group_setup(group_id, chat_id).await;
    }

    /// Verify the bot can reach the group, check Topics are enabled, create
    /// one forum topic per registered project, persist the mapping, and
    /// confirm to `reply_chat_id`. Shared by `/setupgroup <id>` (DM), the
    /// in-group confirm button, and the my_chat_member auto-detect path.
    async fn run_group_setup(&self, group_id: i64, reply_chat_id: i64) {
        use omega_core::telegram_group::TelegramGroupConfig;
        let chat_id = reply_chat_id;
        // Verify the bot can see the group
        let url = format!("{}/bot{}/getChat", API_BASE, self.cfg.bot_token);
        let resp = self
            .client
            .get(&url)
            .query(&[("chat_id", group_id.to_string())])
            .send()
            .await;
        let group_name = match resp {
            Ok(r) => {
                let json: serde_json::Value =
                    r.json().await.unwrap_or(serde_json::Value::Null);
                if !json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let _ = self
                        .send_html(
                            chat_id,
                            &format!(
                                "<i>Bot can't reach <code>{}</code>. Make sure it's added to the group and is admin.</i>",
                                group_id
                            ),
                        )
                        .await;
                    return;
                }
                let res = json.get("result").unwrap_or(&serde_json::Value::Null);
                let is_forum = res.get("is_forum").and_then(|v| v.as_bool()).unwrap_or(false);
                if !is_forum {
                    let _ = self
                        .send_html(
                            chat_id,
                            "<i>This group does NOT have Topics enabled. In Telegram → group settings → enable Topics, then re-run /setupgroup.</i>",
                        )
                        .await;
                    return;
                }
                res.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string()
            }
            Err(e) => {
                let _ = self
                    .send_html(
                        chat_id,
                        &format!("<i>getChat failed: <code>{}</code></i>", formatting::escape_html(&e.to_string())),
                    )
                    .await;
                return;
            }
        };

        // Persist + auto-create topics for known projects (best-effort).
        let mut cfg = TelegramGroupConfig::load().unwrap_or_default();
        cfg.group_id = group_id;
        cfg.group_name = group_name.clone();
        cfg.setup_at = chrono::Utc::now().to_rfc3339();
        let registry = omega_core::project_manager::ProjectRegistry::load();
        let mut created = 0usize;
        let mut recreated = 0usize;
        for project in &registry.projects {
            if let Some(existing) = cfg.topic_for(&project.name) {
                // The user may have DELETED the topic in Telegram. The stored
                // mapping alone is not proof it still exists — verify, and
                // recreate if it's gone (the reported bug: deleted topics were
                // never re-created on a re-run).
                if self.topic_exists(group_id, existing, &project.name).await {
                    continue;
                }
                if let Some(topic_id) = self.create_forum_topic(group_id, &project.name).await {
                    cfg.set_topic(&project.name, topic_id);
                    recreated += 1;
                }
                continue;
            }
            if let Some(topic_id) = self.create_forum_topic(group_id, &project.name).await {
                cfg.set_topic(&project.name, topic_id);
                created += 1;
            }
        }
        let _ = cfg.save();

        let body = format!(
            "Group <b>{}</b> registered.\nProjects mapped to topics: {}\nNew topics this run: {}\nRecreated (were deleted): {}\n\nOracle reports for each project will land in its topic.",
            formatting::escape_html(&group_name),
            cfg.topics.len(),
            created,
            recreated
        );
        let _ = self.send_html(chat_id, &body).await;
    }

    /// Auto-setup triggered by a my_chat_member event when the bot is
    /// added/promoted to admin in a forum-enabled supergroup. Persists the
    /// group config, creates one topic per registered project, and DMs the
    /// owner with a confirmation card. Idempotent — running it twice for
    /// the same group only fills missing topics.
    async fn auto_setup_group(&self, group_id: i64, group_name: String) {
        use omega_core::telegram_group::TelegramGroupConfig;
        let mut cfg = TelegramGroupConfig::load().unwrap_or_default();
        let is_new = cfg.group_id != group_id;
        cfg.group_id = group_id;
        if !group_name.is_empty() {
            cfg.group_name = group_name.clone();
        }
        cfg.setup_at = chrono::Utc::now().to_rfc3339();

        let registry = omega_core::project_manager::ProjectRegistry::load();
        let mut created = 0usize;
        for project in &registry.projects {
            if let Some(existing) = cfg.topic_for(&project.name) {
                if self.topic_exists(group_id, existing, &project.name).await {
                    continue;
                }
                if let Some(topic_id) = self.create_forum_topic(group_id, &project.name).await {
                    cfg.set_topic(&project.name, topic_id);
                    created += 1;
                }
                continue;
            }
            if let Some(topic_id) = self.create_forum_topic(group_id, &project.name).await {
                cfg.set_topic(&project.name, topic_id);
                created += 1;
            }
        }
        let _ = cfg.save();

        // Confirm to the owner in their DM.
        let owner = *self
            .cfg
            .allow_user_ids
            .first()
            .unwrap_or(&self.cfg.chat_id);
        let label = if is_new { "registered" } else { "updated" };
        let body = format!(
            "Project group <b>{label}</b> ✓\n\n\
             Group: <b>{name}</b>\n\
             ID: <code>{id}</code>\n\
             Topics mapped: {n}\n\
             New this run: {created}\n\n\
             <i>Oracle reports will land in each project's topic. PDF artifacts via pdfgen.</i>",
            label = label,
            name = formatting::escape_html(&cfg.group_name),
            id = group_id,
            n = cfg.topics.len(),
            created = created,
        );
        let _ = self.send_html(owner, &body).await;
        tracing::info!(
            group_id = group_id,
            new = is_new,
            topics = cfg.topics.len(),
            created = created,
            "auto-setup of project group complete"
        );
    }

    /// Probe whether a forum topic still exists. Telegram exposes no read
    /// endpoint for topics, so we abuse `editForumTopic` as a probe.
    ///
    /// Why the probe MUST use a DIFFERENT name (empirically verified against
    /// the live Bot API):
    ///   • `editForumTopic(thread, name = <SAME current name>)`
    ///       → `Bad Request: TOPIC_NOT_MODIFIED` for BOTH live AND deleted
    ///         topics — useless, can't tell them apart.
    ///   • `reopenForumTopic(thread)` on a live (open) topic
    ///       → also `TOPIC_NOT_MODIFIED` — useless.
    ///   • `editForumTopic(thread, name = <DIFFERENT name>)`
    ///       → `ok:true`                   on a LIVE topic (rename applied,
    ///                                      so we restore the canonical name
    ///                                      with a follow-up edit).
    ///       → `TOPIC_ID_INVALID`          on a DELETED or never-existed
    ///                                      topic. This is the only signal
    ///                                      we can rely on.
    ///
    /// Behavior:
    ///   • ok:true                              → exists. Restore the
    ///                                            canonical name (best-effort).
    ///   • description ~ TOPIC_ID_INVALID/etc.  → DELETED → caller recreates.
    ///   • any other error (incl. TOPIC_NOT_MODIFIED, network/parse, rate
    ///     limit, unknown)                      → assume alive (transient
    ///                                            failures must not spawn
    ///                                            duplicate topics).
    async fn topic_exists(&self, group_id: i64, thread_id: i64, name: &str) -> bool {
        let url = format!("{}/bot{}/editForumTopic", API_BASE, self.cfg.bot_token);
        // Distinct from `name` so Telegram doesn't shortcut to TOPIC_NOT_MODIFIED.
        let probe = format!("{name} ·");
        let body = serde_json::json!({
            "chat_id": group_id,
            "message_thread_id": thread_id,
            "name": probe,
        });
        let json = match self.client.post(&url).json(&body).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(j) => j,
                Err(_) => return true,
            },
            Err(_) => return true,
        };
        if json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            // Live topic — undo the probe rename (best-effort; ignore result).
            let restore = serde_json::json!({
                "chat_id": group_id,
                "message_thread_id": thread_id,
                "name": name,
            });
            let _ = self.client.post(&url).json(&restore).send().await;
            return true;
        }
        let desc = json
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_uppercase();
        let deleted = desc.contains("TOPIC_ID_INVALID")
            || desc.contains("THREAD NOT FOUND")
            || desc.contains("THREAD_NOT_FOUND")
            || desc.contains("MESSAGE THREAD NOT FOUND")
            || desc.contains("TOPIC_DELETED");
        !deleted
    }

    /// createForumTopic — returns the new topic's message_thread_id, or
    /// None on failure.
    async fn create_forum_topic(&self, group_id: i64, name: &str) -> Option<i64> {
        let url = format!("{}/bot{}/createForumTopic", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": group_id,
            "name": name,
        });
        let resp = self.client.post(&url).json(&body).send().await.ok()?;
        let json: serde_json::Value = resp.json().await.ok()?;
        if !json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            tracing::warn!(?json, project = %name, "createForumTopic failed");
            return None;
        }
        json.get("result")
            .and_then(|r| r.get("message_thread_id"))
            .and_then(|v| v.as_i64())
    }

    /// deleteForumTopic — remove the topic from the supergroup. Returns
    /// true on `{ok:true}`, false on any failure (network, non-ok, etc.).
    /// Used by `/project → delete` to clean up the topic when a project
    /// is removed from the registry.
    async fn delete_forum_topic(&self, group_id: i64, thread_id: i64) -> bool {
        let url = format!("{}/bot{}/deleteForumTopic", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": group_id,
            "message_thread_id": thread_id,
        });
        let resp = match self.client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, thread_id = thread_id, "deleteForumTopic network error");
                return false;
            }
        };
        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(_) => return false,
        };
        let ok = json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ok {
            tracing::warn!(?json, thread_id = thread_id, "deleteForumTopic failed");
        }
        ok
    }

    /// Ensure a topic exists for `project`. If we have a group registered
    /// but no topic for the project yet, create one and persist. Returns
    /// the topic id on success.
    async fn ensure_topic_for_project(&self, project: &str) -> Option<i64> {
        use omega_core::telegram_group::TelegramGroupConfig;
        let mut cfg = TelegramGroupConfig::load()?;
        if let Some(t) = cfg.topic_for(project) {
            return Some(t);
        }
        let topic = self.create_forum_topic(cfg.group_id, project).await?;
        cfg.set_topic(project, topic);
        let _ = cfg.save();
        Some(topic)
    }

    /// Send an HTML message into a specific forum topic.
    async fn send_html_to_topic(&self, group_id: i64, topic_id: i64, text: &str) -> Result<()> {
        let url = format!("{}/bot{}/sendMessage", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": group_id,
            "message_thread_id": topic_id,
            "text": text,
            "parse_mode": "HTML",
        });
        let _ = self.client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Send HTML, threading the reply back into the originating forum topic
    /// when `thread_id` is `Some`. Without this, slash-command replies typed
    /// inside a project topic (`/status`, `/dispatch`, …) post into the
    /// group's General thread instead of the topic the user was in — broken
    /// UX. Falls back to `send_html` when `thread_id` is None so DM behavior
    /// is unchanged.
    async fn send_html_smart(
        &self,
        chat_id: i64,
        thread_id: Option<i64>,
        text: &str,
    ) -> Result<()> {
        match thread_id {
            Some(tid) => self.send_html_to_topic(chat_id, tid, text).await,
            None => {
                let _ = self.send_html(chat_id, text).await;
                Ok(())
            }
        }
    }

    /// Send a document into a specific forum topic (oracle PDF reports).
    async fn send_document_to_topic(
        &self,
        group_id: i64,
        topic_id: i64,
        filename: &str,
        mime: &str,
        bytes: &[u8],
        caption: &str,
    ) -> Result<()> {
        let url = format!("{}/bot{}/sendDocument", API_BASE, self.cfg.bot_token);
        let part = reqwest::multipart::Part::bytes(bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime)
            .unwrap_or_else(|_| {
                reqwest::multipart::Part::bytes(bytes.to_vec()).file_name(filename.to_string())
            });
        let form = reqwest::multipart::Form::new()
            .text("chat_id", group_id.to_string())
            .text("message_thread_id", topic_id.to_string())
            .text("caption", caption.to_string())
            .text("parse_mode", "HTML")
            .part("document", part);
        let _ = self.client.post(&url).multipart(form).send().await?;
        Ok(())
    }

    async fn mirror_to_master_pane(&self, user_msg: &str, response: &str) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let log = home.join(".omega/state/aisb-conversation.log");
        if let Some(parent) = log.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let ts = chrono::Local::now().format("%H:%M:%S");
        let entry = format!(
            "\n\x1b[90m──────── {} ────────\x1b[0m\n\x1b[36m▶ You:\x1b[0m {}\n\x1b[33m◆ AISB:\x1b[0m {}\n",
            ts, user_msg.trim(), response.trim()
        );
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log) {
            let _ = f.write_all(entry.as_bytes());
        }
    }

    /// Kill the AISB Master session and respawn it FRESH (no --continue).
    /// Also resets the curator-triggered flags so the new session starts
    /// with a clean self-improvement slate. Invoked by the /clean command.
    async fn clean_master(&self) -> Result<()> {
        let master = omega_core::aisb::MASTER_SESSION_NAME;

        // 1. Reset the BRAIN — drop the persistent Claude SDK subprocess so
        //    the next message starts a brand-new conversation (clean slate).
        self.claude_stream.reset().await;

        // 2. Truncate the conversation log so the viewer starts fresh too.
        let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
        let log = home.join(".omega/state/aisb-conversation.log");
        let _ = std::fs::write(
            &log,
            "  Ω  AISB Master — fresh conversation (cleaned)\n\
             ─────────────────────────────────────────────────\n",
        );

        // 3. Kill + respawn the viewer session (tail -F picks up the
        //    truncated log automatically, but respawn guarantees a clean
        //    pane even if the old viewer was detached).
        let _ = self.mgr.kill_session(master).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let home_str = home.to_string_lossy().to_string();
        omega_core::aisb::ensure_master(
            &self.mgr,
            omega_core::agents::Agent::Claude,
            &home_str,
        )
        .await?;
        Ok(())
    }

    async fn edit_message_html(&self, chat_id: i64, message_id: i64, text: &str) -> Result<()> {
        let url = format!("{}/bot{}/editMessageText", API_BASE, self.cfg.bot_token);
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": text,
            "parse_mode": "HTML",
        });
        match self.client.post(&url).json(&body).send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!(error = %e, "editMessageText failed");
                Ok(())
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
        use omega_core::telegram_group::TelegramGroupConfig;
        let reports = ReportPipeline::check_for_reports();
        let group_cfg = TelegramGroupConfig::load();
        for report in reports {
            let html = report.to_telegram_html();
            let keyboard = report.inline_keyboard();

            // Route by project to a topic when the supergroup is set up
            // AND a topic exists (or can be created) for this project.
            // Otherwise fall back to the owner's DM.
            let topic_target: Option<(i64, i64)> = if let Some(ref cfg) = group_cfg {
                let topic = match cfg.topic_for(&report.project) {
                    Some(t) => Some(t),
                    None => self.ensure_topic_for_project(&report.project).await,
                };
                topic.map(|t| (cfg.group_id, t))
            } else {
                None
            };

            // 1. Text report into the topic (or DM).
            let sent_to_topic = if let Some((gid, tid)) = topic_target {
                self.send_html_to_topic(gid, tid, &html).await.is_ok()
            } else {
                false
            };
            if !sent_to_topic {
                match self
                    .send_html_with_keyboard(self.reply_chat_id, &html, &keyboard)
                    .await
                {
                    Ok(Some(msg_id)) => {
                        let mut router = self.reply_router.lock().await;
                        router.track(msg_id, &report.project);
                    }
                    Ok(None) => {
                        tracing::warn!(project = %report.project, "report sent but no message_id");
                    }
                    Err(e) => {
                        tracing::error!(project = %report.project, error = %e, "failed to deliver report");
                    }
                }
            }

            // 2. PDF artifact (rendered via pdfgen tool) into the topic
            //    when group + topic exist. Best-effort, never blocks.
            if let Some((gid, tid)) = topic_target {
                if let Some((pdf_bytes, filename)) = render_report_pdf(&report).await {
                    let caption = format!(
                        "<b>Oracle report — {}</b>",
                        formatting::escape_html(&report.project)
                    );
                    let _ = self
                        .send_document_to_topic(gid, tid, &filename, "application/pdf", &pdf_bytes, &caption)
                        .await;
                }
            }
            tracing::info!(
                project = %report.project,
                topic = ?topic_target.map(|(_, t)| t),
                "delivered oracle report"
            );
        }
    }
}

/// Render an oracle report into a PDF via the global `pdfgen` tool
/// (template=doc, theme=agentik). Returns (bytes, filename) on success,
/// None if pdfgen isn't installed or rendering fails.
async fn render_report_pdf(report: &OracleReport) -> Option<(Vec<u8>, String)> {
    let body_md = format!(
        "# Oracle report — {project}\n\n\
         **Status:** {status}    **Build:** {build}\n\n\
         {body}\n",
        project = report.project,
        status = report.status,
        build = report.build,
        body = report.body,
    );
    let data = serde_json::json!({
        "template": "doc",
        "theme": "agentik",
        "eyebrow": "ORACLE REPORT",
        "title": format!("Oracle report — {}", report.project),
        "subtitle": format!("Status: {}", report.status),
        "author": "OmegaOS",
        "date": chrono::Local::now().format("%B %Y").to_string(),
        "docId": format!("ORC-{}", chrono::Utc::now().format("%Y%m%d-%H%M")),
        "brand": "OmegaOS",
        "body": body_md,
    });
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let data_path = format!("/tmp/oracle-report-{}-{}.json", report.project, ts);
    std::fs::write(&data_path, data.to_string()).ok()?;
    let pdf_path = format!("/tmp/oracle-report-{}-{}.pdf", report.project, ts);
    let status = tokio::process::Command::new("pdfgen")
        .args([
            "--template=doc",
            "--theme=agentik",
            &format!("--data={}", data_path),
            &format!("--out={}", pdf_path),
        ])
        .status()
        .await
        .ok()?;
    if !status.success() {
        return None;
    }
    let bytes = std::fs::read(&pdf_path).ok()?;
    let filename = format!("oracle-{}-{}.pdf", report.project, ts);
    let _ = std::fs::remove_file(&data_path);
    let _ = std::fs::remove_file(&pdf_path);
    Some((bytes, filename))
}

// ── Main entry point ──

pub async fn run(cfg: OmegaTelegramConfig) -> Result<()> {
    println!("◆ Omega Telegram bot engine starting");
    println!("  Relay session: {}", cfg.relay_session);
    println!("  Chat ID:       {}", cfg.chat_id);

    // Keep full scrollback for the oracles this bridge spawns (default 2000
    // lines lost the top of long chats). Global rmux daemon option, set once
    // at startup so any session spawned later retains 100k lines. Best-effort.
    let _ = tokio::process::Command::new("rmux")
        .args(["set-option", "-g", "history-limit", "100000"])
        .output()
        .await;

    let engine = std::sync::Arc::new(TelegramBotEngine::new(cfg.clone()).await?);
    engine.ensure_master().await;
    engine.register_bot_commands().await;
    engine.send_back_online_card().await;

    // ── Local inbox watcher ──────────────────────────────────────────
    // The aisb-master pane runs `omega aisb-chat`, which appends typed
    // lines to ~/.omega/state/aisb-local-inbox.jsonl. This task watches
    // that file and injects each new line as a synthetic Telegram message
    // (same brain, response also goes to Telegram). Runs independently of
    // the 25s long-poll so local input is processed immediately.
    {
        let watcher = engine.clone();
        let owner_chat = cfg
            .allow_user_ids
            .first()
            .copied()
            .unwrap_or(cfg.chat_id);
        tokio::spawn(async move {
            let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
            let inbox = home.join(".omega/state/aisb-local-inbox.jsonl");
            let mut processed: usize = std::fs::read_to_string(&inbox)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            loop {
                tokio::time::sleep(Duration::from_millis(400)).await;
                let Ok(content) = std::fs::read_to_string(&inbox) else { continue };
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() <= processed {
                    continue;
                }
                for line in &lines[processed..] {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                            watcher.process_local_text(owner_chat, text).await;
                        }
                    }
                }
                processed = lines.len();
            }
        });
    }

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
            "{}/bot{}/getUpdates?timeout=25&offset={}&allowed_updates=[\"message\",\"callback_query\",\"my_chat_member\"]",
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

            // Auto-detect a project supergroup the moment the bot is added
            // or promoted to admin in a forum-enabled chat. Triggers full
            // auto-setup so the user never has to run /setupgroup manually.
            if let Some(mcm) = upd.my_chat_member {
                let chat_type = mcm.chat.chat_type.as_deref().unwrap_or("");
                let is_forum = mcm.chat.is_forum.unwrap_or(false);
                let status = mcm
                    .new_chat_member
                    .as_ref()
                    .map(|m| m.status.as_str())
                    .unwrap_or("");
                let is_admin = status == "administrator" || status == "creator";
                let actor_id = mcm.from.as_ref().map(|u| u.id);
                if !cfg.is_authorized(mcm.chat.id, actor_id) {
                    tracing::warn!(
                        chat_id = mcm.chat.id,
                        actor_id = ?actor_id,
                        "rejected unauthorized my_chat_member auto-setup"
                    );
                    continue;
                }
                if is_admin && chat_type == "supergroup" && is_forum {
                    engine
                        .auto_setup_group(
                            mcm.chat.id,
                            mcm.chat.title.clone().unwrap_or_default(),
                        )
                        .await;
                }
                continue;
            }

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

/// Render the Oracle role prompt (agents/oracle.md) with project
/// placeholders filled, write it to ~/.omega/state/oracle-prompts/<name>.md,
/// and return the path. Returns None if the template can't be found.
///
/// Placeholders: {{PROJECT}}, {{WORKDIR}}, {{SESSION}}.
fn render_oracle_prompt(project: &str, workdir: &str, session: &str) -> Option<String> {
    let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
    // Prefer the installed copy, fall back to the repo copy.
    let candidates = [
        home.join(".omega/agents/oracle.md"),
        std::path::PathBuf::from("agents/oracle.md"),
    ];
    let template = candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())?;
    let mut rendered = template
        .replace("{{PROJECT}}", project)
        .replace("{{WORKDIR}}", workdir)
        .replace("{{SESSION}}", session);
    // Prepend the hardened brief preamble (Opus 4.8 system-card surface).
    let preamble = omega_core::rules::brief_preamble();
    if !preamble.is_empty() {
        rendered = format!("{}\n\n---\n\n{}", preamble, rendered);
    }
    // Inject the Oracle-scoped rules (single source of truth — rules.rs).
    let rules = omega_core::rules::rules_prompt_block(omega_core::rules::RuleScope::Oracle);
    if !rules.is_empty() {
        rendered.push_str("\n\n");
        rendered.push_str(&rules);
    }
    let dir = home.join(".omega/state/oracle-prompts");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.md", session));
    std::fs::write(&path, rendered).ok()?;
    Some(path.to_string_lossy().to_string())
}

/// Extract a single readable "current activity" line from a captured pane.
///
/// Walks the pane bottom-up to surface the most recent line of REAL agent
/// content. Crucially does NOT use clean_terminal_output (which strips
/// `●`-prefixed lines wholesale — those are exactly Claude's status updates
/// like "● Reading file X" and are the BEST activity signal). Instead does
/// targeted cleanup: strip ANSI escapes, drop dividers + keyboard hints +
/// the bare input prompt, keep everything else. Truncates to `max_chars`
/// chars with an ellipsis. Returns None when nothing readable remains.
fn last_activity_line(pane_text: &str, max_chars: usize) -> Option<String> {
    fn is_chrome_hint(t: &str) -> bool {
        let lower = t.to_lowercase();
        t.contains('↑')
            || t.contains('↓')
            || t.contains('⏵')
            || t.contains('⎿')
            || lower.contains("enter to ")
            || lower.contains("esc to ")
            || lower.contains("shift+tab")
            || lower.contains("ctrl+")
            || lower.contains("bypass permissions")
            || lower.contains("type something")
            || lower.contains("to navigate")
            || lower.contains("to interrupt")
            || lower.contains("press up to edit")
    }
    fn is_divider(t: &str) -> bool {
        !t.is_empty()
            && t.chars()
                .all(|c| matches!(c, '─' | '═' | '·' | '│' | ' ' | '\t'))
    }
    // ANSI strip (same pattern as clean_terminal_output but without the
    // wholesale `●` drop).
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07")
        .unwrap_or_else(|_| regex::Regex::new(r"$^").unwrap());
    let stripped = ansi_re.replace_all(pane_text, "");

    let picked = stripped
        .lines()
        .rev()
        .map(|l| l.trim_end())
        .find(|l| {
            let trimmed = l.trim_start();
            if trimmed.is_empty() {
                return false;
            }
            // The bare input prompt `❯ ` followed by nothing is chrome.
            if trimmed == "❯" || trimmed.starts_with("❯ ") && trimmed.len() <= 4 {
                return false;
            }
            if is_divider(trimmed) {
                return false;
            }
            if is_chrome_hint(trimmed) {
                return false;
            }
            true
        })?;
    let t = picked.trim().to_string();
    if t.is_empty() {
        return None;
    }
    let truncated: String = if t.chars().count() > max_chars {
        let mut s: String = t.chars().take(max_chars.saturating_sub(1)).collect();
        s.push('…');
        s
    } else {
        t
    };
    Some(truncated)
}

fn clean_terminal_output(text: &str) -> String {
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07").unwrap_or_else(|_| {
        regex::Regex::new(r"$^").unwrap()
    });
    let stripped = ansi_re.replace_all(text, "");
    let no_reminders = regex::Regex::new(r"(?s)<system-reminder>.*?</system-reminder>")
        .map(|re| re.replace_all(&stripped, "").to_string())
        .unwrap_or_else(|_| stripped.to_string());
    // Strip the whole <claude-mem-context> block (multi-line) — agents keep it
    // in context (injected silently via additionalContext), but the human-facing
    // mirror/Telegram view must never show the memory dump.
    let no_reminders = regex::Regex::new(r"(?s)<claude-mem-context>.*?</claude-mem-context>")
        .map(|re| re.replace_all(&no_reminders, "").to_string())
        .unwrap_or(no_reminders);
    let lines: Vec<&str> = no_reminders
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| {
            if l.is_empty() {
                return false;
            }
            let t = l.trim();
            // Claude UI chrome lines — drop them (the actual response text
            // had ● already stripped by extract_response). NOTE: these used
            // to be `starts_with("")` empty-string checks which always
            // matched and silently stripped EVERY line — root cause of
            // the "Empty response" bug.
            if t.starts_with('❯') { return false; }
            if t.starts_with('✻') { return false; }
            if t.starts_with('⎿') { return false; }
            if t.starts_with('·') { return false; }
            if t.starts_with('●') { return false; }
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
            if t.chars().all(|c| c == '─' || c == ' ') { return false; }
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

fn extract_response(before: &str, after: &str) -> String {
    // Find the LAST ● in `after` (most recent agent response).
    // Then verify it's NEW: either the line content differs from the last
    // ● in `before`, or there are now more ● lines than before.
    // Without this check, the polling would re-send the previous response
    // as soon as polling starts (before the agent has answered the new msg).
    let after_lines: Vec<&str> = after.lines().collect();
    let before_bullets: Vec<&str> = before
        .lines()
        .filter(|l| l.trim().starts_with("●"))
        .collect();
    let after_bullets: Vec<(usize, &&str)> = after_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().starts_with("●"))
        .collect();

    // No bullets yet → agent hasn't responded
    let Some(&(start, last_after_bullet)) = after_bullets.last() else {
        return String::new();
    };

    // Compare: new bullet count > before, OR last bullet text differs.
    // If neither → still the same old response, not new.
    let last_before_bullet = before_bullets.last().map(|s| s.trim()).unwrap_or("");
    let is_new = after_bullets.len() > before_bullets.len()
        || last_after_bullet.trim() != last_before_bullet;
    if !is_new {
        return String::new();
    }

    let mut response_lines: Vec<String> = Vec::new();
    let first = after_lines[start].trim().trim_start_matches("●").trim();
    if !first.is_empty() {
        response_lines.push(first.to_string());
    }
    let lines = after_lines;

    // Collect continuation lines (until stop marker).
    // Empty-string starts_with checks (which always matched) were the
    // accidental-strip side-effect of the emoji cleanup pass — removed.
    for line in lines.iter().skip(start + 1) {
        let t = line.trim();
        if t.is_empty() { continue; }

        // Stop markers — line starts with Claude's chrome characters
        // OR contains a known status phrase.
        let first_char = t.chars().next();
        let is_chrome_prefix = matches!(first_char, Some('❯' | '✻' | '⎿' | '·' | '●'));
        let is_separator = t.starts_with("───") || t.starts_with("━━━");
        let is_status = t.contains("bypass permissions")
            || t.contains("esc to interrupt")
            || t.contains("shift+tab")
            || t.contains("Churned")
            || t.contains("Brewed")
            || t.contains("Cultivating")
            || t.contains("Crunched")
            || t.contains("Pontificating")
            || t.contains("Cogitated");
        if is_chrome_prefix || is_separator || is_status {
            break;
        }
        response_lines.push(t.to_string());
    }

    response_lines.join("\n")
}

/// Read the active provider+model from ~/.omega/state/telegram-active-model.json
/// Falls back to ("claude", "") for default Claude with default model.
fn read_active_provider_model() -> (String, String) {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let path = home.join(".omega/state/telegram-active-model.json");
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
            let provider = json.get("active_provider").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
            let model = json.get("active_model").and_then(|v| v.as_str()).unwrap_or("").to_string();
            return (provider, model);
        }
    }
    ("claude".to_string(), "".to_string())
}

/// Run a one-shot LLM query using the selected provider's CLI.
/// Returns the same Output type as Command::output() so the caller can
/// inspect stdout/stderr.
fn run_llm_oneshot(provider: &str, model: &str, prompt: &str) -> std::io::Result<std::process::Output> {
    match provider {
        "claude" | "" => {
            let mut cmd = std::process::Command::new("claude");
            // Use an isolated config dir with NO hooks → ~4s vs ~11s with
            // the user's full settings.json (whose SessionEnd hook hangs ~7s).
            // Credentials are symlinked into this dir so OAuth still works.
            if let Some(home) = dirs::home_dir() {
                let cfg = home.join(".omega/claude-bridge-config");
                if cfg.join("settings.json").exists() {
                    cmd.env("CLAUDE_CONFIG_DIR", &cfg);
                }
            }
            cmd.args(["--print", "--dangerously-skip-permissions"]);
            if !model.is_empty() {
                cmd.args(["--model", model]);
            }
            // Close stdin so claude doesn't wait 3s for piped input.
            cmd.stdin(std::process::Stdio::null());
            cmd.arg(prompt).output()
        }
        "codex" => {
            // codex CLI: codex "prompt"
            std::process::Command::new("codex").arg(prompt).output()
        }
        "gemini" => {
            let mut cmd = std::process::Command::new("gemini");
            if !model.is_empty() {
                cmd.args(["-m", model]);
            }
            cmd.arg(prompt).output()
        }
        "glm" => {
            std::process::Command::new("glm").arg(prompt).output()
        }
        "pi" => {
            let mut cmd = std::process::Command::new("pi");
            cmd.args(["--provider", "openrouter"]);
            if !model.is_empty() {
                cmd.args(["--model", model]);
            }
            cmd.arg(prompt).output()
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unknown provider: {}", provider),
        )),
    }
}

#[cfg(test)]
mod activity_line_tests {
    use super::last_activity_line;

    #[test]
    fn skips_ui_chrome_picks_real_content() {
        // Real fixture from an oracle pane tail (DentistryGPT-1, post /goal).
        let pane = "\
  3. Tester avec vrais doublons DM↔Orthalis
     Mettre en place un cabinet
  4. Type something.
───────────────────────────────────
  5. Chat about this

Enter to select · ↑/↓ to navigate · Esc to cancel
";
        let got = last_activity_line(pane, 80).unwrap();
        // Must skip the chrome line + "Type something" + "Chat about this" not chrome but
        // it's a label — the most recent REAL content line is "5. Chat about this".
        // Acceptable: the picker walks bottom-up, skips chrome, lands on the
        // last non-chrome line.
        assert!(
            got.contains("5. Chat about this") || got.contains("Mettre en place")
                || got.contains("Tester"),
            "got: {got}"
        );
        assert!(!got.contains("Enter to select"));
        assert!(!got.contains("↑/↓"));
    }

    #[test]
    fn truncates_long_lines_with_ellipsis() {
        let pane = "● ".to_string() + &"x".repeat(200);
        let got = last_activity_line(&pane, 20).unwrap();
        assert_eq!(got.chars().count(), 20);
        assert!(got.ends_with('…'));
    }

    #[test]
    fn empty_pane_returns_none() {
        assert_eq!(last_activity_line("", 80), None);
        assert_eq!(last_activity_line("   \n   \n", 80), None);
    }

    #[test]
    fn only_chrome_returns_none() {
        let pane = "
Enter to select · ↑/↓ to navigate · Esc to cancel
  ⏵⏵ bypass permissions on (shift+tab to cycle)
";
        assert_eq!(last_activity_line(pane, 80), None);
    }
}
