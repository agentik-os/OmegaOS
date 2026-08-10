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
/// `ChatAgent::Codex` before ever calling this (debug builds assert the
/// precondition so misuse is caught in tests rather than silently spawning
/// `claude` for a Codex chat).
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
    debug_assert_eq!(
        meta.agent,
        ChatAgent::Claude,
        "agent_command only builds Claude commands; ChatAgent::Codex must be intercepted by run_turn"
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

/// The outcome of parsing one NDJSON stdout line from the agent process.
pub enum ParsedLine {
    /// A frame to forward to the client.
    Frame(ChatStreamServerMsg),
    /// The provider's session id, discovered from an `init` line.
    Session(String),
    /// A line carrying nothing the client needs (thinking blocks, hook
    /// events, rate-limit events, unparseable text, ...).
    Ignore,
}

/// Parses one NDJSON stdout line from `claude -p --output-format stream-json`.
/// Pure and I/O-free: unparseable or irrelevant lines are `Ignore`, never a panic.
///
/// Real observed shapes (claude 2.1.226):
/// - `{"type":"system","subtype":"init","session_id":"..."}` -> `Session(id)`
/// - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}` -> `Frame(AssistantMessage)`
///   (text content blocks are concatenated; a message with only `"thinking"` blocks is `Ignore`)
/// - `{"type":"result","is_error":false,...}` -> `Frame(TurnDone)`
/// - `{"type":"result","is_error":true,"result":"msg"}` -> `Frame(Error{message: "msg"})`
/// - `rate_limit_event`, `system`/`hook_started`, or anything else unparseable -> `Ignore`
pub fn parse_line(line: &str) -> ParsedLine {
    let line = line.trim();
    if line.is_empty() {
        return ParsedLine::Ignore;
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return ParsedLine::Ignore;
    };

    match v.get("type").and_then(Value::as_str) {
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            match v.get("session_id").and_then(Value::as_str) {
                Some(id) => ParsedLine::Session(id.to_string()),
                None => ParsedLine::Ignore,
            }
        }
        Some("assistant") => {
            let text = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
                .map(|blocks| {
                    blocks
                        .iter()
                        .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                        .filter_map(|b| b.get("text").and_then(Value::as_str))
                        .collect::<String>()
                })
                .unwrap_or_default();
            if text.is_empty() {
                ParsedLine::Ignore
            } else {
                ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text })
            }
        }
        Some("result") => {
            let is_error = v.get("is_error").and_then(Value::as_bool).unwrap_or(false);
            if is_error {
                let message = v
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("agent turn failed")
                    .to_string();
                ParsedLine::Frame(ChatStreamServerMsg::Error { message })
            } else {
                ParsedLine::Frame(ChatStreamServerMsg::TurnDone)
            }
        }
        _ => ParsedLine::Ignore,
    }
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

    let stdout = child.stdout.take().expect("stdout was piped");
    let mut lines = BufReader::new(stdout).lines();
    let mut session_id: Option<String> = None;
    let mut sent_turn_done = false;

    let read_loop = async {
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => match parse_line(&line) {
                    ParsedLine::Session(id) => session_id = Some(id),
                    ParsedLine::Frame(frame) => {
                        if matches!(frame, ChatStreamServerMsg::TurnDone) {
                            sent_turn_done = true;
                        }
                        if tx.send(frame).await.is_err() {
                            // Receiver dropped: the caller no longer wants
                            // frames, so stop reading and kill the child.
                            let _ = child.kill().await;
                            return;
                        }
                    }
                    ParsedLine::Ignore => {}
                },
                // EOF or a stdout read error both mean the stream is over.
                Ok(None) | Err(_) => return,
            }
        }
    };

    if tokio::time::timeout(timeout, read_loop).await.is_err() {
        let _ = child.kill().await;
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
        match parse_line(line) {
            ParsedLine::Session(id) => assert_eq!(id, "3d48bb5b-abc"),
            _ => panic!("expected Session"),
        }
    }

    #[test]
    fn parse_line_assistant_text_yields_assistant_message() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"PONG"}]},"session_id":"s1"}"#;
        match parse_line(line) {
            ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }) => {
                assert_eq!(text, "PONG");
            }
            _ => panic!("expected AssistantMessage frame"),
        }
    }

    #[test]
    fn parse_line_multiple_text_blocks_are_concatenated() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"foo"},{"type":"text","text":"bar"}]}}"#;
        match parse_line(line) {
            ParsedLine::Frame(ChatStreamServerMsg::AssistantMessage { text }) => {
                assert_eq!(text, "foobar");
            }
            _ => panic!("expected AssistantMessage frame"),
        }
    }

    #[test]
    fn parse_line_thinking_block_is_ignored() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"pondering"}]}}"#;
        assert!(matches!(parse_line(line), ParsedLine::Ignore));
    }

    #[test]
    fn parse_line_result_success_yields_turn_done() {
        let line = r#"{"type":"result","is_error":false,"stop_reason":"end_turn","result":"PONG","session_id":"s1"}"#;
        assert!(matches!(parse_line(line), ParsedLine::Frame(ChatStreamServerMsg::TurnDone)));
    }

    #[test]
    fn parse_line_result_error_yields_error_frame() {
        let line = r#"{"type":"result","is_error":true,"result":"boom"}"#;
        match parse_line(line) {
            ParsedLine::Frame(ChatStreamServerMsg::Error { message }) => {
                assert_eq!(message, "boom");
            }
            _ => panic!("expected Error frame"),
        }
    }

    #[test]
    fn parse_line_rate_limit_event_is_ignored() {
        let line = r#"{"type":"rate_limit_event","limit":100}"#;
        assert!(matches!(parse_line(line), ParsedLine::Ignore));
    }

    #[test]
    fn parse_line_hook_started_is_ignored() {
        let line = r#"{"type":"system","subtype":"hook_started"}"#;
        assert!(matches!(parse_line(line), ParsedLine::Ignore));
    }

    #[test]
    fn parse_line_unparseable_is_ignored() {
        assert!(matches!(parse_line("not json at all"), ParsedLine::Ignore));
    }

    #[test]
    fn parse_line_empty_is_ignored() {
        assert!(matches!(parse_line(""), ParsedLine::Ignore));
    }
}
