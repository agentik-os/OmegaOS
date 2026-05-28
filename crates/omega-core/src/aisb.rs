use crate::agents::{Agent, LaunchOptions};
use crate::session::SessionManager;
use anyhow::Result;
use std::path::PathBuf;

/// Canonical name of the Master AISB session — the always-on brain.
pub const MASTER_SESSION_NAME: &str = "aisb-master";

/// The system prompt for the Master AISB session.
/// Inlined at compile time so it always travels with the binary.
pub const MASTER_SYSTEM_PROMPT: &str = include_str!("../../../agents/aisb-master.md");

/// Returns the on-disk path where the AISB system prompt is materialised
/// for `--append-system-prompt-file`. Created on first use.
pub fn ensure_prompt_file() -> Result<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let path = home.join(".omega").join("aisb-master.system.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Always rewrite — guarantees the file matches the binary's embedded prompt
    std::fs::write(&path, MASTER_SYSTEM_PROMPT)?;
    Ok(path)
}

/// Ensures the Master AISB session exists. Idempotent — creates only if missing.
///
/// The system prompt is delivered via `--append-system-prompt-file` so it is
/// HIDDEN from the chat (it's instructions, not content). The conversation
/// resumes from the most recent thread via `--continue` so the user always
/// returns to the same flow.
///
/// Returns true if a new session was created, false if it already existed.
pub async fn ensure_master(
    mgr: &SessionManager,
    agent: Agent,
    working_dir: &str,
) -> Result<bool> {
    let _ = agent; // master is a viewer now, not an agent session
    let sessions = mgr.list_sessions().await?;
    if sessions.iter().any(|s| s.name == MASTER_SESSION_NAME) {
        return Ok(false);
    }

    // NEW MODEL (2026-05-28): the Telegram bot owns its OWN persistent
    // Claude SDK subprocess (claude_stream.rs) with full VPS access — that
    // is the brain. The aisb-master rmux session is now a LIVE VIEWER that
    // tails the conversation log the bridge writes, so the user can WATCH
    // the Telegram chat stream in the TUI. It is no longer an interactive
    // Claude (that caused the "talks in the pane but not Telegram" split).
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home/hacker"));
    let log = home.join(".omega/state/aisb-conversation.log");
    if let Some(parent) = log.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !log.exists() {
        let _ = std::fs::write(
            &log,
            "  Ω  AISB Master — live Telegram conversation viewer\n\
             ─────────────────────────────────────────────────\n\
             Talk to AISB from Telegram. Exchanges stream here.\n",
        );
    }
    let _ = log; // ensured to exist above; the REPL replays + tails it
    // The master pane runs the interactive chat REPL: you type, it goes
    // to the bot exactly like a Telegram message, the reply shows here +
    // in Telegram. `exec bash` keeps the pane alive if the REPL exits.
    let cmd = "exec omega aisb-chat".to_string();

    mgr.create_session(MASTER_SESSION_NAME, Some(working_dir), Some(&cmd))
        .await?;

    Ok(true)
}

/// Minimal shell-quote for the log path (single-quote wrap).
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Returns true if a given session name is the Master AISB.
pub fn is_master(name: &str) -> bool {
    name == MASTER_SESSION_NAME
}
