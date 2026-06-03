//! Persistent Claude subprocess for instant Telegram responses.
//!
//! Spawns one `claude --output-format=stream-json --input-format=stream-json`
//! at bridge startup. Keeps stdin/stdout pipes open. Each user message =
//! one JSON write + read events until `{"type":"result"}`.
//!
//! First-message cost: ~3s (claude CLI startup). Subsequent messages:
//! dominated by LLM inference (streaming tokens). No 3s startup per message.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct PersistentClaude {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

// ─────────────────────────────────────────────────────────────────────────
// Streaming protocol (Chunk 2) — events yielded AS THEY ARRIVE.
//
// `PersistentClaude` historically `ask()`'d in a block: loop to the `result`
// event, return one `String`. Chunk 2 adds an event-yielding path so the
// Telegram bridge can edit the placeholder with REAL assistant text as the
// turn composes (fixes N12/N13 on the master path) instead of a fake progress
// bar, and so the FINAL result is CLASSIFIED (content vs CLI-error like the
// `You've hit your session limit` string) before delivery.
//
// In `--output-format=stream-json --verbose`, the CLI emits one JSON object
// per line. The relevant shapes:
//   {"type":"assistant","message":{"content":[{"type":"text","text":"…"},
//                                              {"type":"tool_use","name":"Bash",…},
//                                              {"type":"tool_use","name":"SendUserMessage",
//                                               "input":{"message":"…"}}]}}
//   {"type":"user","message":{"content":[{"type":"tool_result",…}]}}   (tool output)
//   {"type":"result","subtype":"success"|"error_…","is_error":bool,"result":"…"}
// Assistant events carry WHOLE content blocks (not token deltas) in this mode,
// so the natural streaming granularity is per assistant text block / tool call
// — already vastly better than one block at the very end for multi-step turns.
//
// When the session ALSO runs with `--include-partial-messages` (a headless /
// structured worker — Lane B), the CLI ADDITIONALLY interleaves raw Anthropic
// SSE partial events wrapped as a `stream_event` envelope:
//   {"type":"stream_event","event":{"type":"content_block_delta",
//                                    "delta":{"type":"text_delta","text":"…"}}}
//   {"type":"stream_event","event":{"type":"message_stop"}}
// We parse BOTH shapes from the SAME line-loop: the whole-envelope path always
// works (today's behavior), and the partial path layers token-level deltas +
// a deterministic message-stop boundary on top when the mode is enabled. A
// session that never sets the flag simply never emits `stream_event` lines —
// the parser falls through them harmlessly (honest fallback).
// ─────────────────────────────────────────────────────────────────────────

/// One streamed event from a brain turn, delivered to the bridge as it arrives.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// An assistant text block (the visible answer prose, possibly partial when
    /// the turn is multi-step). The bridge accumulates these into the placeholder.
    /// Emitted from the WHOLE-envelope mode (`assistant` events) — one block per
    /// turn step.
    AssistantText(String),
    /// A token-level text delta. Emitted only when the session runs with
    /// `--include-partial-messages` (the CLI then wraps raw Anthropic SSE
    /// `content_block_delta` events as `{"type":"stream_event",...}`). The bridge
    /// appends these in place so the answer composes character-by-character.
    /// Honest fallback: a session WITHOUT partial messages never emits these and
    /// keeps streaming whole `AssistantText` blocks — both paths render the same.
    TextDelta(String),
    /// The model invoked a tool (e.g. `Bash`, `Read`). Surfaced as an optional
    /// collapsed trace line ("🔧 running Bash…").
    ToolUse { name: String },
    /// A tool returned. Surfaced as an optional collapsed trace line.
    ToolResult,
    /// The `--brief` agent→user push: the model invoked the `SendUserMessage`
    /// tool to surface a structured note to the human MID-TASK (not the final
    /// answer). The bridge fires this as a distinct Telegram alert so a long
    /// turn can report progress before it completes.
    UserMessage(String),
    /// A message-level stop marker. In partial-message mode the CLI emits a
    /// `message_stop` stream event when one assistant message finishes; the
    /// bridge uses it as a DETERMINISTIC turn-boundary signal (the assistant
    /// block is complete) instead of scraping a pane for a settled prompt. The
    /// authoritative turn END remains the `result` event (which carries the
    /// classified outcome) — `message_stop` is only an intra-turn boundary.
    MessageStop,
}

/// Final outcome of a brain turn, classified. `is_error` true means the CLI
/// surfaced an error (session-limit, auth-expiry, transport failure) — the
/// bridge MUST render it as an error card, NOT as the answer body.
#[derive(Debug, Clone)]
pub struct BrainOutcome {
    /// The final result text (answer body, or the error message when `is_error`).
    pub text: String,
    /// True when the `result` event was an error, OR the text matches a known
    /// CLI-error signature even on a "success" subtype.
    pub is_error: bool,
}

/// Classify a `result` event: an explicit `is_error`/error subtype is an error;
/// a "success" subtype whose body still matches a known CLI-error signature
/// (the model never produced an answer, the CLI surfaced its own failure
/// string verbatim — e.g. `You've hit your session limit`) is ALSO an error.
/// Returning a clean classification here is the whole point: today that raw
/// string is delivered as the reply.
pub fn classify_result(is_error_flag: bool, subtype: &str, result_text: &str) -> BrainOutcome {
    let subtype_is_error = subtype.starts_with("error");
    let signature_is_error = looks_like_cli_error(result_text);
    BrainOutcome {
        text: result_text.to_string(),
        is_error: is_error_flag || subtype_is_error || signature_is_error,
    }
}

/// Known CLI-error signatures that some claude builds emit on stdout as a
/// `result` body even with a non-error subtype. Case-insensitive substring
/// match — deliberately conservative (only unambiguous CLI failures) so a
/// genuine answer that merely mentions one of these phrases isn't misflagged:
/// we anchor on the CLI's own phrasing, not the topic.
fn looks_like_cli_error(text: &str) -> bool {
    let t = text.to_lowercase();
    const SIGNATURES: &[&str] = &[
        "you've hit your session limit",
        "you have hit your session limit",
        "session limit reached",
        "credit balance is too low",
        "please run /login",
        "invalid api key",
        "authentication_error",
        "oauth token has expired",
        "your authentication token has expired",
        "rate limit exceeded",
        "overloaded_error",
    ];
    SIGNATURES.iter().any(|s| t.contains(s))
}

impl PersistentClaude {
    /// Spawn the persistent claude subprocess as the AISB Master:
    /// - cwd = $HOME (full system access, not scoped to any project)
    /// - --append-system-prompt-file loads the AISB Master prompt
    ///   (router/dispatcher identity — NEVER does the work itself)
    pub async fn spawn(config_dir: Option<PathBuf>) -> Result<Self> {
        let mut cmd = Command::new("claude");
        if let Some(dir) = config_dir {
            cmd.env("CLAUDE_CONFIG_DIR", dir);
        }

        // AISB Master runs from $HOME — full system access, all projects in scope.
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        cmd.current_dir(&home);

        // Inject the AISB Master system prompt so the model knows it's the
        // dispatcher, not a project agent.
        //
        // The Master now carries the live Laws + Master-scoped operational
        // rules from the typed registry (omega_core::rules). We compose a
        // runtime file = static aisb-master.md + rules_prompt_block(Master)
        // and append THAT. Single source of truth → zero drift between the
        // registry and what the Master LLM actually sees. If composition
        // fails, we fall back to the raw static file (current behavior);
        // if neither exists, we append nothing (current behavior).
        let aisb_prompt = home.join(".omega/agents/aisb-master.md");
        let runtime_prompt = home.join(".omega/agents/_master-runtime.md");
        let rules_block = omega_core::rules::rules_prompt_block(
            omega_core::rules::RuleScope::Master,
        );
        let base = std::fs::read_to_string(&aisb_prompt).unwrap_or_default();
        let composed = format!("{}\n\n{}\n", base.trim_end(), rules_block);
        let composed_ok = if let Some(parent) = runtime_prompt.parent() {
            std::fs::create_dir_all(parent).is_ok()
                && std::fs::write(&runtime_prompt, &composed).is_ok()
        } else {
            false
        };

        let mut args: Vec<&str> = vec![
            "--print",
            "--output-format=stream-json",
            "--input-format=stream-json",
            "--dangerously-skip-permissions",
            "--verbose",
        ];
        let prompt_arg;
        if composed_ok && runtime_prompt.exists() {
            prompt_arg = runtime_prompt.to_string_lossy().to_string();
            args.push("--append-system-prompt-file");
            args.push(&prompt_arg);
        } else if aisb_prompt.exists() {
            prompt_arg = aisb_prompt.to_string_lossy().to_string();
            args.push("--append-system-prompt-file");
            args.push(&prompt_arg);
        }
        cmd.args(&args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());

        let mut child = cmd.spawn().context("spawn claude stream subprocess")?;
        let stdin = child.stdin.take().context("no stdin pipe")?;
        let stdout = child.stdout.take().context("no stdout pipe")?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    /// Send a prompt, read events until a `result` event, return the text.
    pub async fn ask(&mut self, prompt: &str) -> Result<String> {
        let msg = serde_json::json!({
            "type": "user",
            "message": { "role": "user", "content": prompt }
        });
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        // Read events until we see a `result` event. Each read is bounded by a
        // 120s timeout — if the subprocess hangs (no event at all), bail with a
        // recoverable error so the caller drops the corpse and respawns fresh.
        loop {
            let mut buf = String::new();
            let n = match tokio::time::timeout(
                std::time::Duration::from_secs(120),
                self.stdout.read_line(&mut buf),
            )
            .await
            {
                Ok(r) => r?,
                Err(_) => anyhow::bail!("claude subprocess timed out (no event in 120s)"),
            };
            if n == 0 {
                anyhow::bail!("claude subprocess closed stdout");
            }
            let val: serde_json::Value = match serde_json::from_str(&buf) {
                Ok(v) => v,
                Err(_) => continue, // skip non-JSON lines
            };
            let t = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if t == "result" {
                let result = val
                    .get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(result);
            }
        }
    }

    /// Streaming counterpart of `ask` (Chunk 2). Sends the prompt, then reads
    /// events and FORWARDS them to `sink` as they arrive (assistant text blocks,
    /// tool_use/tool_result), until the `result` event — which is CLASSIFIED and
    /// returned as a `BrainOutcome`. Preserves the same 120s per-event timeout +
    /// recoverable-error semantics as `ask` (so the caller can drop the corpse
    /// and respawn). `sink` send errors are ignored — a dropped consumer (chat
    /// went away) must not abort the turn.
    pub async fn ask_streaming(
        &mut self,
        prompt: &str,
        sink: &mpsc::Sender<StreamEvent>,
    ) -> Result<BrainOutcome> {
        let msg = serde_json::json!({
            "type": "user",
            "message": { "role": "user", "content": prompt }
        });
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        loop {
            let mut buf = String::new();
            let n = match tokio::time::timeout(
                std::time::Duration::from_secs(120),
                self.stdout.read_line(&mut buf),
            )
            .await
            {
                Ok(r) => r?,
                Err(_) => anyhow::bail!("claude subprocess timed out (no event in 120s)"),
            };
            if n == 0 {
                anyhow::bail!("claude subprocess closed stdout");
            }
            let val: serde_json::Value = match serde_json::from_str(&buf) {
                Ok(v) => v,
                Err(_) => continue, // skip non-JSON lines
            };
            let t = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match t {
                "assistant" => {
                    // Forward each content block: text → AssistantText,
                    // tool_use → ToolUse trace.
                    if let Some(content) = val
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            let bt = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match bt {
                                "text" => {
                                    if let Some(s) =
                                        block.get("text").and_then(|v| v.as_str())
                                    {
                                        if !s.is_empty() {
                                            let _ = sink
                                                .send(StreamEvent::AssistantText(
                                                    s.to_string(),
                                                ))
                                                .await;
                                        }
                                    }
                                }
                                "tool_use" => {
                                    let name = block
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("tool")
                                        .to_string();
                                    // `--brief` SendUserMessage: an agent→user
                                    // push, NOT a normal tool call. Surface its
                                    // message text as a distinct mid-task alert
                                    // so the bridge can ping the human before the
                                    // turn finishes, instead of a tool-trace line.
                                    if name == "SendUserMessage" {
                                        let m = block
                                            .get("input")
                                            .and_then(|i| {
                                                i.get("message")
                                                    .or_else(|| i.get("text"))
                                            })
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        if !m.is_empty() {
                                            let _ = sink
                                                .send(StreamEvent::UserMessage(m))
                                                .await;
                                        }
                                    } else {
                                        let _ = sink
                                            .send(StreamEvent::ToolUse { name })
                                            .await;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "user" => {
                    // tool_result lines come back as a `user` event whose content
                    // carries `tool_result` blocks — surface a collapsed trace.
                    if let Some(content) = val
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        if content.iter().any(|b| {
                            b.get("type").and_then(|v| v.as_str()) == Some("tool_result")
                        }) {
                            let _ = sink.send(StreamEvent::ToolResult).await;
                        }
                    }
                }
                "stream_event" => {
                    // Partial-message mode (`--include-partial-messages`): raw
                    // Anthropic SSE wrapped in `event`. We care about two shapes;
                    // any other inner event type falls through harmlessly.
                    let inner = val.get("event");
                    let inner_type = inner
                        .and_then(|e| e.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match inner_type {
                        "content_block_delta" => {
                            // text_delta → token-level streaming. A non-text
                            // delta (e.g. input_json_delta on a tool call) has no
                            // `delta.text` → skipped (honest fallback).
                            if let Some(s) = inner
                                .and_then(|e| e.get("delta"))
                                .and_then(|d| d.get("text"))
                                .and_then(|v| v.as_str())
                            {
                                if !s.is_empty() {
                                    let _ = sink
                                        .send(StreamEvent::TextDelta(s.to_string()))
                                        .await;
                                }
                            }
                        }
                        "message_stop" => {
                            // Deterministic intra-turn boundary: one assistant
                            // message finished. NOT the turn end — the `result`
                            // event below still terminates the loop and carries
                            // the classified outcome.
                            let _ = sink.send(StreamEvent::MessageStop).await;
                        }
                        _ => {}
                    }
                }
                "result" => {
                    let is_error_flag =
                        val.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                    let subtype =
                        val.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
                    let result_text = val
                        .get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    return Ok(classify_result(is_error_flag, subtype, &result_text));
                }
                _ => {}
            }
        }
    }
}

/// Shared handle: lazy init on first use, kept alive for the bridge lifetime.
pub struct ClaudeStreamHandle {
    inner: Mutex<Option<PersistentClaude>>,
    config_dir: Option<PathBuf>,
}

impl ClaudeStreamHandle {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        Self {
            inner: Mutex::new(None),
            config_dir,
        }
    }

    /// Drop the current subprocess so the next `ask` spawns a brand-new
    /// Claude SDK session (fresh conversation). Used by /clean.
    pub async fn reset(&self) {
        let mut guard = self.inner.lock().await;
        *guard = None;
    }

    /// Ask claude. Lazy-spawns on first call. SELF-HEALING: if the persistent
    /// subprocess has died since the last turn (idle timeout, OOM, a bridge
    /// redeploy that orphaned it, etc.), the first write hits a broken pipe.
    /// We must NOT surface that to the user — instead drop the corpse, respawn
    /// a fresh subprocess, and retry the SAME prompt once. Only a second
    /// consecutive failure (i.e. claude genuinely can't start) is returned.
    pub async fn ask(&self, prompt: &str) -> Result<String> {
        let mut guard = self.inner.lock().await;
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            if guard.is_none() {
                match PersistentClaude::spawn(self.config_dir.clone()).await {
                    Ok(c) => *guard = Some(c),
                    Err(e) => {
                        last_err = Some(e);
                        continue;
                    }
                }
            }
            let claude = guard.as_mut().unwrap();
            match claude.ask(prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Dead pipe / closed stdout — drop the corpse so the next
                    // loop iteration respawns fresh and retries the prompt.
                    *guard = None;
                    last_err = Some(e);
                    if attempt == 0 {
                        tracing::warn!("claude brain subprocess died — respawning and retrying");
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("claude brain unavailable")))
    }

    /// Streaming counterpart of `ask` (Chunk 2). Same lazy-spawn + self-heal
    /// (respawn-and-retry-once) contract; forwards `StreamEvent`s to `sink` and
    /// returns the classified `BrainOutcome`. The death-of-subprocess case
    /// surfaces on the first stdin write (broken pipe) BEFORE any event reaches
    /// `sink`, so the retry never double-emits partial text.
    pub async fn ask_streaming(
        &self,
        prompt: &str,
        sink: &mpsc::Sender<StreamEvent>,
    ) -> Result<BrainOutcome> {
        let mut guard = self.inner.lock().await;
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            if guard.is_none() {
                match PersistentClaude::spawn(self.config_dir.clone()).await {
                    Ok(c) => *guard = Some(c),
                    Err(e) => {
                        last_err = Some(e);
                        continue;
                    }
                }
            }
            let claude = guard.as_mut().unwrap();
            match claude.ask_streaming(prompt, sink).await {
                Ok(outcome) => return Ok(outcome),
                Err(e) => {
                    *guard = None;
                    last_err = Some(e);
                    if attempt == 0 {
                        tracing::warn!(
                            "claude brain subprocess died (streaming) — respawning and retrying"
                        );
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("claude brain unavailable")))
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Brain actor (Chunk 1) — FIFO, single-consumer, single-writer.
//
// N1 root cause: the Telegram update loop `.await`ed each brain turn inline,
// so one 120s LLM turn blocked ALL other Telegram traffic (callbacks, /status,
// other chats) — the `(no response within 90s — check the bridge)` symptom.
//
// Fix: the update loop spawns each update (never blocks), and the brain lives
// behind a BOUNDED mpsc owned by ONE dedicated consumer task (R-SCOPE: one
// writer per subprocess). Concurrent messages enqueue instantly; the consumer
// drains them in arrival order (FIFO); each turn's result is delivered back to
// ITS chat via a `oneshot` reply channel. The existing self-heal/respawn and
// the 120s per-event timeout in `PersistentClaude` are preserved unchanged —
// the actor just calls `ClaudeStreamHandle::ask`.
// ─────────────────────────────────────────────────────────────────────────

/// Default bounded-channel capacity. Small on purpose: a single brain turn can
/// take up to 120s, so a deep queue only hides backpressure. At capacity, the
/// caller is told to surface a "queued…" note rather than the message being
/// dropped.
pub const BRAIN_QUEUE_CAPACITY: usize = 32;

/// One queued brain turn. `meta` (chat_id / msg_id) is carried purely for
/// structured tracing (Chunk 0) so logs prove a second message's *enqueue*
/// timestamp ≪ its *brain-start* timestamp under load.
struct BrainRequest {
    prompt: String,
    chat_id: i64,
    msg_id: i64,
    /// When the request was enqueued (loop side) — for the enqueue→start gap.
    enqueued_at: std::time::Instant,
    /// When `Some`, the consumer streams `StreamEvent`s here as the turn
    /// composes (Chunk 2). When `None`, it uses the block-return path.
    stream_sink: Option<mpsc::Sender<StreamEvent>>,
    /// Classified outcome is sent back to the originating chat's handler task.
    reply_tx: oneshot::Sender<Result<BrainOutcome>>,
}

/// Error returned when the bounded brain queue is full. The handler should edit
/// its placeholder with a "queued…" note (backpressure) rather than dropping.
#[derive(Debug)]
pub struct BrainQueueFull;

impl std::fmt::Display for BrainQueueFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "brain queue is full (backpressure)")
    }
}
impl std::error::Error for BrainQueueFull {}

/// Cloneable handle to the single brain consumer task. Cheap to clone (just an
/// `mpsc::Sender` + an `Arc` to the underlying stream handle for `/clean`).
#[derive(Clone)]
pub struct BrainActor {
    tx: mpsc::Sender<BrainRequest>,
    /// Kept so callers (e.g. `/clean`) can still reset the conversation. The
    /// consumer task owns the same `Arc`, so a reset on either side is visible
    /// to the next turn.
    handle: Arc<ClaudeStreamHandle>,
}

impl BrainActor {
    /// Spawn the consumer task and return a cloneable handle. The task owns the
    /// brain and processes turns FIFO from a bounded channel until every sender
    /// is dropped (bridge shutdown).
    pub fn spawn(config_dir: Option<PathBuf>) -> Self {
        Self::spawn_with_capacity(config_dir, BRAIN_QUEUE_CAPACITY)
    }

    pub fn spawn_with_capacity(config_dir: Option<PathBuf>, capacity: usize) -> Self {
        let handle = Arc::new(ClaudeStreamHandle::new(config_dir));
        let (tx, mut rx) = mpsc::channel::<BrainRequest>(capacity);
        let consumer_handle = handle.clone();
        tokio::spawn(async move {
            // Single consumer → FIFO, single-writer over the subprocess.
            while let Some(req) = rx.recv().await {
                let BrainRequest {
                    prompt,
                    chat_id,
                    msg_id,
                    enqueued_at,
                    stream_sink,
                    reply_tx,
                } = req;
                // Chunk 0: brain-start (consumer dequeued + about to acquire the
                // brain). enqueue→start gap proves head-of-line behavior is gone:
                // a second message's enqueue lands immediately even while the
                // first turn is still running.
                let queue_wait_ms = enqueued_at.elapsed().as_millis();
                tracing::info!(
                    chat_id,
                    msg_id,
                    queue_wait_ms,
                    "brain turn start (dequeued)"
                );
                let started = std::time::Instant::now();
                let result: Result<BrainOutcome> = match &stream_sink {
                    Some(sink) => consumer_handle.ask_streaming(&prompt, sink).await,
                    None => consumer_handle
                        .ask(&prompt)
                        .await
                        // Non-streaming callers don't classify; treat as content.
                        .map(|text| BrainOutcome {
                            text,
                            is_error: false,
                        }),
                };
                let elapsed_ms = started.elapsed().as_millis();
                match &result {
                    Ok(outcome) => tracing::info!(
                        chat_id,
                        msg_id,
                        elapsed_ms,
                        len = outcome.text.len(),
                        is_error = outcome.is_error,
                        "brain turn done"
                    ),
                    Err(e) => tracing::warn!(
                        chat_id,
                        msg_id,
                        elapsed_ms,
                        error = %e,
                        "brain turn failed"
                    ),
                }
                // Receiver may have been dropped if the handler task was
                // cancelled (e.g. chat went away). Ignore the send error.
                let _ = reply_tx.send(result);
            }
            tracing::info!("brain actor consumer stopped (all senders dropped)");
        });
        Self { tx, handle }
    }

    /// Enqueue a brain turn and await its result. Returns `BrainQueueFull` (via
    /// `try_send`) WITHOUT blocking the caller when the bounded queue is full,
    /// so the handler can show a "queued…" note instead of dropping the message.
    /// Block-return enqueue (no streaming). Kept per the Chunk-2 brief as a
    /// helper for non-streaming callers; the brain turn itself now streams.
    #[allow(dead_code)]
    pub async fn ask(&self, prompt: &str, chat_id: i64, msg_id: i64) -> Result<String> {
        self.ask_outcome(prompt, chat_id, msg_id, None)
            .await
            .map(|o| o.text)
    }

    /// Streaming enqueue (Chunk 2): hand the consumer a `StreamEvent` sink so the
    /// caller watches the answer compose, and get back the CLASSIFIED outcome
    /// (so a CLI-error result renders as an error card, not the answer body).
    /// `try_send` semantics identical to `ask` — full queue → `BrainQueueFull`.
    pub async fn ask_streaming(
        &self,
        prompt: &str,
        chat_id: i64,
        msg_id: i64,
        sink: mpsc::Sender<StreamEvent>,
    ) -> Result<BrainOutcome> {
        self.ask_outcome(prompt, chat_id, msg_id, Some(sink)).await
    }

    /// Shared try_send enqueue used by both `ask` (no sink) and `ask_streaming`.
    async fn ask_outcome(
        &self,
        prompt: &str,
        chat_id: i64,
        msg_id: i64,
        stream_sink: Option<mpsc::Sender<StreamEvent>>,
    ) -> Result<BrainOutcome> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let enqueued_at = std::time::Instant::now();
        // Chunk 0: enqueue time, keyed by chat. This lands instantly even when a
        // prior 120s turn is in flight — that's the whole point of the actor.
        tracing::info!(chat_id, msg_id, "brain turn enqueued");
        let req = BrainRequest {
            prompt: prompt.to_string(),
            chat_id,
            msg_id,
            enqueued_at,
            stream_sink,
            reply_tx,
        };
        // try_send → never blocks the loop/handler; full = explicit backpressure.
        self.tx
            .try_send(req)
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => anyhow::Error::new(BrainQueueFull),
                mpsc::error::TrySendError::Closed(_) => {
                    anyhow::anyhow!("brain actor channel closed")
                }
            })?;
        // Await this turn's result from the FIFO consumer.
        reply_rx
            .await
            .map_err(|_| anyhow::anyhow!("brain actor dropped the reply before answering"))?
    }

    /// Like `ask`, but WAITS for a free slot when the bounded queue is full
    /// (`send().await` applies backpressure instead of erroring). The handler
    /// uses this after showing a "queued…" note so a burst message is never
    /// dropped — it just waits its turn behind the FIFO.
    /// Block-return + backpressure-waiting enqueue (no streaming). Kept per the
    /// Chunk-2 brief as a helper for non-streaming callers.
    #[allow(dead_code)]
    pub async fn ask_waiting(&self, prompt: &str, chat_id: i64, msg_id: i64) -> Result<String> {
        self.ask_outcome_waiting(prompt, chat_id, msg_id, None)
            .await
            .map(|o| o.text)
    }

    /// Streaming + backpressure-waiting enqueue (Chunk 2). Combines
    /// `ask_streaming` (event sink + classified outcome) with `ask_waiting`'s
    /// `send().await` so a burst message waits its FIFO turn instead of being
    /// dropped when the bounded queue is full.
    pub async fn ask_streaming_waiting(
        &self,
        prompt: &str,
        chat_id: i64,
        msg_id: i64,
        sink: mpsc::Sender<StreamEvent>,
    ) -> Result<BrainOutcome> {
        self.ask_outcome_waiting(prompt, chat_id, msg_id, Some(sink))
            .await
    }

    /// Shared `send().await` enqueue used by both waiting variants.
    async fn ask_outcome_waiting(
        &self,
        prompt: &str,
        chat_id: i64,
        msg_id: i64,
        stream_sink: Option<mpsc::Sender<StreamEvent>>,
    ) -> Result<BrainOutcome> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let enqueued_at = std::time::Instant::now();
        tracing::info!(chat_id, msg_id, "brain turn enqueued (waiting for slot)");
        let req = BrainRequest {
            prompt: prompt.to_string(),
            chat_id,
            msg_id,
            enqueued_at,
            stream_sink,
            reply_tx,
        };
        self.tx
            .send(req)
            .await
            .map_err(|_| anyhow::anyhow!("brain actor channel closed"))?;
        reply_rx
            .await
            .map_err(|_| anyhow::anyhow!("brain actor dropped the reply before answering"))?
    }

    /// Reset the brain conversation (delegates to the underlying stream handle).
    /// Used by `/clean`. The consumer task shares the same `Arc`, so the next
    /// dequeued turn spawns a fresh subprocess.
    pub async fn reset(&self) {
        self.handle.reset().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIFO ordering: enqueue 3 turns, assert the consumer processes them in
    /// arrival order. We can't spawn a real `claude` subprocess in CI, so this
    /// test drives the *channel + consumer* contract directly with a stub
    /// consumer that mirrors `BrainActor::spawn`'s loop, proving the queue is
    /// single-consumer FIFO (the property Chunk 1 guarantees).
    #[tokio::test]
    async fn brain_actor_processes_fifo() {
        let (tx, mut rx) = mpsc::channel::<BrainRequest>(BRAIN_QUEUE_CAPACITY);
        let order: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let order_c = order.clone();
        let consumer = tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                // Record arrival order, then answer with the prompt echoed.
                order_c.lock().await.push(req.msg_id);
                let _ = req.reply_tx.send(Ok(BrainOutcome {
                    text: req.prompt.clone(),
                    is_error: false,
                }));
            }
        });

        // Enqueue 3 in order; collect their reply receivers.
        let mut rxs = Vec::new();
        for i in 1..=3i64 {
            let (reply_tx, reply_rx) = oneshot::channel();
            tx.try_send(BrainRequest {
                prompt: format!("msg-{i}"),
                chat_id: 100,
                msg_id: i,
                enqueued_at: std::time::Instant::now(),
                stream_sink: None,
                reply_tx,
            })
            .expect("queue not full");
            rxs.push((i, reply_rx));
        }
        drop(tx); // let the consumer finish
        for (i, reply_rx) in rxs {
            let outcome = reply_rx.await.expect("reply").expect("ok");
            assert_eq!(outcome.text, format!("msg-{i}"));
        }
        consumer.await.unwrap();
        assert_eq!(*order.lock().await, vec![1, 2, 3], "FIFO order violated");
    }

    /// classify_result: explicit error flag, error subtype, and known CLI-error
    /// signatures all classify as errors; a normal success body does not — and a
    /// genuine answer that merely discusses limits isn't misflagged.
    #[test]
    fn classify_result_flags_cli_errors() {
        assert!(classify_result(true, "success", "anything").is_error);
        assert!(classify_result(false, "error_max_turns", "x").is_error);
        assert!(
            classify_result(false, "success", "You've hit your session limit · resets 2:30am")
                .is_error
        );
        assert!(classify_result(false, "success", "Invalid API key · please run /login").is_error);
        let ok = classify_result(false, "success", "Here is the answer you asked for.");
        assert!(!ok.is_error);
        assert_eq!(ok.text, "Here is the answer you asked for.");
        // Topic mention without the CLI's own phrasing is NOT misflagged.
        assert!(
            !classify_result(false, "success", "Your plan has a generous usage allowance.")
                .is_error
        );
    }
}
