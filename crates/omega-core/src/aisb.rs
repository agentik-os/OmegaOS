use crate::agents::Agent;
use crate::session::SessionManager;
use anyhow::Result;

/// Canonical name of the Master AISB session — the always-on brain.
pub const MASTER_SESSION_NAME: &str = "aisb-master";

/// The system prompt for the Master AISB session.
/// Inlined at compile time so it always travels with the binary.
pub const MASTER_SYSTEM_PROMPT: &str = include_str!("../../../agents/aisb-master.md");

/// Ensures the Master AISB session exists. Idempotent — creates only if missing.
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

    // Launch with the AISB super-prompt as the initial message
    mgr.create_session_with_agent(
        MASTER_SESSION_NAME,
        Some(working_dir),
        agent,
        Some(MASTER_SYSTEM_PROMPT),
    )
    .await?;

    Ok(true)
}

/// Returns true if a given session name is the Master AISB.
pub fn is_master(name: &str) -> bool {
    name == MASTER_SESSION_NAME
}
