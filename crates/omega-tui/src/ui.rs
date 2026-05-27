use crate::app::{App, InfoSection, InputMode, MenuAction, MonitorAction, SessionEntry, SessionFocus, SessionRow, SettingsSection, Tab};
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
        Tab::Monitor => draw_monitor(frame, app, chunks[1]),
        Tab::Settings => draw_settings(frame, app, chunks[1]),
        Tab::Info => draw_info(frame, app, chunks[1]),
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
    let titles = vec!["Sessions", "Menu", "Monitor", "Settings", "Info", "Help"];
    let selected = match app.tab {
        Tab::Sessions => 0,
        Tab::Menu => 1,
        Tab::Monitor => 2,
        Tab::Settings => 3,
        Tab::Info => 4,
        Tab::Help => 5,
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
                        .fg(Color::DarkGray)
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

    draw_sessions_right(frame, app, split[1], chat_focused);
}

/// Render the right column of the Sessions tab (preview + optional chat input).
/// Used both in split layout and chat-fullscreen mode.
fn draw_sessions_right(frame: &mut Frame, app: &App, area: Rect, chat_focused: bool) {
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
        Style::default().fg(Color::DarkGray)
    };

    let total_lines = app.preview_content.lines().count() as u16;
    let viewport_height = area.height.saturating_sub(2);
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
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let preview = Paragraph::new(preview_lines)
            .scroll((scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{}{}", preview_title, scroll_indicator))
                    .border_style(preview_border_style),
            );
        frame.render_widget(preview, right_split[0]);

        let target = app
            .selected_session()
            .map(|e| e.session.name.clone())
            .unwrap_or_default();
        let hint = if fullscreen {
            " (Enter send, Tab-Tab to exit fullscreen, Esc back to list) "
        } else {
            " (Enter send, Tab-Tab fullscreen, Tab/Esc back) "
        };
        let input_line = Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Yellow)),
            Span::raw(app.chat_input.clone()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        let chat = Paragraph::new(input_line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" → {}{}", target, hint))
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
        frame.render_widget(preview, area);
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

fn draw_monitor(frame: &mut Frame, app: &App, area: Rect) {
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
                    Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::DarkGray),
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
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "    We just READ its usage cache (/tmp/aisb-usage.json) for the billing view.",
        Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::DarkGray),
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
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Monitor "),
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

fn draw_settings(frame: &mut Frame, app: &App, area: Rect) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    // ── Left: section list ──────────────────────────────────────────────────
    let items: Vec<ListItem> = SettingsSection::all()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let selected = i == app.settings_selected;
            let prefix = if selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(section.label(), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Settings — ↑/↓ to select ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(list, split[0]);

    // ── Right: details panel for the selected section ──────────────────────
    let providers = omega_core::providers::ProvidersConfig::load();
    let lines = render_settings_detail(app, &providers);

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.selected_settings_section().label()))
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(paragraph, split[1]);
}

fn render_settings_detail(
    app: &App,
    providers: &omega_core::providers::ProvidersConfig,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = vec![Line::from("")];

    let cfg = &app.config;
    let section = app.selected_settings_section();

    match section {
        SettingsSection::Install => {
            lines.push(Line::from(Span::styled(
                "  Install or re-install CLI agents — required before launching their sessions.",
                Style::default().fg(Color::Yellow),
            )));
            lines.push(Line::from(""));
            for agent in omega_core::agents::Agent::all() {
                if matches!(agent, omega_core::agents::Agent::Shell) {
                    continue; // shell is always available
                }
                let (status_icon, status_color, status_text) = if agent.is_available() {
                    ("✓", Color::Green, "installed")
                } else {
                    ("✗", Color::Red, "not installed")
                };
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{:8}", agent.name()), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:32} ", agent.display_name())),
                    Span::styled(format!("{} {}", status_icon, status_text), Style::default().fg(status_color)),
                ]));
                if let Some(home) = agent.homepage() {
                    lines.push(Line::from(vec![
                        Span::raw("    home: "),
                        Span::styled(home.to_string(), Style::default().fg(Color::Blue)),
                    ]));
                }
                if let Some(install) = agent.install_command() {
                    lines.push(Line::from(vec![
                        Span::raw("    install: "),
                        Span::styled(install.to_string(), Style::default().fg(Color::Yellow)),
                    ]));
                }
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "  Run the installer from a shell, or use:",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "    omega install <agent>    (e.g. omega install hermes)",
                Style::default().fg(Color::Yellow),
            )));
        }
        SettingsSection::Hermes => {
            let c = &providers.hermes;
            lines.push(kv("Model", default_or(&c.model, "(default)")));
            lines.push(kv("API key", &mask_key(&c.api_key)));
            availability(&mut lines, omega_core::agents::Agent::Hermes);
            lines.push(Line::from(""));
            if !omega_core::agents::Agent::Hermes.is_available() {
                lines.push(Line::from(Span::styled(
                    "  Install with:",
                    Style::default().fg(Color::DarkGray),
                )));
                if let Some(cmd) = omega_core::agents::Agent::Hermes.install_command() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", cmd),
                        Style::default().fg(Color::Yellow),
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "  Or:  omega install hermes",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
        SettingsSection::General => {
            lines.push(kv("Default AISB agent", &cfg.aisb_agent));
            lines.push(kv("Default model", &cfg.default_model));
            lines.push(kv("Auto-spawn Master on launch", &cfg.auto_spawn_master.to_string()));
            lines.push(kv("Auto-naming sessions", &cfg.auto_naming.to_string()));
            lines.push(kv("State dir", &cfg.state_dir.to_string_lossy()));
            lines.push(kv("Logs dir", &cfg.logs_dir.to_string_lossy()));
        }
        SettingsSection::Claude => {
            let c = &providers.claude;
            lines.push(kv("Model", default_or(&c.model, "(opus)")));
            lines.push(kv("Effort", default_or(&c.effort, "(default)")));
            lines.push(kv("API key", &mask_key(&c.api_key)));
            lines.push(kv("Skip-perms by default", &c.dangerously_skip_permissions.to_string()));
            availability(&mut lines, omega_core::agents::Agent::Claude);
        }
        SettingsSection::Codex => {
            let c = &providers.codex;
            lines.push(kv("Model", default_or(&c.model, "(default)")));
            lines.push(kv("API key", &mask_key(&c.api_key)));
            lines.push(kv("Base URL", default_or(&c.base_url, "(default)")));
            availability(&mut lines, omega_core::agents::Agent::Codex);
        }
        SettingsSection::Gemini => {
            let c = &providers.gemini;
            lines.push(kv("Model", default_or(&c.model, "(default)")));
            lines.push(kv("API key", &mask_key(&c.api_key)));
            availability(&mut lines, omega_core::agents::Agent::Gemini);
        }
        SettingsSection::Pi => {
            let c = &providers.pi;
            lines.push(kv("Provider", default_or(&c.provider, "openrouter")));
            lines.push(kv("Model", default_or(&c.model, "anthropic/claude-sonnet-4.6")));
            lines.push(kv("Extension", default_or(&c.extension, "(none)")));
            availability(&mut lines, omega_core::agents::Agent::Pi);
        }
        SettingsSection::Glm => {
            let c = &providers.glm;
            lines.push(kv("Model", default_or(&c.model, "(default)")));
            lines.push(kv("API key", &mask_key(&c.api_key)));
            availability(&mut lines, omega_core::agents::Agent::Glm);
        }
        SettingsSection::Aisb => {
            lines.push(kv("Master session name", omega_core::aisb::MASTER_SESSION_NAME));
            lines.push(kv("Auto-spawn", &cfg.auto_spawn_master.to_string()));
            lines.push(kv("Agent", &cfg.aisb_agent));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  System prompt: agents/aisb-master.md (compiled into binary)",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "  Materialised at: ~/.omega/aisb-master.system.md",
                Style::default().fg(Color::DarkGray),
            )));
        }
        SettingsSection::Telegram => {
            match omega_core::monitor::OmegaTelegramConfig::read() {
                Some(tg) => {
                    if !tg.label.is_empty() {
                        lines.push(kv("Label", &tg.label));
                    }
                    lines.push(kv("Enabled", &tg.enabled.to_string()));
                    lines.push(kv("Chat ID", &tg.chat_id.to_string()));
                    lines.push(kv("Relay session", &tg.relay_session));
                    let sender_filter = if tg.allow_user_ids.is_empty() {
                        "chat_id only".to_string()
                    } else {
                        format!("user_ids {:?}", tg.allow_user_ids)
                    };
                    lines.push(kv("Sender filter", &sender_filter));
                }
                None => {
                    lines.push(Line::from(Span::styled(
                        "  Not configured.",
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from(""));
                    lines.push(Line::from("  Set up with:"));
                    lines.push(Line::from(Span::styled(
                        "    omega telegram setup <BOT_TOKEN> <CHAT_ID> [--user-id …]",
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Edit settings via CLI (changes apply to all sessions):",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "    omega config set <provider>.<key> <value>      (e.g. claude.model opus)",
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::styled(
        "    omega config get <provider>.<key>",
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::styled(
        "    ~/.omega/providers.toml  ~/.omega/config.toml",
        Style::default().fg(Color::DarkGray),
    )));

    lines
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

fn draw_info(frame: &mut Frame, app: &App, area: Rect) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    // Left: sub-section list
    let items: Vec<ListItem> = InfoSection::all()
        .iter()
        .enumerate()
        .map(|(i, sec)| {
            let selected = i == app.info_section_selected;
            let prefix = if selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(sec.label(), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Info — ↑/↓ to select ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(list, split[0]);

    // Right: detail of the selected sub-section
    let lines = match app.selected_info_section() {
        InfoSection::AisbAgents => render_info_aisb_agents(app),
        InfoSection::Oracle => render_info_oracle(),
        InfoSection::Workers => render_info_workers(),
        InfoSection::Rules => render_info_rules(),
    };

    let title = format!(" {} ", app.selected_info_section().label());
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(paragraph, split[1]);
}

fn render_info_aisb_agents(app: &App) -> Vec<Line<'static>> {
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
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    // Compact list
    for (i, agent) in agents.iter().enumerate() {
        let def = agent.definition();
        let selected = i == app.info_agent_selected;
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
        Style::default().fg(Color::DarkGray),
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
    lines
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
            Style::default().fg(Color::DarkGray),
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
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                format!("    Why: {}", r.reason),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }
    }
    lines
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        "",
        "  OmegaOS — Agentic Terminal Operating System",
        "",
        "  Tabs:",
        "    ← / →              Switch tabs (Sessions | Menu | Monitor | Settings | Help)",
        "    Shift+Tab          Same as ←",
        "",
        "  Sessions tab:",
        "    ↑ / ↓ or j/k       Navigate sessions",
        "    Enter              Attach to selected session (rmux switch-client)",
        "    Tab                Focus the chat pane (talk to selected session)",
        "    Tab-Tab (rapide)   Toggle chat FULLSCREEN (hides list)",
        "    Esc (in chat)      Back to session list",
        "    r / R              Rename selected session",
        "    x / X              Kill selected session (skipped if locked)",
        "    .                  Toggle lock/protection",
        "    F5                 Refresh session list + preview",
        "    PageUp / PageDown  Scroll the preview pane",
        "    Home / End         Jump to top / bottom of preview (tail-follow on at End)",
        "",
        "  Menu tab — direct agent launchers:",
        "    [c] New Claude     [C] New Codex      [g] New Gemini",
        "    [p] New Pi         [G] New GLM        [t] New Terminal",
        "    [d] Dispatch oracle (project + mission)",
        "    [.] Toggle lock    [x] Kill selected",
        "",
        "  Monitor tab — billing / accounts / Telegram:",
        "    ↑ / ↓ + Enter      Run the highlighted action",
        "    [L] Login Claude   (opens session with `claude /login`)",
        "    [T] Telegram setup [D] Telegram disconnect",
        "    [B] Refresh billing now",
        "",
        "  Settings tab — provider configuration:",
        "    ↑ / ↓              Browse sections (General / Claude / Codex / ... / Telegram)",
        "    Per-provider:      model, API key (masked), CLI availability ✓/✗",
        "    Edit via CLI:      omega config set claude.model opus",
        "                       omega config set codex.api_key sk-…",
        "",
        "  Global keys (everywhere):",
        "    Option+Z, Option+/, Ctrl+Space",
        "      → Pop the OmegaOS menu from ANY rmux session (install with",
        "        `omega install-bindings`).",
        "    q                  Quit OmegaOS TUI",
        "",
        "  CLI cheatsheet (outside the TUI):",
        "    omega                       Launch TUI",
        "    omega master / omega aisb   Attach the AISB Master",
        "    omega list                  List all sessions",
        "    omega monitor               Billing / accounts / bot status",
        "    omega orchestrate <P> <M>   Full mission pipeline",
        "    omega telegram setup …      Bot setup with --user-id allow-list",
        "    omega config set …          Provider config (propagates to sessions)",
        "    omega projects              Auto-discover projects under $HOME",
        "    omega install-bindings      Install Option+Z global bindings",
        "",
        "  Status Icons:",
        "    ★  Master AISB      ◆  Oracle      ●  Worker      ⌂  Home      ⚙  System",
        "    §  Locked / protected (immune to `x` kill)",
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
            InputMode::RenameSession(old) => (
                "Rename session",
                format!("[{} →] {}", old, app.input_buffer),
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
