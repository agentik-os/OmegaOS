use crate::app::{App, InputMode, MenuAction, MonitorAction, SessionFocus, Tab};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub enum Action {
    None,
    Quit,
    AttachSession(String),
    KillSession(String),
    /// Kill all sessions except current + protected + infrastructure.
    KillAllSessions,
    /// Nuclear cleanup: kill all + prune stale state + clear scratch + drop cache.
    NuclearCleanup,
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
    /// New-project wizard result: spawn a Claude session that runs
    /// `/omega-new-project <stack> <category> <name>` (provision + scaffold +
    /// vision/PRD/planner). `category` is "works" | "client", `stack` a stack id.
    CreateProject {
        name: String,
        category: String,
        stack: String,
        launch_prompt: Option<String>,
        launch_docs: Option<String>,
    },
    /// Start the provisioning-keys wizard (Monitor tab, Telegram-style).
    ProvisioningSetup,
    /// Commit the provisioning-keys wizard — (env_key, value) pairs; blanks are
    /// ignored by the writer (existing values preserved).
    ProvisioningCommit { values: Vec<(String, String)> },
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
    /// Run a shell command (Settings Install/Uninstall actions).
    RunShellCommand { label: String, command: String },
    /// Begin editing a settings text field (opens input modal pre-filled).
    EditSettingsField {
        config_key: String,
        current: String,
        masked: bool,
    },
    /// Toggle a boolean settings field.
    ToggleSettingsBool { config_key: String },
    /// Commit an edited settings text field (saves to providers.toml).
    CommitSettingsEdit { config_key: String, value: String },
    /// Ctrl+L — force a full terminal clear + redraw.
    ForceRedraw,
    /// Menu → Restart OmegaOS: tear down the terminal and re-exec the
    /// `omega menu` binary in place (picks up a freshly-built binary).
    Restart,
    /// Projects tab: open the selected project in a terminal — attach to its
    /// Oracle session if one is alive, otherwise spawn a shell in its dir.
    OpenProject { name: String, path: String, oracle_session: Option<String> },
    /// Projects tab: dispatch `omega planner` for the selected project.
    RunPlannerForProject { name: String, path: String },
    /// Real-time keystroke forwarding to a rmux session (preview interactive
    /// mode). One key per Action — printable chars, special keys, Ctrl-combos
    /// all route through this so plan-mode / OAuth / choice menus work.
    ForwardCharToSession { session: String, ch: char },
    ForwardKeyToSession { session: String, key: &'static str },
    /// Multi-char paste forwarded as a single literal write to the session
    /// (used by handle_paste so the entire bracketed-paste block lands as
    /// one PTY write rather than N individual keystrokes).
    SendTextRawToSession { session: String, text: String },
    /// Insert a literal newline into the agent's input box WITHOUT submitting
    /// (Shift+Enter / Alt+Enter). Empirically, Claude Code treats a trailing
    /// backslash followed by Enter as a newline-insert, so the main loop emits
    /// a `\` text write then an Enter key.
    InsertNewlineToSession { session: String },
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Paste(text) => handle_paste(app, text),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => Action::ForceRedraw,
        _ => Action::None,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Action {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            scroll_active_panel_at(app, 3, true, mouse.column);
            Action::None
        }
        MouseEventKind::ScrollUp => {
            scroll_active_panel_at(app, 3, false, mouse.column);
            Action::None
        }
        // Click in a panel = focus it (left = list, right = preview)
        MouseEventKind::Down(_) => {
            if app.tab == Tab::Sessions {
                // Heuristic: list is on the left ~25-30% of screen width.
                // If click is in the right portion, focus the preview/chat.
                // We can't read terminal width here directly, but column 30+ is
                // almost always the right panel for any reasonable terminal.
                if mouse.column >= 30 {
                    if matches!(app.session_focus, SessionFocus::List) {
                        // Use the canonical focus path (follow_tail = true) so a
                        // mouse click behaves like the keyboard Enter — entering
                        // chat shows the latest output instead of freezing the view.
                        app.enter_chat_focus();
                    }
                } else {
                    app.session_focus = SessionFocus::List;
                }
            }
            Action::None
        }
        _ => Action::None,
    }
}

/// Position-aware scroll: in Sessions tab, scrolling over the right panel
/// (column >= 30) scrolls the preview regardless of focus state.
fn scroll_active_panel_at(app: &mut App, lines: u16, down: bool, column: u16) {
    if app.tab == Tab::Sessions && column >= 30 {
        if down { app.scroll_preview_down(lines); }
        else { app.scroll_preview_up(lines); }
        return;
    }
    scroll_active_panel(app, lines, down);
}

fn scroll_active_panel(app: &mut App, lines: u16, down: bool) {
    match app.tab {
        Tab::Sessions => {
            if matches!(app.session_focus, SessionFocus::Chat | SessionFocus::ChatFullscreen) {
                if down { app.scroll_preview_down(lines); }
                else { app.scroll_preview_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.select_next(); } else { app.select_prev(); }
                }
            }
        }
        Tab::Menu => {
            let max = MenuAction::all().len().saturating_sub(1);
            for _ in 0..lines {
                if down {
                    app.menu_selected = (app.menu_selected + 1).min(max);
                } else {
                    app.menu_selected = app.menu_selected.saturating_sub(1);
                }
            }
        }
        Tab::Monitor => {
            if down { app.scroll_detail_down(lines); }
            else { app.scroll_detail_up(lines); }
        }
        Tab::Projects => {
            if app.detail_focused {
                if down { app.scroll_detail_down(lines); }
                else { app.scroll_detail_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.select_project_next(); } else { app.select_project_prev(); }
                }
            }
        }
        Tab::Settings => {
            if app.detail_focused {
                if down { app.scroll_detail_down(lines); }
                else { app.scroll_detail_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.select_settings_next(); } else { app.select_settings_prev(); }
                }
            }
        }
        Tab::Agentic => {
            if app.detail_focused {
                if down { app.scroll_detail_down(lines); }
                else { app.scroll_detail_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.select_info_next(); } else { app.select_info_prev(); }
                }
            }
        }
        Tab::Help => {
            if down { app.scroll_detail_down(lines); }
            else { app.scroll_detail_up(lines); }
        }
    }
}

fn handle_paste(app: &mut App, text: String) -> Action {
    // Sessions tab + chat-focused → forward the paste DIRECTLY to the rmux
    // session (no chat_input buffer). Dispatched as ForwardMsg::Paste →
    // send_paste_raw, which wraps the block in bracketed-paste markers and
    // sends NO trailing Enter, so embedded newlines don't submit each line
    // as a separate command and multi-line / special chars survive intact.
    if app.input_mode == InputMode::Normal
        && app.tab == Tab::Sessions
        && matches!(
            app.session_focus,
            SessionFocus::Chat | SessionFocus::ChatFullscreen
        )
    {
        if let Some(entry) = app.selected_session() {
            let session = entry.session.name.clone();
            app.status_message = Some(format!("Pasted {} chars → {}", text.len(), session));
            return Action::SendTextRawToSession { session, text };
        }
    }

    // Active input modal: keep buffer semantics
    app.input_buffer.push_str(&text);
    app.status_message = Some(format!("Pasted {} chars", text.len()));
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

        // New-project wizard — step 1: name → category picker
        InputMode::NewProjectName => handle_key_input(app, key, |app, value| {
            let name = value.trim().to_lowercase().replace(' ', "-");
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewProjectCategory(name, 0);
            app.status_message =
                Some("↑/↓ category — Enter to continue, Esc to cancel".to_string());
            Action::None
        }),

        // New-project wizard — step 2: category picker (works/client)
        InputMode::NewProjectCategory(name, sel) => {
            let count = crate::app::NEW_PROJECT_CATEGORIES.len();
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::NewProjectCategory(name, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::NewProjectCategory(name, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let category = crate::app::NEW_PROJECT_CATEGORIES[sel].0.to_string();
                    if category == "customer" {
                        // Customer → choose/create a credential group (separate accounts).
                        app.new_project_cred_group = None;
                        app.input_buffer = String::new();
                        let groups = omega_core::provisioning::list_groups().join(", ");
                        app.input_mode = InputMode::NewProjectCredGroup(name, category);
                        app.status_message = Some(format!(
                            "Credential group — existing: {}  —  type one to reuse OR a new customer name (Enter, Esc=default)",
                            groups
                        ));
                    } else {
                        app.new_project_cred_group = None; // works → shared/default creds
                        app.input_mode = InputMode::NewProjectStack(name, category, 0);
                        app.status_message =
                            Some("↑/↓ stack — Enter to continue, Esc to cancel".to_string());
                    }
                    Action::None
                }
                _ => Action::None,
            }
        }

        // New-project wizard — step 2b (client only): pick/create a credential group.
        InputMode::NewProjectCredGroup(name, category) => match key.code {
            KeyCode::Esc => {
                app.new_project_cred_group = Some("default".to_string());
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectStack(name, category, 0);
                app.status_message = Some("Credential group: default (shared)".to_string());
                Action::None
            }
            KeyCode::Enter => {
                let typed = app.input_buffer.trim().to_string();
                let group = if typed.is_empty() { "default".to_string() } else { typed };
                let existing = group == "default"
                    || omega_core::provisioning::list_groups()
                        .iter()
                        .any(|g| g == &omega_core::provisioning::sanitize_group(&group));
                app.new_project_cred_group = Some(group.clone());
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectStack(name, category, 0);
                app.status_message = Some(format!(
                    "Credential group: {} ({})",
                    group,
                    if existing {
                        "reuse"
                    } else {
                        "NEW — fill it later with: omega provision set"
                    }
                ));
                Action::None
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
        },

        // New-project wizard — step 3: stack picker → spawn the provisioning session
        InputMode::NewProjectStack(name, category, sel) => {
            let count = crate::app::NEW_PROJECT_STACKS.len();
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::NewProjectStack(name, category, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::NewProjectStack(name, category, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let stack = crate::app::NEW_PROJECT_STACKS[sel].0.to_string();
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::NewProjectLaunchPrompt(name, category, stack);
                    app.status_message = Some(
                        "Optional kickoff — describe the idea/requirements (Enter to continue, Esc to skip)".to_string(),
                    );
                    Action::None
                }
                _ => Action::None,
            }
        }

        // New-project wizard — step 4 (optional): kickoff prompt.
        InputMode::NewProjectLaunchPrompt(name, category, stack) => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: None,
                    launch_docs: None,
                }
            }
            KeyCode::Enter => {
                let kickoff = if app.input_buffer.trim().is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut app.input_buffer))
                };
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectLaunchDocs(name, category, stack, kickoff);
                app.status_message = Some(
                    "Optional docs — comma-separated paths to seed the project (Enter to launch, Esc to skip)".to_string(),
                );
                Action::None
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
        },

        // New-project wizard — step 5 (optional): doc paths, then spawn.
        InputMode::NewProjectLaunchDocs(name, category, stack, kickoff) => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: kickoff,
                    launch_docs: None,
                }
            }
            KeyCode::Enter => {
                let docs = if app.input_buffer.trim().is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut app.input_buffer))
                };
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: kickoff,
                    launch_docs: docs,
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
        },

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

        InputMode::SessionFilter => handle_key_input(app, key, |app, query| {
            app.input_mode = InputMode::Normal;
            let q = query.trim();
            app.session_filter = if q.is_empty() { None } else { Some(q.to_string()) };
            app.selected = 0; // the filtered list changes length — reset selection.
            app.status_message = Some(match &app.session_filter {
                Some(f) => format!("Filter: '{}' (press / to change)", f),
                None => "Filter cleared".to_string(),
            });
            Action::Refresh
        }),

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
        InputMode::EditSettingsField { config_key, .. } => {
            let cfg_key = config_key.clone();
            handle_key_input(app, key, move |app, value| {
                app.input_mode = InputMode::Normal;
                Action::CommitSettingsEdit {
                    config_key: cfg_key,
                    value,
                }
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

        // ── Provisioning-keys wizard (N skippable steps) ────────────────────
        InputMode::ProvisioningSetup { step, collected } => match key.code {
            KeyCode::Esc => provisioning_advance(app, collected, String::new(), step),
            KeyCode::Enter => {
                let value = std::mem::take(&mut app.input_buffer);
                provisioning_advance(app, collected, value, step)
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
        },
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
        // Ctrl+L — force full terminal redraw (fixes corrupted view)
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ForceRedraw
        }
        // Ctrl+R — hot-reload OmegaOS in place (re-exec the updated binary).
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::Restart
        }

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
            } else if matches!(app.tab, Tab::Settings | Tab::Agentic | Tab::Monitor | Tab::Projects) {
                // 2-column tabs: Tab toggles list↔detail, Tab-Tab → fullscreen
                app.handle_tab_in_2col();
                // When entering detail on Settings, snap cursor to first actionable
                if app.tab == Tab::Settings && app.detail_focused {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(
                        section,
                        &providers,
                        &app.config,
                    );
                    if let Some(first) = fields.iter().position(|f| f.is_actionable()) {
                        app.settings_field_selected = first;
                    }
                }
                app.status_message = Some(if app.detail_fullscreen {
                    "Focus: detail FULLSCREEN (Tab → list, Tab-Tab → exit)".to_string()
                } else if app.detail_focused {
                    "Focus: detail (↑/↓ navigate, Enter activate, Tab → list)".to_string()
                } else {
                    "Focus: section list (Tab → detail, Tab-Tab → fullscreen)".to_string()
                });
            } else {
                app.next_tab();
            }
            Action::None
        }

        // Scroll: depends on the active tab + focus. PageUp/PageDown super-
        // scroll a FULL page of the preview (Termius swipe rips through fast).
        KeyCode::PageDown => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic | Tab::Monitor | Tab::Projects) {
                app.scroll_detail_down(10);
            } else {
                app.scroll_preview_down(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::PageUp => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic | Tab::Monitor | Tab::Projects) {
                app.scroll_detail_up(10);
            } else {
                app.scroll_preview_up(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::Home => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic | Tab::Monitor | Tab::Projects) {
                app.detail_scroll = 0;
            } else {
                app.scroll_preview_home();
            }
            Action::None
        }
        KeyCode::End => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic | Tab::Monitor | Tab::Projects) {
                app.detail_scroll = u16::MAX / 2;
            } else {
                app.scroll_preview_end();
            }
            Action::None
        }

        // Alt+↑ / Alt+↓ (Option on macOS) — scroll the active panel
        // (preview, detail, etc.) regardless of focus. Faster than the
        // selection-cycle that bare arrows do.
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
            scroll_active_panel(app, 3, false);
            return Action::None;
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
            scroll_active_panel(app, 3, true);
            return Action::None;
        }

        // Navigation: ↑/↓ AND j/k — context-aware (sessions vs menu)
        KeyCode::Down | KeyCode::Char('j') => {
            // Settings tab + detail focused: navigate ACTIONABLE fields
            if app.tab == Tab::Settings && app.detail_focused {
                let section = app.selected_settings_section();
                let providers = app.providers();
                let fields = crate::app::fields_for_section(
                    section,
                    &providers,
                    &app.config,
                );
                advance_to_next_actionable(app, &fields, true);
                return Action::None;
            }
            // Info tab: when detail focused on AISB Agents → navigate agents,
            // else scroll detail. When list focused → navigate sub-sections.
            if app.tab == Tab::Agentic && app.detail_focused {
                if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                    app.select_info_agent_next();
                } else {
                    app.scroll_detail_down(1);
                }
                return Action::None;
            }
            if app.tab == Tab::Monitor && app.detail_focused {
                app.scroll_detail_down(1);
                return Action::None;
            }
            if app.tab == Tab::Projects && app.detail_focused {
                app.scroll_detail_down(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_next(),
                Tab::Menu => app.select_menu_next(),
                Tab::Monitor => app.select_monitor_next(),
                Tab::Projects => app.select_project_next(),
                Tab::Settings => app.select_settings_next(),
                Tab::Agentic => app.select_info_next(),
                Tab::Help => app.scroll_detail_down(1),
            }
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if app.tab == Tab::Settings && app.detail_focused {
                let section = app.selected_settings_section();
                let providers = app.providers();
                let fields = crate::app::fields_for_section(
                    section,
                    &providers,
                    &app.config,
                );
                advance_to_next_actionable(app, &fields, false);
                return Action::None;
            }
            if app.tab == Tab::Agentic && app.detail_focused {
                if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                    app.select_info_agent_prev();
                } else {
                    app.scroll_detail_up(1);
                }
                return Action::None;
            }
            if app.tab == Tab::Monitor && app.detail_focused {
                app.scroll_detail_up(1);
                return Action::None;
            }
            if app.tab == Tab::Projects && app.detail_focused {
                app.scroll_detail_up(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_prev(),
                Tab::Menu => app.select_menu_prev(),
                Tab::Monitor => app.select_monitor_prev(),
                Tab::Projects => app.select_project_prev(),
                Tab::Settings => app.select_settings_prev(),
                Tab::Agentic => app.select_info_prev(),
                Tab::Help => app.scroll_detail_up(1),
            }
            Action::None
        }

        // Left/Right inside Info navigates between sub-sections (independent of agent sub-cursor)
        // We use a separate explicit handler via PgUp/PgDn — but since arrow keys are taken
        // for tabs, users can use Home/End or [/] to jump between sub-sections:
        KeyCode::Char('[') if app.tab == Tab::Agentic => {
            app.select_info_prev();
            Action::None
        }
        KeyCode::Char(']') if app.tab == Tab::Agentic => {
            app.select_info_next();
            Action::None
        }

        // Enter: context-aware. Crucially: Enter in 2-col tabs behaves like
        // Tab — focuses the right panel — instead of taking the user away
        // from Omega.
        KeyCode::Enter => match app.tab {
            Tab::Sessions => {
                // Two-panel default: Enter focuses the preview (acts like Tab).
                // Once focused, Enter is forwarded to the rmux session below
                // (interactive passthrough — see the SessionFocus::Chat branch
                // earlier in this function).
                if let Some(_entry) = app.selected_session() {
                    if app.session_focus == SessionFocus::List {
                        app.session_focus = SessionFocus::Chat;
                        app.status_message = Some(
                            "Focus: preview — keys forward to session (Tab → list, Tab-Tab fullscreen, Esc release)".to_string(),
                        );
                    }
                    Action::None
                } else {
                    Action::None
                }
            }
            Tab::Menu => execute_menu_action(app, app.selected_menu_action()),
            Tab::Monitor => execute_monitor_action(app.selected_monitor_action()),
            Tab::Settings => {
                // Enter on the section list → focus the right detail panel
                // (same as Tab). Once focused, Enter activates the selected field.
                if !app.detail_focused {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(
                        section,
                        &providers,
                        &app.config,
                    );
                    app.detail_focused = true;
                    if let Some(first) = fields.iter().position(|f| f.is_actionable()) {
                        app.settings_field_selected = first;
                    }
                    app.status_message = Some(
                        "Focus: detail (↑/↓ navigate fields, Enter activate, Tab → list, Tab-Tab → fullscreen)".to_string(),
                    );
                    Action::None
                } else {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(
                        section,
                        &providers,
                        &app.config,
                    );
                    let idx = app.settings_field_selected.min(fields.len().saturating_sub(1));
                    match fields.into_iter().nth(idx) {
                        Some(crate::app::SettingsField::Action { label, command, confirm_first }) => {
                            // Special: trigger Telegram wizard inline
                            if command == "__INTERNAL_TELEGRAM_SETUP__" {
                                app.settings_confirm_pending = None;
                                Action::TelegramSetup
                            } else if confirm_first && app.settings_confirm_pending != Some(idx) {
                                // First Enter on a destructive action → arm it,
                                // require a second Enter on the same field.
                                app.settings_confirm_pending = Some(idx);
                                app.status_message = Some(format!(
                                    "Press Enter again to confirm: {}",
                                    label.trim()
                                ));
                                Action::None
                            } else {
                                app.settings_confirm_pending = None;
                                Action::RunShellCommand { label, command }
                            }
                        }
                        Some(crate::app::SettingsField::EditText { config_key, current_value, masked, .. }) => {
                            app.settings_confirm_pending = None;
                            Action::EditSettingsField { config_key, current: current_value, masked }
                        }
                        Some(crate::app::SettingsField::Toggle { config_key, .. }) => {
                            app.settings_confirm_pending = None;
                            Action::ToggleSettingsBool { config_key }
                        }
                        _ => Action::None,
                    }
                }
            }
            Tab::Projects => {
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    app.status_message = Some(
                        "Focus: project detail (↑/↓ scroll, Enter → open in terminal, Tab → list)".to_string(),
                    );
                    Action::None
                } else {
                    // Detail focused → Enter opens the project in a terminal.
                    match app.selected_project() {
                        Some(p) => Action::OpenProject {
                            name: p.name.clone(),
                            path: p.path.to_string_lossy().to_string(),
                            oracle_session: p.oracle_session.clone(),
                        },
                        None => {
                            app.status_message = Some("No project selected".to_string());
                            Action::None
                        }
                    }
                }
            }
            Tab::Agentic => {
                // Enter on Info section list → focus the right detail panel
                // (same as Tab). Lets users browse Oracle/Workers/Rules content.
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    app.status_message = Some(
                        "Focus: detail (↑/↓ scroll or navigate agents, Tab → list, Tab-Tab → fullscreen)".to_string(),
                    );
                }
                Action::None
            }
            Tab::Help => Action::None,
        },

        // Monitor tab letter shortcuts
        KeyCode::Char('L') if app.tab == Tab::Monitor => Action::LoginClaude,
        KeyCode::Char('T') if app.tab == Tab::Monitor => Action::TelegramSetup,
        KeyCode::Char('P') if app.tab == Tab::Monitor => Action::ProvisioningSetup,
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
        // Projects tab: 'p' runs the planner for the selected project
        // (the global 'p' = new Pi session applies on every other tab).
        KeyCode::Char('p') if app.tab == Tab::Projects => {
            match app.selected_project() {
                Some(p) => Action::RunPlannerForProject {
                    name: p.name.clone(),
                    path: p.path.to_string_lossy().to_string(),
                },
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
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
        // 'h' is advertised in the Help/shortcut row as Hermes; wire it like
        // the other global launchers (p/G/t) so the advert isn't a dead key.
        KeyCode::Char('h') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("hermes".to_string());
            app.status_message = Some("Session name for new Hermes (Enter, Esc to cancel)".to_string());
            Action::None
        }

        // Projects tab: 'd' pre-fills the dispatch with the selected project,
        // skipping the project-name step → straight to mission entry.
        KeyCode::Char('d') if app.tab == Tab::Projects => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::DispatchMission(name.clone());
                    app.status_message =
                        Some(format!("Dispatch to {} — type the mission (Enter to send)", name));
                }
                None => {
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::DispatchProject;
                    app.status_message = Some("Project name (Enter to continue)".to_string());
                }
            }
            Action::None
        }
        KeyCode::Char('d') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::DispatchProject;
            app.status_message = Some("Project name (Enter to continue)".to_string());
            Action::None
        }

        // Kill — both lowercase x and uppercase X work. Sessions tab only:
        // the list isn't visible elsewhere, so don't kill a hidden selection
        // from another tab.
        KeyCode::Char('x') | KeyCode::Char('X') if app.tab == Tab::Sessions => {
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

        // Rename selected session (Sessions tab only)
        KeyCode::Char('r') | KeyCode::Char('R') if app.tab == Tab::Sessions => {
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
        // Sessions tab: '/' filters the list; 'b' jumps to the next blocked/failed.
        KeyCode::Char('/') if app.tab == Tab::Sessions => {
            app.input_buffer = app.session_filter.clone().unwrap_or_default();
            app.input_mode = InputMode::SessionFilter;
            app.status_message =
                Some("Filter: type a substring, Enter to apply, empty+Enter clears".to_string());
            Action::None
        }
        KeyCode::Char('b') if app.tab == Tab::Sessions => {
            match app.jump_to_next_flagged() {
                Some(name) => app.status_message = Some(format!("→ {}", name)),
                None => app.status_message = Some("No blocked/failed sessions".to_string()),
            }
            Action::None
        }

        KeyCode::F(5) => Action::Refresh,

        KeyCode::Char('.') if app.tab == Tab::Sessions => {
            // Toggle on the source entry, then return owned (name, state) so the
            // &mut borrow ends before we touch app.rows / app.status_message.
            let toggled = app.sessions.get_mut(app.selected).map(|entry| {
                entry.is_protected = !entry.is_protected;
                (entry.session.name.clone(), entry.is_protected)
            });
            if let Some((nm, prot)) = toggled {
                // Reflect in the RENDERED rows immediately — they are clones made
                // at refresh time, so without this the § marker lags ~2s.
                for row in app.rows.iter_mut() {
                    if let crate::app::SessionRow::Entry(e) = row {
                        if e.session.name == nm {
                            e.is_protected = prot;
                        }
                    }
                }
                app.status_message = Some(format!(
                    "{} is now {}",
                    nm,
                    if prot { "protected" } else { "unprotected" }
                ));
            }
            Action::None
        }

        KeyCode::F(1) => {
            app.tab = Tab::Help;
            app.detail_scroll = 0;
            Action::None
        }

        KeyCode::Esc => {
            // Cancel any armed destructive-menu confirm first.
            if app.menu_confirm_pending.take().is_some() {
                app.status_message = Some("Cancelled".to_string());
                return Action::None;
            }
            // Esc is a layered "back" key:
            // 1. If detail/fullscreen focused → return to section list
            // 2. If on section list → go to Sessions tab
            // 3. If on Sessions tab → quit
            if app.detail_fullscreen {
                app.detail_fullscreen = false;
                app.status_message = Some("Exited fullscreen".to_string());
                Action::None
            } else if app.detail_focused {
                app.detail_focused = false;
                app.detail_scroll = 0;
                app.status_message = Some("Focus: section list".to_string());
                Action::None
            } else if app.tab == Tab::Sessions {
                if matches!(app.session_focus, SessionFocus::Chat | SessionFocus::ChatFullscreen) {
                    app.session_focus = SessionFocus::List;
                    app.status_message = Some("Focus: session list".to_string());
                    Action::None
                } else {
                    app.should_quit = true;
                    Action::Quit
                }
            } else {
                app.tab = Tab::Sessions;
                Action::None
            }
        }

        _ => Action::None,
    }
}

/// Move the settings field cursor to the next/previous actionable field,
/// skipping Info (non-interactable) entries.
fn advance_to_next_actionable(app: &mut App, fields: &[crate::app::SettingsField], forward: bool) {
    let n = fields.len();
    if n == 0 {
        return;
    }
    let actionable: Vec<usize> = fields
        .iter()
        .enumerate()
        .filter_map(|(i, f)| if f.is_actionable() { Some(i) } else { None })
        .collect();
    if actionable.is_empty() {
        return;
    }
    // Moving the field cursor cancels any pending destructive confirmation.
    app.settings_confirm_pending = None;
    let current = app.settings_field_selected;
    let target = if forward {
        actionable
            .iter()
            .copied()
            .find(|i| *i > current)
            .unwrap_or(actionable[0])
    } else {
        actionable
            .iter()
            .copied()
            .rev()
            .find(|i| *i < current)
            .unwrap_or(*actionable.last().unwrap())
    };
    app.settings_field_selected = target;
}

fn execute_monitor_action(action: MonitorAction) -> Action {
    match action {
        MonitorAction::Login => Action::LoginClaude,
        MonitorAction::TelegramSetup => Action::TelegramSetup,
        MonitorAction::TelegramDisconnect => Action::TelegramDisconnect,
        MonitorAction::ProvisioningSetup => Action::ProvisioningSetup,
        MonitorAction::RefreshBilling => Action::RefreshBilling,
    }
}

/// Chat-input mode — REAL-TIME keystroke passthrough to the streamed rmux
/// session. Every key (printable, Enter, Backspace, arrows, Ctrl-combos,
/// Esc) is forwarded one-by-one so plan mode, OAuth code paste, and choice
/// menus work natively inside the agent.
///
/// TUI-local keys (never forwarded):
///   Tab           → cycle focus (List → Chat → Fullscreen → List)
///   Alt+Up/Down   → scroll preview
///   PageUp/Down   → scroll preview
///   Home/End      → scroll preview to top/bottom
///   Ctrl+L        → handled by the global redraw branch BEFORE this
fn handle_key_chat(app: &mut App, key: KeyEvent) -> Action {
    let session = match app.selected_session() {
        Some(entry) => entry.session.name.clone(),
        None => return Action::None,
    };

    // --- TUI-local (never forwarded) ---

    // Ctrl+R — hot-reload OmegaOS in place (re-exec the binary). Intercepted
    // even in chat focus so it always works, at the cost of Claude's own
    // Ctrl+R (reverse-search) inside the mirror — the user asked for ^R reload.
    if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Restart;
    }

    // Super-scroll: PageUp/PageDown jump a FULL page (viewport height) so a
    // Termius up/down swipe rips through history fast. Home/End below jump to
    // the absolute top/bottom of the full scrollback.
    let page = app.preview_inner_height.max(10);
    match key.code {
        KeyCode::PageUp => {
            app.scroll_preview_up(page);
            return Action::None;
        }
        KeyCode::PageDown => {
            app.scroll_preview_down(page);
            return Action::None;
        }
        _ => {}
    }

    // Tab behavior (corrected per user):
    //   Shift+Tab  → FORWARD to Claude — Claude Code uses Shift+Tab to
    //                cycle modes (plan mode, bypass, accept-edits, …).
    //   Tab        → return to the session list (back out of chat focus).
    // crossterm delivers Shift+Tab as KeyCode::BackTab (and on some
    // terminals as Tab+SHIFT) — handle both, forward the rmux "BTab".
    if key.code == KeyCode::BackTab
        || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
    {
        return Action::ForwardKeyToSession { session, key: "BTab" };
    }
    if key.code == KeyCode::Tab {
        app.session_focus = SessionFocus::List;
        app.status_message = Some("Focus: session list (Enter → chat, Shift+Tab → Claude modes)".to_string());
        return Action::None;
    }

    // Alt+arrows = TUI scroll preview
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Up => { app.scroll_preview_up(3); return Action::None; }
            KeyCode::Down => { app.scroll_preview_down(3); return Action::None; }
            _ => {}
        }
    }

    // Home / End = jump to the very top / bottom of the full scrollback.
    // (PageUp/PageDown super-scroll handled above.)
    match key.code {
        KeyCode::Home => { app.scroll_preview_home(); return Action::None; }
        KeyCode::End => { app.scroll_preview_end(); return Action::None; }
        _ => {}
    }

    // --- Forwarded to rmux session ---
    // Important: rmux key names come from rmux-core/src/keys/string_table.rs.
    // The backspace key is "BSpace" (NOT "BackSpace"); forward-delete is
    // "Delete" or "DC". Using the wrong name makes rmux send the literal text
    // instead of the key event — that was the original "BackSpace appears in
    // the agent's input" bug.

    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        // (Tab + Shift+Tab handled earlier: Tab=back to list,
        //  Shift+Tab=forward BTab to Claude for mode cycling.)

        // Word-delete (readline conventions):
        //   Ctrl+W            → kill word back   (universal)
        //   Shift+Backspace   → kill word back   (tmux convention, what the user wants)
        //   Alt+Backspace     → kill word back   (macOS / readline)
        //   Ctrl+Backspace    → kill word back   (Windows convention)
        KeyCode::Backspace if shift || alt || ctrl => {
            Action::ForwardKeyToSession { session, key: "C-w" }
        }
        KeyCode::Backspace => {
            Action::ForwardKeyToSession { session, key: "BSpace" }
        }
        // Forward-delete word:
        //   Shift+Delete or Alt+Delete → kill word forward (M-d in readline)
        KeyCode::Delete if shift || alt => {
            Action::ForwardKeyToSession { session, key: "M-d" }
        }
        KeyCode::Delete => Action::ForwardKeyToSession { session, key: "Delete" },
        // Shift+Enter / Alt+Enter → insert a newline in the input (don't
        // submit), matching real Claude Code multi-line input. Plain Enter
        // still submits.
        KeyCode::Enter if shift || alt => Action::InsertNewlineToSession { session },
        KeyCode::Enter => Action::ForwardKeyToSession { session, key: "Enter" },
        KeyCode::Esc => Action::ForwardKeyToSession { session, key: "Escape" },
        KeyCode::Up => Action::ForwardKeyToSession { session, key: "Up" },
        KeyCode::Down => Action::ForwardKeyToSession { session, key: "Down" },
        KeyCode::Left if ctrl || alt => Action::ForwardKeyToSession { session, key: "M-b" },
        KeyCode::Right if ctrl || alt => Action::ForwardKeyToSession { session, key: "M-f" },
        KeyCode::Left => Action::ForwardKeyToSession { session, key: "Left" },
        KeyCode::Right => Action::ForwardKeyToSession { session, key: "Right" },
        KeyCode::Insert => Action::ForwardKeyToSession { session, key: "IC" },
        KeyCode::F(n) if (1..=12).contains(&n) => {
            let key_str: &'static str = match n {
                1 => "F1", 2 => "F2", 3 => "F3", 4 => "F4",
                5 => "F5", 6 => "F6", 7 => "F7", 8 => "F8",
                9 => "F9", 10 => "F10", 11 => "F11", 12 => "F12",
                _ => return Action::None,
            };
            Action::ForwardKeyToSession { session, key: key_str }
        }
        KeyCode::Char(c) => {
            // Option+< / Option+> (readline beginning/end of buffer):
            //   M-<  → readline beginning-of-buffer
            //   M->  → readline end-of-buffer
            // User-friendly text navigation in Claude's input box.
            if alt {
                match c {
                    '<' | ',' if shift || c == '<' => {
                        return Action::ForwardKeyToSession { session, key: "M-<" };
                    }
                    '>' | '.' if shift || c == '>' => {
                        return Action::ForwardKeyToSession { session, key: "M->" };
                    }
                    _ => {}
                }
            }
            // Ctrl+<letter> → rmux "C-<letter>" so Ctrl+C interrupts the agent
            if ctrl {
                let lower = c.to_ascii_lowercase();
                let key_str: &'static str = match lower {
                    'a' => "C-a", 'b' => "C-b", 'c' => "C-c", 'd' => "C-d",
                    'e' => "C-e", 'f' => "C-f", 'g' => "C-g", 'h' => "C-h",
                    'i' => "C-i", 'j' => "C-j", 'k' => "C-k", 'l' => "C-l",
                    'm' => "C-m", 'n' => "C-n", 'o' => "C-o", 'p' => "C-p",
                    'q' => "C-q", 'r' => "C-r", 's' => "C-s", 't' => "C-t",
                    'u' => "C-u", 'v' => "C-v", 'w' => "C-w", 'x' => "C-x",
                    'y' => "C-y", 'z' => "C-z",
                    _ => return Action::None,
                };
                Action::ForwardKeyToSession { session, key: key_str }
            } else {
                Action::ForwardCharToSession { session, ch: c }
            }
        }
        _ => Action::None,
    }
}

fn execute_menu_action(app: &mut App, action: MenuAction) -> Action {
    // Destructive items (KillAll / NuclearCleanup) need a two-press confirm:
    // first Enter arms, second Enter on the SAME item fires. Any other
    // selection disarms.
    let armed = app.menu_confirm_pending == Some(action);
    if !armed {
        app.menu_confirm_pending = None;
    }
    if matches!(action, MenuAction::KillAll | MenuAction::NuclearCleanup) {
        if armed {
            app.menu_confirm_pending = None;
            return match action {
                MenuAction::KillAll => Action::KillAllSessions,
                MenuAction::NuclearCleanup => Action::NuclearCleanup,
                _ => Action::None,
            };
        }
        app.menu_confirm_pending = Some(action);
        let verb = if matches!(action, MenuAction::NuclearCleanup) {
            "NUCLEAR CLEANUP (kill all + prune state + free RAM)"
        } else {
            "KILL ALL sessions"
        };
        app.status_message =
            Some(format!("[!] {} — press Enter again to CONFIRM, Esc to cancel", verb));
        return Action::None;
    }

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
        MenuAction::NewProject => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewProjectName;
            app.status_message =
                Some("New project — name (Enter to continue, Esc to cancel)".to_string());
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
        MenuAction::Restart => Action::Restart,
        MenuAction::Quit => {
            app.should_quit = true;
            Action::Quit
        }
        // Per-agent variants handled above
        _ => Action::None,
    }
}

/// Advance the provisioning-keys wizard: record `value` for the current `step`,
/// then move to the next field or, when the last field is done, emit the commit
/// action with all (key, value) pairs zipped from `PROVISIONING_FIELDS`.
fn provisioning_advance(
    app: &mut App,
    mut collected: Vec<String>,
    value: String,
    step: usize,
) -> Action {
    let fields = crate::app::PROVISIONING_FIELDS;
    collected.push(value);
    let next = step + 1;
    if next >= fields.len() {
        app.input_mode = InputMode::Normal;
        let values: Vec<(String, String)> = fields
            .iter()
            .zip(collected.iter())
            .map(|((k, _, _), v)| (k.to_string(), v.clone()))
            .collect();
        Action::ProvisioningCommit { values }
    } else {
        app.input_buffer = String::new();
        app.input_mode = InputMode::ProvisioningSetup { step: next, collected };
        app.status_message = Some(format!(
            "Step {}/{}: {}",
            next + 1,
            fields.len(),
            fields[next].1
        ));
        Action::None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SessionEntry;
    use omega_core::config::OmegaConfig;
    use omega_core::session::OmegaSession;

    fn test_app() -> App {
        App::new(OmegaConfig::default())
    }
    fn press(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    // Bug B: 'h' is advertised as the Hermes launcher; it must work from any
    // tab like the other global launchers (p/G/t), not be a dead key.
    #[test]
    fn h_launches_hermes_from_any_tab() {
        let mut app = test_app();
        app.tab = Tab::Monitor;
        let action = handle_key(&mut app, press('h'));
        assert!(matches!(action, Action::None));
        assert!(matches!(&app.input_mode, InputMode::NewNamedSession(s) if s == "hermes"));
    }

    // Bug A: destructive keys act only on the Sessions tab, never on a hidden
    // selection from another tab.
    #[test]
    fn kill_key_is_guarded_to_sessions_tab() {
        let mut app = test_app();
        app.sessions.push(SessionEntry {
            session: OmegaSession::classify("test-worker"),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        });
        app.selected = 0;

        app.tab = Tab::Monitor;
        assert!(
            matches!(handle_key(&mut app, press('x')), Action::None),
            "x off the Sessions tab must not kill the hidden selection"
        );

        app.tab = Tab::Sessions;
        assert!(
            matches!(handle_key(&mut app, press('x')), Action::KillSession(n) if n == "test-worker"),
            "x on the Sessions tab must kill the selected session"
        );
    }
}
