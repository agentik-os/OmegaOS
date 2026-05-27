use omega_core::config::OmegaConfig;
use omega_core::progress::ProgressInfo;
use omega_core::session::{OmegaSession, SessionManager, SessionRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Sessions,
    Menu,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    NewSession,
    DispatchProject,
    DispatchMission(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewSession,
    DispatchOracle,
    Refresh,
    ToggleProtection,
    KillSelected,
    Help,
    Quit,
}

impl MenuAction {
    pub fn all() -> &'static [MenuAction] {
        &[
            MenuAction::NewSession,
            MenuAction::DispatchOracle,
            MenuAction::Refresh,
            MenuAction::ToggleProtection,
            MenuAction::KillSelected,
            MenuAction::Help,
            MenuAction::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuAction::NewSession => "New session",
            MenuAction::DispatchOracle => "Dispatch oracle",
            MenuAction::Refresh => "Refresh sessions",
            MenuAction::ToggleProtection => "Toggle protection",
            MenuAction::KillSelected => "Kill selected session",
            MenuAction::Help => "Show help",
            MenuAction::Quit => "Quit OmegaOS",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            MenuAction::NewSession => "n",
            MenuAction::DispatchOracle => "d",
            MenuAction::Refresh => "r",
            MenuAction::ToggleProtection => ".",
            MenuAction::KillSelected => "x",
            MenuAction::Help => "?",
            MenuAction::Quit => "q",
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

pub struct App {
    pub tab: Tab,
    pub sessions: Vec<SessionEntry>,
    pub selected: usize,
    pub menu_selected: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub config: OmegaConfig,
    pub preview_content: String,
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
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            current_session,
        }
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
                let lines: Vec<&str> = content.lines().collect();
                let start = lines.len().saturating_sub(40);
                self.preview_content = lines[start..].join("\n");
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

        let mut last_project: Option<String> = None;
        let mut group: Vec<(usize, &OmegaSession)> = Vec::new();

        for (idx, session) in sessions.iter().enumerate() {
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
            Tab::Menu => Tab::Help,
            Tab::Help => Tab::Sessions,
        };
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Sessions => Tab::Help,
            Tab::Menu => Tab::Sessions,
            Tab::Help => Tab::Menu,
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
