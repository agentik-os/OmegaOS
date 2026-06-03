use crate::agents::Agent;
use crate::session::SessionManager;
use anyhow::Result;

/// Canonical name of the Master AISB session — the always-on brain.
pub const MASTER_SESSION_NAME: &str = "aisb-master";

/// Ensures the Master AISB session exists. Idempotent — creates only if missing.
///
/// The session is a PURE READ-ONLY VIEWER: it `tail -F`s the conversation log
/// the brain stream writes, so the user can WATCH the live Telegram exchange
/// in the TUI. It is NOT the brain and NOT interactive (the brain is the
/// Telegram bot's own SDK subprocess — see `claude_stream.rs`).
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
    let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let log = home.join(".omega/state/aisb-conversation.log");
    // Surface filesystem errors instead of swallowing them: if the log can't be
    // created the viewer pane would `tail -F` a path that never appears, with no
    // diagnostic. A failed first call must error so the caller can retry; on a
    // successful create the log is guaranteed to exist for the idempotent path.
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !log.exists() {
        std::fs::write(
            &log,
            "  Ω  AISB Master — live Telegram conversation viewer\n\
             ─────────────────────────────────────────────────\n\
             Talk to AISB from Telegram. Exchanges stream here.\n",
        )?;
    }
    // The master pane is a PURE READ-ONLY MIRROR of the brain's
    // conversation: it tails the same `aisb-conversation.log` the brain
    // stream appends to via `mirror_to_master_pane`. It is NOT interactive
    // — there is one brain (the Telegram bot's SDK subprocess) and the user
    // talks to it from Telegram; this pane only lets them WATCH the live
    // exchange in the TUI. `exec` so the pane dies cleanly if the tail does.
    let log_path = log.to_string_lossy();
    let cmd = format!("exec tail -n 200 -F {}", shell_quote(&log_path));

    mgr.create_session(MASTER_SESSION_NAME, Some(working_dir), Some(&cmd))
        .await?;

    Ok(true)
}

/// Returns true if a given session name is the Master AISB.
pub fn is_master(name: &str) -> bool {
    name == MASTER_SESSION_NAME
}

/// Single-quote a path for safe interpolation into a shell command
/// (the viewer pane's `tail -F <path>`). Wraps in single quotes and
/// escapes any embedded single quote, so spaces/specials in the home
/// path can't break or inject into the command.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
