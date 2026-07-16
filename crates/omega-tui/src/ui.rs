use crate::app::{App, InfoSection, InputMode, MenuAction, MonitorAction, MonitorSection, SessionEntry, SessionFocus, SessionRow, SettingsSection, Tab};
use omega_core::done::DoneStatus;
use omega_core::session::{PreviewColor, SessionRole};

/// Map a preview color to a ratatui Color, PRESERVING depth so the emitted
/// escape matches the terminal's capability. ANSI 0–15 → the 16-color named
/// variants (emit `3x`/`9x`, render on any color terminal incl. over mosh);
/// 16–255 → 256-indexed; true RGB → 24-bit. (Forcing everything to RGB made
/// Claude's 16-color stream render as grey truecolor on non-truecolor terminals.)
fn preview_to_color(c: PreviewColor) -> Color {
    match c {
        PreviewColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        PreviewColor::Indexed(i) => match i {
            0 => Color::Reset,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::Gray,
            8 => Color::DarkGray,
            9 => Color::LightRed,
            10 => Color::LightGreen,
            11 => Color::LightYellow,
            12 => Color::LightBlue,
            13 => Color::LightMagenta,
            14 => Color::LightCyan,
            // 0 (black) and 15 (white) → Reset (the terminal's own fg) instead of
            // a fixed black/white, which goes invisible when the theme's bg is the
            // same shade. This is the mirror passthrough for Claude's own output —
            // critical now that Claude renders to the normal screen (the whole
            // conversation, incl. the user's own messages, flows through here).
            15 => Color::Reset,
            n => Color::Indexed(n),
        },
    }
}

/// Foreground mapping for a span that carries an EXPLICIT background. The
/// 0/15 → Reset safety net above exists so black/white text on the DEFAULT
/// background can't vanish into a same-shade theme canvas — but on a span
/// with its own bg it backfires: the REVERSE fallback in omega-core
/// (`styled_rows_from_snapshot`) synthesizes black-on-gray precisely so the
/// swap stays visible, and 0→Reset turned that into default-fg-on-gray
/// (near-white on light gray on dark terminals — selections unreadable).
/// With an explicit bg the invisibility risk is gone, so black/white stay
/// literal; everything else keeps the depth-preserving passthrough.
fn preview_fg_color(c: PreviewColor, has_explicit_bg: bool) -> Color {
    match c {
        PreviewColor::Indexed(0) if has_explicit_bg => Color::Black,
        PreviewColor::Indexed(15) if has_explicit_bg => Color::White,
        other => preview_to_color(other),
    }
}

/// Re-style a rendered line so display columns `[from, to)` carry REVERSED —
/// the mouse drag-selection highlight. Splits spans at the boundaries and
/// counts DISPLAY width (emoji/CJK = 2 cells) so the highlight tracks the
/// pointer on wide glyphs.
fn reverse_cols(line: &Line<'_>, from: usize, to: usize) -> Line<'static> {
    use unicode_width::UnicodeWidthChar;
    let mut out: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 2);
    let mut col = 0usize;
    for sp in &line.spans {
        let mut seg = String::new();
        let mut seg_rev: Option<bool> = None;
        for ch in sp.content.chars() {
            let rev = col >= from && col < to;
            if seg_rev != Some(rev) {
                if let (Some(prev), false) = (seg_rev, seg.is_empty()) {
                    let style = if prev {
                        sp.style.add_modifier(Modifier::REVERSED)
                    } else {
                        sp.style
                    };
                    out.push(Span::styled(std::mem::take(&mut seg), style));
                }
                seg_rev = Some(rev);
            }
            seg.push(ch);
            col += ch.width().unwrap_or(0);
        }
        if let (Some(prev), false) = (seg_rev, seg.is_empty()) {
            let style = if prev {
                sp.style.add_modifier(Modifier::REVERSED)
            } else {
                sp.style
            };
            out.push(Span::styled(seg, style));
        }
    }
    Line::from(out)
}
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

// All TUI-chrome colors are semantic theme roles (Settings -> Theme). Only the
// `preview_to_color` passthrough above keeps raw terminal colors -- that is
// the agent's own pane content, never re-skinned.
use crate::theme as th;

// ── Render-path I/O memo ────────────────────────────────────────────────────
// draw() runs at 15-60 FPS (TICK_ACTIVE 16ms / TICK_IDLE 66ms in main.rs), so
// anything a renderer reads from the OS multiplies by the frame rate: the
// status bar's SystemStats::read() forks a `df` subprocess, and the
// monitor/project detail renderers re-read + re-parse JSON/TOML files — per
// frame, per TUI. (The run-loop comment records that 5 idle menus once
// saturated a 2-core VPS.) These memos hold each read for RENDER_TTL; status
// surfaces tolerate a couple seconds of staleness invisibly. Same medicine as
// `App::providers_cache`, generalized for the renderers that take no `App`.
const RENDER_TTL: std::time::Duration = std::time::Duration::from_secs(2);

/// True when an ancestor of this process is mosh-server — the transport that
/// silently eats the mouse handshake (mobile-shell/mosh#101). Walked once and
/// cached for the process lifetime: ancestry can't change after spawn.
fn under_mosh() -> bool {
    static UNDER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *UNDER.get_or_init(|| {
        let mut pid = std::process::id();
        for _ in 0..16 {
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) else {
                return false;
            };
            // /proc/<pid>/stat: "pid (comm) state ppid …" — comm may contain
            // spaces/parens, so split on the LAST ')'.
            let Some(rest) = stat.rsplit_once(')').map(|(_, r)| r) else { return false };
            let comm = stat
                .split_once('(')
                .and_then(|(_, r)| r.rsplit_once(')').map(|(c, _)| c))
                .unwrap_or("");
            if comm.contains("mosh-server") {
                return true;
            }
            let Some(ppid) = rest.split_whitespace().nth(1).and_then(|p| p.parse::<u32>().ok())
            else {
                return false;
            };
            if ppid <= 1 {
                return false;
            }
            pid = ppid;
        }
        false
    })
}

fn render_memo<T: Clone + 'static>(
    slot: &'static std::thread::LocalKey<
        std::cell::RefCell<Option<(std::time::Instant, T)>>,
    >,
    load: impl FnOnce() -> T,
) -> T {
    slot.with(|cell| {
        let mut cached = cell.borrow_mut();
        if let Some((at, v)) = cached.as_ref() {
            if at.elapsed() < RENDER_TTL {
                return v.clone();
            }
        }
        let v = load();
        *cached = Some((std::time::Instant::now(), v.clone()));
        v
    })
}

/// Global breathing room around the whole UI — some terminals render cell
/// (0,0) flush against the window edge, which looks cramped. The theme
/// background still paints the FULL frame, so the margin shows the theme's
/// canvas (or the terminal's own bg for Omega/Transparent). Skipped on tiny
/// terminals where every row counts.
fn padded(area: Rect) -> Rect {
    if area.width < 70 || area.height < 20 {
        return area;
    }
    area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    })
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Theme background: paint the whole frame first so every widget sits on
    // the active theme's canvas (None = keep the terminal's own background).
    // This is what makes themes visually distinct at a glance -- accent
    // colors alone read as near-identical between palettes.
    // Skippable via Settings → Theme → "Theme background": OFF keeps the
    // terminal's own background (transparency / background image) visible.
    if app.config.theme_background {
        if let Some(bg) = th::bg() {
            // fg too: spans rendered without an explicit fg inherit this themed
            // text color instead of the terminal default (which may be unreadable
            // on the painted background -- e.g. white-on-white for Paper).
            frame.render_widget(
                Block::default().style(Style::default().fg(th::text()).bg(bg)),
                frame.area(),
            );
        }
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(padded(frame.area()));

    draw_tabs(frame, app, chunks[0]);

    match app.tab {
        Tab::Sessions => draw_sessions(frame, app, chunks[1]),
        Tab::Menu => draw_menu(frame, app, chunks[1]),
        Tab::Projects => draw_info(frame, app, chunks[1]),
        Tab::Settings => draw_settings(frame, app, chunks[1]),
        Tab::Marketing => draw_marketing(frame, app, chunks[1]),
        Tab::Help => draw_help(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Overlay modals — drawn LAST so they paint on top of everything
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
        InputMode::NewNamedSession(ref agent) => {
            let title = format!("New {} session", agent);
            let hint = format!("Session name (Enter to launch, Esc to cancel)");
            draw_simple_input_modal_owned(frame, app, &title, &hint, false);
        }
        InputMode::DispatchProject(..) => draw_dispatch_picker(frame, app),
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
        InputMode::NewProjectName => draw_simple_input_modal(
            frame,
            app,
            "New project — step 1/3",
            "Project name (Enter to continue, Esc to cancel)",
            false,
        ),
        InputMode::NewProjectCategory(..) | InputMode::NewProjectStack(..) => {
            draw_new_project_picker(frame, app);
        }
        InputMode::SelectModel(..) => {
            draw_model_picker(frame, app);
        }
        InputMode::ProjectOpenAgent(..) => {
            draw_project_open_agent_picker(frame, app);
        }
        InputMode::ProjectDelete(..) => {
            draw_project_delete_picker(frame, app);
        }
        InputMode::NewProjectCredGroup(..) => {
            let groups = omega_core::provisioning::list_groups().join(", ");
            draw_simple_input_modal(
                frame,
                app,
                "Client credentials — group",
                &format!(
                    "Existing: {}  —  type one to reuse, or a NEW client name (Enter; Esc = default)",
                    groups
                ),
                false,
            );
        }
        InputMode::NewProjectLaunchPrompt(..) => draw_simple_input_modal(
            frame,
            app,
            "New project — kickoff (optional)",
            "Describe the idea / requirements (Enter to continue, Esc to skip)",
            false,
        ),
        InputMode::NewProjectLaunchDocs(..) => draw_simple_input_modal(
            frame,
            app,
            "New project — docs (optional)",
            "Comma-separated doc paths to seed the project (Enter to launch, Esc to skip)",
            false,
        ),
        InputMode::ProvisioningSetup { step, .. } => {
            let fields = crate::app::PROVISIONING_FIELDS;
            let (key, hint, masked) = fields
                .get(step)
                .map(|f| (f.0, f.1, f.2))
                .unwrap_or(("", "", true));
            let title =
                format!("Provisioning keys — {}/{}: {}", step + 1, fields.len(), key);
            draw_simple_input_modal_owned(frame, app, &title, hint, masked);
        }
        InputMode::GroupSetupId => draw_simple_input_modal(
            frame,
            app,
            "Telegram project group",
            "Supergroup id (negative, e.g. -1001234567890) — Enter to save, Esc to cancel",
            false,
        ),
        InputMode::AddProjectPath => draw_simple_input_modal(
            frame,
            app,
            "Add project — register an existing folder",
            "Absolute path to the project folder (Enter to register, Esc to cancel)",
            false,
        ),
        InputMode::ReauthCode => draw_simple_input_modal(
            frame,
            app,
            "Claude re-login — paste authorize code",
            "Paste the code from your browser (Enter to submit, Esc to cancel)",
            false,
        ),
        _ => {}
    }
}

/// Overlay picker for new-project wizard steps 2 (category) and 3 (stack).
/// Reads the option list + selection index straight from the InputMode variant.
fn draw_new_project_picker(frame: &mut Frame, app: &App) {
    let (title, options, sel): (String, &[(&str, &str)], usize) = match &app.input_mode {
        InputMode::NewProjectCategory(name, sel) => (
            format!(" New project [{}] — step 2/3: category — ↑/↓, Enter, Esc ", name),
            crate::app::NEW_PROJECT_CATEGORIES,
            *sel,
        ),
        InputMode::NewProjectStack(name, _category, sel) => (
            format!(" New project [{}] — step 3/3: stack — ↑/↓, Enter, Esc ", name),
            crate::app::NEW_PROJECT_STACKS,
            *sel,
        ),
        _ => return,
    };

    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (id, label))| {
            let selected = i == sel;
            if selected {
                // Full-width bar — consistent with the Select overlay.
                let row = format!(" ▶ {:10} {}", id, label);
                let pad = inner_w.saturating_sub(row.chars().count());
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", row, " ".repeat(pad)),
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent())
                        .add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!(" {:10} ", id),
                        Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*label, Style::default().fg(th::text())),
                ]))
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(th::accent()))
            .style(Style::default().fg(th::text()).bg(modal_bg(app))),
    );
    frame.render_widget(list, area);
}

/// Modal canvas color: the theme background, unless the user turned theme
/// backgrounds off (Settings → Theme) — then fall back to the terminal's own.
fn modal_bg(app: &App) -> Color {
    if app.config.theme_background {
        th::bg().unwrap_or(Color::Reset)
    } else {
        Color::Reset
    }
}

/// Arrow-key overlay to pick a model from a fixed list (NO typing). Mirrors
/// `draw_new_project_picker`. Reads the option list + selection straight from
/// the `SelectModel` InputMode variant.
fn draw_model_picker(frame: &mut Frame, app: &App) {
    let (config_key, options, sel): (&str, &[String], usize) = match &app.input_mode {
        InputMode::SelectModel(key, opts, sel) => (key.as_str(), opts.as_slice(), *sel),
        _ => return,
    };

    // Theme selector rows show the human label, not the raw slug.
    let is_theme = config_key == "general.theme";
    // Wider modal for themes: label + blurb need the room.
    let area = centered_rect(if is_theme { 70 } else { 50 }, 50, frame.area());
    frame.render_widget(Clear, area);

    let inner_w = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let selected = i == sel;
            let text = if is_theme {
                match crate::theme::ThemeId::from_slug(opt) {
                    Some(id) => format!("{:26}{}", id.label(), id.blurb()),
                    None => opt.clone(),
                }
            } else {
                opt.clone()
            };
            // Pad to the full inner width: the highlight reads as a SOLID
            // selection bar across the modal (the old text-chip highlight was
            // easy to miss on several themes).
            let row = format!(" {} {}", if selected { "▶" } else { " " }, text);
            // Char-safe ellipsis instead of a hard mid-word clip at the
            // border (slicing by byte would also panic on multi-byte chars).
            let row = if row.chars().count() > inner_w && inner_w > 1 {
                format!("{}…", row.chars().take(inner_w - 1).collect::<String>())
            } else {
                row
            };
            let pad = inner_w.saturating_sub(row.chars().count());
            let row = format!("{}{}", row, " ".repeat(pad));
            let style = if selected {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::text())
            };
            ListItem::new(Line::from(Span::styled(row, style)))
        })
        .collect();

    let title = if is_theme {
        " Theme — ↑/↓ live preview, Enter saves, Esc reverts ".to_string()
    } else {
        format!(" Select {} — ↑/↓, Enter, Esc ", config_key)
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(th::accent()))
            .style(Style::default().fg(th::text()).bg(modal_bg(app))),
    );
    // Scroll-follow: stateful render keeps the selected row visible when the
    // list is taller than the modal (15 themes vs ~10 visible rows at 80x24 —
    // arrowing below the fold was blind). Selection visuals stay baked into
    // the items; the state only drives the offset window.
    let mut state = ListState::default();
    state.select(Some(sel));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Open-project agent picker — which agent runs the NEW blank session opened
/// on a project. ↑/↓ or 1/2, Enter, Esc. Non-destructive → accent border.
fn draw_project_open_agent_picker(frame: &mut Frame, app: &App) {
    let (name, sel): (&str, usize) = match &app.input_mode {
        InputMode::ProjectOpenAgent(name, _path, sel) => (name.as_str(), *sel),
        _ => return,
    };
    let options = [
        "1. Claude Code — Opus 4.8",
        "2. Codex — OpenAI",
        "   Cancel",
    ];
    let area = centered_rect(60, 22, frame.area());
    frame.render_widget(Clear, area);
    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == sel {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::bright())
            };
            let row = format!("{} {}", if i == sel { "▶" } else { " " }, opt);
            let pad = if i == sel { inner_w.saturating_sub(row.chars().count()) } else { 0 };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", row, " ".repeat(pad)),
                style,
            )))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" ▶ Open {} — new session — ↑/↓ or 1/2, Enter, Esc ", name))
            .border_style(Style::default().fg(th::accent())),
    );
    frame.render_widget(list, area);
}

/// Project delete menu — the SAME three escalating tiers as the Telegram bot
/// (omega → local → all), as a visible arrow-key overlay. ↑/↓ or 1/2/3, Enter,
/// Esc. Destructive → red border, explicit consequences per tier.
fn draw_project_delete_picker(frame: &mut Frame, app: &App) {
    let (name, sel): (&str, usize) = match &app.input_mode {
        InputMode::ProjectDelete(name, sel) => (name.as_str(), *sel),
        _ => return,
    };
    let options = [
        "1. Remove from OmegaOS — topic + dashboard agent + agent-bot + registry (folder & GitHub kept)",
        "2. Delete local machine — that + kill oracle + DELETE the local folder (GitHub kept)",
        "3. Delete ALL (+ GitHub) — that + DELETE the GitHub repo (nothing remains)",
        "   Cancel",
    ];
    let area = centered_rect(80, 30, frame.area());
    frame.render_widget(Clear, area);
    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let danger = matches!(i, 1 | 2);
            let style = if i == sel {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(if danger { th::error() } else { th::accent() })
                    .add_modifier(Modifier::BOLD)
            } else if danger {
                Style::default().fg(th::error())
            } else {
                Style::default().fg(th::bright())
            };
            // Full-width bar when selected — consistent with the other pickers.
            let row = format!("{} {}", if i == sel { "▶" } else { " " }, opt);
            let pad = if i == sel { inner_w.saturating_sub(row.chars().count()) } else { 0 };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", row, " ".repeat(pad)),
                style,
            )))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 🗑 Delete {} — ↑/↓ or 1/2/3, Enter, Esc ", name))
            .border_style(Style::default().fg(th::error())),
    );
    frame.render_widget(list, area);
}

/// Dispatch oracle — step 1: project picker overlay (no typing). Lists the
/// added projects from the shared ProjectRegistry; ↑/↓ move, Enter selects,
/// Esc cancels. Mirrors `draw_model_picker`.
fn draw_dispatch_picker(frame: &mut Frame, app: &App) {
    let (projects, sel): (&[String], usize) = match &app.input_mode {
        InputMode::DispatchProject(projects, sel) => (projects.as_slice(), *sel),
        _ => return,
    };

    let area = centered_rect(50, 50, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let selected = i == sel;
            let prefix = if selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::text())
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(th::accent())),
                Span::styled(format!(" 🚀 {} ", p), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Dispatch oracle — step 1/2: pick a project — ↑/↓, Enter, Esc ")
            .border_style(Style::default().fg(th::accent())),
    );
    frame.render_widget(list, area);
}

/// Centered overlay modal for the 3-step Telegram setup wizard.
fn draw_telegram_setup_modal(frame: &mut Frame, app: &App) {
    let (step_num, step_label, hint, masked, value): (u8, &str, String, bool, String) = match &app.input_mode {
        InputMode::TelegramSetupToken => (
            1,
            "BOT_TOKEN",
            "Paste the bot token from @BotFather. Bracketed paste handles long tokens — no need to type.".to_string(),
            true,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupChatId(_) => (
            2,
            "CHAT_ID",
            "Numeric chat id. Get yours by sending /start to @userinfobot on Telegram.".to_string(),
            false,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupUserId(_, chat) => (
            3,
            "ALLOWED user_ids (optional)",
            format!(
                "Comma-separated user_ids allowed to talk to the bot (chat_id={}). Esc to skip.",
                chat
            ),
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
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("· Step {}/3", step_num),
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("{}: ", step_label),
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(th::accent2())),
            Span::styled(display, Style::default().fg(th::text()).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "    The whole flow is inline — no shell required.",
            Style::default().fg(th::dim()),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Telegram Setup ")
        .border_style(Style::default().fg(th::accent2()));
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
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(th::accent2())),
            Span::styled(display, Style::default().fg(th::text()).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(th::dim()),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(th::accent2()));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
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
    // Both derive from Tab::ORDER — reorder the tab bar there, not here.
    let titles: Vec<&str> = Tab::ORDER.iter().map(|t| t.title()).collect();
    let selected = app.tab.index();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" OmegaOS "),
        )
        .select(selected)
        // All tabs use the same color when inactive; active tab is
        // accent+bold+underline — the underline is the non-color cue so
        // focus survives bold-as-bright emulators (a11y: never hue-only).
        // No background fill — keeps the toolbar visually clean and avoids
        // any single tab looking "different" beyond just being highlighted.
        .style(Style::default().fg(th::dim()))
        .highlight_style(
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, area);
}

/// A dim `─── label ───` group header row for a grouped left-list (same grammar
/// as the Menu tab's section headers).
fn group_header(label: &str) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("  ─── {} ───", label),
        Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
    )))
}

/// A selectable section row for a grouped left-list. `current` = this is the
/// cursor row; `focused_sel` = cursor row AND the list panel holds focus (full
/// cyan highlight). Visual selection is baked into the item — the `ListState`
/// only drives scrolling.
fn section_row(label: String, current: bool, focused_sel: bool) -> ListItem<'static> {
    let prefix = if current { "▶ " } else { "  " };
    let style = if focused_sel {
        Style::default()
            .fg(th::sel_fg())
            .bg(th::accent())
            .add_modifier(Modifier::BOLD)
    } else if current {
        Style::default().fg(th::text()).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    ListItem::new(Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(th::accent())),
        Span::styled(label, style),
    ]))
}

fn draw_sessions(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_focused = app.session_focus == SessionFocus::List;
    let chat_focused = matches!(
        app.session_focus,
        SessionFocus::Chat | SessionFocus::ChatFullscreen
    );
    let fullscreen = app.session_focus == SessionFocus::ChatFullscreen;

    // Responsive layout. On narrow terminals (mobile / phone-width SSH) a
    // 25/75 split squeezes the Claude preview into an unusable sliver, so we
    // collapse to a SINGLE column: the focused panel fills the whole width —
    // the session list while browsing, the Claude preview while chatting.
    // Wide terminals keep the two-column 25/75 split. Fullscreen is always
    // preview-only. Tab toggles focus, so on mobile it flips list ⇄ chat,
    // each full-width.
    // Breakpoint: below this, a 25/75 split would leave Claude under ~70
    // cols (unusable). At 100 the split still gives the preview ≥75 cols;
    // below it we go single-column so the focused panel gets the full width.
    const NARROW_COLS: u16 = 100;
    let narrow = area.width < NARROW_COLS;

    let (list_area, preview_area): (Option<Rect>, Option<Rect>) = if fullscreen {
        (None, Some(area))
    } else if narrow {
        if chat_focused {
            (None, Some(area))
        } else {
            (Some(area), None)
        }
    } else {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);
        (Some(split[0]), Some(split[1]))
    };

    // Record the REAL panel rects for mouse hit-testing (input.rs
    // handle_mouse) — the Menu tab's geometry-cache pattern. The old
    // `column >= 30` heuristic misrouted clicks whenever the layout above
    // disagreed with it (25% of a wide terminal passes col 30; the narrow
    // single-column list owns the whole width).
    app.sessions_list_area = list_area;
    app.sessions_preview_area = preview_area;

    // Preview-only frame (fullscreen, or narrow + chat-focused): render the
    // Claude view full-width and return.
    if list_area.is_none() {
        app.sessions_rendered_rows.clear();
        app.sessions_list_fits = false;
        if let Some(pa) = preview_area {
            draw_sessions_right(frame, app, pa, chat_focused);
        }
        return;
    }
    let _ = list_focused;

    // ── Left: session list with project headers ─────────────────────────────
    // Rendered-row → entry-index map for click hit-testing (headers = None),
    // mirroring `menu_rendered_actions`. Valid only while the whole list fits
    // (ListState offset 0) — `sessions_list_fits` below guards that.
    let mut rendered_rows: Vec<Option<usize>> = Vec::with_capacity(app.rows.len());
    let mut entry_idx: usize = 0;
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            SessionRow::Header(label) => {
                rendered_rows.push(None);
                ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", label),
                    Style::default()
                        .fg(th::dim())
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            }
            SessionRow::Entry(entry) => {
                rendered_rows.push(Some(entry_idx));
                // `selected` = focused selection bar (you're browsing the list).
                // `active`   = this is the session whose Claude pane fills the
                //              right panel right now — keep it visibly marked in
                //              the left list even while you type in the chat
                //              panel (list unfocused), so you always see WHICH
                //              session you're inside.
                let is_sel = entry_idx == app.selected;
                let item = render_session_item(
                    entry,
                    is_sel && list_focused,
                    is_sel && !list_focused,
                    app.session_badges.get(&entry.session.name).copied(),
                );
                entry_idx += 1;
                item
            }
        })
        .collect();

    let list_border_style = if list_focused {
        Style::default().fg(th::accent())
    } else {
        Style::default().fg(th::dim())
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

    // Context hint: when the highlighted row is the AISB Master and Telegram
    // isn't connected yet, advertise the in-TUI Enter→setup path so a non-dev
    // discovers it.
    let master_cta = app
        .selected_session()
        .map(|e| {
            omega_core::aisb::is_master(&e.session.name)
                && !omega_core::monitor::OmegaTelegramConfig::exists()
        })
        .unwrap_or(false);
    let list_hint = match app.session_focus {
        crate::app::SessionFocus::List if master_cta => "LIST  ★ Enter: connect Telegram  x:kill",
        crate::app::SessionFocus::List => "LIST  x:kill  .:lock  r:rename",
        _ => "CHAT (Tab → list to manage)",
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sessions ({}) — {} ", app.sessions.len(), list_hint))
                .border_style(list_border_style),
        )
        .highlight_style(Style::default());

    // Click → row mapping is only trustworthy at ListState offset 0; a taller
    // list scrolls internally, so clicks then just change panel focus (the
    // same `menu_fits` guard the Menu tab uses).
    app.sessions_list_fits = list_area
        .map(|la| rendered_rows.len() <= la.height.saturating_sub(2) as usize)
        .unwrap_or(false);
    app.sessions_rendered_rows = rendered_rows;

    let mut state = ListState::default().with_selected(rendered_selected);
    if let Some(la) = list_area {
        frame.render_stateful_widget(list, la, &mut state);
    }

    // Wide layout shows the preview alongside the list. (Narrow + list-focused
    // is single-column and already returned above.)
    if let Some(pa) = preview_area {
        draw_sessions_right(frame, app, pa, chat_focused);
    }
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
        Style::default().fg(th::accent2())
    } else {
        Style::default().fg(th::dim())
    };

    // Record the REAL text-area dimensions so main.rs can resize the rmux
    // pane to exactly this width (avoids the right-edge clipping that a
    // terminal-percentage estimate produced).
    app.preview_inner_width = area.width.saturating_sub(2);
    app.preview_inner_height = area.height.saturating_sub(2);

    // Saturate instead of `as`-truncating: a deep history capture can exceed
    // 65 535 lines, and modulo-wrapped math made the scroll bound garbage.
    let total_lines = app.preview_content.lines().count().min(u16::MAX as usize) as u16;
    let viewport_height = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(viewport_height);
    // Publish the real clamp bound so the scroll setters (which don't know the
    // panel geometry) can stop at the top of history.
    app.preview_max_scroll = max_scroll;
    // `preview_scroll` is measured from the TAIL; the Paragraph wants a
    // from-top offset. Clamp the from-tail value, then convert.
    let from_tail = app.preview_scroll.min(max_scroll);
    let scroll = max_scroll.saturating_sub(from_tail);

    let mut preview_lines: Vec<Line> = if app.preview_content.is_empty() {
        vec![Line::from(Span::styled(
            "(select a session to preview)",
            Style::default().fg(th::dim()),
        ))]
    } else if let Some(styled) = &app.preview_styled {
        // Styled path: build colored spans from the pane snapshot so the
        // `/` command-menu selection highlight + Claude's colored UI show.
        //
        // Claude marks the SELECTED menu row with one subtle cue only: the
        // text turns its accent blue (ANSI 94 → rgb 59,142,234) while every
        // sibling stays gray (229,229,229). Empirically (examples/dump_styled)
        // that hue shift is too faint in the mirror — the user "can't tell
        // which one is selected". Since this is a mirror (not the real Claude
        // TTY) we are free to make it unmistakable: any row whose leading
        // visible span is the accent blue gets a full-width highlight bar
        // (▶ marker + blue background + bold).
        //
        // GATED on a menu actually being open: Claude paints OTHER things in
        // the same bright blue (the "Tip:" suggested command, e.g.
        // "/plugin install …"), and the color-only cue turned those into
        // phantom full-width accent bars in the mirror (reported via a
        // Telegram screenshot, 2026-06-11). A completion menu is only ever
        // open while the input box is being typed into, and the snapshot
        // cursor sits in that box — so require the cursor row to be an input
        // line whose content starts with a menu trigger (/ @ #).
        const ACCENT: PreviewColor = PreviewColor::Indexed(12); // Claude's bright-blue selection accent
        let menu_open = app.preview_cursor.map_or(false, |(crow, _, _)| {
            styled.get(crow as usize).map_or(false, |row| {
                let text: String = row.iter().map(|s| s.text.as_str()).collect();
                let after_prompt = text.trim_start().strip_prefix('❯');
                after_prompt.map_or(false, |rest| {
                    matches!(rest.trim_start().chars().next(), Some('/' | '@' | '#'))
                })
            })
        });
        let inner_w = area.width.saturating_sub(2) as usize;
        styled
            .iter()
            .map(|row| {
                let is_selected = menu_open
                    && row
                        .iter()
                        .find(|s| !s.text.trim().is_empty())
                        .map_or(false, |s| s.fg == Some(ACCENT));

                if is_selected {
                    // Selected line: the same highlight every other selection in
                    // the app uses — black on cyan. Named colors track the
                    // terminal theme's palette, so it reads on light AND dark
                    // themes, instead of the old hardcoded dark-navy Rgb bar that
                    // looked like a black block on a light Termius theme.
                    let bar_bg = th::accent();
                    let mut spans: Vec<Span> = Vec::with_capacity(row.len() + 2);
                    let mut width = 0usize;
                    spans.push(Span::styled(
                        "▶ ",
                        Style::default()
                            .fg(th::sel_fg())
                            .bg(bar_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                    width += 2;
                    for sp in row {
                        // Display width (columns), not char count: emoji/CJK
                        // render 2+ cols per char, so counting chars under-pads
                        // and the bar bleeds past the panel. Span::width() uses
                        // unicode-width internally (no new dep needed).
                        width += Span::raw(sp.text.as_str()).width();
                        spans.push(Span::styled(
                            sp.text.clone(),
                            Style::default()
                                .fg(th::sel_fg())
                                .bg(bar_bg)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    // Pad to full inner width so the bar spans the whole row.
                    if width < inner_w {
                        spans.push(Span::styled(
                            " ".repeat(inner_w - width),
                            Style::default().bg(bar_bg),
                        ));
                    }
                    Line::from(spans)
                } else {
                    // Extra emphasis on top of the ANSI baseline (which rmux
                    // already preserves — verified via examples/dump_styled:
                    // bright red 241,76,76 is captured). The user wants
                    // tmux-style visual distinction for: Claude activity
                    // status, user input echo, TodoWrite items, Task-tool
                    // sub-agent dispatches. High-confidence patterns only —
                    // anything that doesn't match falls through to the
                    // straight ANSI render.
                    let row_text: String = row.iter().map(|s| s.text.as_str()).collect();
                    let trimmed = row_text.trim();

                    // ── Pattern catalog (all tight, anchored at line start
                    //    or with multiple co-occurrences to avoid false
                    //    positives on prose) ──────────────────────────────
                    // 1. Activity status footer: "Twisting… (22s · ↓ 1.0k
                    //    tokens · thought for 2s)" and siblings. Anchored on
                    //    the "… (" verb→parenthesis seam — plain conversation
                    //    prose mentioning tokens with a stray '·' or ellipsis
                    //    was painted bold red across the row.
                    let is_activity = trimmed.contains("tokens")
                        && trimmed.contains('·')
                        && trimmed.contains("… (");
                    // 2. User input echo:
                    //    - "❯ <text>"        : Claude Code's input prompt
                    //    - "▶ You: <text>"   : AISB-master mirror format
                    //                          (~/.omega/state/aisb-conversation.log)
                    let is_user_input = (trimmed.starts_with("❯ ") && trimmed.len() > 2)
                        || trimmed.starts_with("▶ You: ");
                    // 2b. AISB-master agent reply echo: "You ▶ <text>".
                    let is_agent_reply = trimmed.starts_with("You ▶ ")
                        || trimmed.starts_with("You ▶");
                    // 3. TodoWrite items: anchored on the status glyph at
                    //    line start (after trim) so a glyph inside a table
                    //    cell or sentence doesn't match.
                    let is_todo_done = trimmed.starts_with("☑ ")
                        || trimmed.starts_with("✅ ")
                        || trimmed.starts_with("✓ ");
                    let is_todo_pending = trimmed.starts_with("☐ ");
                    let is_todo_progress = trimmed.starts_with("⏳ ");
                    let is_todo_failed = trimmed.starts_with("☒ ")
                        || trimmed.starts_with("✗ ")
                        || trimmed.starts_with("⊘ ");
                    // 4. Task-tool / sub-agent dispatch: "● Task(...)" or a
                    //    tool-echo bullet carrying "subagent_type=". The
                    //    bare contains() matched prose DISCUSSING the field
                    //    (this very kind of sentence) — require the line to
                    //    be a ● bullet.
                    let is_task_dispatch = trimmed.starts_with("● Task(")
                        || (trimmed.starts_with('●') && trimmed.contains("subagent_type="));

                    // Order matters: more-specific patterns first so a TODO
                    // line that happens to contain "tokens" isn't mis-typed
                    // as activity. Activity is checked LAST as a fallback.
                    if is_todo_done {
                        // Green bold across the row. Named color so the terminal
                        // theme renders it readable on light AND dark backgrounds.
                        let green = th::success();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(green).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_todo_failed {
                        // Red bold — same family as activity but distinct
                        // by anchor (line-start glyph).
                        let red = th::error();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(red).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_todo_progress {
                        // Magenta bold — actively-running todo stands out
                        // from the pending grey and the done green.
                        let magenta = th::special();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(magenta).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_todo_pending {
                        // Yellow, NOT bold — pending todos read as backlog. Named
                        // color: the bright Rgb yellow was near-invisible on a
                        // light background; the theme's yellow stays readable.
                        let yellow = th::accent2();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| Span::styled(sp.text.clone(), Style::default().fg(yellow)))
                            .collect();
                        Line::from(spans)
                    } else if is_task_dispatch {
                        // Cyan bold — a worker is being spawned, you want to see
                        // it. No forced bg (it became a dark bar on light themes).
                        let cyan = th::accent();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(cyan).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_activity {
                        // Bold red across the row.
                        let red = th::error();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(red).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_user_input {
                        // Bold, preserving Claude's own ANSI fg. NO forced bg:
                        // a hardcoded dark tint renders as an ugly black bar on
                        // light terminal themes (Termius light schemes). BOLD +
                        // the preserved fg distinguish the prompt on any theme.
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                let mut style = Style::default().add_modifier(Modifier::BOLD);
                                if let Some(c) = sp.fg {
                                    style = style.fg(preview_to_color(c));
                                }
                                Span::styled(sp.text.clone(), style)
                            })
                            .collect();
                        Line::from(spans)
                    } else if is_agent_reply {
                        // AISB-master mirror: agent reply line. Soft green fg to
                        // distinguish from user lines. No forced bg (dark bar on
                        // light themes).
                        let green = th::success();
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                Span::styled(
                                    sp.text.clone(),
                                    Style::default().fg(green).add_modifier(Modifier::BOLD),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    } else {
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| {
                                let mut style = Style::default();
                                if let Some(c) = sp.fg {
                                    // bg-aware: keeps the reverse-video swap
                                    // (black-on-gray) visible — see
                                    // preview_fg_color.
                                    style = style.fg(preview_fg_color(c, sp.bg.is_some()));
                                }
                                if let Some(c) = sp.bg {
                                    style = style.bg(preview_to_color(c));
                                }
                                if sp.bold {
                                    style = style.add_modifier(Modifier::BOLD);
                                }
                                // DIM/ITALIC/UNDERLINE pass through so Claude's
                                // secondary (dim) text reads as secondary here too.
                                if sp.dim {
                                    style = style.add_modifier(Modifier::DIM);
                                }
                                if sp.italic {
                                    style = style.add_modifier(Modifier::ITALIC);
                                }
                                if sp.underline {
                                    style = style.add_modifier(Modifier::UNDERLINED);
                                }
                                Span::styled(sp.text.clone(), style)
                            })
                            .collect();
                        Line::from(spans)
                    }
                }
            })
            .collect()
    } else {
        app.preview_content
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect()
    };

    // Bottom-anchor the live terminal mirror. A real terminal keeps the
    // prompt pinned to the bottom while history scrolls up off the top. When
    // the pane content is shorter than the panel (a freshly-started Claude
    // session), pad the TOP so the input line sits at the bottom instead of
    // leaving the lower half of the panel blank.
    let mut bottom_pad: u16 = 0;
    if app.preview_styled.is_some() && max_scroll == 0 {
        let len = preview_lines.len() as u16;
        if len < viewport_height {
            bottom_pad = viewport_height - len;
            let mut padded: Vec<Line> = Vec::with_capacity(viewport_height as usize);
            for _ in 0..bottom_pad {
                padded.push(Line::from(String::new()));
            }
            padded.append(&mut preview_lines);
            preview_lines = padded;
        }
    }

    // ── Mouse drag-selection (tmux-style) ───────────────────────────────
    // Capture the viewport rows as plain text so button-release can resolve
    // the drag rectangle to real text (app.take_preview_selection_text), and
    // paint the in-flight selection REVERSED so you see what you're grabbing.
    let inner_h = area.height.saturating_sub(2) as usize;
    app.preview_screen_rows = (0..inner_h)
        .map(|r| {
            preview_lines
                .get(scroll as usize + r)
                .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
                .unwrap_or_default()
        })
        .collect();
    if app.preview_select_dragging {
        if let Some(((sc, sr), (ec, er))) = app.preview_selection_viewport() {
            for r in sr..=er.min(inner_h.saturating_sub(1)) {
                let li = scroll as usize + r;
                if let Some(line) = preview_lines.get_mut(li) {
                    let from = if r == sr { sc } else { 0 };
                    let to = if r == er { ec + 1 } else { usize::MAX };
                    *line = reverse_cols(line, from, to);
                }
            }
        }
    }

    let scroll_indicator = if max_scroll > 0 {
        format!(" [{}/{}] ", scroll, max_scroll)
    } else {
        String::new()
    };

    // Right side of the title: the previewed session's model + cumulative
    // token consumption (e.g. "opus-4.8 · 45.4M tok"), refreshed off the hot
    // path in main.rs. Replaces the old static key-hint text. Falls back to a
    // tiny interactive marker only when there's no meta yet (non-Claude
    // session, or first 3s before the first scan).
    let meta_suffix = app
        .selected_session()
        .and_then(|e| app.session_meta.get(&e.session.name))
        .map(|(model, tokens)| {
            format!("  ⟨ {} · {} tok ⟩", model, omega_core::claude_meta::fmt_tokens(*tokens))
        });
    let title = match meta_suffix {
        Some(meta) => format!("{}{}{}", preview_title, scroll_indicator, meta),
        None if chat_focused => format!("{}{}  [keys → session]", preview_title, scroll_indicator),
        None => format!("{}{}", preview_title, scroll_indicator),
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
        let inner_w = area.width.saturating_sub(2);
        let inner_h = area.height.saturating_sub(2);

        // Prefer the REAL cursor (row, col) reported by the pane snapshot.
        // That's exactly where the agent's input caret is. Fall back to the
        // last-non-empty-line heuristic only if the snapshot didn't carry a
        // cursor (e.g. history-browsing mode).
        let (cur_row, cur_col) = match app.preview_cursor {
            Some((row, col, _visible)) => (row, col),
            None => {
                let all_lines: Vec<&str> = app.preview_content.lines().collect();
                let (idx, line) = all_lines
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, l)| !l.trim().is_empty())
                    .map(|(i, l)| (i as u16, *l))
                    .unwrap_or((0, ""));
                (idx, line.chars().count() as u16)
            }
        };

        // Map the snapshot row to a viewport row using the same from-top
        // `scroll` offset the Paragraph uses, plus the bottom-anchor padding
        // we prepended so the caret tracks the content down to the bottom.
        let viewport_row = (cur_row + bottom_pad).saturating_sub(scroll);
        if viewport_row < inner_h {
            // Saturating add: a large `area.x` near u16::MAX could otherwise
            // wrap, falsifying the `cursor_x < area.x + area.width` bounds check.
            let cursor_x = area
                .x
                .saturating_add(1)
                .saturating_add(cur_col.min(inner_w.saturating_sub(1)));
            let cursor_y = area.y + 1 + viewport_row;
            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                // Layer 1: painted glyph (always visible).
                let buf = frame.buffer_mut();
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_symbol("▏");
                    cell.set_style(
                        Style::default()
                            .fg(th::accent2())
                            .add_modifier(Modifier::RAPID_BLINK),
                    );
                }
                // Layer 2: OS-native caret.
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
    let _ = fullscreen;
}

fn render_session_item(
    entry: &SessionEntry,
    selected: bool,
    active: bool,
    badge: Option<DoneStatus>,
) -> ListItem<'static> {
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
        th::special()
    } else {
        match entry.session.role {
            SessionRole::Oracle => th::accent2(),
            SessionRole::Worker => th::success(),
            SessionRole::Home => th::info(),
            SessionRole::System => th::dim(),
        }
    };

    let progress_str = match &entry.progress {
        Some(p) => format!(" {} {:.0}%", p.bar(8), p.percentage()),
        None => String::new(),
    };

    let prefix = if selected || active {
        "▶ ".to_string()
    } else {
        "  ".to_string()
    };

    let name_style = if selected {
        Style::default()
            .fg(th::sel_fg())
            .bg(th::accent())
            .add_modifier(Modifier::BOLD)
    } else if active {
        // Connected session, but the list isn't focused (you're typing in the
        // chat panel). A softer cue than the solid selection bar — underlined
        // cyan name + "▶" — so you can always tell WHICH session you're inside
        // without implying the list has keyboard focus.
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else if is_master {
        Style::default()
            .fg(th::special())
            .add_modifier(Modifier::BOLD)
    } else {
        // Passive: plain body text, NO bold — bold on every row carries zero
        // hierarchy. Bold stays reserved for the active session and the
        // selection bar (doctrine: actif = vif + gras, passif = normal).
        Style::default().fg(th::text())
    };

    let protect_marker = if entry.is_protected { "§ " } else { "" };

    // Done/blocked badge from the worker's done.json / worker-blocked signal.
    let (badge_glyph, badge_color) = match badge {
        Some(DoneStatus::DoneClean) => ("+ ", th::success()),
        Some(DoneStatus::Pending) => ("~ ", th::accent2()),
        Some(DoneStatus::Failed) => ("x ", th::error()),
        Some(DoneStatus::Blocked) => ("! ", th::warn()),
        None => ("", th::text()),
    };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(th::accent())),
        Span::raw(entry.tree_prefix.clone()),
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(icon_color),
        ),
        Span::styled(protect_marker, Style::default().fg(th::special())),
        Span::styled(
            badge_glyph,
            Style::default().fg(badge_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(entry.session.name.clone(), name_style),
        Span::styled(progress_str, Style::default().fg(th::accent())),
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
        MenuAction::NewProject | MenuAction::DispatchOracle => "Orchestration",
        MenuAction::Refresh | MenuAction::ToggleProtection | MenuAction::KillSelected => "Session actions",
        MenuAction::KillAll | MenuAction::NuclearCleanup => "Danger zone",
        MenuAction::Restart | MenuAction::Quit => "OmegaOS",
    }
}

fn draw_menu(frame: &mut Frame, app: &mut App, area: Rect) {
    // Build items with section headers so the menu reads as grouped sections.
    let mut items: Vec<ListItem> = Vec::new();
    // Parallel to `items`: rendered-row → action index (None for header/blank
    // rows). Lets a mouse click hit-test which action was clicked.
    let mut rendered_actions: Vec<Option<usize>> = Vec::new();
    let mut last_group: Option<&'static str> = None;
    for (i, action) in MenuAction::all().iter().enumerate() {
        let group = menu_group(action);
        if last_group != Some(group) {
            if last_group.is_some() {
                items.push(ListItem::new(Line::from("")));
                rendered_actions.push(None);
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  ─── {} ───", group),
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            ))));
            rendered_actions.push(None);
            last_group = Some(group);
        }
        rendered_actions.push(Some(i));
        let selected = i == app.menu_selected;
        let prefix = if selected { "▶ " } else { "  " };
        let label_style = if selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
        ])));
    }

    // Record layout for mouse hit-testing (input.rs handle_mouse). `menu_fits`
    // is true only when the whole list shows without scrolling (inner height =
    // area minus the 1-row top+bottom border) → click row maps 1:1 to a row.
    app.menu_area = area;
    app.menu_fits = items.len() <= area.height.saturating_sub(2) as usize;
    app.menu_rendered_actions = rendered_actions;

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
                .title(" Actions — ↑/↓ or click · Enter runs · Ctrl-T text-select/copy ")
                .border_style(Style::default().fg(th::accent())),
        )
        .highlight_style(Style::default()); // selection visual is already baked into items

    let mut state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Build the unified Settings-tab left list: a top-padding blank, the Monitor
/// group, a blank gap, then the Settings group. Returns (items, flat_selected)
/// where `flat_selected` is the rendered row index of the cursor (for the
/// `ListState` so it scrolls to keep the cursor visible).
fn build_settings_list(app: &App) -> (Vec<ListItem<'static>>, usize) {
    let list_focused = !app.detail_focused;
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_selected = 0usize;

    // Top padding.
    items.push(ListItem::new(Line::from("")));

    // ── Group 1: Monitor ────────────────────────────────────────────────────
    items.push(group_header("Monitor"));
    for (i, sec) in MonitorSection::all().iter().enumerate() {
        let current = app.settings_group == 0 && i == app.monitor_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(sec.label().to_string(), current, current && list_focused));
    }

    // Gap between groups.
    items.push(ListItem::new(Line::from("")));

    // ── Group 2: Settings ───────────────────────────────────────────────────
    items.push(group_header("Settings"));
    for (i, sec) in SettingsSection::all().iter().enumerate() {
        let current = app.settings_group == 1 && i == app.settings_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(sec.label().to_string(), current, current && list_focused));
    }

    (items, flat_selected)
}

/// Build the right-panel lines for the currently selected Monitor section.
/// Returns (lines, selected_line) — `selected_line` is the line of the
/// highlighted action (Actions section, focused); 0 otherwise.
fn render_monitor_detail(app: &App) -> (Vec<Line<'static>>, usize) {
    // Inline sub-header for a merged section's second block.
    let sub = |label: &str| {
        Line::from(Span::styled(
            format!("  ─── {} ───", label),
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        ))
    };
    match app.selected_monitor_section() {
        // Connected account + live billing, one panel.
        MonitorSection::AccountBilling => {
            let mut lines = render_monitor_account(app);
            lines.push(Line::from(""));
            lines.push(sub("Billing (live)"));
            lines.extend(render_monitor_billing());
            (lines, 0)
        }
        // Telegram bot config + the project group, one panel.
        MonitorSection::Telegram => {
            let mut lines = render_monitor_telegram();
            lines.push(Line::from(""));
            lines.push(sub("Project group"));
            lines.extend(render_monitor_projects());
            (lines, 0)
        }
        MonitorSection::Actions => render_monitor_actions(app),
    }
}

fn render_monitor_account(app: &App) -> Vec<Line<'static>> {
    use crate::app::ReauthStatus;
    use omega_core::monitor;
    let mut lines: Vec<Line> = vec![Line::from("")];
    if let Some(acc) = monitor::connected_account() {
        lines.push(Line::from(vec![
            Span::raw("    Email:          "),
            Span::styled(acc.email.clone(), Style::default().fg(th::success()).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    Plan:           "),
            Span::styled(
                format!("Claude {}", acc.plan.to_uppercase()),
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   ({})", acc.auth_method)),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "    (no connected account)",
            Style::default().fg(th::dim()),
        )));
    }
    lines.push(Line::from(""));

    // OAuth re-login engine state — drives the guided flow in-place.
    match &app.reauth_status {
        ReauthStatus::Idle => {
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to re-auth Claude (guided OAuth — captures the login URL)",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Generating => {
            lines.push(Line::from(Span::styled(
                "    ⏳ Starting login session and capturing the authorize URL… (~15s)",
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::ShowUrl(url) => {
            lines.push(Line::from(Span::styled(
                "    1) Open this URL in your browser and authorize:",
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("    {}", url),
                Style::default().fg(th::info()).add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    2) Press Enter to paste the code you get back.",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Validating => {
            lines.push(Line::from(Span::styled(
                "    ⏳ Submitting code and waiting for credentials to refresh… (~20s)",
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Done(msg) => {
            lines.push(Line::from(Span::styled(
                format!("    ✓ {}", msg),
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to re-auth again.",
                Style::default().fg(th::dim()),
            )));
        }
        ReauthStatus::Error(msg) => {
            lines.push(Line::from(Span::styled(
                format!("    ✗ {}", msg),
                Style::default().fg(th::error()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to retry.",
                Style::default().fg(th::dim()),
            )));
        }
    }
    lines
}

fn render_monitor_billing() -> Vec<Line<'static>> {
    use omega_core::monitor;
    // Runs every frame while the Billing section is open — memoize the disk
    // reads + process scan (see RENDER_TTL above).
    thread_local! {
        #[allow(clippy::type_complexity)]
        static BILLING_MEMO: std::cell::RefCell<
            Option<(
                std::time::Instant,
                (
                    Option<omega_core::monitor::UsageSnapshot>,
                    Option<u64>,
                    omega_core::monitor::AisbBotStatus,
                ),
            )>,
        > = const { std::cell::RefCell::new(None) };
    }
    let (snap, cache_age, bot_status) = render_memo(&BILLING_MEMO, || {
        (
            monitor::UsageSnapshot::read().ok().flatten(),
            monitor::UsageSnapshot::cache_age_secs(),
            monitor::aisb_bot_status(),
        )
    });
    let mut lines: Vec<Line> = vec![Line::from("")];

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
                    Style::default().fg(th::dim()),
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
            "    (no usage snapshot yet)",
            Style::default().fg(th::dim()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "    ▶ Press Enter to refresh billing now (live OAuth usage check)",
        Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "    ── Usage cache (AISB legacy Python bot, separate from OmegaOS) ──",
        Style::default().fg(th::accent2()),
    )));
    lines.push(Line::from(Span::styled(
        "    Billing reads ~/.omega/state/usage.json (omega usage --check, native OAuth).",
        Style::default().fg(th::dim()),
    )));
    let (bot_icon, bot_color, bot_text) = if bot_status.bot_alive {
        ("●", th::success(), "running")
    } else {
        ("○", th::error(), "not detected")
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
    lines
}

fn render_monitor_telegram() -> Vec<Line<'static>> {
    use omega_core::monitor;
    // Per-frame TOML read → RENDER_TTL memo (see above).
    thread_local! {
        static TG_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, Option<omega_core::monitor::OmegaTelegramConfig>)>,
        > = const { std::cell::RefCell::new(None) };
    }
    let tg_config = render_memo(&TG_MEMO, monitor::OmegaTelegramConfig::read);
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(Span::styled(
        "    Omega Telegram Bot (Rust — this system)",
        Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "      Omega's OWN Telegram bot (no Python, no AISB-Python dependency).",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(Span::styled(
        "      Once configured, text you send via Telegram reaches the OmegaMC",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(Span::styled(
        "      dashboard — your phone-side control surface for the system.",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(""));
    if let Some(cfg) = tg_config {
        let state = if cfg.enabled { "enabled" } else { "configured (disabled)" };
        let color = if cfg.enabled { th::success() } else { th::accent2() };
        lines.push(Line::from(vec![
            Span::raw("    Status:         "),
            Span::styled(state.to_string(), Style::default().fg(color)),
        ]));
        if !cfg.label.is_empty() {
            lines.push(Line::from(format!("    Label:          {}", cfg.label)));
        }
        lines.push(Line::from(format!("    Chat ID:        {}", cfg.chat_id)));
        let sender = if cfg.allow_user_ids.is_empty() {
            "chat_id only (any user in this chat)".to_string()
        } else {
            format!("user_ids {:?}", cfg.allow_user_ids)
        };
        lines.push(Line::from(format!("    Sender filter:  {}", sender)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    ▶ Press Enter to DISCONNECT the bot (two-press confirm)",
            Style::default().fg(th::error()).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    Status:         (not configured)",
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    Before you start: create a bot via @BotFather (save the token) and",
            Style::default().fg(th::text()),
        )));
        lines.push(Line::from(Span::styled(
            "    get your chat id from @userinfobot. The wizard asks for them step by step.",
            Style::default().fg(th::text()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    ▶ Press Enter to set up the bot (guided, no command needed)",
            Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
        )));
    }
    lines
}

fn render_monitor_projects() -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "    Project group (auto-detected)",
            Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    // Per-frame TOML read → RENDER_TTL memo (see above).
    thread_local! {
        static GROUP_MEMO: std::cell::RefCell<
            Option<(
                std::time::Instant,
                Option<omega_core::telegram_group::TelegramGroupConfig>,
            )>,
        > = const { std::cell::RefCell::new(None) };
    }
    match render_memo(&GROUP_MEMO, omega_core::telegram_group::TelegramGroupConfig::load) {
        Some(gcfg) => {
            lines.push(Line::from(vec![
                Span::raw("    Status:         "),
                Span::styled("● Connected", Style::default().fg(th::success()).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(format!(
                "    Group:          {}  ({})",
                if gcfg.group_name.is_empty() { "—".to_string() } else { gcfg.group_name.clone() },
                gcfg.group_id
            )));
            lines.push(Line::from(format!(
                "    Topics:         {} mapped",
                gcfg.topics.len()
            )));
            if !gcfg.topics.is_empty() {
                let names: Vec<String> = gcfg.topics.keys().cloned().collect();
                let preview = names.join(", ");
                let trimmed: String = preview.chars().take(80).collect();
                lines.push(Line::from(format!(
                    "                    {}{}",
                    trimmed,
                    if preview.len() > 80 { " …" } else { "" }
                )));
            }
            lines.push(Line::from(format!("    Set up at:      {}", gcfg.setup_at)));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to change the group id",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )));
        }
        None => {
            lines.push(Line::from(Span::styled(
                "    Status:         ○ Not configured",
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    Setup is automatic:",
                Style::default().fg(th::accent()),
            )));
            lines.push(Line::from(Span::styled(
                "      1. Create a Telegram supergroup, enable Topics in its settings",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "      2. Add the bot to the group and make it admin",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "      3. That's it — the bot auto-detects the promotion, persists the",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "         group, creates one topic per project, and DMs you a confirmation.",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to set the group id manually (guided, no command needed)",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )));
        }
    }
    lines
}

fn render_monitor_actions(app: &App) -> (Vec<Line<'static>>, usize) {
    let detail_active = app.detail_focused;
    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut selected_line: usize = 0;

    if detail_active {
        lines.push(Line::from(Span::styled(
            "  ↑/↓ navigate · Enter runs · Tab → back to list",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab → focus this panel to run actions (every section is also Enter-actionable)",
            Style::default().fg(th::dim()),
        )));
    }
    lines.push(Line::from(""));

    for (i, action) in MonitorAction::all().iter().enumerate() {
        let selected = detail_active && i == app.monitor_action_selected;
        if selected { selected_line = lines.len(); }
        let prefix = if selected { "  ▶ " } else { "    " };
        let label_style = if selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  This tab refreshes every 5s.",
        Style::default().fg(th::dim()),
    )));

    (lines, selected_line)
}

fn render_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn pct_color(pct: f32) -> Color {
    if pct < 50.0 { th::success() }
    else if pct < 80.0 { th::accent2() }
    else { th::error() }
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

/// Build the unified Projects-tab left list: a top-padding blank, the System
/// info group, a blank gap, then the Projects group. Returns (items,
/// flat_selected) — the rendered row index of the cursor for `ListState`
/// scroll tracking.
fn build_projects_list(app: &App) -> (Vec<ListItem<'static>>, usize) {
    let list_focused = !app.detail_focused;
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_selected = 0usize;

    // Top padding.
    items.push(ListItem::new(Line::from("")));

    // ── Group 1: System info sections ────────────────────────────────────────
    // Named "System" (not "Projects") so it never collides with the tab name:
    // the tab is Projects, and these rows are the Atlas/Oracle/Workers/Rules
    // status readouts, not projects.
    items.push(group_header("System"));
    for (i, sec) in InfoSection::all().iter().enumerate() {
        let current = app.projects_group == 0 && i == app.info_section_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(sec.label().to_string(), current, current && list_focused));
    }

    // Gap between groups.
    items.push(ListItem::new(Line::from("")));

    // ── Group 2: Projects ────────────────────────────────────────────────────
    items.push(group_header("Projects"));
    if app.project_registry.projects.is_empty() {
        let current = app.projects_group == 1;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(
            "(no projects — press n to add)".to_string(),
            current,
            current && list_focused,
        ));
    } else {
        for (i, project) in app.project_registry.projects.iter().enumerate() {
            let current = app.projects_group == 1 && i == app.projects_selected;
            if current {
                flat_selected = items.len();
            }
            let icon = project.icon.as_deref().unwrap_or("▣");
            // 🔕 marks a project whose Telegram toggle is OFF.
            let tg_mark = if project.telegram_enabled() { "" } else { " 🔕" };
            items.push(section_row(
                format!("{} {}{}", icon, project.name, tg_mark),
                current,
                current && list_focused,
            ));
        }
    }

    (items, flat_selected)
}

fn render_project_detail(app: &App) -> Vec<Line<'static>> {
    let Some(project) = app.selected_project() else {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No project selected.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  ▶ Press n to add a project (register an existing folder).",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "    Or press Enter on this empty list to do the same.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  For a brand-new project (scaffold + provision), use the Menu tab → New project.",
                Style::default().fg(th::dim()),
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
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Path
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Path"),
            Style::default().fg(th::accent2()),
        ),
        Span::raw(project.path.to_string_lossy().to_string()),
    ]));

    // Git email
    if let Some(ref email) = project.git_email {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Git email"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(email.clone()),
        ]));
    }

    // Telegram topic
    if let Some(topic_id) = project.telegram_topic_id {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Telegram topic"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(topic_id.to_string()),
        ]));
    }

    // Oracle session
    if let Some(ref oracle) = project.oracle_session {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Oracle session"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(oracle.clone()),
        ]));
    }

    // Created at
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Created"),
            Style::default().fg(th::accent2()),
        ),
        Span::raw(project.created_at.clone()),
    ]));

    lines.push(Line::from(""));


    // Planner + bootstrap status (if .planner/tracker.json etc. exist).
    // Per-frame JSON read + parse → RENDER_TTL memo, keyed by the project
    // path so switching the selection refreshes immediately.
    thread_local! {
        #[allow(clippy::type_complexity)]
        static PROJECT_MEMO: std::cell::RefCell<
            Option<(
                std::time::Instant,
                std::path::PathBuf,
                (
                    Option<omega_core::planner::PlanTracker>,
                    Option<omega_core::bootstrap::BootstrapState>,
                ),
            )>,
        > = const { std::cell::RefCell::new(None) };
    }
    let (tracker, bootstrap) = PROJECT_MEMO.with(|cell| {
        let mut cached = cell.borrow_mut();
        if let Some((at, path, v)) = cached.as_ref() {
            if *path == project.path && at.elapsed() < RENDER_TTL {
                return v.clone();
            }
        }
        let v = (
            omega_core::planner::PlanTracker::load(&project.path),
            omega_core::bootstrap::BootstrapState::load(&project.path),
        );
        *cached = Some((std::time::Instant::now(), project.path.clone(), v.clone()));
        v
    });
    if let Some(ref tracker) = tracker {
        let status = tracker.status();
        lines.push(Line::from(Span::styled(
            "  ── Planner ──",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
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
                Style::default().fg(th::accent2()),
            )));
        }
        if status.failed > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} failed", status.failed),
                Style::default().fg(th::error()),
            )));
        }
        if status.ready > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} ready to start", status.ready),
                Style::default().fg(th::success()),
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
            let phase_icon = if phase_done { "[+]" } else if phase.id == status.active_phase { "[~]" } else { "[ ]" };
            lines.push(Line::from(format!(
                "    {} Phase {}: {}",
                phase_icon, phase.id, phase.name
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  No planner active — run /planner to create a plan.",
            Style::default().fg(th::dim()),
        )));
    }

    // Bootstrap status (read in the PROJECT_MEMO block above)
    if let Some(ref state) = bootstrap {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Bootstrap Pipeline ──",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )));
        for phase in omega_core::bootstrap::BootstrapPhase::all() {
            let done = state.is_done(*phase);
            let active = *phase == state.current_phase && !done;
            let icon = if done {
                "[+]"
            } else if active {
                "[~]"
            } else {
                "[ ]"
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
    // ── ONE actions menu — every project feature, one line each, [key] style. ──
    let tg_on = project.telegram_enabled();
    lines.push(Line::from(Span::styled(
        "  ── Actions ──",
        Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
    )));
    let action = |key: &str, label: String| -> Line<'static> {
        Line::from(Span::styled(
            format!("    [{}]{}{}", key, " ".repeat(7usize.saturating_sub(key.len())), label),
            Style::default().fg(th::accent()),
        ))
    };
    lines.push(action("Enter", "Open in terminal".to_string()));
    lines.push(action("d", "Dispatch oracle".to_string()));
    lines.push(action("p", "Run planner".to_string()));
    lines.push(action(
        "T",
        format!(
            "Telegram topic: {} → toggle {}",
            if tg_on { "🔔 ON" } else { "🔕 OFF" },
            if tg_on { "OFF" } else { "ON" },
        ),
    ));
    lines.push(action("x", "Delete… (1 OmegaOS · 2 + local folder · 3 + GitHub)".to_string()));
    lines.push(action("D", "Quick delete local machine (press twice)".to_string()));
    lines.push(action("n", "Add another project".to_string()));

    lines
}

fn draw_settings(frame: &mut Frame, app: &mut App, area: Rect) {
    // Detail panel + the focused-line for auto-scroll depend on which group the
    // cursor sits in: the Monitor group renders the monitor detail, the Settings
    // group renders the provider fields.
    let on_monitor = app.settings_on_monitor();
    let providers = app.providers();
    // Inner text width of the detail panel — gallery rows ellipsize their
    // blurbs against it instead of hard-clipping mid-word at the border.
    // (-2 borders, -1 margin for percentage-split rounding.)
    let detail_inner_w = if app.detail_fullscreen {
        area.width.saturating_sub(2)
    } else {
        (area.width.saturating_mul(75) / 100).saturating_sub(3)
    } as usize;
    let (lines, selected_field_line) = if on_monitor {
        render_monitor_detail(app)
    } else {
        render_settings_detail(app, &providers, detail_inner_w)
    };
    let section_label: String = if on_monitor {
        app.selected_monitor_section().label().to_string()
    } else {
        app.selected_settings_section().label().to_string()
    };

    // Auto-scroll to keep the selected field visible
    if app.detail_focused {
        let panel_height = area.height.saturating_sub(2);
        let field_line = selected_field_line as u16;
        if field_line < app.detail_scroll {
            app.detail_scroll = field_line.saturating_sub(1);
        } else if field_line >= app.detail_scroll + panel_height {
            app.detail_scroll = field_line.saturating_sub(panel_height.saturating_sub(2));
        }
    }

    // Publish the real clamp bound (the preview_max_scroll contract) and pin
    // the offset to it: End / wheel-overscroll stop at the last content line
    // instead of scrolling the Paragraph into blank space. The horizontal
    // split below keeps the full height, so area.height is the panel height
    // in both the split and fullscreen paths.
    app.detail_max_scroll =
        (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    // Fullscreen detail mode: skip the left list, detail takes 100% width
    if app.detail_fullscreen {
        let title = format!(" {}  [FULLSCREEN — Tab/Tab-Tab to exit] ", section_label);
        let paragraph = Paragraph::new(lines)
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(th::accent2())),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { th::accent() } else { th::dim() };
    let detail_border = if app.detail_focused { th::accent2() } else { th::dim() };

    // ── Left: grouped section list (Monitor group + Settings group) ──────────
    let (items, rendered_selected) = build_settings_list(app);

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

    let mut settings_list_state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, split[0], &mut settings_list_state);

    // ── Right: details panel ────────────────────────────────────────────────
    let detail_title = if app.detail_focused {
        format!(
            " {}  [FOCUSED — ↑/↓ navigate, Tab → list, Tab-Tab → fullscreen] ",
            section_label
        )
    } else {
        format!(" {} ", section_label)
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
/// `inner_w` is the detail panel's inner text width (gallery blurb budget).
fn render_settings_detail(
    app: &App,
    providers: &omega_core::providers::ProvidersConfig,
    inner_w: usize,
) -> (Vec<Line<'static>>, usize) {
    use crate::app::{fields_for_section, SettingsField};
    let fields = fields_for_section(app.selected_settings_section(), providers, &app.config);
    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut selected_line: usize = 0;

    let detail_active = app.detail_focused;

    // Top hint
    if detail_active {
        lines.push(Line::from(Span::styled(
            "  ↑/↓ navigate · Enter activates · x clear field (2×) · Tab → back to list",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab → focus this panel to interact (Install/Uninstall/Edit)",
            Style::default().fg(th::dim()),
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
                        .fg(th::sel_fg())
                        .bg(th::accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(th::accent())
                        .add_modifier(Modifier::BOLD)
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(label.clone(), label_style),
                ]));
                if is_selected {
                    let cmd_preview = if command.chars().count() > 100 {
                        // Truncate by chars, not bytes: `&command[..100]` panics
                        // if byte 100 splits a multi-byte UTF-8 char.
                        format!("{}…", command.chars().take(100).collect::<String>())
                    } else {
                        command.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("      → Enter runs:  {}", cmd_preview),
                        Style::default().fg(th::dim()),
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
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(display, Style::default().fg(th::accent())),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to edit (opens input modal)",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Toggle { label, current, .. } => {
                let badge = if *current { "● on" } else { "○ off" };
                let badge_color = if *current { th::success() } else { th::dim() };
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(badge.to_string(), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to toggle",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Select { label, options, current_index, .. } => {
                let display = options
                    .get(*current_index)
                    .cloned()
                    .unwrap_or_else(|| "(not set)".to_string());
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(display, Style::default().fg(th::accent()).add_modifier(Modifier::BOLD)),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to choose (↑/↓ selector, no typing)",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Info(text) => {
                if text.is_empty() {
                    lines.push(Line::from(""));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(th::dim()),
                    )));
                }
            }
        }
    }

    // Theme gallery: one swatch line per theme, each painted with its OWN
    // palette so the user sees every option at a glance. The Select overlay
    // (Enter on "Active theme") live-previews the whole TUI while arrowing.
    if app.selected_settings_section() == crate::app::SettingsSection::Theme {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Gallery — Enter on \"Active theme\" opens the selector; \u{2191}/\u{2193} previews live",
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(""));
        let active = crate::theme::active();
        for id in crate::theme::ThemeId::all() {
            let p = id.palette();
            // Paint each row on ITS OWN theme background so the gallery shows
            // the real contrast between palettes, not just accent swaps.
            let row_bg = p.bg.unwrap_or(Color::Reset);
            let on = |c: Color| Style::default().fg(c).bg(row_bg);
            let marker = if *id == active { "  \u{25b6} " } else { "    " };
            let mut spans: Vec<Span> = vec![
                Span::styled(marker.to_string(), on(p.accent).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:28}", id.label()),
                    if *id == active {
                        on(p.accent).add_modifier(Modifier::BOLD)
                    } else {
                        on(p.text)
                    },
                ),
            ];
            // warn replaces success in the strip: on mono themes success IS
            // the accent (duplicate swatch), while warn is a real distinct role.
            for c in [p.accent, p.accent2, p.warn, p.error, p.info, p.special, p.dim] {
                spans.push(Span::styled("\u{2588}\u{2588}", on(c)));
            }
            // Char-safe ellipsis on the blurb: 4 marker + 28 label +
            // 14 swatches + 2 gap = 48 fixed columns before it.
            let blurb_budget = inner_w.saturating_sub(48);
            let blurb = if id.blurb().chars().count() > blurb_budget && blurb_budget > 1 {
                format!("{}…", id.blurb().chars().take(blurb_budget - 1).collect::<String>())
            } else {
                id.blurb().to_string()
            };
            spans.push(Span::styled(format!("  {}", blurb), on(p.dim)));
            lines.push(Line::from(spans));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Config files: ~/.omega/config.toml  ~/.omega/providers.toml",
        Style::default().fg(th::dim()),
    )));
    return (lines, selected_line);
}

/// Mask sensitive characters in an inline input (show prefix/suffix only).
fn mask_inline(s: &str) -> String {
    // Count chars, not bytes: byte slicing/length panics or miscounts on
    // multi-byte UTF-8 (emoji, accented, CJK).
    let char_count = s.chars().count();
    if char_count <= 6 {
        "•".repeat(char_count)
    } else {
        // char-boundary-safe: `&s[..3]` panics if byte 3 splits a multi-byte
        // char. Take 3 chars, not 3 bytes.
        let prefix: String = s.chars().take(3).collect();
        format!("{}…{}", prefix, "•".repeat(char_count - 3))
    }
}

fn mask_key(key: &str) -> String {
    // Count/slice by chars, not bytes: `&key[..4]` and `&key[key.len()-4..]`
    // panic if a byte index splits a multi-byte UTF-8 char (emoji, CJK).
    let char_count = key.chars().count();
    if key.is_empty() {
        "(not set)".to_string()
    } else if char_count <= 8 {
        "•".repeat(char_count)
    } else {
        let prefix: String = key.chars().take(4).collect();
        let suffix: String = key.chars().skip(char_count - 4).collect();
        format!("{}…{}", prefix, suffix)
    }
}

fn draw_info(frame: &mut Frame, app: &mut App, area: Rect) {
    // Detail + label depend on the group: the System info group renders the
    // info sections, the Projects group renders the selected project's detail.
    let on_projects = app.on_projects_group();
    let (lines, scroll_target) = if on_projects {
        (render_project_detail(app), 0usize)
    } else {
        match app.selected_info_section() {
            InfoSection::Atlas => (render_info_atlas(), 0),
            InfoSection::AisbAgents => render_info_aisb_agents(app),
            InfoSection::Oracle => (render_info_oracle(), 0),
            InfoSection::Workers => (render_info_workers(), 0),
            InfoSection::Rules => (render_info_rules(), 0),
        }
    };
    let section_label: String = if on_projects {
        app.selected_project()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Projects".to_string())
    } else {
        app.selected_info_section().label().to_string()
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

    // Publish + pin the scroll bound (same contract as draw_settings): End
    // and wheel-overscroll stop at the content edge, never a blank panel.
    app.detail_max_scroll =
        (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    // Fullscreen detail
    if app.detail_fullscreen {
        let title = format!(" {}  [FULLSCREEN — Tab/Tab-Tab to exit] ", section_label);
        let paragraph = Paragraph::new(lines)
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(th::accent2())),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { th::accent() } else { th::dim() };
    let detail_border = if app.detail_focused { th::accent2() } else { th::dim() };

    // ── Left: grouped list (System info group + Projects group) ──────────────
    let (items, rendered_selected) = build_projects_list(app);

    let list_title = if list_focused {
        " ▶ FOCUSED Projects — ↑/↓ select, Tab → focus detail "
    } else {
        " Projects — Tab to focus list "
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut info_list_state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, split[0], &mut info_list_state);

    let detail_title = if app.detail_focused {
        format!(
            " {}  [FOCUSED — ↑/↓ scroll, Tab → list, Tab-Tab → fullscreen] ",
            section_label
        )
    } else {
        format!(" {} ", section_label)
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

/// Marketing tab — 25/75 split: left = selectable list of marketing-enabled
/// projects (status glyph + name + posts), right = the selected project's
/// detail + an ACTIONS block. Symmetric to the Projects tab (`draw_info`), same
/// theme grammar. Fast/local status only (see `omega_core::marketing`).
fn draw_marketing(frame: &mut Frame, app: &mut App, area: Rect) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused { th::accent() } else { th::dim() };
    let detail_border = if app.detail_focused { th::accent2() } else { th::dim() };

    // ── Left: selectable project list ────────────────────────────────────────
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from("")));
    items.push(group_header("Marketing"));
    let mut rendered_selected = 1usize;
    if app.marketing_projects.is_empty() {
        items.push(section_row(
            "(no marketing projects — F5 to scan)".to_string(),
            true,
            list_focused,
        ));
        rendered_selected = items.len() - 1;
    } else {
        for (i, p) in app.marketing_projects.iter().enumerate() {
            let current = i == app.marketing_selected;
            if current {
                rendered_selected = items.len();
            }
            let posts = if p.calendar_posts > 0 {
                format!(" · {}p", p.calendar_posts)
            } else {
                String::new()
            };
            items.push(section_row(
                format!("{} {}{}", p.glyph(), p.name, posts),
                current,
                current && list_focused,
            ));
        }
    }

    let list_title = if list_focused {
        " ▶ FOCUSED Marketing — ↑/↓ select, Tab → detail "
    } else {
        " Marketing — Tab to focus list "
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());
    let mut state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, split[0], &mut state);

    // ── Right: detail of the selected project ────────────────────────────────
    let lines = render_marketing_detail(app);
    let section_label = app
        .selected_marketing_project()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Marketing".to_string());

    app.detail_max_scroll =
        (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    let detail_title = if app.detail_focused {
        format!(" {}  [FOCUSED — ↑/↓ scroll, Tab → list] ", section_label)
    } else {
        format!(" {} ", section_label)
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

/// Right-pane detail for the selected marketing project (status + ACTIONS).
fn render_marketing_detail(app: &App) -> Vec<Line<'static>> {
    let Some(p) = app.selected_marketing_project() else {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No marketing project selected.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  A project is marketing-enabled once it has a marketing/ directory.",
                Style::default().fg(th::dim()),
            )),
            Line::from(Span::styled(
                "  Press F5 to scan for projects.",
                Style::default().fg(th::success()).add_modifier(Modifier::BOLD),
            )),
        ];
    };

    let field = |label: &str| {
        Span::styled(
            format!("  {:20}", label),
            Style::default().fg(th::accent2()),
        )
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("  {} ", p.glyph())),
            Span::styled(
                p.name.clone(),
                Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![field("Path"), Span::raw(p.path.to_string_lossy().to_string())]),
        Line::from(vec![field("Slug"), Span::raw(p.slug.clone())]),
        Line::from(vec![
            field("Calendar posts"),
            Span::raw(if p.has_content {
                p.calendar_posts.to_string()
            } else {
                "— (no calendar)".to_string()
            }),
        ]),
        Line::from(vec![
            field("Daily engine"),
            if p.engine_on {
                Span::styled("ON", Style::default().fg(th::success()).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("off", Style::default().fg(th::dim()))
            },
        ]),
        Line::from(vec![
            field("Accounts"),
            match p.accounts {
                Some(n) => Span::raw(n.to_string()),
                None => Span::styled("…", Style::default().fg(th::dim())),
            },
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ─── Marketing machine ───",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            field("Layers"),
            Span::styled(format!("context {}   ", if p.has_context { "✓" } else { "·" }), Style::default().fg(if p.has_context { th::success() } else { th::dim() })),
            Span::styled(format!("strategy {}   ", if p.has_strategy { "✓" } else { "·" }), Style::default().fg(if p.has_strategy { th::success() } else { th::dim() })),
            Span::styled(format!("copy {}   ", if p.has_copy { "✓" } else { "·" }), Style::default().fg(if p.has_copy { th::success() } else { th::dim() })),
            Span::styled(format!("visual {}   ", if p.has_visual { "✓" } else { "·" }), Style::default().fg(if p.has_visual { th::success() } else { th::dim() })),
            Span::styled(format!("branding {}", if p.has_branding { "✓" } else { "·" }), Style::default().fg(if p.has_branding { th::success() } else { th::dim() })),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ─── Actions ───",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Enter  ", Style::default().fg(th::accent()).add_modifier(Modifier::BOLD)),
            Span::raw("💬 Parler marketing (session scopée)"),
        ]),
        Line::from(vec![
            Span::styled("  p      ", Style::default().fg(th::accent()).add_modifier(Modifier::BOLD)),
            Span::raw("publier (dry-run)"),
        ]),
        Line::from(vec![
            Span::styled("  F5     ", Style::default().fg(th::accent()).add_modifier(Modifier::BOLD)),
            Span::raw("refresh"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ─── Quick paths ───",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            field("Marketing dir"),
            Span::raw(p.marketing_dir().to_string_lossy().to_string()),
        ]),
        Line::from(vec![
            field("Calendar (90d)"),
            Span::raw(p.calendar_md().to_string_lossy().to_string()),
        ]),
    ];
    lines.push(Line::from(""));
    lines
}

/// Returns (lines, selected_agent_line) for auto-scroll.
fn render_info_aisb_agents(app: &App) -> (Vec<Line<'static>>, usize) {
    use omega_core::aisb_agents::AisbAgent;
    let agents = AisbAgent::all();
    let selected_def = agents[app.info_agent_selected].definition();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            // Count derived from the roster (see InfoSection::label) so the
            // blurb can't drift when an agent joins.
            format!(
                "  AISB = AI Super Brain — {} Matrix agents the Master delegates to.",
                agents.len()
            ),
            Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  ↑/↓ navigate the agent list below.",
            Style::default().fg(th::dim()),
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
                .fg(th::sel_fg())
                .bg(th::accent2())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(format!("{:13}", def.name), name_style),
            Span::styled(
                format!(" {} ", def.model.name()),
                Style::default().fg(th::special()),
            ),
            Span::raw(format!("· {}", def.role)),
        ]));
    }

    // Detail card for the selected agent
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  ── {} ──", selected_def.name),
        Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  \"{}\"", selected_def.tagline),
        Style::default().fg(th::dim()),
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
        Style::default().fg(th::accent2()),
    )));
    for r in selected_def.responsibilities {
        lines.push(Line::from(format!("    • {}", r)));
    }
    (lines, selected_agent_line)
}

fn render_info_atlas() -> Vec<Line<'static>> {
    let master = omega_core::aisb::MASTER_SESSION_NAME;
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Ω  ATLAS — the Director brain (omega-tg-bot.ts)",
            Style::default().fg(th::special()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Atlas is the single brain reached over Telegram: you message it,"),
        Line::from("  it classifies intent and dispatches to the right oracle, agent or"),
        Line::from("  skill. The 15 Matrix agents (see 'AISB Agents') are its faculties."),
        Line::from("  One conversation, many agents, shared evolution."),
        Line::from(""),
        Line::from(Span::styled("  Live session", Style::default().fg(th::accent()))),
        Line::from(format!("    {}  — live viewer of the Atlas Telegram conversation", master)),
        Line::from("    Select it in the Sessions tab + Tab to watch the live conversation."),
        Line::from(""),
        Line::from(Span::styled("  Telegram bridge", Style::default().fg(th::accent()))),
        Line::from("    Set up:  Settings tab → Telegram & projects → Enter (or press 'T')"),
        Line::from("             to connect Telegram (guided wizard — no command needed)."),
        Line::from("    Once set, Atlas streams its replies + accepts voice / documents /"),
        Line::from("    photos (transcribed + analysed). Per-project topics on sync."),
        Line::from(""),
        Line::from(Span::styled("  OmegaMC dashboard & gateway", Style::default().fg(th::accent()))),
        Line::from("    The phone-side web control surface (agents, conversations, tasks,"),
        Line::from("    swarms) backed by the on-demand gateway. Repo: agentik-os/"),
        Line::from("    agentik-telegram (MIT); runs as a Docker container on :8080."),
        Line::from("    Open:  Settings tab → Actions → 'O' (Open Dashboard) — launches it"),
        Line::from("           from ~/.omega/repos/omega-mc when installed."),
        Line::from("    The 15 AISB agents are mapped into its registry"),
        Line::from("    (config/omega-aisb.yaml)."),
        Line::from(""),
        Line::from(Span::styled(
            "  One brain (Atlas), one Telegram channel, one dashboard — all 15 agents.",
            Style::default().fg(th::dim()),
        )),
    ]
}

fn render_info_oracle() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ORACLE — the brain of every dispatched mission",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
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
            Style::default().fg(th::accent()),
        )),
        Line::from("    Sessions: oracle-<Project>     (1st)"),
        Line::from("              oracle-<Project>-2   (parallel oracle)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Rules enforced",
            Style::default().fg(th::accent()),
        )),
        Line::from("    R-19 — Rubric before execution"),
        Line::from("    R-21 — Multi-grader consensus ≥ 2/3"),
        Line::from("    R-28 — Token budget enforced (default 500K)"),
        Line::from("    L3   — Workers must decide, not wait"),
        Line::from(""),
        Line::from(Span::styled(
            "  ORACLES NEVER write code — they decide who does, then verify.",
            Style::default().fg(th::dim()),
        )),
    ]
}

fn render_info_workers() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  WORKERS — ephemeral execution sessions",
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  A worker is one rmux session running an agent (usually Claude) with"),
        Line::from("  a specific task prompt. Workers are short-lived: spawned by an Oracle,"),
        Line::from("  they execute one task, signal done, and the patrol cleans up."),
        Line::from(""),
        Line::from(Span::styled(
            "  Naming",
            Style::default().fg(th::accent()),
        )),
        Line::from("    <Project>-worker-<task>    e.g. Causio-worker-auth"),
        Line::from(""),
        Line::from(Span::styled(
            "  Lifecycle",
            Style::default().fg(th::accent()),
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
            Style::default().fg(th::accent()),
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
    use omega_core::rules::{all_rules, laws, RuleCategory, RuleKind};
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  System invariants — every rule has a reason, a date, and who it binds.",
            Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // ── LAWS — inviolable, render first, visually distinct ──
    let law_list = laws();
    if !law_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "  THE LAWS — inviolable, bind every agent, override every rule",
            Style::default().fg(th::special()).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for r in &law_list {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:14}", r.id),
                    Style::default().fg(th::special()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    r.title.to_string(),
                    Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                format!("    {}", r.description),
                Style::default().fg(th::text()),
            )));
            let applies = if r.applies_to.is_empty() {
                "all agents".to_string()
            } else {
                r.applies_to.iter().map(|a| a.name()).collect::<Vec<_>>().join(", ")
            };
            lines.push(Line::from(Span::styled(
                format!("    Applies to: {}  ·  Added: {}", applies, r.added_at),
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(Span::styled(
                format!("    Why: {}", r.reason),
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(""));
        }
    }

    let categories = [
        RuleCategory::Universal,
        RuleCategory::QualityGate,
        RuleCategory::Orchestration,
        RuleCategory::Reporting,
        RuleCategory::Safety,
    ];

    for cat in &categories {
        let rules: Vec<_> = all_rules()
            .into_iter()
            .filter(|r| r.kind == RuleKind::Rule && r.category == *cat)
            .collect();
        if rules.is_empty() {
            continue;
        }
        lines.push(Line::from(Span::styled(
            format!("  ── {} ──", cat.label()),
            Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
        )));
        for r in rules {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:14}", r.id),
                    Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
                ),
                Span::raw(r.title.to_string()),
            ]));
            lines.push(Line::from(Span::styled(
                format!("    {}", r.description),
                Style::default().fg(th::text()),
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
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(Span::styled(
                format!("    Why: {}", r.reason),
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(""));
        }
    }
    lines
}

fn draw_help(frame: &mut Frame, app: &mut App, area: Rect) {
    let cy = Style::default().fg(th::accent()).add_modifier(Modifier::BOLD);
    let yl = Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD);
    let wh = Style::default().fg(th::text());
    let gr = Style::default().fg(th::dim());
    let mg = Style::default().fg(th::special());

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
            Span::styled("  Ω  ", Style::default().fg(th::accent()).add_modifier(Modifier::BOLD)),
            Span::styled("OmegaOS", Style::default().fg(th::text()).add_modifier(Modifier::BOLD)),
            Span::styled("  —  Agentic Terminal Operating System", gr),
            Span::styled(concat!("   v", env!("CARGO_PKG_VERSION")), cy),
        ]),
        Line::from(""),

        section("Navigation"),
        key("← / →", "Switch tabs"),
        key("Shift+Tab", "Previous tab"),
        key("F1", "Open this Help tab"),
        key("Drag", "Select text in the session view → copies to clipboard (OSC 52)"),
        key("Ctrl+T", "Toggle mouse capture (off = native text selection)"),
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
        Line::from(Span::styled("    Same pattern on Sessions, Settings, Projects.", gr)),
        Line::from(""),

        section("Sessions"),
        // DESIGN-016: j/k are deliberately swallowed on Sessions (operator
        // preference) — only the arrows navigate; don't document dead keys.
        key("↑ / ↓", "Navigate sessions"),
        key("Enter / Tab", "Focus chat (talk to agent)"),
        key("Tab-Tab", "Chat fullscreen (hide list)"),
        key("r  /  R", "Rename selected session"),
        key("x  /  X", "Kill session (skip if locked)"),
        key(".", "Toggle lock/protection"),
        key("/", "Filter the session list (empty+Enter clears)"),
        key("b", "Jump to next blocked/failed session"),
        key("PgUp / PgDn", "Scroll preview"),
        key("Home / End", "Top / bottom (tail-follow)"),
        Line::from(""),

        section("Menu — Agent Launchers"),
    ];

    let launchers = [
        ("c", "Claude"), ("o", "Codex"), ("g", "Gemini"),
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

        section("Settings → Monitor group"),
        key("↑ / ↓ + Enter", "Run highlighted action"),
        key("L", "Login Claude (OAuth)"),
        key("T / D", "Telegram setup / disconnect"),
        key("P", "Set up provisioning keys"),
        key("B", "Refresh billing"),
        key("O", "Open OmegaMC dashboard"),
        Line::from(""),

        section("Settings"),
        key("↑ / ↓", "Browse sections"),
        key("Enter / Tab", "Focus detail panel → edit fields"),
        key("Enter (on field)", "Activate (install/edit/toggle)"),
        key("x", "Clear selected text field (press twice)"),
        Line::from(""),

        section("Projects → Projects group"),
        key("↑ / ↓", "Browse projects"),
        key("Enter", "Focus detail; Enter again → open in terminal"),
        key("d", "Dispatch oracle to selected project"),
        key("p", "Run planner for selected project"),
        key("n", "Register an existing folder as a project"),
        key("T", "Toggle the project's Telegram topic"),
        key("x", "Delete… (1 OmegaOS · 2 + local folder · 3 + GitHub)"),
        key("D", "Quick delete local (press twice)"),
        Line::from(""),

        section("Projects → System group"),
        key("↑ / ↓", "Navigate sub-sections (or AISB agents in detail)"),
        key("[  /  ]", "Previous / next sub-section"),
        Line::from(""),

        section("Chat (Sessions, when chat-focused)"),
        key("Tab", "Return to session list"),
        key("Shift+Tab", "Forward to Claude (cycle modes)"),
        key("Esc", "Sent to the agent (escape vim/less/prompts) — Tab goes back"),
        key("Ctrl+X", "Close (kill) the focused session"),
        key("Ctrl+R", "Reload the TUI (re-exec the binary)"),
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

    // Publish + pin the scroll bound (same contract as draw_settings/draw_info)
    // so wheel-overscroll stops at the last line instead of a blank panel.
    app.detail_max_scroll =
        (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help — OmegaOS  (↑/↓ scroll) ")
                .border_style(Style::default().fg(th::accent())),
        );

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    // fix6-T8: expire an async sticky notice render-side — the keypress TTL
    // only runs on input, so without this an idle operator's status bar held
    // the stale notice (masking the git segment) indefinitely.
    app.expire_sticky_status();
    // Input mode: show a prompt line (no stats)
    if !matches!(app.input_mode, InputMode::Normal) {
        let (prompt, value) = match &app.input_mode {
            InputMode::Normal => unreachable!(),
            InputMode::NewNamedSession(agent) => (
                "Session name",
                format!("[{}] {}", agent, app.input_buffer),
            ),
            InputMode::NewSessionPromptDirect(name, agent) => (
                "Initial prompt (optional, Esc to skip)",
                format!("[{}/{}] {}", name, agent, app.input_buffer),
            ),
            InputMode::DispatchProject(projects, sel) => (
                "Dispatch — pick project (↑/↓, Enter)",
                projects.get(*sel).cloned().unwrap_or_default(),
            ),
            InputMode::DispatchMission(p) => (
                "Dispatch — mission",
                format!("[{}] {}", p, app.input_buffer),
            ),
            InputMode::RenameSession(old) => (
                "Rename session",
                format!("[{} →] {}", old, app.input_buffer),
            ),
            InputMode::SessionFilter => ("Filter sessions", app.input_buffer.clone()),
            InputMode::NewProjectName => ("New project — name", app.input_buffer.clone()),
            InputMode::NewProjectCategory(name, _) => (
                "New project — category (overlay ↑/↓)",
                format!("[{}]", name),
            ),
            InputMode::NewProjectStack(name, category, _) => (
                "New project — stack (overlay ↑/↓)",
                format!("[{}/{}]", category, name),
            ),
            InputMode::NewProjectCredGroup(..) => {
                ("New project — credential group", app.input_buffer.clone())
            }
            InputMode::NewProjectLaunchPrompt(..) => {
                ("New project — kickoff (optional)", app.input_buffer.clone())
            }
            InputMode::NewProjectLaunchDocs(..) => {
                ("New project — docs (optional)", app.input_buffer.clone())
            }
            InputMode::SelectModel(config_key, ..) => {
                // The same overlay also drives the theme picker — don't tell
                // the user they're selecting a "model" there.
                if config_key == "general.theme" {
                    ("Select theme — ↑/↓ live preview, Enter saves, Esc reverts", String::new())
                } else {
                    ("Select model — ↑/↓, Enter, Esc", String::new())
                }
            }
            InputMode::ProjectDelete(..) => {
                ("Delete project — ↑/↓ or 1/2/3, Enter, Esc", String::new())
            }
            InputMode::ProjectOpenAgent(..) => {
                ("Open project — pick agent — ↑/↓ or 1/2, Enter, Esc", String::new())
            }
            InputMode::ProvisioningSetup { step, .. } => {
                let f = crate::app::PROVISIONING_FIELDS.get(*step);
                let key = f.map(|x| x.0).unwrap_or("");
                let masked = f.map(|x| x.2).unwrap_or(true);
                let echo = if masked {
                    mask_inline(&app.input_buffer)
                } else {
                    app.input_buffer.clone()
                };
                ("Provisioning keys", format!("[{}] {}", key, echo))
            }
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
            InputMode::GroupSetupId => ("Telegram group id", app.input_buffer.clone()),
            InputMode::AddProjectPath => ("Add project — folder path", app.input_buffer.clone()),
            InputMode::ReauthCode => ("Claude re-login — authorize code", app.input_buffer.clone()),
        };

        let status = Paragraph::new(Line::from(vec![
            Span::styled(" ▶ ", Style::default().fg(th::sel_fg()).bg(th::accent2())),
            Span::styled(
                format!(" {}: ", prompt),
                Style::default().fg(th::accent2()).add_modifier(Modifier::BOLD),
            ),
            Span::raw(value),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]));
        frame.render_widget(status, area);
        return;
    }

    // Normal mode: status bar with live system stats (UX inspired by tmux-claude,
    // re-implemented against the rmux SDK). Memoized: SystemStats::read forks
    // a `df` subprocess — at 15-60 FPS that's a fork+exec storm per TUI.
    thread_local! {
        static STATS_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, omega_core::sysinfo::SystemStats)>,
        > = const { std::cell::RefCell::new(None) };
        static USAGE_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, Option<omega_core::monitor::UsageSnapshot>)>,
        > = const { std::cell::RefCell::new(None) };
    }
    let stats = render_memo(&STATS_MEMO, omega_core::sysinfo::SystemStats::read);

    let cpu = format!("CPU {:.2}", stats.cpu_load);
    let ram = format!("RAM {}%", stats.ram_pct);
    let disk = format!("DSK {}%", stats.disk_used_pct);
    let n_sessions = format!("{} sess", app.sessions.len());
    // 5h token-budget snapshot (written by the `omega usage --check` cron) —
    // the real "before the hard stop" signal. None until the first check.
    let usage = render_memo(&USAGE_MEMO, || {
        omega_core::monitor::UsageSnapshot::read().ok().flatten()
    });

    // Localized to `config.timezone` (IANA) so the headless-VPS UTC clock shows
    // the operator's wall time; falls back to $TZ, then system local. See
    // omega_core::clock.
    let time_str = omega_core::clock::hm(app.config.timezone.as_deref());

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
        .constraints([Constraint::Min(0), Constraint::Length(70)])
        .split(area);

    // Transport / staleness banners — the two silent killers of "scroll ne
    // marche pas" / "le fix ne marche pas" reports. (1) Under mosh the mouse
    // handshake never reaches the terminal (mobile-shell/mosh#101): say so on
    // the bar instead of letting the operator debug rmux for the third time.
    // (2) After a deploy the running TUI keeps executing the DELETED binary —
    // fixes silently absent until a reload; /proc/self/exe gains a
    // " (deleted)" suffix the moment the file is replaced.
    thread_local! {
        static ENV_WARN_MEMO: std::cell::RefCell<Option<(std::time::Instant, Option<&'static str>)>> =
            const { std::cell::RefCell::new(None) };
    }
    let env_warn: Option<&'static str> = render_memo(&ENV_WARN_MEMO, || {
        if std::fs::read_link("/proc/self/exe")
            .map(|p| p.to_string_lossy().ends_with(" (deleted)"))
            .unwrap_or(false)
        {
            return Some("⟳ omega UPDATED — Ctrl+R to reload");
        }
        if under_mosh() {
            return Some("⚠ mosh — mouse OFF (use plain SSH for wheel/select)");
        }
        None
    });

    // Left side: Ω badge (no bg, bold) + selected session + status message
    let left = Paragraph::new(Line::from(vec![
        Span::styled(
            " Ω ",
            Style::default()
                .fg(th::text())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            env_warn.map(|w| format!(" {} ", w)).unwrap_or_default(),
            Style::default().fg(th::warn()).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            session_info,
            Style::default()
                .fg(th::text())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        // FIX-A (fix5): an armed two-press confirm renders STATE-DRIVEN with
        // top priority — the warning comes from `armed_confirm_warning()`
        // (the state itself), not from `status_message`, so it is TTL-immune
        // and overwrite-immune by construction: launcher prompts, Ctrl-T,
        // paste acks and sticky forwarder errors can no longer hide a live
        // armed confirm (R-1/R-2/D-3/D-4). Below that, a live status notice
        // wins (F-7): kill/create/vanish/focus confirmations and forwarder
        // errors were built for this channel, and the event loop clears the
        // message on the next keypress — so on the Sessions tab the bar falls
        // back to the selected session's compact git status (`↑4h • main`)
        // once the notice is consumed. The keyboard hints that used to live
        // here are all under the Help tab.
        {
            let on_sessions = app.tab == crate::app::Tab::Sessions;
            let git_text = if on_sessions {
                app.selected_session()
                    .and_then(|e| app.session_git_status.get(&e.session.name).cloned())
            } else {
                None
            };
            let armed = app.armed_confirm_warning();
            let (text, style) = match (armed, app.status_message.as_deref(), git_text) {
                (Some(warn), _, _) => (
                    warn,
                    Style::default().fg(th::error()).add_modifier(Modifier::BOLD),
                ),
                (None, Some(msg), _) => (msg.to_string(), Style::default().fg(th::dim())),
                (None, None, Some(g)) => (g, Style::default().fg(th::success())),
                (None, None, None) => (String::new(), Style::default()),
            };
            Span::styled(text, style)
        },
    ]));
    frame.render_widget(left, split[0]);

    // Right side: system stats (n_sessions in BOLD white so it pops)
    let stat_color = |pct: u8| -> Color {
        match pct {
            0..=60 => th::success(),
            61..=85 => th::accent2(),
            _ => th::error(),
        }
    };

    // Token-budget meter color escalates toward the 80/90% usage alerts.
    let usage_color = |pct: u32| -> Color {
        match pct {
            0..=69 => th::success(),
            70..=89 => th::accent2(),
            _ => th::error(),
        }
    };
    let mut right_spans: Vec<Span> = Vec::new();
    if let Some(ref u) = usage {
        right_spans.push(Span::styled(
            format!("TKN {}%", u.session_pct),
            Style::default()
                .fg(usage_color(u.session_pct))
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::raw("  "));
    }
    right_spans.push(Span::styled(
        cpu,
        Style::default().fg(stat_color(((stats.cpu_load * 25.0) as u8).min(99))),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(ram, Style::default().fg(stat_color(stats.ram_pct))));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(disk, Style::default().fg(stat_color(stats.disk_used_pct))));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        n_sessions,
        Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        time_str,
        Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
    ));
    right_spans.push(Span::raw(" "));
    let right = Paragraph::new(Line::from(right_spans))
        .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right, split[1]);
}
