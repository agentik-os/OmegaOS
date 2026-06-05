use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use omega_core::config::OmegaConfig;
use omega_core::done::{DoneSignal, DoneStatus};
use omega_core::session::SessionManager;

mod forwarder;
mod usage;

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

    /// Print the getting-started guide (the post-install onboarding steps:
    /// Claude login, Telegram remote, service keys, first project). The npx
    /// installer's animation hides install.sh output, so this is the durable
    /// way to (re)read it.
    Guide,

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

    /// Bootstrap a brand-new project (provision + scaffold + vision/PRD/plan) via the
    /// workflow-driven /omega-new-project pipeline. Spawns a Claude session in the
    /// resolved category dir. Use --dry-run to print the plan without spawning.
    NewProject {
        /// Project name (lowercase [a-z0-9-])
        name: String,
        /// Stack (default: nextstack — the only stack today)
        #[arg(default_value = "nextstack")]
        stack: String,
        /// Category: works | client | 1-life | AgentikOS
        #[arg(default_value = "works")]
        category: String,
        /// Provisioning credential group (client projects)
        #[arg(long, default_value = "default")]
        group: String,
        /// Resume a half-finished bootstrap (continues from the first non-completed phase)
        #[arg(long)]
        resume: bool,
        /// Re-enter from a specific phase
        #[arg(long)]
        from: Option<String>,
        /// Skip phases (csv) — the reason is recorded, never silently dropped
        #[arg(long)]
        skip: Option<String>,
        /// Token budget ceiling for the run
        #[arg(long)]
        budget: Option<u64>,
        /// Opt into the execute (/build) phase
        #[arg(long)]
        build: bool,
        /// Print the resolved DAG and exit — zero mutation, no session spawned
        #[arg(long)]
        dry_run: bool,
    },

    /// List supported agents and their availability
    Agents,

    /// Remove "junk" rmux sessions — ones omega could not have created (their
    /// rmux name isn't its own sanitized slug, e.g. "istryGPT -" from a mangled
    /// multi-line paste). Dry-run by default; pass --force to actually kill them.
    #[command(name = "clean-junk")]
    CleanJunk {
        /// Actually kill the junk sessions (default is a dry-run preview).
        #[arg(long)]
        force: bool,
    },

    /// Print the localized wall-clock for the rmux status bar. Honors the
    /// `timezone` config field (falls back to $TZ, then system local). One
    /// source of truth shared with the TUI clock — see omega_core::clock.
    Clock {
        /// Append the date → "HH:MM DD-Mon-YY"
        #[arg(long)]
        full: bool,
    },

    /// Auto-discover projects on this machine (walks $HOME)
    Projects,

    /// Mark a folder as trusted in ~/.claude.json so Claude Code skips the
    /// "Do you trust the files in this folder?" dialog. Ran automatically by
    /// every agent launch command right before `claude` starts (concurrent
    /// sessions clobber the shared config, so trust must be written fresh).
    #[command(name = "trust-dir", hide = true)]
    TrustDir {
        /// Folder to trust (defaults to the current directory)
        dir: Option<String>,
    },

    /// Run the official installer for an agent (pi, hermes, codex, gemini, glm, claude)
    Install {
        /// Name of the agent to install
        agent: String,
        /// Don't actually run the installer — just print the command
        #[arg(long)]
        dry_run: bool,
    },

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

    /// Generate a PDF report (whitepaper, audit, marketing, doc)
    Pdf {
        /// Template: whitepaper, audit, marketing, doc
        #[arg(long, default_value = "whitepaper")]
        template: String,
        /// Path to data JSON file (omit for demo)
        #[arg(long)]
        data: Option<String>,
        /// Render a demo PDF with sample data
        #[arg(long)]
        demo: bool,
        /// Theme: agentik (the classic whitepaper theme — the only theme)
        #[arg(long, default_value = "agentik")]
        theme: String,
        /// Output PDF path
        #[arg(long, default_value = "/tmp/omega-report.pdf")]
        out: String,
        /// Send the PDF to Telegram after generation
        #[arg(long)]
        send: bool,
        /// Caption for the Telegram message
        #[arg(long)]
        caption: Option<String>,
    },

    /// List, export, or manage operational rules
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    /// Manage Quality Arsenal audits (23 Gestalt-Popper forensic audits)
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },

    /// Sync OmegaOS config into all LLM config directories (symlinks)
    Sync,

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
        /// Bypass the prompt-completeness gate (downgrade reject to a warning)
        #[arg(long)]
        force: bool,
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
        /// Status: done_clean, pending, failed, blocked
        status: String,
        /// Summary of work done
        summary: String,
        /// Git commit hash (optional)
        #[arg(short, long)]
        commit: Option<String>,
    },
    /// Report live mission progress (oracles call this as they finish plan tasks).
    /// Writes ~/.omega/state/oracle-<key>.progress.json; the Telegram bot renders the
    /// task checklist (✓/✗/▸/☐) + bar in the project topic. Preserves the bot's
    /// chat/thread/msg fields. Two ways to drive it:
    ///   omega progress <s> --plan "audit|fix N+1|merge"      (set the plan; all todo)
    ///   omega progress <s> --task "audit" --status done       (mark one task)
    /// status = done | fail | doing | todo.
    Progress {
        /// Session name (e.g. oracle-dentistrygpt-7)
        session: String,
        /// Set the full plan: a pipe-separated task list (each starts as todo).
        #[arg(long)]
        plan: Option<String>,
        /// Upsert one task by title (use with --status).
        #[arg(long)]
        task: Option<String>,
        /// Status for --task: done | fail | doing | todo.
        #[arg(long)]
        status: Option<String>,
    },

    /// Read/drain oracle inbox events (JSONL event queue)
    Inbox {
        /// Oracle name
        oracle: String,
        /// Action: peek, drain, count
        #[arg(default_value = "peek")]
        action: String,
    },

    /// Ship pipeline: build → commit → push → deploy → verify
    Ship {
        /// Project name
        project: String,
        /// Commit message
        #[arg(short, long, default_value = "chore: ship via omega")]
        message: String,
        /// Unfreeze a frozen pipeline
        #[arg(long)]
        unfreeze: bool,
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

    /// Token-budget usage monitor + Telegram 80%/90% alerts.
    Usage {
        /// Run one check tick (cron mode): fetch usage, alert if a
        /// threshold is crossed, then exit.
        #[arg(long)]
        check: bool,
    },

    /// Kill all sessions except the current one + infrastructure (your
    /// Home/System shells, the Telegram bridge, the master).
    KillAll {
        /// Actually kill. Without it, just lists what would be killed.
        #[arg(long)]
        yes: bool,
    },

    /// Nuclear cleanup — kill stray sessions, prune stale state, clear /tmp
    /// scratch, and drop the page cache where permitted. Adapts to the host;
    /// never touches current/infra sessions and never hard-requires root.
    Cleanup {
        /// Actually run. Without it, prints the plan (dry run).
        #[arg(long)]
        yes: bool,
    },

    /// One-shot health check of the whole stack — daemon, socket, doctrine,
    /// agent CLI, Telegram service, hooks, secrets, memory. Run it first after
    /// a fresh install / VPS reset.
    Doctor {
        /// Pre-reset readiness report: what irreproducible state you'd lose if
        /// you wiped this machine right now (secrets present, memory size,
        /// crontab, and which project repos have uncommitted / unpushed work).
        /// Read-only — writes nothing.
        #[arg(long)]
        pre_reset: bool,
        /// Apply safe mechanical fixes for the warnings it can resolve
        /// (duplicate Telegram pollers, dead bot service, stale usage cache,
        /// expired oauth), then re-run the checks. Used by the self-heal cron.
        #[arg(long)]
        fix: bool,
    },

    /// Back up the irreproducible OmegaOS state (`~/.omega` + crontab) to a
    /// single `.tgz` you can `scp` off the machine before a reset. Your project
    /// repos are NOT bundled (they live in your own git) — they are only
    /// reported if they have unpushed work. Run `omega doctor --pre-reset` first.
    Backup {
        /// Output archive path (default: ~/omega-backup-<timestamp>.tgz).
        #[arg(long)]
        out: Option<String>,
        /// Also include the claude-mem memory store (~/.claude/projects — large).
        #[arg(long)]
        include_memory: bool,
    },

    /// Replay an oracle's full dispatch→done history (debug stuck missions).
    Timeline {
        /// Oracle session name (e.g. oracle-Causio-1).
        oracle: String,
    },

    /// Re-spawn a crashed oracle from its persisted OracleState (survives a
    /// daemon restart). No arg = resurrect every dead oracle.
    Resurrect {
        /// Oracle session name; omit to resurrect all dead oracles.
        oracle: Option<String>,
    },

    /// Manage provisioning credential groups (per-client accounts): list, show,
    /// or set tokens. Each client project uses its own group for push/deploy.
    Provision {
        #[command(subcommand)]
        action: ProvisionAction,
    },

    /// Interactive AISB Master chat REPL (runs inside the aisb-master
    /// pane). Each line you type is injected into the running bot exactly
    /// as if it had arrived from Telegram — same brain, same response,
    /// which also lands in your Telegram chat.
    AisbChat,

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

    /// Show plan progress from .planner/tracker.json (read-only)
    PlanStatus {
        /// Project directory containing .planner/tracker.json
        #[arg(default_value = ".")]
        path: String,
    },

    /// Drive a plan to completion via the executor (spawns real workers per step)
    PlanRun {
        /// Project directory containing .planner/tracker.json
        #[arg(default_value = ".")]
        path: String,
    },

    /// Start a Claude OAuth re-login: spawn the reauth session, send /login, and
    /// print the captured authorize URL as JSON (`{"ok":true,"url":"..."}`).
    /// Headless — the shared engine for the TUI and the Telegram bridge. Open the
    /// URL, authorize, then finish with `omega claude-login-code <code>`.
    #[command(name = "claude-login")]
    ClaudeLogin,

    /// Finish a Claude OAuth re-login: paste the authorize code into the waiting
    /// reauth session, wait for the credentials to refresh, and print the result
    /// as JSON (`{"ok":bool,"email":...,"expires_min":...}`).
    #[command(name = "claude-login-code")]
    ClaudeLoginCode {
        /// The OAuth code from the browser (may include a `#state` suffix).
        code: String,
    },
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

    // SSOT auto-heal: Claude's atomic write on /login replaces the native
    // ~/.claude/.credentials.json symlink with a real file, diverging from the
    // canonical ~/.omega/credentials/claude.json. Repair the symlink on every
    // startup so reads stay funneled through the canonical store. Non-fatal.
    match omega_core::credentials::CredentialStore::new() {
        Ok(store) => {
            if let Err(e) = store.ensure_legacy_symlink("claude") {
                tracing::warn!(error = %e, "could not heal claude credential symlink");
            } else {
                tracing::debug!("claude credential symlink checked/healed");
            }
        }
        Err(e) => tracing::warn!(error = %e, "could not open credential store for symlink heal"),
    }

    match cli.command {
        Some(Commands::Menu) | None => run_menu().await,
        Some(Commands::New { name, dir, cmd, agent, prompt, files }) => {
            cmd_new(&name, dir.as_deref(), cmd.as_deref(), agent.as_deref(), prompt.as_deref(), files).await
        }
        Some(Commands::NewProject { name, stack, category, group, resume, from, skip, budget, build, dry_run }) => {
            cmd_new_project(&name, &stack, &category, &group, resume, from.as_deref(), skip.as_deref(), budget, build, dry_run).await
        }
        Some(Commands::Agents) => cmd_agents(),
        Some(Commands::CleanJunk { force }) => cmd_clean_junk(force).await,
        Some(Commands::Clock { full }) => cmd_clock(full),
        Some(Commands::Projects) => cmd_projects(),
        Some(Commands::TrustDir { dir }) => cmd_trust_dir(dir.as_deref()),
        Some(Commands::Install { agent, dry_run }) => cmd_install(&agent, dry_run),
        Some(Commands::Master) => cmd_master().await,
        Some(Commands::Config { action }) => cmd_config(action),
        Some(Commands::Monitor) => cmd_monitor(),
        Some(Commands::Telegram { action }) => cmd_telegram(action).await,
        Some(Commands::Pdf { template, data, demo, theme, out, send, caption }) => {
            cmd_pdf(&template, data.as_deref(), demo, &theme, &out, send, caption.as_deref()).await
        }
        Some(Commands::Rules { action }) => cmd_rules(action),
        Some(Commands::Audit { action }) => cmd_audit(action),
        Some(Commands::Sync) => cmd_sync(),
        Some(Commands::InstallBindings) => cmd_install_bindings().await,
        Some(Commands::List) => cmd_list().await,
        Some(Commands::Attach { name }) => cmd_attach(&name).await,
        Some(Commands::Kill { name }) => cmd_kill(&name).await,
        Some(Commands::Dispatch { project, mission }) => cmd_dispatch(&project, &mission).await,
        Some(Commands::Orchestrate { project, mission, dir, timeout, no_gate }) => {
            cmd_orchestrate(&project, &mission, dir.as_deref(), timeout, no_gate).await
        }
        Some(Commands::SpawnWorker { task, prompt, dir, project, files, force }) => {
            cmd_spawn_worker(&task, &prompt, dir.as_deref(), project.as_deref(), files, force).await
        }
        Some(Commands::Team { project, count, dir, members }) => {
            cmd_team(&project, count, dir.as_deref(), &members).await
        }
        Some(Commands::Done { session, status, summary, commit }) => {
            cmd_done(&session, &status, &summary, commit.as_deref()).await
        }
        Some(Commands::Progress { session, plan, task, status }) => {
            cmd_progress(&session, plan.as_deref(), task.as_deref(), status.as_deref())
        }
        Some(Commands::Inbox { oracle, action }) => cmd_inbox(&oracle, &action).await,
        Some(Commands::Ship { project, message, unfreeze }) => {
            cmd_ship(&project, &message, unfreeze).await
        }
        Some(Commands::Patrol { interval, once }) => cmd_patrol(interval, once).await,
        Some(Commands::AisbChat) => cmd_aisb_chat().await,
        Some(Commands::KillAll { yes }) => cmd_kill_all(yes).await,
        Some(Commands::Cleanup { yes }) => cmd_cleanup(yes).await,
        Some(Commands::Guide) => {
            // Prefer the installed copy (matches the installed version); fall
            // back to the guide embedded at compile time so `omega guide`
            // always answers, even if ~/.omega was wiped.
            let installed = dirs::home_dir()
                .map(|h| h.join(".omega/GETTING-STARTED.md"))
                .filter(|p| p.exists());
            match installed.and_then(|p| std::fs::read_to_string(p).ok()) {
                Some(text) => print!("{}", text),
                None => print!("{}", include_str!("../../../docs/GETTING-STARTED.md")),
            }
            Ok(())
        }
        Some(Commands::Doctor { pre_reset, fix }) => {
            if pre_reset {
                cmd_doctor_pre_reset()
            } else {
                cmd_doctor(fix).await
            }
        }
        Some(Commands::Backup { out, include_memory }) => cmd_backup(out, include_memory),
        Some(Commands::Timeline { oracle }) => cmd_timeline(&oracle).await,
        Some(Commands::Resurrect { oracle }) => cmd_resurrect(oracle).await,
        Some(Commands::Provision { action }) => cmd_provision(action),
        Some(Commands::Usage { check }) => {
            if check {
                // --check: actively fetch from the OAuth endpoint + alert on threshold.
                match usage::check_and_alert().await {
                    Ok(Some(snap)) => {
                        println!(
                            "usage: 5h={}% week={}% (alert={}%)",
                            snap.session_pct,
                            snap.week_pct,
                            snap.alert_pct()
                        );
                    }
                    Ok(None) => println!("usage: OAuth endpoint unavailable (no alert)"),
                    Err(e) => eprintln!("usage check failed: {}", e),
                }
            } else {
                // no flag: show the last cached snapshot without a network call.
                match omega_core::monitor::UsageSnapshot::read().ok().flatten() {
                    Some(snap) => println!(
                        "usage (cached): 5h={}% week={}%",
                        snap.session_pct, snap.week_pct
                    ),
                    None => {
                        println!("usage: no cached snapshot — run 'omega usage --check'")
                    }
                }
            }
            Ok(())
        }
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
        Some(Commands::PlanStatus { path }) => cmd_plan_status(&path),
        Some(Commands::PlanRun { path }) => cmd_plan_run(&path).await,
        Some(Commands::ClaudeLogin) => cmd_claude_login().await,
        Some(Commands::ClaudeLoginCode { code }) => cmd_claude_login_code(&code).await,
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

    // Raise rmux's scrollback retention so agent conversations keep their full
    // history (default is 2000 lines → the top of long chats was lost). Global
    // = applies to all sessions spawned from here on. Best-effort.
    let _ = tokio::process::Command::new("rmux")
        .args(["set-option", "-g", "history-limit", "500000"])
        .output()
        .await;

    // Force truecolor advertisement. SSH clients (Termius and friends) often
    // strip COLORTERM, leaving the process to see only `TERM=xterm` — which
    // makes crossterm/ratatui downgrade 24-bit RGB to the closest 256-color
    // index, washing out the styled-preview reds/greens/blues we paint in
    // ui.rs. Forcing this here is safe: terminals that genuinely can't render
    // truecolor will downgrade themselves, but every modern client we ship
    // to (Termius, Blink, native macOS Terminal/iTerm2, gnome-terminal,
    // alacritty…) handles 24-bit RGB correctly. Verified empirically:
    // without this, my (241,76,76) bright red appeared as a muddy washed
    // orange; with it, true bright red.
    if std::env::var("COLORTERM").as_deref().unwrap_or("") != "truecolor" {
        std::env::set_var("COLORTERM", "truecolor");
    }

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        // Bracketed paste — long pastes arrive as a single Event::Paste
        // instead of fragmenting into per-character Key events (which would
        // hit Enter on embedded \n and submit prematurely).
        crossterm::event::EnableBracketedPaste,
    )?;
    let backend = ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_tui_loop(&mut terminal, &mut app).await;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste,
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
/// Best-effort: post a confirmation message to the configured chat via
/// the Telegram HTTP API. Errors are swallowed (we still tell the user
/// the bot is configured even if the network call failed).
async fn send_telegram_confirmation(bot_token: &str, chat_id: i64, text: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
    });
    let _ = client.post(&url).json(&body).send().await;
}

fn shell_escape_for_bash(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Toggle a boolean config field in either OmegaConfig or ProvidersConfig.
fn toggle_bool_config(key: &str) -> Result<()> {
    match key {
        "general.auto_spawn_master" => {
            let mut c = OmegaConfig::load().unwrap_or_default();
            c.auto_spawn_master = !c.auto_spawn_master;
            save_omega_config(&c)?;
        }
        "general.auto_naming" => {
            let mut c = OmegaConfig::load().unwrap_or_default();
            c.auto_naming = !c.auto_naming;
            save_omega_config(&c)?;
        }
        "general.session_shortcuts" => {
            let mut c = OmegaConfig::load().unwrap_or_default();
            c.session_shortcuts = !c.session_shortcuts;
            save_omega_config(&c)?;
        }
        "claude.dangerously_skip_permissions" => {
            let mut p = omega_core::providers::ProvidersConfig::load();
            p.claude.dangerously_skip_permissions = !p.claude.dangerously_skip_permissions;
            p.save()?;
        }
        _ => anyhow::bail!("Unknown toggle key: {}", key),
    }
    Ok(())
}

fn save_omega_config(c: &OmegaConfig) -> Result<()> {
    let path = OmegaConfig::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(c)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// keep-set for a TUI-initiated kill-all / nuclear cleanup: infrastructure
/// singletons + the current session + every protected session.
fn tui_cleanup_keep(
    app: &omega_tui::app::App,
    sessions: &[omega_core::session::OmegaSession],
) -> std::collections::HashSet<String> {
    let mut keep = omega_core::cleanup::infrastructure_keep(sessions);
    if let Some(ref cur) = app.current_session {
        keep.insert(cur.clone());
    }
    for e in &app.sessions {
        if e.is_protected {
            keep.insert(e.session.name.clone());
        }
    }
    keep
}

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

    // Frame rate: ADAPTIVE. 60 FPS (16ms) while the user is actively interacting
    // so typing/streaming feels like a live tmux attach; ~15 FPS (66ms) at rest.
    // The loop redraws every tick, and on a small/oversubscribed box several idle
    // TUIs each spinning a full widget rebuild at 60 FPS pegged the CPU (the rmux
    // daemon + 5 idle menus saturated a 2-core VPS). At rest nothing needs 60 FPS:
    // the status-bar clock only ticks once a second and agent output is throttled
    // by the preview cadence below, so 66ms idle is visually identical at a
    // fraction of the CPU. The active window keys off the SAME last_input_at as
    // the preview cadence, so a keystroke instantly restores 60 FPS.
    const TICK_ACTIVE: std::time::Duration = std::time::Duration::from_millis(16);
    const TICK_IDLE: std::time::Duration = std::time::Duration::from_millis(66);

    // Preview capture: ADAPTIVE cadence. The only rmux daemon RPC per loop is
    // the capture, so we throttle it separately from the 60 FPS draw. Idle
    // cadence is 80ms (enough to look "live" without spamming the daemon);
    // for a short window after the user interacts we drop to 30ms so the
    // typed-char echo feels near-instant. Idle load is unchanged.
    const PREVIEW_IDLE_MS: u64 = 80;
    // 16ms = 60 fps echo while the user is actively typing — matches the
    // event-loop tick so each keystroke can echo on the very next frame.
    // 30ms (~33 fps) was perceivably laggy on fast typing/pasting; user
    // explicitly asked for snappier interaction.
    const PREVIEW_ACTIVE_MS: u64 = 16;
    // Keep the fast cadence going for half a second after each keystroke
    // so a short typing pause doesn't immediately drop us back to 80ms idle
    // (avoids a perceptible "stutter" between bursts).
    const PREVIEW_ACTIVE_WINDOW_MS: u64 = 500;
    let mut last_preview_refresh = std::time::Instant::now();
    let mut last_refresh = std::time::Instant::now();
    // Last keystroke/forward activity; starts "stale" so we boot in idle mode.
    let mut last_input_at =
        std::time::Instant::now() - std::time::Duration::from_millis(PREVIEW_ACTIVE_WINDOW_MS);

    // Async status sink — keystroke forwarding happens off the event loop so
    // it doesn't block on the rmux RPC (was 5-15ms per keystroke, perceived as
    // input lag in chat-focus mode). Failures land here and are drained into
    // `app.status_message` at the start of each tick so the UI still surfaces
    // forwarder errors.
    let async_status: std::sync::Arc<std::sync::Mutex<Option<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));

    // OAuth re-login engine sink. request_reauth / handle_code block on internal
    // sleeps (~16s / ~20s); we run them in a detached task so the UI never
    // freezes, and the task writes the resulting ReauthStatus here. Drained into
    // `app.reauth_status` at the top of each tick.
    let reauth_sink: std::sync::Arc<
        std::sync::Mutex<Option<omega_tui::app::ReauthStatus>>,
    > = std::sync::Arc::new(std::sync::Mutex::new(None));

    // Single ordered keystroke forwarder. One consumer task drains a FIFO
    // mpsc channel and is the only task that reaches the SDK transport, so
    // delivery order is guaranteed (per-keystroke tokio::spawn raced on the
    // multi-threaded runtime and could reorder fast typing). The loop only
    // does a synchronous non-blocking `fwd_tx.send`, so input stays instant.
    let fwd_tx = forwarder::spawn_forwarder(
        SessionManager::connect_cached().await?,
        async_status.clone(),
    );

    // Track the last pane resize we issued so we only resize on change
    // (session switch OR terminal resize), not every tick.
    let mut last_resized: Option<(String, u16, u16)> = None;
    // Throttle for the per-session model/token meta scan (transcript parse).
    let mut last_meta_refresh =
        std::time::Instant::now() - std::time::Duration::from_secs(10);
    // Per-session transcript mtime, so we re-scan the (possibly tens-of-MB)
    // JSONL only when it actually changed.
    let mut meta_mtimes: std::collections::HashMap<String, std::time::SystemTime> =
        std::collections::HashMap::new();
    // Throttle for the per-session git status (branch + age of oldest unpushed
    // commit). Shown in the status bar on the Sessions tab as e.g. `↑4h • main`.
    let mut last_git_refresh =
        std::time::Instant::now() - std::time::Duration::from_secs(60);

    loop {
        // Drain any error reported by a backgrounded keystroke forwarder.
        if let Ok(mut guard) = async_status.lock() {
            if let Some(msg) = guard.take() {
                app.status_message = Some(msg);
            }
        }

        // Drain the OAuth re-login engine result (set by a detached task).
        if let Ok(mut guard) = reauth_sink.lock() {
            if let Some(status) = guard.take() {
                app.reauth_status = status;
            }
        }

        terminal.draw(|f| draw(f, app))?; // app is &mut, allows auto-scroll

        // ── Responsive pane sizing ──────────────────────────────────────
        // Make the embedded Claude/agent view track the rmux preview panel's
        // CURRENT geometry. This runs EVERY tick (~16ms), right after the
        // draw that just wrote the live `preview_inner_*` dims — so the inner
        // terminal follows the panel on the very next frame after the user
        // resizes their terminal, enters/exits fullscreen, or switches
        // sessions. The (session, cols, rows) change-detection means an
        // actual resize RPC fires ONLY when the geometry truly changed, so
        // running this every tick costs a couple of cheap cached lookups at
        // rest, not a daemon round-trip. Verified live: rmux pane.resize →
        // SIGWINCH → Claude redraws at the new width (e.g. 200→120).
        if app.tab == omega_tui::app::Tab::Sessions {
            if let Some(entry) = app.selected_session() {
                let name = entry.session.name.clone();
                let cols = app.preview_inner_width;
                // 1:1 with the visible panel — except the pre-existing
                // .max(10) floor below: inner height <10 still gets rows=10
                // (top 10−inner rows hidden on tiny terminals). Hidden extra rows (the
                // old CHAT_INPUT_HEADROOM = 50 on chat focus) made every
                // full-screen agent UI — e.g. the dynamic-workflow live view —
                // lay out its header/status ~50 rows above the visible tail
                // slice: the operator saw an empty box at tail ([49/49]), and
                // scrolling up to the content drops the mirror into frozen
                // plain-text history mode where any keystroke snaps back to
                // tail — watching a live workflow was impossible. Known
                // tradeoff: a TYPED input taller than the panel clips its top
                // inside Claude while composing (content intact; typed lines
                // stay recoverable via Alt+Up history browse — only EDITS to
                // clipped lines act on a stale view; the [Pasted text #N]
                // collapse that makes pastes immune is Claude-Code-specific —
                // other CLI agents may differ).
                let rows = app.preview_inner_height.max(10);
                if cols >= 20 {
                    let want = (name.clone(), cols, rows);
                    if last_resized.as_ref() != Some(&want) {
                        if let Ok(m) = SessionManager::connect_cached().await {
                            let _ = m.resize_pane(&name, cols, rows).await;
                        }
                        last_resized = Some(want);
                    }
                }
            }
        }

        // Refresh the previewed session's model + token meta (shown on the
        // right of the preview title). Throttled to 3s and run via
        // spawn_blocking — it scans the Claude transcript JSONL, which must
        // never touch the UI hot path. Only the selected session with a known
        // working_dir is scanned.
        if app.tab == omega_tui::app::Tab::Sessions
            && last_meta_refresh.elapsed() >= std::time::Duration::from_secs(3)
        {
            let sel = app.selected_session().map(|e| e.session.name.clone());
            if let Some(name) = sel {
                let n = name.clone();
                let prev = meta_mtimes.get(&name).copied();
                let res = tokio::task::spawn_blocking(move || {
                    omega_core::claude_meta::read_meta_for_session_if_changed(&n, prev)
                })
                .await
                .ok()
                .flatten();
                if let Some((m, mtime)) = res {
                    app.session_meta.insert(name.clone(), (m.model, m.tokens));
                    meta_mtimes.insert(name, mtime);
                }
            }
            last_meta_refresh = std::time::Instant::now();
        }

        // Refresh the selected session's git status (branch + ↑Nh) every 10s
        // for the status-bar display on the Sessions tab. Throttled + off the
        // hot path. Two short git invocations — negligible at this cadence.
        if app.tab == omega_tui::app::Tab::Sessions
            && last_git_refresh.elapsed() >= std::time::Duration::from_secs(10)
        {
            let sel = app.selected_session().map(|e| e.session.name.clone());
            if let Some(name) = sel {
                let n = name.clone();
                let status = tokio::task::spawn_blocking(move || {
                    omega_core::git_status::status_for_session(&n)
                })
                .await
                .ok()
                .flatten();
                match status {
                    Some(s) => {
                        app.session_git_status.insert(name, s);
                    }
                    None => {
                        app.session_git_status.remove(&name);
                    }
                }
            }
            last_git_refresh = std::time::Instant::now();
        }

        // Decoupled preview refresh — runs whether or not the user is
        // typing. Hot-path Forward* actions no longer await capture,
        // they just forward + return; this loop tick picks up the
        // visual change on the next 80ms boundary.
        //
        // Burst-typing guard: if there's already a pending input event,
        // skip the capture this tick. Preview catches up on the very
        // next tick (~16ms), and the user keeps a zero-lag echo loop.
        let event_pending = crossterm::event::poll(std::time::Duration::ZERO)?;
        // Faster echo for a short window right after the user interacts, then
        // back to the idle cadence so we don't hammer the daemon at rest.
        let interacting = last_input_at.elapsed()
            < std::time::Duration::from_millis(PREVIEW_ACTIVE_WINDOW_MS);
        let preview_refresh_interval = if interacting {
            std::time::Duration::from_millis(PREVIEW_ACTIVE_MS)
        } else {
            std::time::Duration::from_millis(PREVIEW_IDLE_MS)
        };
        // 60 FPS while interacting, ~15 FPS at rest (cuts idle render CPU ~4×).
        let tick_rate = if interacting { TICK_ACTIVE } else { TICK_IDLE };
        if !event_pending && last_preview_refresh.elapsed() >= preview_refresh_interval {
            let _ = app.refresh_preview().await;
            last_preview_refresh = std::time::Instant::now();
        }

        // Drain *all* events queued for this tick before redrawing. The
        // old shape was `if poll(tick_rate)` → process one → redraw, which
        // capped throughput at 1 event per ~10ms iteration (~100 chars/s
        // ceiling, visible as input lag during fast typing or paste).
        // Now we wait up to tick_rate for the first event, then drain any
        // additional pending events with a ZERO-timeout poll. A wall-clock
        // budget (8ms) bounds the drain so a never-ending stream cannot
        // starve the redraw / preview / housekeeping work below.
        let drain_start = std::time::Instant::now();
        let drain_budget = std::time::Duration::from_millis(8);
        let mut first_iter = true;
        let mut saw_resize = false;
        while {
            let timeout = if first_iter {
                tick_rate
            } else {
                std::time::Duration::ZERO
            };
            crossterm::event::poll(timeout)?
        } {
            first_iter = false;
            let evt = crossterm::event::read()?;
            // A terminal zoom / font-resize emits a Resize event. The pane
            // capture can briefly fail mid-reflow; force a clear + fresh preview
            // AFTER the drain so the view re-resolves immediately instead of
            // waiting up to 2s for the housekeeping refresh (or a manual Ctrl+R).
            if matches!(evt, crossterm::event::Event::Resize(_, _)) {
                saw_resize = true;
            }
            let selected_before = app.selected;
            let tab_before = app.tab;
            let detail_focused_before = app.detail_focused;
            let status_before = app.status_message.clone();
            match handle_event(app, evt) {
                Action::Quit => break,
                Action::ToggleMouseCapture => {
                    // Flip terminal mouse capture live. OFF → the terminal does
                    // native drag-select + copy/paste; ON → clickable menus + scroll.
                    app.mouse_capture = !app.mouse_capture;
                    if app.mouse_capture {
                        crossterm::execute!(terminal.backend_mut(), crossterm::event::EnableMouseCapture).ok();
                        app.status_message = Some("🖱  Mouse ON — click menus & scroll  ·  Ctrl-T for text selection".to_string());
                    } else {
                        crossterm::execute!(terminal.backend_mut(), crossterm::event::DisableMouseCapture).ok();
                        app.status_message = Some("📋 Selection mode — drag to select & copy/paste  ·  Ctrl-T to re-enable clicks".to_string());
                    }
                }
                Action::Restart => {
                    // Tear down the terminal cleanly, then re-exec the
                    // current binary so a freshly-built `omega` is picked up
                    // in place (same PID on Unix via exec).
                    crossterm::terminal::disable_raw_mode().ok();
                    crossterm::execute!(
                        terminal.backend_mut(),
                        crossterm::terminal::LeaveAlternateScreen,
                        crossterm::event::DisableMouseCapture,
                        crossterm::event::DisableBracketedPaste,
                    )
                    .ok();
                    terminal.show_cursor().ok();
                    // Resolve a binary path that actually exists. current_exe()
                    // can point at a now-replaced/deleted inode after a redeploy
                    // (cp over ~/.local/bin/omega), which makes exec() fail with
                    // ENOENT. Prefer the canonical install path, then current_exe,
                    // then a bare PATH lookup.
                    use std::os::unix::process::CommandExt;
                    let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
                    let candidates = [
                        home.join(".local/bin/omega"),
                        std::env::current_exe().unwrap_or_default(),
                    ];
                    let chosen = candidates
                        .iter()
                        .find(|p| p.exists())
                        .cloned()
                        .unwrap_or_else(|| std::path::PathBuf::from("omega"));
                    // exec() replaces the process image; on success it never
                    // returns. If the absolute path failed, fall back to a
                    // PATH-resolved "omega menu" via the shell.
                    let err = std::process::Command::new(&chosen).arg("menu").exec();
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("exec omega menu")
                        .exec();
                    eprintln!("restart failed: {} (binary: {})", err, chosen.display());
                    break;
                }
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
                    let is_master = omega_core::aisb::is_master(&name);
                    match mgr.kill_session(&name).await {
                        Ok(()) => {
                            let _ = omega_core::scope::ScopeClaim::release(&cfg.state_dir, &name);
                            // Master auto-respawns: killing it just re-spawns
                            // a fresh process. The Telegram bridge is unaffected
                            // (its persistent claude_stream subprocess handles
                            // chat independently of the rmux session).
                            if is_master && cfg.auto_spawn_master {
                                if let Some(agent) = omega_core::agents::Agent::from_name(&cfg.aisb_agent) {
                                    let cwd = std::env::current_dir()
                                        .ok()
                                        .and_then(|p| p.to_str().map(String::from))
                                        .unwrap_or_else(|| "/home".to_string());
                                    match omega_core::aisb::ensure_master(&mgr, agent, &cwd).await {
                                        Ok(_) => app.status_message = Some(
                                            format!("Killed {} → auto-respawned", name)
                                        ),
                                        Err(e) => app.status_message = Some(
                                            format!("Killed {} but respawn failed: {}", name, e)
                                        ),
                                    }
                                }
                            } else {
                                app.status_message = Some(format!("Killed {}", name));
                            }
                            let _ = app.refresh().await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Kill failed: {}", e));
                        }
                    }
                }
                Action::KillAllSessions => {
                    let mgr = SessionManager::connect().await?;
                    let sessions = mgr.list_sessions().await.unwrap_or_default();
                    let keep = tui_cleanup_keep(&app, &sessions);
                    match omega_core::cleanup::kill_all(&mgr, &keep).await {
                        Ok(killed) => {
                            app.status_message =
                                Some(format!("Killed {} session(s)", killed.len()));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Kill-all failed: {}", e))
                        }
                    }
                    let _ = app.refresh().await;
                }
                Action::NuclearCleanup => {
                    let mgr = SessionManager::connect().await?;
                    let cfg = OmegaConfig::load().unwrap_or_default();
                    let sessions = mgr.list_sessions().await.unwrap_or_default();
                    let keep = tui_cleanup_keep(&app, &sessions);
                    match omega_core::cleanup::nuclear_cleanup(&mgr, &cfg, &keep).await {
                        Ok(report) => {
                            app.status_message = Some(format!("Nuclear cleanup: {}", report.summary()));
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Nuclear cleanup failed: {}", e))
                        }
                    }
                    let _ = app.refresh().await;
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
                Action::CreateProject { name, category, stack, launch_prompt, launch_docs } => {
                    // Cross-user: resolve the category dir from config (projects_dir),
                    // NEVER a hardcoded ~/VibeCoding. The skill creates <base>/<name>.
                    let cfg = OmegaConfig::load().unwrap_or_default();
                    let base = cfg.resolve_category_path(&category);
                    let _ = std::fs::create_dir_all(&base);
                    let session = format!("{}-setup", name);
                    // Credential group chosen in the wizard (client projects); default = shared.
                    let group = app
                        .new_project_cred_group
                        .take()
                        .unwrap_or_else(|| "default".to_string());

                    // Append an optional kickoff brief + doc contents so the project
                    // session starts from the user's idea / existing docs.
                    let mut prompt =
                        format!("/omega-new-project {} {} {} {}", stack, category, name, group);
                    if let Some(kick) = launch_prompt.as_deref() {
                        if !kick.trim().is_empty() {
                            prompt.push_str("\n\n--- PROJECT KICKOFF BRIEF ---\n");
                            prompt.push_str(kick.trim());
                            prompt.push_str("\n--- END BRIEF ---");
                        }
                    }
                    if let Some(docs) = launch_docs.as_deref() {
                        let attached = read_launch_docs(docs);
                        if !attached.trim().is_empty() {
                            prompt.push_str("\n\n--- REFERENCED DOCS ---");
                            prompt.push_str(&attached);
                            prompt.push_str("\n--- END DOCS ---");
                        }
                    }

                    let mgr = SessionManager::connect().await?;
                    let agent = omega_core::agents::Agent::Claude;
                    match mgr
                        .create_session_with_agent(&session, base.to_str(), agent, Some(&prompt))
                        .await
                    {
                        Ok(_) => {
                            app.status_message = Some(format!(
                                "New project '{}' ({}) — provisioning in {} ...",
                                name, stack, session
                            ));
                            auto_focus_chat(app, &session).await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("New project failed: {}", e));
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
                    if app.tab == omega_tui::app::Tab::Agentic {
                        app.refresh_projects();
                    }
                    app.status_message = Some("Refreshed".to_string());
                }
                Action::LoginClaude => {
                    // Drive the real OAuth engine (oauth::request_reauth) instead
                    // of spawning a raw `claude /login` pane. It blocks on ~16s of
                    // internal sleeps, so run it in a detached task and surface the
                    // result via the reauth_sink (drained into app.reauth_status).
                    use omega_tui::app::ReauthStatus;
                    app.reauth_status = ReauthStatus::Generating;
                    app.status_message =
                        Some("Starting Claude re-login — capturing authorize URL…".to_string());
                    let sink = reauth_sink.clone();
                    tokio::spawn(async move {
                        let result = match SessionManager::connect().await {
                            Ok(mgr) => {
                                match omega_core::oauth::request_reauth(&mgr, "tui", None, true).await {
                                    Ok(Some(req)) => ReauthStatus::ShowUrl(req.auth_url),
                                    Ok(None) => ReauthStatus::Error(
                                        "re-login already pending or on cooldown".to_string(),
                                    ),
                                    Err(e) => ReauthStatus::Error(e.to_string()),
                                }
                            }
                            Err(e) => ReauthStatus::Error(format!("connect failed: {}", e)),
                        };
                        if let Ok(mut g) = sink.lock() {
                            *g = Some(result);
                        }
                    });
                }
                Action::SubmitReauthCode { code } => {
                    // Finish the re-login: paste the code into the waiting reauth
                    // session and watch the credentials refresh. Detached (the
                    // engine blocks ~20s); result lands in reauth_sink.
                    use omega_tui::app::ReauthStatus;
                    app.reauth_status = ReauthStatus::Validating;
                    app.status_message = Some("Submitting code, validating login…".to_string());
                    let sink = reauth_sink.clone();
                    tokio::spawn(async move {
                        let result = match SessionManager::connect().await {
                            Ok(mgr) => match omega_core::oauth::handle_code(&mgr, &code).await {
                                Ok(res) if res.success => ReauthStatus::Done(format!(
                                    "Logged in as {} — expires in {} min",
                                    res.email, res.expires_min
                                )),
                                Ok(res) => ReauthStatus::Error(format!(
                                    "login did not refresh credentials. {}",
                                    res.pane_tail.lines().last().unwrap_or("")
                                )),
                                Err(e) => ReauthStatus::Error(e.to_string()),
                            },
                            Err(e) => ReauthStatus::Error(format!("connect failed: {}", e)),
                        };
                        if let Ok(mut g) = sink.lock() {
                            *g = Some(result);
                        }
                    });
                }
                Action::RefreshBilling => {
                    // Use the native `omega usage --check` which hits the REAL
                    // OAuth utilization endpoint and writes the accurate
                    // ~/.omega/state/usage.json. The legacy bash script wrote a
                    // local-token ESTIMATE that over-reported (89% vs real 36%).
                    let exe = std::env::current_exe()
                        .unwrap_or_else(|_| std::path::PathBuf::from("omega"));
                    let _ = std::process::Command::new(exe)
                        .args(["usage", "--check"])
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    app.status_message =
                        Some("Billing refresh (real OAuth %) running…".to_string());
                }
                Action::TelegramSetup => {
                    app.input_buffer = String::new();
                    app.input_mode = omega_tui::app::InputMode::TelegramSetupToken;
                    app.status_message = Some(
                        "Step 1/3: paste your Telegram BOT_TOKEN (from @BotFather) — Enter to confirm, Esc to cancel"
                            .to_string(),
                    );
                }
                Action::ProvisioningSetup => {
                    app.input_buffer = String::new();
                    let fields = omega_tui::app::PROVISIONING_FIELDS;
                    app.input_mode = omega_tui::app::InputMode::ProvisioningSetup {
                        step: 0,
                        collected: Vec::new(),
                    };
                    app.status_message =
                        Some(format!("Step 1/{}: {}", fields.len(), fields[0].1));
                }
                Action::ProvisioningCommit { values } => {
                    match omega_core::provisioning::update_services_env(&values) {
                        Ok(()) => {
                            let n = values.iter().filter(|(_, v)| !v.trim().is_empty()).count();
                            app.status_message = Some(format!(
                                "[+] Saved {} provisioning key(s) → ~/.omega/provisioning/services.env",
                                n
                            ));
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Provisioning save failed: {}", e));
                        }
                    }
                }
                Action::TelegramSetupCommit { bot_token, chat_id, user_ids } => {
                    let cfg = omega_core::monitor::OmegaTelegramConfig {
                        bot_token: bot_token.clone(),
                        chat_id,
                        allow_user_ids: user_ids.clone(),
                        relay_session: omega_core::aisb::MASTER_SESSION_NAME.to_string(),
                        label: String::new(),
                        enabled: true,
                    };
                    match cfg.write() {
                        Ok(()) => {
                            // 1) Send a confirmation message via Telegram API so
                            //    the user can see the bot works.
                            let confirm = format!(
                                "🟢 <b>Ω OmegaOS</b> — Telegram setup complete\n\
                                 ━━━━━━━━━━\n\n\
                                 <b>Chat:</b> <code>{}</code>\n\
                                 <b>Filter:</b> {}\n\n\
                                 <i>Messages are relayed to AISB Master.\n\
                                 Type /help for commands.</i>",
                                chat_id,
                                if user_ids.is_empty() {
                                    "chat_id only".to_string()
                                } else {
                                    format!("<code>{:?}</code>", user_ids)
                                }
                            );
                            send_telegram_confirmation(&bot_token, chat_id, &confirm).await;

                            // 2) Ensure the SINGLE canonical poller is running.
                            //    The real bridge is the Bun bot shipped as the
                            //    systemd --user unit `omega-tg-bot.service`. We must
                            //    NOT spawn a competing Rust bridge (that produced two
                            //    pollers hitting getUpdates → permanent HTTP 409).
                            //    Kill any stale rmux bridge first, then enable+start
                            //    the canonical unit. The `enable --now` is wrapped in
                            //    a timeout so a slow/hung systemd can never FREEZE the
                            //    wizard UI (the "had to refresh manually" bug).
                            let mgr = SessionManager::connect().await?;
                            let _ = mgr.kill_session("omega-telegram-bridge").await;
                            let systemd_ok = matches!(
                                tokio::time::timeout(
                                    std::time::Duration::from_secs(8),
                                    tokio::process::Command::new("systemctl")
                                        .args(["--user", "enable", "--now", "omega-tg-bot.service"])
                                        .status(),
                                )
                                .await,
                                Ok(Ok(s)) if s.success()
                            );
                            if systemd_ok {
                                app.status_message = Some(
                                    "[+] Telegram setup done — bridge running as the persistent omega-tg-bot.service".to_string(),
                                );
                            } else {
                                app.status_message = Some(
                                    "[+] Telegram setup saved, but omega-tg-bot.service could not be started. Start it with: systemctl --user enable --now omega-tg-bot.service".to_string(),
                                );
                            }
                            // The bridge creates the aisb-master mirror session
                            // ASYNCHRONOUSLY on its own startup — racing the refresh
                            // below, which left the Sessions view empty until the
                            // user manually refreshed. Create the mirror SYNCHRONOUSLY
                            // here so the auto-refresh at the end of the wizard
                            // immediately shows the master (no manual refresh needed).
                            let omega_cfg = OmegaConfig::load().unwrap_or_default();
                            if omega_cfg.auto_spawn_master {
                                if let Some(agent) =
                                    omega_core::agents::Agent::from_name(&omega_cfg.aisb_agent)
                                {
                                    let cwd = std::env::current_dir()
                                        .ok()
                                        .and_then(|p| p.to_str().map(String::from))
                                        .unwrap_or_else(|| "/home".to_string());
                                    let _ = omega_core::aisb::ensure_master(&mgr, agent, &cwd).await;
                                }
                            }
                            let _ = app.refresh().await;
                            // Close the loop: drop the user onto the master's live
                            // mirror so they immediately SEE the confirmation
                            // message streaming in — all via Enter, no command.
                            app.tab = omega_tui::app::Tab::Sessions;
                            if app.select_by_name(omega_core::aisb::MASTER_SESSION_NAME) {
                                app.session_focus = omega_tui::app::SessionFocus::Chat;
                            }
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Telegram setup failed: {}", e));
                        }
                    }
                }
                Action::RunShellCommand { label, command } => {
                    let mgr = SessionManager::connect().await?;
                    let safe = label
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .take(20)
                        .collect::<String>();
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let name = format!("install-{}-{:06x}", safe, ts & 0xffffff);
                    // After install completes, auto-run `omega sync` to wire the
                    // new LLM into the centralized ~/.omega/ config
                    let post_install = if label.contains("nstall") {
                        "; echo '── syncing OmegaOS config ──'; omega sync 2>/dev/null || true"
                    } else {
                        ""
                    };
                    let cmd = format!("bash -c {}", shell_escape_for_bash(
                        &format!("{}{}; echo; echo '─── done ───'; exec bash", command, post_install)
                    ));
                    match mgr.create_session(&name, None, Some(&cmd)).await {
                        Ok(_) => {
                            // Drop the cached provider state so the [+]/[x] badge
                            // re-evaluates (live `command -v`) when the user
                            // returns to Settings after the install finishes.
                            app.invalidate_providers();
                            app.status_message = Some(format!(
                                "▶ '{}' running in session '{}'. Watch it there; come back to Settings to see [+]/[x] update.",
                                label.trim(), name
                            ));
                            auto_focus_chat(app, &name).await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Could not spawn session: {}", e));
                        }
                    }
                }
                Action::EditSettingsField { config_key, current, masked } => {
                    app.input_buffer = current;
                    app.input_mode = omega_tui::app::InputMode::EditSettingsField {
                        config_key: config_key.clone(),
                        masked,
                    };
                    app.status_message =
                        Some(format!("Editing {} — Enter to save, Esc to cancel", config_key));
                }
                Action::ToggleSettingsBool { config_key } => {
                    if let Err(e) = toggle_bool_config(&config_key) {
                        app.status_message = Some(format!("Toggle failed: {}", e));
                    } else {
                        app.status_message = Some(format!("Toggled {} — saved [+]", config_key));
                        // Reload the app's config so the change is reflected
                        app.config = OmegaConfig::load().unwrap_or_default();
                        // Bust the providers cache so Settings re-reads fresh.
                        app.invalidate_providers();
                    }
                }
                Action::CommitSettingsEdit { config_key, value } => {
                    let mut providers = omega_core::providers::ProvidersConfig::load();
                    if let Err(e) = set_config_value(&mut providers, &config_key, &value) {
                        app.status_message = Some(format!("Save failed: {}", e));
                    } else if let Err(e) = providers.save() {
                        app.status_message = Some(format!("Save failed: {}", e));
                    } else {
                        app.status_message =
                            Some(format!("Saved {} to providers.toml [+]", config_key));
                        // Bust the cache so the Settings panel reflects the
                        // value just typed (not the stale in-memory copy).
                        app.invalidate_providers();
                    }
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
                            app.status_message = Some("[+] Telegram bot disconnected".to_string());
                        }
                        Ok(false) => {
                            app.status_message =
                                Some("Nothing to disconnect — no Telegram config present".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Disconnect failed: {}", e));
                        }
                    }
                    // Return focus to section list so user can navigate to
                    // reconnect or pick another section.
                    app.detail_focused = false;
                    app.detail_scroll = 0;
                }
                Action::ForwardCharToSession { session, ch } => {
                    // HOT PATH (~1 call per keystroke in chat focus). Push onto
                    // the ordered forwarder channel — synchronous + non-blocking
                    // on an unbounded channel, so the loop returns instantly to
                    // read the next keystroke. The lone consumer delivers in
                    // FIFO order (per-keystroke tokio::spawn raced and reordered).
                    if ch == ' ' {
                        // Space char: route as the rmux "Space" key token. A
                        // bare " " literal sometimes renders as the word "space"
                        // in pane echo; the named key avoids that.
                        let _ = fwd_tx.send(forwarder::ForwardMsg::Key {
                            session,
                            key: "Space".to_string(),
                        });
                    } else {
                        let mut buf = [0u8; 4];
                        let s = ch.encode_utf8(&mut buf).to_string();
                        let _ = fwd_tx.send(forwarder::ForwardMsg::Text { session, text: s });
                    }
                    app.scroll_preview_end();
                    last_input_at = std::time::Instant::now();
                }
                Action::ForwardKeyToSession { session, key } => {
                    // HOT PATH: arrow keys / Enter / BackSpace / Escape in chat
                    // focus. Same ordered channel — a Key flushes any pending
                    // coalesced text before it, preserving interleave order.
                    let _ = fwd_tx.send(forwarder::ForwardMsg::Key {
                        session,
                        key: key.to_string(),
                    });
                    app.scroll_preview_end();
                    last_input_at = std::time::Instant::now();
                }
                Action::SendTextRawToSession { session, text } => {
                    // User paste — same ordered channel, but as a bracketed
                    // Paste block (no auto-Enter) so embedded newlines don't
                    // submit each line as a separate command in the target app.
                    let _ = fwd_tx.send(forwarder::ForwardMsg::Paste { session, text });
                    app.scroll_preview_end();
                    last_input_at = std::time::Instant::now();
                }
                Action::InsertNewlineToSession { session } => {
                    // Shift/Alt+Enter → newline-insert. Empirically Claude Code
                    // treats a trailing `\` + Enter as a literal newline (not a
                    // submit). Emit the backslash as text, then the Enter key —
                    // the Key handler flushes the pending text first, preserving
                    // order: `\` lands, then CR turns it into a newline.
                    let _ = fwd_tx.send(forwarder::ForwardMsg::Text {
                        session: session.clone(),
                        text: "\\".to_string(),
                    });
                    let _ = fwd_tx.send(forwarder::ForwardMsg::Key {
                        session,
                        key: "Enter".to_string(),
                    });
                    app.scroll_preview_end();
                    last_input_at = std::time::Instant::now();
                }
                Action::ForceRedraw => {
                    terminal.clear()?;
                    app.status_message = Some("Redrawn (Ctrl+L)".to_string());
                }
                Action::OpenProject { name, path, oracle_session } => {
                    let mgr = SessionManager::connect().await?;
                    // Attach to the project's Oracle session if it is alive.
                    let alive = if let Some(ref oracle) = oracle_session {
                        mgr.list_sessions()
                            .await
                            .map(|ss| ss.iter().any(|s| &s.name == oracle))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if let (true, Some(oracle)) = (alive, oracle_session.clone()) {
                        app.status_message = Some(format!("Attaching to oracle {}", oracle));
                        auto_focus_chat(app, &oracle).await;
                    } else {
                        // No live oracle → open a shell in the project dir.
                        let safe = name
                            .chars()
                            .filter(|c| c.is_alphanumeric() || *c == '-')
                            .take(24)
                            .collect::<String>();
                        let session = format!("{}-shell", safe);
                        let cmd = format!(
                            "bash -c {}",
                            shell_escape_for_bash(&format!("cd {} 2>/dev/null; exec bash", path))
                        );
                        match mgr.create_session(&session, Some(&path), Some(&cmd)).await {
                            Ok(_) => {
                                app.status_message =
                                    Some(format!("Opened shell in {} ({})", name, session));
                                auto_focus_chat(app, &session).await;
                            }
                            Err(e) => {
                                app.status_message =
                                    Some(format!("Could not open {}: {}", name, e));
                            }
                        }
                    }
                }
                Action::RunPlannerForProject { name, path } => {
                    let mgr = SessionManager::connect().await?;
                    let safe = name
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .take(24)
                        .collect::<String>();
                    let session = format!("{}-planner", safe);
                    let cmd = format!(
                        "bash -c {}",
                        shell_escape_for_bash(&format!(
                            "cd {} 2>/dev/null; omega planner; echo; echo '─── planner done ───'; exec bash",
                            path
                        ))
                    );
                    match mgr.create_session(&session, Some(&path), Some(&cmd)).await {
                        Ok(_) => {
                            app.status_message =
                                Some(format!("Running planner for {} ({})", name, session));
                            auto_focus_chat(app, &session).await;
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Planner spawn failed for {}: {}", name, e));
                        }
                    }
                }
                Action::RegisterProject { path } => {
                    let p = std::path::PathBuf::from(&path);
                    match omega_core::project_manager::add_existing_project(&p) {
                        Ok(proj) => {
                            app.refresh_projects();
                            // Select the freshly-added project.
                            if let Some(idx) = app
                                .project_registry
                                .projects
                                .iter()
                                .position(|x| x.name == proj.name)
                            {
                                app.projects_selected = idx;
                            }
                            app.status_message =
                                Some(format!("[+] Registered project '{}' ({})", proj.name, path));
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Could not register '{}': {}", path, e));
                        }
                    }
                }
                Action::ToggleProjectTelegram { name } => {
                    // Flip the per-project Telegram toggle in the shared registry.
                    // The next `/sync` reconciles the forum topic (creates when ON,
                    // removes when OFF); the Atlas bot marks OFF projects 🔕.
                    let mut registry = omega_core::project_manager::ProjectRegistry::load();
                    let now_on = registry
                        .find(&name)
                        .map(|p| p.telegram_enabled())
                        .unwrap_or(true);
                    if registry.set_telegram(&name, !now_on) {
                        match registry.save() {
                            Ok(()) => {
                                app.refresh_projects();
                                app.status_message = Some(format!(
                                    "[+] Telegram {} for '{}' — run /sync in the bot to update its topic",
                                    if now_on { "OFF 🔕" } else { "ON 🔔" },
                                    name
                                ));
                            }
                            Err(e) => {
                                app.status_message =
                                    Some(format!("Toggled '{}' but save failed: {}", name, e));
                            }
                        }
                    } else {
                        app.status_message = Some(format!("Project '{}' not found", name));
                    }
                }
                Action::DeleteProjectTier { name, mode } => {
                    // The SAME three escalating tiers as the Telegram delete menu,
                    // executed through the bot's one-shot CLI — ONE canonical
                    // deletion impl across every surface (TUI / Telegram / CLI):
                    //   omega → unmanage (topic + dashboard agent + agent-bot + registry)
                    //   local → that + kill oracle + rm -rf the local folder
                    //   all   → that + delete the GitHub repo (irreversible)
                    let omega_dir = std::env::var("OMEGA_DIR").unwrap_or_else(|_| {
                        format!(
                            "{}/.omega",
                            dirs::home_dir().unwrap_or_default().display()
                        )
                    });
                    let bot = format!("{}/telegram-bot/omega-tg-bot.ts", omega_dir);
                    let label = match mode {
                        "all" => "all (+ GitHub)",
                        "local" => "local machine",
                        _ => "OmegaOS view",
                    };
                    app.status_message = Some(format!("Deleting '{}' ({})…", name, label));
                    let out = std::process::Command::new("bun")
                        .args([bot.as_str(), "project-delete", name.as_str(), mode])
                        .output();
                    app.refresh_projects();
                    match out {
                        Ok(o) => {
                            let txt = String::from_utf8_lossy(&o.stdout);
                            let last = txt.lines().last().unwrap_or("done").trim().to_string();
                            app.status_message =
                                Some(format!("[x] Deleted '{}' ({}) — {}", name, label, last));
                        }
                        Err(e) => {
                            app.status_message = Some(format!(
                                "Delete failed to launch (bun): {} — run `bun {} project-delete {} {}`",
                                e, bot, name, mode
                            ));
                        }
                    }
                }
                Action::GroupSetupCommit { group_id } => {
                    // Preserve any existing topic mappings / name when re-running.
                    let mut cfg = omega_core::telegram_group::TelegramGroupConfig::load()
                        .unwrap_or_default();
                    cfg.group_id = group_id;
                    cfg.setup_at = chrono::Utc::now().to_rfc3339();
                    match cfg.save() {
                        Ok(()) => {
                            app.status_message = Some(format!(
                                "[+] Telegram project group saved (group_id {}). The bot maps one topic per project on first dispatch.",
                                group_id
                            ));
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Group setup save failed: {}", e));
                        }
                    }
                }
                Action::None => {}
            }

            // Auto-reframe: clear terminal when tab/focus changes so frames
            // never stay corrupted after resize or navigation
            if app.tab != tab_before || app.detail_focused != detail_focused_before {
                terminal.clear()?;
            }

            // Entering the Agentic tab (which now hosts the Projects group) →
            // reload the registry so projects added via `omega project add` in
            // another shell show up without restart.
            if app.tab == omega_tui::app::Tab::Agentic && tab_before != omega_tui::app::Tab::Agentic {
                app.refresh_projects();
            }

            // On tab switch, seed the status bar with a per-tab hint so a new
            // user lands with guidance instead of an empty bar. Skip if the
            // action handler already set a meaningful message this iteration
            // (e.g. a dispatch/login that also switched to the Sessions tab).
            if app.tab != tab_before && app.status_message == status_before {
                use omega_tui::app::Tab;
                app.status_message = Some(match app.tab {
                    Tab::Sessions => "↑/↓ select · Enter/Tab chat · c/C/g new agent · x kill · . lock · F5 refresh".to_string(),
                    Tab::Menu => "↑/↓ select · Enter run · or press the shortcut key shown".to_string(),
                    Tab::Settings => "↑/↓ Monitor + Settings sections · Enter/Tab edit · L login · T telegram · B billing".to_string(),
                    Tab::Agentic => "↑/↓ Agentic + Projects · Tab focus detail · n add · p plan · d dispatch · Enter open".to_string(),
                    Tab::Help => "↑/↓ scroll · Esc back to Sessions".to_string(),
                });
            }

            if app.selected != selected_before || app.tab != tab_before {
                let _ = app.refresh_preview().await;
            }

            // Bounded drain — don't let a flood starve the redraw loop.
            if drain_start.elapsed() > drain_budget {
                break;
            }
        }

        // Terminal was resized/zoomed this tick → clear the (now mis-sized)
        // frame and re-resolve the preview so the session view recovers at once
        // instead of getting stuck on "(session has no pane content)".
        if saw_resize {
            let _ = terminal.clear();
            let _ = app.refresh_preview().await;
            last_preview_refresh = std::time::Instant::now();
        }

        // First scroll-up out of tail mode: load scrollback NOW (this tick,
        // before the next draw) so the renderer has the full history and
        // computes a real max_scroll on the SAME frame — the view moves on
        // press #1 instead of needing a wasted second press. Only fires on the
        // tail→history transition (flag is set once in scroll_preview_up);
        // subsequent presses already have history and ride the normal cadence.
        if app.preview_needs_history {
            app.preview_needs_history = false;
            let _ = app.refresh_preview().await;
            last_preview_refresh = std::time::Instant::now();
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

/// Bootstrap a new project via the workflow-driven /omega-new-project pipeline.
/// Mirrors the TUI `Action::CreateProject` path: resolve the category dir from
/// config (never a hardcoded ~/VibeCoding), create it, and spawn a Claude
/// session whose first line invokes the v2 command. `--dry-run` prints the plan
/// and spawns nothing (zero mutation).
#[allow(clippy::too_many_arguments)]
async fn cmd_new_project(
    name: &str,
    stack: &str,
    category: &str,
    group: &str,
    resume: bool,
    from: Option<&str>,
    skip: Option<&str>,
    budget: Option<u64>,
    build: bool,
    dry_run: bool,
) -> Result<()> {
    let cfg = OmegaConfig::load().unwrap_or_default();
    let base = cfg.resolve_category_path(category);
    let project_dir = base.join(name);

    // Assemble the flag string passed through to the /omega-new-project command.
    let mut flags = String::new();
    if resume { flags.push_str(" --resume"); }
    if let Some(f) = from { flags.push_str(&format!(" --from={}", f)); }
    if let Some(s) = skip { flags.push_str(&format!(" --skip={}", s)); }
    if let Some(b) = budget { flags.push_str(&format!(" --budget={}", b)); }
    if build { flags.push_str(" --build"); }
    if dry_run { flags.push_str(" --dry-run"); }

    let prompt = format!("/omega-new-project {} {} {} {}{}", stack, category, name, group, flags);

    if dry_run {
        println!("omega new-project (DRY-RUN) — no session spawned, zero mutation");
        println!("  name:        {}", name);
        println!("  stack:       {}", stack);
        println!("  category:    {}", category);
        println!("  project dir: {}", project_dir.display());
        println!("  session:     {}-setup", name);
        println!("  invocation:  {}", prompt);
        println!("  DAG: P0 Capability -> P1 Provision(5//) -> GATE-A(2of3) -> P2 Scaffold(pipeline) -> P3 Wire -> GATE-B(2of3) -> vision -> prd -> brand -> deepux -> planner -> [build] -> audit/verify");
        return Ok(());
    }

    let _ = std::fs::create_dir_all(&base);
    let session = format!("{}-setup", name);
    let mgr = SessionManager::connect().await?;
    let agent = omega_core::agents::Agent::Claude;
    mgr.create_session_with_agent(&session, project_dir.to_str(), agent, Some(&prompt))
        .await?;
    println!("New project '{}' ({}/{}) — bootstrap running in session '{}'", name, stack, category, session);
    println!("  dir: {}", project_dir.display());
    Ok(())
}

async fn cmd_install_bindings() -> Result<()> {
    // Option+Z / Option+/ have been REMOVED — they didn't toggle (popup spawned
    // a nested omega instead of returning to the main one). Use Tab-Tab in the
    // TUI for fullscreen and Ctrl+Space / prefix+z for popup entry.
    let popup_cmd = "display-popup -E -w 100% -h 100% \"omega menu\"";

    // Root-table bindings (no prefix required) — single reliable shortcut
    let root_bindings: Vec<(&str, &str)> = vec![
        ("C-Space", "Open OmegaOS menu (Ctrl+Space)"),
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
                println!("[+] {} → {}", key, desc);
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
                println!("[+] C-b {} → {}", key, desc);
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
    let content = r#"# OmegaOS rmux bindings — open the session menu from any rmux session.
#
# Option+Z and Option+/ were REMOVED — they spawned a nested popup that
# couldn't return to the parent omega cleanly. Use Tab-Tab inside the TUI
# for fullscreen, and one of these to open omega from anywhere:
#
# Source from your ~/.rmux.conf with:
#   source-file ~/.omega/rmux.conf.omega
#
# Root-table (no prefix):
bind-key -n C-Space display-popup -E -w 100% -h 100% "omega menu"

# Prefix-table (C-b first, then key):
bind-key o display-popup -E -w 100% -h 100% "omega menu"
bind-key z display-popup -E -w 100% -h 100% "omega menu"
"#;
    std::fs::write(&conf_path, content)?;
    println!("[+] Persistent config written to {}", conf_path.display());

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
            println!("[+] Added source-file to {}", rmux_conf.display());
        } else {
            println!("[+] ~/.rmux.conf already sources OmegaOS bindings");
        }
    }

    println!();
    println!("Open the OmegaOS menu from any rmux session with:");
    println!("  • Ctrl+Space        — most reliable, no prefix needed");
    println!("  • Ctrl+B then o     — prefix chord");
    println!("  • Ctrl+B then z     — prefix chord (alternate)");
    println!();
    println!("Inside the TUI: Tab toggles chat focus, Tab-Tab → fullscreen.");

    // Also remove any stale Option+Z / Option+/ bindings the user might have
    // from earlier OmegaOS versions.
    for stale in &["M-z", "M-/"] {
        let _ = std::process::Command::new("rmux")
            .args(["unbind-key", "-n", stale])
            .output();
    }

    Ok(())
}

fn cmd_install(agent_name: &str, dry_run: bool) -> Result<()> {
    let agent = omega_core::agents::Agent::from_name(agent_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown agent: {}", agent_name))?;

    let cmd = agent.install_command().ok_or_else(|| {
        anyhow::anyhow!(
            "{} has no install command (already bundled or no public installer)",
            agent.display_name()
        )
    })?;

    if agent.is_available() && !dry_run {
        println!("[+] {} is already installed.", agent.display_name());
        println!("  Re-run with `--dry-run` to see the install command anyway.");
        return Ok(());
    }

    println!("Installing {} via:", agent.display_name());
    println!("  $ {}", cmd);

    if dry_run {
        println!("\n(dry-run — nothing executed)");
        return Ok(());
    }

    if let Some(homepage) = agent.homepage() {
        println!("\nProject homepage: {}", homepage);
    }
    println!();

    // Execute the install command in a shell so curl pipes work.
    let status = std::process::Command::new("bash")
        .args(["-c", cmd])
        .status()
        .context("running installer")?;

    if !status.success() {
        anyhow::bail!(
            "Installer exited with status {:?}",
            status.code().unwrap_or(-1)
        );
    }

    // Verify
    if agent.is_available() {
        println!("\n[+] {} is now installed and on PATH.", agent.display_name());
    } else {
        println!(
            "\n[!] Installer reported success but `{}` is not on PATH yet.",
            agent.name()
        );
        println!("  You may need to restart your shell or add the binary directory to PATH.");
    }

    // Auto-sync: wire the new LLM into ~/.omega/ centralized config
    println!("\nSyncing OmegaOS config...");
    let _ = cmd_sync();

    Ok(())
}

/// `omega trust-dir [dir]` — pre-trust a folder in ~/.claude.json (see
/// omega_core::claude_trust). Called inline by every Claude launch command;
/// exits 0 with a one-line status either way so the launch never breaks.
fn cmd_trust_dir(dir: Option<&str>) -> Result<()> {
    let dir = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()?,
    };
    // Canonicalize so "." / relative paths land on the same key claude uses.
    let dir = dir.canonicalize().unwrap_or(dir);
    match omega_core::claude_trust::trust_dir(&dir) {
        Ok(true) => println!("trusted: {}", dir.display()),
        Ok(false) => println!("already trusted: {}", dir.display()),
        Err(e) => println!("trust-dir skipped ({}): {}", dir.display(), e),
    }
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

#[derive(Subcommand)]
enum RulesAction {
    /// List all operational rules
    List,
    /// Export compiled rules to ~/.omega/rules/ as individual .md files
    Export,
    /// Print the role-scoped doctrine block (Laws + Rules + orchestration) to inject
    /// into any agent prompt. scope = master | oracle | worker.
    Context {
        /// Agent scope: master | oracle | worker (default: oracle)
        #[arg(default_value = "oracle")]
        scope: String,
    },
}

#[derive(Subcommand)]
enum ProvisionAction {
    /// List credential groups (default = the shared services.env).
    Groups,
    /// Show which provisioning keys a group has set (values never printed).
    Show {
        /// Group name (or `default`).
        group: String,
    },
    /// Set tokens in a group: omega provision set <group> KEY=VALUE [KEY=VALUE...]
    Set {
        /// Group name (a new name creates the group; `default` = shared).
        group: String,
        /// One or more KEY=VALUE pairs.
        #[arg(required = true)]
        kv: Vec<String>,
    },
    /// Live-verify a group's tokens against each service API (curl).
    Verify {
        /// Group name (or `default`). Defaults to `default`.
        #[arg(default_value = "default")]
        group: String,
    },
}

#[derive(Subcommand)]
enum AuditAction {
    /// List all 23 Quality Arsenal audits with metadata
    List,
    /// Show which audits would be selected for a mission
    Select {
        /// Mission text to match against
        mission: String,
    },
    /// Show audit results for an oracle/mission
    Results {
        /// Oracle session name
        oracle: String,
    },
    /// Show an audit's metadata + print the spawn-worker command to run it
    Run {
        /// Audit id (e.g. codeaudit, secaudit)
        audit_id: String,
        /// Working directory for the audit
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}

#[derive(Subcommand)]
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
            println!("[+] Telegram config saved to ~/.omega/telegram.toml");
            if !cfg.label.is_empty() {
                println!("  Label:         {}", cfg.label);
            }
            println!("  Relay session: {}", cfg.relay_session);
            println!("  Chat ID:       {}", cfg.chat_id);
            if cfg.allow_user_ids.is_empty() {
                println!("  Sender filter: only chat_id={} accepted", cfg.chat_id);
                println!("  [!] For shared chats, restrict further with --user-id");
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
                true => println!("[+] Telegram bot disconnected (~/.omega/telegram.toml removed)"),
                false => println!("(nothing to disconnect — no config present)"),
            }
            Ok(())
        }
        TelegramAction::Enable => {
            if let Some(mut cfg) = OmegaTelegramConfig::read() {
                cfg.enabled = true;
                cfg.write()?;
                println!("[+] Telegram bot enabled");
            } else {
                anyhow::bail!("Not configured. Run: omega telegram setup …");
            }
            Ok(())
        }
        TelegramAction::Disable => {
            if let Some(mut cfg) = OmegaTelegramConfig::read() {
                cfg.enabled = false;
                cfg.write()?;
                println!("[+] Telegram bot disabled");
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
            // The Telegram bot is the Bun bot (omega-tg-bot.ts): its Claude session
            // IS Atlas, the 13 agents live in its system prompt, and it
            // dispatches to per-project oracles via the `omega` CLI. We exec it so
            // this process becomes the bot — the SAME entry point the systemd
            // service uses. (The legacy native Rust bridge was removed: the Bun bot
            // is the single canonical implementation, shipped by install.sh.)
            use std::os::unix::process::CommandExt;
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            let bot_ts = home.join(".omega/telegram-bot/omega-tg-bot.ts");
            if !bot_ts.exists() {
                anyhow::bail!(
                    "Telegram bot not found at {}. Reinstall OmegaOS (install.sh ships the Bun bot).",
                    bot_ts.display()
                );
            }
            println!("◆ Launching OmegaOS Telegram bot (Bun) — Atlas + 13 agents");
            // exec() replaces this process; it only returns on failure.
            let err = std::process::Command::new("bun").arg(&bot_ts).exec();
            anyhow::bail!("Failed to launch the Bun Telegram bot ({err}). Is `bun` installed and on PATH?");
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
    /// List the canonical providers (no arg) or a provider's known models (one per
    /// line). SSOT for any UI building a model picker (TUI, Telegram) so the curated
    /// lists live ONLY in providers.rs::models_for / all_providers.
    Models { provider: Option<String> },
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
            println!("[+] Set {} = {}", key, value);
            println!("Applies to all newly spawned sessions.");
        }
        ConfigAction::Models { provider } => match provider {
            // No provider → the canonical provider list. With one → its known models.
            // Empty list (unknown provider) prints nothing and exits 0 so callers can
            // fall back to a free-text field.
            None => {
                for p in ProvidersConfig::all_providers() {
                    println!("{}", p);
                }
            }
            Some(p) => {
                for m in ProvidersConfig::models_for(&p) {
                    println!("{}", m);
                }
            }
        },
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
        ("pi", "api_key") => cfg.pi.api_key.clone(),
        ("glm", "model") => cfg.glm.model.clone(),
        ("glm", "api_key") => cfg.glm.api_key.clone(),
        ("hermes", "model") => cfg.hermes.model.clone(),
        ("hermes", "api_key") => cfg.hermes.api_key.clone(),
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
        ("pi", "api_key") => cfg.pi.api_key = value.to_string(),
        ("glm", "model") => cfg.glm.model = value.to_string(),
        ("glm", "api_key") => cfg.glm.api_key = value.to_string(),
        ("hermes", "model") => cfg.hermes.model = value.to_string(),
        ("hermes", "api_key") => cfg.hermes.api_key = value.to_string(),
        _ => anyhow::bail!("Unknown key: {}", key),
    }
    Ok(())
}

fn cmd_monitor() -> Result<()> {
    use omega_core::monitor;
    let snap = monitor::UsageSnapshot::read().ok().flatten().unwrap_or_default();
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

/// Print the display clock (rmux status bar calls this every status-interval).
/// Single source of truth with the TUI clock: both read `config.timezone`.
/// Always exits 0 with a sane string so a config error never breaks the bar.
fn cmd_clock(full: bool) -> Result<()> {
    let tz = omega_core::config::OmegaConfig::load()
        .ok()
        .and_then(|c| c.timezone);
    let fmt = if full { "%H:%M %d-%b-%y" } else { "%H:%M" };
    println!("{}", omega_core::clock::now_fmt(tz.as_deref(), fmt));
    Ok(())
}

/// The rmux session NAME this process runs inside, or None if not in rmux.
/// Resolved by asking rmux to expand `#{session_name}` for our pane
/// (`$RMUX_PANE`). `$RMUX` only carries `socket,server_pid,session_id` — never
/// the name — so parsing it for a name silently fails; this is the correct way.
fn current_session_name() -> Option<String> {
    let pane = std::env::var("RMUX_PANE").ok().filter(|p| !p.is_empty())?;
    let out = std::process::Command::new("rmux")
        .args(["display-message", "-p", "-t", &pane, "#{session_name}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Remove junk rmux sessions — ones omega could not have created (their rmux
/// name isn't its own sanitized slug). Dry-run unless `force`. The current
/// session is always kept so this never kills the shell you're running it from.
async fn cmd_clean_junk(force: bool) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    let mut keep = std::collections::HashSet::new();
    if let Ok(v) = std::env::var("RMUX") {
        if let Some(name) = v.split(',').next() {
            keep.insert(name.to_string());
        }
    }
    let junk = omega_core::cleanup::find_junk_sessions(&sessions, &keep);
    if junk.is_empty() {
        println!("✓ No junk sessions — every session name is a clean slug.");
        return Ok(());
    }
    println!(
        "Junk sessions (rmux name ≠ sanitized slug — not created by omega):"
    );
    for j in &junk {
        println!("  • {:?}", j);
    }
    if !force {
        println!(
            "\nDry run — nothing killed. Re-run with --force to kill these {} session(s).",
            junk.len()
        );
        return Ok(());
    }
    let mut killed = 0;
    for j in &junk {
        // Kill via the RAW rmux name — NOT mgr.kill_session, which sanitizes the
        // target and would look for a slug that doesn't exist ("Session not
        // found"). Junk sessions are exactly the ones whose real name has chars
        // sanitize strips, so we must hand rmux the original bytes verbatim.
        let out = tokio::process::Command::new("rmux")
            .args(["kill-session", "-t", j])
            .output()
            .await;
        match out {
            Ok(o) if o.status.success() => {
                println!("  ✗ killed {:?}", j);
                killed += 1;
            }
            Ok(o) => println!(
                "  ! failed {:?}: {}",
                j,
                String::from_utf8_lossy(&o.stderr).trim()
            ),
            Err(e) => println!("  ! failed {:?}: {}", j, e),
        }
    }
    println!("Removed {} junk session(s).", killed);
    Ok(())
}

fn cmd_agents() -> Result<()> {
    println!("Available agents:\n");
    for agent in omega_core::agents::Agent::all() {
        let status = if agent.is_available() { "[+]" } else { "[x]" };
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

/// `omega claude-login` — start the OAuth re-login engine and print the captured
/// authorize URL as JSON. Headless (no TUI); shared by the TUI + Telegram bridge.
/// Prints `{"ok":true,"url":"..."}` on success, `{"ok":false,"error":"..."}` and
/// exits non-zero on failure.
async fn cmd_claude_login() -> Result<()> {
    let mgr = match SessionManager::connect().await {
        Ok(m) => m,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": format!("connect failed: {}", e) })
            );
            std::process::exit(1);
        }
    };
    match omega_core::oauth::request_reauth(&mgr, "cli", None, true).await {
        Ok(Some(req)) => {
            println!("{}", serde_json::json!({ "ok": true, "url": req.auth_url }));
            Ok(())
        }
        // A None result means a reauth is already pending or the cooldown is
        // active — surface it as a non-fatal informational error for the caller.
        Ok(None) => {
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": "reauth already pending or on cooldown" })
            );
            std::process::exit(1);
        }
        Err(e) => {
            println!("{}", serde_json::json!({ "ok": false, "error": e.to_string() }));
            std::process::exit(1);
        }
    }
}

/// `omega claude-login-code <code>` — finish the OAuth re-login by pasting the
/// authorize code into the waiting reauth session. Prints
/// `{"ok":bool,"email":...,"expires_min":...}`; exits non-zero on failure.
async fn cmd_claude_login_code(code: &str) -> Result<()> {
    let mgr = match SessionManager::connect().await {
        Ok(m) => m,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": format!("connect failed: {}", e) })
            );
            std::process::exit(1);
        }
    };
    match omega_core::oauth::handle_code(&mgr, code).await {
        Ok(res) => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": res.success,
                    "email": res.email,
                    "expires_min": res.expires_min,
                })
            );
            if !res.success {
                std::process::exit(1);
            }
            Ok(())
        }
        Err(e) => {
            println!("{}", serde_json::json!({ "ok": false, "error": e.to_string() }));
            std::process::exit(1);
        }
    }
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
            println!("[+] Mission completed successfully");
        }
        omega_core::mission::OutcomeStatus::PartialSuccess => {
            println!("[!] Mission partially completed");
        }
        omega_core::mission::OutcomeStatus::Failed => {
            println!("[x] Mission failed");
            std::process::exit(1);
        }
        omega_core::mission::OutcomeStatus::Aborted => {
            println!("⊘ Mission aborted");
            std::process::exit(2);
        }
    }

    Ok(())
}

/// Read-only plan progress from .planner/tracker.json.
fn cmd_plan_status(path: &str) -> Result<()> {
    let dir = std::path::Path::new(path);
    let tracker = omega_core::planner::PlanTracker::load(dir)
        .ok_or_else(|| anyhow::anyhow!("no .planner/tracker.json in {path}"))?;
    let st = tracker.status();
    println!(
        "Plan: {} | {:.0}% ({}/{} done) | ready {} | blocked {} | failed {} | phase {}/{}",
        tracker.project,
        st.progress_pct(),
        st.done,
        st.total,
        st.ready,
        st.blocked,
        st.failed,
        st.active_phase,
        st.total_phases,
    );
    for s in &tracker.steps {
        println!("  {} {} {}", s.status.icon(), s.step_id, s.title);
    }
    // Surface the same strict gate `plan-run` enforces — so the planner can fix the
    // tracker BEFORE dispatching workers (dangling deps / trivial verifies / dups).
    match tracker.validate() {
        Ok(()) => println!("\n[+] plan validation: OK (no dangling deps, no trivial verify_commands, no dup ids)"),
        Err(e) => println!("\n[!] plan validation FAILED — `omega plan-run` will refuse this plan:\n{e}"),
    }
    Ok(())
}

/// Drive a plan to completion via the executor. Spawns one real rmux worker
/// per ready step (RmuxRuntime), gates every completion through the Guardian.
async fn cmd_plan_run(path: &str) -> Result<()> {
    use omega_core::executor::{run, RmuxRuntime, RunOptions};

    let dir = std::path::Path::new(path);
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    // Construct the SessionManager exactly as Orchestrator::new does.
    let mgr = SessionManager::connect().await?;
    let project = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let runtime = RmuxRuntime {
        mgr: &mgr,
        state_dir: config.state_dir.clone(),
        project,
        agent: omega_core::agents::Agent::Claude,
        poll: std::time::Duration::from_secs(5),
    };

    let report = run(dir, &runtime, RunOptions::default()).await?;
    println!(
        "Run finished: success={} | completed {} | failed {:?} | blocked {:?}",
        report.success,
        report.completed.len(),
        report.failed,
        report.blocked,
    );
    if !report.success {
        std::process::exit(1);
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
    force: bool,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    // The current rmux session IS the oracle when spawn-worker runs inside one.
    // Capture it to register the worker under it + derive the project.
    //
    // NOTE: $RMUX is "socket,server_pid,session_id" — it does NOT carry the
    // session NAME, so the old `RMUX.split(',').next()` read the socket PATH and
    // never matched "oracle-*". Result: the worker→oracle link was silently
    // dropped for EVERY worker, so the menu could never nest workers under their
    // oracle. Resolve the real name by asking rmux to expand #{session_name} for
    // our pane ($RMUX_PANE).
    let oracle_session = current_session_name().filter(|s| s.starts_with("oracle-"));

    let project_name = match project {
        Some(p) => Some(p.to_string()),
        None => oracle_session
            .as_deref()
            .and_then(|s| omega_core::session::OmegaSession::classify(s).project),
    };

    let work_dir = dir.unwrap_or(".");
    let worker_name = match &project_name {
        Some(p) => format!("{}-worker-{}", p, task),
        None => format!("worker-{}", task),
    };

    // Worker-prompt completeness gate (cheap, no LLM): the brief MUST carry both a
    // Done-criteria signal AND a Verify-command signal. Checked BEFORE the scope
    // claim so a rejected dispatch never leaves a file lock behind.
    let prompt_lc = prompt.to_lowercase();
    let has_done = prompt_lc.contains("done criteria")
        || prompt_lc.contains("done:")
        || prompt_lc.contains("done-criteria");
    let has_verify = prompt_lc.contains("verify");
    if !(has_done && has_verify) {
        let missing = match (has_done, has_verify) {
            (false, false) => "Done Criteria + Verify Command",
            (false, true) => "Done Criteria",
            (true, false) => "Verify Command",
            (true, true) => unreachable!(),
        };
        if force {
            tracing::warn!(
                "worker prompt missing {} — --force set, dispatching anyway (quality gate may fail)",
                missing
            );
            eprintln!(
                "[!] worker prompt missing {} — --force set, dispatching anyway (quality gate may fail)",
                missing
            );
        } else {
            anyhow::bail!(
                "worker prompt missing {missing}. Add explicit \"Done Criteria:\" and a \"Verify Command:\" \
                 to the prompt so the worker has measurable success criteria (rule R-RUBRIC), or pass --force to override."
            );
        }
    }

    if let Some(ref files) = files {
        omega_core::scope::claim_or_reject(&config.state_dir, &worker_name, files.clone())?;
    }

    // THE FUNNEL — inject the Worker-scoped Laws + operational rules, exactly
    // like Dispatcher::dispatch_worker_with_context. Without this, a worker
    // spawned via the CLI (the live path oracles use) gets NO doctrine.
    let mut full_prompt = prompt.to_string();
    let agent_ctx = omega_core::rules::agent_context_block(omega_core::rules::RuleScope::Worker);
    if !agent_ctx.is_empty() {
        full_prompt.push_str("\n\n");
        full_prompt.push_str(&agent_ctx);
    }

    // Per-role LaunchOptions for the WORKER (Claude only — other providers
    // ignore the Claude-only fields). A worker is a hermetic, trusted executor:
    //   * permission-mode "bypassPermissions" — never prompt the operator (every
    //     OmegaOS session runs fully autonomous; "acceptEdits" still gated on Bash
    //     and stalled hermetic workers waiting for an answer no one gives).
    //   * disallowed_tools — the real safety rail (orthogonal to permission mode,
    //     a hard deny that survives bypass): the destructive/irreversible ops a
    //     worker must never run (git push, rm, sudo). Oracles keep full access.
    //   * mcp_config + --strict-mcp-config — ONLY the OmegaOS MCP servers, no
    //     user/project .mcp.json (hermetic).
    //   * NO --bare — bare mode skips OAuth credential loading in Claude Code
    //     >= 2.1.x, so a bare worker dies at the login screen (see below).
    let agent = omega_core::agents::Agent::from_name(&config.agent_command)
        .unwrap_or(omega_core::agents::Agent::Claude);
    let spawn_result = if matches!(agent, omega_core::agents::Agent::Claude) {
        let mut opts = omega_core::agents::LaunchOptions::default();
        opts.permission_mode = Some("bypassPermissions".to_string());
        opts.disallowed_tools = Some("Bash(git push:*) Bash(rm:*) Bash(sudo:*)".to_string());
        // NOT bare: --bare skips OAuth credential loading in Claude Code >= 2.1.x
        // (runtime-verified 2026-06-05: `claude --bare --print` -> "Not logged in"
        // while plain `claude --print` succeeds on an OAuth-only host), so hermetic
        // workers must NOT use bare mode until upstream fixes it.
        match omega_core::mcp_servers::generate_mcp_config(&config, &worker_name) {
            Ok(json) => {
                let path = config.state_dir.join(format!("{}.mcp.json", worker_name));
                match std::fs::write(&path, json) {
                    Ok(()) => {
                        opts.mcp_config = Some(vec![path.to_string_lossy().to_string()]);
                        opts.strict_mcp_config = true;
                    }
                    Err(e) => tracing::warn!(
                        worker = %worker_name, error = %e,
                        "failed to write worker mcp-config — launching without it"
                    ),
                }
            }
            Err(e) => tracing::warn!(
                worker = %worker_name, error = %e,
                "failed to generate worker mcp-config — launching without it"
            ),
        }
        mgr.create_agent_session_with_opts(&worker_name, work_dir, agent, Some(&full_prompt), opts)
            .await
    } else {
        mgr.create_agent_session(&worker_name, work_dir, &config.agent_command, Some(&full_prompt))
            .await
    };
    if let Err(e) = spawn_result {
        // Roll back the scope claim so a failed spawn doesn't lock files forever.
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, &worker_name);
        return Err(e);
    }

    // Register the worker under its oracle so the patrol routes its done/blocked
    // events to the right parent and the TUI shows it under the oracle.
    if let Some(ref oracle_name) = oracle_session {
        // Serialize the read-modify-write of the oracle state behind an exclusive
        // advisory lock so two concurrent spawns can't both read the old state,
        // both append their worker, and have the second write clobber the first's
        // entry (the idempotent check in register_worker only dedups the SAME
        // session, not concurrent different sessions). std-only mutex: an atomic
        // create_dir on a per-oracle lock dir, bounded-spin then proceed best-effort.
        let lock_dir = config
            .state_dir
            .join(format!(".oracle-{}.lock", oracle_name));
        let mut held_lock = false;
        for _ in 0..50 {
            match std::fs::create_dir(&lock_dir) {
                Ok(()) => {
                    held_lock = true;
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(_) => break, // unexpected IO error — proceed unlocked, best-effort
            }
        }

        // Upsert: if the oracle never wrote a full state, create a minimal one
        // so the worker→oracle link is ALWAYS persisted. Previously this only
        // updated an EXISTING state and silently dropped the link otherwise —
        // which is why the menu couldn't nest these workers under their oracle.
        let mut state = omega_core::oracle_lifecycle::OracleState::read(&config.state_dir, oracle_name)
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                omega_core::oracle_lifecycle::OracleState::new_minimal(
                    oracle_name,
                    project_name.as_deref().unwrap_or(""),
                    std::path::PathBuf::from(work_dir),
                )
            });
        state.register_worker(omega_core::oracle_lifecycle::WorkerEntry {
            session_name: worker_name.clone(),
            task_id: task.to_string(),
            task_name: task.to_string(),
            files_owned: files.clone().unwrap_or_default(),
            dispatched_at: chrono::Utc::now(),
            status: omega_core::oracle_lifecycle::WorkerEntryStatus::Running,
        });
        // The worker is ALREADY spawned at this point, so a write failure cannot
        // be rolled back by releasing the scope (that would free the files the
        // running worker still owns and let another worker clobber them). Instead
        // surface the failure loudly + retry a few times so the worker→oracle link
        // isn't silently lost; patrol can still reconcile an orphan from the error.
        let mut last_err = None;
        for attempt in 0..3 {
            match state.write(&config.state_dir) {
                Ok(()) => {
                    last_err = None;
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        }
        if let Some(e) = last_err {
            tracing::error!(
                worker = %worker_name, oracle = %oracle_name, error = %e,
                "worker spawned but FAILED to persist worker→oracle registration after retries — \
                 worker is running orphaned (not nested under oracle); patrol must reconcile"
            );
            eprintln!(
                "[!] worker {} spawned but registration under oracle {} failed: {} \
                 (worker is running; it will not show nested under its oracle until reconciled)",
                worker_name, oracle_name, e
            );
        }

        if held_lock {
            let _ = std::fs::remove_dir(&lock_dir);
        }
    }

    println!("● Worker spawned: {}", worker_name);
    if let Some(p) = &project_name {
        println!("  Under project: {}", p);
    }
    if let Some(ref files) = files {
        println!("  Scope claimed: {}", files.join(", "));
    }
    Ok(())
}

/// Read comma-separated doc paths into a brief: inline small files (<=10KB),
/// reference larger/missing ones by path. Lets a new project start from existing
/// docs ("j'ai des docs qu'on peut utiliser directement").
fn read_launch_docs(paths: &str) -> String {
    let mut out = String::new();
    for raw in paths.split(',') {
        let p = raw.trim();
        if p.is_empty() {
            continue;
        }
        let expanded = match p.strip_prefix("~/") {
            Some(rest) => dirs::home_dir()
                .map(|h| h.join(rest))
                .unwrap_or_else(|| std::path::PathBuf::from(p)),
            None => std::path::PathBuf::from(p),
        };
        match std::fs::metadata(&expanded) {
            Ok(m) if m.is_file() && m.len() <= 10_240 => {
                if let Ok(content) = std::fs::read_to_string(&expanded) {
                    out.push_str(&format!("\n## {}\n{}\n", p, content));
                    continue;
                }
            }
            _ => {}
        }
        out.push_str(&format!("\n## {} (reference — read it yourself)\n", p));
    }
    out
}

/// The rmux session this process runs inside, if any (first RMUX field).
fn current_session() -> Option<String> {
    std::env::var("RMUX")
        .ok()
        .and_then(|v| v.split(',').next().map(|s| s.to_string()))
}

/// keep-set for kill-all / cleanup: current session + infrastructure singletons.
fn cleanup_keep_set(
    sessions: &[omega_core::session::OmegaSession],
) -> std::collections::HashSet<String> {
    let mut keep = omega_core::cleanup::infrastructure_keep(sessions);
    if let Some(cur) = current_session() {
        keep.insert(cur);
    }
    keep
}

async fn cmd_kill_all(yes: bool) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    let keep = cleanup_keep_set(&sessions);
    let targets: Vec<String> = sessions
        .iter()
        .map(|s| s.name.clone())
        .filter(|n| !keep.contains(n))
        .collect();
    if targets.is_empty() {
        println!("Nothing to kill — only the current + infrastructure sessions are live.");
        return Ok(());
    }
    if !yes {
        println!("Would kill {} session(s):", targets.len());
        for t in &targets {
            println!("  [x] {}", t);
        }
        println!("Re-run with --yes to kill them.");
        return Ok(());
    }
    let killed = omega_core::cleanup::kill_all(&mgr, &keep).await?;
    println!("Killed {} session(s): {}", killed.len(), killed.join(", "));
    Ok(())
}

async fn cmd_cleanup(yes: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    let keep = cleanup_keep_set(&sessions);
    if !yes {
        let targets = omega_core::cleanup::killable(&mgr, &keep).await;
        println!("NUCLEAR CLEANUP — would:");
        println!("  -kill {} session(s): {}", targets.len(), targets.join(", "));
        println!("  -prune stale state from dead sessions (scope claims, done/blocked signals)");
        println!("  -clear /tmp omega-*/claude-* scratch");
        println!("  -drop the Linux page cache (if permitted)");
        println!("Re-run with --yes to execute.");
        return Ok(());
    }
    let report = omega_core::cleanup::nuclear_cleanup(&mgr, &config, &keep).await?;
    println!("Nuclear cleanup complete.");
    println!("  {}", report.summary());
    for note in &report.notes {
        println!("  - {}", note);
    }
    Ok(())
}

fn cmd_provision(action: ProvisionAction) -> Result<()> {
    use omega_core::provisioning;
    match action {
        ProvisionAction::Groups => {
            println!("Credential groups (default = shared services.env):");
            for g in provisioning::list_groups() {
                let p = provisioning::group_env_path(&g);
                let mark = if p.exists() { "" } else { "  (empty)" };
                println!("  {:18} {}{}", g, p.display(), mark);
            }
        }
        ProvisionAction::Show { group } => {
            println!(
                "Group '{}' → {}",
                group,
                provisioning::group_env_path(&group).display()
            );
            for key in [
                "VERCEL_TOKEN",
                "CONVEX_TEAM_TOKEN",
                "CONVEX_TEAM_SLUG",
                "GITHUB_TOKEN",
                "STRIPE_SECRET_KEY",
            ] {
                let set = provisioning::read_value_in(&group, key).is_some();
                println!("  {:20} {}", key, if set { "set" } else { "-" });
            }
        }
        ProvisionAction::Set { group, kv } => {
            let mut updates = Vec::new();
            for pair in &kv {
                match pair.split_once('=') {
                    Some((k, v)) => updates.push((k.trim().to_string(), v.trim().to_string())),
                    None => eprintln!("skipping '{}' (expected KEY=VALUE)", pair),
                }
            }
            if updates.is_empty() {
                anyhow::bail!("no valid KEY=VALUE pairs");
            }
            provisioning::update_group_env(&group, &updates)?;
            println!(
                "Updated {} key(s) in group '{}' → {}",
                updates.len(),
                group,
                provisioning::group_env_path(&group).display()
            );
        }
        ProvisionAction::Verify { group } => {
            provision_verify(&group)?;
        }
    }
    Ok(())
}

/// Live-verify a credential group's tokens by SHELLING OUT to `curl` (no new
/// HTTP crate dep). Blank tokens are never invented — they print MISSING and
/// the live call is skipped. Prints a clean per-service table.
fn provision_verify(group: &str) -> Result<()> {
    use omega_core::provisioning;

    // Resolve a token from the group (None = blank/unset).
    let tok = |key: &str| provisioning::read_value_in(group, key);

    // One live HTTP status probe via curl. Returns the numeric status (or None
    // on transport failure). `auth` is the full -H / -u argument list.
    fn http_status(url: &str, auth: &[&str]) -> Option<u32> {
        let mut cmd = std::process::Command::new("curl");
        cmd.args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "15"]);
        cmd.args(auth);
        cmd.arg(url);
        let out = cmd.output().ok()?;
        String::from_utf8_lossy(&out.stdout).trim().parse::<u32>().ok()
    }

    println!(
        "Provision verify — group '{}' → {}",
        group,
        provisioning::group_env_path(group).display()
    );
    println!("  {:<10} {}", "SERVICE", "RESULT");

    // GitHub.
    match tok("GITHUB_TOKEN") {
        Some(t) => {
            let auth = format!("Authorization: Bearer {}", t);
            let status = http_status("https://api.github.com/user", &["-H", &auth]);
            println!("  {:<10} {}", "GitHub", fmt_status(status, 200));
        }
        None => println!("  {:<10} MISSING (needs operator input)", "GitHub"),
    }

    // Vercel.
    match tok("VERCEL_TOKEN") {
        Some(t) => {
            let auth = format!("Authorization: Bearer {}", t);
            let status = http_status("https://api.vercel.com/v2/user", &["-H", &auth]);
            println!("  {:<10} {}", "Vercel", fmt_status(status, 200));
        }
        None => println!("  {:<10} MISSING (needs operator input)", "Vercel"),
    }

    // Convex — no simple public probe; presence check only.
    match tok("CONVEX_TEAM_TOKEN") {
        Some(_) => println!("  {:<10} set (presence only — no public probe)", "Convex"),
        None => println!("  {:<10} MISSING (needs operator input)", "Convex"),
    }

    // Stripe.
    match tok("STRIPE_SECRET_KEY") {
        Some(t) => {
            let userpass = format!("{}:", t);
            let status = http_status("https://api.stripe.com/v1/balance", &["-u", &userpass]);
            println!("  {:<10} {}", "Stripe", fmt_status(status, 200));
        }
        None => println!("  {:<10} MISSING (needs operator input)", "Stripe"),
    }

    // Clerk — report the configured provisioning mode, not a live call.
    match tok("CLERK_PROVISION_MODE") {
        Some(mode) => println!("  {:<10} mode={}", "Clerk", mode),
        None => println!("  {:<10} mode=unset", "Clerk"),
    }

    Ok(())
}

/// Render an HTTP probe result: `OK (200)` on the expected code, `FAIL (<code>)`
/// otherwise, or `ERROR (no response)` when curl gave no status.
fn fmt_status(status: Option<u32>, expected: u32) -> String {
    match status {
        Some(c) if c == expected => format!("OK ({})", c),
        Some(c) => format!("FAIL ({})", c),
        None => "ERROR (no response)".to_string(),
    }
}

async fn cmd_resurrect(oracle: Option<String>) -> Result<()> {
    use omega_core::dispatch::ResurrectOutcome;
    let config = OmegaConfig::load().unwrap_or_default();
    let mgr = SessionManager::connect().await?;
    let dispatcher = omega_core::dispatch::Dispatcher::new(mgr, config.clone());
    let targets = match oracle {
        Some(o) => vec![o],
        None => {
            let dead = dispatcher.dead_oracles().await;
            if dead.is_empty() {
                println!("No dead oracles — every OracleState has a live session (or none exist).");
                return Ok(());
            }
            println!("Dead oracles with persisted state: {}", dead.join(", "));
            dead
        }
    };
    for o in targets {
        match dispatcher.resurrect_oracle(&o).await {
            Ok(ResurrectOutcome::Resurrected) => println!("◆ resurrected {}", o),
            Ok(ResurrectOutcome::AlreadyAlive) => println!("• {} already alive — skipped", o),
            Ok(ResurrectOutcome::Finished) => {
                println!("• {} already finished (closeable done signal) — skipped", o)
            }
            Ok(ResurrectOutcome::NotFound) => println!("[x] no OracleState for {}", o),
            Err(e) => println!("[x] {} failed: {}", o, e),
        }
    }
    Ok(())
}

async fn cmd_timeline(oracle: &str) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    match omega_core::timeline::build(&config.state_dir, oracle)? {
        Some(tl) => {
            println!("◆ {} [{}]  phase={}", tl.oracle_name, tl.project, tl.phase);
            println!();
            for e in &tl.events {
                println!("  {}  {} {}", e.at.format("%m-%d %H:%M:%S"), e.marker, e.text);
            }
            println!("\n{} event(s)", tl.events.len());
        }
        None => {
            println!("No timeline — no OracleState for '{}'.", oracle);
            let states = omega_core::oracle_lifecycle::OracleState::read_all(&config.state_dir);
            if states.is_empty() {
                println!("(no oracles have written state yet)");
            } else {
                println!("Known oracles:");
                for s in states {
                    println!("  - {}", s.oracle_name);
                }
            }
        }
    }
    Ok(())
}

async fn cmd_doctor(fix: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let mut checks = omega_core::doctor::run_all(&config).await;
    println!("OmegaOS doctor\n");
    for c in &checks {
        println!("  {} {:16} {}", c.health.glyph(), c.name, c.detail);
    }
    // --fix: apply safe mechanical fixes, then re-run the checks so the verdict
    // below reflects the post-fix state.
    if fix {
        let actions = omega_core::doctor::auto_fix(&checks);
        println!("\n── auto-fix ──");
        if actions.is_empty() {
            println!("  (nothing auto-fixable)");
        } else {
            for a in &actions {
                println!("  [~] {}", a);
            }
            checks = omega_core::doctor::run_all(&config).await;
            println!("\n── after fix ──");
            for c in &checks {
                println!("  {} {:16} {}", c.health.glyph(), c.name, c.detail);
            }
        }
    }
    println!();
    match omega_core::doctor::overall(&checks) {
        omega_core::doctor::Health::Ok => println!("[+] all systems healthy"),
        omega_core::doctor::Health::Warn => println!("[!] healthy, with warnings above"),
        omega_core::doctor::Health::Fail => {
            println!("[x] problems detected — see [x] lines above");
            std::process::exit(1);
        }
    }
    Ok(())
}

/// `omega doctor --pre-reset` — read-only readiness report before a VPS wipe.
fn cmd_doctor_pre_reset() -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let r = omega_core::backup::pre_reset_report(&config);
    println!("OmegaOS — pre-reset readiness\n");

    if r.omega_present {
        println!(
            "  [+] ~/.omega present  ({} secret/config file(s))",
            r.secret_files.len()
        );
        for f in &r.secret_files {
            println!("        - ~/.omega/{}", f);
        }
    } else {
        println!("  [!] ~/.omega NOT found — nothing OmegaOS-owned to back up");
    }

    match r.memory_mb {
        Some(mb) => println!(
            "  [i] claude-mem memory: {} MB (~/.claude/projects) — opt-in via --include-memory",
            mb
        ),
        None => println!("  [i] claude-mem memory: none"),
    }
    match r.crontab_lines {
        Some(n) => println!("  [+] crontab: {} line(s) (captured by `omega backup`)", n),
        None => println!("  [i] crontab: none / unavailable"),
    }

    println!(
        "\n  Projects under {} — {} scanned:",
        r.projects_dir.display(),
        r.projects_scanned
    );
    if r.at_risk.is_empty() {
        println!("  [+] all scanned repos are clean and pushed — safe");
    } else {
        println!(
            "  [!] {} repo(s) with work NOT safely on a remote — push these to YOUR git first:",
            r.at_risk.len()
        );
        for repo in &r.at_risk {
            println!("        - {}  [{}]", repo.path.display(), risk_tags(repo));
        }
    }

    println!("\n  Next: `omega backup`  (writes ~/omega-backup-<ts>.tgz — scp it OFF this machine).");
    Ok(())
}

/// `omega backup` — archive the irreproducible OmegaOS state. Projects are never
/// bundled (only reported); they belong to the user's own git.
fn cmd_backup(out: Option<String>, include_memory: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let report =
        omega_core::backup::run_backup(&config, out.map(std::path::PathBuf::from), include_memory, &ts)?;

    println!("OmegaOS backup\n");
    println!("  archive : {}", report.archive.display());
    println!("  size    : {}", human_bytes(report.archive_bytes));
    println!("  included: {}", report.included.join(", "));
    if let Some(target) = &report.omega_symlink_target {
        println!(
            "  note    : ~/.omega is a symlink → {} (legacy layout) — dereferenced into the archive",
            target.display()
        );
    }
    if report.memory_included {
        println!("            (claude-mem memory included)");
    }
    println!();
    if report.at_risk.is_empty() {
        println!("  [+] all project repos clean + pushed — nothing else to save");
    } else {
        println!(
            "  [!] {} project repo(s) have work NOT on a remote — push to YOUR git (NOT bundled here):",
            report.at_risk.len()
        );
        for repo in &report.at_risk {
            println!("        - {}  [{}]", repo.path.display(), risk_tags(repo));
        }
    }
    println!("\n  Now copy it OFF this machine, e.g.:");
    println!("    scp {} you@backup-host:~/", report.archive.display());
    Ok(())
}

/// Format a byte count as B / KB / MB / GB.
fn human_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    let f = n as f64;
    if f < KB {
        format!("{} B", n)
    } else if f < KB * KB {
        format!("{:.1} KB", f / KB)
    } else if f < KB * KB * KB {
        format!("{:.1} MB", f / (KB * KB))
    } else {
        format!("{:.2} GB", f / (KB * KB * KB))
    }
}

/// Human tags for an at-risk repo (uncommitted / no-remote / unpushed).
fn risk_tags(repo: &omega_core::backup::RepoRisk) -> String {
    let mut tags = Vec::new();
    if repo.dirty {
        tags.push("uncommitted");
    }
    if repo.no_upstream {
        tags.push("no-remote");
    } else if repo.unpushed {
        tags.push("unpushed");
    }
    tags.join(", ")
}

async fn cmd_team(
    project: &str,
    count: usize,
    dir: Option<&str>,
    member_specs: &[String],
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    let work_dir = dir.unwrap_or(".").to_string();
    let session_name = format!("Team-{}", project);

    let mut members: Vec<omega_core::team::TeamMember> = member_specs
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

    // `--count N` (no explicit members) → spawn N generic workers. This is what
    // the flag always meant; it was previously parsed and ignored.
    if members.is_empty() && count > 0 {
        members = (1..=count)
            .map(|i| omega_core::team::TeamMember {
                name: format!("worker-{}", i),
                role: "worker".to_string(),
                prompt: "Implement your assigned task".to_string(),
                files_owned: Vec::new(),
            })
            .collect();
    }

    if members.is_empty() {
        anyhow::bail!("No team members. Use: omega team Project member1:prompt member2:prompt  (or --count N for N generic workers)");
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

/// Live mission progress: merge-write ~/.omega/state/oracle-<key>.progress.json,
/// preserving the bot-written chat/thread/msg fields so the Telegram bot can edit
/// the progress card in place. Oracles call this as they complete plan tasks.
fn cmd_progress(
    session: &str,
    plan: Option<&str>,
    task: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let key = session.strip_prefix("oracle-").unwrap_or(session);
    let path = config.state_dir.join(format!("oracle-{}.progress.json", key));
    // Preserve existing fields (chat/thread/msg/mission written by the bot).
    let mut obj: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let m = obj.as_object_mut().unwrap();
    // tasks: ordered [{t: title, s: status}]
    let mut tasks: Vec<serde_json::Value> = m
        .get("tasks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if let Some(p) = plan {
        // Set the whole plan; each task starts todo.
        tasks = p
            .split('|')
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| serde_json::json!({ "t": t, "s": "todo" }))
            .collect();
    }
    if let Some(t) = task {
        let st = status.unwrap_or("done");
        // Upsert by title (case-insensitive).
        if let Some(existing) = tasks
            .iter_mut()
            .find(|x| x.get("t").and_then(|v| v.as_str()).map(|s| s.eq_ignore_ascii_case(t)) == Some(true))
        {
            existing["s"] = serde_json::json!(st);
        } else {
            tasks.push(serde_json::json!({ "t": t, "s": st }));
        }
    }
    let total = tasks.len();
    let done = tasks
        .iter()
        .filter(|x| x.get("s").and_then(|v| v.as_str()) == Some("done"))
        .count();
    m.insert("tasks".into(), serde_json::json!(tasks));
    m.insert("done".into(), serde_json::json!(done));
    m.insert("total".into(), serde_json::json!(total));
    m.insert("ts".into(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
    std::fs::create_dir_all(&config.state_dir).ok();
    std::fs::write(&path, serde_json::to_string_pretty(&obj)?)?;
    println!("[+] progress {}/{} for oracle-{}", done, total, key);

    // L4 GATE RESOLUTION: the `omega done` oracle path downgrades done_clean →
    // pending while the plan is <100% (gate_pending=true) — but the oracle's own
    // final task ("report done") is by contract still unfinished at omega-done
    // time. When THIS progress tick completes the plan (100% done, no failure),
    // upgrade the stuck signal back to done_clean and auto-close the session,
    // mirroring the inline auto-close in cmd_done. Oracle sessions only.
    if session.starts_with("oracle-")
        && task.is_some()
        && status.unwrap_or("done") == "done"
        && total > 0
        && done == total
        && !tasks
            .iter()
            .any(|x| x.get("s").and_then(|v| v.as_str()) == Some("fail"))
    {
        if let Ok(Some(mut osignal)) =
            omega_core::done::OracleDoneSignal::read(&config.state_dir, session)
        {
            if osignal.status == omega_core::done::DoneStatus::Pending && osignal.gate_pending {
                osignal.status = omega_core::done::DoneStatus::DoneClean;
                osignal.pending_actions.clear();
                osignal.gate_pending = false;
                osignal.finished_at = chrono::Utc::now();
                osignal.duration_secs =
                    (osignal.finished_at - osignal.started_at).num_seconds().max(0) as u64;
                osignal.write(&config.state_dir)?;
                // The 1-min notifier cron may have already reported the transient
                // Pending state and written its per-path .notified marker — without
                // invalidating it, the corrected done_clean would NEVER be sent and
                // the operator's record would permanently say "mission incomplète".
                omega_core::done::OracleDoneSignal::invalidate_notified(
                    &config.state_dir,
                    session,
                );
                println!("[+] L4 gate satisfied - done upgraded to done_clean, auto-closing session");
                if let Ok(exe) = std::env::current_exe() {
                    // Session names are sanitized to [A-Za-z0-9._-] (no shell
                    // metachars), so this format is injection-safe.
                    let _ = std::process::Command::new("bash")
                        .arg("-c")
                        .arg(format!(
                            "sleep 3; '{}' kill '{}' >/dev/null 2>&1",
                            exe.to_string_lossy(),
                            session
                        ))
                        .spawn();
                }
            }
        }
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
        "blocked" => DoneStatus::Blocked,
        _ => anyhow::bail!("Invalid status: {}. Use: done_clean, pending, failed, blocked", status),
    };

    // Role-aware done signal. An Oracle session emits an OracleDoneSignal
    // (oracle-<key>.done.json — the schema patrol's curator/auto-resurrect and
    // the mission close-gate read), NOT a worker DoneSignal. Until now `omega
    // done` always wrote a worker signal, so an oracle calling it produced a
    // worker-<oracle>.done.json that no oracle-side consumer ever read, and
    // OracleDoneSignal::write had zero callers. The key is the session name
    // minus its single "oracle-" prefix (index retained), matching
    // OracleDoneSignal's read/write normalization. Workers fall through below.
    if omega_core::session::OmegaSession::classify(session).role
        == omega_core::session::SessionRole::Oracle
    {
        let key = session.strip_prefix("oracle-").unwrap_or(session);
        // The PROJECT is the oracle key minus its numeric session index
        // (oracle-<project>-<n>): e.g. "dentistrygpt-8" → "dentistrygpt". Keeping the
        // "-8" suffix broke Telegram topic routing (the report fell back to the main
        // chat instead of the project's topic) and mislabelled the report.
        let project = key
            .rsplit_once('-')
            .filter(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .map(|(p, _)| p)
            .unwrap_or(key);
        // L4 COMPLETENESS GATE: an oracle cannot claim done_clean while its plan is
        // unfinished. If the live progress (oracle-<key>.progress.json) shows tasks not
        // all done (or any failed), downgrade done_clean → pending and surface the
        // remaining tasks — the report then honestly shows incomplete (no 92%-is-done).
        let mut final_status = done_status;
        let mut gate_pending: Vec<String> = Vec::new();
        if final_status == omega_core::done::DoneStatus::DoneClean {
            let pp = config.state_dir.join(format!("oracle-{}.progress.json", key));
            if let Ok(pj) = std::fs::read_to_string(&pp)
                .ok()
                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                .ok_or(())
            {
                let total = pj.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                let done = pj.get("done").and_then(|v| v.as_u64()).unwrap_or(0);
                let tasks = pj.get("tasks").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let failed: Vec<String> = tasks.iter()
                    .filter(|t| t.get("s").and_then(|v| v.as_str()) == Some("fail"))
                    .filter_map(|t| t.get("t").and_then(|v| v.as_str()).map(|s| format!("échec: {}", s)))
                    .collect();
                let unfinished: Vec<String> = tasks.iter()
                    .filter(|t| matches!(t.get("s").and_then(|v| v.as_str()), Some("todo") | Some("doing")))
                    .filter_map(|t| t.get("t").and_then(|v| v.as_str()).map(|s| format!("non fait: {}", s)))
                    .collect();
                if total > 0 && (done < total || !failed.is_empty()) {
                    final_status = omega_core::done::DoneStatus::Pending;
                    gate_pending.extend(failed);
                    gate_pending.extend(unfinished);
                    if gate_pending.is_empty() {
                        gate_pending.push(format!("plan {}/{} — pas 100% (L4)", done, total));
                    }
                }
            }
        }
        let mut osignal =
            omega_core::done::OracleDoneSignal::new(key, project, final_status, summary);
        osignal.summary = summary.to_string();
        // Mark the L4-gate downgrade so `omega progress` / patrol can upgrade the
        // signal back to done_clean once the plan hits 100% (the oracle's own
        // final "report" task is unfinished at omega-done time by contract).
        osignal.gate_pending =
            done_status == omega_core::done::DoneStatus::DoneClean
                && final_status == omega_core::done::DoneStatus::Pending;
        osignal.pending_actions = gate_pending;
        if let Some(c) = commit.filter(|c| !c.is_empty()) {
            osignal.ship = Some(omega_core::done::OracleShipResult {
                requested: false,
                result: "committed".to_string(),
                commit: Some(c.to_string()),
                push_url: None,
                deploy_url: None,
                deploy_status: None,
            });
        }
        osignal.write(&config.state_dir)?;
        // Release the scope claim on a clean close, mirroring the worker path.
        if osignal.is_closeable() {
            let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, session);
        }
        println!("[+] Oracle done signal written: oracle-{}.done.json", key);
        // AUDIT JOURNAL: append the mission outcome to the per-project audit log,
        // organized under ~/.omega/audit/<project>/audit.jsonl (governance trail — who
        // did what, when, with what result). Best-effort, never blocks the done signal.
        {
            let dir = config.state_dir.parent().map(|p| p.join("audit").join(project));
            if let Some(dir) = dir {
                let _ = std::fs::create_dir_all(&dir);
                let line = format!(
                    "{{\"ts\":\"{}\",\"event\":\"done\",\"oracle\":\"{}\",\"status\":\"{:?}\",\"summary\":{}}}\n",
                    chrono::Utc::now().to_rfc3339(),
                    key,
                    final_status,
                    serde_json::to_string(summary).unwrap_or_else(|_| "\"\"".into()),
                );
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("audit.jsonl")) {
                    let _ = f.write_all(line.as_bytes());
                }
            }
        }
        // AUTO-CLOSE: writing the report IS the close condition (operator contract).
        // On a clean done, close the oracle's own session — detached + a short delay so
        // THIS `omega done` returns cleanly and the done.json is on disk for the
        // notifier cron before the pane is killed. (Non-clean statuses stay open so the
        // operator can inspect a failed/blocked/pending oracle.)
        if final_status == omega_core::done::DoneStatus::DoneClean {
            if let Ok(exe) = std::env::current_exe() {
                // Session names are sanitized to [A-Za-z0-9._-] (no shell metachars),
                // so this format is injection-safe.
                let _ = std::process::Command::new("bash")
                    .arg("-c")
                    .arg(format!(
                        "sleep 3; '{}' kill '{}' >/dev/null 2>&1",
                        exe.to_string_lossy(),
                        session
                    ))
                    .spawn();
            }
        }
        return Ok(());
    }

    let mut signal = DoneSignal::new(session, done_status, summary);
    signal.commit = commit.map(|s| s.to_string());

    // Opus 4.8 ground-truth substrate: a worker's narration is inadmissible
    // as proof. Auto-capture the REAL git state of the cwd (the worker runs
    // `omega done` from its work_dir) so a legitimate done_clean carries a
    // verifiable artifact + a non-self-report corroboration source. The
    // patrol gate (verify_done_against_repo) then catches fabricated claims
    // without false-positiving honest work.
    use omega_core::done::{CorroborationSource, DoneArtifact};
    signal.corroboration.push(CorroborationSource::WorkerSelfReport);
    if let Ok(cwd) = std::env::current_dir() {
        let head = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&cwd)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(head_sha) = head {
            let branch = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&cwd)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty() && s != "HEAD");
            signal.artifacts.push(DoneArtifact::GitSha {
                sha: head_sha.clone(),
                branch: branch.clone(),
            });
            signal.corroboration.push(CorroborationSource::FilesystemCheck);
            // If the worker named a specific commit that is NOT the current
            // HEAD, record it as its own claim so the gate verifies it too.
            if let Some(c) = commit {
                if !c.is_empty() && c != head_sha && !head_sha.starts_with(c) {
                    signal.artifacts.push(DoneArtifact::GitSha {
                        sha: c.to_string(),
                        branch,
                    });
                }
            }
        }
    }
    signal.write(&config.state_dir)?;

    // Release scope claim on done_clean. Gate ONLY on status, not is_complete():
    // is_complete() also requires todos_completed >= todos_total, which a worker
    // that never tracked todos trivially satisfies (0 >= 0) — that would release
    // the scope on ANY done_clean signal even when the work isn't really finished.
    // status == DoneClean is the authoritative completion signal here.
    if signal.status == omega_core::done::DoneStatus::DoneClean {
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, session);
    }

    println!("[+] Done signal written for: {}", session);
    Ok(())
}

async fn cmd_inbox(oracle: &str, action: &str) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let inbox = omega_core::inbox::Inbox::for_oracle(&config.state_dir, oracle);

    match action {
        "peek" => {
            let events = inbox.peek()?;
            if events.is_empty() {
                println!("No events in inbox for {}", oracle);
            } else {
                for event in &events {
                    println!(
                        "[{}] {:?} → {}",
                        event.timestamp.format("%H:%M:%S"),
                        event.event_type,
                        event.payload
                    );
                }
            }
        }
        "drain" => {
            let events = inbox.drain()?;
            if events.is_empty() {
                println!("No events to drain for {}", oracle);
            } else {
                println!("Drained {} events:", events.len());
                for event in &events {
                    println!(
                        "  [{:?}] {}",
                        event.event_type,
                        event.payload
                    );
                }
            }
        }
        "count" => {
            let count = inbox.count()?;
            println!("{}", count);
        }
        _ => anyhow::bail!("Invalid action: {}. Use: peek, drain, count", action),
    }
    Ok(())
}

async fn cmd_ship(project: &str, message: &str, unfreeze: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;

    let project_dir = match config.find_project(project) {
        Some(pc) => pc.path.clone(),
        None => std::env::current_dir()?,
    };

    let ship_config = omega_core::ship::ShipConfig::default();
    let pipeline = omega_core::ship::ShipPipeline::new(
        project_dir,
        config.state_dir.clone(),
        ship_config,
    );

    if unfreeze {
        pipeline.unfreeze(project)?;
        println!("[+] Ship pipeline unfrozen for {}", project);
        return Ok(());
    }

    if pipeline.is_frozen(project) {
        println!("[x] Ship pipeline is FROZEN for {}. Use --unfreeze to clear.", project);
        return Ok(());
    }

    println!("◆ Ship pipeline starting for {}...", project);
    let result = pipeline.execute(project, message, &Vec::<String>::new()).await;

    for step in &result.steps_completed {
        let icon = if step.passed { "[+]" } else { "[x]" };
        println!("  {} {} ({}ms)", icon, step.name, step.duration_ms);
    }

    match result.result {
        omega_core::ship::ShipOutcome::Ok => {
            println!("◆ Ship complete!");
            if let Some(ref commit) = result.commit {
                println!("  Commit: {}", commit);
            }
            if let Some(ref url) = result.deploy_url {
                println!("  Deploy: {}", url);
            }
        }
        omega_core::ship::ShipOutcome::Failed => {
            println!("[x] Ship failed: {}", result.error.as_deref().unwrap_or("unknown"));
        }
        omega_core::ship::ShipOutcome::Frozen => {
            println!("[x] Ship pipeline is frozen — resolve the issue first");
        }
        omega_core::ship::ShipOutcome::Skipped => {
            println!("- Ship skipped");
        }
    }

    pipeline.write_result(project, &result)?;
    Ok(())
}

/// Interactive AISB Master chat REPL. Runs in the aisb-master pane.
/// Each typed line is appended to the local inbox the running Telegram
/// bridge watches; the bridge processes it as a synthetic Telegram
/// message (same brain), so the response lands in Telegram AND in the
/// conversation log we tail here. Turn-based: type → wait for response →
/// type again.
async fn cmd_aisb_chat() -> Result<()> {
    use std::io::{BufRead, Write};
    let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let log = home.join(".omega/state/aisb-conversation.log");
    let inbox = home.join(".omega/state/aisb-local-inbox.jsonl");
    if let Some(p) = inbox.parent() {
        let _ = std::fs::create_dir_all(p);
    }

    // Header + replay the existing conversation.
    print!("\x1b[2J\x1b[H"); // clear
    println!("\x1b[1;36m  Ω  AISB Master — chat (local input → Telegram)\x1b[0m");
    println!("  Type a message; it goes to AISB exactly like a Telegram message.");
    println!("  The reply appears here AND in your Telegram chat. Ctrl-D to exit.\n");
    if let Ok(existing) = std::fs::read_to_string(&log) {
        let tail: Vec<&str> = existing.lines().rev().take(40).collect();
        for line in tail.into_iter().rev() {
            println!("{}", line);
        }
    }

    let stdin = std::io::stdin();
    loop {
        print!("\n\x1b[1;36mYou ▶ \x1b[0m");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line)?;
        if n == 0 {
            println!("\n(bye)");
            break; // EOF (Ctrl-D)
        }
        let msg = line.trim();
        if msg.is_empty() {
            continue;
        }
        if msg == "/quit" || msg == "/exit" {
            break;
        }

        // Record current log size, then inject the message into the inbox.
        let before_len = std::fs::metadata(&log).map(|m| m.len()).unwrap_or(0);
        let entry = serde_json::json!({
            "text": msg,
            "ts": chrono::Utc::now().to_rfc3339(),
        });
        {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&inbox)?;
            writeln!(f, "{}", entry)?;
        }

        // Wait (up to 90s) for the bridge to append the AISB response to
        // the conversation log, then print the delta.
        print!("\x1b[90m  … thinking …\x1b[0m");
        let _ = std::io::stdout().flush();
        let mut shown = false;
        for _ in 0..180 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let now_len = std::fs::metadata(&log).map(|m| m.len()).unwrap_or(0);
            if now_len > before_len {
                // Print the new tail (the bridge wrote the You:/AISB: block).
                if let Ok(content) = std::fs::read_to_string(&log) {
                    let delta = &content[before_len.min(content.len() as u64) as usize..];
                    print!("\r\x1b[K{}", delta); // clear "thinking" line
                    let _ = std::io::stdout().flush();
                }
                shown = true;
                break;
            }
        }
        if !shown {
            println!("\r\x1b[K\x1b[33m  (no response within 90s — check the bridge)\x1b[0m");
        }
    }
    Ok(())
}

async fn cmd_patrol(interval: u64, once: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mut patrol = omega_core::patrol::Patrol::new(config);

    if once {
        let report = patrol.run_once().await?;
        println!("Sessions: {} (◆{} ●{})", report.total_sessions, report.oracles, report.workers);
        if !report.done_workers.is_empty() {
            println!("Done workers: {}", report.done_workers.join(", "));
        }
        if !report.stalled_workers.is_empty() {
            println!("[!] Stalled: {}", report.stalled_workers.join(", "));
        }
        if !report.blocked_workers.is_empty() {
            println!("⊘ Blocked: {}", report.blocked_workers.join(", "));
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

    // Prefer the ACTUAL gate result if the oracle has been graded — that is what
    // "check the gate" means. Falls back to showing the rubric (the criteria the
    // gate will grade against) when no result exists yet.
    let result_path = config.state_dir.join(format!("{}.gate-result.json", oracle));
    if result_path.exists() {
        let content = std::fs::read_to_string(&result_path)?;
        let r: omega_core::gate::GateResult = serde_json::from_str(&content)?;
        let mark = |b: bool| if b { "PASS" } else { "FAIL" };
        println!("Gate result for {} — {} ({:.1}/100)", oracle, mark(r.overall_pass), r.score);
        println!("  rubric={}  consensus={}  adversarial={}  regression={}", mark(r.rubric_pass), mark(r.consensus_pass), mark(r.adversarial_pass), mark(r.regression_pass));
        println!("  audit={}  token_budget={}  citation={}", mark(r.audit_pass), mark(r.token_budget_pass), mark(r.citation_pass));
        return Ok(());
    }

    match omega_core::gate::Rubric::read(&config.state_dir, oracle)? {
        Some(rubric) => {
            println!("No gate result yet for {} — showing the rubric it will grade against.", oracle);
            println!("Mission: {}", rubric.mission);
            println!("Criteria:");
            for c in &rubric.criteria {
                println!("  [{}] {} (weight: {:.1})", c.id, c.description, c.weight);
            }
        }
        None => {
            println!("No gate result or rubric found for {}. Create a rubric with: omega gate {} --mission \"...\"", oracle, oracle);
        }
    }
    Ok(())
}

async fn cmd_scope(session: &str, files: &[String]) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let conflicts = omega_core::scope::check_conflicts(&config.state_dir, session, files)?;

    if conflicts.is_empty() {
        println!("[+] No scope conflicts for {}", session);
    } else {
        println!("[x] Scope conflicts detected:");
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
        println!("  -{}", r);
    }
    if !decision.audit_skills.is_empty() {
        println!();
        println!("Audit skills detected:");
        for audit in &decision.audit_skills {
            println!("  /{} (trigger: '{}')", audit.skill, audit.trigger);
        }
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

async fn cmd_pdf(
    template: &str,
    data: Option<&str>,
    demo: bool,
    theme: &str,
    out: &str,
    send_telegram: bool,
    caption: Option<&str>,
) -> Result<()> {
    // Resolve the pdfgen directory — bundled with OmegaOS
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let pdfgen_dir = find_pdfgen_dir(exe_dir.as_deref())?;

    // Auto-install deps on first use
    let nm = pdfgen_dir.join("node_modules");
    if !nm.exists() {
        println!("Installing PDF generator dependencies (first time)…");
        let status = std::process::Command::new("npm")
            .args(["install", "--silent"])
            .current_dir(&pdfgen_dir)
            .status()
            .context("npm install for pdfgen failed")?;
        if !status.success() {
            anyhow::bail!("npm install failed in {}", pdfgen_dir.display());
        }
    }

    // Build the pdfgen CLI args
    let mut args = vec![
        "tsx".to_string(),
        "bin/pdfgen.ts".to_string(),
        format!("--template={}", template),
        format!("--theme={}", theme),
        format!("--out={}", out),
    ];
    if demo {
        args.push("--demo".to_string());
    }
    if let Some(d) = data {
        args.push(format!("--data={}", d));
    }

    println!("Generating PDF → {}", out);
    let status = std::process::Command::new("npx")
        .args(&args)
        .current_dir(&pdfgen_dir)
        .status()
        .context("pdfgen execution failed")?;

    if !status.success() {
        anyhow::bail!("PDF generation failed");
    }

    let pdf_path = std::path::Path::new(out);
    if !pdf_path.exists() {
        anyhow::bail!("PDF not found at {}", out);
    }

    let size = std::fs::metadata(pdf_path)?.len();
    println!("[+] PDF generated: {} ({:.1} KB)", out, size as f64 / 1024.0);

    // Send via Telegram if requested
    if send_telegram {
        send_pdf_telegram(out, caption).await?;
    }

    Ok(())
}

fn find_pdfgen_dir(exe_dir: Option<&std::path::Path>) -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    // 1. ~/.omega/skills/pdfgen (canonical installed location)
    let skills_dir = home.join(".omega/skills/pdfgen");
    if skills_dir.join("bin/pdfgen.ts").exists() {
        return Ok(skills_dir);
    }
    // 2. ~/.omega/pdfgen (legacy installed location)
    let user_dir = home.join(".omega/pdfgen");
    if user_dir.join("bin/pdfgen.ts").exists() {
        return Ok(user_dir);
    }
    // 2. Relative to the OmegaOS repo (dev mode)
    let cwd = std::path::PathBuf::from("tools/pdfgen");
    if cwd.join("bin/pdfgen.ts").exists() {
        return Ok(cwd);
    }
    // 3. Relative to binary
    if let Some(dir) = exe_dir {
        let rel = dir.join("../tools/pdfgen");
        if rel.join("bin/pdfgen.ts").exists() {
            return Ok(rel);
        }
    }
    anyhow::bail!(
        "PDF generator not found. Expected at tools/pdfgen/ or ~/.omega/pdfgen/.\n\
         Run `omega init` to set up, or copy the pdfgen/ directory manually."
    )
}

async fn send_pdf_telegram(pdf_path: &str, caption: Option<&str>) -> Result<()> {
    use omega_core::monitor::OmegaTelegramConfig;

    let cfg = OmegaTelegramConfig::read()
        .ok_or_else(|| anyhow::anyhow!("Telegram not configured. Run: omega telegram setup …"))?;

    let chat_id = if !cfg.allow_user_ids.is_empty() {
        cfg.allow_user_ids[0]
    } else {
        cfg.chat_id
    };

    let url = format!(
        "https://api.telegram.org/bot{}/sendDocument",
        cfg.bot_token
    );

    let file_bytes = tokio::fs::read(pdf_path).await.context("reading PDF")?;
    let filename = std::path::Path::new(pdf_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(filename)
        .mime_str("application/pdf")?;

    let mut form = reqwest::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
        .part("document", part);

    if let Some(cap) = caption {
        form = form.text("caption", cap.to_string())
            .text("parse_mode", "HTML".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client.post(&url).multipart(form).send().await.context("sendDocument")?;
    if resp.status().is_success() {
        println!("[+] PDF sent via Telegram");
    } else {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Telegram sendDocument failed: {}", body);
    }

    Ok(())
}

fn cmd_rules(action: RulesAction) -> Result<()> {
    use omega_core::rules::{self, RuleKind};
    match action {
        RulesAction::Context { scope } => {
            let s = match scope.to_lowercase().as_str() {
                "master" | "atlas" | "director" => rules::RuleScope::Master,
                "worker" => rules::RuleScope::Worker,
                _ => rules::RuleScope::Oracle,
            };
            print!("{}", rules::agent_context_block(s));
            return Ok(());
        }
        RulesAction::List => {
            let laws = rules::laws();
            let ops = rules::operational_rules();
            println!("THE LAWS (inviolable — bind every agent, override every rule)\n");
            for r in &laws {
                println!("  {:16} {}", r.id, r.title);
            }
            println!("\nOPERATIONAL RULES ({})\n", ops.len());
            // Group by a FIXED category order so each header prints exactly
            // once, regardless of the registry's declaration order.
            use rules::RuleCategory::*;
            for cat in [Universal, QualityGate, Orchestration, Reporting, Safety] {
                let in_cat: Vec<_> = ops.iter().filter(|r| r.category == cat).collect();
                if in_cat.is_empty() {
                    continue;
                }
                println!("─── {:?} ───", cat);
                for r in in_cat {
                    println!("  {:16} {}", r.id, r.title);
                }
            }
            println!("\nRules dir: ~/.omega/rules/");
            println!("Export:    omega rules export");
        }
        RulesAction::Export => {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            let rules_dir = home.join(".omega/rules");
            std::fs::create_dir_all(&rules_dir)?;

            // Idempotent: clear stale exports first so a re-export always
            // mirrors the current registry exactly (no lingering old-id files
            // when rules are renamed or removed).
            if let Ok(entries) = std::fs::read_dir(&rules_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().and_then(|x| x.to_str()) == Some("md") {
                        let _ = std::fs::remove_file(&p);
                    }
                }
            }

            let all = rules::all_rules();
            for r in &all {
                let slug = r.title.to_lowercase()
                    .chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '-' })
                    .collect::<String>();
                let slug = slug.trim_matches('-').replace("--", "-");
                let fname = format!("{}-{}.md", r.id, &slug[..slug.len().min(40)]);
                let kind_label = match r.kind {
                    RuleKind::Law => "Law",
                    RuleKind::Rule => "Rule",
                };
                let content = format!(
                    "# {} — {}\n\n**Kind:** {}\n**Category:** {:?}\n**Added:** {}\n\n## Rule\n\n{}\n\n## Origin\n\n{}\n",
                    r.id, r.title, kind_label, r.category, r.added_at, r.description, r.reason
                );
                std::fs::write(rules_dir.join(&fname), &content)?;
                println!("  [+] {}", fname);
            }
            println!("\n{} rules exported to {}", all.len(), rules_dir.display());
        }
    }
    Ok(())
}

fn cmd_audit(action: AuditAction) -> Result<()> {
    use omega_core::audit;
    match action {
        AuditAction::List => {
            let all = audit::all_audits();
            println!("Quality Arsenal — {} forensic audits\n", all.len());
            println!(
                "  {:<18} {:<24} {:<8} {:<6} {:<6} {}",
                "ID", "NAME", "DOMAIN", "PHASES", "MAX", "READ-ONLY"
            );
            println!("  {}", "─".repeat(80));
            for a in &all {
                println!(
                    "  {:<18} {:<24} {:<8} {:<6} {:<6} {}",
                    a.id,
                    a.name,
                    a.domain.label(),
                    a.phases,
                    format!("/{}", a.max_score),
                    if a.read_only { "yes" } else { "" }
                );
            }
            println!("\nUsage:");
            println!("  omega audit select \"fix the auth flow\"");
            println!("  omega audit run codeaudit --dir ~/project");
        }
        AuditAction::Select { mission } => {
            let selected = audit::select_audits(&mission, &[]);
            println!("Mission: {}\n", mission);
            if selected.is_empty() {
                println!("No audits matched.");
            } else {
                println!("Selected {} audit(s):\n", selected.len());
                for id in &selected {
                    if let Some(a) = audit::find_audit(id) {
                        println!(
                            "  /{:<18} {} — {}",
                            a.id, a.domain.label(), a.description
                        );
                    }
                }
            }
        }
        AuditAction::Results { oracle } => {
            let config = OmegaConfig::load().unwrap_or_default();
            let path = config.state_dir.join(format!("{}.audit-report.json", oracle));
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let report: audit::AuditReport = serde_json::from_str(&content)?;
                println!("Audit Report for: {}\n", report.mission_id);
                println!(
                    "  Overall: {:.1}/100 ({:?})\n",
                    report.overall_score, report.overall_verdict
                );
                for r in &report.audits {
                    println!(
                        "  {:<18} {:.1}/100  {:?}  (raw {:.0}/{})",
                        r.audit_id, r.normalized_score, r.verdict, r.raw_score, r.max_score
                    );
                }
            } else {
                println!("No audit results found for {}.", oracle);
                println!("Results are stored at: {}", path.display());
            }
        }
        AuditAction::Run { audit_id, dir } => {
            let skill = audit::find_audit(&audit_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown audit: {}. Run `omega audit list` to see available audits.",
                    audit_id
                )
            })?;
            println!("◆ Audit: {} ({})", skill.name, skill.domain.label());
            println!("  Phases: {}, Max score: /{}", skill.phases, skill.max_score);
            println!("  Skill:  {}", skill.skill_path);
            println!("  Dir:    {}", dir);
            if skill.read_only {
                println!("  Mode:   READ-ONLY (proposes, never edits)");
            }
            println!("\nTo dispatch as a worker session:");
            println!(
                "  omega spawn-worker {0} \"/{0} --dir={1}\" --dir {1}",
                audit_id, dir
            );
        }
    }
    Ok(())
}

fn cmd_sync() -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let omega_dir = home.join(".omega");

    // Ensure master dirs exist
    for sub in &["rules", "agents", "agents/aisb", "skills", "hooks", "plugins", "docs", "projects", "state", "logs"] {
        std::fs::create_dir_all(omega_dir.join(sub))?;
    }

    // Export rules if not already present
    let rules_dir = omega_dir.join("rules");
    if std::fs::read_dir(&rules_dir)?.count() == 0 {
        println!("Exporting rules...");
        cmd_rules(RulesAction::Export)?;
    }

    // Copy OMEGA.md to ~/.omega/
    let omega_md_src = std::path::Path::new("OMEGA.md");
    let omega_md_dst = omega_dir.join("OMEGA.md");
    if omega_md_src.exists() {
        std::fs::copy(omega_md_src, &omega_md_dst)?;
        println!("[+] OMEGA.md → {}", omega_md_dst.display());
    }

    // Copy agents from repo if available
    let agents_src = std::path::Path::new("agents");
    if agents_src.exists() {
        let agents_dst = omega_dir.join("agents");
        std::fs::create_dir_all(agents_dst.join("aisb"))?;
        for entry in std::fs::read_dir(agents_src).into_iter().flatten() {
            let entry = entry?;
            let dst = agents_dst.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                // aisb/ subdirectory
                for sub in std::fs::read_dir(entry.path()).into_iter().flatten() {
                    let sub = sub?;
                    if sub.file_name().to_string_lossy().ends_with(".md") {
                        std::fs::copy(sub.path(), agents_dst.join("aisb").join(sub.file_name()))?;
                    }
                }
            } else if entry.file_name().to_string_lossy().ends_with(".md") {
                std::fs::copy(entry.path(), &dst)?;
            }
        }
        println!("[+] Agents synced to {}", agents_dst.display());
    }

    // Copy skills from repo if available (pdfgen etc.)
    let skills_src = std::path::Path::new("tools/pdfgen");
    let skills_dst = omega_dir.join("skills/pdfgen");
    if skills_src.exists() && !skills_dst.join("bin/pdfgen.ts").exists() {
        std::fs::create_dir_all(&skills_dst)?;
        let status = std::process::Command::new("rsync")
            .args(["-a", "--exclude=node_modules", "--exclude=.next", "--exclude=output"])
            .arg(format!("{}/", skills_src.display()))
            .arg(format!("{}/", skills_dst.display()))
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("[+] PDF generator synced to {}", skills_dst.display());
            }
        }
    }

    // ── Claude Code integration ──
    let claude_dir = home.join(".claude");
    if claude_dir.exists() {
        // Rules: symlink each omega rule with omega- prefix
        let claude_rules = claude_dir.join("rules");
        std::fs::create_dir_all(&claude_rules)?;
        for entry in std::fs::read_dir(&rules_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.ends_with(".md") { continue; }
            let link = claude_rules.join(format!("omega-{}", name_str));
            if !link.exists() {
                #[cfg(unix)]
                std::os::unix::fs::symlink(entry.path(), &link)?;
                println!("  [+] Claude rule: {}", name_str);
            }
        }

        // Skills: symlink each omega skill directory
        let skills_dir = omega_dir.join("skills");
        let claude_skills = claude_dir.join("skills");
        std::fs::create_dir_all(&claude_skills)?;
        if skills_dir.exists() {
            for entry in std::fs::read_dir(&skills_dir)? {
                let entry = entry?;
                if !entry.file_type()?.is_dir() { continue; }
                let name = entry.file_name();
                let link = claude_skills.join(&name);
                if !link.exists() {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(entry.path(), &link)?;
                    println!("  [+] Claude skill: {}", name.to_string_lossy());
                }
            }
        }
        println!("[+] Claude Code synced (rules + skills)");
    }

    // ── Gemini CLI integration ──
    let gemini_dir = home.join(".gemini");
    if gemini_dir.exists() {
        let gemini_md = gemini_dir.join("GEMINI.md");
        let omega_ref = "\n# OmegaOS\n@import ~/.omega/OMEGA.md\n";
        if gemini_md.exists() {
            let content = std::fs::read_to_string(&gemini_md)?;
            if !content.contains("OmegaOS") {
                std::fs::write(&gemini_md, format!("{}{}", content, omega_ref))?;
                println!("[+] Gemini: appended OmegaOS reference to GEMINI.md");
            }
        } else {
            std::fs::write(&gemini_md, omega_ref)?;
            println!("[+] Gemini: created GEMINI.md → OmegaOS");
        }
    }

    // ── Codex integration ──
    let codex_dir = home.join(".codex");
    if codex_dir.exists() || std::fs::create_dir_all(&codex_dir).is_ok() {
        let agents_md = codex_dir.join("AGENTS.md");
        if !agents_md.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&omega_md_dst, &agents_md)?;
            println!("[+] Codex: AGENTS.md → OMEGA.md");
        }
    }

    println!("\n[+] OmegaOS sync complete — all LLMs reference ~/.omega/");
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
