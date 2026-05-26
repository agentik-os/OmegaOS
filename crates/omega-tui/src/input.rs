use crate::app::{App, InputMode};
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
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            Action::Quit
        }

        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_tab();
            } else {
                app.next_tab();
            }
            Action::None
        }

        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
            Action::None
        }

        KeyCode::Enter => {
            if let Some(entry) = app.selected_session() {
                Action::AttachSession(entry.session.name.clone())
            } else {
                Action::None
            }
        }

        KeyCode::Char('n') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewSession;
            app.status_message = Some("New session name (Enter to confirm, Esc to cancel)".to_string());
            Action::None
        }

        KeyCode::Char('d') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchProject;
            app.status_message = Some("Dispatch — project name? (Enter to continue)".to_string());
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

        KeyCode::Char('?') => {
            app.tab = crate::app::Tab::Help;
            Action::None
        }

        _ => Action::None,
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
