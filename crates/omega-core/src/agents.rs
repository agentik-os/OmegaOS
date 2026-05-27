use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    Claude,
    Codex,
    Gemini,
    Pi,
    Glm,
    Shell,
}

impl Agent {
    pub fn all() -> &'static [Agent] {
        &[
            Agent::Claude,
            Agent::Codex,
            Agent::Gemini,
            Agent::Pi,
            Agent::Glm,
            Agent::Shell,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Gemini => "gemini",
            Agent::Pi => "pi",
            Agent::Glm => "glm",
            Agent::Shell => "shell",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Agent::Claude => "Claude Code (Anthropic)",
            Agent::Codex => "Codex (OpenAI)",
            Agent::Gemini => "Gemini (Google)",
            Agent::Pi => "Pi (earendil coding-agent)",
            Agent::Glm => "GLM (Z.AI / Zhipu)",
            Agent::Shell => "Plain shell",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Agent::Claude),
            "codex" => Some(Agent::Codex),
            "gemini" => Some(Agent::Gemini),
            "pi" => Some(Agent::Pi),
            "glm" => Some(Agent::Glm),
            "shell" | "bash" | "" => Some(Agent::Shell),
            _ => None,
        }
    }

    /// Returns the shell command to launch this agent.
    /// `initial_prompt` is the first message sent to the agent (if it supports it).
    pub fn launch_command(&self, initial_prompt: Option<&str>) -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/hacker".to_string());

        match self {
            Agent::Claude => {
                // Interactive Claude with permissions bypass + prompt as first message
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "claude --dangerously-skip-permissions {}; exec bash",
                            shell_quote(p)
                        ))
                    ),
                    None => "claude --dangerously-skip-permissions".to_string(),
                }
            }
            Agent::Codex => match initial_prompt {
                Some(p) => format!(
                    "bash -c {}",
                    shell_quote(&format!("codex {}; exec bash", shell_quote(p)))
                ),
                None => "codex".to_string(),
            },
            Agent::Gemini => {
                // Try alias first, fall back to npm-global, fall back to plain gemini
                let gemini_bin = format!("{}/.npm-global/bin/gemini", home);
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{} {}; exec bash",
                            gemini_bin,
                            shell_quote(p)
                        ))
                    ),
                    None => gemini_bin,
                }
            }
            Agent::Pi => {
                let pi_bin = format!("{}/.npm-global/bin/pi", home);
                let pi_args = "--provider openrouter --model anthropic/claude-sonnet-4.6";
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{} {} {}; exec bash",
                            pi_bin,
                            pi_args,
                            shell_quote(p)
                        ))
                    ),
                    None => format!("{} {}", pi_bin, pi_args),
                }
            }
            Agent::Glm => {
                // GLM via z-ai cli — falls back to a helpful message if not installed
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "if command -v glm >/dev/null 2>&1; then glm {}; else echo 'GLM CLI not installed. Install: npm install -g @z-ai/glm-cli'; fi; exec bash",
                            shell_quote(p)
                        ))
                    ),
                    None => "bash -c \"if command -v glm >/dev/null 2>&1; then glm; else echo 'GLM CLI not installed. Install: npm install -g @z-ai/glm-cli'; fi; exec bash\"".to_string(),
                }
            }
            Agent::Shell => match initial_prompt {
                Some(p) => format!(
                    "bash -c {}",
                    shell_quote(&format!("echo {}; exec bash", shell_quote(p)))
                ),
                None => "bash".to_string(),
            },
        }
    }

    pub fn is_available(&self) -> bool {
        let home = std::env::var("HOME").unwrap_or_default();
        match self {
            Agent::Claude => has_cmd("claude"),
            Agent::Codex => has_cmd("codex"),
            Agent::Gemini => {
                has_cmd("gemini")
                    || std::path::Path::new(&format!("{}/.npm-global/bin/gemini", home)).exists()
            }
            Agent::Pi => {
                std::path::Path::new(&format!("{}/.npm-global/bin/pi", home)).exists()
            }
            Agent::Glm => has_cmd("glm"),
            Agent::Shell => true,
        }
    }
}

fn has_cmd(name: &str) -> bool {
    let path = std::env::var("PATH").unwrap_or_default();
    path.split(':').any(|dir| {
        std::path::Path::new(dir).join(name).exists()
    })
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
