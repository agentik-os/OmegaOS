use crate::app::App;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    None,
    Quit,
    AttachSession(String),
    KillSession(String),
    Refresh,
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        _ => Action::None,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
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

        KeyCode::Char('x') => {
            if let Some(entry) = app.selected_session() {
                if !entry.is_protected {
                    Action::KillSession(entry.session.name.clone())
                } else {
                    app.status_message = Some("Session is protected".to_string());
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

        _ => Action::None,
    }
}
