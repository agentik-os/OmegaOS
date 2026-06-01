use crate::session::SessionManager;
use anyhow::{Context, Result};
use rmux_sdk::SplitDirection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub prompt: String,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub project: String,
    pub session_name: String,
    pub working_dir: String,
    pub agent_command: String,
    pub members: Vec<TeamMember>,
}

pub struct TeamSpawner<'a> {
    session_mgr: &'a SessionManager,
}

impl<'a> TeamSpawner<'a> {
    pub fn new(session_mgr: &'a SessionManager) -> Self {
        Self { session_mgr }
    }

    pub async fn spawn_team(&self, config: &TeamConfig) -> Result<Vec<String>> {
        let session = self
            .session_mgr
            .create_session(
                &config.session_name,
                Some(&config.working_dir),
                None,
            )
            .await
            .context("Failed to create team session")?;

        let first_pane = session.pane(0, 0);
        let mut pane_names = Vec::new();

        for (i, member) in config.members.iter().enumerate() {
            let agent_prompt = format!(
                "[DISPATCHED] Team member: {} ({})\n\
                 Third Law: decide and proceed, never wait.\n\n\
                 {}\n\n\
                 Files owned: {}\n\
                 When done: omega done {}-{} done_clean \"<summary>\"",
                member.name,
                member.role,
                member.prompt,
                if member.files_owned.is_empty() {
                    "none (read-only)".to_string()
                } else {
                    member.files_owned.join(", ")
                },
                config.session_name,
                member.name,
            );

            let cmd = format!(
                "{} -p {}",
                config.agent_command,
                shell_escape(&agent_prompt)
            );

            if i == 0 {
                first_pane.send_text(&cmd).await?;
                first_pane.send_key("Enter").await?;
            } else {
                let direction = if i % 2 == 1 {
                    SplitDirection::Right
                } else {
                    SplitDirection::Down
                };
                let new_pane = first_pane
                    .split_with(direction)
                    .shell(&cmd)
                    .await?;
                new_pane.set_title(&member.name).await?;
            }

            pane_names.push(format!("{}-{}", config.session_name, member.name));
        }

        // Even out the grid. The alternating Right/Down splits above produce a
        // lopsided binary-tree layout (pane 0 stays huge, later panes get
        // cramped) — on a client smaller than the spawn size that reads as
        // "empty space + agents you have to scroll to find". `tiled` arranges
        // every member in an even grid that reflows proportionally when the
        // window resizes to the attaching client, so the grid stays balanced
        // at attach. Best-effort: a layout hiccup must never fail the spawn.
        if config.members.len() > 1 {
            match tokio::process::Command::new("rmux")
                .args(["select-layout", "-t", &config.session_name, "tiled"])
                .output()
                .await
            {
                Ok(o) if !o.status.success() => tracing::warn!(
                    team = %config.session_name,
                    stderr = %String::from_utf8_lossy(&o.stderr),
                    "select-layout tiled failed (panes left in zigzag layout)"
                ),
                Err(e) => tracing::warn!(
                    team = %config.session_name,
                    error = %e,
                    "could not run rmux select-layout (panes left in zigzag layout)"
                ),
                _ => {}
            }
        }

        tracing::info!(
            team = %config.session_name,
            members = config.members.len(),
            "Team spawned"
        );

        Ok(pane_names)
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
