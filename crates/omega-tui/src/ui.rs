use crate::app::{App, SessionEntry, Tab};
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

    frame.render_widget(list, area);
}

fn render_session_item(entry: &SessionEntry, _selected: bool) -> ListItem<'static> {
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

    let project_tag = match &entry.session.project {
        Some(p) => format!(" [{}]", p),
        None => String::new(),
    };

    let line = Line::from(vec![
        Span::raw(entry.tree_prefix.clone()),
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(icon_color),
        ),
        Span::styled(
            entry.session.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(project_tag, Style::default().fg(Color::DarkGray)),
        Span::styled(progress_str, Style::default().fg(Color::Cyan)),
    ]);

    ListItem::new(line)
}

fn draw_menu(frame: &mut Frame, _app: &App, area: Rect) {
    let menu_items = vec![
        "[n] New session",
        "[o] New oracle for project",
        "[w] New worker",
        "[d] Dispatch mission",
        "[k] Kill session",
        "[K] Kill all workers",
        "[p] Protect/unprotect",
        "[r] Refresh",
        "[q] Quit",
    ];

    let items: Vec<ListItem> = menu_items
        .into_iter()
        .map(|s| {
            let parts: Vec<&str> = s.splitn(2, ']').collect();
            if parts.len() == 2 {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{}]", parts[0]),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(parts[1]),
                ]))
            } else {
                ListItem::new(s.to_string())
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Actions "),
    );

    frame.render_widget(list, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        "",
        "  OmegaOS — Agentic Terminal Operating System",
        "",
        "  Navigation:",
        "    Tab/Shift+Tab    Switch tabs",
        "    ↑/↓ or j/k       Navigate sessions",
        "    Enter             Attach to session",
        "    q/Esc             Quit menu",
        "",
        "  Session Management:",
        "    n                 New session",
        "    o                 New oracle (project picker)",
        "    x                 Kill selected session",
        "    .                 Toggle protection",
        "    r                 Refresh session list",
        "",
        "  Dispatch:",
        "    d                 Dispatch mission to project",
        "    w                 Spawn worker under oracle",
        "",
        "  Status Icons:",
        "    ◆  Oracle         ●  Worker",
        "    ⌂  Home           ⚙  System",
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
    let msg = app
        .status_message
        .as_deref()
        .unwrap_or("Press ? for help | Tab to switch views | q to quit");

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" Ω ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" "),
        Span::raw(msg),
    ]));

    frame.render_widget(status, area);
}
