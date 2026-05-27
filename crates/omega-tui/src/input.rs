use crate::app::{App, InputMode, MenuAction, Tab};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    None,
    Quit,
    AttachSession(String),
    KillSession(String),
    Refresh,
    CreateSession(String),
    DispatchOracle(String, String),
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        _ => Action::None,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    match app.input_mode {
        InputMode::Normal => handle_key_normal(app, key),
        InputMode::NewSession => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            Action::CreateSession(value)
        }),
        InputMode::DispatchProject => handle_key_input(app, key, |app, value| {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchMission(value);
            Action::None
        }),
        InputMode::DispatchMission(_) => {
            let project = match &app.input_mode {
                InputMode::DispatchMission(p) => p.clone(),
                _ => return Action::None,
            };
            handle_key_input(app, key, move |app, mission| {
                app.input_mode = InputMode::Normal;
                Action::DispatchOracle(project.clone(), mission)
            })
        }
    }
}

fn handle_key_normal(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
            Action::Quit
        }

        // Tab switching: ←/→ AND Tab/Shift+Tab
        KeyCode::Left | KeyCode::BackTab => {
            app.prev_tab();
            Action::None
        }
        KeyCode::Right => {
            app.next_tab();
            Action::None
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_tab();
            } else {
                app.next_tab();
            }
            Action::None
        }

        // Navigation: ↑/↓ AND j/k — context-aware (sessions vs menu)
        KeyCode::Down | KeyCode::Char('j') => {
            match app.tab {
                Tab::Sessions => app.select_next(),
                Tab::Menu => app.select_menu_next(),
                Tab::Help => {}
            }
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            match app.tab {
                Tab::Sessions => app.select_prev(),
                Tab::Menu => app.select_menu_prev(),
                Tab::Help => {}
            }
            Action::None
        }

        // Enter: context-aware
        KeyCode::Enter => match app.tab {
            Tab::Sessions => {
                if let Some(entry) = app.selected_session() {
                    Action::AttachSession(entry.session.name.clone())
                } else {
                    Action::None
                }
            }
            Tab::Menu => execute_menu_action(app, app.selected_menu_action()),
            Tab::Help => Action::None,
        },

        // Shortcut keys (work in any tab)
        KeyCode::Char('n') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewSession;
            app.status_message = Some("Session name (Enter to confirm, Esc to cancel)".to_string());
            Action::None
        }

        KeyCode::Char('d') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchProject;
            app.status_message = Some("Project name (Enter to continue)".to_string());
            Action::None
        }

        KeyCode::Char('x') => {
            if let Some(entry) = app.selected_session() {
                if !entry.is_protected {
                    Action::KillSession(entry.session.name.clone())
                } else {
                    app.status_message = Some("Session is protected (press . to unprotect)".to_string());
                    Action::None
                }
            } else {
                Action::None
            }
        }

        KeyCode::Char('r') => Action::Refresh,

        KeyCode::Char('.') => {
            if let Some(entry) = app.sessions.get_mut(app.selected) {
                entry.is_protected = !entry.is_protected;
                let state = if entry.is_protected {
                    "protected"
                } else {
                    "unprotected"
                };
                app.status_message =
                    Some(format!("{} is now {}", entry.session.name, state));
            }
            Action::None
        }

        KeyCode::Char('?') | KeyCode::F(1) => {
            app.tab = Tab::Help;
            Action::None
        }

        KeyCode::Esc => {
            // Esc → switch to Sessions tab from anywhere, or quit if already there
            if app.tab == Tab::Sessions {
                app.should_quit = true;
                Action::Quit
            } else {
                app.tab = Tab::Sessions;
                Action::None
            }
        }

        _ => Action::None,
    }
}

fn execute_menu_action(app: &mut App, action: MenuAction) -> Action {
    match action {
        MenuAction::NewSession => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewSession;
            app.status_message = Some("Session name (Enter to confirm)".to_string());
            Action::None
        }
        MenuAction::DispatchOracle => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchProject;
            app.status_message = Some("Project name (Enter to continue)".to_string());
            Action::None
        }
        MenuAction::Refresh => Action::Refresh,
        MenuAction::ToggleProtection => {
            if let Some(entry) = app.sessions.get_mut(app.selected) {
                entry.is_protected = !entry.is_protected;
                app.status_message = Some(format!(
                    "{} {}",
                    entry.session.name,
                    if entry.is_protected { "protected" } else { "unprotected" }
                ));
            }
            Action::None
        }
        MenuAction::KillSelected => {
            if let Some(entry) = app.selected_session() {
                if entry.is_protected {
                    app.status_message = Some("Selected session is protected".to_string());
                    Action::None
                } else {
                    Action::KillSession(entry.session.name.clone())
                }
            } else {
                app.status_message = Some("No session selected".to_string());
                Action::None
            }
        }
        MenuAction::Help => {
            app.tab = Tab::Help;
            Action::None
        }
        MenuAction::Quit => {
            app.should_quit = true;
            Action::Quit
        }
    }
}

fn handle_key_input<F>(app: &mut App, key: KeyEvent, on_submit: F) -> Action
where
    F: FnOnce(&mut App, String) -> Action,
{
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer = String::new();
            app.status_message = Some("Cancelled".to_string());
            Action::None
        }
        KeyCode::Enter => {
            let value = std::mem::take(&mut app.input_buffer);
            if value.trim().is_empty() {
                app.input_mode = InputMode::Normal;
                app.status_message = Some("Cancelled (empty input)".to_string());
                Action::None
            } else {
                on_submit(app, value)
            }
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
            Action::None
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
            Action::None
        }
        _ => Action::None,
    }
}
