//! Chat process driver — spawns a headless CLI agent (`claude -p`) and parses
//! its NDJSON stdout stream into typed [`ChatStreamServerMsg`] frames.
//!
//! Three pure/composable pieces:
//! - [`agent_command`] builds the child-process invocation (no I/O, unit-testable).
//! - [`parse_line`] parses one NDJSON stdout line into a [`ParsedLine`] (no I/O).
//! - [`run_turn`] spawns the process, drives the read loop, and forwards frames.
//!
//! KNOWN LIMIT: `ChatAgent::Codex` streaming JSON support is not implemented —
//! [`run_turn`] intercepts it before ever building or spawning a process, so a
//! Codex chat never spawns `claude`.

use crate::protocol::{ChatAgent, ChatMeta, ChatStreamServerMsg};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

/// Builds the child-process command for a Claude chat turn.
///
/// Only meaningful for `ChatAgent::Claude` — [`run_turn`] intercepts
/// `ChatAgent::Codex` before ever calling this. The precondition is
/// enforced with a real (non-debug-only) `assert!` so misuse is caught in
/// every build, not just debug ones, rather than silently spawning `claude`
/// for a Codex chat.
///
/// Program is `$OMEGA_CHAT_BIN` when set, else `claude`. Args:
/// `-p <user_text> --output-format stream-json --verbose`, plus
/// `--resume <provider_session_id>` when `meta.provider_session_id` is
/// `Some`, plus `--model <m>` when `model` is `Some`. `current_dir` is
/// `meta.cwd`; when `account_dir` is `Some`, `CLAUDE_CONFIG_DIR` is set.
pub fn agent_command(
    meta: &ChatMeta,
    user_text: &str,
    model: Option<&str>,
    account_dir: Option<&Path>,
) -> Command {
    assert!(
        meta.agent == ChatAgent::Claude,
        "agent_command is Claude-only; Codex is handled in run_turn"
    );

    let program = std::env::var("OMEGA_CHAT_BIN").unwrap_or_else(|_| "claude".to_string());
    let mut cmd = Command::new(program);
    cmd.arg("-p")
        .arg(user_text)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose");
    if let Some(session_id) = &meta.provider_session_id {
        cmd.arg("--resume").arg(session_id);
    }
    if let Some(model) = model {
        cmd.arg("--model").arg(model);
    }
    cmd.current_dir(&meta.cwd);
    if let Some(dir) = account_dir {
        cmd.env("CLAUDE_CONFIG_DIR", dir);
    }
    cmd
}

/// One thing discovered while parsing an NDJSON stdout line from the agent
/// process. A single line can yield zero, one, or several of these — an
/// empty `Vec<ParsedLine>` from [`parse_line`] means the line carried
/// nothing the client needs (thinking blocks, hook events, rate-limit
/// events, unparseable text, ...).
pub enum ParsedLine {
    /// A frame to forward to the client.
    Frame(ChatStreamServerMsg),
    /// The provider's session id, discovered from an `init` line.
    Session(String),
}

/// Parses one NDJSON stdout line from `claude -p --output-format stream-json`.
/// Pure and I/O-free: unparseable or irrelevant lines yield an empty `Vec`, never a panic.
///
/// An assistant line's `content[]` blocks are walked IN ORDER and each block
/// contributes independently, so a single line can yield multiple entries
/// (e.g. a `text` block followed by a `tool_use` block yields both an
/// `AssistantMessage` and a `ToolEvent`).
///
/// Real observed shapes (claude 2.1.226):
/// - `{"type":"system","subtype":"init","session_id":"..."}` -> `[Session(id)]`
/// - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}` -> `[Frame(AssistantMessage)]`
///   (consecutive text blocks are concatenated into one `AssistantMessage`; a
///   `"thinking"` block is skipped; a `"tool_use"` block yields its own
///   `Frame(ToolEvent)`)
/// - `{"type":"result","is_error":false,...}` -> `[Frame(TurnDone)]`
/// - `{"type":"result","is_error":true,"result":"msg"}` -> `[Frame(Error{message: "msg"})]`
/// - `rate_limit_event`, `system`/`hook_started`, or anything else unparseable -> `[]`
pub fn parse_line(line: &str) -> Vec<ParsedLine> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };

    match v.get("type").and_then(Value::as_str) {
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            match v.get("session_id").and_then(Value::as_str) {
                Some(id) => vec![ParsedLine::Session(id.to_string())],
                None => Vec::new(),
            }
        }
        Some("assistant") => {
            let Some(blocks) = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
            else {
                return Vec::new();
            };

            let mut out = Vec::new();
            let mut text = String::new();
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(t) = block.get("text").and_then(Value::as_str) {
                            text.push_str(t);
                        }
                    }
                    Some("tool_use") => {
                        if !text.is_empty() {
                            out.push(ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage {
                                text: std::mem::take(&mut text),
                            }));
                        }
                        let name = block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("tool")
                            .to_string();
                        let detail = block.get("input").map(compact_json);
                        out.push(ParsedLine::Frame(ChatStreamServerMsg::ToolEvent {
                            name,
                            detail,
                        }));
                    }
                    // "thinking" and anything else: ignored.
                    _ => {}
                }
            }
            if !text.is_empty() {
                out.push(ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }));
            }
            out
        }
        Some("result") => {
            let is_error = v.get("is_error").and_then(Value::as_bool).unwrap_or(false);
            if is_error {
                let message = v
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("agent turn failed")
                    .to_string();
                vec![ParsedLine::Frame(ChatStreamServerMsg::Error { message })]
            } else {
                vec![ParsedLine::Frame(ChatStreamServerMsg::TurnDone)]
            }
        }
        _ => Vec::new(),
    }
}

/// Renders a `serde_json::Value` as a compact one-line string, for use as a
/// `ToolEvent`'s `detail`.
fn compact_json(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` (`kill -- -
/// <pid>`) — the same idiom as `routes_duo.rs::kill_process_group`,
/// duplicated locally since that one is private to its own module. Needed
/// because `child.kill()` alone (I-3) only reaches the DIRECT agent
/// process, never a nested process it spawned into the same group (see
/// `run_turn`'s `process_group(0)` on its `Command`).
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill").arg("--").arg(format!("-{pid}")).status()
    })
    .await;
}

/// Spawns the agent process for one chat turn, forwards parsed frames on
/// `tx`, and returns the provider session id discovered from the stream (if
/// any). The child is killed if `tx`'s receiver is dropped mid-stream, on
/// timeout, and (via `kill_on_drop`) if this future is itself dropped/cancelled.
/// A final `TurnDone` is always sent if the stream didn't already carry one.
///
/// KNOWN LIMIT: `ChatAgent::Codex` never spawns a process — it sends an
/// `Error` then `TurnDone` and returns `None` immediately.
pub async fn run_turn(
    meta: &ChatMeta,
    user_text: &str,
    model: Option<&str>,
    account_dir: Option<&Path>,
    timeout: Duration,
    tx: Sender<ChatStreamServerMsg>,
) -> Option<String> {
    if meta.agent == ChatAgent::Codex {
        let _ = tx
            .send(ChatStreamServerMsg::Error {
                message: "codex chat not yet supported".to_string(),
            })
            .await;
        let _ = tx.send(ChatStreamServerMsg::TurnDone).await;
        return None;
    }

    let mut cmd = agent_command(meta, user_text, model, account_dir);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    cmd.kill_on_drop(true);
    // I-3: place the child in its own process group so a nested process it
    // spawns (Claude may spawn nested tool processes) stays reachable by a
    // whole-group kill below — `child.kill()` alone only reaches this
    // direct process, never anything nested under it.
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = tx
                .send(ChatStreamServerMsg::Error {
                    message: format!("failed to spawn agent: {e}"),
                })
                .await;
            let _ = tx.send(ChatStreamServerMsg::TurnDone).await;
            return None;
        }
    };

    // Captured now, before `child` is consumed by anything else (`Child::
    // id()` returns `None` once the child has been polled to completion) —
    // used to reach a nested process via the whole-group kill on every kill
    // path below.
    let child_pid = child.id();

    let stdout = child.stdout.take().expect("stdout was piped");
    let mut lines = BufReader::new(stdout).lines();
    let mut session_id: Option<String> = None;
    let mut sent_turn_done = false;

    let read_loop = async {
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    for parsed in parse_line(&line) {
                        match parsed {
                            ParsedLine::Session(id) => session_id = Some(id),
                            ParsedLine::Frame(frame) => {
                                if matches!(frame, ChatStreamServerMsg::TurnDone) {
                                    sent_turn_done = true;
                                }
                                if tx.send(frame).await.is_err() {
                                    // Receiver dropped: the caller no longer
                                    // wants frames, so stop reading and kill
                                    // the child, AND (I-3) the whole process
                                    // group so a nested process it spawned
                                    // does not survive it.
                                    let _ = child.kill().await;
                                    if let Some(pid) = child_pid {
                                        kill_process_group(pid).await;
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
                // EOF or a stdout read error both mean the stream is over.
                Ok(None) | Err(_) => return,
            }
        }
    };

    if tokio::time::timeout(timeout, read_loop).await.is_err() {
        let _ = child.kill().await;
        // I-3: also kill the whole process group, not just this direct
        // child, so a nested process (e.g. a tool the agent shelled out to)
        // does not outlive the timeout.
        if let Some(pid) = child_pid {
            kill_process_group(pid).await;
        }
        let _ = tx
            .send(ChatStreamServerMsg::Error {
                message: "agent turn timed out".to_string(),
            })
            .await;
        let _ = tx.send(ChatStreamServerMsg::TurnDone).await;
        return session_id;
    }

    let _ = child.wait().await;
    if !sent_turn_done {
        let _ = tx.send(ChatStreamServerMsg::TurnDone).await;
    }
    session_id
}

#[cfg(test)]
mod tests {
    use super::*;

    // agent_command reads the process-global OMEGA_CHAT_BIN env var, so any
    // test that sets it must be serialized against the others (same pattern
    // as the rmux tests' OMEGA_RMUX_BIN lock; tokio::sync::Mutex because the
    // guard is held across .await points in these #[tokio::test]s).
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn test_meta(provider_session_id: Option<&str>) -> ChatMeta {
        ChatMeta {
            id: "chat1".to_string(),
            title: None,
            agent: ChatAgent::Claude,
            cwd: "/tmp/proj".to_string(),
            created_at: "t".to_string(),
            updated_at: "t".to_string(),
            provider_session_id: provider_session_id.map(str::to_string),
            account_slug: None,
        }
    }

    #[tokio::test]
    async fn agent_command_uses_omega_chat_bin_and_base_flags() {
        let _g = LOCK.lock().await;
        std::env::set_var("OMEGA_CHAT_BIN", "/usr/bin/fake-claude");
        let meta = test_meta(None);
        let cmd = agent_command(&meta, "hello", None, None);
        std::env::remove_var("OMEGA_CHAT_BIN");

        let std_cmd = cmd.as_std();
        assert_eq!(std_cmd.get_program().to_str().unwrap(), "/usr/bin/fake-claude");
        let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(args, vec!["-p", "hello", "--output-format", "stream-json", "--verbose"]);
        assert_eq!(std_cmd.get_current_dir().unwrap().to_str().unwrap(), "/tmp/proj");
    }

    #[tokio::test]
    async fn agent_command_defaults_to_claude_when_env_unset() {
        let _g = LOCK.lock().await;
        std::env::remove_var("OMEGA_CHAT_BIN");
        let meta = test_meta(None);
        let cmd = agent_command(&meta, "hello", None, None);

        let std_cmd = cmd.as_std();
        assert_eq!(std_cmd.get_program().to_str().unwrap(), "claude");
    }

    #[tokio::test]
    async fn agent_command_adds_resume_when_provider_session_set() {
        let _g = LOCK.lock().await;
        std::env::set_var("OMEGA_CHAT_BIN", "/usr/bin/fake-claude");
        let meta = test_meta(Some("sess-123"));
        let cmd = agent_command(&meta, "hi", None, None);
        std::env::remove_var("OMEGA_CHAT_BIN");

        let std_cmd = cmd.as_std();
        let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();
        assert!(args.windows(2).any(|w| w == ["--resume", "sess-123"]));
    }

    #[tokio::test]
    async fn agent_command_adds_model_when_given() {
        let _g = LOCK.lock().await;
        std::env::set_var("OMEGA_CHAT_BIN", "/usr/bin/fake-claude");
        let meta = test_meta(None);
        let cmd = agent_command(&meta, "hi", Some("claude-fable-5"), None);
        std::env::remove_var("OMEGA_CHAT_BIN");

        let std_cmd = cmd.as_std();
        let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();
        assert!(args.windows(2).any(|w| w == ["--model", "claude-fable-5"]));
    }

    #[test]
    #[should_panic(expected = "agent_command is Claude-only")]
    fn agent_command_panics_for_codex_agent() {
        // Real (non-debug-only) assert: this must fire in every build
        // profile, not just debug, since run_turn's own short-circuit is
        // the primary guard and this is the belt-and-suspenders backstop.
        let mut meta = test_meta(None);
        meta.agent = ChatAgent::Codex;
        let _ = agent_command(&meta, "hi", None, None);
    }

    #[tokio::test]
    async fn agent_command_sets_claude_config_dir_when_account_dir_given() {
        let _g = LOCK.lock().await;
        std::env::set_var("OMEGA_CHAT_BIN", "/usr/bin/fake-claude");
        let meta = test_meta(None);
        let dir = std::path::PathBuf::from("/tmp/acct");
        let cmd = agent_command(&meta, "hi", None, Some(&dir));
        std::env::remove_var("OMEGA_CHAT_BIN");

        let std_cmd = cmd.as_std();
        let (_, val) = std_cmd
            .get_envs()
            .find(|(k, _)| *k == std::ffi::OsStr::new("CLAUDE_CONFIG_DIR"))
            .expect("CLAUDE_CONFIG_DIR should be set");
        assert_eq!(val.unwrap().to_str().unwrap(), "/tmp/acct");
    }

    #[tokio::test]
    async fn agent_command_omits_claude_config_dir_when_account_dir_absent() {
        let _g = LOCK.lock().await;
        std::env::set_var("OMEGA_CHAT_BIN", "/usr/bin/fake-claude");
        let meta = test_meta(None);
        let cmd = agent_command(&meta, "hi", None, None);
        std::env::remove_var("OMEGA_CHAT_BIN");

        let std_cmd = cmd.as_std();
        assert!(std_cmd
            .get_envs()
            .find(|(k, _)| *k == std::ffi::OsStr::new("CLAUDE_CONFIG_DIR"))
            .is_none());
    }

    #[test]
    fn parse_line_init_captures_session_id() {
        let line = r#"{"type":"system","subtype":"init","session_id":"3d48bb5b-abc","cwd":"/tmp","model":"claude-fable-5"}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Session(id) => assert_eq!(id, "3d48bb5b-abc"),
            _ => panic!("expected Session"),
        }
    }

    #[test]
    fn parse_line_assistant_text_yields_assistant_message() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"PONG"}]},"session_id":"s1"}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }) => {
                assert_eq!(text, "PONG");
            }
            _ => panic!("expected AssistantMessage frame"),
        }
    }

    #[test]
    fn parse_line_multiple_text_blocks_are_concatenated() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"foo"},{"type":"text","text":"bar"}]}}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }) => {
                assert_eq!(text, "foobar");
            }
            _ => panic!("expected AssistantMessage frame"),
        }
    }

    #[test]
    fn parse_line_thinking_block_is_ignored() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"pondering"}]}}"#;
        assert!(parse_line(line).is_empty());
    }

    #[test]
    fn parse_line_tool_use_block_yields_tool_event() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/tmp/x"}}]}}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Frame(ChatStreamServerMsg::ToolEvent { name, detail }) => {
                assert_eq!(name, "Read");
                assert!(detail.unwrap().contains("file_path"));
            }
            _ => panic!("expected ToolEvent frame"),
        }
    }

    #[test]
    fn parse_line_tool_use_without_name_defaults_to_tool() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","input":{}}]}}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Frame(ChatStreamServerMsg::ToolEvent { name, .. }) => {
                assert_eq!(name, "tool");
            }
            _ => panic!("expected ToolEvent frame"),
        }
    }

    #[test]
    fn parse_line_text_and_tool_use_yield_both_frames_in_order() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"looking it up"},{"type":"tool_use","id":"t1","name":"Grep","input":{"pattern":"foo"}}]}}"#;
        let out = parse_line(line);
        assert_eq!(out.len(), 2, "expected an AssistantMessage and a ToolEvent");
        match &out[0] {
            ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }) => {
                assert_eq!(text, "looking it up");
            }
            _ => panic!("expected AssistantMessage frame first"),
        }
        match &out[1] {
            ParsedLine::Frame(ChatStreamServerMsg::ToolEvent { name, .. }) => {
                assert_eq!(name, "Grep");
            }
            _ => panic!("expected ToolEvent frame second"),
        }
    }

    #[test]
    fn parse_line_result_success_yields_turn_done() {
        let line = r#"{"type":"result","is_error":false,"stop_reason":"end_turn","result":"PONG","session_id":"s1"}"#;
        let out = parse_line(line);
        assert_eq!(out.len(), 1);
        assert!(matches!(&out[0], ParsedLine::Frame(ChatStreamServerMsg::TurnDone)));
    }

    #[test]
    fn parse_line_result_error_yields_error_frame() {
        let line = r#"{"type":"result","is_error":true,"result":"boom"}"#;
        let mut out = parse_line(line);
        assert_eq!(out.len(), 1);
        match out.remove(0) {
            ParsedLine::Frame(ChatStreamServerMsg::Error { message }) => {
                assert_eq!(message, "boom");
            }
            _ => panic!("expected Error frame"),
        }
    }

    #[test]
    fn parse_line_rate_limit_event_is_ignored() {
        let line = r#"{"type":"rate_limit_event","limit":100}"#;
        assert!(parse_line(line).is_empty());
    }

    #[test]
    fn parse_line_hook_started_is_ignored() {
        let line = r#"{"type":"system","subtype":"hook_started"}"#;
        assert!(parse_line(line).is_empty());
    }

    #[test]
    fn parse_line_unparseable_is_ignored() {
        assert!(parse_line("not json at all").is_empty());
    }

    #[test]
    fn parse_line_empty_is_ignored() {
        assert!(parse_line("").is_empty());
    }
}
