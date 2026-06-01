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

    // ── Claude-only smart features (2026-w20+) ────────────────────────
    // Other providers (Gemini, Codex, GLM, Pi, Hermes) ignore these
    // fields silently because their CLIs don't have equivalents. We
    // pass them only when Agent::Claude.

    /// `/goal` condition (v2.1.139+) — Claude auto-loops until this
    /// is met. Injected as the first slash command in the initial
    /// prompt. Example: "all tests in tests/auth pass and lint is clean".
    pub goal_condition: Option<String>,

    /// `--effort low|medium|high|xhigh|max` — model reasoning depth.
    /// We map (see dispatch.rs): SIMPLE→high, MEDIUM→xhigh, COMPLEX→xhigh, EPIC→max.
    pub effort: Option<String>,

    /// `--model <name>` — explicit model pin (e.g. "claude-opus-4-8"). When
    /// set, we emit `--model <name>` so the spawned session never silently
    /// drifts onto the CLI's default model. Claude-only.
    pub model: Option<String>,

    /// `--max-turns N` — hard cap on conversation turns. Bounds
    /// runaway oracles (rule R-28 cost tracking).
    pub max_turns: Option<u32>,

    /// `--max-budget-usd N` — hard cap on token spend.
    pub max_budget_usd: Option<f32>,

    /// `--name <name>` — deterministic session label for resume.
    pub session_name: Option<String>,
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
            "shell" | "bash" => Some(Agent::Shell),
            _ => None,
        }
    }

    /// Official one-line installer command for this agent, or None if it
    /// comes pre-installed / not installable via a script.
    pub fn install_command(&self) -> Option<&'static str> {
        match self {
            Agent::Glm => Some("npm install -g @z-ai/glm-cli"),
            Agent::Claude => Some(
                "T=$(mktemp) && curl -fsSL https://claude.ai/install.sh -o \"$T\" && bash \"$T\"; rm -f \"$T\"",
            ),
            Agent::Codex => Some("npm install -g @openai/codex"),
            Agent::Gemini => Some("npm install -g @google/gemini-cli"),
            Agent::Pi => Some(
                "T=$(mktemp) && curl -fsSL https://pi.dev/install.sh -o \"$T\" && sh \"$T\"; rm -f \"$T\"",
            ),
            Agent::Hermes => Some(
                "T=$(mktemp) && curl -fsSL https://hermes-agent.nousresearch.com/install.sh -o \"$T\" && bash \"$T\" && hermes setup; rm -f \"$T\"",
            ),
            Agent::Shell => None,
        }
    }

    /// Best-effort uninstall command for an agent. Documents how to
    /// remove the binary from the user's PATH. Not all agents have a
    /// turnkey uninstaller, so this is informational + best-effort.
    pub fn uninstall_command(&self) -> Option<&'static str> {
        match self {
            Agent::Claude => Some("rm -f $(which claude) && rm -rf ~/.claude"),
            Agent::Codex => Some("npm uninstall -g @openai/codex"),
            Agent::Gemini => Some("npm uninstall -g @google/gemini-cli"),
            Agent::Pi => Some("rm -f $(which pi) && rm -rf ~/.pi"),
            Agent::Hermes => Some("rm -f $(which hermes) && rm -rf ~/.hermes"),
            Agent::Glm => Some("npm uninstall -g @z-ai/glm-cli"),
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
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

        match self {
            Agent::Claude => {
                let mut args = String::from("claude --dangerously-skip-permissions");
                if let Some(ref sys_file) = opts.system_prompt_file {
                    args.push_str(&format!(" --append-system-prompt-file {}", shell_quote(sys_file)));
                }
                if opts.resume_conversation {
                    args.push_str(" --continue");
                }
                // Claude-only smart flags (2026-w20+). Silently ignored
                // by older Claude Code installs.
                if let Some(ref m) = opts.model {
                    args.push_str(&format!(" --model {}", shell_quote(m)));
                }
                if let Some(ref e) = opts.effort {
                    args.push_str(&format!(" --effort {}", shell_quote(e)));
                }
                if let Some(t) = opts.max_turns {
                    args.push_str(&format!(" --max-turns {}", t));
                }
                if let Some(b) = opts.max_budget_usd {
                    args.push_str(&format!(" --max-budget-usd {}", b));
                }
                if let Some(ref n) = opts.session_name {
                    args.push_str(&format!(" --name {}", shell_quote(n)));
                }
                // /goal is a slash command, not a flag — prepend to the
                // initial prompt so Claude registers it as the first
                // turn's instruction (per docs: works in interactive + -p).
                let final_prompt: Option<String> = match (&opts.goal_condition, initial_prompt) {
                    (Some(goal), Some(p)) => Some(format!("/goal {}\n\n{}", goal, p)),
                    (Some(goal), None) => Some(format!("/goal {}", goal)),
                    (None, Some(p)) => Some(p.to_string()),
                    (None, None) => None,
                };
                match final_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!("{} {}; exec bash", args, shell_quote(&p)))
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
                let pi_args = "--provider openrouter --model anthropic/claude-sonnet-4.6";
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!("pi {} {}; exec bash", pi_args, shell_quote(p)))
                    ),
                    None => format!("pi {}", pi_args),
                }
            }
            Agent::Hermes => match initial_prompt {
                Some(p) => format!(
                    "bash -c {}",
                    shell_quote(&format!("hermes {}; exec bash", shell_quote(p)))
                ),
                None => "bash -c \"hermes; exec bash\"".to_string(),
            },
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
