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
    /// Toggle terminal mouse capture. OFF → the terminal does native
    /// click-drag text selection + copy/paste; ON → clickable menus + scroll.
    ToggleMouseCapture,
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
    /// Start the Claude OAuth re-login engine (request_reauth) — captures the
    /// authorize URL and surfaces it in the Monitor → Account view.
    LoginClaude,
    /// Submit the OAuth authorize code (Monitor → Account) — runs handle_code
    /// against the waiting reauth session.
    SubmitReauthCode { code: String },
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
    /// Projects tab: register an existing folder into the project registry
    /// (reuses `project_manager::add_existing_project`).
    RegisterProject { path: String },
    /// Projects tab: remove a project from the registry (two-press confirmed).
    /// Delete a project at one of the three Telegram-parity tiers:
    /// mode = "omega" (unmanage), "local" (+ folder), "all" (+ GitHub repo).
    /// Runs the bot's one-shot CLI — the ONE canonical deletion impl.
    DeleteProjectTier { name: String, mode: &'static str },
    /// Projects tab: flip the selected project's Telegram toggle (topic sync +
    /// Atlas bot visibility). Writes `ManagedProject.telegram`; the next `/sync`
    /// reconciles the forum topic (creates when ON, removes when OFF).
    ToggleProjectTelegram { name: String },
    /// Projects tab: "Delete forever" (two-press confirmed) — runs the canonical
    /// deletion (Telegram topic + dashboard agent + agent-bot + registry + the
    /// local folder) via the bot's one-shot CLI so there is ONE implementation.
    /// Monitor → Project group: persist the Telegram supergroup id
    /// (`TelegramGroupConfig`). The manual fallback to the bot's auto-detect.
    GroupSetupCommit { group_id: i64 },
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

/// Enter on the Monitor → Account section. Context-aware re-login driver: when
/// the authorize URL is already captured (`ShowUrl`), Enter opens the code-input
/// modal; otherwise it (re)starts the OAuth engine.
fn account_enter_action(app: &mut App) -> Action {
    use crate::app::{InputMode, ReauthStatus};
    match &app.reauth_status {
        ReauthStatus::ShowUrl(_) => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::ReauthCode;
            app.status_message = Some(
                "Paste the authorize code from your browser — Enter to submit, Esc to cancel"
                    .to_string(),
            );
            Action::None
        }
        // Generating / Validating — a step is in flight; ignore re-entry.
        ReauthStatus::Generating | ReauthStatus::Validating => {
            app.status_message = Some("Re-login already in progress…".to_string());
            Action::None
        }
        // Idle / Done / Error — (re)start the flow.
        _ => Action::LoginClaude,
    }
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
            // Menu tab: click an action row → select + run it (like Enter).
            // Guarded to when the list fits on screen (offset 0) so a click never
            // maps to the wrong row while scrolled — keyboard always works too.
            if app.tab == Tab::Menu {
                let a = app.menu_area;
                if app.menu_fits
                    && mouse.column > a.x
                    && mouse.column < a.x + a.width.saturating_sub(1)
                    && mouse.row > a.y
                    && mouse.row < a.y + a.height.saturating_sub(1)
                {
                    let ridx = (mouse.row - a.y - 1) as usize; // rendered row (border-relative)
                    if let Some(Some(act_idx)) = app.menu_rendered_actions.get(ridx).copied() {
                        app.menu_selected = act_idx;
                        return execute_menu_action(app, app.selected_menu_action());
                    }
                }
                return Action::None;
            }
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
        Tab::Settings => {
            if app.detail_focused {
                if down { app.scroll_detail_down(lines); }
                else { app.scroll_detail_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.settings_tab_next(); } else { app.settings_tab_prev(); }
                }
            }
        }
        Tab::Agentic => {
            if app.detail_focused {
                if down { app.scroll_detail_down(lines); }
                else { app.scroll_detail_up(lines); }
            } else {
                for _ in 0..lines {
                    if down { app.agentic_tab_next(); } else { app.agentic_tab_prev(); }
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

/// Open the dispatch step-1 project picker over the shared `ProjectRegistry`
/// (the SAME source the Telegram dispatch picker uses, so the added-projects list
/// stays in sync). No project added yet → a status hint instead of an empty picker.
fn open_dispatch_picker(app: &mut App) {
    let names = crate::app::dispatch_project_names();
    if names.is_empty() {
        app.input_mode = InputMode::Normal;
        app.status_message = Some(
            "No projects yet — add one in the Projects tab ([n] new / register), then dispatch."
                .to_string(),
        );
    } else {
        app.input_buffer = String::new();
        app.input_mode = InputMode::DispatchProject(names, 0);
        app.status_message = Some("Pick a project — ↑/↓, Enter, Esc".to_string());
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Global: Ctrl-T toggles mouse capture. OFF lets you drag-select + copy/paste
    // with the terminal's native selection; ON gives clickable menus + scroll.
    if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::ToggleMouseCapture;
    }
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

        InputMode::DispatchProject(projects, sel) => {
            let count = projects.len().max(1);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::DispatchProject(projects, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::DispatchProject(projects, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let project = projects.get(sel).cloned().unwrap_or_default();
                    app.input_buffer = String::new();
                    app.status_message = Some(format!(
                        "Dispatch to {} — type the mission (Enter to send)",
                        project
                    ));
                    app.input_mode = InputMode::DispatchMission(project);
                    Action::None
                }
                _ => Action::None,
            }
        }
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
        InputMode::ProjectDelete(name, sel) => {
            const COUNT: usize = 4; // 3 tiers + cancel
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::ProjectDelete(name, (sel + 1) % COUNT);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { COUNT - 1 } else { sel - 1 };
                    app.input_mode = InputMode::ProjectDelete(name, next);
                    Action::None
                }
                // Digit shortcuts mirror the Telegram buttons (1️⃣/2️⃣/3️⃣).
                KeyCode::Char('1') => { app.input_mode = InputMode::Normal; Action::DeleteProjectTier { name, mode: "omega" } }
                KeyCode::Char('2') => { app.input_mode = InputMode::Normal; Action::DeleteProjectTier { name, mode: "local" } }
                KeyCode::Char('3') => { app.input_mode = InputMode::Normal; Action::DeleteProjectTier { name, mode: "all" } }
                KeyCode::Enter => {
                    app.input_mode = InputMode::Normal;
                    match sel {
                        0 => Action::DeleteProjectTier { name, mode: "omega" },
                        1 => Action::DeleteProjectTier { name, mode: "local" },
                        2 => Action::DeleteProjectTier { name, mode: "all" },
                        _ => { app.status_message = Some("Cancelled".to_string()); Action::None }
                    }
                }
                _ => Action::None,
            }
        }

        InputMode::SelectModel(config_key, options, sel) => {
            let count = options.len().max(1);
            // Theme selector: live-preview the highlighted theme on every
            // arrow move so the whole TUI re-skins under the overlay.
            let is_theme = config_key == "general.theme";
            match key.code {
                KeyCode::Esc => {
                    if is_theme {
                        // Revert the live preview to the saved theme.
                        crate::theme::set_active_slug(&app.config.theme);
                    }
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (sel + 1) % count;
                    if is_theme {
                        if let Some(slug) = options.get(next) {
                            crate::theme::set_active_slug(slug);
                        }
                    }
                    app.input_mode = InputMode::SelectModel(config_key, options, next);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    if is_theme {
                        if let Some(slug) = options.get(next) {
                            crate::theme::set_active_slug(slug);
                        }
                    }
                    app.input_mode = InputMode::SelectModel(config_key, options, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let value = options.get(sel).cloned().unwrap_or_default();
                    app.input_mode = InputMode::Normal;
                    Action::CommitSettingsEdit { config_key, value }
                }
                _ => Action::None,
            }
        }

        // Monitor → Project group: single-field group_id capture (manual
        // fallback to the bot's auto-detect). Numeric-validated like the
        // Telegram chat_id step.
        InputMode::GroupSetupId => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            match value.trim().parse::<i64>() {
                Ok(group_id) => Action::GroupSetupCommit { group_id },
                Err(_) => {
                    app.status_message =
                        Some(format!("Invalid group_id '{}' — must be a numeric id", value.trim()));
                    Action::None
                }
            }
        }),

        // Register-existing-folder: single-field path capture → register it
        // (the folder name becomes the project name, per add_existing_project).
        InputMode::AddProjectPath => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            let path = value.trim().to_string();
            if path.is_empty() {
                app.status_message = Some("No path entered".to_string());
                Action::None
            } else {
                Action::RegisterProject { path }
            }
        }),

        // Claude OAuth re-login: paste the authorize code → handle_code.
        InputMode::ReauthCode => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            let code = value.trim().to_string();
            if code.is_empty() {
                app.status_message = Some("No code entered".to_string());
                Action::None
            } else {
                Action::SubmitReauthCode { code }
            }
        }),

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
        // Ctrl+R — RELOAD the TUI: tear down + re-exec the freshly-built binary
        // (browser-style ^R = reload). Soft in-place refresh (no teardown) is on
        // F5 / the menu's Refresh item; menu "R" also reloads.
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
                    SessionFocus::List => "Session list — Tab: open · Tab-Tab: hide/show menu".to_string(),
                    SessionFocus::Chat => "In session — Tab: back to list · Ctrl+X: close session · Tab-Tab: hide/show menu".to_string(),
                    SessionFocus::ChatFullscreen => "Session FULLSCREEN — Ctrl+X: close · Tab-Tab: show menu".to_string(),
                });
            } else if matches!(app.tab, Tab::Settings | Tab::Agentic) {
                // 2-column tabs: Tab toggles list↔detail, Tab-Tab → fullscreen
                app.handle_tab_in_2col();
                // When entering detail on the Settings group, snap cursor to the
                // first actionable field (the Monitor group has no field list).
                if app.tab == Tab::Settings && app.detail_focused && !app.settings_on_monitor() {
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
            if matches!(app.tab, Tab::Settings | Tab::Agentic) {
                app.scroll_detail_down(10);
            } else {
                app.scroll_preview_down(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::PageUp => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic) {
                app.scroll_detail_up(10);
            } else {
                app.scroll_preview_up(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::Home => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic) {
                app.detail_scroll = 0;
            } else {
                app.scroll_preview_home();
            }
            Action::None
        }
        KeyCode::End => {
            if matches!(app.tab, Tab::Settings | Tab::Agentic) {
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

        // Sessions tab: navigate with ARROWS ONLY — j/k must NOT move the cursor
        // (operator preference). Catch bare j/k here first so they're ignored on
        // Sessions; on every other tab j/k still navigate via the arms below.
        KeyCode::Char('j') | KeyCode::Char('k') if app.tab == Tab::Sessions => Action::None,
        // Navigation: ↑/↓ AND j/k — context-aware (sessions vs menu)
        KeyCode::Down | KeyCode::Char('j') => {
            // Settings tab + detail focused: Monitor group → action cursor (on
            // the Actions section) or scroll; Settings group → navigate fields.
            if app.tab == Tab::Settings && app.detail_focused {
                if app.settings_on_monitor() {
                    if matches!(app.selected_monitor_section(), crate::app::MonitorSection::Actions) {
                        app.select_monitor_action_next();
                    } else {
                        app.scroll_detail_down(1);
                    }
                } else {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    advance_to_next_actionable(app, &fields, true);
                }
                return Action::None;
            }
            // Agentic tab + detail focused: Projects group → scroll the project
            // detail; info group → navigate AISB agents (on that section) or scroll.
            if app.tab == Tab::Agentic && app.detail_focused {
                if app.agentic_on_projects() {
                    app.scroll_detail_down(1);
                } else if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                    app.select_info_agent_next();
                } else {
                    app.scroll_detail_down(1);
                }
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_next(),
                Tab::Menu => app.select_menu_next(),
                Tab::Settings => app.settings_tab_next(),
                Tab::Agentic => app.agentic_tab_next(),
                Tab::Help => app.scroll_detail_down(1),
            }
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if app.tab == Tab::Settings && app.detail_focused {
                if app.settings_on_monitor() {
                    if matches!(app.selected_monitor_section(), crate::app::MonitorSection::Actions) {
                        app.select_monitor_action_prev();
                    } else {
                        app.scroll_detail_up(1);
                    }
                } else {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    advance_to_next_actionable(app, &fields, false);
                }
                return Action::None;
            }
            if app.tab == Tab::Agentic && app.detail_focused {
                if app.agentic_on_projects() {
                    app.scroll_detail_up(1);
                } else if matches!(app.selected_info_section(), crate::app::InfoSection::AisbAgents) {
                    app.select_info_agent_prev();
                } else {
                    app.scroll_detail_up(1);
                }
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_prev(),
                Tab::Menu => app.select_menu_prev(),
                Tab::Settings => app.settings_tab_prev(),
                Tab::Agentic => app.agentic_tab_prev(),
                Tab::Help => app.scroll_detail_up(1),
            }
            Action::None
        }

        // Left/Right inside Info navigates between sub-sections (independent of agent sub-cursor)
        // We use a separate explicit handler via PgUp/PgDn — but since arrow keys are taken
        // for tabs, users can use Home/End or [/] to jump between sub-sections:
        KeyCode::Char('[') if app.tab == Tab::Agentic && !app.agentic_on_projects() => {
            app.select_info_prev();
            Action::None
        }
        KeyCode::Char(']') if app.tab == Tab::Agentic && !app.agentic_on_projects() => {
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
                if let Some(entry) = app.selected_session() {
                    // Master + Telegram-not-yet-configured → Enter opens the
                    // existing 3-step Telegram setup wizard instead of focusing
                    // the (empty) live mirror. Reuses the canonical wizard;
                    // commit auto-attaches the master so the user watches the
                    // confirmation stream in (see TelegramSetupCommit handler).
                    if app.session_focus == SessionFocus::List
                        && omega_core::aisb::is_master(&entry.session.name)
                        && !omega_core::monitor::OmegaTelegramConfig::exists()
                    {
                        return Action::TelegramSetup;
                    }
                    if app.session_focus == SessionFocus::List {
                        app.session_focus = SessionFocus::Chat;
                        app.cmd_capture = None;
                        app.chat_line_chars = 0;
                        app.status_message = Some(
                            "Focus: chat — keys forward to agent (Tab → agent, Tab-Tab → close to list, / = OmegaOS commands)".to_string(),
                        );
                    }
                    Action::None
                } else {
                    Action::None
                }
            }
            Tab::Menu => execute_menu_action(app, app.selected_menu_action()),
            Tab::Settings => {
                // 2-column model. Enter on the section list focuses the detail
                // panel. Once focused, behaviour splits by group: the Monitor
                // group routes each section to its primary action; the Settings
                // group activates the selected provider field.
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    // Settings group → snap to the first actionable field.
                    if !app.settings_on_monitor() {
                        let section = app.selected_settings_section();
                        let providers = app.providers();
                        let fields = crate::app::fields_for_section(section, &providers, &app.config);
                        if let Some(first) = fields.iter().position(|f| f.is_actionable()) {
                            app.settings_field_selected = first;
                        }
                    }
                    app.status_message = Some(
                        "Focus: detail (↑/↓ navigate, Enter activate, Tab → list, Tab-Tab → fullscreen)".to_string(),
                    );
                    Action::None
                } else if app.settings_on_monitor() {
                    // ── Monitor group: focused-Enter routes the section to its
                    // primary wizard/action (no command-line, no new wizard). ──
                    use crate::app::MonitorSection;
                    match app.selected_monitor_section() {
                        MonitorSection::Actions => {
                            let action = app.selected_monitor_action();
                            // OpenDashboard needs `&mut App` → its own handler.
                            if matches!(action, MonitorAction::OpenDashboard) {
                                open_dashboard_action(app)
                            } else {
                                execute_monitor_action(action)
                            }
                        }
                        // Account & billing → the OAuth re-login engine. Context-
                        // aware: once the authorize URL is captured, Enter opens
                        // the code modal; otherwise it kicks off request_reauth.
                        MonitorSection::AccountBilling => account_enter_action(app),
                        // Telegram & projects → set up when absent, or a two-press-
                        // confirmed disconnect when a config exists.
                        MonitorSection::Telegram => {
                            if omega_core::monitor::OmegaTelegramConfig::exists() {
                                if app.monitor_disconnect_armed {
                                    app.monitor_disconnect_armed = false;
                                    Action::TelegramDisconnect
                                } else {
                                    app.monitor_disconnect_armed = true;
                                    app.status_message = Some(
                                        "Press Enter again to DISCONNECT the Telegram bot (Esc to cancel)".to_string(),
                                    );
                                    Action::None
                                }
                            } else {
                                Action::TelegramSetup
                            }
                        }
                    }
                } else {
                    // ── Settings group: activate the selected provider field. ──
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
                        Some(crate::app::SettingsField::Select { config_key, options, current_index, .. }) => {
                            // Open the arrow-key overlay (no typing).
                            app.settings_confirm_pending = None;
                            app.input_mode = crate::app::InputMode::SelectModel(config_key, options, current_index);
                            Action::None
                        }
                        Some(crate::app::SettingsField::Toggle { config_key, .. }) => {
                            app.settings_confirm_pending = None;
                            Action::ToggleSettingsBool { config_key }
                        }
                        _ => Action::None,
                    }
                }
            }
            Tab::Agentic => {
                if app.agentic_on_projects() {
                    // ── Projects group ──
                    if app.project_registry.projects.is_empty() {
                        // Empty registry: Enter opens the same add-project modal
                        // as 'n' — the literal "Enter adds a project" affordance.
                        app.input_buffer = String::new();
                        app.input_mode = InputMode::AddProjectPath;
                        app.status_message = Some(
                            "Add project — path to an existing folder (Enter to register, Esc to cancel)".to_string(),
                        );
                        Action::None
                    } else if !app.detail_focused {
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
                } else {
                    // ── Agentic info group: Enter focuses the detail panel so
                    // users can browse Oracle/Workers/Rules content. ──
                    if !app.detail_focused {
                        app.detail_focused = true;
                        app.detail_scroll = 0;
                        app.status_message = Some(
                            "Focus: detail (↑/↓ scroll or navigate agents, Tab → list, Tab-Tab → fullscreen)".to_string(),
                        );
                    }
                    Action::None
                }
            }
            Tab::Help => Action::None,
        },

        // Settings tab → Monitor group: letter shortcuts (only when the cursor
        // sits in the Monitor group, so they don't fire while editing providers).
        KeyCode::Char('L') if app.tab == Tab::Settings && app.settings_on_monitor() => Action::LoginClaude,
        KeyCode::Char('T') if app.tab == Tab::Settings && app.settings_on_monitor() => Action::TelegramSetup,
        KeyCode::Char('P') if app.tab == Tab::Settings && app.settings_on_monitor() => Action::ProvisioningSetup,
        KeyCode::Char('D') if app.tab == Tab::Settings && app.settings_on_monitor() => Action::TelegramDisconnect,
        KeyCode::Char('B') if app.tab == Tab::Settings && app.settings_on_monitor() => Action::RefreshBilling,
        KeyCode::Char('O') if app.tab == Tab::Settings && app.settings_on_monitor() => open_dashboard_action(app),

        // Projects tab: 'n' opens a guided "register existing folder" modal
        // (the in-TUI replacement for the `omega project add` CLI hint). For a
        // green-field scaffold the Menu tab's New-project wizard still applies.
        KeyCode::Char('n') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::AddProjectPath;
            app.status_message =
                Some("Add project — path to an existing folder (Enter to register, Esc to cancel)".to_string());
            app.project_confirm_pending = None;
            Action::None
        }
        // Settings group: 'x' clears the selected text field (e.g. unlink a saved
        // API key) — two-press confirm, same affordance as destructive actions.
        // Backspace can't work here: it's only meaningful inside the edit modal.
        KeyCode::Char('x') | KeyCode::Char('X')
            if app.tab == Tab::Settings && app.detail_focused && !app.settings_on_monitor() =>
        {
            let section = app.selected_settings_section();
            let providers = app.providers();
            let fields = crate::app::fields_for_section(section, &providers, &app.config);
            let idx = app.settings_field_selected.min(fields.len().saturating_sub(1));
            match fields.into_iter().nth(idx) {
                Some(crate::app::SettingsField::EditText { config_key, current_value, label, .. })
                    if !current_value.is_empty() =>
                {
                    if app.settings_confirm_pending == Some(idx) {
                        app.settings_confirm_pending = None;
                        app.status_message = Some(format!("Cleared: {}", label.trim()));
                        Action::CommitSettingsEdit { config_key, value: String::new() }
                    } else {
                        app.settings_confirm_pending = Some(idx);
                        app.status_message =
                            Some(format!("Press x again to clear: {}", label.trim()));
                        Action::None
                    }
                }
                _ => Action::None,
            }
        }
        // Projects tab: 'x' opens the DELETE menu — the same three escalating
        // tiers as the Telegram bot (visible options, no hidden hotkey-guessing).
        KeyCode::Char('x') | KeyCode::Char('X') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    app.input_mode = InputMode::ProjectDelete(name, 0);
                    Action::None
                }
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }
        KeyCode::Char('T') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => Action::ToggleProjectTelegram { name },
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }
        // Projects tab: 'D' = Delete forever (two-press confirm) — removes the
        // project from OmegaOS AND deletes its local folder. Distinct from 'x'
        // (registry-only removal). First press arms it; second 'D' fires.
        KeyCode::Char('D') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    if app.project_delete_pending.as_deref() == Some(name.as_str()) {
                        app.project_delete_pending = None;
                        Action::DeleteProjectTier { name, mode: "local" }
                    } else {
                        app.project_delete_pending = Some(name.clone());
                        app.status_message = Some(format!(
                            "Press D again to DELETE LOCAL MACHINE '{}' (OmegaOS + kill oracle + rm -rf LOCAL FOLDER; GitHub kept) — Esc to cancel",
                            name
                        ));
                        Action::None
                    }
                }
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }

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
        KeyCode::Char('p') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
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
        KeyCode::Char('d') if app.tab == Tab::Agentic && app.agentic_on_projects() => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::DispatchMission(name.clone());
                    app.status_message =
                        Some(format!("Dispatch to {} — type the mission (Enter to send)", name));
                }
                None => open_dispatch_picker(app),
            }
            Action::None
        }
        KeyCode::Char('d') => {
            open_dispatch_picker(app);
            Action::None
        }

        // Kill — both lowercase x and uppercase X work. Sessions tab only:
        // the list isn't visible elsewhere, so don't kill a hidden selection
        // from another tab.
        KeyCode::Char('x') | KeyCode::Char('X') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
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
        KeyCode::Char('r') | KeyCode::Char('R') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
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
        KeyCode::Char('/') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
            app.input_buffer = app.session_filter.clone().unwrap_or_default();
            app.input_mode = InputMode::SessionFilter;
            app.status_message =
                Some("Filter: type a substring, Enter to apply, empty+Enter clears".to_string());
            Action::None
        }
        KeyCode::Char('b') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
            match app.jump_to_next_flagged() {
                Some(name) => app.status_message = Some(format!("→ {}", name)),
                None => app.status_message = Some("No blocked/failed sessions".to_string()),
            }
            Action::None
        }

        KeyCode::F(5) => Action::Refresh,

        KeyCode::Char('.') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
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
            // Cancel an armed project-remove or Telegram-disconnect confirm.
            if app.project_confirm_pending.take().is_some()
                || app.monitor_disconnect_armed
            {
                app.monitor_disconnect_armed = false;
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
                // Chat/ChatFullscreen Esc never reaches here — the router
                // short-circuits into handle_key_chat, which owns chat Esc.
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
        // OpenDashboard needs the `&mut App` to surface the honest "not
        // installed" status when OmegaMC is absent, so the Enter/letter
        // handlers route through `open_dashboard_action(app)` directly. This
        // arm is unreachable in practice but keeps the match total.
        MonitorAction::OpenDashboard => Action::None,
    }
}

/// Resolve + dispatch the Monitor "Open Dashboard" action. When OmegaMC is
/// installed (`$OMEGA_DIR/omega-mc/.git`), launch it via `Action::RunShellCommand`
/// (`docker compose up -d`) — the same session-spawning mechanism the Settings
/// install/uninstall actions use. When absent, set an honest install message
/// and dispatch nothing.
fn open_dashboard_action(app: &mut App) -> Action {
    match MonitorAction::resolve_open_dashboard() {
        crate::app::DashboardLaunch::Launch { command, message } => {
            app.status_message = Some(message);
            Action::RunShellCommand {
                label: "OmegaMC dashboard".to_string(),
                command,
            }
        }
        crate::app::DashboardLaunch::NotInstalled { message } => {
            app.status_message = Some(message);
            Action::None
        }
    }
}

/// Chat-input mode — REAL-TIME keystroke passthrough to the streamed rmux
/// session. Every key (printable, Enter, Backspace, arrows, Ctrl-combos)
/// is forwarded one-by-one so plan mode, OAuth code paste, and choice
/// menus work natively inside the agent.
///
/// TUI-local keys (never forwarded):
///   Esc           → back to session list (F-2; interrupt agent = Ctrl+C)
///   Tab           → cycle focus (List → Chat → Fullscreen → List)
///   Alt+Up/Down   → scroll preview
///   PageUp/Down   → scroll preview
///   Home/End      → scroll preview to top/bottom
///   Ctrl+L        → handled by the global redraw branch BEFORE this
fn handle_key_chat(app: &mut App, key: KeyEvent) -> Action {
    let session = match app.selected_session() {
        Some(entry) => entry.session.name.clone(),
        None => {
            // No session to chat with — e.g. the focused session died and the
            // list emptied. Returning None here swallowed EVERY key (even Tab
            // and q), soft-locking the whole UI. Instead drop back to list focus
            // and re-dispatch this keystroke in normal mode so it still acts
            // (Tab, q, arrows all work again). No recursion: focus is now List,
            // so handle_key_normal won't route back into the chat handler.
            app.session_focus = SessionFocus::List;
            return handle_key_normal(app, key);
        }
    };

    // --- OmegaOS slash-command capture (shared command set) ---
    // While capturing, keys build the command buffer locally instead of going
    // to the agent. Enter resolves: a known OmegaOS command runs in the TUI;
    // anything else is typed into the agent verbatim (its own slash commands
    // still work — press Enter again to submit). Esc cancels. This is the TUI
    // half of the command set the Telegram bridge also serves.
    if let Some(mut buf) = app.cmd_capture.take() {
        return match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                buf.push(c);
                app.status_message = Some(format!("OmegaOS » {}   (Enter run · Esc cancel)", buf));
                app.cmd_capture = Some(buf);
                Action::None
            }
            KeyCode::Backspace => {
                buf.pop();
                if buf.is_empty() {
                    app.status_message = Some("Command cancelled".to_string());
                } else {
                    app.status_message = Some(format!("OmegaOS » {}", buf));
                    app.cmd_capture = Some(buf);
                }
                Action::None
            }
            KeyCode::Esc => {
                app.status_message = Some("Command cancelled".to_string());
                Action::None
            }
            KeyCode::Enter => {
                if let Some(tab) = omega_chat_command(&buf) {
                    app.tab = tab;
                    if tab == Tab::Sessions {
                        app.session_focus = SessionFocus::List;
                    }
                    app.reset_2col_focus();
                    app.status_message = Some(format!("→ {}  (OmegaOS {})", tab_label(tab), buf));
                    Action::None
                } else {
                    app.status_message = Some(format!(
                        "'{}' isn't an OmegaOS command — typed to the agent (Enter to send)",
                        buf
                    ));
                    Action::SendTextRawToSession { session, text: buf }
                }
            }
            _ => {
                app.cmd_capture = Some(buf);
                Action::None
            }
        };
    }

    // --- TUI-local (never forwarded) ---

    // Ctrl+R — RELOAD the TUI even from chat focus (browser-style ^R = reload):
    // tear down + re-exec the freshly-built binary. Consistent with list focus.
    // Soft in-place refresh stays on F5 / the menu's Refresh item.
    if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Restart;
    }

    // Ctrl+X — CLOSE (kill) the session you're focused in, WITHOUT first Tabbing
    // back to the list. Plain 'x' must stay typable into the agent, so the close
    // action is the Ctrl+X chord (mnemonic: eXit; same deliberate override style
    // as Ctrl+R above). Drop to the list first so the user lands on a live
    // selection after the focused pane dies instead of a dead/empty chat.
    if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.session_focus = SessionFocus::List;
        return Action::KillSession(session);
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
        // Unified Tab behavior (same as from the list): single Tab navigates
        // list↔session, Tab-Tab toggles the left session menu (hide↔show).
        // Shift+Tab (→ BTab for Claude mode-cycling) is handled above.
        app.handle_tab_in_sessions();
        app.status_message = Some(match app.session_focus {
            SessionFocus::List => "Session list — Tab: open · Tab-Tab: hide/show menu".to_string(),
            SessionFocus::Chat => "In session — Tab: back to list · Tab-Tab: hide/show menu".to_string(),
            SessionFocus::ChatFullscreen => "Session FULLSCREEN — Tab-Tab: show menu".to_string(),
        });
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
            app.chat_line_chars = app.chat_line_chars.saturating_sub(1);
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
        KeyCode::Enter if shift || alt => {
            app.chat_line_chars = 0;
            Action::InsertNewlineToSession { session }
        }
        KeyCode::Enter => {
            app.chat_line_chars = 0;
            Action::ForwardKeyToSession { session, key: "Enter" }
        }
        KeyCode::Esc => {
            // Esc = back to the session list — matches the title hint, the Help
            // tab, and the layered-Esc pattern on every other tab (F-2). NOT
            // forwarded: interrupting the agent stays available via Ctrl+C (C-c).
            app.session_focus = SessionFocus::List;
            app.status_message = Some("Focus: session list".to_string());
            Action::None
        }
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
                // Forward EVERY printable char to the agent — including a
                // leading "/". We must NOT intercept "/" for OmegaOS command
                // capture: doing so swallowed the keystrokes Claude Code needs
                // for its OWN "/" slash-command menu, so the menu never opened,
                // the selection was invisible, and the capture state could wedge
                // the whole TUI. Claude's "/" now works natively; reach OmegaOS
                // tabs via the menu (Tab out of chat) instead.
                app.chat_line_chars += 1;
                Action::ForwardCharToSession { session, ch: c }
            }
        }
        _ => Action::None,
    }
}

/// Map an OmegaOS slash command typed in a chat to the tab it opens — the TUI
/// half of the shared command set (the Telegram bridge accepts the same names).
/// Returns None for anything that isn't an OmegaOS navigation command, so the
/// caller hands it to the agent verbatim (the agent's own slash commands work).
fn omega_chat_command(raw: &str) -> Option<Tab> {
    match raw
        .trim()
        .trim_start_matches('/')
        .to_ascii_lowercase()
        .as_str()
    {
        // Projects + the Monitor view live inside Agentic / Settings now; keep
        // the old command words working by routing them to their new home tab.
        "projects" | "project" => Some(Tab::Agentic),
        "sessions" | "relay" => Some(Tab::Sessions),
        "monitor" | "status" => Some(Tab::Settings),
        "settings" | "config" => Some(Tab::Settings),
        "agents" | "agentic" | "aisb" => Some(Tab::Agentic),
        "menu" => Some(Tab::Menu),
        "help" => Some(Tab::Help),
        _ => None,
    }
}

fn tab_label(tab: Tab) -> &'static str {
    match tab {
        Tab::Sessions => "Sessions",
        Tab::Menu => "Menu",
        Tab::Settings => "Settings",
        Tab::Agentic => "Agentic",
        Tab::Help => "Help",
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
            open_dispatch_picker(app);
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

    // Dispatch step-1 is a PROJECT PICKER (no typing): ↑/↓ navigate the
    // added-projects list, Enter selects → step-2 mission entry. (User ask:
    // present the added projects instead of forcing the operator to type a name.)
    #[test]
    fn dispatch_picker_navigates_and_selects() {
        let mut app = test_app();
        let projects = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        app.input_mode = InputMode::DispatchProject(projects, 0);

        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Down twice → index 2 (Gamma), Up once → index 1 (Beta).
        handle_key(&mut app, down);
        handle_key(&mut app, down);
        handle_key(&mut app, up);
        assert!(matches!(&app.input_mode, InputMode::DispatchProject(_, 1)));

        // Enter commits the highlighted project and advances to mission entry.
        let action = handle_key(&mut app, enter);
        assert!(matches!(action, Action::None));
        assert!(matches!(&app.input_mode, InputMode::DispatchMission(p) if p == "Beta"));
    }

    // Up from the first item wraps to the last (matches the SelectModel picker).
    #[test]
    fn dispatch_picker_wraps_at_edges() {
        let mut app = test_app();
        app.input_mode =
            InputMode::DispatchProject(vec!["A".into(), "B".into(), "C".into()], 0);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        handle_key(&mut app, up);
        assert!(
            matches!(&app.input_mode, InputMode::DispatchProject(_, 2)),
            "Up from first item wraps to last"
        );
    }

    // Bug B: 'h' is advertised as the Hermes launcher; it must work from any
    // tab like the other global launchers (p/G/t), not be a dead key.
    #[test]
    fn h_launches_hermes_from_any_tab() {
        let mut app = test_app();
        app.tab = Tab::Settings;
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

        app.tab = Tab::Settings;
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

    // Soft-lock regression: chat focus with an EMPTY session list used to
    // swallow every key (even q and Tab) because handle_key_chat returned
    // Action::None when selected_session() was None — locking the whole UI. It
    // must instead recover to list focus and still act on the keystroke.
    #[test]
    fn chat_focus_with_empty_list_does_not_soft_lock() {
        let mut app = test_app();
        app.tab = Tab::Sessions;
        app.session_focus = SessionFocus::Chat;
        assert!(app.sessions.is_empty(), "precondition: no session to chat with");

        let action = handle_key(&mut app, press('q'));
        assert!(
            matches!(action, Action::Quit),
            "q must still quit, not be swallowed in empty-list chat focus"
        );
        assert!(
            matches!(app.session_focus, SessionFocus::List),
            "focus must recover to the session list, not stay stuck in chat"
        );
    }
}
