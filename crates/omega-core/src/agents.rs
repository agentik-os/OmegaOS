use serde::{Deserialize, Serialize};

/// Options for launching an agent — used by the Master AISB and other
/// callers that need to inject a hidden system prompt or continue an
/// existing conversation.
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    /// Path to a markdown file injected via `--append-system-prompt-file`.
    /// The contents are hidden from the chat (not shown as a user message).
    pub system_prompt_file: Option<String>,
    /// Resume the most recent conversation in the agent's CWD (Claude `--continue`).
    pub resume_conversation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    Claude,
    Codex,
    Gemini,
    Pi,
    Hermes,
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
            Agent::Hermes,
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
            Agent::Hermes => "hermes",
            Agent::Glm => "glm",
            Agent::Shell => "shell",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Agent::Claude => "Claude Code (Anthropic)",
            Agent::Codex => "Codex (OpenAI)",
            Agent::Gemini => "Gemini (Google)",
            Agent::Pi => "Pi (earendil-works)",
            Agent::Hermes => "Hermes (Nous Research)",
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
            "hermes" => Some(Agent::Hermes),
            "glm" => Some(Agent::Glm),
            "shell" | "bash" | "" => Some(Agent::Shell),
            _ => None,
        }
    }

    /// Official one-line installer command for this agent, or None if it
    /// comes pre-installed / not installable via a script.
    pub fn install_command(&self) -> Option<&'static str> {
        match self {
            Agent::Pi => Some("curl -fsSL https://pi.dev/install.sh | sh"),
            Agent::Hermes => Some(
                "curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash && hermes setup",
            ),
            Agent::Glm => Some("npm install -g @z-ai/glm-cli"),
            // Claude/Codex/Gemini: users install via their own channels
            // (claude.ai/code, npm i -g @openai/codex, npm i -g @google/gemini-cli)
            Agent::Claude => Some("curl -fsSL https://claude.ai/install.sh | bash"),
            Agent::Codex => Some("npm install -g @openai/codex"),
            Agent::Gemini => Some("npm install -g @google/gemini-cli"),
            Agent::Shell => None,
        }
    }

    /// URL of the project homepage (shown in Settings).
    pub fn homepage(&self) -> Option<&'static str> {
        match self {
            Agent::Claude => Some("https://claude.ai/code"),
            Agent::Codex => Some("https://github.com/openai/codex"),
            Agent::Gemini => Some("https://github.com/google-gemini/gemini-cli"),
            Agent::Pi => Some("https://pi.dev/"),
            Agent::Hermes => Some("https://hermes-agent.nousresearch.com/"),
            Agent::Glm => Some("https://www.z.ai/"),
            Agent::Shell => None,
        }
    }

    /// Returns the shell command to launch this agent.
    /// `initial_prompt` is the first message sent to the agent (if it supports it).
    pub fn launch_command(&self, initial_prompt: Option<&str>) -> String {
        self.launch_command_with(initial_prompt, LaunchOptions::default())
    }

    /// Returns the shell command with options (system prompt file, continue, etc.).
    pub fn launch_command_with(&self, initial_prompt: Option<&str>, opts: LaunchOptions) -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/hacker".to_string());

        match self {
            Agent::Claude => {
                let mut args = String::from("claude --dangerously-skip-permissions");
                if let Some(ref sys_file) = opts.system_prompt_file {
                    args.push_str(&format!(" --append-system-prompt-file {}", shell_quote(sys_file)));
                }
                if opts.resume_conversation {
                    args.push_str(" --continue");
                }
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!("{} {}; exec bash", args, shell_quote(p)))
                    ),
                    None => format!("bash -c {}", shell_quote(&format!("{}; exec bash", args))),
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
                // Try $PATH first (pi.dev installs to ~/.local/bin or similar),
                // fall back to ~/.npm-global/bin/pi for the older npm install.
                let pi_bin = "pi";
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
            Agent::Hermes => {
                // Hermes is invoked from $PATH after `curl … | bash && hermes setup`.
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!("hermes {}; exec bash", shell_quote(p)))
                    ),
                    None => "bash -c \"hermes; exec bash\"".to_string(),
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
                has_cmd("pi")
                    || std::path::Path::new(&format!("{}/.npm-global/bin/pi", home)).exists()
                    || std::path::Path::new(&format!("{}/.local/bin/pi", home)).exists()
            }
            Agent::Hermes => has_cmd("hermes"),
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
