use crate::app::{App, InfoSection, InputMode, MenuAction, MonitorAction, SessionEntry, SessionFocus, SessionRow, SettingsSection, Tab};
use omega_core::session::SessionRole;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_tabs(frame, app, chunks[0]);

    match app.tab {
        Tab::Sessions => draw_sessions(frame, app, chunks[1]),
        Tab::Menu => draw_menu(frame, app, chunks[1]),
        Tab::Monitor => draw_monitor(frame, app, chunks[1]),
        Tab::Projects => draw_projects(frame, app, chunks[1]),
        Tab::Settings => draw_settings(frame, app, chunks[1]),
        Tab::Agentic => draw_info(frame, app, chunks[1]),
        Tab::Help => draw_help(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Overlay modals — drawn LAST so they paint on top of everything
    if let InputMode::NewSessionAgent(ref name) = app.input_mode {
        draw_agent_picker(frame, app, name);
    }
    match app.input_mode {
        InputMode::TelegramSetupToken
        | InputMode::TelegramSetupChatId(_)
        | InputMode::TelegramSetupUserId(_, _) => {
            draw_telegram_setup_modal(frame, app);
        }
        InputMode::RenameSession(_) => draw_simple_input_modal(
            frame,
            app,
            "Rename session",
            "New session name (Enter to confirm, Esc to cancel)",
            false,
        ),
        InputMode::NewSession => draw_simple_input_modal(
            frame,
            app,
            "New session",
            "Session name (Enter to confirm, Esc to cancel)",
            false,
        ),
        InputMode::NewNamedSession(ref agent) => {
            let title = format!("New {} session", agent);
            let hint = format!("Session name (Enter to launch, Esc to cancel)");
            draw_simple_input_modal_owned(frame, app, &title, &hint, false);
        }
        InputMode::DispatchProject => draw_simple_input_modal(
            frame,
            app,
            "Dispatch oracle — step 1/2",
            "Project name (Enter to continue, Esc to cancel)",
            false,
        ),
        InputMode::DispatchMission(ref p) => {
            let title = format!("Dispatch oracle — step 2/2");
            let hint = format!("Mission for project '{}' (Enter to dispatch, Esc to cancel)", p);
            draw_simple_input_modal_owned(frame, app, &title, &hint, false);
        }
        InputMode::EditSettingsField { ref config_key, masked } => {
            let title = format!("Edit setting: {}", config_key);
            let hint = "Type new value, Enter to save, Esc to cancel".to_string();
            draw_simple_input_modal_owned(frame, app, &title, &hint, masked);
        }
        _ => {}
    }
}

/// Centered overlay modal for the 3-step Telegram setup wizard.
fn draw_telegram_setup_modal(frame: &mut Frame, app: &App) {
    let (step_num, step_label, hint, masked, value) = match &app.input_mode {
        InputMode::TelegramSetupToken => (
            1,
            "BOT_TOKEN",
            "Paste the bot token from @BotFather. Bracketed paste handles long tokens — no need to type.",
            true,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupChatId(_) => (
            2,
            "CHAT_ID",
            "Numeric chat id. Get yours by sending /start to @userinfobot on Telegram.",
            false,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupUserId(_, chat) => (
            3,
            "ALLOWED user_ids (optional)",
            &*Box::leak(format!(
                "Comma-separated user_ids allowed to talk to the bot (chat_id={}). Esc to skip.",
                chat
            ).into_boxed_str()),
            false,
            app.input_buffer.clone(),
        ),
        _ => return,
    };

    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let display = if masked { mask_inline(&value) } else { value };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Telegram Setup ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("· Step {}/3", step_num),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("{}: ", step_label),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(Color::Yellow)),
            Span::styled(display, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "    The whole flow is inline — no shell required.",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Telegram Setup ")
        .border_style(Style::default().fg(Color::Yellow));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Centered overlay modal for a single-line text input (rename, new session, etc.).
fn draw_simple_input_modal(frame: &mut Frame, app: &App, title: &str, hint: &str, masked: bool) {
    draw_simple_input_modal_owned(frame, app, title, hint, masked);
}

fn draw_simple_input_modal_owned(
    frame: &mut Frame,
    app: &App,
    title: &str,
    hint: &str,
    masked: bool,
) {
    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);

    let display = if masked {
        mask_inline(&app.input_buffer)
    } else {
        app.input_buffer.clone()
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(Color::Yellow)),
            Span::styled(display, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(Color::Yellow));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_agent_picker(frame: &mut Frame, app: &App, session_name: &str) {
    let area = centered_rect(50, 70, frame.area());

    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = omega_core::agents::Agent::all()
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let selected = i == app.agent_picker_index;
            let prefix = if selected { "▶ " } else { "  " };
            let availability = if agent.is_available() {
                Span::styled(" ✓ ", Style::default().fg(Color::Green))
            } else {
                Span::styled(" ✗ ", Style::default().fg(Color::Red))
            };
            let label_style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                availability,
                Span::styled(
                    format!(" {:8}  ", agent.name()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(agent.display_name(), label_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Choose agent for [{}] — ↑/↓, Enter, Esc ", session_name))
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    let titles = vec!["Sessions", "Menu", "Monitor", "Projects", "Settings", "Agentic", "Help"];
    let selected = match app.tab {
        Tab::Sessions => 0,
        Tab::Menu => 1,
        Tab::Monitor => 2,
        Tab::Projects => 3,
        Tab::Settings => 4,
        Tab::Agentic => 5,
        Tab::Help => 6,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" OmegaOS "),
        )
        .select(selected)
        // All tabs use the same color when inactive; active tab is cyan+bold.
        // No background fill — keeps the toolbar visually clean and avoids
        // any single tab looking "different" beyond just being highlighted.
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

fn draw_sessions(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_focused = app.session_focus == SessionFocus::List;
    let chat_focused = matches!(
        app.session_focus,
        SessionFocus::Chat | SessionFocus::ChatFullscreen
    );
    let fullscreen = app.session_focus == SessionFocus::ChatFullscreen;

    // Fullscreen: chat takes the entire area, no session list rendered
    let split = if fullscreen {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area)
    };

    // Skip rendering the left list in fullscreen mode
    if fullscreen {
        draw_sessions_right(frame, app, split[1], chat_focused);
        return;
    }
    let _ = list_focused;

    // ── Left: session list with project headers ─────────────────────────────
    let mut entry_idx: usize = 0;
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            SessionRow::Header(label) => ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", label),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            SessionRow::Entry(entry) => {
                let item = render_session_item(entry, entry_idx == app.selected && list_focused);
                entry_idx += 1;
                item
            }
        })
        .collect();

    let list_border_style = if list_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Map app.selected (entry index) to rendered row index (includes headers)
    let rendered_selected = {
        let mut eidx: usize = 0;
        let mut result: Option<usize> = None;
        for (row_idx, row) in app.rows.iter().enumerate() {
            if let SessionRow::Entry(_) = row {
                if eidx == app.selected { result = Some(row_idx); break; }
                eidx += 1;
            }
        }
        result
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sessions ({}) ", app.sessions.len()))
                .border_style(list_border_style),
        )
        .highlight_style(Style::default());

    let mut state = ListState::default().with_selected(rendered_selected);
    frame.render_stateful_widget(list, split[0], &mut state);

    draw_sessions_right(frame, app, split[1], chat_focused);
}

/// Render the right column of the Sessions tab (preview + optional chat input).
/// Used both in split layout and chat-fullscreen mode.
fn draw_sessions_right(frame: &mut Frame, app: &mut App, area: Rect, chat_focused: bool) {
    let fullscreen = app.session_focus == SessionFocus::ChatFullscreen;

    let preview_title = match app.selected_session() {
        Some(e) => {
            let suffix = if fullscreen { "  [FULLSCREEN — Tab-Tab to exit]" } else { "" };
            format!(" {}{} ", e.session.name, suffix)
        }
        None => " Preview ".to_string(),
    };

    let preview_border_style = if chat_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let total_lines = app.preview_content.lines().count() as u16;
    let viewport_height = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(viewport_height);
    // Publish the real clamp bound so the scroll setters (which don't know the
    // panel geometry) can stop at the top of history.
    app.preview_max_scroll = max_scroll;
    // `preview_scroll` is measured from the TAIL; the Paragraph wants a
    // from-top offset. Clamp the from-tail value, then convert.
    let from_tail = app.preview_scroll.min(max_scroll);
    let scroll = max_scroll.saturating_sub(from_tail);

    let preview_lines: Vec<Line> = if app.preview_content.is_empty() {
        vec![Line::from(Span::styled(
            "(select a session to preview)",
            Style::default().fg(Color::Gray),
        ))]
    } else {
        app.preview_content
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect()
    };

    let scroll_indicator = if max_scroll > 0 {
        format!(" [{}/{}] ", scroll, max_scroll)
    } else {
        String::new()
    };

    // No buffer-based chat-input. Keystrokes are forwarded LIVE to the rmux
    // session when chat_focused (see input.rs::handle_key_chat). The title
    // signals interactive mode so the user knows their keys are routed.
    let title = if chat_focused {
        format!(
            "{}{}  [INTERACTIVE — keys → session  ·  Tab cycle  ·  Alt+↑↓ scroll]",
            preview_title, scroll_indicator
        )
    } else {
        format!("{}{}  [Enter = type into session]", preview_title, scroll_indicator)
    };
    let preview = Paragraph::new(preview_lines)
        .scroll((scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(preview_border_style),
        );
    frame.render_widget(preview, area);

    // Visible cursor when chat is focused. Two layers (belt + suspenders):
    //   1. A painted yellow block glyph in the buffer — always visible
    //      regardless of terminal cursor settings.
    //   2. The OS-native blinking caret via set_cursor_position.
    // Positioned at the typing point: end of the last non-empty visible
    // line of the pane capture.
    if chat_focused {
        let inner_w = area.width.saturating_sub(2) as usize;
        let inner_h = area.height.saturating_sub(2);
        let all_lines: Vec<&str> = app.preview_content.lines().collect();
        let (last_idx, last_line) = all_lines
            .iter()
            .enumerate()
            .rev()
            .find(|(_, l)| !l.trim().is_empty())
            .map(|(i, l)| (i as u16, *l))
            .unwrap_or((0, ""));
        let viewport_row = last_idx.saturating_sub(scroll);
        if viewport_row < inner_h {
            let col_chars = (last_line.chars().count().min(inner_w.saturating_sub(1))) as u16;
            let cursor_x = area.x + 1 + col_chars;
            let cursor_y = area.y + 1 + viewport_row;
            // Layer 1: paint a block glyph into the buffer cell.
            let buf = frame.buffer_mut();
            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_symbol("▏");
                    cell.set_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::RAPID_BLINK));
                }
            }
            // Layer 2: OS-native caret.
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
    let _ = fullscreen;
}

fn render_session_item(entry: &SessionEntry, selected: bool) -> ListItem<'static> {
    let is_master = omega_core::aisb::is_master(&entry.session.name);

    let icon = if is_master {
        "★"
    } else {
        match entry.session.role {
            SessionRole::Oracle => "◆",
            SessionRole::Worker => "●",
            SessionRole::Home => "⌂",
            SessionRole::System => "⚙",
        }
    };

    let icon_color = if is_master {
        Color::Magenta
    } else {
        match entry.session.role {
            SessionRole::Oracle => Color::Yellow,
            SessionRole::Worker => Color::Green,
            SessionRole::Home => Color::Blue,
            SessionRole::System => Color::Gray,
        }
    };

    let progress_str = match &entry.progress {
        Some(p) => format!(" {} {:.0}%", p.bar(8), p.percentage()),
        None => String::new(),
    };

    let prefix = if selected {
        "▶ ".to_string()
    } else {
        "  ".to_string()
    };

    let name_style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if is_master {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        // Brighter than the previous default — clearly visible on dark terminals
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };

    let protect_marker = if entry.is_protected { "§ " } else { "" };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Cyan)),
        Span::raw(entry.tree_prefix.clone()),
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(icon_color),
        ),
        Span::styled(protect_marker, Style::default().fg(Color::Magenta)),
        Span::styled(entry.session.name.clone(), name_style),
        Span::styled(progress_str, Style::default().fg(Color::Cyan)),
    ]);

    ListItem::new(line)
}

fn menu_group(action: &MenuAction) -> &'static str {
    match action {
        MenuAction::NewClaude
        | MenuAction::NewCodex
        | MenuAction::NewGemini
        | MenuAction::NewPi
        | MenuAction::NewHermes
        | MenuAction::NewGlm => "New agent sessions",
        MenuAction::NewTerminal => "Terminal",
        MenuAction::DispatchOracle => "Orchestration",
        MenuAction::Refresh | MenuAction::ToggleProtection | MenuAction::KillSelected => "Session actions",
        MenuAction::Quit => "Quit",
    }
}

fn draw_menu(frame: &mut Frame, app: &mut App, area: Rect) {
    // Build items with section headers so the menu reads as grouped sections.
    let mut items: Vec<ListItem> = Vec::new();
    let mut last_group: Option<&'static str> = None;
    for (i, action) in MenuAction::all().iter().enumerate() {
        let group = menu_group(action);
        if last_group != Some(group) {
            if last_group.is_some() {
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  ─── {} ───", group),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))));
            last_group = Some(group);
        }
        let selected = i == app.menu_selected;
        let prefix = if selected { "▶ " } else { "  " };
        let label_style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
        ])));
    }

    // Compute rendered row index for the selected action (accounting for
    // header rows + blank separator rows between groups).
    let rendered_selected = {
        let mut idx: usize = 0;
        let mut last: Option<&'static str> = None;
        for (i, action) in MenuAction::all().iter().enumerate() {
            let g = menu_group(action);
            if last != Some(g) {
                if last.is_some() { idx += 1; } // blank line
                idx += 1; // header line
                last = Some(g);
            }
            if i == app.menu_selected { break; }
            idx += 1;
        }
        idx
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Actions — ↑/↓ navigate, Enter runs the highlighted action ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default()); // selection visual is already baked into items

    let mut state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_monitor(frame: &mut Frame, app: &mut App, area: Rect) {
    use omega_core::monitor;

    let snap = monitor::UsageSnapshot::read().ok().flatten();
    let cache_age = monitor::UsageSnapshot::cache_age_secs();
    let bot_status = monitor::aisb_bot_status();
    let accounts = monitor::list_accounts();
    let tg_config = monitor::OmegaTelegramConfig::read();

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  AISB Monitor — Claude Code billing, accounts, bots",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // ── Billing ─────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "  ── Billing (live) ──",
        Style::default().fg(Color::Yellow),
    )));
    if let Some(snap) = &snap {
        let cache_label = match cache_age {
            Some(s) if s < 60 => format!("{}s ago", s),
            Some(s) if s < 3600 => format!("{}m ago", s / 60),
            Some(s) => format!("{}h ago", s / 3600),
            None => "?".to_string(),
        };
        lines.push(Line::from(format!(
            "    Account:        {}  ({})",
            if !snap.active_account.is_empty() {
                snap.active_account.as_str()
            } else {
                "—"
            },
            if !snap.email.is_empty() { snap.email.as_str() } else { "—" }
        )));
        lines.push(Line::from(format!(
            "    Source:         {}    Cache: {}",
            snap.source, cache_label
        )));
        lines.push(Line::from(""));

        // Progress bars
        for (label, pct, tokens, budget) in [
            ("5h session", snap.precise_5h(), snap.tokens_5h, snap.budget_5h),
            ("Week",       snap.precise_week(), snap.tokens_7d, snap.budget_week),
        ] {
            lines.push(Line::from(vec![
                Span::raw(format!("    {:11} ", label)),
                Span::styled(
                    render_bar(pct, 30),
                    Style::default().fg(pct_color(pct)),
                ),
                Span::raw(format!(" {:5.1}%  ", pct)),
                Span::styled(
                    format!("{} / {} tok", short_num(tokens), short_num(budget)),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            "    Sonnet:         {}%        Extra: {}%",
            snap.sonnet_pct, snap.extra_pct
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    (no /tmp/aisb-usage.json — usage-monitor cron not running)",
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(""));

    // ── AISB Telegram Bot (legacy, Python, on the VPS) ──────────────────────
    lines.push(Line::from(Span::styled(
        "  ── AISB Telegram Bot (legacy Python bot, separate from OmegaOS) ──",
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::styled(
        "    The original AISB bot running on this VPS — not part of OmegaOS.",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::styled(
        "    We just READ its usage cache (/tmp/aisb-usage.json) for the billing view.",
        Style::default().fg(Color::Gray),
    )));
    let (bot_icon, bot_color, bot_text) = if bot_status.bot_alive {
        ("●", Color::Green, "running")
    } else {
        ("○", Color::Red, "not detected")
    };
    lines.push(Line::from(vec![
        Span::raw("    Process status: "),
        Span::styled(format!("{} {}", bot_icon, bot_text), Style::default().fg(bot_color)),
    ]));
    let cache_text = match bot_status.cache_status {
        monitor::CacheStatus::Fresh(s) => format!("fresh ({}s ago)", s),
        monitor::CacheStatus::Stale(s) => format!("stale ({}s ago)", s),
        monitor::CacheStatus::Missing => "missing".to_string(),
    };
    lines.push(Line::from(format!("    Usage cache:    {}", cache_text)));

    lines.push(Line::from(""));

    // ── Accounts ────────────────────────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        "  ── Claude Accounts (~/.claude/accounts) ──",
        Style::default().fg(Color::Yellow),
    )));
    if accounts.is_empty() {
        lines.push(Line::from(Span::styled(
            "    (no saved accounts)",
            Style::default().fg(Color::Gray),
        )));
    } else {
        for acc in &accounts {
            let marker = if acc.is_active { "▶" } else { " " };
            let color = if acc.is_active { Color::Green } else { Color::White };
            lines.push(Line::from(vec![
                Span::styled(format!("    {} ", marker), Style::default().fg(Color::Cyan)),
                Span::styled(acc.label.clone(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(format!("   {}", acc.email.as_deref().unwrap_or(""))),
            ]));
        }
    }

    lines.push(Line::from(""));

    // ── Omega Telegram Bot (Rust, this system) ──────────────────────────────
    lines.push(Line::from(Span::styled(
        "  ── Omega Telegram Bot (Rust — talk to AISB Master from anywhere) ──",
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::styled(
        "    What this is:",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(Span::styled(
        "      Omega's OWN Telegram bot (no Python, no AISB-Python dependency).",
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(Span::styled(
        "      Once configured, any text you send via Telegram is relayed to the",
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(Span::styled(
        "      AISB Master session (aisb-master) — the AI Super Brain.",
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(Span::styled(
        "      → you talk to all 13 Matrix agents through one chat, from your phone.",
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));
    if let Some(cfg) = tg_config {
        let state = if cfg.enabled { "enabled" } else { "configured (disabled)" };
        let color = if cfg.enabled { Color::Green } else { Color::Yellow };
        lines.push(Line::from(vec![
            Span::raw("    Status:         "),
            Span::styled(state.to_string(), Style::default().fg(color)),
        ]));
        if !cfg.label.is_empty() {
            lines.push(Line::from(format!("    Label:          {}", cfg.label)));
        }
        lines.push(Line::from(format!(
            "    Relay session:  {}    (where messages go)",
            cfg.relay_session
        )));
        lines.push(Line::from(format!("    Chat ID:        {}", cfg.chat_id)));
        let sender = if cfg.allow_user_ids.is_empty() {
            "chat_id only (any user in this chat)".to_string()
        } else {
            format!("user_ids {:?}", cfg.allow_user_ids)
        };
        lines.push(Line::from(format!("    Sender filter:  {}", sender)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    Run the bot:    omega telegram run",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    Status:         (not configured)",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    Setup:",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "      1. Create a bot via @BotFather on Telegram → save the BOT_TOKEN",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            "      2. Get your CHAT_ID (e.g. send a msg to @userinfobot)",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            "      3. Run:",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            "         omega telegram setup <BOT_TOKEN> <CHAT_ID> --user-id <YOUR_USER_ID>",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(Span::styled(
            "      4. Start the bot:",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            "         omega telegram run",
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));

    // ── Actions (arrow-navigable + letter shortcuts) ───────────────────────
    lines.push(Line::from(Span::styled(
        "  ── Actions  (↑/↓ navigate, Enter to run, or press letter) ──",
        Style::default().fg(Color::Yellow),
    )));
    let monitor_actions_start_line = lines.len();
    for (i, action) in MonitorAction::all().iter().enumerate() {
        let selected = i == app.monitor_selected;
        let prefix = if selected { "  ▶ " } else { "    " };
        let label_style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  This tab refreshes every 5s. Use ↑/↓ + Enter or the letter shortcut.",
        Style::default().fg(Color::Gray),
    )));

    // Auto-scroll to keep selected monitor action visible
    if !app.detail_focused {
        let action_line = (monitor_actions_start_line + app.monitor_selected) as u16;
        let panel_h = area.height.saturating_sub(2);
        if action_line < app.detail_scroll {
            app.detail_scroll = action_line.saturating_sub(1);
        } else if action_line >= app.detail_scroll + panel_h {
            app.detail_scroll = action_line.saturating_sub(panel_h.saturating_sub(2));
        }
    }

    let title = if app.detail_fullscreen {
        " Monitor  [FULLSCREEN — Tab/Tab-Tab to exit] ".to_string()
    } else if app.detail_focused {
        " Monitor  [↑/↓ scroll, Tab back to actions] ".to_string()
    } else {
        " Monitor  [↑/↓ select action, Enter to run, Tab to scroll] ".to_string()
    };
    let border = if app.detail_focused || app.detail_fullscreen {
        Color::Yellow
    } else {
        Color::Cyan
    };
    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border)),
        );
    frame.render_widget(paragraph, area);
}

fn render_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn pct_color(pct: f32) -> Color {
    if pct < 50.0 { Color::Green }
    else if pct < 80.0 { Color::Yellow }
    else { Color::Red }
}

/// Wrap a string into visual lines. Honours embedded `\n` as hard breaks
/// (paste with newlines stays multi-line). Within each logical line we
/// soft-wrap on word boundaries; long tokens are hard-cut.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for logical in text.split('\n') {
        if logical.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in logical.split(' ').filter(|w| !w.is_empty()) {
            if word.chars().count() > width {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
                let mut buf = String::new();
                for c in word.chars() {
                    if buf.chars().count() + 1 > width {
                        out.push(std::mem::take(&mut buf));
                    }
                    buf.push(c);
                }
                current = buf;
                continue;
            }
            let needed = if current.is_empty() {
                word.chars().count()
            } else {
                current.chars().count() + 1 + word.chars().count()
            };
            if needed > width {
                out.push(std::mem::take(&mut current));
                current.push_str(word);
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
        }
        // Always emit a row per logical line (even empty/space-only ones)
        out.push(current);
    }
    out
}

fn visual_line_count(text: &str, width: usize) -> usize {
    let n = wrap_text(text, width).len();
    n.max(1)
}

fn short_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}G", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn draw_projects(frame: &mut Frame, app: &mut App, area: Rect) {
    let registry = &app.project_registry;

    // Fullscreen detail
    if app.detail_fullscreen {
        let lines = render_project_detail(app);
        let title = app
            .selected_project()
            .map(|p| format!(" {}  [FULLSCREEN — Tab/Tab-Tab to exit] ", p.name))
            .unwrap_or_else(|| " Projects  [FULLSCREEN] ".to_string());
        let paragraph = Paragraph::new(lines)
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { Color::Cyan } else { Color::Gray };
    let detail_border = if app.detail_focused { Color::Yellow } else { Color::Gray };

    // Left: project list
    let items: Vec<ListItem> = if registry.projects.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (no projects registered)",
            Style::default().fg(Color::Gray),
        )))]
    } else {
        registry
            .projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let selected = i == app.projects_selected && list_focused;
                let prefix = if i == app.projects_selected { "▶ " } else { "  " };
                let icon = project.icon.as_deref().unwrap_or("▣");
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if i == app.projects_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{} ", icon)),
                    Span::styled(project.name.clone(), style),
                ]))
            })
            .collect()
    };

    let list_title = if list_focused {
        format!(" ▶ FOCUSED Projects ({}) — ↑/↓ select, Tab → detail ", registry.projects.len())
    } else {
        format!(" Projects ({}) — Tab to focus list ", registry.projects.len())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut state = ListState::default().with_selected(Some(app.projects_selected));
    frame.render_stateful_widget(list, split[0], &mut state);

    // Right: project detail
    let lines = render_project_detail(app);
    let detail_title = if app.detail_focused {
        " Project Detail  [FOCUSED — ↑/↓ scroll, Tab → list] "
    } else {
        " Project Detail "
    };
    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(detail_title)
                .border_style(Style::default().fg(detail_border)),
        );
    frame.render_widget(paragraph, split[1]);
}

fn render_project_detail(app: &App) -> Vec<Line<'static>> {
    let Some(project) = app.selected_project() else {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No project selected.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Add projects via CLI:",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "    omega project add /path/to/project",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "    omega project scan ~/VibeCoding/work",
                Style::default().fg(Color::Yellow),
            )),
        ];
    };

    let icon = project.icon.as_deref().unwrap_or("▣");
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("  {} ", icon)),
            Span::styled(
                project.name.clone(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Path
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Path"),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(project.path.to_string_lossy().to_string()),
    ]));

    // Git email
    if let Some(ref email) = project.git_email {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Git email"),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(email.clone()),
        ]));
    }

    // Telegram topic
    if let Some(topic_id) = project.telegram_topic_id {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Telegram topic"),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(topic_id.to_string()),
        ]));
    }

    // Oracle session
    if let Some(ref oracle) = project.oracle_session {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Oracle session"),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(oracle.clone()),
        ]));
    }

    // Created at
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Created"),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(project.created_at.clone()),
    ]));

    lines.push(Line::from(""));

    // Planner status (if .planner/tracker.json exists)
    let tracker = omega_core::planner::PlanTracker::load(&project.path);
    if let Some(ref tracker) = tracker {
        let status = tracker.status();
        lines.push(Line::from(Span::styled(
            "  ── Planner ──",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "    Phase {}/{} — {:.0}% complete ({}/{} steps)",
            status.active_phase,
            status.total_phases,
            status.progress_pct(),
            status.done,
            status.total,
        )));
        if status.in_progress > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} in progress", status.in_progress),
                Style::default().fg(Color::Yellow),
            )));
        }
        if status.failed > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} failed", status.failed),
                Style::default().fg(Color::Red),
            )));
        }
        if status.ready > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} ready to start", status.ready),
                Style::default().fg(Color::Green),
            )));
        }

        // Show phases
        lines.push(Line::from(""));
        for phase in &tracker.phases {
            let phase_done = phase.step_ids.iter().all(|sid| {
                tracker
                    .get_step(sid)
                    .map(|s| s.status == omega_core::planner::StepStatus::Done)
                    .unwrap_or(false)
            });
            let phase_icon = if phase_done { "✅" } else if phase.id == status.active_phase { "⏳" } else { "⬚" };
            lines.push(Line::from(format!(
                "    {} Phase {}: {}",
                phase_icon, phase.id, phase.name
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  No planner active — run /planner to create a plan.",
            Style::default().fg(Color::Gray),
        )));
    }

    // Bootstrap status
    let bootstrap = omega_core::bootstrap::BootstrapState::load(&project.path);
    if let Some(ref state) = bootstrap {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Bootstrap Pipeline ──",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        for phase in omega_core::bootstrap::BootstrapPhase::all() {
            let done = state.is_done(*phase);
            let active = *phase == state.current_phase && !done;
            let icon = if done {
                "✅"
            } else if active {
                "⏳"
            } else {
                "⬚"
            };
            lines.push(Line::from(format!(
                "    {} {} {}",
                icon,
                phase.icon(),
                phase.label()
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ── Actions ──",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "    [d] Dispatch oracle    [p] Run planner    [Enter on detail] Open in terminal",
        Style::default().fg(Color::Cyan),
    )));

    lines
}

fn draw_settings(frame: &mut Frame, app: &mut App, area: Rect) {
    let providers = app.providers();
    let (lines, selected_field_line) = render_settings_detail(app, &providers);

    // Auto-scroll to keep the selected field visible
    if app.detail_focused {
        let panel_height = if app.detail_fullscreen {
            area.height.saturating_sub(2) // borders
        } else {
            // 75% of area (detail panel) minus borders
            (area.width as f32 * 0.75) as u16; // dummy, real height is area.height
            area.height.saturating_sub(2)
        };
        let field_line = selected_field_line as u16;
        if field_line < app.detail_scroll {
            app.detail_scroll = field_line.saturating_sub(1);
        } else if field_line >= app.detail_scroll + panel_height {
            app.detail_scroll = field_line.saturating_sub(panel_height.saturating_sub(2));
        }
    }

    // Fullscreen detail mode: skip the left list, detail takes 100% width
    if app.detail_fullscreen {
        let title = format!(
            " {}  [FULLSCREEN — Tab/Tab-Tab to exit] ",
            app.selected_settings_section().label()
        );
        let paragraph = Paragraph::new(lines)
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { Color::Cyan } else { Color::Gray };
    let detail_border = if app.detail_focused { Color::Yellow } else { Color::Gray };

    // ── Left: section list ──────────────────────────────────────────────────
    let items: Vec<ListItem> = SettingsSection::all()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let selected = i == app.settings_selected && list_focused;
            let prefix = if i == app.settings_selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.settings_selected {
                // Selected but not focused — dim highlight
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(section.label(), style),
            ]))
        })
        .collect();

    let list_title = if list_focused {
        " ▶ FOCUSED Settings — ↑/↓ select, Tab → focus detail "
    } else {
        " Settings — Tab to focus list "
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut settings_list_state = ListState::default().with_selected(Some(app.settings_selected));
    frame.render_stateful_widget(list, split[0], &mut settings_list_state);

    // ── Right: details panel ────────────────────────────────────────────────
    let detail_title = if app.detail_focused {
        format!(" {}  [FOCUSED — ↑/↓ scroll, Tab → list, Tab-Tab → fullscreen] ",
            app.selected_settings_section().label())
    } else {
        format!(" {} ", app.selected_settings_section().label())
    };

    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(detail_title)
                .border_style(Style::default().fg(detail_border)),
        );
    frame.render_widget(paragraph, split[1]);
}

/// Returns (lines, selected_field_line) — the line index where the selected field starts.
fn render_settings_detail(
    app: &App,
    providers: &omega_core::providers::ProvidersConfig,
) -> (Vec<Line<'static>>, usize) {
    use crate::app::{fields_for_section, SettingsField};
    let fields = fields_for_section(app.selected_settings_section(), providers, &app.config);
    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut selected_line: usize = 0;

    let detail_active = app.detail_focused;

    // Top hint
    if detail_active {
        lines.push(Line::from(Span::styled(
            "  ↑/↓ navigate · Enter activates · Tab → back to list",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab → focus this panel to interact (Install/Uninstall/Edit)",
            Style::default().fg(Color::Gray),
        )));
    }
    lines.push(Line::from(""));

    for (i, field) in fields.iter().enumerate() {
        let is_selected = detail_active && i == app.settings_field_selected;
        if is_selected { selected_line = lines.len(); }
        let prefix = if is_selected { "  ▶ " } else { "    " };
        match field {
            SettingsField::Action { label, command, .. } => {
                let label_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(label.clone(), label_style),
                ]));
                if is_selected {
                    let cmd_preview = if command.len() > 100 {
                        format!("{}…", &command[..100])
                    } else {
                        command.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("      → Enter runs:  {}", cmd_preview),
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
            SettingsField::EditText {
                label,
                current_value,
                masked,
                ..
            } => {
                let display = if current_value.is_empty() {
                    "(not set)".to_string()
                } else if *masked {
                    mask_key(current_value)
                } else {
                    current_value.clone()
                };
                let label_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(display, Style::default().fg(Color::Cyan)),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to edit (opens input modal)",
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
            SettingsField::Toggle { label, current, .. } => {
                let badge = if *current { "● on" } else { "○ off" };
                let badge_color = if *current { Color::Green } else { Color::Gray };
                let label_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(badge.to_string(), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to toggle",
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
            SettingsField::Info(text) => {
                if text.is_empty() {
                    lines.push(Line::from(""));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Config files: ~/.omega/config.toml  ~/.omega/providers.toml",
        Style::default().fg(Color::Gray),
    )));
    return (lines, selected_line);
}

fn kv(key: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:30}", key),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(value.to_string()),
    ])
}

fn default_or<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.is_empty() { default } else { value }
}

/// Mask sensitive characters in an inline input (show prefix/suffix only).
fn mask_inline(s: &str) -> String {
    if s.len() <= 6 {
        "•".repeat(s.len())
    } else {
        format!("{}…{}", &s[..3], "•".repeat(s.len() - 3))
    }
}

fn mask_key(key: &str) -> String {
    if key.is_empty() {
        "(not set)".to_string()
    } else if key.len() <= 8 {
        "•".repeat(key.len())
    } else {
        format!("{}…{}", &key[..4], &key[key.len() - 4..])
    }
}

fn availability(lines: &mut Vec<Line<'static>>, agent: omega_core::agents::Agent) {
    let (icon, color, label) = if agent.is_available() {
        ("✓", Color::Green, "installed")
    } else {
        ("✗", Color::Red, "not installed")
    };
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:30}", "CLI availability"), Style::default().fg(Color::Cyan)),
        Span::styled(format!("{} {}", icon, label), Style::default().fg(color)),
    ]));
}

fn draw_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let (lines, scroll_target) = match app.selected_info_section() {
        InfoSection::AisbAgents => render_info_aisb_agents(app),
        InfoSection::Oracle => { let l = render_info_oracle(); (l, 0) }
        InfoSection::Workers => { let l = render_info_workers(); (l, 0) }
        InfoSection::Rules => { let l = render_info_rules(); (l, 0) }
    };

    // Auto-scroll to keep selected item visible (agents list, etc.)
    if app.detail_focused && scroll_target > 0 {
        let panel_h = area.height.saturating_sub(2);
        let target = scroll_target as u16;
        if target < app.detail_scroll {
            app.detail_scroll = target.saturating_sub(1);
        } else if target >= app.detail_scroll + panel_h {
            app.detail_scroll = target.saturating_sub(panel_h.saturating_sub(2));
        }
    }

    // Fullscreen detail
    if app.detail_fullscreen {
        let title = format!(
            " {}  [FULLSCREEN — Tab/Tab-Tab to exit] ",
            app.selected_info_section().label()
        );
        let paragraph = Paragraph::new(lines)
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { Color::Cyan } else { Color::Gray };
    let detail_border = if app.detail_focused { Color::Yellow } else { Color::Gray };

    let items: Vec<ListItem> = InfoSection::all()
        .iter()
        .enumerate()
        .map(|(i, sec)| {
            let selected = i == app.info_section_selected && list_focused;
            let prefix = if i == app.info_section_selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.info_section_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(sec.label(), style),
            ]))
        })
        .collect();

    let list_title = if list_focused {
        " ▶ FOCUSED Agentic — ↑/↓ select, Tab → focus detail "
    } else {
        " Agentic — Tab to focus list "
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut info_list_state = ListState::default().with_selected(Some(app.info_section_selected));
    frame.render_stateful_widget(list, split[0], &mut info_list_state);

    let detail_title = if app.detail_focused {
        format!(" {}  [FOCUSED — ↑/↓ scroll, Tab → list, Tab-Tab → fullscreen] ",
            app.selected_info_section().label())
    } else {
        format!(" {} ", app.selected_info_section().label())
    };

    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(detail_title)
                .border_style(Style::default().fg(detail_border)),
        );
    frame.render_widget(paragraph, split[1]);
}

/// Returns (lines, selected_agent_line) for auto-scroll.
fn render_info_aisb_agents(app: &App) -> (Vec<Line<'static>>, usize) {
    use omega_core::aisb_agents::AisbAgent;
    let agents = AisbAgent::all();
    let selected_def = agents[app.info_agent_selected].definition();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  AISB = AI Super Brain — 13 Matrix agents the Master delegates to.",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  ↑/↓ navigate the agent list below.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    let mut selected_agent_line: usize = 4;
    // Compact list
    for (i, agent) in agents.iter().enumerate() {
        let def = agent.definition();
        let selected = i == app.info_agent_selected;
        if selected { selected_agent_line = lines.len(); }
        let prefix = if selected { "▶ " } else { "  " };
        let name_style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:13}", def.name), name_style),
            Span::styled(
                format!(" {} ", def.model.name()),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw(format!("· {}", def.role)),
        ]));
    }

    // Detail card for the selected agent
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  ── {} ──", selected_def.name),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  \"{}\"", selected_def.tagline),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(format!("  Role:    {}", selected_def.role)));
    lines.push(Line::from(format!("  Model:   {}", selected_def.model.name())));
    lines.push(Line::from(format!(
        "  Tools:   {}",
        selected_def.tools.join(", ")
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Responsibilities:",
        Style::default().fg(Color::Yellow),
    )));
    for r in selected_def.responsibilities {
        lines.push(Line::from(format!("    • {}", r)));
    }
    (lines, selected_agent_line)
}

fn render_info_oracle() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ORACLE — the brain of every dispatched mission",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  An Oracle is spawned by `omega dispatch <Project> \"<mission>\"`."),
        Line::from("  Its job is to:"),
        Line::from("    1. CLASSIFY the mission's complexity (SIMPLE / MEDIUM / COMPLEX / EPIC)"),
        Line::from("    2. PLAN — if COMPLEX or EPIC, KEYMAKER decomposes into a DAG"),
        Line::from("    3. DISPATCH workers via rmux sessions (1 per task)"),
        Line::from("    4. MONITOR — wait for each worker's done.json"),
        Line::from("    5. VERIFY — run the quality gate (rubric + multi-grader + adversarial)"),
        Line::from("    6. REPORT — write its own done.json with the outcome summary"),
        Line::from(""),
        Line::from(Span::styled(
            "  Naming",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("    Sessions: oracle-<Project>     (1st)"),
        Line::from("              oracle-<Project>-2   (parallel oracle)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Rules enforced",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("    R-19 — Rubric before execution"),
        Line::from("    R-21 — Multi-grader consensus ≥ 2/3"),
        Line::from("    R-28 — Token budget enforced (default 500K)"),
        Line::from("    L3   — Workers must decide, not wait"),
        Line::from(""),
        Line::from(Span::styled(
            "  ORACLES NEVER write code — they decide who does, then verify.",
            Style::default().fg(Color::Gray),
        )),
    ]
}

fn render_info_workers() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  WORKERS — ephemeral execution sessions",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  A worker is one rmux session running an agent (usually Claude) with"),
        Line::from("  a specific task prompt. Workers are short-lived: spawned by an Oracle,"),
        Line::from("  they execute one task, signal done, and the patrol cleans up."),
        Line::from(""),
        Line::from(Span::styled(
            "  Naming",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("    <Project>-worker-<task>    e.g. Causio-worker-auth"),
        Line::from(""),
        Line::from(Span::styled(
            "  Lifecycle",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("    1. Oracle spawns:  omega spawn-worker auth \"<prompt>\" --project Causio"),
        Line::from("    2. Scope-claim:    files_owned locked in ~/.omega/state/scope-*.json"),
        Line::from("    3. Worker runs:    Claude (or other agent) executes the task"),
        Line::from("    4. Worker reports: omega done <session> done_clean \"<summary>\""),
        Line::from("    5. Patrol acks:    omega patrol --once releases the scope claim"),
        Line::from("    6. Quality gate:   Oracle's rubric is graded against the result"),
        Line::from(""),
        Line::from(Span::styled(
            "  Rules enforced",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("    L3           — autonomy: decide, never wait"),
        Line::from("    SCOPE-CLAIM  — no two workers may edit the same file"),
        Line::from("    R-18         — long-running missions go here, short go to Agent tool"),
        Line::from(""),
        Line::from("  Workers can run in PARALLEL when their file scopes are disjoint."),
        Line::from("  When they overlap, the dispatcher serializes them automatically."),
    ]
}

fn render_info_rules() -> Vec<Line<'static>> {
    use omega_core::rules::{all_rules, RuleCategory};
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  System invariants — every rule has a reason, a date, and who it binds.",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let categories = [
        RuleCategory::Universal,
        RuleCategory::QualityGate,
        RuleCategory::Orchestration,
        RuleCategory::Reporting,
        RuleCategory::Safety,
    ];

    for cat in &categories {
        let rules: Vec<_> = all_rules().into_iter().filter(|r| r.category == *cat).collect();
        if rules.is_empty() {
            continue;
        }
        lines.push(Line::from(Span::styled(
            format!("  ── {} ──", cat.label()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        for r in rules {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:14}", r.id),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw(r.title.to_string()),
            ]));
            lines.push(Line::from(Span::styled(
                format!("    {}", r.description),
                Style::default().fg(Color::White),
            )));
            let applies = if r.applies_to.is_empty() {
                "all agents".to_string()
            } else {
                r.applies_to
                    .iter()
                    .map(|a| a.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            lines.push(Line::from(Span::styled(
                format!("    Applies to: {}  ·  Added: {}", applies, r.added_at),
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(Span::styled(
                format!("    Why: {}", r.reason),
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(""));
        }
    }
    lines
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let cy = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let yl = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let wh = Style::default().fg(Color::White);
    let gr = Style::default().fg(Color::Gray);
    let mg = Style::default().fg(Color::Magenta);

    let section = |title: &str| -> Line<'static> {
        Line::from(Span::styled(format!("  ─── {} ───", title), yl))
    };
    let key = |k: &str, desc: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("    {:22}", k), cy),
            Span::styled(desc.to_string(), wh),
        ])
    };

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Ω  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("OmegaOS", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  —  Agentic Terminal Operating System", gr),
            Span::styled(concat!("   v", env!("CARGO_PKG_VERSION")), cy),
        ]),
        Line::from(""),

        section("Navigation"),
        key("← / →", "Switch tabs"),
        key("Shift+Tab", "Previous tab"),
        key("Ctrl+L", "Redraw screen (fix corrupted view)"),
        key("Esc", "Back (detail → list → Sessions → quit)"),
        key("q", "Quit OmegaOS"),
        Line::from(""),

        section("Tab Behavior (Enter & Tab)"),
        Line::from(vec![
            Span::styled("    Tab", cy),
            Span::styled("  = focus right panel  ", wh),
            Span::styled("Tab-Tab", cy),
            Span::styled("  = fullscreen  ", wh),
            Span::styled("Esc", cy),
            Span::styled("  = back", wh),
        ]),
        Line::from(Span::styled("    Same pattern on Sessions, Settings, Info, Monitor.", gr)),
        Line::from(""),

        section("Sessions"),
        key("↑ / ↓  or  j / k", "Navigate sessions"),
        key("Enter / Tab", "Focus chat (talk to agent)"),
        key("Tab-Tab", "Chat fullscreen (hide list)"),
        key("r  /  R", "Rename selected session"),
        key("x  /  X", "Kill session (skip if locked)"),
        key(".", "Toggle lock/protection"),
        key("PgUp / PgDn", "Scroll preview"),
        key("Home / End", "Top / bottom (tail-follow)"),
        Line::from(""),

        section("Menu — Agent Launchers"),
    ];

    let launchers = [
        ("c", "Claude"), ("C", "Codex"), ("g", "Gemini"),
        ("p", "Pi"), ("h", "Hermes"), ("G", "GLM"), ("t", "Terminal"),
    ];
    let mut row = vec![Span::raw("    ")];
    for (i, (k, name)) in launchers.iter().enumerate() {
        row.push(Span::styled(format!("[{}]", k), yl));
        row.push(Span::styled(format!(" {:10}", name), wh));
        if (i + 1) % 4 == 0 {
            lines.push(Line::from(std::mem::take(&mut row)));
            row.push(Span::raw("    "));
        }
    }
    if row.len() > 1 { lines.push(Line::from(row)); }

    lines.extend([
        key("d", "Dispatch oracle → project + mission"),
        key("F5", "Refresh sessions (r/R = Rename, everywhere)"),
        Line::from(""),

        section("Monitor"),
        key("↑ / ↓ + Enter", "Run highlighted action"),
        key("L", "Login Claude (OAuth)"),
        key("T / D", "Telegram setup / disconnect"),
        key("B", "Refresh billing"),
        Line::from(""),

        section("Settings"),
        key("↑ / ↓", "Browse sections"),
        key("Enter / Tab", "Focus detail panel → edit fields"),
        key("Enter (on field)", "Activate (install/edit/toggle)"),
        Line::from(""),

        section("Projects"),
        key("↑ / ↓", "Browse projects"),
        key("Enter", "Focus detail; Enter again → open in terminal"),
        key("d", "Dispatch oracle to selected project"),
        key("p", "Run planner for selected project"),
        Line::from(""),

        section("Agentic"),
        key("↑ / ↓", "Navigate sub-sections (or AISB agents in detail)"),
        key("[  /  ]", "Previous / next sub-section"),
        Line::from(""),

        section("Chat (Sessions, when chat-focused)"),
        key("Tab", "Forward to Claude (cycle modes)"),
        key("Shift+Tab", "Return to session list"),
        key("Alt+↑ / Alt+↓", "Scroll preview"),
        key("Ctrl+W / Alt+Bksp", "Delete word backwards"),
        key("Shift+Del / Alt+Del", "Delete word forwards"),
        key("Opt+< / Opt+>", "Jump to start / end of input (M-< / M->)"),
        key("Mouse scroll", "Scroll the panel under the cursor"),
        Line::from(""),

        section("Integrated Tools"),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("Hermes  ", cy),
            Span::styled("Nous Research multi-agent coordinator", gr),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("Pi      ", cy),
            Span::styled("earendil-works coding agent (OpenRouter)", gr),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("PDF Gen ", cy),
            Span::styled("Whitepaper/audit/marketing reports → Telegram", gr),
        ]),
        Line::from(Span::styled("    Install: omega install hermes | omega install pi", mg)),
        Line::from(Span::styled("    Generate: omega pdf --template=whitepaper --demo --send", mg)),
        Line::from(""),

        section("CLI Commands"),
    ]);

    let cmds = [
        ("omega", "Launch TUI"),
        ("omega master", "Attach AISB Master"),
        ("omega list", "List sessions"),
        ("omega pdf --demo --send", "Generate + send PDF to Telegram"),
        ("omega orchestrate <P> <M>", "Full mission pipeline"),
        ("omega telegram setup …", "Bot setup"),
        ("omega config set …", "Provider config"),
        ("omega projects", "Auto-discover projects"),
        ("omega install <agent>", "Install an agent CLI"),
    ];
    for (cmd, desc) in cmds {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{:30}", cmd), mg),
            Span::styled(desc.to_string(), gr),
        ]));
    }

    lines.extend([
        Line::from(""),
        section("Status Icons"),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("★ ", cy), Span::styled("Master   ", gr),
            Span::styled("◆ ", cy), Span::styled("Oracle   ", gr),
            Span::styled("● ", cy), Span::styled("Worker   ", gr),
            Span::styled("⌂ ", cy), Span::styled("Home   ", gr),
            Span::styled("⚙ ", cy), Span::styled("System   ", gr),
            Span::styled("§ ", yl), Span::styled("Locked", gr),
        ]),
        Line::from(""),
    ]);

    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help — OmegaOS  (↑/↓ scroll) ")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    // Input mode: show a prompt line (no stats)
    if !matches!(app.input_mode, InputMode::Normal) {
        let (prompt, value) = match &app.input_mode {
            InputMode::Normal => unreachable!(),
            InputMode::NewSession => ("New session name", app.input_buffer.clone()),
            InputMode::NewNamedSession(agent) => (
                "Session name",
                format!("[{}] {}", agent, app.input_buffer),
            ),
            InputMode::NewSessionPromptDirect(name, agent) => (
                "Initial prompt (optional, Esc to skip)",
                format!("[{}/{}] {}", name, agent, app.input_buffer),
            ),
            InputMode::NewSessionAgent(name) => (
                "Choose agent",
                format!("[{}] (overlay open — ↑/↓)", name),
            ),
            InputMode::NewSessionPrompt(name, agent) => (
                "Initial prompt (optional)",
                format!("[{}/{}] {}", name, agent, app.input_buffer),
            ),
            InputMode::DispatchProject => ("Dispatch — project", app.input_buffer.clone()),
            InputMode::DispatchMission(p) => (
                "Dispatch — mission",
                format!("[{}] {}", p, app.input_buffer),
            ),
            InputMode::RenameSession(old) => (
                "Rename session",
                format!("[{} →] {}", old, app.input_buffer),
            ),
            InputMode::TelegramSetupToken => (
                "Telegram setup 1/3 — BOT_TOKEN",
                mask_inline(&app.input_buffer),
            ),
            InputMode::TelegramSetupChatId(_) => (
                "Telegram setup 2/3 — CHAT_ID (numeric)",
                app.input_buffer.clone(),
            ),
            InputMode::TelegramSetupUserId(_, chat) => (
                "Telegram setup 3/3 — user_id (Esc to skip)",
                format!("[chat={}] {}", chat, app.input_buffer),
            ),
            InputMode::EditSettingsField { config_key, masked } => (
                "Edit setting",
                if *masked {
                    format!("[{}] {}", config_key, mask_inline(&app.input_buffer))
                } else {
                    format!("[{}] {}", config_key, app.input_buffer)
                },
            ),
        };

        let status = Paragraph::new(Line::from(vec![
            Span::styled(" ▶ ", Style::default().fg(Color::Black).bg(Color::Yellow)),
            Span::styled(
                format!(" {}: ", prompt),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(value),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]));
        frame.render_widget(status, area);
        return;
    }

    // Normal mode: status bar with live system stats (UX inspired by tmux-claude,
    // re-implemented against the rmux SDK)
    let stats = omega_core::sysinfo::SystemStats::read();

    let cpu = format!("CPU {:.2}", stats.cpu_load);
    let ram = format!("RAM {}%", stats.ram_pct);
    let disk = format!("DSK {}%", stats.disk_used_pct);
    let n_sessions = format!("{} sess", app.sessions.len());

    let now = chrono::Local::now();
    let time_str = now.format("%H:%M").to_string();

    let session_info = app
        .selected_session()
        .map(|e| {
            let icon = match e.session.role {
                omega_core::session::SessionRole::Oracle => "◆",
                omega_core::session::SessionRole::Worker => "●",
                omega_core::session::SessionRole::Home => "⌂",
                omega_core::session::SessionRole::System => "⚙",
            };
            format!("{} {}", icon, e.session.name)
        })
        .unwrap_or_else(|| "—".to_string());

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(60)])
        .split(area);

    // Left side: Ω badge (no bg, bold) + selected session + status message
    let left = Paragraph::new(Line::from(vec![
        Span::styled(
            " Ω ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            session_info,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            app.status_message.as_deref().unwrap_or(""),
            Style::default().fg(Color::Gray),
        ),
    ]));
    frame.render_widget(left, split[0]);

    // Right side: system stats (n_sessions in BOLD white so it pops)
    let stat_color = |pct: u8| -> Color {
        match pct {
            0..=60 => Color::Green,
            61..=85 => Color::Yellow,
            _ => Color::Red,
        }
    };

    let right = Paragraph::new(Line::from(vec![
        Span::styled(cpu, Style::default().fg(stat_color(((stats.cpu_load * 25.0) as u8).min(99)))),
        Span::raw("  "),
        Span::styled(ram, Style::default().fg(stat_color(stats.ram_pct))),
        Span::raw("  "),
        Span::styled(disk, Style::default().fg(stat_color(stats.disk_used_pct))),
        Span::raw("  "),
        Span::styled(
            n_sessions,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            time_str,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]))
    .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right, split[1]);
}
