use crate::app::{App, InputMode, MenuAction, MonitorAction, SessionFocus, Tab};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    None,
    Quit,
    AttachSession(String),
    KillSession(String),
    Refresh,
    CreateSession(String),
    CreateSessionWithAgent {
        name: String,
        agent: omega_core::agents::Agent,
        prompt: Option<String>,
    },
    CreateSessionAutoName {
        agent: omega_core::agents::Agent,
        prompt: Option<String>,
    },
    DispatchOracle(String, String),
    SendToSession { session: String, text: String },
    /// Open Claude /login in a fresh session for OAuth re-auth.
    LoginClaude,
    /// Refresh the AISB usage cache (runs the usage-monitor.sh script).
    RefreshBilling,
    /// Open the Telegram bot setup flow.
    TelegramSetup,
    /// Disconnect the currently active Omega Telegram bot.
    TelegramDisconnect,
    /// Rename a session (old, new).
    RenameSession { old: String, new: String },
    /// Commit a freshly-completed Telegram setup wizard.
    TelegramSetupCommit {
        bot_token: String,
        chat_id: i64,
        user_ids: Vec<i64>,
    },
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        // Bracketed paste — arrives as one big string. Route to the active
        // text input (chat or modal) without firing any submit triggers.
        Event::Paste(text) => handle_paste(app, text),
        _ => Action::None,
    }
}

fn handle_paste(app: &mut App, text: String) -> Action {
    // Strip nothing — preserve user's text exactly. \n inside the paste
    // is appended as a literal newline (the input box will wrap/grow).
    match app.input_mode {
        InputMode::Normal => {
            // Sessions tab + chat-focused → append to chat input buffer
            if app.tab == Tab::Sessions
                && matches!(
                    app.session_focus,
                    SessionFocus::Chat | SessionFocus::ChatFullscreen
                )
            {
                app.chat_input.push_str(&text);
                app.status_message = Some(format!("Pasted {} chars", text.len()));
            }
        }
        // Any active input modal: append to its buffer
        _ => {
            app.input_buffer.push_str(&text);
            app.status_message = Some(format!("Pasted {} chars", text.len()));
        }
    }
    Action::None
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    let mode = app.input_mode.clone();
    match mode {
        InputMode::Normal => handle_key_normal(app, key),

        // Direct-agent flow: type name → create + chat focus (NO prompt step)
        InputMode::NewNamedSession(agent_name) => {
            handle_key_input(app, key, move |app, name| {
                let agent = omega_core::agents::Agent::from_name(&agent_name)
                    .unwrap_or(omega_core::agents::Agent::Shell);
                app.input_mode = InputMode::Normal;
                Action::CreateSessionWithAgent { name, agent, prompt: None }
            })
        }

        // Direct-agent flow: step 2 — optional prompt (name may be empty = auto-generate)
        InputMode::NewSessionPromptDirect(name, agent_name) => {
            let agent = omega_core::agents::Agent::from_name(&agent_name)
                .unwrap_or(omega_core::agents::Agent::Shell);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    if name.is_empty() {
                        Action::CreateSessionAutoName { agent, prompt: None }
                    } else {
                        Action::CreateSessionWithAgent { name, agent, prompt: None }
                    }
                }
                KeyCode::Enter => {
                    let value = std::mem::take(&mut app.input_buffer);
                    let prompt = if value.trim().is_empty() { None } else { Some(value) };
                    app.input_mode = InputMode::Normal;
                    if name.is_empty() {
                        Action::CreateSessionAutoName { agent, prompt }
                    } else {
                        Action::CreateSessionWithAgent { name, agent, prompt }
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

        // Step 1: enter session name → move to agent picker
        InputMode::NewSession => handle_key_input(app, key, |app, value| {
            app.agent_picker_index = 0;
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewSessionAgent(value);
            app.status_message = Some(
                "↑/↓ choose agent, Enter to confirm, Esc to skip prompt".to_string(),
            );
            Action::None
        }),

        // Step 2: pick agent via arrows
        InputMode::NewSessionAgent(name) => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.status_message = Some("Cancelled".to_string());
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.agent_picker_next();
                Action::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.agent_picker_prev();
                Action::None
            }
            KeyCode::Enter => {
                let agent = app.selected_agent();
                // Shell agent has no prompt → submit immediately
                if matches!(agent, omega_core::agents::Agent::Shell) {
                    app.input_mode = InputMode::Normal;
                    Action::CreateSessionWithAgent { name, agent, prompt: None }
                } else {
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::NewSessionPrompt(name, agent.name().to_string());
                    app.status_message = Some(
                        "Optional initial prompt (Enter to launch, Esc to skip)".to_string(),
                    );
                    Action::None
                }
            }
            _ => Action::None,
        },

        // Step 3: optional prompt for the agent
        InputMode::NewSessionPrompt(name, agent_name) => {
            let agent = omega_core::agents::Agent::from_name(&agent_name)
                .unwrap_or(omega_core::agents::Agent::Shell);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    Action::CreateSessionWithAgent {
                        name,
                        agent,
                        prompt: None,
                    }
                }
                KeyCode::Enter => {
                    let value = std::mem::take(&mut app.input_buffer);
                    let prompt = if value.trim().is_empty() { None } else { Some(value) };
                    app.input_mode = InputMode::Normal;
                    Action::CreateSessionWithAgent { name, agent, prompt }
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

        InputMode::RenameSession(old_name) => {
            handle_key_input(app, key, move |app, new_name| {
                app.input_mode = InputMode::Normal;
                if new_name.trim() == old_name {
                    app.status_message = Some("(no change)".to_string());
                    return Action::None;
                }
                Action::RenameSession { old: old_name.clone(), new: new_name }
            })
        }

        // ── Telegram setup wizard (3 steps) ─────────────────────────────────
        InputMode::TelegramSetupToken => handle_key_input(app, key, |app, token| {
            app.input_buffer = String::new();
            app.input_mode = InputMode::TelegramSetupChatId(token);
            app.status_message = Some(
                "Step 2/3: Telegram CHAT_ID (numeric, get yours from @userinfobot)".to_string(),
            );
            Action::None
        }),
        InputMode::TelegramSetupChatId(token) => {
            handle_key_input(app, key, move |app, chat_id_str| {
                let chat_id: i64 = match chat_id_str.trim().parse() {
                    Ok(n) => n,
                    Err(_) => {
                        app.status_message = Some(
                            format!("Invalid chat_id '{}' — must be numeric", chat_id_str),
                        );
                        app.input_mode = InputMode::Normal;
                        return Action::None;
                    }
                };
                app.input_buffer = String::new();
                app.input_mode =
                    InputMode::TelegramSetupUserId(token.clone(), chat_id.to_string());
                app.status_message = Some(
                    "Step 3/3: ALLOWED user_id (or Esc to skip — chat_id-only filter)".to_string(),
                );
                Action::None
            })
        }
        InputMode::TelegramSetupUserId(token, chat_id_str) => {
            let token = token.clone();
            let chat_id: i64 = chat_id_str.parse().unwrap_or(0);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    Action::TelegramSetupCommit {
                        bot_token: token,
                        chat_id,
                        user_ids: Vec::new(),
                    }
                }
                KeyCode::Enter => {
                    let value = std::mem::take(&mut app.input_buffer);
                    let user_ids = value
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i64>().ok())
                        .collect();
                    app.input_mode = InputMode::Normal;
                    Action::TelegramSetupCommit {
                        bot_token: token,
                        chat_id,
                        user_ids,
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
    }
}

fn handle_key_normal(app: &mut App, key: KeyEvent) -> Action {
    // When in Sessions tab and chat-focused (split or fullscreen), route keys
    // to the chat input handler first.
    if app.tab == Tab::Sessions
        && matches!(
            app.session_focus,
            SessionFocus::Chat | SessionFocus::ChatFullscreen
        )
    {
        return handle_key_chat(app, key);
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
            Action::Quit
        }

        // Tab switching: ←/→ for tabs, Tab inside Sessions toggles focus list↔chat
        KeyCode::Left => {
            app.prev_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::Right => {
            app.next_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::BackTab => {
            app.prev_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_tab();
            } else if app.tab == Tab::Sessions {
                app.handle_tab_in_sessions();
                app.status_message = Some(match app.session_focus {
                    SessionFocus::List => "Focus: session list (Tab → chat, Tab-Tab → fullscreen)".to_string(),
                    SessionFocus::Chat => "Focus: chat (Tab to list, Tab-Tab → fullscreen, Enter to send)".to_string(),
                    SessionFocus::ChatFullscreen => "Focus: chat FULLSCREEN (Tab-Tab → back to list)".to_string(),
                });
            } else if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) {
                // 2-column tabs: Tab toggles list↔detail, Tab-Tab → fullscreen
                app.handle_tab_in_2col();
                app.status_message = Some(if app.detail_fullscreen {
                    "Focus: detail FULLSCREEN (Tab → list, Tab-Tab → exit)".to_string()
                } else if app.detail_focused {
                    "Focus: detail panel (↑/↓ scroll, Tab → list, Tab-Tab → fullscreen)".to_string()
                } else {
                    "Focus: section list (Tab → detail, Tab-Tab → detail fullscreen)".to_string()
                });
            } else {
                app.next_tab();
            }
            Action::None
        }

        // Scroll: depends on the active tab + focus
        KeyCode::PageDown => {
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) {
                app.scroll_detail_down(10);
            } else {
                app.scroll_preview_down(10);
            }
            Action::None
        }
        KeyCode::PageUp => {
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) {
                app.scroll_detail_up(10);
            } else {
                app.scroll_preview_up(10);
            }
            Action::None
        }
        KeyCode::Home => {
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) {
                app.detail_scroll = 0;
            } else {
                app.scroll_preview_home();
            }
            Action::None
        }
        KeyCode::End => {
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) {
                app.detail_scroll = u16::MAX / 2;
            } else {
                app.scroll_preview_end();
            }
            Action::None
        }

        // Navigation: ↑/↓ AND j/k — context-aware (sessions vs menu)
        KeyCode::Down | KeyCode::Char('j') => {
            // In 2-col tabs with detail focused: ↓ scrolls the detail
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) && app.detail_focused {
                app.scroll_detail_down(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_next(),
                Tab::Menu => app.select_menu_next(),
                Tab::Monitor => app.select_monitor_next(),
                Tab::Settings => app.select_settings_next(),
                Tab::Info => {
                    if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                        app.select_info_agent_next();
                    } else {
                        app.select_info_next();
                    }
                }
                Tab::Help => {}
            }
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if matches!(app.tab, Tab::Settings | Tab::Info | Tab::Monitor) && app.detail_focused {
                app.scroll_detail_up(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_prev(),
                Tab::Menu => app.select_menu_prev(),
                Tab::Monitor => app.select_monitor_prev(),
                Tab::Settings => app.select_settings_prev(),
                Tab::Info => {
                    if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                        app.select_info_agent_prev();
                    } else {
                        app.select_info_prev();
                    }
                }
                Tab::Help => {}
            }
            Action::None
        }

        // Left/Right inside Info navigates between sub-sections (independent of agent sub-cursor)
        // We use a separate explicit handler via PgUp/PgDn — but since arrow keys are taken
        // for tabs, users can use Home/End or [/] to jump between sub-sections:
        KeyCode::Char('[') if app.tab == Tab::Info => {
            app.select_info_prev();
            Action::None
        }
        KeyCode::Char(']') if app.tab == Tab::Info => {
            app.select_info_next();
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
            Tab::Monitor => execute_monitor_action(app.selected_monitor_action()),
            Tab::Settings | Tab::Info | Tab::Help => Action::None,
        },

        // Monitor tab letter shortcuts
        KeyCode::Char('L') if app.tab == Tab::Monitor => Action::LoginClaude,
        KeyCode::Char('T') if app.tab == Tab::Monitor => Action::TelegramSetup,
        KeyCode::Char('D') if app.tab == Tab::Monitor => Action::TelegramDisconnect,
        KeyCode::Char('B') if app.tab == Tab::Monitor => Action::RefreshBilling,

        // Shortcut keys (work in any tab) — direct agent launchers
        KeyCode::Char('c') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("claude".to_string());
            app.status_message = Some("Session name for new Claude (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('C') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("codex".to_string());
            app.status_message = Some("Session name for new Codex (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('g') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("gemini".to_string());
            app.status_message = Some("Session name for new Gemini (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('p') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("pi".to_string());
            app.status_message = Some("Session name for new Pi (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('G') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("glm".to_string());
            app.status_message = Some("Session name for new GLM (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('t') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("shell".to_string());
            app.status_message = Some("Session name for new Terminal (Enter, Esc to cancel)".to_string());
            Action::None
        }

        KeyCode::Char('d') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchProject;
            app.status_message = Some("Project name (Enter to continue)".to_string());
            Action::None
        }

        // Kill — both lowercase x and uppercase X work
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if let Some(entry) = app.selected_session() {
                if !entry.is_protected {
                    Action::KillSession(entry.session.name.clone())
                } else {
                    app.status_message = Some("Session is protected (press . to unlock)".to_string());
                    Action::None
                }
            } else {
                Action::None
            }
        }

        // Rename selected session
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(entry) = app.selected_session() {
                let old = entry.session.name.clone();
                app.input_buffer = old.clone();
                app.input_mode = InputMode::RenameSession(old.clone());
                app.status_message = Some(format!("Rename '{}' (Enter to confirm, Esc to cancel)", old));
            } else {
                app.status_message = Some("No session selected".to_string());
            }
            Action::None
        }

        // Refresh
        KeyCode::F(5) => Action::Refresh,

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

        KeyCode::F(1) => {
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

fn execute_monitor_action(action: MonitorAction) -> Action {
    match action {
        MonitorAction::Login => Action::LoginClaude,
        MonitorAction::TelegramSetup => Action::TelegramSetup,
        MonitorAction::TelegramDisconnect => Action::TelegramDisconnect,
        MonitorAction::RefreshBilling => Action::RefreshBilling,
    }
}

/// Chat-input mode — typing flows into the selected session's pane via SDK.
fn handle_key_chat(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        // Tab cycles focus (single = toggle, double = fullscreen)
        KeyCode::Tab => {
            app.handle_tab_in_sessions();
            app.status_message = Some(match app.session_focus {
                SessionFocus::List => "Focus: session list".to_string(),
                SessionFocus::Chat => "Focus: chat (Tab-Tab to fullscreen)".to_string(),
                SessionFocus::ChatFullscreen => "Focus: chat FULLSCREEN (Tab-Tab to exit)".to_string(),
            });
            Action::None
        }
        // Esc always returns to list
        KeyCode::Esc => {
            app.session_focus = SessionFocus::List;
            app.chat_input.clear();
            app.status_message = Some("Focus: session list".to_string());
            Action::None
        }
        // Submit: send buffer + Enter to the rmux pane
        KeyCode::Enter => {
            if let Some(entry) = app.selected_session() {
                let session = entry.session.name.clone();
                let text = std::mem::take(&mut app.chat_input);
                Action::SendToSession { session, text }
            } else {
                Action::None
            }
        }
        KeyCode::Backspace => {
            app.chat_input.pop();
            Action::None
        }
        // Scroll preview while in chat
        KeyCode::PageDown => {
            app.scroll_preview_down(10);
            Action::None
        }
        KeyCode::PageUp => {
            app.scroll_preview_up(10);
            Action::None
        }
        KeyCode::Char(c) => {
            // Ctrl+C inside chat → back to list (don't quit, that's surprising)
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                app.session_focus = SessionFocus::List;
                app.chat_input.clear();
                return Action::None;
            }
            app.chat_input.push(c);
            Action::None
        }
        _ => Action::None,
    }
}

fn execute_menu_action(app: &mut App, action: MenuAction) -> Action {
    // Per-agent direct launchers — ALL go straight to session creation with
    // chat focus. No "initial prompt" step. The user talks via the chat input
    // box once the session is up.
    if let Some(agent) = action.agent() {
        // Guard: block launch if the agent CLI is not installed.
        // Shell is always available, so it short-circuits.
        if !matches!(agent, omega_core::agents::Agent::Shell) && !agent.is_available() {
            let install_hint = agent
                .install_command()
                .map(|c| format!(" — install: {}", c))
                .unwrap_or_default();
            app.status_message = Some(format!(
                "{} not installed. Open Settings → Install agents{}",
                agent.display_name(),
                install_hint
            ));
            return Action::None;
        }

        if app.config.auto_naming {
            // Fire-and-attach: agent + auto-name + no prompt, then chat focus
            return Action::CreateSessionAutoName { agent, prompt: None };
        }
        // Auto-naming disabled → only ask for name (still no prompt step)
        app.input_buffer = String::new();
        app.input_mode = InputMode::NewNamedSession(agent.name().to_string());
        app.status_message = Some(format!(
            "Session name for new {} (Enter to launch, Esc to cancel)",
            agent.display_name()
        ));
        return Action::None;
    }

    match action {
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
        MenuAction::Quit => {
            app.should_quit = true;
            Action::Quit
        }
        // Per-agent variants handled above
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
