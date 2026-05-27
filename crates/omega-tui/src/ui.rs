use crate::app::{App, InputMode, MenuAction, SessionEntry, Tab};
use omega_core::session::SessionRole;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
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
        Tab::Help => draw_help(frame, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Sessions", "Menu", "Help"];
    let selected = match app.tab {
        Tab::Sessions => 0,
        Tab::Menu => 1,
        Tab::Help => 2,
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
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, entry)| render_session_item(entry, i == app.selected))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sessions ({}) ", app.sessions.len())),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, split[0]);

    // Live preview pane (right side)
    let preview_title = match app.selected_session() {
        Some(e) => format!(" Preview: {} ", e.session.name),
        None => " Preview ".to_string(),
    };

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

    let preview = Paragraph::new(preview_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(preview_title)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(preview, split[1]);
}

fn render_session_item(entry: &SessionEntry, selected: bool) -> ListItem<'static> {
    let icon = match entry.session.role {
        SessionRole::Oracle => "◆",
        SessionRole::Worker => "●",
        SessionRole::Home => "⌂",
        SessionRole::System => "⚙",
    };

    let icon_color = match entry.session.role {
        SessionRole::Oracle => Color::Yellow,
        SessionRole::Worker => Color::Green,
        SessionRole::Home => Color::Blue,
        SessionRole::System => Color::DarkGray,
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

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        "",
        "  OmegaOS — Agentic Terminal Operating System",
        "",
        "  Navigation:",
        "    ← / →              Switch tabs (Sessions ↔ Menu ↔ Help)",
        "    ↑ / ↓ or j/k       Navigate items in current tab",
        "    Enter              Attach session / execute menu action",
        "    q                  Quit",
        "    Esc                Back to Sessions tab (or quit if there)",
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
    let (prompt, value) = match &app.input_mode {
        InputMode::Normal => {
            let msg = app
                .status_message
                .as_deref()
                .unwrap_or("n=new  d=dispatch  Enter=attach  x=kill  .=protect  r=refresh  Tab=tabs  q=quit");
            let status = Paragraph::new(Line::from(vec![
                Span::styled(" Ω ", Style::default().fg(Color::Black).bg(Color::Cyan)),
                Span::raw(" "),
                Span::raw(msg.to_string()),
            ]));
            frame.render_widget(status, area);
            return;
        }
        InputMode::NewSession => ("New session name", app.input_buffer.clone()),
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
}
