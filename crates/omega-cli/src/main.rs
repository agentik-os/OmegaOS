use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use omega_core::config::OmegaConfig;
use omega_core::done::{DoneSignal, DoneStatus};
use omega_core::session::SessionManager;

mod telegram_bridge;

#[derive(Parser)]
#[command(
    name = "omega",
    about = "OmegaOS — Agentic Terminal Operating System",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the TUI session manager
    Menu,

    /// Create a new rmux session
    New {
        /// Session name
        name: String,
        /// Working directory
        #[arg(short, long)]
        dir: Option<String>,
        /// Command to run (default: shell). Overrides --agent.
        #[arg(short, long)]
        cmd: Option<String>,
        /// Agent to launch: claude, codex, gemini, pi, glm, shell
        #[arg(short, long)]
        agent: Option<String>,
        /// Initial prompt for the agent
        #[arg(short, long)]
        prompt: Option<String>,
        /// Files owned by this session (scope-claim)
        #[arg(long, value_delimiter = ',')]
        files: Option<Vec<String>>,
    },

    /// List supported agents and their availability
    Agents,

    /// Auto-discover projects on this machine (walks $HOME)
    Projects,

    /// Attach to the Master AISB session (auto-spawns if missing)
    #[command(alias = "aisb")]
    Master,

    /// Get or set provider configuration values (propagates to all sessions)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show billing / accounts / bot status (one-shot, also visible in TUI Monitor tab)
    Monitor,

    /// Manage the Omega Telegram bot bridge (setup/run/enable/disable)
    Telegram {
        #[command(subcommand)]
        action: TelegramAction,
    },

    /// Install Option+Z / Option+/ rmux keybindings (apply now, no daemon restart)
    InstallBindings,

    /// List all sessions
    #[command(alias = "ls")]
    List,

    /// Attach to a session
    Attach {
        /// Session name
        name: String,
    },

    /// Kill a session
    Kill {
        /// Session name
        name: String,
    },

    /// Dispatch a mission to an oracle
    Dispatch {
        /// Project name
        project: String,
        /// Mission description
        mission: String,
    },

    /// Run a full orchestrated mission end-to-end (classify → plan → dispatch → monitor → gate)
    Orchestrate {
        /// Project name
        project: String,
        /// Mission description
        mission: String,
        /// Working directory (default: current dir)
        #[arg(short, long)]
        dir: Option<String>,
        /// Max wait time for workers (seconds)
        #[arg(long, default_value = "3600")]
        timeout: u64,
        /// Skip the quality gate
        #[arg(long)]
        no_gate: bool,
    },

    /// Spawn a worker under the current oracle
    SpawnWorker {
        /// Task name (used in session name)
        task: String,
        /// Task prompt/description
        prompt: String,
        /// Working directory
        #[arg(short, long)]
        dir: Option<String>,
        /// Project name (worker will be <Project>-worker-<task>)
        #[arg(short, long)]
        project: Option<String>,
        /// Files owned by this worker (scope-claim)
        #[arg(long, value_delimiter = ',')]
        files: Option<Vec<String>>,
    },

    /// Spawn a team of agents in split panes
    Team {
        /// Project name
        project: String,
        /// Number of team members
        #[arg(short, long, default_value = "3")]
        count: usize,
        /// Working directory
        #[arg(short, long)]
        dir: Option<String>,
        /// Team member specs (name:prompt, ...)
        members: Vec<String>,
    },

    /// Signal task completion (called by workers)
    Done {
        /// Session name
        session: String,
        /// Status: done_clean, pending, failed
        status: String,
        /// Summary of work done
        summary: String,
        /// Git commit hash (optional)
        #[arg(short, long)]
        commit: Option<String>,
    },

    /// Run patrol daemon (session health watchdog)
    Patrol {
        /// Poll interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
        /// Run once and exit (no daemon loop)
        #[arg(long)]
        once: bool,
    },

    /// Check quality gate for an oracle
    Gate {
        /// Oracle session name
        oracle: String,
        /// Mission description for rubric
        #[arg(short, long)]
        mission: Option<String>,
    },

    /// Check scope-claim conflicts
    Scope {
        /// Session name to check
        session: String,
        /// Files to check
        files: Vec<String>,
    },

    /// Show session status and pane content
    Status {
        /// Session name
        name: String,
    },

    /// Send text to a session
    Send {
        /// Session name
        name: String,
        /// Text to send
        text: String,
    },

    /// Capture pane content from a session
    Capture {
        /// Session name
        name: String,
    },

    /// Show session log (JSONL history)
    Log {
        /// Session name
        session: String,
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },

    /// Run in RPC mode (JSONL stdin/stdout for external orchestration)
    Rpc,

    /// Classify a mission's complexity (SIMPLE/MEDIUM/COMPLEX/EPIC)
    Route {
        /// Mission text to classify
        mission: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell type: bash, zsh, fish, elvish, powershell
        shell: String,
    },

    /// Initialize OmegaOS configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("omega=info".parse()?),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Menu) | None => run_menu().await,
        Some(Commands::New { name, dir, cmd, agent, prompt, files }) => {
            cmd_new(&name, dir.as_deref(), cmd.as_deref(), agent.as_deref(), prompt.as_deref(), files).await
        }
        Some(Commands::Agents) => cmd_agents(),
        Some(Commands::Projects) => cmd_projects(),
        Some(Commands::Master) => cmd_master().await,
        Some(Commands::Config { action }) => cmd_config(action),
        Some(Commands::Monitor) => cmd_monitor(),
        Some(Commands::Telegram { action }) => cmd_telegram(action).await,
        Some(Commands::InstallBindings) => cmd_install_bindings().await,
        Some(Commands::List) => cmd_list().await,
        Some(Commands::Attach { name }) => cmd_attach(&name).await,
        Some(Commands::Kill { name }) => cmd_kill(&name).await,
        Some(Commands::Dispatch { project, mission }) => cmd_dispatch(&project, &mission).await,
        Some(Commands::Orchestrate { project, mission, dir, timeout, no_gate }) => {
            cmd_orchestrate(&project, &mission, dir.as_deref(), timeout, no_gate).await
        }
        Some(Commands::SpawnWorker { task, prompt, dir, project, files }) => {
            cmd_spawn_worker(&task, &prompt, dir.as_deref(), project.as_deref(), files).await
        }
        Some(Commands::Team { project, count, dir, members }) => {
            cmd_team(&project, count, dir.as_deref(), &members).await
        }
        Some(Commands::Done { session, status, summary, commit }) => {
            cmd_done(&session, &status, &summary, commit.as_deref()).await
        }
        Some(Commands::Patrol { interval, once }) => cmd_patrol(interval, once).await,
        Some(Commands::Gate { oracle, mission }) => cmd_gate(&oracle, mission.as_deref()).await,
        Some(Commands::Scope { session, files }) => cmd_scope(&session, &files).await,
        Some(Commands::Status { name }) => cmd_status(&name).await,
        Some(Commands::Send { name, text }) => cmd_send(&name, &text).await,
        Some(Commands::Capture { name }) => cmd_capture(&name).await,
        Some(Commands::Log { session, count }) => cmd_log(&session, count).await,
        Some(Commands::Rpc) => omega_core::rpc::run_rpc_loop().await,
        Some(Commands::Route { mission }) => cmd_route(&mission),
        Some(Commands::Completions { shell }) => cmd_completions(&shell),
        Some(Commands::Init) => cmd_init().await,
    }
}

async fn run_menu() -> Result<()> {
    use omega_tui::app::App;

    let config = OmegaConfig::load().unwrap_or_default();
    let mut app = App::new(config);

    if let Err(e) = app.refresh().await {
        eprintln!("Warning: could not refresh sessions: {}", e);
    }

    // Auto-spawn Master AISB on first launch (if enabled and not already present)
    let cfg = OmegaConfig::load().unwrap_or_default();
    if cfg.auto_spawn_master {
        if let Ok(mgr) = SessionManager::connect().await {
            if let Some(agent) = omega_core::agents::Agent::from_name(&cfg.aisb_agent) {
                let cwd = std::env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(String::from))
                    .unwrap_or_else(|| "/home".to_string());
                match omega_core::aisb::ensure_master(&mgr, agent, &cwd).await {
                    Ok(true) => app.status_message = Some(
                        "Master AISB session spawned automatically — ready to delegate".to_string()
                    ),
                    Ok(false) => app.status_message = Some(
                        "Master AISB already running".to_string()
                    ),
                    Err(e) => eprintln!("Warning: Master AISB auto-spawn failed: {}", e),
                }
                let _ = app.refresh().await;
            }
        }
    }

    let _ = app.refresh_preview().await;

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_tui_loop(&mut terminal, &mut app).await;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

/// After creating a session, switch to the Sessions tab, select the new
/// session, enter chat focus, and refresh the live preview — so the user
/// is immediately ready to talk to it.
async fn auto_focus_chat(app: &mut omega_tui::app::App, session_name: &str) {
    use omega_tui::app::Tab;

    // Sessions can take a moment to register in rmux — retry refresh a few times
    for _ in 0..10 {
        let _ = app.refresh().await;
        if app.select_by_name(session_name) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }
    app.tab = Tab::Sessions;
    app.enter_chat_focus();
    let _ = app.refresh_preview().await;
}

async fn run_tui_loop(
    terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
    app: &mut omega_tui::app::App,
) -> Result<()> {
    use omega_tui::input::{handle_event, Action};
    use omega_tui::ui::draw;

    let tick_rate = std::time::Duration::from_millis(250);
    let mut last_refresh = std::time::Instant::now();

    loop {
        terminal.draw(|f| draw(f, app))?;

        if crossterm::event::poll(tick_rate)? {
            let evt = crossterm::event::read()?;
            let selected_before = app.selected;
            let tab_before = app.tab;
            match handle_event(app, evt) {
                Action::Quit => break,
                Action::AttachSession(name) => {
                    let inside_rmux = std::env::var("RMUX").is_ok();

                    if inside_rmux {
                        // We're already inside rmux — use switch-client to swap to target session
                        // without nesting. Doesn't need terminal handover.
                        let status = std::process::Command::new("rmux")
                            .args(["switch-client", "-t", &name])
                            .status();
                        match status {
                            Ok(s) if s.success() => {
                                app.should_quit = true;
                                break;
                            }
                            Ok(s) => {
                                app.status_message =
                                    Some(format!("switch-client failed (exit {})", s.code().unwrap_or(-1)));
                            }
                            Err(e) => {
                                app.status_message = Some(format!("switch-client error: {}", e));
                            }
                        }
                    } else {
                        // Standalone mode — full terminal handover
                        crossterm::terminal::disable_raw_mode()?;
                        crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::terminal::LeaveAlternateScreen
                        )?;

                        let status = std::process::Command::new("rmux")
                            .args(["attach-session", "-t", &name])
                            .status();

                        crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::terminal::EnterAlternateScreen
                        )?;
                        crossterm::terminal::enable_raw_mode()?;
                        terminal.clear()?;
                        let _ = app.refresh().await;
                        if let Err(e) = status {
                            app.status_message = Some(format!("Attach failed: {}", e));
                        }
                    }
                }
                Action::KillSession(name) => {
                    let mgr = SessionManager::connect().await?;
                    let cfg = OmegaConfig::load().unwrap_or_default();
                    match mgr.kill_session(&name).await {
                        Ok(()) => {
                            let _ = omega_core::scope::ScopeClaim::release(&cfg.state_dir, &name);
                            app.status_message = Some(format!("Killed {}", name));
                            let _ = app.refresh().await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Kill failed: {}", e));
                        }
                    }
                }
                Action::CreateSession(name) => {
                    let mgr = SessionManager::connect().await?;
                    match mgr.create_session(&name, None, None).await {
                        Ok(_) => {
                            app.status_message = Some(format!("Created {}", name));
                            let _ = app.refresh().await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Create failed: {}", e));
                        }
                    }
                }
                Action::CreateSessionWithAgent { name, agent, prompt } => {
                    let mgr = SessionManager::connect().await?;
                    match mgr
                        .create_session_with_agent(&name, None, agent, prompt.as_deref())
                        .await
                    {
                        Ok(_) => {
                            app.status_message =
                                Some(format!("Created {} with {} — opening chat…", name, agent.name()));
                            auto_focus_chat(app, &name).await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Create failed: {}", e));
                        }
                    }
                }
                Action::CreateSessionAutoName { agent, prompt } => {
                    let mgr = SessionManager::connect().await?;
                    match omega_core::naming::auto_name(agent, &mgr).await {
                        Ok(name) => {
                            match mgr
                                .create_session_with_agent(&name, None, agent, prompt.as_deref())
                                .await
                            {
                                Ok(_) => {
                                    app.status_message =
                                        Some(format!("Created {} ({}) — opening chat…", name, agent.name()));
                                    auto_focus_chat(app, &name).await;
                                }
                                Err(e) => {
                                    app.status_message = Some(format!("Create failed: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Auto-name failed: {}", e));
                        }
                    }
                }
                Action::DispatchOracle(project, mission) => {
                    let cfg = OmegaConfig::load().unwrap_or_default();
                    let mgr = SessionManager::connect().await?;
                    let dispatcher = omega_core::dispatch::Dispatcher::new(mgr, cfg.clone());
                    match dispatcher.dispatch_oracle(&project, &mission).await {
                        Ok(oracle_name) => {
                            let sessions_dir = cfg.state_dir.join("sessions");
                            if let Ok(mut log) =
                                omega_core::session_log::SessionLog::create(&sessions_dir, &oracle_name, ".")
                            {
                                let _ = log.append_message(
                                    "system",
                                    &format!("Mission dispatched: {}", mission),
                                );
                            }
                            app.status_message =
                                Some(format!("◆ Dispatched: {}", oracle_name));
                            let _ = app.refresh().await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Dispatch failed: {}", e));
                        }
                    }
                }
                Action::Refresh => {
                    let _ = app.refresh().await;
                    let _ = app.refresh_preview().await;
                    app.status_message = Some("Refreshed".to_string());
                }
                Action::LoginClaude => {
                    // Spawn a fresh rmux session that runs `claude /login`
                    let mgr = SessionManager::connect().await?;
                    let name = match omega_core::naming::auto_name(omega_core::agents::Agent::Claude, &mgr).await {
                        Ok(n) => format!("login-{}", &n[n.len().saturating_sub(2)..]),
                        Err(_) => "claude-login".to_string(),
                    };
                    let cmd = "bash -c 'claude /login; exec bash'";
                    if let Err(e) = mgr.create_session(&name, None, Some(cmd)).await {
                        app.status_message = Some(format!("Login spawn failed: {}", e));
                    } else {
                        app.status_message = Some(format!("Login session '{}' opened — switch to it to enter code", name));
                        let _ = app.refresh().await;
                    }
                }
                Action::RefreshBilling => {
                    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
                    let script = home.join(".aisb/lib/usage-monitor.sh");
                    if script.exists() {
                        let _ = std::process::Command::new("bash")
                            .arg(&script)
                            .stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn();
                        app.status_message = Some("Billing refresh kicked off in background".to_string());
                    } else {
                        app.status_message = Some(
                            "Script not found: ~/.aisb/lib/usage-monitor.sh (AISB must be installed)".to_string(),
                        );
                    }
                }
                Action::TelegramSetup => {
                    app.status_message = Some(
                        "From shell: omega telegram setup <BOT_TOKEN> <CHAT_ID> [--user-id 12345]".to_string(),
                    );
                }
                Action::RenameSession { old, new } => {
                    let mgr = SessionManager::connect().await?;
                    match mgr.rename_session(&old, &new).await {
                        Ok(()) => {
                            app.status_message = Some(format!("Renamed {} → {}", old, new));
                            let _ = app.refresh().await;
                            let _ = app.select_by_name(&new);
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Rename failed: {}", e));
                        }
                    }
                }
                Action::TelegramDisconnect => {
                    match omega_core::monitor::OmegaTelegramConfig::disconnect() {
                        Ok(true) => {
                            app.status_message = Some("✓ Telegram bot disconnected".to_string());
                        }
                        Ok(false) => {
                            app.status_message =
                                Some("Nothing to disconnect — no Telegram config present".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Disconnect failed: {}", e));
                        }
                    }
                }
                Action::SendToSession { session, text } => {
                    let mgr = SessionManager::connect().await?;
                    if !text.is_empty() {
                        if let Err(e) = mgr.send_text(&session, &text).await {
                            app.status_message = Some(format!("Send failed: {}", e));
                        } else {
                            app.status_message = Some(format!("Sent to {}", session));
                            // Auto-scroll to bottom so the new line is visible
                            app.scroll_preview_end();
                            let _ = app.refresh_preview().await;
                        }
                    }
                }
                Action::None => {}
            }

            // Immediate preview refresh when user changes selection or tab
            if app.selected != selected_before || app.tab != tab_before {
                let _ = app.refresh_preview().await;
            }
        }

        if last_refresh.elapsed() > std::time::Duration::from_secs(2) {
            let _ = app.refresh().await;
            let _ = app.refresh_preview().await;
            last_refresh = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

async fn cmd_new(
    name: &str,
    dir: Option<&str>,
    cmd: Option<&str>,
    agent: Option<&str>,
    prompt: Option<&str>,
    files: Option<Vec<String>>,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;

    if let Some(ref files) = files {
        omega_core::scope::claim_or_reject(&config.state_dir, name, files.clone())?;
    }

    let mgr = SessionManager::connect().await?;

    // Priority: explicit --cmd overrides --agent
    if let Some(explicit_cmd) = cmd {
        let _session = mgr.create_session(name, dir, Some(explicit_cmd)).await?;
    } else if let Some(agent_name) = agent {
        let agent_enum = omega_core::agents::Agent::from_name(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown agent: {}. Run `omega agents` to list options.", agent_name))?;
        if !agent_enum.is_available() {
            eprintln!("Warning: {} not detected on this system. Session will be created anyway.", agent_enum.display_name());
        }
        let _session = mgr.create_session_with_agent(name, dir, agent_enum, prompt).await?;
        println!("Agent: {}", agent_enum.display_name());
    } else {
        let _session = mgr.create_session(name, dir, None).await?;
    }

    println!("Created session: {}", name);
    if let Some(ref files) = files {
        println!("  Scope claimed: {}", files.join(", "));
    }
    Ok(())
}

async fn cmd_install_bindings() -> Result<()> {
    // Multiple bindings — fallbacks for terminals that don't pass Alt as Meta:
    //   M-z / M-/         classic Alt+Z and Alt+/
    //   Ctrl+Space        easy chord for terminals that swallow Alt
    //   C-b o / C-b z     prefix variants (Ctrl-B then o or z)
    //   C-Space           one more no-prefix fallback
    let popup_cmd = "display-popup -E -w 100% -h 100% \"omega menu\"";

    // Root-table bindings (no prefix required)
    let root_bindings: Vec<(&str, &str)> = vec![
        ("M-z", "Open OmegaOS menu (Option+Z)"),
        ("M-/", "Open OmegaOS menu (Option+/)"),
        ("C-Space", "Open OmegaOS menu (Ctrl+Space — most reliable)"),
    ];

    // Prefix-table bindings (Ctrl-B then key)
    let prefix_bindings: Vec<(&str, &str)> = vec![
        ("o", "Open OmegaOS menu (prefix + o)"),
        ("z", "Open OmegaOS menu (prefix + z)"),
    ];

    let mut installed = 0usize;
    let mut failed = Vec::new();

    for (key, desc) in &root_bindings {
        let result = std::process::Command::new("rmux")
            .args(["bind-key", "-n", key])
            .arg(popup_cmd)
            .output();
        match result {
            Ok(o) if o.status.success() => {
                println!("✓ {} → {}", key, desc);
                installed += 1;
            }
            Ok(o) => failed.push(format!("{}: {}", key, String::from_utf8_lossy(&o.stderr).trim())),
            Err(e) => failed.push(format!("{}: {}", key, e)),
        }
    }

    for (key, desc) in &prefix_bindings {
        let result = std::process::Command::new("rmux")
            .args(["bind-key", key])
            .arg(popup_cmd)
            .output();
        match result {
            Ok(o) if o.status.success() => {
                println!("✓ C-b {} → {}", key, desc);
                installed += 1;
            }
            Ok(o) => failed.push(format!("C-b {}: {}", key, String::from_utf8_lossy(&o.stderr).trim())),
            Err(e) => failed.push(format!("C-b {}: {}", key, e)),
        }
    }

    println!("\n{} binding(s) installed live", installed);
    if !failed.is_empty() {
        eprintln!("Failed:");
        for f in &failed {
            eprintln!("  - {}", f);
        }
    }

    // Always write the persistent config (overwrite to keep in sync with this binary)
    let omega_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".omega");
    std::fs::create_dir_all(&omega_dir)?;
    let conf_path = omega_dir.join("rmux.conf.omega");
    let content = r#"# OmegaOS rmux bindings — multiple fallbacks for opening the session menu.
#
# Source from your ~/.rmux.conf with:
#   source-file ~/.omega/rmux.conf.omega
#
# Root-table (no prefix):
bind-key -n M-z display-popup -E -w 100% -h 100% "omega menu"
bind-key -n M-/ display-popup -E -w 100% -h 100% "omega menu"
bind-key -n C-Space display-popup -E -w 100% -h 100% "omega menu"

# Prefix-table (C-b first, then key):
bind-key o display-popup -E -w 100% -h 100% "omega menu"
bind-key z display-popup -E -w 100% -h 100% "omega menu"
"#;
    std::fs::write(&conf_path, content)?;
    println!("✓ Persistent config written to {}", conf_path.display());

    // Also patch the user's ~/.rmux.conf to source this file if not already done.
    if let Some(home) = dirs::home_dir() {
        let rmux_conf = home.join(".rmux.conf");
        let source_line = format!("source-file {}", conf_path.display());
        let existing = std::fs::read_to_string(&rmux_conf).unwrap_or_default();
        if !existing.contains("rmux.conf.omega") {
            let mut content = existing;
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str("\n# OmegaOS bindings\n");
            content.push_str(&source_line);
            content.push('\n');
            std::fs::write(&rmux_conf, content)?;
            println!("✓ Added source-file to {}", rmux_conf.display());
        } else {
            println!("✓ ~/.rmux.conf already sources OmegaOS bindings");
        }
    }

    println!();
    println!("Try one of these to open the menu:");
    println!("  • Option+Z  (or Alt+Z)");
    println!("  • Option+/  (or Alt+/)");
    println!("  • Ctrl+Space");
    println!("  • Ctrl+B then o  (or z)");

    Ok(())
}

fn cmd_projects() -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home"));
    let projects = omega_core::projects::discover(&home);

    if projects.is_empty() {
        println!("No projects discovered under {}", home.display());
        println!("Tip: OmegaOS looks for directories named Projects, Code, Dev, Work, Repos, etc.");
        println!("containing at least 2 git repos or files like package.json / Cargo.toml.");
        return Ok(());
    }

    println!("Discovered {} project(s):\n", projects.len());

    let mut current = String::new();
    for p in &projects {
        if p.container != current {
            println!("─── {} ───", p.container);
            current = p.container.clone();
        }
        let stack = if p.stack.is_empty() {
            String::new()
        } else {
            format!("  [{}]", p.stack.join(", "))
        };
        println!("  {} {}{}", p.name, p.path.display(), stack);
    }
    Ok(())
}

#[derive(clap::Subcommand)]
enum TelegramAction {
    /// Save bot token + chat id (+ optional sender allow-list) to ~/.omega/telegram.toml
    Setup {
        bot_token: String,
        chat_id: i64,
        /// Optional Telegram sender user_ids allowed to talk to the bot.
        /// When set, every message MUST come from one of these users; others
        /// are silently dropped. Recommended for shared chats.
        #[arg(long, value_delimiter = ',')]
        user_id: Vec<i64>,
        /// Which rmux session the bot relays messages to (default: aisb-master)
        #[arg(long, default_value = "aisb-master")]
        relay_session: String,
        /// Human-readable label for this profile (shown in the Monitor tab).
        #[arg(long, default_value = "")]
        label: String,
    },
    /// Show current telegram config
    Status,
    /// Enable the configured bot
    Enable,
    /// Disable the configured bot
    Disable,
    /// Remove the Telegram config (~/.omega/telegram.toml) — bot can be re-configured afterwards
    Disconnect,
    /// Run the bot in foreground (polls Telegram, relays messages to AISB Master)
    Run,
}

async fn cmd_telegram(action: TelegramAction) -> Result<()> {
    use omega_core::monitor::OmegaTelegramConfig;
    match action {
        TelegramAction::Setup { bot_token, chat_id, user_id, relay_session, label } => {
            let cfg = OmegaTelegramConfig {
                bot_token,
                chat_id,
                allow_user_ids: user_id,
                relay_session,
                label,
                enabled: true,
            };
            cfg.write()?;
            println!("✓ Telegram config saved to ~/.omega/telegram.toml");
            if !cfg.label.is_empty() {
                println!("  Label:         {}", cfg.label);
            }
            println!("  Relay session: {}", cfg.relay_session);
            println!("  Chat ID:       {}", cfg.chat_id);
            if cfg.allow_user_ids.is_empty() {
                println!("  Sender filter: only chat_id={} accepted", cfg.chat_id);
                println!("  ⚠ For shared chats, restrict further with --user-id");
            } else {
                println!("  Sender filter: only user_ids {:?} accepted", cfg.allow_user_ids);
            }
            println!("\nRun the bot with:  omega telegram run");
            Ok(())
        }
        TelegramAction::Status => {
            match OmegaTelegramConfig::read() {
                Some(cfg) => {
                    println!("Configured: yes");
                    if !cfg.label.is_empty() {
                        println!("  Label:         {}", cfg.label);
                    }
                    println!("  Enabled:       {}", cfg.enabled);
                    println!("  Chat ID:       {}", cfg.chat_id);
                    println!("  Relay session: {}", cfg.relay_session);
                    if cfg.allow_user_ids.is_empty() {
                        println!("  Sender filter: chat_id only (any sender in chat)");
                    } else {
                        println!("  Sender filter: user_ids {:?}", cfg.allow_user_ids);
                    }
                }
                None => {
                    println!("Not configured.");
                    println!("Run: omega telegram setup <BOT_TOKEN> <CHAT_ID> [--user-id 1,2,3]");
                }
            }
            Ok(())
        }
        TelegramAction::Disconnect => {
            match OmegaTelegramConfig::disconnect()? {
                true => println!("✓ Telegram bot disconnected (~/.omega/telegram.toml removed)"),
                false => println!("(nothing to disconnect — no config present)"),
            }
            Ok(())
        }
        TelegramAction::Enable => {
            if let Some(mut cfg) = OmegaTelegramConfig::read() {
                cfg.enabled = true;
                cfg.write()?;
                println!("✓ Telegram bot enabled");
            } else {
                anyhow::bail!("Not configured. Run: omega telegram setup …");
            }
            Ok(())
        }
        TelegramAction::Disable => {
            if let Some(mut cfg) = OmegaTelegramConfig::read() {
                cfg.enabled = false;
                cfg.write()?;
                println!("✓ Telegram bot disabled");
            } else {
                anyhow::bail!("Not configured.");
            }
            Ok(())
        }
        TelegramAction::Run => {
            let cfg = OmegaTelegramConfig::read()
                .ok_or_else(|| anyhow::anyhow!("Not configured. Run: omega telegram setup …"))?;
            if !cfg.enabled {
                anyhow::bail!("Bot is disabled. Run: omega telegram enable");
            }
            telegram_bridge::run(cfg).await
        }
    }
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// Get a config value: <provider>.<key>  e.g. claude.model
    Get { key: String },
    /// Set a config value: <provider>.<key> <value>  e.g. claude.model opus
    Set { key: String, value: String },
    /// Show all provider configs
    Show,
}

fn cmd_config(action: ConfigAction) -> Result<()> {
    use omega_core::providers::ProvidersConfig;
    let mut cfg = ProvidersConfig::load();

    match action {
        ConfigAction::Show => {
            let toml = toml::to_string_pretty(&cfg)?;
            println!("{}", toml);
        }
        ConfigAction::Get { key } => {
            let value = get_config_value(&cfg, &key)?;
            println!("{}", value);
        }
        ConfigAction::Set { key, value } => {
            set_config_value(&mut cfg, &key, &value)?;
            cfg.save()?;
            println!("✓ Set {} = {}", key, value);
            println!("Applies to all newly spawned sessions.");
        }
    }
    Ok(())
}

fn get_config_value(cfg: &omega_core::providers::ProvidersConfig, key: &str) -> Result<String> {
    let mut parts = key.splitn(2, '.');
    let provider = parts.next().context("missing provider")?;
    let field = parts.next().context("missing field (use provider.field)")?;
    let s = match (provider, field) {
        ("claude", "model") => cfg.claude.model.clone(),
        ("claude", "effort") => cfg.claude.effort.clone(),
        ("claude", "api_key") => cfg.claude.api_key.clone(),
        ("claude", "dangerously_skip_permissions") => cfg.claude.dangerously_skip_permissions.to_string(),
        ("codex", "model") => cfg.codex.model.clone(),
        ("codex", "api_key") => cfg.codex.api_key.clone(),
        ("codex", "base_url") => cfg.codex.base_url.clone(),
        ("gemini", "model") => cfg.gemini.model.clone(),
        ("gemini", "api_key") => cfg.gemini.api_key.clone(),
        ("pi", "provider") => cfg.pi.provider.clone(),
        ("pi", "model") => cfg.pi.model.clone(),
        ("pi", "extension") => cfg.pi.extension.clone(),
        ("glm", "model") => cfg.glm.model.clone(),
        ("glm", "api_key") => cfg.glm.api_key.clone(),
        _ => anyhow::bail!("Unknown key: {}", key),
    };
    Ok(s)
}

fn set_config_value(
    cfg: &mut omega_core::providers::ProvidersConfig,
    key: &str,
    value: &str,
) -> Result<()> {
    let mut parts = key.splitn(2, '.');
    let provider = parts.next().context("missing provider")?;
    let field = parts.next().context("missing field (use provider.field)")?;
    match (provider, field) {
        ("claude", "model") => cfg.claude.model = value.to_string(),
        ("claude", "effort") => cfg.claude.effort = value.to_string(),
        ("claude", "api_key") => cfg.claude.api_key = value.to_string(),
        ("claude", "dangerously_skip_permissions") => {
            cfg.claude.dangerously_skip_permissions = value.parse().unwrap_or(false);
        }
        ("codex", "model") => cfg.codex.model = value.to_string(),
        ("codex", "api_key") => cfg.codex.api_key = value.to_string(),
        ("codex", "base_url") => cfg.codex.base_url = value.to_string(),
        ("gemini", "model") => cfg.gemini.model = value.to_string(),
        ("gemini", "api_key") => cfg.gemini.api_key = value.to_string(),
        ("pi", "provider") => cfg.pi.provider = value.to_string(),
        ("pi", "model") => cfg.pi.model = value.to_string(),
        ("pi", "extension") => cfg.pi.extension = value.to_string(),
        ("glm", "model") => cfg.glm.model = value.to_string(),
        ("glm", "api_key") => cfg.glm.api_key = value.to_string(),
        _ => anyhow::bail!("Unknown key: {}", key),
    }
    Ok(())
}

fn cmd_monitor() -> Result<()> {
    use omega_core::monitor;
    let snap = monitor::UsageSnapshot::read()?.unwrap_or_default();
    let accounts = monitor::list_accounts();
    let bot = monitor::aisb_bot_status();
    let tg = monitor::OmegaTelegramConfig::read();

    println!("─── Billing ───");
    println!("  5h session:  {:.1}%  ({}/{})", snap.precise_5h(),
        snap.tokens_5h, snap.budget_5h);
    println!("  Week:        {:.1}%  ({}/{})", snap.precise_week(),
        snap.tokens_7d, snap.budget_week);
    println!("  Account:     {} ({})", snap.active_account, snap.email);
    println!();
    println!("─── AISB Bot ───");
    println!("  Running:     {}", bot.bot_alive);
    println!("  Cache:       {:?}", bot.cache_status);
    println!();
    println!("─── Accounts ({}) ───", accounts.len());
    for acc in &accounts {
        let marker = if acc.is_active { "▶" } else { " " };
        println!("  {} {}  {}", marker, acc.label, acc.email.as_deref().unwrap_or(""));
    }
    println!();
    println!("─── Omega Telegram ───");
    match tg {
        Some(c) => println!("  Configured: yes (enabled={}, relay={})", c.enabled, c.relay_session),
        None => println!("  Not configured. Run: omega telegram setup <TOKEN> <CHAT_ID>"),
    }
    Ok(())
}

async fn cmd_master() -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    let agent = omega_core::agents::Agent::from_name(&config.aisb_agent)
        .unwrap_or(omega_core::agents::Agent::Claude);
    let cwd = std::env::current_dir()?
        .to_str()
        .unwrap_or("/home")
        .to_string();

    let created = omega_core::aisb::ensure_master(&mgr, agent, &cwd).await?;
    if created {
        println!("★ Master AISB spawned");
    } else {
        println!("★ Master AISB already running — attaching");
    }

    // Attach (use switch-client if inside rmux, else attach-session)
    let inside_rmux = std::env::var("RMUX").is_ok();
    let arg = if inside_rmux {
        "switch-client"
    } else {
        "attach-session"
    };
    let status = std::process::Command::new("rmux")
        .args([arg, "-t", omega_core::aisb::MASTER_SESSION_NAME])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to attach to Master AISB");
    }
    Ok(())
}

fn cmd_agents() -> Result<()> {
    println!("Available agents:\n");
    for agent in omega_core::agents::Agent::all() {
        let status = if agent.is_available() { "✓" } else { "✗" };
        let color = if agent.is_available() { "\x1b[32m" } else { "\x1b[31m" };
        println!(
            "  {}{}\x1b[0m  {:8}  {}",
            color,
            status,
            agent.name(),
            agent.display_name()
        );
    }
    println!("\nUsage:");
    println!("  omega new my-session --agent claude");
    println!("  omega new researcher --agent pi --prompt \"Investigate X\"");
    println!("  omega new dev --agent codex --dir ~/my-project");
    Ok(())
}

async fn cmd_list() -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    let config = OmegaConfig::load().unwrap_or_default();

    if sessions.is_empty() {
        println!("No active sessions");
        return Ok(());
    }

    let mut current_project: Option<String> = None;

    for session in &sessions {
        if session.project != current_project {
            if current_project.is_some() {
                println!();
            }
            match &session.project {
                Some(p) => println!("─── {} ───", p),
                None => {
                    let label = match session.role {
                        omega_core::session::SessionRole::Home => "Home",
                        omega_core::session::SessionRole::System => "System",
                        _ => "Other",
                    };
                    println!("─── {} ───", label);
                }
            }
            current_project = session.project.clone();
        }

        let icon = match session.role {
            omega_core::session::SessionRole::Oracle => "◆",
            omega_core::session::SessionRole::Worker => "●",
            omega_core::session::SessionRole::Home => "⌂",
            omega_core::session::SessionRole::System => "⚙",
        };

        let progress = omega_core::progress::ProgressInfo::read(&config.state_dir, &session.name);
        let progress_str = match progress {
            Some(p) => format!(" {} {:.0}%", p.bar(8), p.percentage()),
            None => String::new(),
        };

        let scope = omega_core::scope::ScopeClaim::read(&config.state_dir, &session.name);
        let scope_str = match scope {
            Some(s) => format!(" [{}]", s.files_owned.join(", ")),
            None => String::new(),
        };

        println!("  {} {}{}{}", icon, session.name, progress_str, scope_str);
    }
    Ok(())
}

async fn cmd_attach(name: &str) -> Result<()> {
    let status = std::process::Command::new("rmux")
        .args(["attach-session", "-t", name])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to attach to session {}", name);
    }
    Ok(())
}

async fn cmd_kill(name: &str) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let mgr = SessionManager::connect().await?;
    mgr.kill_session(name).await?;
    let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, name);
    println!("Killed session: {}", name);
    Ok(())
}

async fn cmd_orchestrate(
    project: &str,
    mission: &str,
    dir: Option<&str>,
    timeout_secs: u64,
    no_gate: bool,
) -> Result<()> {
    use omega_core::mission::Mission;
    use omega_core::orchestration::{Orchestrator, OrchestratorOptions};
    use std::path::PathBuf;
    use std::time::Duration;

    let config = OmegaConfig::load().unwrap_or_default();
    let opts = OrchestratorOptions {
        worker_timeout: Duration::from_secs(timeout_secs),
        poll_interval: Duration::from_secs(5),
        enforce_gate: !no_gate,
        auto_ack: true,
    };

    let orchestrator = Orchestrator::new(config, opts).await?;
    let working_dir = match dir {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let mission_obj = Mission::new(project, mission, working_dir);
    println!("◆ Mission {} dispatched", mission_obj.id.0);
    println!("  Project: {}", mission_obj.project);
    println!("  Text:    {}", mission_obj.text);
    println!();

    let outcome = orchestrator.execute(mission_obj).await?;

    println!("─── Outcome ───");
    println!("{}", outcome.summary);
    println!();

    match outcome.status {
        omega_core::mission::OutcomeStatus::Success => {
            println!("✓ Mission completed successfully");
        }
        omega_core::mission::OutcomeStatus::PartialSuccess => {
            println!("⚠ Mission partially completed");
        }
        omega_core::mission::OutcomeStatus::Failed => {
            println!("✗ Mission failed");
            std::process::exit(1);
        }
        omega_core::mission::OutcomeStatus::Aborted => {
            println!("⊘ Mission aborted");
            std::process::exit(2);
        }
    }

    Ok(())
}

async fn cmd_dispatch(project: &str, mission: &str) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;
    let dispatcher = omega_core::dispatch::Dispatcher::new(mgr, config.clone());

    let oracle_name = dispatcher.dispatch_oracle(project, mission).await?;

    // Create session log
    let sessions_dir = config.state_dir.join("sessions");
    let mut log = omega_core::session_log::SessionLog::create(&sessions_dir, &oracle_name, ".")?;
    log.append_message("system", &format!("Mission dispatched: {}", mission))?;

    println!("◆ Oracle dispatched: {}", oracle_name);
    println!("  Mission: {}", mission);
    Ok(())
}

async fn cmd_spawn_worker(
    task: &str,
    prompt: &str,
    dir: Option<&str>,
    project: Option<&str>,
    files: Option<Vec<String>>,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    // Auto-detect project from current rmux session name if we're inside an oracle
    let project_name = match project {
        Some(p) => Some(p.to_string()),
        None => std::env::var("RMUX")
            .ok()
            .and_then(|v| v.split(',').next().map(|s| s.to_string()))
            .and_then(|sess| sess.strip_prefix("oracle-").map(|p| {
                p.trim_end_matches(char::is_numeric)
                    .trim_end_matches('-')
                    .to_string()
            })),
    };

    let work_dir = dir.unwrap_or(".");
    let worker_name = match &project_name {
        Some(p) => format!("{}-worker-{}", p, task),
        None => format!("worker-{}", task),
    };

    if let Some(ref files) = files {
        omega_core::scope::claim_or_reject(&config.state_dir, &worker_name, files.clone())?;
    }

    mgr.create_agent_session(&worker_name, work_dir, &config.agent_command, Some(prompt))
        .await?;
    println!("● Worker spawned: {}", worker_name);
    if let Some(p) = &project_name {
        println!("  Under project: {}", p);
    }
    if let Some(ref files) = files {
        println!("  Scope claimed: {}", files.join(", "));
    }
    Ok(())
}

async fn cmd_team(
    project: &str,
    _count: usize,
    dir: Option<&str>,
    member_specs: &[String],
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    let work_dir = dir.unwrap_or(".").to_string();
    let session_name = format!("Team-{}", project);

    let members: Vec<omega_core::team::TeamMember> = member_specs
        .iter()
        .map(|spec| {
            let parts: Vec<&str> = spec.splitn(2, ':').collect();
            let name = parts[0].to_string();
            let prompt = parts.get(1).unwrap_or(&"Implement your assigned task").to_string();
            omega_core::team::TeamMember {
                name,
                role: "worker".to_string(),
                prompt,
                files_owned: Vec::new(),
            }
        })
        .collect();

    if members.is_empty() {
        anyhow::bail!("No team members specified. Use: omega team Project member1:prompt member2:prompt");
    }

    let team_config = omega_core::team::TeamConfig {
        project: project.to_string(),
        session_name: session_name.clone(),
        working_dir: work_dir,
        agent_command: config.agent_command.clone(),
        members: members.clone(),
    };

    let spawner = omega_core::team::TeamSpawner::new(&mgr);
    let _panes = spawner.spawn_team(&team_config).await?;

    println!("◆ Team spawned: {}", session_name);
    for (i, member) in members.iter().enumerate() {
        println!("  ● [{}] {}", i, member.name);
    }
    Ok(())
}

async fn cmd_done(session: &str, status: &str, summary: &str, commit: Option<&str>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;

    let done_status = match status {
        "done_clean" => DoneStatus::DoneClean,
        "pending" => DoneStatus::Pending,
        "failed" => DoneStatus::Failed,
        _ => anyhow::bail!("Invalid status: {}. Use: done_clean, pending, failed", status),
    };

    let mut signal = DoneSignal::new(session, done_status, summary);
    signal.commit = commit.map(|s| s.to_string());
    signal.write(&config.state_dir)?;

    // Release scope claim on done_clean
    if signal.is_complete() {
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, session);
    }

    println!("✓ Done signal written for: {}", session);
    Ok(())
}

async fn cmd_patrol(interval: u64, once: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let patrol = omega_core::patrol::Patrol::new(config);

    if once {
        let report = patrol.run_once().await?;
        println!("Sessions: {} (◆{} ●{})", report.total_sessions, report.oracles, report.workers);
        if !report.done_workers.is_empty() {
            println!("Done workers: {}", report.done_workers.join(", "));
        }
        if !report.orphaned_sessions.is_empty() {
            println!("Orphaned: {}", report.orphaned_sessions.join(", "));
        }
        for action in &report.actions_taken {
            println!("  → {}", action);
        }
    } else {
        println!("Patrol daemon started (interval: {}s)", interval);
        patrol
            .run_loop(std::time::Duration::from_secs(interval))
            .await?;
    }
    Ok(())
}

async fn cmd_gate(oracle: &str, mission: Option<&str>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();

    if let Some(mission_text) = mission {
        let rubric = omega_core::gate::Rubric::new(
            mission_text,
            vec![
                omega_core::gate::RubricCriterion {
                    id: "F1".to_string(),
                    description: "Core feature implemented".to_string(),
                    weight: 3.0,
                    category: omega_core::gate::CriterionCategory::Functional,
                },
                omega_core::gate::RubricCriterion {
                    id: "Q1".to_string(),
                    description: "Build passes with zero errors".to_string(),
                    weight: 2.0,
                    category: omega_core::gate::CriterionCategory::Quality,
                },
                omega_core::gate::RubricCriterion {
                    id: "Q2".to_string(),
                    description: "No console errors in runtime".to_string(),
                    weight: 1.0,
                    category: omega_core::gate::CriterionCategory::Quality,
                },
            ],
        );
        rubric.write(&config.state_dir, oracle)?;
        println!("Rubric created for {}: {} criteria", oracle, rubric.criteria.len());
        return Ok(());
    }

    match omega_core::gate::Rubric::read(&config.state_dir, oracle)? {
        Some(rubric) => {
            println!("Mission: {}", rubric.mission);
            println!("Criteria:");
            for c in &rubric.criteria {
                println!("  [{}] {} (weight: {:.1})", c.id, c.description, c.weight);
            }
        }
        None => {
            println!("No rubric found for {}. Create one with: omega gate {} --mission \"...\"", oracle, oracle);
        }
    }
    Ok(())
}

async fn cmd_scope(session: &str, files: &[String]) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let conflicts = omega_core::scope::check_conflicts(&config.state_dir, session, files)?;

    if conflicts.is_empty() {
        println!("✓ No scope conflicts for {}", session);
    } else {
        println!("✗ Scope conflicts detected:");
        for conflict in &conflicts {
            println!(
                "  {} owns: {}",
                conflict.blocking_session,
                conflict.overlapping_files.join(", ")
            );
        }
    }
    Ok(())
}

async fn cmd_status(name: &str) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let content = mgr.capture_pane(name).await?;
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(30);
    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

async fn cmd_send(name: &str, text: &str) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    mgr.send_text(name, text).await?;
    println!("Sent to {}: {}", name, text);
    Ok(())
}

async fn cmd_capture(name: &str) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let content = mgr.capture_pane(name).await?;
    print!("{}", content);
    Ok(())
}

async fn cmd_log(session: &str, count: usize) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let sessions_dir = config.state_dir.join("sessions");

    match omega_core::session_log::SessionLog::find_latest(&sessions_dir, session) {
        Some(path) => {
            let entries = omega_core::session_log::SessionLog::read_entries(&path)?;
            let start = entries.len().saturating_sub(count);
            for entry in &entries[start..] {
                match entry {
                    omega_core::session_log::SessionEntry::Header(h) => {
                        println!("[{}] SESSION {} cwd={}", h.timestamp.format("%H:%M:%S"), h.session_name, h.cwd);
                    }
                    omega_core::session_log::SessionEntry::Message(m) => {
                        let preview: String = m.content.chars().take(80).collect();
                        println!("[{}] {} {}", m.timestamp.format("%H:%M:%S"), m.role, preview);
                    }
                    omega_core::session_log::SessionEntry::ToolCall(t) => {
                        println!("[{}] TOOL {}", t.timestamp.format("%H:%M:%S"), t.tool_name);
                    }
                    omega_core::session_log::SessionEntry::Done(d) => {
                        println!("[{}] DONE {} — {}", d.timestamp.format("%H:%M:%S"), d.status, d.summary);
                    }
                    omega_core::session_log::SessionEntry::Event(e) => {
                        println!("[{}] EVENT {}", e.timestamp.format("%H:%M:%S"), e.event_type);
                    }
                    omega_core::session_log::SessionEntry::Compaction(c) => {
                        println!("[{}] COMPACT {} entries", c.timestamp.format("%H:%M:%S"), c.entries_compacted);
                    }
                }
            }
        }
        None => {
            println!("No session log found for {}", session);
        }
    }
    Ok(())
}

fn cmd_route(mission: &str) -> Result<()> {
    let decision = omega_core::routing::classify_mission(mission);
    println!("Mission: {}", mission);
    println!();
    println!("Complexity:        {}", decision.complexity.label());
    println!("Suggested agent:   {}", decision.suggested_agent);
    println!("Recommended team:  {} agent(s)", decision.complexity.recommended_agents());
    println!("Estimated time:    ~{} min", decision.complexity.estimated_minutes());
    println!("Decompose:         {}", decision.decompose);
    println!("Use team:          {}", decision.use_team);
    println!("Use quality gate:  {}", decision.use_quality_gate);
    println!();
    println!("Reasoning:");
    for r in &decision.reasoning {
        println!("  • {}", r);
    }
    Ok(())
}

fn cmd_completions(shell: &str) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell};

    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "elvish" => Shell::Elvish,
        "powershell" | "ps" => Shell::PowerShell,
        _ => anyhow::bail!("Unknown shell: {}. Use: bash, zsh, fish, elvish, powershell", shell),
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "omega", &mut std::io::stdout());
    Ok(())
}

async fn cmd_init() -> Result<()> {
    let config = OmegaConfig::default();
    config.ensure_dirs()?;
    std::fs::create_dir_all(config.state_dir.join("sessions"))?;

    let config_path = OmegaConfig::config_path();
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, content)?;
        println!("Created config at: {}", config_path.display());
    } else {
        println!("Config already exists at: {}", config_path.display());
    }

    println!("State directory: {}", config.state_dir.display());
    println!("Logs directory: {}", config.logs_dir.display());
    println!("\nOmegaOS initialized. Run 'omega' to launch the session manager.");
    Ok(())
}
