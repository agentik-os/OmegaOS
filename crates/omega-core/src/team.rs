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
