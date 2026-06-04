use omega_core::config::OmegaConfig;
use omega_core::progress::ProgressInfo;
use omega_core::session::{OmegaSession, SessionManager, SessionRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Sessions,
    Menu,
    Monitor,
    Projects,
    Settings,
    Agentic,
    Help,
}

/// Claude OAuth re-login progress, surfaced in the Monitor → Account view.
/// Driven asynchronously: the engine call runs off the event loop and writes
/// the result back into `App::reauth_status` via a shared sink (see main.rs).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReauthStatus {
    /// No re-login in progress.
    #[default]
    Idle,
    /// `request_reauth` is running (spawn session + /login + capture URL, ~16s).
    Generating,
    /// URL captured — show it and prompt the user to enter the returned code.
    ShowUrl(String),
    /// `handle_code` is running (paste code + watch credentials).
    Validating,
    /// Login finished successfully (human-readable summary, e.g. email + expiry).
    Done(String),
    /// Login failed (human-readable error).
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoSection {
    Master,
    AisbAgents,
    Oracle,
    Workers,
    Rules,
}

impl InfoSection {
    pub fn all() -> &'static [InfoSection] {
        &[
            InfoSection::Master,
            InfoSection::AisbAgents,
            InfoSection::Oracle,
            InfoSection::Workers,
            InfoSection::Rules,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            InfoSection::Master => "AISB Master — the brain hub",
            InfoSection::AisbAgents => "AISB Agents (13)",
            InfoSection::Oracle => "Oracle — routing & coordination",
            InfoSection::Workers => "Workers — dispatch & lifecycle",
            InfoSection::Rules => "Rules — system invariants",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    NewNamedSession(String),
    NewSessionPromptDirect(String, String),
    /// Dispatch oracle — step 1: project PICKER (no typing). Holds the project
    /// names + the selected index, mirroring `SelectModel`. The list comes from
    /// the shared `ProjectRegistry` — the SAME source the Telegram dispatch
    /// picker uses — so the added-projects list stays in sync across surfaces.
    DispatchProject(Vec<String>, usize),
    DispatchMission(String),
    /// Renaming an existing session — holds the original name.
    RenameSession(String),
    /// Live session-list filter — the in-progress query is in `input_buffer`,
    /// the applied query in `session_filter`.
    SessionFilter,

    /// New-project wizard — step 1: project name (text input).
    NewProjectName,
    /// New-project wizard — step 2: category picker. (name, sel) where `sel`
    /// indexes `NEW_PROJECT_CATEGORIES`. Selection lives in the variant so the
    /// wizard needs no extra App state.
    NewProjectCategory(String, usize),
    /// New-project wizard — step 2b (client only): pick/create a credential
    /// group (separate client accounts). (name, category)
    NewProjectCredGroup(String, String),
    /// New-project wizard — step 3: stack picker. (name, category, sel) where
    /// `sel` indexes `NEW_PROJECT_STACKS`.
    NewProjectStack(String, String, usize),
    /// New-project wizard — step 4 (optional): kickoff prompt. (name, category, stack)
    NewProjectLaunchPrompt(String, String, String),
    /// New-project wizard — step 5 (optional): doc paths. (name, category, stack, kickoff)
    NewProjectLaunchDocs(String, String, String, Option<String>),

    /// Provisioning-keys wizard (Monitor tab, Telegram-style). `step` indexes
    /// `PROVISIONING_FIELDS`; `collected` holds the values entered so far (one
    /// per completed step, in order). The current field's value is in
    /// `input_buffer`.
    ProvisioningSetup {
        step: usize,
        collected: Vec<String>,
    },

    /// Telegram setup wizard — step 1: bot token
    TelegramSetupToken,
    /// Step 2: chat id (carries the bot token)
    TelegramSetupChatId(String),
    /// Step 3: optional user id allow-list (carries token + chat_id)
    TelegramSetupUserId(String, String),

    /// Editing a settings text field — holds (config_key, masked).
    EditSettingsField {
        config_key: String,
        masked: bool,
    },
    /// Picking a value from a fixed list via an arrow-key overlay (NO typing).
    /// Holds (config_key, options, selected_index). Mirrors the new-project
    /// category/stack pickers — Up/Down move, Enter commits, Esc cancels.
    SelectModel(String, Vec<String>, usize),

    /// Monitor → Project group: single-field capture of the Telegram supergroup
    /// id (manual fallback to the bot's auto-detect). Persists via
    /// `TelegramGroupConfig`. The in-progress value lives in `input_buffer`.
    GroupSetupId,

    /// Register-existing-folder wizard — step 1: project path (text input).
    /// Distinct from `NewProjectName` (which CREATES a project on disk); this
    /// only registers an already-existing folder into the project registry.
    AddProjectPath,

    /// Claude OAuth re-login — paste the authorize code returned by the browser.
    /// Entered after the URL is shown (Monitor → Account); on submit the main
    /// loop runs `oauth::handle_code` and renders the result.
    ReauthCode,
}

/// Added-project names from the shared `ProjectRegistry` — the SAME source the
/// Telegram dispatch picker reads, so the dispatch project list is synced across
/// the TUI menu, the Telegram bot, and the project menu. Empty when no project
/// has been added/registered yet.
pub fn dispatch_project_names() -> Vec<String> {
    omega_core::project_manager::ProjectRegistry::load()
        .projects
        .iter()
        .map(|p| p.name.clone())
        .collect()
}

/// New-project wizard option lists. `(id, label)` — `id` is the token passed to
/// the `/omega-new-project` skill; `label` is what the picker shows. Single
/// source of truth for both the menu UI and the spawned command.
pub const NEW_PROJECT_CATEGORIES: &[(&str, &str)] = &[
    ("customer", "Customer — client work  (customers/ under your projects dir)"),
    ("side-business", "Side business — your own products  (side-business/ under your projects dir)"),
    ("tools", "Tools — internal tooling / libraries  (tools/ under your projects dir)"),
];
/// Stacks by project type. `id` is passed to /omega-new-project (which branches
/// per id); `label` carries the type hint. Aligned with R-STACK doctrine.
pub const NEW_PROJECT_STACKS: &[(&str, &str)] = &[
    ("nextstack", "SaaS — Next.js 16 + Convex + Clerk + Stripe + shadcn"),
    ("nextstack-content", "Content / multi-user — Next.js 16 + Convex"),
    ("nextstack-static", "Marketing / landing — Next.js 16 static (no backend)"),
    ("rust-cli", "CLI / daemon / internal tool — Rust"),
    ("bun-script", "Script / tooling / DOM — Bun + TypeScript"),
    ("expo-mobile", "Mobile iOS/Android — Expo + React Native"),
];

/// Provisioning-keys wizard fields (Monitor tab). `(env_key, prompt, masked)` in
/// step order. `masked` hides the secret in the input echo. Written to
/// `~/.omega/provisioning/services.env` by `omega_core::provisioning`. Each step
/// is skippable (Esc = leave blank → keeps any existing value).
pub const PROVISIONING_FIELDS: &[(&str, &str, bool)] = &[
    ("VERCEL_TOKEN", "Vercel token — vercel.com/account/tokens (Full Account). Esc to skip.", true),
    ("CONVEX_TEAM_TOKEN", "Convex team token — dashboard → Team Settings → access token. Esc to skip.", true),
    ("CONVEX_TEAM_SLUG", "Convex team slug — your team's URL slug (not secret). Esc to skip.", false),
    ("GITHUB_TOKEN", "GitHub token (repo+workflow) — or Esc to skip & use `gh auth`.", true),
    ("STRIPE_SECRET_KEY", "Stripe secret key — dashboard → Developers → API keys. Esc to skip.", true),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewClaude,
    NewCodex,
    NewGemini,
    NewPi,
    NewHermes,
    NewGlm,
    NewTerminal,
    NewProject,
    DispatchOracle,
    Refresh,
    ToggleProtection,
    KillSelected,
    KillAll,
    NuclearCleanup,
    Restart,
    Quit,
}

impl MenuAction {
    pub fn all() -> &'static [MenuAction] {
        &[
            MenuAction::NewClaude,
            MenuAction::NewCodex,
            MenuAction::NewGemini,
            MenuAction::NewPi,
            MenuAction::NewHermes,
            MenuAction::NewGlm,
            MenuAction::NewTerminal,
            MenuAction::NewProject,
            MenuAction::DispatchOracle,
            MenuAction::Refresh,
            MenuAction::ToggleProtection,
            MenuAction::KillSelected,
            MenuAction::KillAll,
            MenuAction::NuclearCleanup,
            MenuAction::Restart,
            MenuAction::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "New Claude session",
            MenuAction::NewCodex => "New Codex session",
            MenuAction::NewGemini => "New Gemini session",
            MenuAction::NewPi => "New Pi session (earendil-works)",
            MenuAction::NewHermes => "New Hermes session (Nous Research)",
            MenuAction::NewGlm => "New GLM session",
            MenuAction::NewTerminal => "New Terminal (plain shell)",
            MenuAction::NewProject => {
                "New project  →  pick stack + auto-provision (Convex/Vercel/Clerk/Stripe)"
            }
            MenuAction::DispatchOracle => "Dispatch oracle  →  project + mission",
            MenuAction::Refresh => "Refresh sessions list",
            MenuAction::ToggleProtection => "Toggle protection on selected",
            MenuAction::KillSelected => "Kill selected session",
            MenuAction::KillAll => "Kill ALL sessions (keeps current + protected + infra)",
            MenuAction::NuclearCleanup => {
                "Nuclear cleanup — kill all + prune state + clear scratch + free RAM"
            }
            MenuAction::Restart => "Restart OmegaOS (reload binary)",
            MenuAction::Quit => "Quit OmegaOS",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "c",
            MenuAction::NewCodex => "C",
            MenuAction::NewGemini => "g",
            MenuAction::NewPi => "p",
            MenuAction::NewHermes => "h",
            MenuAction::NewGlm => "G",
            MenuAction::NewTerminal => "t",
            MenuAction::NewProject => "N",
            MenuAction::DispatchOracle => "d",
            MenuAction::Refresh => "F5",
            MenuAction::ToggleProtection => ".",
            MenuAction::KillSelected => "x",
            MenuAction::KillAll => "—",
            MenuAction::NuclearCleanup => "—",
            MenuAction::Restart => "R",
            MenuAction::Quit => "q",
        }
    }

    pub fn agent(&self) -> Option<omega_core::agents::Agent> {
        match self {
            MenuAction::NewClaude => Some(omega_core::agents::Agent::Claude),
            MenuAction::NewCodex => Some(omega_core::agents::Agent::Codex),
            MenuAction::NewGemini => Some(omega_core::agents::Agent::Gemini),
            MenuAction::NewPi => Some(omega_core::agents::Agent::Pi),
            MenuAction::NewHermes => Some(omega_core::agents::Agent::Hermes),
            MenuAction::NewGlm => Some(omega_core::agents::Agent::Glm),
            MenuAction::NewTerminal => Some(omega_core::agents::Agent::Shell),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SessionRow {
    /// A visible section header (e.g. "── Causio ──")
    Header(String),
    Entry(SessionEntry),
}

impl SessionRow {
    pub fn is_selectable(&self) -> bool {
        matches!(self, SessionRow::Entry(_))
    }
    pub fn as_entry(&self) -> Option<&SessionEntry> {
        match self {
            SessionRow::Entry(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub session: OmegaSession,
    pub progress: Option<ProgressInfo>,
    pub is_current: bool,
    pub is_protected: bool,
    pub tree_prefix: String,
}

impl SessionEntry {
    pub fn clone_for_row(&self) -> Self {
        self.clone()
    }
}

fn section_for(session: &OmegaSession) -> String {
    use omega_core::session::SessionRole;
    match (&session.role, &session.project) {
        (SessionRole::Oracle | SessionRole::Worker, Some(p)) => p.clone(),
        (SessionRole::Home, _) => "Home".to_string(),
        (SessionRole::System, _) => "System".to_string(),
        _ => "Other".to_string(),
    }
}

/// Which side of the Sessions tab has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFocus {
    /// Default: left list focused, right shows preview only
    List,
    /// Tab pressed once: split layout (list + chat input)
    Chat,
    /// Tab pressed twice quickly: chat takes the full width
    ChatFullscreen,
}

/// Interactive items within a Settings sub-section.
#[derive(Debug, Clone)]
pub enum SettingsField {
    /// Run a shell command (typically install/uninstall).
    /// `confirm_first`: if true, the field shows the command and Enter runs it
    /// without further prompting; if false, fires immediately.
    Action {
        label: String,
        command: String,
        confirm_first: bool,
    },
    /// Open a text input modal that updates a config key on confirm.
    EditText {
        label: String,
        config_key: String,
        current_value: String,
        masked: bool,
    },
    /// Toggle a boolean config key.
    Toggle {
        label: String,
        config_key: String,
        current: bool,
    },
    /// Pick one value from a fixed list via an arrow-key overlay (NO typing).
    /// `current_index` points at the currently-saved value in `options`.
    Select {
        label: String,
        config_key: String,
        options: Vec<String>,
        current_index: usize,
    },
    /// Show informational text only (homepage link, status, etc.).
    Info(String),
}

impl SettingsField {
    pub fn is_actionable(&self) -> bool {
        !matches!(self, SettingsField::Info(_))
    }
    pub fn label(&self) -> &str {
        match self {
            SettingsField::Action { label, .. } => label,
            SettingsField::EditText { label, .. } => label,
            SettingsField::Toggle { label, .. } => label,
            SettingsField::Select { label, .. } => label,
            SettingsField::Info(s) => s,
        }
    }
}

/// Build a model field for a provider. When the provider has a known model
/// list (`providers::models_for`), this is an arrow-key Select (NO typing);
/// otherwise it falls back to a free-text field so providers without a curated
/// list (e.g. pi/hermes) still work.
fn model_field(provider: &str, config_key: &str, current: &str) -> SettingsField {
    let opts: Vec<String> = omega_core::providers::ProvidersConfig::models_for(provider)
        .iter()
        .map(|s| s.to_string())
        .collect();
    if opts.is_empty() {
        SettingsField::EditText {
            label: "Model".to_string(),
            config_key: config_key.to_string(),
            current_value: current.to_string(),
            masked: false,
        }
    } else {
        // Point at the saved value; if it isn't in the list, prepend it so the
        // user never loses a custom value and index 0 stays valid.
        let mut options = opts;
        let current_index = match options.iter().position(|m| m == current) {
            Some(i) => i,
            None if current.is_empty() => 0,
            None => {
                options.insert(0, current.to_string());
                0
            }
        };
        SettingsField::Select {
            label: "Model".to_string(),
            config_key: config_key.to_string(),
            options,
            current_index,
        }
    }
}

/// Build the field list for a settings section.
pub fn fields_for_section(
    section: SettingsSection,
    providers: &omega_core::providers::ProvidersConfig,
    config: &OmegaConfig,
) -> Vec<SettingsField> {
    use omega_core::agents::Agent;
    let mut out = Vec::new();
    let agent_for_section = |s: SettingsSection| -> Option<Agent> {
        match s {
            SettingsSection::Claude => Some(Agent::Claude),
            SettingsSection::Codex => Some(Agent::Codex),
            SettingsSection::Gemini => Some(Agent::Gemini),
            SettingsSection::Glm => Some(Agent::Glm),
            _ => None,
        }
    };

    match section {
        SettingsSection::General => {
            out.push(SettingsField::Info(format!(
                "Default AISB agent: {}",
                config.aisb_agent
            )));
            out.push(SettingsField::Info(format!("Default model: {}", config.default_model)));
            out.push(SettingsField::Toggle {
                label: "Auto-spawn Master on launch".to_string(),
                config_key: "general.auto_spawn_master".to_string(),
                current: config.auto_spawn_master,
            });
            out.push(SettingsField::Toggle {
                label: "Auto-naming sessions".to_string(),
                config_key: "general.auto_naming".to_string(),
                current: config.auto_naming,
            });
        }
        SettingsSection::Install => {
            // Per-agent install / uninstall buttons
            for agent in Agent::all() {
                if matches!(agent, Agent::Shell) {
                    continue;
                }
                let installed = agent.is_available();
                let badge = if installed { "[+]" } else { "[x]" };
                let agent_name = agent.name();
                out.push(SettingsField::Info(format!(
                    "{}  {:8}  {}",
                    badge,
                    agent_name,
                    agent.display_name()
                )));
                if let Some(cmd) = agent.install_command() {
                    out.push(SettingsField::Action {
                        label: format!(
                            "    [{}] {}",
                            if installed { "Re-install" } else { "Install" },
                            agent_name
                        ),
                        command: cmd.to_string(),
                        confirm_first: true,
                    });
                }
                if installed {
                    if let Some(cmd) = agent.uninstall_command() {
                        out.push(SettingsField::Action {
                            label: format!("    [Uninstall] {}", agent_name),
                            command: cmd.to_string(),
                            confirm_first: true,
                        });
                    }
                }
                out.push(SettingsField::Info(String::new())); // spacer
            }
        }
        SettingsSection::Claude => {
            let c = &providers.claude;
            out.push(model_field("claude", "claude.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Effort (low/medium/high/max)".to_string(),
                config_key: "claude.effort".to_string(),
                current_value: c.effort.clone(),
                masked: false,
            });
            out.push(SettingsField::EditText {
                label: "Anthropic API key".to_string(),
                config_key: "claude.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::Toggle {
                label: "Dangerously skip permissions".to_string(),
                config_key: "claude.dangerously_skip_permissions".to_string(),
                current: c.dangerously_skip_permissions,
            });
            out.extend(install_actions_for(Agent::Claude));
        }
        SettingsSection::Codex => {
            let c = &providers.codex;
            out.push(model_field("codex", "codex.model", &c.model));
            out.push(SettingsField::EditText {
                label: "OpenAI API key".to_string(),
                config_key: "codex.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::EditText {
                label: "Base URL".to_string(),
                config_key: "codex.base_url".to_string(),
                current_value: c.base_url.clone(),
                masked: false,
            });
            out.extend(install_actions_for(Agent::Codex));
        }
        SettingsSection::Gemini => {
            let c = &providers.gemini;
            out.push(model_field("gemini", "gemini.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Google API key".to_string(),
                config_key: "gemini.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Gemini));
        }
        SettingsSection::Glm => {
            let c = &providers.glm;
            out.push(model_field("glm", "glm.model", &c.model));
            out.push(SettingsField::EditText {
                label: "GLM API key".to_string(),
                config_key: "glm.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Glm));
        }
        SettingsSection::Pi => {
            let c = &providers.pi;
            out.push(SettingsField::EditText {
                label: "Provider".to_string(),
                config_key: "pi.provider".to_string(),
                current_value: c.provider.clone(),
                masked: false,
            });
            out.push(model_field("pi", "pi.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Pi API key (OpenRouter)".to_string(),
                config_key: "pi.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Pi));
        }
        SettingsSection::Hermes => {
            let c = &providers.hermes;
            out.push(model_field("hermes", "hermes.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Hermes API key".to_string(),
                config_key: "hermes.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Hermes));
        }
        SettingsSection::Aisb => {
            out.push(SettingsField::Info(format!(
                "Master session name: {}",
                omega_core::aisb::MASTER_SESSION_NAME
            )));
            out.push(SettingsField::Info(format!(
                "Current AISB agent: {}",
                config.aisb_agent
            )));
            out.push(SettingsField::Action {
                label: "[Re-spawn Master AISB now]".to_string(),
                command: "omega master".to_string(),
                confirm_first: true,
            });
            out.push(SettingsField::Action {
                label: "[Kill Master AISB]".to_string(),
                command: format!("omega kill {}", omega_core::aisb::MASTER_SESSION_NAME),
                confirm_first: true,
            });
        }
        SettingsSection::Telegram => {
            match omega_core::monitor::OmegaTelegramConfig::read() {
                Some(cfg) => {
                    out.push(SettingsField::Info(format!("Enabled: {}", cfg.enabled)));
                    out.push(SettingsField::Info(format!("Chat ID: {}", cfg.chat_id)));
                    out.push(SettingsField::Info(format!("Relay: {}", cfg.relay_session)));
                    out.push(SettingsField::Action {
                        label: "[Disconnect Telegram bot]".to_string(),
                        command: "omega telegram disconnect".to_string(),
                        confirm_first: true,
                    });
                    out.push(SettingsField::Action {
                        label: "[Run Telegram bot (foreground)]".to_string(),
                        command: "omega telegram run".to_string(),
                        confirm_first: true,
                    });
                }
                None => {
                    out.push(SettingsField::Info(
                        "Not configured. Use the Setup action below or the Monitor tab [T].".to_string(),
                    ));
                    out.push(SettingsField::Action {
                        label: "[Set up Telegram bot] (opens wizard)".to_string(),
                        command: "__INTERNAL_TELEGRAM_SETUP__".to_string(),
                        confirm_first: false,
                    });
                }
            }
        }
    }

    // Common: every provider section gets an "Open homepage" info line
    if let Some(agent) = agent_for_section(section) {
        if let Some(home) = agent.homepage() {
            out.push(SettingsField::Info(format!("Homepage: {}", home)));
        }
    }

    out
}

fn install_actions_for(agent: omega_core::agents::Agent) -> Vec<SettingsField> {
    let mut out = Vec::new();
    let installed = agent.is_available();
    let badge = if installed { "[+] installed" } else { "[x] not installed" };
    out.push(SettingsField::Info(String::new()));
    out.push(SettingsField::Info(format!("Status: {}", badge)));
    if let Some(cmd) = agent.install_command() {
        out.push(SettingsField::Action {
            label: format!(
                "[{}] {}",
                if installed { "Re-install" } else { "Install" },
                agent.display_name()
            ),
            command: cmd.to_string(),
            // Installing is NON-destructive → launch on the FIRST Enter (a
            // double-Enter "confirm" made it feel like Enter did nothing). Only
            // uninstall (below) keeps the confirm gate.
            confirm_first: false,
        });
    }
    if installed {
        if let Some(cmd) = agent.uninstall_command() {
            out.push(SettingsField::Action {
                label: format!("[Uninstall] {}", agent.display_name()),
                command: cmd.to_string(),
                confirm_first: true,
            });
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Install,
    Claude,
    Codex,
    Gemini,
    Pi,
    Hermes,
    Glm,
    Aisb,
    Telegram,
}

impl SettingsSection {
    pub fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Install,
            SettingsSection::Claude,
            SettingsSection::Codex,
            SettingsSection::Gemini,
            SettingsSection::Pi,
            SettingsSection::Hermes,
            SettingsSection::Glm,
            SettingsSection::Aisb,
            SettingsSection::Telegram,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Install => "Install agents",
            SettingsSection::Claude => "Claude (Anthropic)",
            SettingsSection::Codex => "Codex (OpenAI)",
            SettingsSection::Gemini => "Gemini (Google)",
            SettingsSection::Pi => "Pi (earendil-works)",
            SettingsSection::Hermes => "Hermes (Nous Research)",
            SettingsSection::Glm => "GLM (Z.AI)",
            SettingsSection::Aisb => "AISB Master",
            SettingsSection::Telegram => "Telegram",
        }
    }
}

/// Left-hand sections of the Monitor tab. Mirrors `SettingsSection` /
/// `InfoSection`: a section list on the left, the selected section's detail on
/// the right. The `Actions` section hosts the interactive `MonitorAction` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorSection {
    Account,
    Billing,
    Telegram,
    Accounts,
    Projects,
    Actions,
}

impl MonitorSection {
    pub fn all() -> &'static [MonitorSection] {
        &[
            MonitorSection::Account,
            MonitorSection::Billing,
            MonitorSection::Telegram,
            MonitorSection::Accounts,
            MonitorSection::Projects,
            MonitorSection::Actions,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorSection::Account => "Connected account",
            MonitorSection::Billing => "Billing (live)",
            MonitorSection::Telegram => "Telegram bot & group",
            MonitorSection::Accounts => "Claude accounts",
            MonitorSection::Projects => "Project group",
            MonitorSection::Actions => "Actions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAction {
    Login,
    TelegramSetup,
    TelegramDisconnect,
    ProvisioningSetup,
    RefreshBilling,
    OpenDashboard,
}

impl MonitorAction {
    pub fn all() -> &'static [MonitorAction] {
        &[
            MonitorAction::Login,
            MonitorAction::TelegramSetup,
            MonitorAction::TelegramDisconnect,
            MonitorAction::ProvisioningSetup,
            MonitorAction::RefreshBilling,
            MonitorAction::OpenDashboard,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorAction::Login => "Login / re-auth Claude   (opens the guided OAuth session)",
            MonitorAction::TelegramSetup => "Set up Omega Telegram bot   (guided 3-step wizard)",
            MonitorAction::TelegramDisconnect => "Disconnect Telegram bot",
            MonitorAction::ProvisioningSetup => "Set up project provisioning keys   (Vercel/Convex/GitHub/Stripe)",
            MonitorAction::RefreshBilling => "Refresh billing now   (live OAuth usage check)",
            MonitorAction::OpenDashboard => "Open Dashboard   (OmegaMC Telegram dashboard — replaces aisb-master)",
        }
    }
    pub fn shortcut(&self) -> &'static str {
        match self {
            MonitorAction::Login => "L",
            MonitorAction::TelegramSetup => "T",
            MonitorAction::TelegramDisconnect => "D",
            MonitorAction::ProvisioningSetup => "P",
            MonitorAction::RefreshBilling => "B",
            MonitorAction::OpenDashboard => "O",
        }
    }
    /// Resolve the "Open Dashboard" action against the real filesystem.
    ///
    /// OmegaMC (the Telegram-controlled web dashboard, `agentik-os/agentik-telegram`)
    /// is installed by `install.sh` Phase 6.95 into `$OMEGA_DIR/omega-mc` — a
    /// best-effort clone that may be absent (private repo / `OMEGA_SKIP_DASHBOARD=1`).
    ///
    /// Present → return the real `docker compose up -d` launch in that dir (the
    /// caller turns this into `Action::RunShellCommand`, the same mechanism the
    /// Settings install/uninstall actions use). Absent → return the honest
    /// install instructions so the operator can clone it.
    pub fn resolve_open_dashboard() -> DashboardLaunch {
        let dir = omega_core::config::omega_dir().join("repos").join("omega-mc");
        let dir_str = dir.to_string_lossy().to_string();
        // The directory alone isn't proof of a usable clone; require the .git
        // marker install.sh checks (a failed clone is `rm -rf`'d, but a partial
        // manual copy could leave a bare dir). Runtime truth over assumption.
        if dir.join(".git").is_dir() {
            DashboardLaunch::Launch {
                command: format!(
                    "cd {dir} && echo '── Starting OmegaMC dashboard (docker compose up -d) ──' && docker compose up -d && echo && echo 'Dashboard up. Local URL: http://localhost:8080 (see {dir}/docker-compose.yml for the published port; AISB agents in config/omega-aisb.yaml).'",
                    dir = shell_quote(&dir_str),
                ),
                message: format!(
                    "▶ Starting OmegaMC dashboard via `docker compose up -d` in {dir_str} — watch the spawned session; URL printed there once containers are up."
                ),
            }
        } else {
            DashboardLaunch::NotInstalled {
                message: format!(
                    "OmegaMC dashboard not installed. Install it with: git clone https://github.com/agentik-os/agentik-telegram.git {dir_str} && cd {dir_str} && docker compose up -d"
                ),
            }
        }
    }
}

/// Result of resolving the Monitor "Open Dashboard" action against the
/// filesystem. Mapped to an `Action` by the input layer (kept Action-free here
/// so `app.rs` stays decoupled from `input.rs`).
#[derive(Debug, Clone)]
pub enum DashboardLaunch {
    /// OmegaMC is installed — launch it in a session via the given shell command.
    Launch { command: String, message: String },
    /// OmegaMC is absent — show honest install instructions, no command run.
    NotInstalled { message: String },
}

/// Single-quote a string for safe interpolation into a `bash -c` command. Wraps
/// in single quotes and escapes embedded single quotes the POSIX way (`'\''`).
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub struct App {
    pub tab: Tab,
    pub sessions: Vec<SessionEntry>,
    /// Renderable rows including section headers (parallel to sessions, but
    /// includes Header variants between groups). Built by refresh().
    pub rows: Vec<SessionRow>,
    pub selected: usize,
    pub menu_selected: usize,
    /// Terminal mouse capture state. true → clickable menus + scroll; false →
    /// native terminal text selection / copy-paste. Toggled with Ctrl-T.
    pub mouse_capture: bool,
    /// Last rendered Menu-tab list area + a rendered-row→action-index map, so a
    /// mouse click can hit-test which action was clicked. `menu_fits` is false
    /// when the list is taller than the area (scrolled) — clicks are then ignored.
    pub menu_area: ratatui::layout::Rect,
    pub menu_rendered_actions: Vec<Option<usize>>,
    pub menu_fits: bool,
    /// Selected Monitor section (left list). Indexes `MonitorSection::all()`.
    pub monitor_selected: usize,
    /// Cursor within the Monitor `Actions` section's `MonitorAction` list.
    pub monitor_action_selected: usize,
    pub settings_selected: usize,
    /// Cursor within the focused Settings section's interactive field list.
    pub settings_field_selected: usize,
    /// Field index awaiting a second Enter to confirm a destructive Action
    /// (the `confirm_first` flag). Cleared on navigation or section change.
    pub settings_confirm_pending: Option<usize>,
    /// Two-press confirm for destructive menu items (KillAll / NuclearCleanup):
    /// first Enter arms it, second Enter on the same item fires.
    pub menu_confirm_pending: Option<MenuAction>,
    /// Session-list status badges (done/blocked) by name, refreshed each tick
    /// from done.json + worker-blocked signals.
    pub session_badges: std::collections::HashMap<String, omega_core::done::DoneStatus>,
    /// Active session-list filter (case-insensitive substring on name); None =
    /// show all. Applied at the data source in refresh() so navigation is
    /// unaffected — the list is simply narrower.
    pub session_filter: Option<String>,
    /// New-project wizard: chosen credential group for a client project (set in
    /// the cred-group step, read by the CreateProject handler). None = default.
    pub new_project_cred_group: Option<String>,
    pub info_section_selected: usize,
    /// When the AISB Agents sub-section is active, which of the 13 is highlighted.
    pub info_agent_selected: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub config: OmegaConfig,
    pub preview_content: String,
    /// Actual rendered inner size of the preview panel (text area, minus
    /// borders), written by the renderer each frame. main.rs reads this to
    /// resize the rmux pane to the EXACT panel width — computing from a
    /// terminal-width percentage was off by a few cols and clipped the
    /// agent's content on the right.
    pub preview_inner_width: u16,
    pub preview_inner_height: u16,
    /// Per-session Claude model + cumulative token count, shown on the right
    /// of the preview title. Keyed by session name; refreshed off the hot
    /// path (main.rs, throttled). `(short_model, tokens)`.
    pub session_meta: std::collections::HashMap<String, (String, u64)>,
    /// Per-session compact git status (e.g. "↑4h • main" or just "main")
    /// shown in the status bar on the Sessions tab instead of the Focus
    /// hint chatter — that lives in the Help tab. Cached + refreshed off
    /// the hot path (main.rs, ~10s throttle).
    pub session_git_status: std::collections::HashMap<String, String>,
    /// REAL cursor position from the pane snapshot (row, col, visible),
    /// zero-based within the visible pane. Used to paint the caret exactly
    /// where the agent's input cursor is, instead of guessing the last
    /// non-empty line. None when no session is previewed.
    pub preview_cursor: Option<(u16, u16, bool)>,
    /// Styled preview rows (fg/bg/bold per span) from the pane snapshot.
    /// Renders Claude's colored UI — crucially the `/` command-menu
    /// selection highlight that plain text drops. None when browsing
    /// scrollback (plain-text path).
    pub preview_styled: Option<Vec<omega_core::session::PreviewLine>>,
    /// Pane revision of the currently-cached styled preview. The capture is
    /// skipped (no restyle/flatten) when the pane's revision still matches this.
    pub preview_revision: u64,
    /// Session name the cached preview/revision belongs to — reset the gate on
    /// a session switch so the new pane always gets a fresh capture.
    pub preview_session: Option<String>,
    /// Consecutive pane-capture failures for the current session. A pane in
    /// transition (a login pane swap, a zoom/terminal-resize reflow, a brief
    /// daemon hiccup) errors for a frame or two — clobbering the last-good
    /// preview with "(session has no pane content)" was the bug behind the
    /// view sticking on that message until a manual Ctrl+R. We keep the last
    /// good frame and force a recapture until this streak crosses a threshold,
    /// so only a GENUINELY dead pane shows the message.
    pub preview_fail_streak: u32,
    /// Scroll position measured as LINES UP FROM THE TAIL (0 = newest line).
    /// Bottom-anchored so the view stays stable when the capture buffer grows
    /// (visible-only → full scrollback history): "3 lines up from the tail"
    /// means the same thing whether the buffer is 50 or 1000 lines, so
    /// scrolling never jumps. Converted to a from-top offset at render time.
    pub preview_scroll: u16,
    /// Max scrollable lines (`content_lines - viewport_height`), written by
    /// the renderer each frame so the scroll setters can clamp against the
    /// real viewport without knowing the panel geometry themselves.
    pub preview_max_scroll: u16,
    /// Auto-follow tail — preview stays glued to the bottom (latest content)
    /// unless the user manually scrolls up. Mirrors `preview_scroll == 0`.
    pub preview_follow_tail: bool,
    /// Set true on the tail→history TRANSITION (first scroll-up out of tail
    /// mode). Signals the event loop to load full scrollback IMMEDIATELY —
    /// before the next draw — so the renderer computes a real `preview_max_scroll`
    /// on the SAME frame the user first scrolled. Without it, the first press
    /// sees the short visible buffer (max_scroll == 0) and the view can't move
    /// until the next cadence tick, requiring a wasted double-press. The loop
    /// clears it after consuming.
    pub preview_needs_history: bool,
    /// Session whose FULL scrollback is cached in `preview_content` while the
    /// user browses history. Set after a one-time deep capture on entry to
    /// history mode; reused on every subsequent frame so we don't re-capture
    /// the whole buffer at the idle cadence. Cleared when we return to the tail.
    pub preview_history_for: Option<String>,
    pub session_focus: SessionFocus,
    /// Tracks the last Tab press for double-tap detection (any tab).
    pub last_tab_press: Option<std::time::Instant>,
    /// Focus state at the START of a Tab sequence, captured on the first tap so
    /// a following double-tap can toggle the left menu cleanly (the single tap
    /// already moved focus, so the double tap reverts to this and toggles).
    pub tab_seq_start: Option<SessionFocus>,
    /// OmegaOS slash-command capture in chat focus. `Some(buf)` while the user
    /// is typing a shared OmegaOS command (e.g. "/projects") at the start of a
    /// chat line — keys are buffered by the TUI (not forwarded) until Enter
    /// resolves it. The SAME command set works on Telegram, the TUI, and any
    /// agent pane (one system, many brains).
    pub cmd_capture: Option<String>,
    /// Char count on the current chat line (reset on Enter) so a "/" only opens
    /// command capture when it is the first character of the line — typing a
    /// path/URL mid-line never triggers it. Best-effort.
    pub chat_line_chars: usize,
    /// Generic right-panel focus for non-Sessions 2-column tabs (Settings/Info).
    /// false = list focused, true = detail focused.
    pub detail_focused: bool,
    /// Detail panel fullscreen (Tab-Tab on a 2-column tab).
    pub detail_fullscreen: bool,
    /// Scroll position for the detail panel in Settings/Info/Monitor.
    pub detail_scroll: u16,
    pub current_session: Option<String>,
    /// Projects tab — selected project index.
    pub projects_selected: usize,
    /// Cached project registry for the Projects tab.
    pub project_registry: omega_core::project_manager::ProjectRegistry,
    /// Two-press confirm for removing a project (Projects tab 'x'): holds the
    /// project name armed by the first press; second 'x' on the same name fires.
    pub project_confirm_pending: Option<String>,
    /// Two-press confirm for the Monitor Telegram section's Enter→disconnect.
    /// Armed by the first focused-Enter, fired by the second. Cleared on nav.
    pub monitor_disconnect_armed: bool,
    /// Lazily-loaded providers config. Settings reads this on every keystroke,
    /// so we cache it here and only reload from disk after an edit/toggle
    /// commit (see `invalidate_providers`). Avoids per-keystroke disk I/O.
    providers_cache: Option<omega_core::providers::ProvidersConfig>,
    /// Claude OAuth re-login progress (Monitor → Account). Updated by the async
    /// engine via a shared sink drained each tick in the main loop.
    pub reauth_status: ReauthStatus,
}

impl App {
    pub fn new(config: OmegaConfig) -> Self {
        let current_session = std::env::var("RMUX")
            .ok()
            .and_then(|rmux_var| {
                rmux_var
                    .split(',')
                    .next()
                    .map(|s| s.to_string())
            })
            .or_else(|| std::env::var("RMUX_SESSION").ok())
            // Legacy fallback for users coming from a tmux setup
            .or_else(|| std::env::var("TMUX_SESSION").ok());

        Self {
            tab: Tab::Sessions,
            sessions: Vec::new(),
            rows: Vec::new(),
            selected: 0,
            menu_selected: 0,
            mouse_capture: true,
            menu_area: ratatui::layout::Rect::default(),
            menu_rendered_actions: Vec::new(),
            menu_fits: true,
            monitor_selected: 0,
            monitor_action_selected: 0,
            settings_selected: 0,
            settings_field_selected: 0,
            settings_confirm_pending: None,
            menu_confirm_pending: None,
            session_badges: std::collections::HashMap::new(),
            session_filter: None,
            new_project_cred_group: None,
            info_section_selected: 0,
            info_agent_selected: 0,
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            preview_styled: None,
            preview_revision: 0,
            preview_session: None,
            preview_fail_streak: 0,
            preview_inner_width: 0,
            preview_inner_height: 0,
            session_meta: std::collections::HashMap::new(),
            session_git_status: std::collections::HashMap::new(),
            preview_cursor: None,
            preview_scroll: 0,
            preview_max_scroll: 0,
            preview_follow_tail: true,
            preview_needs_history: false,
            preview_history_for: None,
            session_focus: SessionFocus::List,
            last_tab_press: None,
            tab_seq_start: None,
            cmd_capture: None,
            chat_line_chars: 0,
            detail_focused: false,
            detail_fullscreen: false,
            detail_scroll: 0,
            current_session,
            projects_selected: 0,
            project_registry: omega_core::project_manager::ProjectRegistry::load(),
            project_confirm_pending: None,
            monitor_disconnect_armed: false,
            providers_cache: None,
            reauth_status: ReauthStatus::Idle,
        }
    }

    /// Cached providers config, cloned out for the caller. The disk read +
    /// TOML parse (the expensive part) happens once; subsequent calls clone
    /// the in-memory struct, which is cheap relative to per-keystroke I/O.
    /// A clone (not a borrow) sidesteps the simultaneous `&app.config` borrow
    /// that `fields_for_section` needs.
    pub fn providers(&mut self) -> omega_core::providers::ProvidersConfig {
        if self.providers_cache.is_none() {
            self.providers_cache = Some(omega_core::providers::ProvidersConfig::load());
        }
        self.providers_cache.as_ref().unwrap().clone()
    }

    /// Drop the cached providers so the next `providers()` re-reads from disk.
    /// Call after any commit/toggle that mutates `~/.omega/providers.toml`.
    pub fn invalidate_providers(&mut self) {
        self.providers_cache = None;
    }

    pub fn refresh_projects(&mut self) {
        self.project_registry = omega_core::project_manager::ProjectRegistry::load();
        if self.projects_selected >= self.project_registry.projects.len()
            && !self.project_registry.projects.is_empty()
        {
            self.projects_selected = self.project_registry.projects.len() - 1;
        }
    }

    pub fn select_project_next(&mut self) {
        let count = self.project_registry.projects.len();
        if count > 0 {
            self.projects_selected = (self.projects_selected + 1) % count;
        }
        // Moving the cursor disarms any pending remove-confirm.
        self.project_confirm_pending = None;
    }

    pub fn select_project_prev(&mut self) {
        let count = self.project_registry.projects.len();
        if count > 0 {
            self.projects_selected = if self.projects_selected == 0 {
                count - 1
            } else {
                self.projects_selected - 1
            };
        }
        self.project_confirm_pending = None;
    }

    pub fn selected_project(&self) -> Option<&omega_core::project_manager::ManagedProject> {
        self.project_registry.projects.get(self.projects_selected)
    }

    /// Select a session by name (used after creating a session to auto-focus it).
    pub fn select_by_name(&mut self, name: &str) -> bool {
        for (i, entry) in self.sessions.iter().enumerate() {
            if entry.session.name == name {
                self.selected = i;
                return true;
            }
        }
        false
    }

    /// Enter chat focus on the currently selected session.
    pub fn enter_chat_focus(&mut self) {
        self.session_focus = SessionFocus::Chat;
        self.preview_follow_tail = true;
        self.preview_scroll = 0;
    }

    /// Jump the selection to the next session flagged Blocked or Failed
    /// (wraps around). Returns the name jumped to, if any.
    pub fn jump_to_next_flagged(&mut self) -> Option<String> {
        use omega_core::done::DoneStatus;
        let n = self.sessions.len();
        if n == 0 {
            return None;
        }
        let mut target = None;
        for off in 1..=n {
            let idx = (self.selected + off) % n;
            if matches!(
                self.session_badges.get(&self.sessions[idx].session.name),
                Some(DoneStatus::Blocked) | Some(DoneStatus::Failed)
            ) {
                target = Some(idx);
                break;
            }
        }
        let idx = target?;
        self.selected = idx;
        Some(self.sessions[idx].session.name.clone())
    }

    /// Handle a Tab press in the Sessions tab.
    ///   • Single Tab  → NAVIGATE: toggle the session list ↔ the session itself
    ///     (List ↔ Chat; from fullscreen → back to the list).
    ///   • Tab-Tab (rapid, <400ms) → toggle the LEFT SESSION MENU: hide it so the
    ///     Claude session takes the full width (great on small screens), or show
    ///     it again. The toggle keys off the state the sequence STARTED in, so it
    ///     is a clean hide↔show even though the first tap already moved focus.
    /// Called from BOTH the list handler and the chat handler so the behavior is
    /// identical wherever the sequence begins.
    pub fn handle_tab_in_sessions(&mut self) {
        const DOUBLE_TAP_MS: u128 = 400;
        let now = std::time::Instant::now();
        let is_double = self
            .last_tab_press
            .map(|t| now.duration_since(t).as_millis() < DOUBLE_TAP_MS)
            .unwrap_or(false);

        if is_double {
            // Second tap: revert the first tap's navigation and toggle the menu.
            self.last_tab_press = None;
            let start = self.tab_seq_start.take().unwrap_or(self.session_focus);
            self.session_focus = match start {
                SessionFocus::ChatFullscreen => SessionFocus::List, // show the menu
                _ => SessionFocus::ChatFullscreen,                  // hide the menu
            };
        } else {
            // First tap: remember the starting state, then navigate.
            self.tab_seq_start = Some(self.session_focus);
            self.last_tab_press = Some(now);
            self.session_focus = match self.session_focus {
                SessionFocus::List => SessionFocus::Chat,
                _ => SessionFocus::List,
            };
        }

        // A Tab transition starts a fresh chat input line and drops any
        // half-typed OmegaOS command, so the "/" trigger fires at line start.
        self.cmd_capture = None;
        self.chat_line_chars = 0;
        if self.session_focus != SessionFocus::List {
            self.preview_follow_tail = true;
            self.preview_scroll = 0;
        }
    }

    /// Tab in a 2-column tab (Settings / Info): single = toggle list↔detail,
    /// double = fullscreen detail, double again = back to list.
    pub fn handle_tab_in_2col(&mut self) {
        const DOUBLE_TAP_MS: u128 = 400;
        let now = std::time::Instant::now();
        let is_double = self
            .last_tab_press
            .map(|t| now.duration_since(t).as_millis() < DOUBLE_TAP_MS)
            .unwrap_or(false);
        self.last_tab_press = Some(now);

        if is_double {
            // Double tap: toggle fullscreen
            self.detail_fullscreen = !self.detail_fullscreen;
            self.detail_focused = self.detail_fullscreen;
        } else {
            if self.detail_fullscreen {
                // Single tap from fullscreen → back to list
                self.detail_fullscreen = false;
                self.detail_focused = false;
            } else {
                // Toggle list ↔ detail
                self.detail_focused = !self.detail_focused;
            }
        }
    }

    /// Reset 2-column focus to list when switching tabs.
    pub fn reset_2col_focus(&mut self) {
        self.detail_focused = false;
        self.detail_fullscreen = false;
        self.detail_scroll = 0;
    }

    pub fn scroll_detail_down(&mut self, lines: u16) {
        self.detail_scroll = self.detail_scroll.saturating_add(lines);
    }
    pub fn scroll_detail_up(&mut self, lines: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(lines);
    }

    // Scroll is measured from the tail: 0 = newest. "Down" moves toward the
    // newest line (decreasing the from-tail offset), "up" moves into history.

    pub fn scroll_preview_down(&mut self, lines: u16) {
        self.preview_scroll = self.preview_scroll.saturating_sub(lines);
        // Reaching the tail re-glues to live follow (and lets refresh_preview
        // switch back to the cheap visible-only capture).
        if self.preview_scroll == 0 {
            self.preview_follow_tail = true;
        }
    }

    pub fn scroll_preview_up(&mut self, lines: u16) {
        // When leaving tail mode, preview_max_scroll is 0 (content == viewport).
        // Skip the clamp on this first transition so the scroll position actually
        // advances; the next renderer tick loads full history and sets a real
        // preview_max_scroll that subsequent presses clamp against.
        let was_following = self.preview_follow_tail;
        self.preview_follow_tail = false;
        if was_following && self.preview_max_scroll == 0 {
            self.preview_scroll = lines;
            // First press out of tail: the visible buffer is short, so the
            // renderer would clamp the view back to the bottom. Tell the loop
            // to load scrollback NOW so this same press actually moves the view.
            self.preview_needs_history = true;
        } else {
            self.preview_scroll = self
                .preview_scroll
                .saturating_add(lines)
                .min(self.preview_max_scroll);
        }
    }

    pub fn scroll_preview_home(&mut self) {
        // Top of available history.
        self.preview_scroll = self.preview_max_scroll;
        self.preview_follow_tail = false;
    }

    pub fn scroll_preview_end(&mut self) {
        self.preview_scroll = 0;
        self.preview_follow_tail = true;
    }

    pub fn select_menu_next(&mut self) {
        let count = MenuAction::all().len();
        self.menu_selected = (self.menu_selected + 1) % count;
    }

    pub fn select_menu_prev(&mut self) {
        let count = MenuAction::all().len();
        self.menu_selected = if self.menu_selected == 0 {
            count - 1
        } else {
            self.menu_selected - 1
        };
    }

    pub fn selected_menu_action(&self) -> MenuAction {
        MenuAction::all()[self.menu_selected]
    }

    // ── Monitor section list (left panel) ───────────────────────────────────
    pub fn select_monitor_next(&mut self) {
        let count = MonitorSection::all().len();
        self.monitor_selected = (self.monitor_selected + 1) % count;
        self.monitor_action_selected = 0;
        self.detail_scroll = 0;
        // Leaving the Telegram section drops any armed disconnect-confirm.
        self.monitor_disconnect_armed = false;
    }

    pub fn select_monitor_prev(&mut self) {
        let count = MonitorSection::all().len();
        self.monitor_selected = if self.monitor_selected == 0 {
            count - 1
        } else {
            self.monitor_selected - 1
        };
        self.monitor_action_selected = 0;
        self.detail_scroll = 0;
        self.monitor_disconnect_armed = false;
    }

    pub fn selected_monitor_section(&self) -> MonitorSection {
        MonitorSection::all()[self.monitor_selected]
    }

    // ── Monitor action cursor (inside the Actions section) ───────────────────
    pub fn select_monitor_action_next(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_action_selected = (self.monitor_action_selected + 1) % count;
    }

    pub fn select_monitor_action_prev(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_action_selected = if self.monitor_action_selected == 0 {
            count - 1
        } else {
            self.monitor_action_selected - 1
        };
    }

    pub fn selected_monitor_action(&self) -> MonitorAction {
        MonitorAction::all()[self.monitor_action_selected]
    }

    pub fn select_settings_next(&mut self) {
        let count = SettingsSection::all().len();
        self.settings_selected = (self.settings_selected + 1) % count;
        self.settings_field_selected = 0;
        self.settings_confirm_pending = None;
    }

    pub fn select_settings_prev(&mut self) {
        let count = SettingsSection::all().len();
        self.settings_selected = if self.settings_selected == 0 {
            count - 1
        } else {
            self.settings_selected - 1
        };
        self.settings_field_selected = 0;
        self.settings_confirm_pending = None;
    }

    pub fn selected_settings_section(&self) -> SettingsSection {
        SettingsSection::all()[self.settings_selected]
    }

    /// Reset field cursor when section changes
    pub fn reset_settings_field(&mut self) {
        self.settings_field_selected = 0;
    }

    pub fn select_settings_field_next(&mut self, max: usize) {
        if max == 0 { return; }
        self.settings_field_selected = (self.settings_field_selected + 1) % max;
    }
    pub fn select_settings_field_prev(&mut self, max: usize) {
        if max == 0 { return; }
        self.settings_field_selected = if self.settings_field_selected == 0 { max - 1 } else { self.settings_field_selected - 1 };
    }

    pub async fn refresh_preview(&mut self) -> anyhow::Result<()> {
        let name = match self.selected_session() {
            Some(e) => e.session.name.clone(),
            None => {
                // No session selected → nothing can fail. Zero the per-session
                // failure counter so a dead session's streak can't poison the
                // next session that becomes selected.
                self.preview_content = String::new();
                self.preview_fail_streak = 0;
                return Ok(());
            }
        };

        // Avoid recursion: if previewing the session we're running inside, show static msg
        if let Some(ref cur) = self.current_session {
            if cur == &name {
                self.preview_content =
                    "(this is the session running OmegaOS — preview disabled to prevent recursion)"
                        .to_string();
                return Ok(());
            }
        }

        // Master + Telegram unconfigured → replace the bare log-tail mirror
        // with a guided call-to-action. Pressing Enter here opens the existing
        // Telegram setup wizard (see the Sessions Enter hook in input.rs).
        if omega_core::aisb::is_master(&name)
            && !omega_core::monitor::OmegaTelegramConfig::exists()
        {
            self.preview_content = "\n  ★ AISB Master — your Telegram brain\n\n  \
                Not yet connected. Once you link a Telegram bot, every message you\n  \
                send it is classified and routed to the right oracle/agent, and the\n  \
                replies stream here.\n\n  \
                ▶ Press Enter to run the setup wizard (guided, no command needed).\n"
                .to_string();
            self.preview_styled = None;
            self.preview_cursor = None;
            return Ok(());
        }

        // Cached connection — avoid a fresh rmux daemon socket per refresh.
        let mgr = omega_core::session::SessionManager::connect_cached().await?;
        // Hot tail path stays on the cheap visible-only snapshot. Only when the
        // user is browsing history (follow_tail == false) do we pay for a full
        // scrollback capture, so there is real content above the screen to
        // scroll into instead of an empty void.
        if self.preview_follow_tail {
            // Back on the tail: drop any cached scrollback so the next scroll-up
            // re-captures fresh history (picking up lines added since).
            self.preview_history_for = None;
            // Tail path: capture STYLED rows + text + REAL cursor together.
            // Styled rows carry the `/` selector highlight + Claude's
            // colored UI; plain text is kept as a fallback + for scroll math.
            // Revision gate: only pay the full restyle+flatten when the pane
            // actually changed. Force a fresh capture (since=0) on a session
            // switch so the new pane's content always loads.
            let cache_valid = self.preview_styled.is_some()
                && self.preview_session.as_deref() == Some(name.as_str());
            // preview_fail_streak is a PER-SESSION counter. On a session switch
            // (the rendered session no longer matches `name`) the new pane's
            // capture status is independent, so zero the streak — otherwise a
            // stale streak from the previous session can trip the ≥3 "dead pane"
            // threshold on the new session's very first transient failure.
            if self.preview_session.as_deref() != Some(name.as_str()) {
                self.preview_fail_streak = 0;
            }
            let since = if cache_valid { self.preview_revision } else { 0 };
            match mgr.capture_pane_styled(&name, since).await {
                // Pane unchanged since last render — keep the cached preview,
                // skip the ~10k-cell restyle + the text flatten entirely.
                Ok(omega_core::session::StyledCapture::Unchanged) => {
                    self.preview_fail_streak = 0;
                }
                Ok(omega_core::session::StyledCapture::Changed {
                    rows,
                    cursor_row,
                    cursor_col,
                    cursor_visible,
                    revision,
                }) => {
                    // Flatten styled rows to plain text for scroll/cursor math.
                    self.preview_content = rows
                        .iter()
                        .map(|line| {
                            line.iter().map(|s| s.text.as_str()).collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.preview_styled = Some(rows);
                    self.preview_cursor = Some((cursor_row, cursor_col, cursor_visible));
                    self.preview_revision = revision;
                    self.preview_session = Some(name.clone());
                    self.preview_fail_streak = 0;
                }
                Err(_) => {
                    // A capture can fail transiently while a pane is in
                    // transition: a claude-login pane swap, a zoom / terminal
                    // resize reflow, or a brief daemon hiccup. Do NOT flash
                    // "(session has no pane content)" and stick on it — keep the
                    // last good frame and force a full recapture next tick
                    // (revision=0). Only a SUSTAINED failure (≥3 ticks) — i.e. a
                    // genuinely dead pane — replaces the view with the message.
                    self.preview_fail_streak = self.preview_fail_streak.saturating_add(1);
                    self.preview_revision = 0; // force fresh recapture next tick
                    if self.preview_fail_streak >= 3 {
                        self.preview_content = String::from("(session has no pane content)");
                        self.preview_styled = None;
                        self.preview_cursor = None;
                        self.preview_session = None;
                    }
                    // else: retain last-good preview_{content,styled,cursor,session}.
                }
            }
        } else {
            // History-browsing path: plain text (scrollback has no styling),
            // cursor meaningless when scrolled back.
            self.preview_styled = None;
            self.preview_cursor = None;
            // Lazy history: capture the full retained scrollback ONCE on entry to
            // history-browsing for this session, then reuse it while scrolling —
            // instead of re-capturing the whole buffer every cadence tick (which
            // pegged the daemon while browsing). The tail path clears the cache,
            // so the next scroll-up gets a fresh deep capture. Depth matches the
            // rmux history-limit so the user can scroll to the very top.
            if self.preview_history_for.as_deref() != Some(name.as_str()) {
                // Session switch in history-browse mode: same per-session reset
                // as the tail path so a prior session's streak can't poison this
                // one's ≥3 dead-pane threshold on the first transient failure.
                self.preview_fail_streak = 0;
                match mgr.capture_pane_history(&name, 500_000).await {
                    Ok(content) => {
                        self.preview_content = content;
                        self.preview_history_for = Some(name.clone());
                        self.preview_fail_streak = 0;
                    }
                    Err(_) => {
                        // Same sticky-last-good policy as the tail path: a
                        // transient capture error must not clobber the view.
                        self.preview_fail_streak = self.preview_fail_streak.saturating_add(1);
                        if self.preview_fail_streak >= 3 {
                            self.preview_content = String::from("(session has no pane content)");
                        }
                        self.preview_history_for = None;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        // Snapshot user-toggled protection flags BEFORE clearing self.sessions
        // — otherwise every 5s refresh wipes the lock the user just set.
        let protected_names: std::collections::HashSet<String> = self
            .sessions
            .iter()
            .filter(|e| e.is_protected)
            .map(|e| e.session.name.clone())
            .collect();

        // Cached daemon socket — refresh runs every ~2s, so a fresh connect()
        // each time is wasteful. Matches refresh_preview()'s connect_cached().
        let mgr = SessionManager::connect_cached().await?;
        let raw_sessions = mgr.list_sessions().await?;

        // Hide infrastructure daemons (Telegram bridge, reauth helper).
        // Same list as the Telegram bridge filters in /sessions — keep them
        // in sync if you add a new background process.
        // aisb-reauth is intentionally VISIBLE: when the operator triggers a
        // Claude login it must show up in the sessions table so they can see the
        // login session open, run /login, and close on success.
        let hidden_prefixes = ["omega-telegram-bridge"];
        let filter_lc = self.session_filter.as_ref().map(|q| q.to_lowercase());
        let sessions: Vec<_> = raw_sessions
            .into_iter()
            .filter(|s| !hidden_prefixes.iter().any(|p| s.name.starts_with(p)))
            .filter(|s| match &filter_lc {
                Some(q) => s.name.to_lowercase().contains(q.as_str()),
                None => true,
            })
            .collect();

        // Status badges (done/blocked) for the list — read once per refresh.
        // A worker-blocked signal wins over a stale done.json.
        self.session_badges.clear();
        for d in omega_core::done::DoneSignal::read_all(&self.config.state_dir) {
            self.session_badges.insert(d.session, d.status);
        }
        for b in omega_core::done::WorkerBlocked::read_all(&self.config.state_dir) {
            self.session_badges
                .insert(b.session, omega_core::done::DoneStatus::Blocked);
        }

        let all_progress = ProgressInfo::read_all(&self.config.state_dir);

        // Worker → governing-oracle map, read from each oracle's persisted
        // state (spawn-worker records the link there). Lets the menu nest a
        // worker under the SPECIFIC oracle that spawned it, not just its
        // project. Workers with no recorded parent fall back (single-oracle
        // project → that oracle; multi-oracle → project level — no guessing).
        let mut worker_oracle: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for st in omega_core::oracle_lifecycle::OracleState::read_all(&self.config.state_dir) {
            for w in &st.workers {
                worker_oracle.insert(w.session_name.clone(), st.oracle_name.clone());
            }
        }

        self.sessions.clear();
        self.rows.clear();

        // ── Section 1: Master AISB pinned at top ────────────────────────────
        if let Some(master) = sessions
            .iter()
            .find(|s| omega_core::aisb::is_master(&s.name))
        {
            self.rows.push(SessionRow::Header("─ AISB Master ─".to_string()));
            let entry = SessionEntry {
                session: master.clone(),
                progress: None,
                is_current: false,
                // Master is NOT protected anymore — killing it triggers an
                // auto-respawn (see KillSession handler + bridge ensure_master
                // path). The user can press 'x' freely; the daemon comes back
                // on the next Telegram message OR on the next TUI refresh.
                is_protected: false,
                tree_prefix: "★ ".to_string(),
            };
            self.sessions.push(entry);
            self.rows.push(SessionRow::Entry(
                self.sessions.last().unwrap().clone_for_row(),
            ));
        }

        // ── Section 2: Project-grouped sessions (Oracles + Workers + Home) ──
        let mut last_section: Option<String> = None;
        let mut group: Vec<&OmegaSession> = Vec::new();

        for session in sessions.iter() {
            if omega_core::aisb::is_master(&session.name) {
                continue;
            }
            let section_label = section_for(session);
            if last_section.as_ref() != Some(&section_label) && !group.is_empty() {
                self.flush_group_rows(&group, &all_progress, &worker_oracle, last_section.as_deref());
                group.clear();
            }
            group.push(session);
            last_section = Some(section_label);
        }
        if !group.is_empty() {
            self.flush_group_rows(&group, &all_progress, &worker_oracle, last_section.as_deref());
        }

        // Restore the user's manual protection toggles
        for entry in self.sessions.iter_mut() {
            if protected_names.contains(&entry.session.name) {
                entry.is_protected = true;
            }
        }
        // Same for the rendered row clones
        for row in self.rows.iter_mut() {
            if let SessionRow::Entry(e) = row {
                if protected_names.contains(&e.session.name) {
                    e.is_protected = true;
                }
            }
        }

        if self.selected >= self.sessions.len() && !self.sessions.is_empty() {
            self.selected = self.sessions.len() - 1;
        }

        Ok(())
    }

    fn flush_group_rows(
        &mut self,
        group: &[&OmegaSession],
        all_progress: &[ProgressInfo],
        worker_oracle: &std::collections::HashMap<String, String>,
        section_label: Option<&str>,
    ) {
        if let Some(label) = section_label {
            self.rows
                .push(SessionRow::Header(format!("─ {} ─", label)));
        }

        let oracles: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();
        let workers: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role == SessionRole::Worker)
            .collect();
        let others: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role != SessionRole::Oracle && s.role != SessionRole::Worker)
            .collect();

        // Resolve a worker's governing oracle: the recorded link first; else,
        // only when the project has exactly ONE oracle, that sole oracle; else
        // None (unattributed — never guessed between several oracles).
        let sole_oracle: Option<&str> = if oracles.len() == 1 {
            Some(oracles[0].name.as_str())
        } else {
            None
        };
        let parent_of = |w: &OmegaSession| -> Option<String> {
            worker_oracle
                .get(&w.name)
                .cloned()
                .or_else(|| sole_oracle.map(|s| s.to_string()))
        };

        // Each oracle, immediately followed by the workers it governs (├/└).
        let mut shown: std::collections::HashSet<String> = std::collections::HashSet::new();
        for &oracle in &oracles {
            self.push_session_row(oracle, all_progress, String::new());
            let mine: Vec<&OmegaSession> = workers
                .iter()
                .copied()
                .filter(|w| parent_of(w).as_deref() == Some(oracle.name.as_str()))
                .collect();
            let n = mine.len();
            for (i, w) in mine.into_iter().enumerate() {
                let prefix = if i + 1 == n { "  └ " } else { "  ├ " };
                self.push_session_row(w, all_progress, prefix.to_string());
                shown.insert(w.name.clone());
            }
        }

        // Workers with no resolvable parent in this group (multi-oracle project
        // with no recorded link, or a killed oracle): project level, after the
        // oracles. Honest — we show them rather than hide or misattribute them.
        for &w in &workers {
            if !shown.contains(&w.name) {
                self.push_session_row(w, all_progress, String::new());
            }
        }

        // Anything else in the section (e.g. a stray shell classified here).
        for &o in &others {
            self.push_session_row(o, all_progress, String::new());
        }
    }

    /// Append one session as a list row (+ its progress + tree prefix).
    /// `is_protected` is restored by the caller after the whole group is built.
    fn push_session_row(
        &mut self,
        session: &OmegaSession,
        all_progress: &[ProgressInfo],
        tree_prefix: String,
    ) {
        let progress = all_progress
            .iter()
            .find(|p| p.session == session.name)
            .cloned();
        let entry = SessionEntry {
            session: session.clone(),
            progress,
            is_current: false,
            is_protected: false,
            tree_prefix,
        };
        self.sessions.push(entry);
        self.rows
            .push(SessionRow::Entry(self.sessions.last().unwrap().clone_for_row()));
    }

    pub fn selected_session(&self) -> Option<&SessionEntry> {
        self.sessions.get(self.selected)
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Menu,
            Tab::Menu => Tab::Monitor,
            Tab::Monitor => Tab::Projects,
            Tab::Projects => Tab::Settings,
            Tab::Settings => Tab::Agentic,
            Tab::Agentic => Tab::Help,
            Tab::Help => Tab::Sessions,
        };
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Help,
            Tab::Menu => Tab::Sessions,
            Tab::Monitor => Tab::Menu,
            Tab::Projects => Tab::Monitor,
            Tab::Settings => Tab::Projects,
            Tab::Agentic => Tab::Settings,
            Tab::Help => Tab::Agentic,
        };
    }

    pub fn select_info_next(&mut self) {
        let count = InfoSection::all().len();
        self.info_section_selected = (self.info_section_selected + 1) % count;
        self.info_agent_selected = 0;
    }

    pub fn select_info_prev(&mut self) {
        let count = InfoSection::all().len();
        self.info_section_selected = if self.info_section_selected == 0 {
            count - 1
        } else {
            self.info_section_selected - 1
        };
        self.info_agent_selected = 0;
    }

    pub fn selected_info_section(&self) -> InfoSection {
        InfoSection::all()[self.info_section_selected]
    }

    pub fn select_info_agent_next(&mut self) {
        let count = omega_core::aisb_agents::AisbAgent::all().len();
        self.info_agent_selected = (self.info_agent_selected + 1) % count;
    }
    pub fn select_info_agent_prev(&mut self) {
        let count = omega_core::aisb_agents::AisbAgent::all().len();
        self.info_agent_selected = if self.info_agent_selected == 0 {
            count - 1
        } else {
            self.info_agent_selected - 1
        };
    }

    pub fn select_next(&mut self) {
        if !self.sessions.is_empty() {
            self.selected = (self.selected + 1) % self.sessions.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.sessions.is_empty() {
            self.selected = if self.selected == 0 {
                self.sessions.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

#[cfg(test)]
mod nesting_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use omega_core::session::OmegaSession;

    fn rows_of(app: &App) -> Vec<(String, String)> {
        app.rows
            .iter()
            .filter_map(|r| match r {
                SessionRow::Entry(e) => {
                    Some((e.tree_prefix.clone(), e.session.name.clone()))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn single_oracle_nests_all_its_workers() {
        // One oracle in the project → every worker hangs under it (├/└), even
        // with no recorded link (single-oracle fallback). This is the live
        // DentistryGPT case once only one oracle remains.
        let mut app = App::new(OmegaConfig::default());
        let sessions = vec![
            OmegaSession::classify("oracle-DentistryGPT-2"),
            OmegaSession::classify("DentistryGPT-worker-agent-actions"),
            OmegaSession::classify("DentistryGPT-worker-e2e-agents"),
        ];
        let group: Vec<&OmegaSession> = sessions.iter().collect();
        let map = std::collections::HashMap::new();
        app.flush_group_rows(&group, &[], &map, Some("DentistryGPT"));
        let rows = rows_of(&app);
        assert_eq!(rows[0], (String::new(), "oracle-DentistryGPT-2".into()));
        assert_eq!(rows[1].1, "DentistryGPT-worker-agent-actions");
        assert!(rows[1].0.contains('├'), "expected ├, got {:?}", rows[1].0);
        assert_eq!(rows[2].1, "DentistryGPT-worker-e2e-agents");
        assert!(rows[2].0.contains('└'), "expected └, got {:?}", rows[2].0);
    }

    #[test]
    fn recorded_links_nest_each_worker_under_its_own_oracle() {
        // Two oracles in one project: the recorded worker→oracle map decides
        // which worker nests under which oracle (no guessing).
        let mut app = App::new(OmegaConfig::default());
        let sessions = vec![
            OmegaSession::classify("oracle-Causio-1"),
            OmegaSession::classify("oracle-Causio-2"),
            OmegaSession::classify("Causio-worker-a"),
            OmegaSession::classify("Causio-worker-b"),
        ];
        let group: Vec<&OmegaSession> = sessions.iter().collect();
        let mut map = std::collections::HashMap::new();
        map.insert("Causio-worker-a".to_string(), "oracle-Causio-2".to_string());
        map.insert("Causio-worker-b".to_string(), "oracle-Causio-1".to_string());
        app.flush_group_rows(&group, &[], &map, Some("Causio"));
        let order: Vec<String> = rows_of(&app).into_iter().map(|(_, n)| n).collect();
        assert_eq!(
            order,
            vec![
                "oracle-Causio-1",
                "Causio-worker-b", // its recorded child
                "oracle-Causio-2",
                "Causio-worker-a", // its recorded child
            ]
        );
    }
}
