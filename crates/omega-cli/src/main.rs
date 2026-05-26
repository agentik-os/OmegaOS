use anyhow::Result;
use clap::{Parser, Subcommand};
use omega_core::config::OmegaConfig;
use omega_core::done::{DoneSignal, DoneStatus};
use omega_core::session::SessionManager;

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
        /// Command to run (default: shell)
        #[arg(short, long)]
        cmd: Option<String>,
        /// Files owned by this session (scope-claim)
        #[arg(long, value_delimiter = ',')]
        files: Option<Vec<String>>,
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
        Some(Commands::New { name, dir, cmd, files }) => {
            cmd_new(&name, dir.as_deref(), cmd.as_deref(), files).await
        }
        Some(Commands::List) => cmd_list().await,
        Some(Commands::Attach { name }) => cmd_attach(&name).await,
        Some(Commands::Kill { name }) => cmd_kill(&name).await,
        Some(Commands::Dispatch { project, mission }) => cmd_dispatch(&project, &mission).await,
        Some(Commands::SpawnWorker { task, prompt, dir, files }) => {
            cmd_spawn_worker(&task, &prompt, dir.as_deref(), files).await
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
                    app.status_message = Some("Refreshed".to_string());
                }
                Action::None => {}
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

async fn cmd_new(name: &str, dir: Option<&str>, cmd: Option<&str>, files: Option<Vec<String>>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;

    if let Some(ref files) = files {
        omega_core::scope::claim_or_reject(&config.state_dir, name, files.clone())?;
    }

    let mgr = SessionManager::connect().await?;
    let _session = mgr.create_session(name, dir, cmd).await?;
    println!("Created session: {}", name);
    if let Some(ref files) = files {
        println!("  Scope claimed: {}", files.join(", "));
    }
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
    files: Option<Vec<String>>,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    let work_dir = dir.unwrap_or(".");
    let worker_name = format!("worker-{}", task);

    if let Some(ref files) = files {
        omega_core::scope::claim_or_reject(&config.state_dir, &worker_name, files.clone())?;
    }

    mgr.create_agent_session(&worker_name, work_dir, &config.agent_command, Some(prompt))
        .await?;
    println!("● Worker spawned: {}", worker_name);
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
