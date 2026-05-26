use anyhow::Result;
use clap::{Parser, Subcommand};
use omega_core::config::OmegaConfig;
use omega_core::done::{DoneSignal, DoneStatus};
use omega_core::session::SessionManager;

#[derive(Parser)]
#[command(name = "omega", about = "OmegaOS — Agentic Terminal Operating System")]
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
        /// Command to run (default: shell)
        #[arg(short, long)]
        cmd: Option<String>,
    },

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

    /// Spawn a worker under the current oracle
    SpawnWorker {
        /// Task name (used in session name)
        task: String,
        /// Task prompt/description
        prompt: String,
        /// Working directory
        #[arg(short, long)]
        dir: Option<String>,
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
        Some(Commands::New { name, dir, cmd }) => cmd_new(&name, dir.as_deref(), cmd.as_deref()).await,
        Some(Commands::List) => cmd_list().await,
        Some(Commands::Attach { name }) => cmd_attach(&name).await,
        Some(Commands::Kill { name }) => cmd_kill(&name).await,
        Some(Commands::Dispatch { project, mission }) => cmd_dispatch(&project, &mission).await,
        Some(Commands::SpawnWorker { task, prompt, dir }) => {
            cmd_spawn_worker(&task, &prompt, dir.as_deref()).await
        }
        Some(Commands::Done {
            session,
            status,
            summary,
            commit,
        }) => cmd_done(&session, &status, &summary, commit.as_deref()).await,
        Some(Commands::Status { name }) => cmd_status(&name).await,
        Some(Commands::Send { name, text }) => cmd_send(&name, &text).await,
        Some(Commands::Capture { name }) => cmd_capture(&name).await,
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
            match handle_event(app, evt) {
                Action::Quit => break,
                Action::AttachSession(name) => {
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
                Action::KillSession(name) => {
                    let mgr = SessionManager::connect().await?;
                    match mgr.kill_session(&name).await {
                        Ok(()) => {
                            app.status_message = Some(format!("Killed {}", name));
                            let _ = app.refresh().await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Kill failed: {}", e));
                        }
                    }
                }
                Action::Refresh => {
                    let _ = app.refresh().await;
                    app.status_message = Some("Refreshed".to_string());
                }
                Action::None => {}
            }
        }

        if last_refresh.elapsed() > std::time::Duration::from_secs(5) {
            let _ = app.refresh().await;
            last_refresh = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

async fn cmd_new(name: &str, dir: Option<&str>, cmd: Option<&str>) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let _session = mgr.create_session(name, dir, cmd).await?;
    println!("Created session: {}", name);
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

        let progress = omega_core::progress::ProgressInfo::read(
            &config.state_dir,
            &session.name,
        );
        let progress_str = match progress {
            Some(p) => format!(" {} {:.0}%", p.bar(8), p.percentage()),
            None => String::new(),
        };

        println!("  {} {}{}", icon, session.name, progress_str);
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
    let mgr = SessionManager::connect().await?;
    mgr.kill_session(name).await?;
    println!("Killed session: {}", name);
    Ok(())
}

async fn cmd_dispatch(project: &str, mission: &str) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let mgr = SessionManager::connect().await?;
    let dispatcher = omega_core::dispatch::Dispatcher::new(mgr, config);

    let oracle_name = dispatcher.dispatch_oracle(project, mission).await?;
    println!("Dispatched oracle: {}", oracle_name);
    println!("Mission: {}", mission);
    Ok(())
}

async fn cmd_spawn_worker(task: &str, prompt: &str, dir: Option<&str>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let mgr = SessionManager::connect().await?;

    let work_dir = dir.unwrap_or(".");
    let worker_name = format!("worker-{}", task);

    mgr.create_agent_session(&worker_name, work_dir, &config.agent_command, Some(prompt))
        .await?;
    println!("Spawned worker: {}", worker_name);
    Ok(())
}

async fn cmd_done(
    session: &str,
    status: &str,
    summary: &str,
    commit: Option<&str>,
) -> Result<()> {
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

    println!("Done signal written for: {}", session);
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

async fn cmd_init() -> Result<()> {
    let config = OmegaConfig::default();
    config.ensure_dirs()?;

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
