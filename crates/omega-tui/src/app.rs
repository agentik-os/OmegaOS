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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoSection {
    AisbAgents,
    Oracle,
    Workers,
    Rules,
}

impl InfoSection {
    pub fn all() -> &'static [InfoSection] {
        &[
            InfoSection::AisbAgents,
            InfoSection::Oracle,
            InfoSection::Workers,
            InfoSection::Rules,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
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
    NewSession,
    NewSessionAgent(String),
    NewSessionPrompt(String, String),
    DispatchProject,
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
}

/// New-project wizard option lists. `(id, label)` — `id` is the token passed to
/// the `/omega-new-project` skill; `label` is what the picker shows. Single
/// source of truth for both the menu UI and the spawned command.
pub const NEW_PROJECT_CATEGORIES: &[(&str, &str)] = &[
    ("works", "Works — personal / internal  (work/ under your projects dir)"),
    ("client", "Client work  (clients/ under your projects dir)"),
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
            SettingsField::Info(s) => s,
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
            out.push(SettingsField::EditText {
                label: "Model (e.g. opus, sonnet, haiku)".to_string(),
                config_key: "claude.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
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
            out.push(SettingsField::EditText {
                label: "Model".to_string(),
                config_key: "codex.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
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
            out.push(SettingsField::EditText {
                label: "Model".to_string(),
                config_key: "gemini.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
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
            out.push(SettingsField::EditText {
                label: "Model".to_string(),
                config_key: "glm.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
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
            out.push(SettingsField::EditText {
                label: "Model".to_string(),
                config_key: "pi.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
            out.extend(install_actions_for(Agent::Pi));
        }
        SettingsSection::Hermes => {
            let c = &providers.hermes;
            out.push(SettingsField::EditText {
                label: "Model".to_string(),
                config_key: "hermes.model".to_string(),
                current_value: c.model.clone(),
                masked: false,
            });
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
            confirm_first: true,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAction {
    Login,
    TelegramSetup,
    TelegramDisconnect,
    ProvisioningSetup,
    RefreshBilling,
}

impl MonitorAction {
    pub fn all() -> &'static [MonitorAction] {
        &[
            MonitorAction::Login,
            MonitorAction::TelegramSetup,
            MonitorAction::TelegramDisconnect,
            MonitorAction::ProvisioningSetup,
            MonitorAction::RefreshBilling,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorAction::Login => "Login / re-auth Claude   (opens session with `claude /login`)",
            MonitorAction::TelegramSetup => "Set up Omega Telegram bot   (omega telegram setup …)",
            MonitorAction::TelegramDisconnect => "Disconnect Telegram bot   (removes ~/.omega/telegram.toml)",
            MonitorAction::ProvisioningSetup => "Set up project provisioning keys   (Vercel/Convex/GitHub/Stripe → ~/.omega/provisioning)",
            MonitorAction::RefreshBilling => "Refresh billing now   (re-runs usage-monitor.sh)",
        }
    }
    pub fn shortcut(&self) -> &'static str {
        match self {
            MonitorAction::Login => "L",
            MonitorAction::TelegramSetup => "T",
            MonitorAction::TelegramDisconnect => "D",
            MonitorAction::ProvisioningSetup => "P",
            MonitorAction::RefreshBilling => "B",
        }
    }
}

pub struct App {
    pub tab: Tab,
    pub sessions: Vec<SessionEntry>,
    /// Renderable rows including section headers (parallel to sessions, but
    /// includes Header variants between groups). Built by refresh().
    pub rows: Vec<SessionRow>,
    pub selected: usize,
    pub menu_selected: usize,
    pub monitor_selected: usize,
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
    pub info_section_selected: usize,
    /// When the AISB Agents sub-section is active, which of the 13 is highlighted.
    pub info_agent_selected: usize,
    pub agent_picker_index: usize,
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
    pub session_focus: SessionFocus,
    /// Tracks the last Tab press for double-tap detection (any tab).
    pub last_tab_press: Option<std::time::Instant>,
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
    /// Lazily-loaded providers config. Settings reads this on every keystroke,
    /// so we cache it here and only reload from disk after an edit/toggle
    /// commit (see `invalidate_providers`). Avoids per-keystroke disk I/O.
    providers_cache: Option<omega_core::providers::ProvidersConfig>,
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
            monitor_selected: 0,
            settings_selected: 0,
            settings_field_selected: 0,
            settings_confirm_pending: None,
            menu_confirm_pending: None,
            session_badges: std::collections::HashMap::new(),
            session_filter: None,
            info_section_selected: 0,
            info_agent_selected: 0,
            agent_picker_index: 0,
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            preview_styled: None,
            preview_inner_width: 0,
            preview_inner_height: 0,
            session_meta: std::collections::HashMap::new(),
            session_git_status: std::collections::HashMap::new(),
            preview_cursor: None,
            preview_scroll: 0,
            preview_max_scroll: 0,
            preview_follow_tail: true,
            preview_needs_history: false,
            session_focus: SessionFocus::List,
            last_tab_press: None,
            detail_focused: false,
            detail_fullscreen: false,
            detail_scroll: 0,
            current_session,
            projects_selected: 0,
            project_registry: omega_core::project_manager::ProjectRegistry::load(),
            providers_cache: None,
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

    /// Handle a Tab press in the Sessions tab. Detects double-tap (within
    /// 400ms) for chat fullscreen mode.
    pub fn handle_tab_in_sessions(&mut self) {
        const DOUBLE_TAP_MS: u128 = 400;
        let now = std::time::Instant::now();
        let is_double = self
            .last_tab_press
            .map(|t| now.duration_since(t).as_millis() < DOUBLE_TAP_MS)
            .unwrap_or(false);
        self.last_tab_press = Some(now);

        self.session_focus = match (self.session_focus, is_double) {
            // Single tap: cycle List → Chat → List
            (SessionFocus::List, false) => SessionFocus::Chat,
            (SessionFocus::Chat, false) => SessionFocus::List,
            (SessionFocus::ChatFullscreen, false) => SessionFocus::List,
            // Double tap from Chat: expand to fullscreen
            (SessionFocus::Chat, true) => SessionFocus::ChatFullscreen,
            // Double tap from Fullscreen: back to List
            (SessionFocus::ChatFullscreen, true) => SessionFocus::List,
            // Double tap from List: go straight to fullscreen
            (SessionFocus::List, true) => SessionFocus::ChatFullscreen,
        };
        // When entering any chat focus, tail follow on
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

    pub fn agent_picker_next(&mut self) {
        let count = omega_core::agents::Agent::all().len();
        self.agent_picker_index = (self.agent_picker_index + 1) % count;
    }

    pub fn agent_picker_prev(&mut self) {
        let count = omega_core::agents::Agent::all().len();
        self.agent_picker_index = if self.agent_picker_index == 0 {
            count - 1
        } else {
            self.agent_picker_index - 1
        };
    }

    pub fn selected_agent(&self) -> omega_core::agents::Agent {
        omega_core::agents::Agent::all()[self.agent_picker_index]
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

    pub fn select_monitor_next(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_selected = (self.monitor_selected + 1) % count;
    }

    pub fn select_monitor_prev(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_selected = if self.monitor_selected == 0 {
            count - 1
        } else {
            self.monitor_selected - 1
        };
    }

    pub fn selected_monitor_action(&self) -> MonitorAction {
        MonitorAction::all()[self.monitor_selected]
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
                self.preview_content = String::new();
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

        // Cached connection — avoid a fresh rmux daemon socket per refresh.
        let mgr = omega_core::session::SessionManager::connect_cached().await?;
        // Hot tail path stays on the cheap visible-only snapshot. Only when the
        // user is browsing history (follow_tail == false) do we pay for a full
        // scrollback capture, so there is real content above the screen to
        // scroll into instead of an empty void.
        if self.preview_follow_tail {
            // Tail path: capture STYLED rows + text + REAL cursor together.
            // Styled rows carry the `/` selector highlight + Claude's
            // colored UI; plain text is kept as a fallback + for scroll math.
            match mgr.capture_pane_styled(&name).await {
                Ok((styled, row, col, visible)) => {
                    // Flatten styled rows to plain text for scroll/cursor math.
                    self.preview_content = styled
                        .iter()
                        .map(|line| {
                            line.iter().map(|s| s.text.as_str()).collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.preview_styled = Some(styled);
                    self.preview_cursor = Some((row, col, visible));
                }
                Err(_) => {
                    self.preview_content = String::from("(session has no pane content)");
                    self.preview_styled = None;
                    self.preview_cursor = None;
                }
            }
        } else {
            // History-browsing path: plain text (scrollback has no styling),
            // cursor meaningless when scrolled back.
            self.preview_styled = None;
            // Capture the FULL retained scrollback (not a 1000-line slice) so
            // the user can scroll all the way to the very top of the
            // conversation. rmux clamps to whatever history-limit retains.
            match mgr.capture_pane_history(&name, 100_000).await {
                Ok(content) => {
                    self.preview_content = content;
                    self.preview_cursor = None;
                }
                Err(_) => {
                    self.preview_content = String::from("(session has no pane content)");
                    self.preview_cursor = None;
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
        let hidden_prefixes = ["omega-telegram-bridge", "aisb-reauth"];
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
                self.flush_group_rows(&group, &all_progress, last_section.as_deref());
                group.clear();
            }
            group.push(session);
            last_section = Some(section_label);
        }
        if !group.is_empty() {
            self.flush_group_rows(&group, &all_progress, last_section.as_deref());
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
        section_label: Option<&str>,
    ) {
        let has_oracle = group.iter().any(|s| s.role == SessionRole::Oracle);
        let worker_count = group.iter().filter(|s| s.role == SessionRole::Worker).count();
        let show_tree = has_oracle && worker_count > 0;

        if let Some(label) = section_label {
            self.rows
                .push(SessionRow::Header(format!("─ {} ─", label)));
        }

        for (i, session) in group.iter().enumerate() {
            let progress = all_progress
                .iter()
                .find(|p| p.session == session.name)
                .cloned();

            let tree_prefix = if show_tree && session.role == SessionRole::Worker {
                if i == group.len() - 1 {
                    "  └ ".to_string()
                } else {
                    "  ├ ".to_string()
                }
            } else {
                String::new()
            };

            let entry = SessionEntry {
                session: (*session).clone(),
                progress,
                is_current: false,
                is_protected: false, // restored after the loop below
                tree_prefix,
            };
            self.sessions.push(entry);
            self.rows.push(SessionRow::Entry(
                self.sessions.last().unwrap().clone_for_row(),
            ));
        }
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
