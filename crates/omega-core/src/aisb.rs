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
    let sessions = mgr.list_sessions().await?;
    if sessions.iter().any(|s| s.name == MASTER_SESSION_NAME) {
        return Ok(false);
    }

    let prompt_file = ensure_prompt_file()?;
    let opts = LaunchOptions {
        system_prompt_file: Some(prompt_file.to_string_lossy().to_string()),
        resume_conversation: true,
        ..LaunchOptions::default()
    };
    let cmd = agent.launch_command_with(None, opts);

    mgr.create_session(MASTER_SESSION_NAME, Some(working_dir), Some(&cmd))
        .await?;

    Ok(true)
}

/// Returns true if a given session name is the Master AISB.
pub fn is_master(name: &str) -> bool {
    name == MASTER_SESSION_NAME
}
