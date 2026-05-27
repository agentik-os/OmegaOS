use omega_core::config::OmegaConfig;
use omega_core::progress::ProgressInfo;
use omega_core::session::{OmegaSession, SessionManager, SessionRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Sessions,
    Menu,
    Settings,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    // Direct-agent flow: agent already chosen via menu
    NewNamedSession(String),               // agent_name — typing session name
    NewSessionPromptDirect(String, String), // (session_name, agent_name) — optional prompt
    // Legacy "n" key flow: 3-step picker
    NewSession,
    NewSessionAgent(String),
    NewSessionPrompt(String, String),
    DispatchProject,
    DispatchMission(String),
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
pub struct SessionEntry {
    pub session: OmegaSession,
    pub progress: Option<ProgressInfo>,
    pub is_current: bool,
    pub is_protected: bool,
    pub tree_prefix: String,
}

/// Which side of the Sessions tab has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFocus {
    List,
    Chat,
}

pub struct App {
    pub tab: Tab,
    pub sessions: Vec<SessionEntry>,
    pub selected: usize,
    pub menu_selected: usize,
    pub agent_picker_index: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub config: OmegaConfig,
    pub preview_content: String,
    pub preview_scroll: u16,
    pub session_focus: SessionFocus,
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
            selected: 0,
            menu_selected: 0,
            agent_picker_index: 0,
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            preview_scroll: 0,
            session_focus: SessionFocus::List,
            chat_input: String::new(),
            current_session,
        }
    }

    pub fn toggle_session_focus(&mut self) {
        self.session_focus = match self.session_focus {
            SessionFocus::List => SessionFocus::Chat,
            SessionFocus::Chat => SessionFocus::List,
        };
        self.chat_input.clear();
    }

    pub fn scroll_preview_down(&mut self, lines: u16) {
        self.preview_scroll = self.preview_scroll.saturating_add(lines);
    }

    pub fn scroll_preview_up(&mut self, lines: u16) {
        self.preview_scroll = self.preview_scroll.saturating_sub(lines);
    }

    pub fn scroll_preview_home(&mut self) {
        self.preview_scroll = 0;
    }

    pub fn scroll_preview_end(&mut self) {
        // Set a large value — the renderer clamps it
        self.preview_scroll = u16::MAX / 2;
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
        Ok(())
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        let mgr = SessionManager::connect().await?;
        let sessions = mgr.list_sessions().await?;
        let all_progress = ProgressInfo::read_all(&self.config.state_dir);

        self.sessions.clear();

        // Pin Master AISB at the top with a special marker
        if let Some(master) = sessions
            .iter()
            .find(|s| omega_core::aisb::is_master(&s.name))
        {
            self.sessions.push(SessionEntry {
                session: master.clone(),
                progress: None,
                is_current: false,
                is_protected: true, // master is always protected from accidental kill
                tree_prefix: "★ ".to_string(),
            });
        }

        let mut last_project: Option<String> = None;
        let mut group: Vec<(usize, &OmegaSession)> = Vec::new();

        for (idx, session) in sessions.iter().enumerate() {
            // Skip Master AISB — already rendered at top
            if omega_core::aisb::is_master(&session.name) {
                continue;
            }
            let current_project = session.project.clone();

            if current_project != last_project && !group.is_empty() {
                self.flush_group(&group, &all_progress);
                group.clear();
            }

            group.push((idx, session));
            last_project = current_project;
        }
        if !group.is_empty() {
            self.flush_group(&group, &all_progress);
        }

        if self.selected >= self.sessions.len() && !self.sessions.is_empty() {
            self.selected = self.sessions.len() - 1;
        }

        Ok(())
    }

    fn flush_group(
        &mut self,
        group: &[(usize, &OmegaSession)],
        all_progress: &[ProgressInfo],
    ) {
        let has_oracle = group.iter().any(|(_, s)| s.role == SessionRole::Oracle);
        let worker_count = group.iter().filter(|(_, s)| s.role == SessionRole::Worker).count();
        let show_tree = has_oracle && worker_count > 0;

        for (i, (_, session)) in group.iter().enumerate() {
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

            self.sessions.push(SessionEntry {
                session: (*session).clone(),
                progress,
                is_current: false,
                is_protected: false,
                tree_prefix,
            });
        }
    }

    pub fn selected_session(&self) -> Option<&SessionEntry> {
        self.sessions.get(self.selected)
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Menu,
            Tab::Menu => Tab::Settings,
            Tab::Settings => Tab::Help,
            Tab::Help => Tab::Sessions,
        };
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Help,
            Tab::Menu => Tab::Sessions,
            Tab::Settings => Tab::Menu,
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
