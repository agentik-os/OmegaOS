use crate::providers::ProvidersConfig;
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

    // ── Interactive-safe Claude flags (Lane A only) ────────────────────
    // These all keep the TTY attachable (rmux pane). Emitted ONLY when
    // set, in the Agent::Claude arm. Headless-only flags (stream-json /
    // --print / --input-format / --include-partial-messages) are NOT here —
    // they live on Lane B (claude_stream.rs) where there is no human attach.

    /// `--session-id <uuid>` — deterministic session id for resume/dedupe.
    /// Must be a valid UUID; we generate+persist one per oracle in ~/.omega/state.
    pub session_id: Option<String>,
    /// `--fork-session` — on resume, fork to a NEW id instead of mutating the
    /// original (use with resume_conversation/--continue). Interactive-safe.
    pub fork_session: bool,
    /// `--permission-mode <mode>` — per-role: "plan" (oracle planning),
    /// "acceptEdits" (trusted worker), "default"/"auto"/"dontAsk". Validated
    /// against the CLI's 6 choices. NOTE: today Lane A passes
    /// --dangerously-skip-permissions (agents.rs:183); a real permission-mode
    /// policy means dropping skip-perms for roles that get a mode.
    pub permission_mode: Option<String>,
    /// `--allowedTools` — comma/space list (e.g. "Bash(git *) Edit Read").
    pub allowed_tools: Option<String>,
    /// `--disallowedTools` — comma/space deny list.
    pub disallowed_tools: Option<String>,
    /// `--mcp-config <path...>` — JSON file(s) wiring OmegaOS tools as MCP
    /// servers. Binaries resolve under ~/.omega/bin; config under ~/.omega.
    pub mcp_config: Option<Vec<String>>,
    /// `--strict-mcp-config` — ONLY use servers from mcp_config, ignore
    /// user/project .mcp.json. Pair with mcp_config for hermetic worker roles.
    pub strict_mcp_config: bool,
    /// `--debug-file <path>` — debug log to ~/.omega/state/<session>.debug.log
    /// (implicitly enables debug mode). Interactive-safe (writes a file, keeps TTY).
    pub debug_file: Option<String>,
    /// `--verbose` — override verbose config. Interactive-safe on Lane A.
    pub verbose: bool,
    /// `--exclude-dynamic-system-prompt-sections` — move per-machine sections
    /// out of the system prompt into the first user message for cross-session
    /// prompt-cache reuse. Ignored when --system-prompt is set; we use
    /// --append-system-prompt-file so the default prompt still applies → SAFE.
    pub exclude_dynamic_prompt_sections: bool,
    /// `--brief` — enable the SendUserMessage agent→user tool. Interactive-safe;
    /// useful for oracles to push a structured note to the human.
    pub brief: bool,
    /// `--bare` — minimal mode (skip hooks/LSP/plugin-sync/CLAUDE.md auto-discovery,
    /// sets CLAUDE_CODE_SIMPLE=1). Interactive-safe BUT changes auth to API-key-only
    /// and disables CLAUDE.md autodiscovery → only for hermetic/perf worker roles,
    /// never the default oracle. Off by default.
    pub bare: bool,
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
            // GLM (Z.AI/Zhipu) has NO standalone CLI. The official method is to run
            // Claude Code pointed at Z.AI's Anthropic-compatible endpoint. So
            // "installing GLM" = ensuring the Claude Code binary is present; the
            // redirect happens at launch (see launch_command_with) + the key.
            // npm-package CLIs: use npm when present, else fall back to bun
            // (OmegaOS installs bun; reference it by absolute path so a
            // non-interactive `bash -c` pane finds it even when ~/.bun/bin isn't
            // on PATH). A node-less box was why these "Enter → install" actions
            // appeared to do nothing.
            Agent::Glm => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @anthropic-ai/claude-code; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @anthropic-ai/claude-code; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::Claude => Some(
                "T=$(mktemp) && curl -fsSL https://claude.ai/install.sh -o \"$T\" && bash \"$T\"; rm -f \"$T\"",
            ),
            Agent::Codex => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @openai/codex; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @openai/codex; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::Gemini => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @google/gemini-cli; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @google/gemini-cli; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            // Pi: install the npm package directly (the curl|sh installer runs a
            // TTY animation that fails in a non-interactive pane → `pi` never landed).
            Agent::Pi => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @earendil-works/pi-coding-agent; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @earendil-works/pi-coding-agent; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
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
            // Binary ONLY. NEVER `rm -rf ~/.claude` — that directory holds the
            // user's agents, slash commands, settings.json and credentials (and
            // OmegaOS itself installs into it). Uninstalling the CLI must not wipe
            // the user's whole config. Config can be removed by hand if desired.
            Agent::Claude => Some("rm -f \"$(command -v claude)\""),
            Agent::Codex => Some("npm uninstall -g --prefix \"$HOME/.npm-global\" @openai/codex"),
            Agent::Gemini => Some("npm uninstall -g --prefix \"$HOME/.npm-global\" @google/gemini-cli"),
            Agent::Pi => Some("rm -f $(which pi) && rm -rf ~/.pi"),
            Agent::Hermes => Some("rm -f $(which hermes) && rm -rf ~/.hermes"),
            // GLM shares the Claude Code binary — there is nothing GLM-specific to
            // uninstall. Removing it would wrongly delete the user's Claude Code.
            Agent::Glm => None,
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

    /// Inline `export K='V'; …` prefix for the env vars this provider needs in
    /// its pane, scoped to the LAUNCHED provider (never leak a sibling's key
    /// into an unrelated pane). Pulled from `providers.toml` via
    /// [`ProvidersConfig::env_vars`] — this is the wiring that makes the
    /// previously-dead `env_vars()` actually reach the spawned session.
    ///
    /// Returns "" when the provider needs no injected key (Claude/Gemini use
    /// OAuth; Shell needs nothing). The export is emitted INLINE on the command
    /// (not via a shell rc) because panes are launched with `bash -c`, which
    /// reads neither ~/.zshenv nor ~/.bashrc.
    fn provider_env_prefix(&self, cfg: &ProvidersConfig) -> String {
        // The full env map from providers.toml (ANTHROPIC_API_KEY, OPENAI_*,
        // GLM_API_KEY, OPENROUTER_*, …). We pick only the keys relevant to the
        // launched provider so a Codex pane never sees GLM's token, etc.
        let all = cfg.env_vars();
        let pick = |keys: &[&str]| -> String {
            let mut s = String::new();
            for (k, v) in &all {
                if keys.contains(&k.as_str()) {
                    s.push_str(&format!("export {}={}; ", k, shell_quote(v)));
                }
            }
            s
        };
        match self {
            Agent::Claude => pick(&["ANTHROPIC_API_KEY"]),
            Agent::Codex => pick(&["OPENAI_API_KEY", "OPENAI_BASE_URL"]),
            Agent::Gemini => pick(&["GOOGLE_API_KEY", "GEMINI_API_KEY"]),
            // GLM = Claude Code redirected to Z.AI; it reads ANTHROPIC_AUTH_TOKEN
            // (falling back to GLM_API_KEY). Inject GLM_API_KEY so the GLM arm's
            // `${ANTHROPIC_AUTH_TOKEN:-$GLM_API_KEY}` resolves to a real token.
            Agent::Glm => pick(&["GLM_API_KEY"]),
            // Pi and Hermes both route through OpenRouter — they need the
            // OpenRouter key/base-url. Pi additionally honors its own api_key
            // (stored as pi.api_key) as the OpenRouter key when set.
            Agent::Pi => {
                let mut s = pick(&["OPENROUTER_API_KEY", "OPENROUTER_BASE_URL"]);
                if !cfg.pi.api_key.is_empty() {
                    // pi.api_key wins as the OpenRouter credential for the Pi pane.
                    s.push_str(&format!(
                        "export OPENROUTER_API_KEY={}; ",
                        shell_quote(&cfg.pi.api_key)
                    ));
                }
                s
            }
            Agent::Hermes => {
                let mut s = pick(&["OPENROUTER_API_KEY", "OPENROUTER_BASE_URL"]);
                if !cfg.hermes.api_key.is_empty() {
                    s.push_str(&format!(
                        "export OPENROUTER_API_KEY={}; ",
                        shell_quote(&cfg.hermes.api_key)
                    ));
                }
                s
            }
            Agent::Shell => String::new(),
        }
    }

    /// Returns the shell command with options (system prompt file, continue, etc.).
    pub fn launch_command_with(&self, initial_prompt: Option<&str>, opts: LaunchOptions) -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        // Resolve provider credentials once — the env-var wiring (a) above + the
        // configured model fallbacks (b/c/d) both read from providers.toml.
        let providers = ProvidersConfig::load();
        // PATH guard: panes launch via `bash -c`, which reads NO shell rc and
        // inherits the rmux daemon's (possibly stale) PATH — so `claude`/`bun`/
        // `omega` in ~/.local/bin or ~/.bun/bin can be "command not found", and a
        // dispatched oracle drops to a bare shell instead of running its mission.
        // Prepend the user bin dirs so every launched agent + tool always resolves.
        let env_prefix = format!(
            "export PATH=\"{home}/.local/bin:{home}/.bun/bin:{home}/.npm-global/bin:$PATH\"; {}",
            self.provider_env_prefix(&providers),
            home = home
        );

        match self {
            Agent::Claude => {
                // CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1: render in the normal
                // screen (not the alternate screen) so the full conversation
                // flows into rmux's scrollback and scrolls in the panel. Set
                // INLINE on the command because omega launches the agent via
                // `bash -c`, which reads neither ~/.zshenv nor ~/.bashrc, and
                // panes inherit the (older) rmux daemon env — so a shell-rc
                // export never reaches it.
                // A real --permission-mode policy REPLACES blanket
                // --dangerously-skip-permissions: when a role declares a mode
                // (oracle→"plan", trusted worker→"acceptEdits", …) we honor it
                // instead of skipping permissions entirely. With no mode set we
                // keep the existing skip-perms behavior (unchanged default).
                // Pre-trust the pane's cwd in ~/.claude.json IMMEDIATELY before
                // claude reads it (claude_trust.rs): with many concurrent
                // sessions the shared config is last-writer-wins, so an earlier
                // acceptance is routinely clobbered and the "trust this folder?"
                // dialog re-appears — hanging dispatched oracles. Best-effort:
                // an old omega binary without the subcommand just skips it.
                let trust_prefix = "omega trust-dir \"$PWD\" >/dev/null 2>&1; ";
                let mut args = match opts.permission_mode {
                    Some(ref mode) => format!(
                        "{}{}CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 claude --permission-mode {}",
                        env_prefix,
                        trust_prefix,
                        shell_quote(mode)
                    ),
                    None => format!(
                        "{}{}CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 claude --dangerously-skip-permissions",
                        env_prefix, trust_prefix,
                    ),
                };
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
                // ── Interactive-safe flags (Lane A). All keep the TTY; emit
                //    ONLY when set. NONE of these are headless-only (no --print
                //    / --output-format / --input-format), so the rmux pane stays
                //    attachable. permission_mode is handled above (it replaces
                //    --dangerously-skip-permissions in the base command).
                if let Some(ref sid) = opts.session_id {
                    args.push_str(&format!(" --session-id {}", shell_quote(sid)));
                }
                if opts.fork_session {
                    args.push_str(" --fork-session");
                }
                if let Some(ref tools) = opts.allowed_tools {
                    args.push_str(&format!(" --allowedTools {}", shell_quote(tools)));
                }
                if let Some(ref tools) = opts.disallowed_tools {
                    args.push_str(&format!(" --disallowedTools {}", shell_quote(tools)));
                }
                if let Some(ref configs) = opts.mcp_config {
                    for cfg in configs {
                        args.push_str(&format!(" --mcp-config {}", shell_quote(cfg)));
                    }
                }
                if opts.strict_mcp_config {
                    args.push_str(" --strict-mcp-config");
                }
                if let Some(ref dbg) = opts.debug_file {
                    args.push_str(&format!(" --debug-file {}", shell_quote(dbg)));
                }
                if opts.verbose {
                    args.push_str(" --verbose");
                }
                if opts.exclude_dynamic_prompt_sections {
                    args.push_str(" --exclude-dynamic-system-prompt-sections");
                }
                if opts.brief {
                    args.push_str(" --brief");
                }
                if opts.bare {
                    args.push_str(" --bare");
                }
                // /goal is a slash command that consumes the ENTIRE first message
                // as its condition — so appending a large prompt after `/goal …`
                // blows Claude's ~4000-char /goal limit and the dispatch silently
                // aborts (the 27302-char bug: oracle dispatched but never launched).
                // Only inline /goal when the combined message stays under the cap;
                // otherwise ship the prompt alone. The oracle keeps its done.json
                // contract and, per R-GOAL, runs small goals inside dynamic
                // workflows rather than one giant /goal around the whole mission.
                const GOAL_MSG_MAX: usize = 4000;
                let final_prompt: Option<String> = match (&opts.goal_condition, initial_prompt) {
                    (Some(goal), Some(p)) => {
                        let combined = format!("/goal {}\n\n{}", goal, p);
                        if combined.chars().count() <= GOAL_MSG_MAX {
                            Some(combined)
                        } else {
                            tracing::warn!(
                                combined_len = combined.chars().count(),
                                max = GOAL_MSG_MAX,
                                "/goal + prompt exceeds /goal limit — shipping prompt without /goal (oracle uses done.json + in-workflow small goals)"
                            );
                            Some(p.to_string())
                        }
                    }
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
                    shell_quote(&format!("{}codex {}; exec bash", env_prefix, shell_quote(p)))
                ),
                None if env_prefix.is_empty() => "codex".to_string(),
                None => format!("bash -c {}", shell_quote(&format!("{}codex; exec bash", env_prefix))),
            },
            Agent::Gemini => {
                // Try alias first, fall back to npm-global, fall back to plain gemini
                let gemini_bin = format!("{}/.npm-global/bin/gemini", home);
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{}{} {}; exec bash",
                            env_prefix,
                            gemini_bin,
                            shell_quote(p)
                        ))
                    ),
                    None if env_prefix.is_empty() => gemini_bin,
                    None => format!(
                        "bash -c {}",
                        shell_quote(&format!("{}{}; exec bash", env_prefix, gemini_bin))
                    ),
                }
            }
            Agent::Pi => {
                // (b) Use the CONFIGURED pi.provider + pi.model; fall back to the
                // catalog defaults only when unset (was hardcoded
                // `--provider openrouter --model anthropic/claude-sonnet-4.6`).
                let provider = if providers.pi.provider.is_empty() {
                    "openrouter"
                } else {
                    providers.pi.provider.as_str()
                };
                let model = if providers.pi.model.is_empty() {
                    ProvidersConfig::default_model("pi").to_string()
                } else {
                    providers.pi.model.clone()
                };
                let pi_args = format!(
                    "--provider {} --model {}",
                    shell_quote(provider),
                    shell_quote(&model)
                );
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{}pi {} {}; exec bash",
                            env_prefix,
                            pi_args,
                            shell_quote(p)
                        ))
                    ),
                    None => format!(
                        "bash -c {}",
                        shell_quote(&format!("{}pi {}; exec bash", env_prefix, pi_args))
                    ),
                }
            }
            Agent::Hermes => {
                // (c) Pass --model when hermes.model is configured (was ignored).
                let hermes_args = if providers.hermes.model.is_empty() {
                    String::new()
                } else {
                    format!(" --model {}", shell_quote(&providers.hermes.model))
                };
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{}hermes{} {}; exec bash",
                            env_prefix,
                            hermes_args,
                            shell_quote(p)
                        ))
                    ),
                    None => format!(
                        "bash -c {}",
                        shell_quote(&format!("{}hermes{}; exec bash", env_prefix, hermes_args))
                    ),
                }
            }
            Agent::Glm => {
                // GLM (Z.AI/Zhipu) = Claude Code redirected to Z.AI's Anthropic-
                // compatible endpoint. The base URL is a constant; the auth token is
                // taken from $ANTHROPIC_AUTH_TOKEN (or $GLM_API_KEY). Exported INLINE
                // so only this pane is redirected — a sibling `claude` pane is
                // untouched. Note: GLM uses ANTHROPIC_AUTH_TOKEN, not ANTHROPIC_API_KEY.
                // env_prefix injects GLM_API_KEY (from providers.toml) so the
                // `${ANTHROPIC_AUTH_TOKEN:-$GLM_API_KEY}` fallback below resolves
                // to a real token (fix d). The base-url + token-redirect stays
                // inline so only this pane is redirected.
                let pre = format!(
                    "{}export ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic; export ANTHROPIC_AUTH_TOKEN=\"${{ANTHROPIC_AUTH_TOKEN:-$GLM_API_KEY}}\";",
                    env_prefix
                );
                // (d) Pass --model when glm.model is configured.
                let model_arg = if providers.glm.model.is_empty() {
                    String::new()
                } else {
                    format!(" --model {}", shell_quote(&providers.glm.model))
                };
                match initial_prompt {
                    Some(p) => format!(
                        "bash -c {}",
                        shell_quote(&format!(
                            "{} claude{} {}; exec bash",
                            pre,
                            model_arg,
                            shell_quote(p)
                        ))
                    ),
                    None => format!(
                        "bash -c {}",
                        shell_quote(&format!("{} claude{}; exec bash", pre, model_arg))
                    ),
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
            // PATH alone is unreliable under a reduced PATH (systemd/cron/non-login
            // `bash -c` panes that omit ~/.local/bin) — fall back to canonical
            // install locations so doctor doesn't falsely report "claude not on PATH".
            Agent::Claude => claude_available(&home),
            Agent::Codex => {
                has_cmd("codex")
                    || std::path::Path::new(&format!("{}/.npm-global/bin/codex", home)).exists()
            }
            Agent::Gemini => {
                has_cmd("gemini")
                    || std::path::Path::new(&format!("{}/.npm-global/bin/gemini", home)).exists()
            }
            Agent::Pi => {
                has_cmd("pi")
                    || std::path::Path::new(&format!("{}/.local/bin/pi", home)).exists()
                    || std::path::Path::new(&format!("{}/.npm-global/bin/pi", home)).exists()
            }
            Agent::Hermes => has_cmd("hermes"),
            // GLM runs on the Claude Code binary (redirected at launch) — available
            // iff `claude` is present. Shares Claude's reduced-PATH fallback so it
            // resolves the canonical install locations too, not just $PATH.
            Agent::Glm => claude_available(&home),
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

/// True iff the Claude Code binary is reachable — on $PATH, or at a canonical
/// install location the official installer uses. The location fallback keeps
/// detection correct when the caller's PATH is reduced (systemd, cron, non-login
/// `bash -c` panes) and omits ~/.local/bin.
fn claude_available(home: &str) -> bool {
    has_cmd("claude")
        || std::path::Path::new(&format!("{}/.local/bin/claude", home)).exists()
        || std::path::Path::new(&format!("{}/.claude/local/claude", home)).exists()
        || std::path::Path::new(&format!("{}/.npm-global/bin/claude", home)).exists()
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
