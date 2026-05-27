use omega_core::config::OmegaConfig;
use omega_core::progress::ProgressInfo;
use omega_core::session::{OmegaSession, SessionManager, SessionRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Sessions,
    Menu,
    Monitor,
    Settings,
    Help,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewClaude,
    NewCodex,
    NewGemini,
    NewPi,
    NewGlm,
    NewTerminal,
    DispatchOracle,
    Refresh,
    ToggleProtection,
    KillSelected,
    Quit,
}

impl MenuAction {
    pub fn all() -> &'static [MenuAction] {
        &[
            MenuAction::NewClaude,
            MenuAction::NewCodex,
            MenuAction::NewGemini,
            MenuAction::NewPi,
            MenuAction::NewGlm,
            MenuAction::NewTerminal,
            MenuAction::DispatchOracle,
            MenuAction::Refresh,
            MenuAction::ToggleProtection,
            MenuAction::KillSelected,
            MenuAction::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "New Claude session",
            MenuAction::NewCodex => "New Codex session",
            MenuAction::NewGemini => "New Gemini session",
            MenuAction::NewPi => "New Pi session (earendil)",
            MenuAction::NewGlm => "New GLM session",
            MenuAction::NewTerminal => "New Terminal (plain shell)",
            MenuAction::DispatchOracle => "Dispatch oracle  →  project + mission",
            MenuAction::Refresh => "Refresh sessions list",
            MenuAction::ToggleProtection => "Toggle protection on selected",
            MenuAction::KillSelected => "Kill selected session",
            MenuAction::Quit => "Quit OmegaOS",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "c",
            MenuAction::NewCodex => "C",
            MenuAction::NewGemini => "g",
            MenuAction::NewPi => "p",
            MenuAction::NewGlm => "G",
            MenuAction::NewTerminal => "t",
            MenuAction::DispatchOracle => "d",
            MenuAction::Refresh => "r",
            MenuAction::ToggleProtection => ".",
            MenuAction::KillSelected => "x",
            MenuAction::Quit => "q",
        }
    }

    pub fn agent(&self) -> Option<omega_core::agents::Agent> {
        match self {
            MenuAction::NewClaude => Some(omega_core::agents::Agent::Claude),
            MenuAction::NewCodex => Some(omega_core::agents::Agent::Codex),
            MenuAction::NewGemini => Some(omega_core::agents::Agent::Gemini),
            MenuAction::NewPi => Some(omega_core::agents::Agent::Pi),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Claude,
    Codex,
    Gemini,
    Pi,
    Glm,
    Aisb,
    Telegram,
}

impl SettingsSection {
    pub fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Claude,
            SettingsSection::Codex,
            SettingsSection::Gemini,
            SettingsSection::Pi,
            SettingsSection::Glm,
            SettingsSection::Aisb,
            SettingsSection::Telegram,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Claude => "Claude (Anthropic)",
            SettingsSection::Codex => "Codex (OpenAI)",
            SettingsSection::Gemini => "Gemini (Google)",
            SettingsSection::Pi => "Pi (earendil)",
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
    RefreshBilling,
}

impl MonitorAction {
    pub fn all() -> &'static [MonitorAction] {
        &[
            MonitorAction::Login,
            MonitorAction::TelegramSetup,
            MonitorAction::TelegramDisconnect,
            MonitorAction::RefreshBilling,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorAction::Login => "Login / re-auth Claude   (opens session with `claude /login`)",
            MonitorAction::TelegramSetup => "Set up Omega Telegram bot   (omega telegram setup …)",
            MonitorAction::TelegramDisconnect => "Disconnect Telegram bot   (removes ~/.omega/telegram.toml)",
            MonitorAction::RefreshBilling => "Refresh billing now   (re-runs usage-monitor.sh)",
        }
    }
    pub fn shortcut(&self) -> &'static str {
        match self {
            MonitorAction::Login => "L",
            MonitorAction::TelegramSetup => "T",
            MonitorAction::TelegramDisconnect => "D",
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
    pub agent_picker_index: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub config: OmegaConfig,
    pub preview_content: String,
    pub preview_scroll: u16,
    /// Auto-follow tail — preview stays glued to the bottom (latest content)
    /// unless the user manually scrolls up.
    pub preview_follow_tail: bool,
    pub session_focus: SessionFocus,
    /// Tracks the last Tab press for double-tap detection.
    pub last_tab_press: Option<std::time::Instant>,
    pub chat_input: String,
    pub current_session: Option<String>,
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
            .or_else(|| std::env::var("TMUX_SESSION").ok());

        Self {
            tab: Tab::Sessions,
            sessions: Vec::new(),
            rows: Vec::new(),
            selected: 0,
            menu_selected: 0,
            monitor_selected: 0,
            settings_selected: 0,
            agent_picker_index: 0,
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            preview_scroll: 0,
            preview_follow_tail: true,
            session_focus: SessionFocus::List,
            last_tab_press: None,
            chat_input: String::new(),
            current_session,
        }
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
        self.chat_input.clear();
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
        }
        self.chat_input.clear();
    }

    /// Legacy alias kept for older call sites.
    pub fn toggle_session_focus(&mut self) {
        self.handle_tab_in_sessions();
    }

    pub fn scroll_preview_down(&mut self, lines: u16) {
        self.preview_scroll = self.preview_scroll.saturating_add(lines);
        // Scrolling down by hand re-engages follow if we're at the bottom
        // (renderer will clamp). Otherwise we stay where the user wants.
        self.preview_follow_tail = false;
    }

    pub fn scroll_preview_up(&mut self, lines: u16) {
        self.preview_scroll = self.preview_scroll.saturating_sub(lines);
        // User scrolled up → break follow, they want to read history
        self.preview_follow_tail = false;
    }

    pub fn scroll_preview_home(&mut self) {
        self.preview_scroll = 0;
        self.preview_follow_tail = false;
    }

    pub fn scroll_preview_end(&mut self) {
        self.preview_scroll = u16::MAX / 2;
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
    }

    pub fn select_settings_prev(&mut self) {
        let count = SettingsSection::all().len();
        self.settings_selected = if self.settings_selected == 0 {
            count - 1
        } else {
            self.settings_selected - 1
        };
    }

    pub fn selected_settings_section(&self) -> SettingsSection {
        SettingsSection::all()[self.settings_selected]
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

        let mgr = omega_core::session::SessionManager::connect().await?;
        match mgr.capture_pane(&name).await {
            Ok(content) => {
                // Keep the full visible buffer so the user can scroll
                self.preview_content = content;
            }
            Err(_) => {
                self.preview_content = String::from("(session has no pane content)");
            }
        }
        // Keep glued to the bottom when in follow mode (tail -f behavior)
        if self.preview_follow_tail {
            self.preview_scroll = u16::MAX / 2;
        }
        Ok(())
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        let mgr = SessionManager::connect().await?;
        let sessions = mgr.list_sessions().await?;
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
                is_protected: true,
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
                is_protected: false,
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
            Tab::Monitor => Tab::Settings,
            Tab::Settings => Tab::Help,
            Tab::Help => Tab::Sessions,
        };
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Help,
            Tab::Menu => Tab::Sessions,
            Tab::Monitor => Tab::Menu,
            Tab::Settings => Tab::Monitor,
            Tab::Help => Tab::Settings,
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
