use crate::app::{App, InputMode, MenuAction, SessionEntry, SessionFocus, Tab};
use omega_core::session::SessionRole;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
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
        Tab::Settings => draw_settings(frame, app, chunks[1]),
        Tab::Help => draw_help(frame, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Render agent picker overlay if in that input mode
    if let InputMode::NewSessionAgent(ref name) = app.input_mode {
        draw_agent_picker(frame, app, name);
    }
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

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Sessions", "Menu", "Settings", "Help"];
    let selected = match app.tab {
        Tab::Sessions => 0,
        Tab::Menu => 1,
        Tab::Settings => 2,
        Tab::Help => 3,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" OmegaOS "),
        )
        .select(selected)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

fn draw_sessions(frame: &mut Frame, app: &App, area: Rect) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let list_focused = app.session_focus == SessionFocus::List;
    let chat_focused = app.session_focus == SessionFocus::Chat;

    // ── Left: session list ──────────────────────────────────────────────────
    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, entry)| render_session_item(entry, i == app.selected && list_focused))
        .collect();

    let list_border_style = if list_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sessions ({}) ", app.sessions.len()))
                .border_style(list_border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, split[0]);

    // ── Right: preview + (when focused) a chat input box ────────────────────
    let preview_title = match app.selected_session() {
        Some(e) => format!(" {} ", e.session.name),
        None => " Preview ".to_string(),
    };

    let preview_border_style = if chat_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let total_lines = app.preview_content.lines().count() as u16;
    let viewport_height = split[1].height.saturating_sub(2); // borders
    let max_scroll = total_lines.saturating_sub(viewport_height);
    let scroll = app.preview_scroll.min(max_scroll);

    let preview_lines: Vec<Line> = if app.preview_content.is_empty() {
        vec![Line::from(Span::styled(
            "(select a session to preview)",
            Style::default().fg(Color::DarkGray),
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

    if chat_focused {
        // Split right column vertically: preview on top, chat input at bottom
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(split[1]);

        let preview = Paragraph::new(preview_lines)
            .scroll((scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{}{}", preview_title, scroll_indicator))
                    .border_style(preview_border_style),
            );
        frame.render_widget(preview, right_split[0]);

        // Chat input box
        let target = app
            .selected_session()
            .map(|e| e.session.name.clone())
            .unwrap_or_default();
        let input_line = Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Yellow)),
            Span::raw(app.chat_input.clone()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        let chat = Paragraph::new(input_line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" → {} (Enter send, Tab/Esc back) ", target))
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(chat, right_split[1]);
    } else {
        let preview = Paragraph::new(preview_lines)
            .scroll((scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{}{}", preview_title, scroll_indicator))
                    .border_style(preview_border_style),
            );
        frame.render_widget(preview, split[1]);
    }
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
            SessionRole::System => Color::DarkGray,
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
        Style::default().add_modifier(Modifier::BOLD)
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

fn draw_menu(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = MenuAction::all()
        .iter()
        .enumerate()
        .map(|(i, action)| {
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
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("[{}] ", action.shortcut()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action.label(), label_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Actions — ↑/↓ navigate, Enter to execute "),
    );

    frame.render_widget(list, area);
}

fn draw_settings(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  OmegaOS Settings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  Default agent for Master AISB:  {}", app.config.aisb_agent)),
        Line::from(format!("  Default model:                  {}", app.config.default_model)),
        Line::from(format!("  Auto-spawn Master on launch:    {}", app.config.auto_spawn_master)),
        Line::from(format!("  Auto-naming sessions:           {}", app.config.auto_naming)),
        Line::from(""),
        Line::from(Span::styled(
            "  Installed agents (✓ available, ✗ not installed):",
            Style::default().fg(Color::Yellow),
        )),
    ];

    for agent in omega_core::agents::Agent::all() {
        let (icon, color) = if agent.is_available() {
            ("✓", Color::Green)
        } else {
            ("✗", Color::Red)
        };
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(icon, Style::default().fg(color)),
            Span::raw(format!("  {:8}  {}", agent.name(), agent.display_name())),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Edit ~/.omega/config.toml to change settings.",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "  Master AISB session: aisb-master (always pinned at top of Sessions list).",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Settings "),
    );
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        "",
        "  OmegaOS — Agentic Terminal Operating System",
        "",
        "  Navigation:",
        "    ← / →              Switch tabs (Sessions ↔ Menu ↔ Help)",
        "    ↑ / ↓ or j/k       Navigate items in current tab",
        "    Enter              Attach session  OR  execute menu action",
        "    Esc                Back to Sessions tab (or quit if there)",
        "    q                  Quit",
        "",
        "  Menu — direct agent launchers (each Enter creates a new session):",
        "    [c] New Claude     [C] New Codex      [g] New Gemini",
        "    [p] New Pi         [G] New GLM        [t] New Terminal",
        "    [d] Dispatch oracle (project + mission)",
        "    [r] Refresh        [.] Toggle protect [x] Kill selected",
        "",
        "  Session Actions:",
        "    n                  New session (prompts for name)",
        "    d                  Dispatch oracle (prompts for project + mission)",
        "    x                  Kill selected session",
        "    .                  Toggle protection",
        "    r                  Refresh session list",
        "    ?                  Show this help",
        "",
        "  Input Mode:",
        "    Type to fill, Enter to submit, Esc to cancel",
        "    Backspace to delete",
        "",
        "  Status Icons:",
        "    ◆  Oracle           ●  Worker",
        "    ⌂  Home             ⚙  System",
        "",
        "  CLI (outside TUI):",
        "    omega list                       Show all sessions",
        "    omega new <name> [--cmd claude]  Create session",
        "    omega dispatch <project> <msg>   Dispatch oracle",
        "    omega attach <name>              Attach to session",
        "    omega send <name> <text>         Send text to pane",
        "    omega capture <name>             Show pane content",
        "    omega kill <name>                Kill session",
        "    omega --help                     All commands",
        "",
    ];

    let paragraph = Paragraph::new(
        help_text
            .into_iter()
            .map(|s| Line::from(s.to_string()))
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help "),
    );

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
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

    // Normal mode: tmux-claude inspired status bar with stats
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

    // Left side: Ω badge + selected session
    let left = Paragraph::new(Line::from(vec![
        Span::styled(" Ω ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(session_info, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            app.status_message.as_deref().unwrap_or(""),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(left, split[0]);

    // Right side: system stats
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
        Span::styled(n_sessions, Style::default().fg(Color::DarkGray)),
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
