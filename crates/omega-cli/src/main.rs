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

    /// Diagnose mouse support END-TO-END from the terminal you run it in:
    /// enables mouse reporting, listens 10s, and tells you whether your
    /// terminal actually sends wheel/click events (or arrow-key translations,
    /// or nothing — e.g. over mosh, which never forwards the mouse handshake).
    #[command(name = "mouse-test")]
    MouseTest,

    /// Auto-discover projects on this machine (smart whole-$HOME walk,
    /// best-scored first: markers + recent activity)
    Projects {
        /// Machine-readable output (the Telegram bot's discovery feed)
        #[arg(long)]
        json: bool,
    },

    /// Marketing — list marketing-enabled projects and their status.
    /// A project is marketing-enabled when it has a `marketing/` directory.
    #[command(subcommand)]
    Marketing(MarketingAction),

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

    /// Show billing / accounts / bot status, or WATCH a live rmux session
    ///
    /// omega monitor                     billing, accounts and bot status (unchanged)
    /// omega monitor oracle-Kommu        watch a session on this box
    /// omega monitor matrix:MAC-STREAM   watch a session on the `matrix` ssh host
    /// omega monitor list                what is watchable, and what is already watched
    /// omega monitor classify < pane     classify ONE captured pane and print the state
    ///
    /// With no target this is the one-shot billing view it has always been,
    /// also visible in the TUI Monitor tab. With a target it runs the SESSION
    /// monitor, which is a different feature that happens to share the word.
    ///
    /// A cheap watcher polls the pane and sorts what it sees into FOUR states,
    /// because a stop has more than one shape and each shape needs a different
    /// answer. QUESTION needs judgement, so it goes to a human and is never
    /// auto-answered. STALLED is mechanical, so it gets a mechanical nudge.
    /// BLOCKED is never nudged: a nudge with nothing runnable is manufactured
    /// thrash, not persistence. WORKING says nothing, which is the correct
    /// output most of the time.
    ///
    /// --work-probe is what splits STALLED from BLOCKED: a shell command that
    /// prints how much work is LEFT, counting work in ANY form (a step
    /// awaiting a sign-off is work, and counting only runnable steps once
    /// reported BLOCKED on a build that was merely waiting for a signature).
    /// Anything unreadable counts as work, so a broken probe never stalls a
    /// build silently.
    ///
    /// --progress-probe bounds the ABSENCE of progress rather than the work:
    /// it prints a monotonic integer, and every advance resets the nudge
    /// budget. A flat cap stops a healthy long run for no reason.
    ///
    /// Hosts are ALIASES from ~/.ssh/config, and the target is preflighted
    /// before anything is created. Detach with Ctrl-b d; it keeps running.
    /// Kill it with `omega kill monitor-<name>`.
    #[command(verbatim_doc_comment)]
    Monitor {
        /// Omitted shows billing/accounts. Otherwise `<session>`, `<host>:<session>`, `list`, or `classify`
        target: Option<String>,
        /// Create the monitor but do not attach to it
        #[arg(short, long)]
        detach: bool,
        /// Seconds between watcher polls
        #[arg(long, default_value_t = omega_core::session_monitor::DEFAULT_INTERVAL_SECS)]
        interval: u32,
        /// Scrollback lines captured per poll
        #[arg(long, default_value_t = omega_core::session_monitor::DEFAULT_LINES)]
        lines: u32,
        /// Shell command printing how much work is LEFT (splits STALLED from BLOCKED)
        #[arg(long)]
        work_probe: Option<String>,
        /// `classify` only: the work count ALREADY measured by the caller.
        ///
        /// The watcher loop runs the work probe once per poll for its own
        /// logic and then asks for a classification, so re-running the probe
        /// in here would run it TWICE per poll. That is not merely wasteful:
        /// a probe is usually an ssh round trip, and the two runs can disagree
        /// across the gap, which would classify against a count the caller
        /// never saw. Takes precedence over --work-probe when both are given.
        #[arg(long)]
        work: Option<i64>,
        /// Shell command printing a monotonic progress metric (an advance resets the nudge budget)
        #[arg(long)]
        progress_probe: Option<String>,
        /// Watcher polls between two runs of the deep audit team
        #[arg(long, default_value_t = omega_core::session_monitor::DEFAULT_AUDIT_EVERY)]
        audit_every: u32,
    },

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

    /// Compile and validate the canonical OmegaOS skill catalog
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },

    /// Manage Quality Arsenal audits (23 Gestalt-Popper forensic audits)
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },

    /// Sync OmegaOS config into all LLM config directories (symlinks)
    Sync,

    /// Update OmegaOS to the latest version (fetch + fast-forward + reinstall).
    /// Your ~/.omega state — secrets, projects, Telegram config — is preserved.
    Update {
        /// Report what an update WOULD do, then exit. Changes nothing.
        #[arg(long)]
        check: bool,
        /// Unattended daily mode (what the cron runs). Checks, then installs
        /// what is available — but never over local changes, never while an
        /// agent is mid-turn, and never a 4th time on a commit that keeps
        /// failing. Honors the `auto_update` config (apply | check | off).
        #[arg(long)]
        auto: bool,
        /// The OmegaOS checkout to update. Defaults to $OMEGA_SRC, the current
        /// directory, then the usual install locations.
        #[arg(long)]
        dir: Option<String>,
        /// Record the checkout's HEAD as the commit now installed, and exit.
        /// `install.sh` calls this at the end of a successful install so a
        /// hand-run installer leaves the same honest provenance the cron does.
        /// Without it, `auto-update.json` keeps naming whatever the cron last
        /// installed, and the staleness check has nothing true to compare to.
        #[arg(long)]
        record_installed: bool,
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

    /// Kill a session. On an ORACLE this is a mission CLOSURE, not a pane kill:
    /// its live workers are cascaded, EVERY affected scope claim is released,
    /// and the git worktree of a `--worktree` worker is unregistered when it
    /// holds no unsaved work. Killing an already-dead session is a no-op that
    /// exits 0, so the command is safe to run twice.
    Kill {
        /// Session name
        name: String,
        /// Close an oracle even though some of its workers are still RUNNING
        /// (they are killed too). Without it a running worker REFUSES the
        /// closure, which is the safe default: an oracle killed out from under
        /// a live worker leaves that worker holding a scope claim forever.
        #[arg(long)]
        force: bool,
    },

    /// Reconcile this install after an update: re-apply everything mechanical,
    /// then report what needs a human.
    ///
    /// An OmegaOS update installs a binary. It does not, by itself, bring the
    /// things AROUND the binary back in line: the exported doctrine on disk,
    /// the record of what was installed, worker sessions whose done signal was
    /// never swept, projects that moved or lost their canon docs, sessions
    /// still running on the doctrine of the previous binary. Each of those
    /// drifts quietly and is only ever noticed by accident.
    ///
    /// The split is deliberate. Anything DETERMINISTIC is fixed in place, with
    /// no opinion required. Anything needing judgement — a project doc that no
    /// longer matches its repo, a live session mid-turn on old doctrine — is
    /// REPORTED and left alone, because a reconciler that rewrites your project
    /// docs or restarts your working sessions unattended is a worse problem
    /// than the drift it set out to fix.
    ///
    /// Exits non-zero when anything needs a human, so a cron can alert on it.
    Reconcile {
        /// Report the drift and change nothing at all.
        #[arg(long)]
        report_only: bool,
    },

    /// Run a mission graph: drive `graph_executor::advance` to a terminal
    /// outcome, with the risk gate in front of every dispatch.
    ///
    /// The decision core never runs anything itself — that is what makes a run
    /// replayable off a persisted state — so this is the driver it was written
    /// for. A node declares what to run in a `command` field; the driver
    /// executes it, turns its exit status into a `NodeReport`, and hands the
    /// batch back to `advance`.
    ///
    /// Deliberately NOT wired into oracle dispatch. That path runs on other
    /// people's installs and already has its own DAG (`mission::PlanContract`);
    /// this is additive and drives the graphs you hand it.
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    /// Reap finished workers: close every worker session that already wrote a
    /// TERMINAL done signal (done_clean, failed, blocked), release its scope
    /// claim, and unregister its git worktree when it holds nothing unsaved.
    ///
    /// A worker with NO done signal is still working and is never touched, and
    /// `pending` is not a stop either. `omega done` schedules this for the
    /// session it just closed, so a terminal signal now closes its own pane
    /// without a manual `omega kill`; run it by hand to sweep the stragglers.
    /// Reaping twice is identical to reaping once, and an already-closed
    /// session is a quiet exit 0.
    Reap {
        /// Reap only this session. Omit to reconcile every live worker.
        name: Option<String>,
        /// Print what would be reaped and change nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Inspect or resolve the RISK GATE on a mission graph (R-DESTRUCT as a
    /// type): what the gate says about a node, and the attributed approval or
    /// denial a human records against it. Thin by construction — every
    /// judgement belongs to `omega_core::graph_risk`, this only presents it.
    RiskGate {
        #[command(subcommand)]
        action: RiskGateAction,
    },

    /// Dispatch a mission to an oracle
    Dispatch {
        /// Project name
        project: String,
        /// Mission description
        mission: String,
        /// Agent for THIS mission (claude, codex, gemini, pi, hermes, glm).
        /// Defaults to the configured agent_command.
        #[arg(long)]
        agent: Option<String>,
        /// Force a NEW oracle even when one is already working on this project.
        /// Without it, a mission for a project whose oracle is still running is
        /// delivered into that live session as a followup instead of spawning a
        /// sibling.
        #[arg(long = "new")]
        new_oracle: bool,
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
        /// Isolate the worker in its own git worktree (independent HEAD/working-tree
        /// → truly parallel-safe; merge back later with omega-git-merge). Recommended
        /// for any worker that edits files when others run concurrently.
        #[arg(long)]
        worktree: bool,
        /// Agent for THIS worker: claude, codex, glm (default: the configured
        /// agent_command). Restricted to the three finish-guard-covered agents —
        /// a detached worker on any other backend would run without the stop
        /// contract.
        #[arg(long)]
        agent: Option<String>,
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
    ///   omega progress <s> --plan "audit|fix N+1|merge"      (set the plan)
    ///   omega progress <s> --task "audit" --status done       (mark one task)
    ///   omega progress <s>                                    (READ IT BACK, no write)
    /// status = done | fail | doing | todo, and nothing else is accepted.
    /// A re-stated plan KEEPS the status of a title it already knows and DROPS
    /// every title it leaves out, so re-state the WHOLE plan or you delete the
    /// finished items you omitted. Marking a task `doing` sends the previous
    /// `doing` back to `todo`. A `done` task never returns to `todo` or `doing`
    /// (it may still be corrected to `fail`): a walk-back is refused, exit 1,
    /// with nothing written — one invocation is all-or-nothing.
    Progress {
        /// Session name (e.g. oracle-dentistrygpt-7)
        session: String,
        /// Set the full plan: a pipe-separated task list. A title already in the
        /// plan keeps its status, a new title starts as todo, and a title left
        /// out is REMOVED — always pass the complete plan.
        #[arg(long)]
        plan: Option<String>,
        /// Upsert one task by title (use with --status).
        #[arg(long)]
        task: Option<String>,
        /// Status for --task: done | fail | doing | todo.
        #[arg(long)]
        status: Option<String>,
        /// Read-back only: print the plan as JSON instead of a checklist.
        /// Ignored when --plan/--task make this a write.
        #[arg(long)]
        json: bool,
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
        /// Run an explicit live Codex request to verify provider authentication.
        /// This may consume quota and is never enabled by the self-heal cron.
        #[arg(long)]
        deep: bool,
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
    /// The workers of an oracle, split running vs finished. Named as THE
    /// unblocking step by every close/kill refusal — and, until now, not a
    /// command: `omega workers` exited 2 with "unrecognized subcommand" at
    /// exactly the moment the operator had been told to run it.
    Workers {
        /// Oracle session name or bare mission key. Omit for every live oracle.
        oracle: Option<String>,
    },

    /// Every oracle on this box with the one line that matters: phase, plan,
    /// workers, and whether it can close. The roster the operator had to
    /// reconstruct by hand from `omega list`, several `omega status` calls and
    /// a directory listing of ~/.omega/state.
    Oracles {
        /// Include oracles that are no longer live (crashed, or closed and left
        /// on disk). Off by default so the roster answers "what is running".
        #[arg(long)]
        all: bool,
    },

    Gate {
        /// Oracle session name
        oracle: String,
        /// Mission description for rubric
        #[arg(short, long)]
        mission: Option<String>,
        /// Record a HUMAN acceptance of this mission's quality gate. The close
        /// gate demands an independent GateResult, but the only producer of one
        /// is `omega orchestrate`, so a mission dispatched any other way could
        /// never satisfy it and stayed un-closeable forever. This is the signed
        /// alternative: it records WHO accepted and on WHAT evidence.
        #[arg(long)]
        accept: bool,
        /// Who is accepting. Required with --accept, and never defaulted: an
        /// agent must not write its own permission slip.
        #[arg(long)]
        approver: Option<String>,
        /// What was actually verified. Required with --accept.
        #[arg(long)]
        evidence: Option<String>,
    },

    /// Check scope-claim conflicts
    Scope {
        /// Session name to check
        session: String,
        /// Files to check
        files: Vec<String>,
    },

    /// Show session status and pane content. For an ORACLE it also answers the
    /// one question an operator actually has in front of a live mission — "can
    /// this close yet" — with its phase, its plan counts, its current task, its
    /// workers split running/terminal, and the closure verdict with its reason.
    Status {
        /// Session name
        name: String,
        /// Oracle lifecycle as JSON (no pane dump). Ignored for a non-oracle
        /// session, which keeps the plain pane-tail output.
        #[arg(long)]
        json: bool,
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

    /// Mirror a live rmux session into a local viewer session, here or over ssh
    ///
    /// omega stream oracle-Kommu           watch a session on this box
    /// omega stream matrix:MAC-STREAM      watch a session on the `matrix` ssh host
    /// omega stream list                   what is watchable, here and on every ssh host
    ///
    /// The viewer PULLS: it snapshots the RENDERED screen of the source
    /// session (rmux capture-pane) every --interval seconds and reprints it.
    /// It never replays raw bytes, because a full-screen TUI emits cursor
    /// moves that only mean something against a live screen buffer. The source
    /// box ships nothing, so a mirror that stops is noticed on the box that
    /// can do something about it.
    ///
    /// Hosts are ALIASES from ~/.ssh/config: ssh resolves HostName, Port,
    /// User and IdentityFile, so no coordinate is ever hardcoded. The target
    /// is preflighted (host known, box reachable, session present) before a
    /// viewer is created, and an existing viewer is reused rather than
    /// doubled: two pullers on one stream interleave into garbage.
    ///
    /// Detach the viewer with Ctrl-b d. It keeps running; re-attach with
    /// `omega stream` again, or kill it with `omega kill stream-<name>`.
    #[command(verbatim_doc_comment)]
    Stream {
        /// `<session>`, `<host>:<session>`, or the literal `list`
        target: String,
        /// Create the viewer but do not attach to it
        #[arg(short, long)]
        detach: bool,
        /// Seconds between screen snapshots
        #[arg(long, default_value_t = omega_core::stream::DEFAULT_INTERVAL_SECS)]
        interval: u32,
        /// Scrollback lines captured per snapshot
        #[arg(long, default_value_t = omega_core::stream::DEFAULT_LINES)]
        lines: u32,
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

    /// Start a Codex (ChatGPT) device-code re-login: back up the current
    /// credentials, spawn the waiting flow, and print the URL + one-time code as
    /// JSON (`{"ok":true,"url":...,"code":...,"pid":N}`). Headless — the shared
    /// engine for the TUI and the Telegram bridge.
    ///
    /// DESTRUCTIVE BY NATURE: `codex login --device-auth` drops the existing
    /// login the moment it starts, so this backs it up first and
    /// `codex-login-status` restores it if the flow is abandoned.
    #[command(name = "codex-login")]
    CodexLogin,

    /// Settle a Codex device-code re-login: require the recorded child to exit
    /// successfully with a fresh credential, or safely restore canonical
    /// topology when the recorded flow was abandoned.
    /// Prints `{"ok":bool,"status":...,"restored":bool}`.
    #[command(name = "codex-login-status")]
    CodexLoginStatus {
        /// PID from `codex-login`, used only to match the recorded flow owner.
        #[arg(long)]
        pid: Option<u32>,
    },

    /// Abort a recorded Codex device flow only when --pid matches its owned
    /// supervisor identity and exact argv. Telegram Cancel must be repointed
    /// to this command when the Telegram phase owns that integration file.
    #[command(name = "codex-login-abort")]
    CodexLoginAbort {
        /// PID returned by `codex-login`.
        #[arg(long)]
        pid: u32,
    },

    /// Reconcile Codex native and canonical credentials. Active login flows are
    /// reported and preserved. Actual reconciliation errors exit non-zero.
    #[command(name = "codex-reconcile")]
    CodexReconcile {
        /// Print one machine-readable JSON result.
        #[arg(long)]
        json: bool,
    },
}

fn command_owns_codex_reconciliation(command: &Option<Commands>) -> bool {
    matches!(
        command,
        Some(
            Commands::CodexLoginStatus { .. }
                | Commands::CodexLoginAbort { .. }
                | Commands::CodexReconcile { .. },
        )
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("omega=info".parse()?),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let owns_codex_reconciliation = command_owns_codex_reconciliation(&cli.command);

    // SSOT auto-heal: provider CLIs can replace their native credential
    // symlinks with regular files during login/refresh. Reconcile both
    // providers on every startup. Codex is flow-aware: while a recorded device
    // login is active it deliberately leaves the native path alone.
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
    // Explicit settlement/reconcile commands must run exactly once and own
    // their exit status. Every other command gets non-fatal startup recovery.
    if !owns_codex_reconciliation {
        match omega_core::codex_login::reconcile_on_startup() {
            Ok(omega_core::codex_login::StartupReconcile::Reconciled) => {
                tracing::debug!("codex credential topology checked/reconciled");
            }
            Ok(omega_core::codex_login::StartupReconcile::DeferredActiveFlow) => {
                tracing::debug!("codex credential reconciliation deferred for active login flow");
            }
            Ok(omega_core::codex_login::StartupReconcile::DeferredLocked) => {
                tracing::debug!("codex credential reconciliation deferred for held flow lock");
            }
            Err(e) => tracing::warn!(error = %e, "could not reconcile codex credentials"),
        }
    }

    match cli.command {
        Some(Commands::Menu) | None => run_menu().await,
        Some(Commands::New {
            name,
            dir,
            cmd,
            agent,
            prompt,
            files,
        }) => {
            cmd_new(
                &name,
                dir.as_deref(),
                cmd.as_deref(),
                agent.as_deref(),
                prompt.as_deref(),
                files,
            )
            .await
        }
        Some(Commands::NewProject {
            name,
            stack,
            category,
            group,
            resume,
            from,
            skip,
            budget,
            build,
            dry_run,
        }) => {
            cmd_new_project(
                &name,
                &stack,
                &category,
                &group,
                resume,
                from.as_deref(),
                skip.as_deref(),
                budget,
                build,
                dry_run,
            )
            .await
        }
        Some(Commands::Agents) => cmd_agents(),
        Some(Commands::CleanJunk { force }) => cmd_clean_junk(force).await,
        Some(Commands::Clock { full }) => cmd_clock(full),
        Some(Commands::MouseTest) => cmd_mouse_test(),
        Some(Commands::Projects { json }) => cmd_projects(json),
        Some(Commands::Marketing(action)) => cmd_marketing(action),
        Some(Commands::TrustDir { dir }) => cmd_trust_dir(dir.as_deref()),
        Some(Commands::Install { agent, dry_run }) => cmd_install(&agent, dry_run),
        Some(Commands::Master) => cmd_master().await,
        Some(Commands::Config { action }) => cmd_config(action),
        // Two features, one command word. A bare `omega monitor` is the
        // billing view it has always been; a target routes to the session
        // monitor. Splitting on the target rather than renaming keeps every
        // script and TUI binding that calls the shipped command working.
        Some(Commands::Monitor {
            target,
            detach,
            interval,
            lines,
            work_probe,
            work,
            progress_probe,
            audit_every,
        }) => match target {
            None => cmd_monitor(),
            Some(t) => {
                cmd_session_monitor(
                    &t,
                    detach,
                    interval,
                    lines,
                    work_probe.as_deref(),
                    work,
                    progress_probe.as_deref(),
                    audit_every,
                )
                .await
            }
        },
        Some(Commands::Telegram { action }) => cmd_telegram(action).await,
        Some(Commands::Pdf {
            template,
            data,
            demo,
            theme,
            out,
            send,
            caption,
        }) => {
            cmd_pdf(
                &template,
                data.as_deref(),
                demo,
                &theme,
                &out,
                send,
                caption.as_deref(),
            )
            .await
        }
        Some(Commands::Rules { action }) => cmd_rules(action),
        Some(Commands::Skills { action }) => cmd_skills(action),
        Some(Commands::Audit { action }) => cmd_audit(action),
        Some(Commands::Sync) => cmd_sync(),
        Some(Commands::Update {
            check,
            auto,
            dir,
            record_installed,
        }) => {
            if record_installed {
                cmd_update_record_installed(dir.as_deref())
            } else if auto {
                cmd_update_auto(dir.as_deref()).await
            } else {
                cmd_update(check, dir.as_deref())
            }
        }
        Some(Commands::Reconcile { report_only }) => cmd_reconcile(report_only).await,
        Some(Commands::Graph { action }) => match action {
            GraphAction::Run {
                graph,
                state,
                unattended,
                dry_run,
                max_steps,
            } => cmd_graph_run(&graph, state.as_deref(), unattended, dry_run, max_steps).await,
        },
        Some(Commands::InstallBindings) => cmd_install_bindings().await,
        Some(Commands::List) => cmd_list().await,
        Some(Commands::Attach { name }) => cmd_attach(&name).await,
        Some(Commands::Kill { name, force }) => cmd_kill(&name, force).await,
        Some(Commands::Reap { name, dry_run }) => cmd_reap(name.as_deref(), dry_run).await,
        Some(Commands::RiskGate { action }) => cmd_risk_gate(action),
        Some(Commands::Dispatch {
            project,
            mission,
            agent,
            new_oracle,
        }) => cmd_dispatch(&project, &mission, agent.as_deref(), new_oracle).await,
        Some(Commands::Orchestrate {
            project,
            mission,
            dir,
            timeout,
            no_gate,
        }) => cmd_orchestrate(&project, &mission, dir.as_deref(), timeout, no_gate).await,
        Some(Commands::SpawnWorker {
            task,
            prompt,
            dir,
            project,
            files,
            force,
            worktree,
            agent,
        }) => {
            cmd_spawn_worker(
                &task,
                &prompt,
                dir.as_deref(),
                project.as_deref(),
                files,
                force,
                worktree,
                agent.as_deref(),
            )
            .await
        }
        Some(Commands::Team {
            project,
            count,
            dir,
            members,
        }) => cmd_team(&project, count, dir.as_deref(), &members).await,
        Some(Commands::Done {
            session,
            status,
            summary,
            commit,
        }) => cmd_done(&session, &status, &summary, commit.as_deref()).await,
        Some(Commands::Progress {
            session,
            plan,
            task,
            status,
            json,
        }) => cmd_progress(
            &session,
            plan.as_deref(),
            task.as_deref(),
            status.as_deref(),
            json,
        ),
        Some(Commands::Inbox { oracle, action }) => cmd_inbox(&oracle, &action).await,
        Some(Commands::Ship {
            project,
            message,
            unfreeze,
        }) => cmd_ship(&project, &message, unfreeze).await,
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
        Some(Commands::Doctor {
            pre_reset,
            fix,
            deep,
        }) => {
            if pre_reset {
                cmd_doctor_pre_reset()
            } else {
                cmd_doctor(fix, deep).await
            }
        }
        Some(Commands::Backup {
            out,
            include_memory,
        }) => cmd_backup(out, include_memory),
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
        Some(Commands::Workers { oracle }) => cmd_workers(oracle.as_deref()).await,
        Some(Commands::Oracles { all }) => cmd_oracles(all).await,
        Some(Commands::Gate {
            oracle,
            mission,
            accept,
            approver,
            evidence,
        }) => {
            cmd_gate(
                &oracle,
                mission.as_deref(),
                accept,
                approver.as_deref(),
                evidence.as_deref(),
            )
            .await
        }
        Some(Commands::Scope { session, files }) => cmd_scope(&session, &files).await,
        Some(Commands::Status { name, json }) => cmd_status(&name, json).await,
        Some(Commands::Send { name, text }) => cmd_send(&name, &text).await,
        Some(Commands::Capture { name }) => cmd_capture(&name).await,
        Some(Commands::Stream {
            target,
            detach,
            interval,
            lines,
        }) => cmd_stream(&target, detach, interval, lines).await,
        Some(Commands::Log { session, count }) => cmd_log(&session, count).await,
        Some(Commands::Rpc) => omega_core::rpc::run_rpc_loop().await,
        Some(Commands::Route { mission }) => cmd_route(&mission),
        Some(Commands::Completions { shell }) => cmd_completions(&shell),
        Some(Commands::Init) => cmd_init().await,
        Some(Commands::PlanStatus { path }) => cmd_plan_status(&path),
        Some(Commands::PlanRun { path }) => cmd_plan_run(&path).await,
        Some(Commands::ClaudeLogin) => cmd_claude_login().await,
        Some(Commands::ClaudeLoginCode { code }) => cmd_claude_login_code(&code).await,
        Some(Commands::CodexLogin) => cmd_codex_login().await,
        Some(Commands::CodexLoginStatus { pid }) => cmd_codex_login_status(pid).await,
        Some(Commands::CodexLoginAbort { pid }) => cmd_codex_login_abort(pid).await,
        Some(Commands::CodexReconcile { json }) => cmd_codex_reconcile(json).await,
    }
}

/// Whether we pushed kitty keyboard-enhancement flags at TUI init
/// (DESIGN-014) — every teardown path (quit + Ctrl+R restart) must pop
/// exactly what was pushed, and nothing on legacy terminals.
static KBD_ENHANCED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Pop the keyboard-enhancement flags if (and only if) a push recorded them.
/// `swap(false)` keeps the bookkeeping exact: after the pop the flag reflects
/// reality, so a later teardown (or a re-push on attach/restart re-entry)
/// never double-pops or skips a needed push.
fn pop_kbd_enhancement(out: &mut impl std::io::Write) {
    if KBD_ENHANCED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        crossterm::execute!(out, crossterm::event::PopKeyboardEnhancementFlags).ok();
    }
}

/// Probe-guarded push of the kitty keyboard-enhancement flags (DESIGN-014).
/// FIX-C ordering: call AFTER EnterAlternateScreen + enable_raw_mode — the
/// push must land on the ALT-screen keyboard-mode stack (per-screen stacks),
/// and the probe needs raw mode. Records what was pushed in KBD_ENHANCED so
/// the matching pop is exact. Used at init, after the standalone-attach
/// handover returns, and on restart-failure TUI re-entry.
fn push_kbd_enhancement(out: &mut impl std::io::Write) {
    let supported = crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    KBD_ENHANCED.store(supported, std::sync::atomic::Ordering::Relaxed);
    if supported {
        crossterm::execute!(
            out,
            crossterm::event::PushKeyboardEnhancementFlags(
                crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            )
        )
        .ok();
    }
}

/// The ONE terminal-restore sequence (FIX-C ordering, encoded once): pop the
/// kbd flags while still on the alt-screen stack → raw mode off → leave the
/// alt screen + release mouse capture / bracketed paste → re-show the cursor.
/// Best-effort at every step — teardown must never abort halfway and strand
/// the terminal in a worse state. Used by the quit path, the Ctrl+R restart
/// teardown, and the (main-thread) panic hook; a panic hook runs in ordinary
/// code context (not a signal handler), so the cursor::Show write is safe
/// there too.
fn restore_terminal(out: &mut impl std::io::Write) {
    pop_kbd_enhancement(out);
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        out,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste,
        crossterm::cursor::Show,
    );
}

async fn run_menu() -> Result<()> {
    use omega_tui::app::App;

    let config = OmegaConfig::load().unwrap_or_default();
    // Apply the persisted TUI theme before the first frame renders.
    omega_tui::theme::set_active_slug(&config.theme);
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
                    Ok(true) => {
                        app.status_message = Some(
                            "Master AISB session spawned automatically — ready to delegate"
                                .to_string(),
                        )
                    }
                    Ok(false) => {
                        app.status_message = Some("Master AISB already running".to_string())
                    }
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

    // FIX-I (D-12): a panic inside the TUI loop must not strand the user's
    // terminal raw, stuck in the alternate screen, or with the kitty
    // enhancement flags active (Esc arrives as `CSI 27 u`, Ctrl+C stops
    // signalling SIGINT). Restore the terminal FIRST — pop the flags while
    // still in the alt screen (where FIX-C pushes them), then leave it —
    // and only then let the default hook print the panic where it's readable.
    //
    // Main-thread only: tokio polls the root future (this TUI loop) on the
    // thread that called block_on, but spawn_blocking closures (meta/git
    // scans) and detached tokio::spawn tasks (reauth) run on OTHER threads —
    // a panic there must NOT cook the terminal under the still-running render
    // loop. Background panics are appended to ~/.omega/logs/tui-panic.log
    // instead, without touching the terminal or the stderr default hook.
    let default_panic_hook = std::panic::take_hook();
    let tui_thread = std::thread::current().id();
    std::panic::set_hook(Box::new(move |info| {
        if std::thread::current().id() == tui_thread {
            restore_terminal(&mut std::io::stdout());
            default_panic_hook(info);
        } else if let Some(home) = dirs::home_dir() {
            let dir = home.join(".omega/logs");
            let _ = std::fs::create_dir_all(&dir);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("tui-panic.log"))
            {
                use std::io::Write as _;
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let _ = writeln!(f, "[{}] background-thread panic: {}", ts, info);
            }
        }
    }));

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
    // DESIGN-014: make the chat Alt+Esc literal-ESC hatch deliverable. Legacy
    // terminals emit Alt+Esc as an ESC ESC byte pair that crossterm parses as
    // plain Esc — one press when the pair lands in a single read
    // (parse.rs:77), two when delivery splits — so the `KeyCode::Esc if alt`
    // arm never fires from real input. On terminals speaking the kitty
    // keyboard protocol, pushing
    // DISAMBIGUATE_ESCAPE_CODES delivers the real Alt modifier. Probe first
    // (graceful fallback — the probe needs raw mode, enabled above): on
    // unsupported terminals nothing is pushed and the Esc-Esc chord
    // (input.rs) is the literal-ESC path; we only pop what we pushed.
    // FIX-C (D-1/D-2): push AFTER EnterAlternateScreen — kitty-class
    // terminals keep INDEPENDENT per-screen keyboard-mode stacks ("The main
    // and alternate screens … must maintain their own, independent, keyboard
    // mode stacks"). Pushing on the main screen made the flag a no-op inside
    // the TUI AND leaked DISAMBIGUATE into the user's shell after quit. Push
    // in the alt screen; both pops (quit below, Ctrl+R restart) already run
    // before LeaveAlternateScreen, i.e. on the same alt-screen stack.
    push_kbd_enhancement(&mut stdout);
    let backend = ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_tui_loop(&mut terminal, &mut app).await;

    // FIX-C: pop BEFORE LeaveAlternateScreen — the push above landed on the
    // alt-screen stack, so the pop must drain that same stack.
    restore_terminal(terminal.backend_mut());

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
        "general.theme_background" => {
            let mut c = OmegaConfig::load().unwrap_or_default();
            c.theme_background = !c.theme_background;
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

fn should_refresh_preview_after_event(
    selected_before: usize,
    selected_after: usize,
    tab_before: omega_tui::app::Tab,
    tab_after: omega_tui::app::Tab,
    selected_session_before: Option<&str>,
    selected_session_after: Option<&str>,
) -> bool {
    selected_after != selected_before
        || tab_after != tab_before
        || selected_session_after != selected_session_before
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
    let reauth_sink: std::sync::Arc<std::sync::Mutex<Option<omega_tui::app::ReauthStatus>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));

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
    let mut last_meta_refresh = std::time::Instant::now() - std::time::Duration::from_secs(10);
    // Per-session transcript mtime, so we re-scan the (possibly tens-of-MB)
    // JSONL only when it actually changed.
    let mut meta_mtimes: std::collections::HashMap<String, std::time::SystemTime> =
        std::collections::HashMap::new();
    // Throttle for the per-session git status (branch + age of oldest unpushed
    // commit). Shown in the status bar on the Sessions tab as e.g. `↑4h • main`.
    let mut last_git_refresh = std::time::Instant::now() - std::time::Duration::from_secs(60);

    loop {
        // Drain any error reported by a backgrounded keystroke forwarder.
        // Sticky (FIX-2/NR-4): the error targets a user mid-typing — their
        // next in-flight keystroke must not consume it before they read it.
        if let Ok(mut guard) = async_status.lock() {
            if let Some(msg) = guard.take() {
                app.set_status_sticky(msg);
            }
        }

        // Drain the OAuth re-login engine result (set by a detached task).
        if let Ok(mut guard) = reauth_sink.lock() {
            if let Some(status) = guard.take() {
                app.reauth_status = status;
            }
        }

        terminal.draw(|f| draw(f, app))?; // app is &mut, allows auto-scroll

        // ── Mouse-selection clipboard ───────────────────────────────────
        // Push the drag-selected preview text to the user's clipboard via
        // OSC 52, written RAW to stdout (an escape sequence — it doesn't
        // disturb the ratatui buffer). Works over SSH/Termius, and rmux
        // forwards OSC 52 to the outer terminal when the TUI runs nested.
        if let Some(text) = app.pending_clipboard.take() {
            use base64::Engine as _;
            use std::io::Write as _;
            let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
            let mut out = std::io::stdout();
            let _ = write!(out, "\x1b]52;c;{}\x07", b64);
            let _ = out.flush();
        }

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
                // Keep the rmux pane exactly aligned with the visible panel,
                // including very short terminals. The old minimum of 10 rows
                // hid the top of the composer whenever the viewport was
                // shorter than 10 rows. Hidden extra rows (the
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
                let rows = app.preview_inner_height.max(1);
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
        let interacting =
            last_input_at.elapsed() < std::time::Duration::from_millis(PREVIEW_ACTIVE_WINDOW_MS);
        let preview_refresh_interval = if interacting {
            std::time::Duration::from_millis(PREVIEW_ACTIVE_MS)
        } else {
            std::time::Duration::from_millis(PREVIEW_IDLE_MS)
        };
        // 60 FPS while interacting, ~15 FPS at rest (cuts idle render CPU ~4×).
        let tick_rate = if interacting { TICK_ACTIVE } else { TICK_IDLE };
        // Only the Sessions tab renders the preview (draw_sessions_right), so
        // only it needs the rmux capture. Ungated, navigating Marketing or
        // Projects fired a capture RPC every 16ms for a pane nobody was
        // looking at — arrow keys mark the loop "interacting", so browsing
        // those tabs pinned the fast cadence. Same gate as the meta/git
        // refreshes above.
        if !event_pending
            && app.tab == omega_tui::app::Tab::Sessions
            && last_preview_refresh.elapsed() >= preview_refresh_interval
        {
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
            let selected_session_before = app
                .selected_session()
                .map(|entry| entry.session.name.clone());
            let tab_before = app.tab;
            let detail_focused_before = app.detail_focused;
            // F-7: a status notice lives until the NEXT keypress — or mouse
            // click (FIX-6/NEW-6: a notice set during a mouse-only flow must
            // not mask the Sessions git text forever). Clearing it here
            // (before dispatch) lets the handler below set a fresh message
            // that then survives until the user types again — and on the
            // Sessions tab the bar falls back to the per-session git text once
            // the notice is consumed (see draw_status_bar). The exemptions —
            // an armed destructive-menu confirm (FIX-1) and async-origin
            // sticky notices inside their minimum display (FIX-2) — live in
            // consume_status_ttl.
            let consumes_ttl = match &evt {
                crossterm::event::Event::Key(k) => {
                    k.kind != crossterm::event::KeyEventKind::Release
                }
                crossterm::event::Event::Mouse(m) => {
                    matches!(m.kind, crossterm::event::MouseEventKind::Down(_))
                }
                _ => false,
            };
            if consumes_ttl {
                app.consume_status_ttl();
            }
            let status_before = app.status_message.clone();
            match handle_event(app, evt) {
                Action::Quit => break,
                Action::ToggleMouseCapture => {
                    // Flip terminal mouse capture live. OFF → the terminal does
                    // native drag-select + copy/paste; ON → clickable menus + scroll.
                    app.mouse_capture = !app.mouse_capture;
                    if app.mouse_capture {
                        crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::event::EnableMouseCapture
                        )
                        .ok();
                        app.status_message = Some(
                            "🖱  Mouse ON — click menus & scroll  ·  Ctrl-T for text selection"
                                .to_string(),
                        );
                    } else {
                        crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::event::DisableMouseCapture
                        )
                        .ok();
                        app.status_message = Some("📋 Selection mode — drag to select & copy/paste  ·  Ctrl-T to re-enable clicks".to_string());
                    }
                }
                Action::Restart => {
                    // Tear down the terminal cleanly, then re-exec the
                    // current binary so a freshly-built `omega` is picked up
                    // in place (same PID on Unix via exec).
                    // FIX-C: pop BEFORE LeaveAlternateScreen — same alt-screen
                    // stack the init push landed on (no per-restart orphan).
                    restore_terminal(terminal.backend_mut());
                    // Resolve a binary path that actually exists. current_exe()
                    // can point at a now-replaced/deleted inode after a redeploy
                    // (cp over ~/.local/bin/omega), which makes exec() fail with
                    // ENOENT. Prefer the canonical install path, then current_exe,
                    // then a bare PATH lookup.
                    use std::os::unix::process::CommandExt;
                    let home = dirs::home_dir().unwrap_or_else(|| {
                        std::env::var("HOME")
                            .map(std::path::PathBuf::from)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    });
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
                    // BOTH execs failed (we only get here on exec failure —
                    // success never returns). The old `break` only left the
                    // event drain; Restart never set should_quit, so the
                    // outer loop kept rendering frames onto the cooked main
                    // screen. Re-enter the TUI fully instead of quitting:
                    // the failure is transient (e.g. the binary mid-replace
                    // during a redeploy) and the user keeps their session
                    // manager, with the error surfaced as a sticky notice.
                    let _ = crossterm::terminal::enable_raw_mode();
                    let _ = crossterm::execute!(
                        terminal.backend_mut(),
                        crossterm::terminal::EnterAlternateScreen,
                        crossterm::event::EnableBracketedPaste,
                    );
                    if app.mouse_capture {
                        let _ = crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::event::EnableMouseCapture
                        );
                    }
                    // Fresh alt-screen kbd stack — re-probe + re-push, same
                    // guarded sequence as init (FIX-C ordering).
                    push_kbd_enhancement(terminal.backend_mut());
                    let _ = terminal.clear();
                    app.set_status_sticky(format!(
                        "restart failed: {} (binary: {})",
                        err,
                        chosen.display()
                    ));
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
                                app.status_message = Some(format!(
                                    "switch-client failed (exit {})",
                                    s.code().unwrap_or(-1)
                                ));
                            }
                            Err(e) => {
                                app.status_message = Some(format!("switch-client error: {}", e));
                            }
                        }
                    } else {
                        // Standalone mode — full terminal handover.
                        // The init push landed on the ALT-screen kbd stack
                        // (per-screen stacks): pop it BEFORE leaving, or the
                        // flags are stranded there and dead for the rest of
                        // the run after a single attach round-trip.
                        pop_kbd_enhancement(terminal.backend_mut());
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
                        // Re-arm the input modes the ATTACHED CLIENT turned off
                        // on its way out. rmux's client disables mouse reporting
                        // and bracketed paste when it detaches (rmux-server
                        // outer_terminal.rs disable_mouse_sequence: ?1000l ?1002l
                        // ?1006l) — it is undoing ITS OWN setup, but the sequences
                        // hit the shared terminal, so they also silently disarm
                        // OURS. Init enabled both; without this they stay off for
                        // the whole rest of the run, and the symptom is brutal:
                        // after one single attach round-trip the wheel does
                        // nothing in the menu, clicks do nothing, and a long
                        // paste fragments into per-character keys. The Ctrl+R
                        // restart path below already re-arms them for exactly
                        // this reason; the attach path was the one that forgot.
                        // Gated on app.mouse_capture so a deliberate Ctrl-T
                        // "mouse OFF" (native terminal selection) is not undone.
                        if app.mouse_capture {
                            let _ = crossterm::execute!(
                                terminal.backend_mut(),
                                crossterm::event::EnableMouseCapture
                            );
                        }
                        let _ = crossterm::execute!(
                            terminal.backend_mut(),
                            crossterm::event::EnableBracketedPaste
                        );
                        // Fresh alt-screen kbd stack — re-probe + re-push,
                        // same guarded sequence as init (FIX-C ordering:
                        // after EnterAlternateScreen + raw mode).
                        push_kbd_enhancement(terminal.backend_mut());
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
                                if let Some(agent) =
                                    omega_core::agents::Agent::from_name(&cfg.aisb_agent)
                                {
                                    let cwd = std::env::current_dir()
                                        .ok()
                                        .and_then(|p| p.to_str().map(String::from))
                                        .unwrap_or_else(|| "/home".to_string());
                                    match omega_core::aisb::ensure_master(&mgr, agent, &cwd).await {
                                        Ok(_) => {
                                            app.status_message =
                                                Some(format!("Killed {} → auto-respawned", name))
                                        }
                                        Err(e) => {
                                            app.status_message = Some(format!(
                                                "Killed {} but respawn failed: {}",
                                                name, e
                                            ))
                                        }
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
                        Err(e) => app.status_message = Some(format!("Kill-all failed: {}", e)),
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
                            app.status_message =
                                Some(format!("Nuclear cleanup: {}", report.summary()));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Nuclear cleanup failed: {}", e))
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
                Action::CreateSessionWithAgent {
                    name,
                    agent,
                    prompt,
                } => {
                    let mgr = SessionManager::connect().await?;
                    match mgr
                        .create_session_with_agent(&name, None, agent, prompt.as_deref())
                        .await
                    {
                        Ok(_) => {
                            app.status_message = Some(format!(
                                "Created {} with {} — opening chat…",
                                name,
                                agent.name()
                            ));
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
                                    app.status_message = Some(format!(
                                        "Created {} ({}) — opening chat…",
                                        name,
                                        agent.name()
                                    ));
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
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt,
                    launch_docs,
                } => {
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
                    let mut prompt = format!(
                        "/omega-new-project {} {} {} {}",
                        stack, category, name, group
                    );
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
                    let agent = omega_core::agents::Agent::Codex;
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
                    // DETACHED, like the two reauth actions above and for the
                    // same reason. A dispatch used to be prompt, so handling it
                    // inline was free; a followup dispatch now waits for the
                    // target's composer (up to 8s) and then confirms the paste
                    // was accepted (up to 3s more), and every one of those
                    // seconds froze the whole event loop — no redraw, no
                    // keystrokes, no preview. The result lands in async_status,
                    // which the top of the loop drains into the status bar, and
                    // the session list is refreshed by the periodic tick.
                    app.status_message =
                        Some(format!("◆ Dispatching to {} — briefing the oracle…", project));
                    let sink = async_status.clone();
                    tokio::spawn(async move {
                        let cfg = OmegaConfig::load().unwrap_or_default();
                        let msg = match SessionManager::connect().await {
                            Ok(mgr) => {
                                let dispatcher =
                                    omega_core::dispatch::Dispatcher::new(mgr, cfg.clone());
                                match dispatcher
                                    .dispatch_oracle_with_agent(&project, &mission, None, false)
                                    .await
                                {
                                    Ok(outcome) => {
                                        write_dispatch_session_log(&cfg, &outcome, &mission);
                                        format!(
                                            "◆ Dispatched: {} ({})",
                                            outcome.oracle_name,
                                            outcome.delivery.tag()
                                        )
                                    }
                                    Err(e) => format!("Dispatch failed: {}", e),
                                }
                            }
                            Err(e) => format!("Dispatch failed: {}", e),
                        };
                        if let Ok(mut g) = sink.lock() {
                            *g = Some(msg);
                        }
                    });
                }
                Action::Refresh => {
                    // Ack AFTER the refresh completes (pre-series ordering).
                    // The old pre-refresh `is_none()` gate ate the ack
                    // whenever an async sticky notice sat inside its 2s
                    // window (consume_status_ttl keeps it → Some → no ack),
                    // so F5 looked dead. Post-refresh, the ack overwrites any
                    // pre-press leftover — the one thing it still yields to
                    // is a notice refresh() itself just produced (CA-4: the
                    // "<name> ended — back to list" vanish sticky), detected
                    // as a change across the call. FIX-A is unaffected:
                    // armed warnings render state-driven, above any
                    // status_message write.
                    let before_refresh = app.status_message.clone();
                    let _ = app.refresh().await;
                    let _ = app.refresh_preview().await;
                    if app.tab == omega_tui::app::Tab::Projects {
                        app.refresh_projects();
                    }
                    if app.tab == omega_tui::app::Tab::Os {
                        app.refresh_os();
                    }
                    if app.status_message == before_refresh {
                        app.status_message = Some("Refreshed".to_string());
                        // fix7-T4: the ack is a deliberate user action — it
                        // supersedes any async sticky still inside its 2s
                        // window. Clear the pair WITH the overwrite, or
                        // status_sticky_msg/at dangle pointing at a message
                        // no longer shown (pair desync). Refresh-produced
                        // notices (the != branch) keep their sticky untouched.
                        app.status_sticky_at = None;
                        app.status_sticky_msg = None;
                    }
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
                                match omega_core::oauth::request_reauth(&mgr, "tui", None, true)
                                    .await
                                {
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
                    app.status_message = Some(format!("Step 1/{}: {}", fields.len(), fields[0].1));
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
                            app.status_message = Some(format!("Provisioning save failed: {}", e));
                        }
                    }
                }
                Action::TelegramSetupCommit {
                    bot_token,
                    chat_id,
                    user_ids,
                } => {
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
                            //    The real bridge is the Bun bot shipped as a user
                            //    service (systemd --user on Linux, a launchd
                            //    LaunchAgent on macOS — service::tg_bot_start picks
                            //    the right manager). We must NOT spawn a competing
                            //    Rust bridge (that produced two pollers hitting
                            //    getUpdates → permanent HTTP 409). Kill any stale
                            //    rmux bridge first, then enable+start the canonical
                            //    unit. The start is wrapped in a timeout so a
                            //    slow/hung service manager can never FREEZE the
                            //    wizard UI (the "had to refresh manually" bug).
                            let mgr = SessionManager::connect().await?;
                            let _ = mgr.kill_session("omega-telegram-bridge").await;
                            let service_ok = matches!(
                                tokio::time::timeout(
                                    std::time::Duration::from_secs(8),
                                    tokio::task::spawn_blocking(omega_core::service::tg_bot_start),
                                )
                                .await,
                                Ok(Ok(true))
                            );
                            if service_ok {
                                app.status_message = Some(
                                    "[+] Telegram setup done — bridge running as the persistent omega-tg-bot service".to_string(),
                                );
                            } else {
                                app.status_message = Some(format!(
                                    "[+] Telegram setup saved, but the omega-tg-bot service could not be started. Start it with: {}",
                                    omega_core::service::tg_bot_start_hint()
                                ));
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
                                    let _ =
                                        omega_core::aisb::ensure_master(&mgr, agent, &cwd).await;
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
                    let cmd = format!(
                        "bash -c {}",
                        shell_escape_for_bash(&format!(
                            "{}{}; echo; echo '─── done ───'; exec bash",
                            command, post_install
                        ))
                    );
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
                Action::EditSettingsField {
                    config_key,
                    current,
                    masked,
                } => {
                    app.input_buffer = current;
                    app.input_mode = omega_tui::app::InputMode::EditSettingsField {
                        config_key: config_key.clone(),
                        masked,
                    };
                    app.status_message = Some(format!(
                        "Editing {} — Enter to save, Esc to cancel",
                        config_key
                    ));
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
                    // Auto-update policy lives in ~/.omega/config.toml with its
                    // own writer (it must survive a rewrite of that file), so it
                    // takes the same detour as the theme below. Without this
                    // branch the value falls through to providers.toml and is
                    // saved SILENTLY to a file nothing reads it from.
                    if config_key == "general.auto_update" {
                        // `parse` never fails (unknown text falls back to
                        // `apply`, deliberately — a typo must not quietly stop
                        // updates), and the picker only offers the three valid
                        // values, so there is no invalid path to handle here.
                        let policy = omega_core::config::AutoUpdatePolicy::parse(&value);
                        match OmegaConfig::set_auto_update(policy) {
                            Ok(()) => {
                                app.config = OmegaConfig::load().unwrap_or_default();
                                app.status_message = Some(format!(
                                    "Auto-update set to '{}' — saved [+]",
                                    policy.as_str()
                                ));
                            }
                            Err(e) => {
                                app.status_message = Some(format!("Save failed: {}", e));
                            }
                        }
                    } else if config_key == "general.theme" {
                        // Same reason as above: the theme lives in
                        // ~/.omega/config.toml, not providers.toml.
                        let mut c = OmegaConfig::load().unwrap_or_default();
                        c.theme = value.clone();
                        if let Err(e) = save_omega_config(&c) {
                            app.status_message = Some(format!("Save failed: {}", e));
                        } else {
                            omega_tui::theme::set_active_slug(&value);
                            app.config = OmegaConfig::load().unwrap_or_default();
                            let label = omega_tui::theme::ThemeId::from_slug(&value)
                                .map(|t| t.label())
                                .unwrap_or(value.as_str());
                            app.status_message =
                                Some(format!("Theme '{}' applied — saved [+]", label));
                        }
                    } else {
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
                }
                Action::RenameSession { old, new } => {
                    let mgr = SessionManager::connect().await?;
                    match mgr.rename_session(&old, &new).await {
                        // rename_session sanitizes — select/status must use the
                        // name actually applied, not the raw modal input, or the
                        // selection (and every later lookup) misses the session.
                        Ok(safe) => {
                            app.status_message = Some(format!("Renamed {} → {}", old, safe));
                            let _ = app.refresh().await;
                            let _ = app.select_by_name(&safe);
                            // The renamed pane must reload NOW — drop the old
                            // name's preview state instead of letting the
                            // fail-streak walk to "(session has no pane content)".
                            let _ = app.refresh_preview().await;
                        }
                        Err(e) => {
                            omega_core::tuilog::log(format!(
                                "rename '{old}' → '{new}' FAILED: {e:#}"
                            ));
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
                            app.status_message = Some(
                                "Nothing to disconnect — no Telegram config present".to_string(),
                            );
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
                Action::OpenProject { name, path, agent } => {
                    let mgr = SessionManager::connect().await?;
                    // Opening a project ALWAYS spawns a NEW session with the
                    // picked agent. It deliberately does NOT re-attach to the
                    // project's live oracle: re-entering an existing session is
                    // the Sessions tab's job, and silently attaching made "open"
                    // look like it did nothing.
                    //
                    // Every provider comes up as a CLEAN ORACLE: it is seeded with
                    // the per-project oracle system prompt (identity + doctrine +
                    // protocol via OraclePromptGenerator) instead of a blank
                    // shell, so the operator lands in a project manager ready to
                    // decompose and dispatch, not an empty prompt.
                    let safe = name
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .take(24)
                        .collect::<String>();
                    let base = format!("{}-{}", safe, agent.name());
                    // Uniquify so repeated opens stack instead of colliding
                    // with (or silently reusing) an earlier session.
                    let taken: Vec<String> = mgr
                        .list_sessions()
                        .await
                        .map(|ss| ss.iter().map(|s| s.name.clone()).collect())
                        .unwrap_or_default();
                    let mut session = base.clone();
                    let mut n = 2;
                    while taken.iter().any(|t| t == &session) {
                        session = format!("{}-{}", base, n);
                        n += 1;
                    }
                    // Seed the same provider-neutral oracle identity for every
                    // backend. Codex is the fresh-install default, but an
                    // explicit operator choice remains authoritative.
                    let oracle_prompt = Some(
                        omega_core::oracle_lifecycle::OraclePromptGenerator::generate(
                            &name,
                            std::path::Path::new(&path),
                            &session,
                            "Interactive oracle session for this project. Await the operator's \
                             instructions, then analyze, decompose, dispatch workers, verify, and \
                             report. Do NOT edit project code directly (delegate to workers).",
                            false,
                            false,
                        ),
                    );
                    match mgr
                        .create_session_with_agent(
                            &session,
                            Some(&path),
                            agent,
                            oracle_prompt.as_deref(),
                        )
                        .await
                    {
                        Ok(_) => {
                            app.status_message = Some(format!(
                                "▶ {} — new {} {} ({})",
                                name,
                                agent.name(),
                                "oracle",
                                session
                            ));
                            auto_focus_chat(app, &session).await;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Could not open {}: {}", name, e));
                        }
                    }
                }
                Action::OpenMarketingSession { name, cwd, prompt, agent } => {
                    let mgr = SessionManager::connect().await?;
                    // The marketing/ dir may not exist yet on a project that
                    // never did marketing — create it so the session lands in
                    // its real workspace (the agent scaffolds the structure).
                    let _ = std::fs::create_dir_all(&cwd);
                    // If a marketing session for this project already exists, just
                    // re-attach (idempotent — avoids stacking duplicates).
                    let existing = mgr
                        .list_sessions()
                        .await
                        .map(|ss| ss.iter().any(|s| s.name == name))
                        .unwrap_or(false);
                    if existing {
                        app.status_message = Some(format!("Attaching to {}", name));
                        auto_focus_chat(app, &name).await;
                    } else {
                        match mgr
                            .create_session_with_agent(&name, Some(&cwd), agent, Some(&prompt))
                            .await
                        {
                            Ok(_) => {
                                app.status_message = Some(format!(
                                    "📣 {} — marketing session ({}), opening chat…",
                                    name,
                                    agent.name()
                                ));
                                auto_focus_chat(app, &name).await;
                            }
                            Err(e) => {
                                app.status_message =
                                    Some(format!("Marketing session failed: {}", e));
                            }
                        }
                    }
                }
                Action::OpenOsSession { name, cwd, prompt } => {
                    let mgr = SessionManager::connect().await?;
                    // Same idempotent contract as marketing: one master-agent
                    // session per OS — re-attach instead of stacking duplicates.
                    let existing = mgr
                        .list_sessions()
                        .await
                        .map(|ss| ss.iter().any(|s| s.name == name))
                        .unwrap_or(false);
                    if existing {
                        app.status_message = Some(format!("Attaching to {}", name));
                        auto_focus_chat(app, &name).await;
                    } else {
                        match mgr
                            .create_session_with_agent(
                                &name,
                                Some(&cwd),
                                omega_core::agents::Agent::Claude,
                                Some(&prompt),
                            )
                            .await
                        {
                            Ok(_) => {
                                app.status_message =
                                    Some(format!("💬 {} — OS session, opening chat…", name));
                                auto_focus_chat(app, &name).await;
                            }
                            Err(e) => {
                                app.status_message =
                                    Some(format!("OS session failed: {}", e));
                            }
                        }
                    }
                }
                Action::LinkOsBot { slug } => {
                    let mgr = SessionManager::connect().await?;
                    let session = format!("os-{}-bot-link", slug);
                    // Interactive: the script prompts for the @BotFather token
                    // in the terminal, validates it, wires agent-bots.json and
                    // the systemd unit, then verifies the bot is live.
                    let script = format!(
                        "{}/.omega/bin/omega-os-bot.sh",
                        dirs::home_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("~"))
                            .to_string_lossy()
                    );
                    let cmd = format!(
                        "bash -c {}",
                        shell_escape_for_bash(&format!(
                            "bash {} {}; echo; echo '─── done — F5 in the OS tab refreshes the bot status ───'; exec bash",
                            script, slug
                        ))
                    );
                    match mgr.create_session(&session, None, Some(&cmd)).await {
                        Ok(_) => {
                            app.status_message = Some(format!(
                                "🤖 Linking a Telegram bot to {} — paste the token in the session",
                                slug
                            ));
                            auto_focus_chat(app, &session).await;
                        }
                        Err(e) => {
                            app.status_message =
                                Some(format!("Bot-link session failed for {}: {}", slug, e));
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
                        format!("{}/.omega", dirs::home_dir().unwrap_or_default().display())
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
                    let mut cfg =
                        omega_core::telegram_group::TelegramGroupConfig::load().unwrap_or_default();
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
                            app.status_message = Some(format!("Group setup save failed: {}", e));
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

            // Entering the Projects tab (which hosts the Projects group) →
            // reload the registry so projects added via `omega project add` in
            // another shell show up without restart.
            if app.tab == omega_tui::app::Tab::Projects
                && tab_before != omega_tui::app::Tab::Projects
            {
                app.refresh_projects();
            }

            // On tab switch, seed the status bar with a per-tab hint so a new
            // user lands with guidance instead of an empty bar. Skip if the
            // action handler already set a meaningful message this iteration
            // (e.g. a dispatch/login that also switched to the Sessions tab).
            // Lazy-load marketing projects on first entry to the Marketing tab
            // (the fs + crontab scan is heavier than the registry, so we defer
            // it off startup). Reload only if empty — F5 forces a full refresh.
            // Same lazy contract for the OS suite (registry + fs stat — cheap,
            // but keep startup untouched). F5 forces a full refresh.
            if app.tab != tab_before
                && app.tab == omega_tui::app::Tab::Os
                && app.os_entries.is_empty()
            {
                app.refresh_os();
            }

            if app.tab != tab_before
                && app.status_message == status_before
                // FIX-G (D-8): never overwrite an async sticky notice still
                // inside its minimum display window (an in-flight Left/Right
                // was breaking FIX-2's 2s promise). Armed-confirm warnings
                // need no guard here: FIX-A renders them state-driven, with
                // priority over any status_message text.
                && !app.status_sticky_unexpired()
            {
                use omega_tui::app::Tab;
                app.status_message = Some(match app.tab {
                    Tab::Sessions => "↑/↓ select · Enter/Tab chat · c/C/g new agent · x kill · . lock · F5 refresh".to_string(),
                    Tab::Menu => "↑/↓ select · Enter run · or press the shortcut key shown".to_string(),
                    Tab::Settings => "↑/↓ Monitor + Settings sections · Enter/Tab edit · L login · T telegram · B billing".to_string(),
                    Tab::Projects => "↑/↓ projects · Tab focus detail · n add · p plan · d dispatch · Enter open".to_string(),
                    Tab::System => "↑/↓ sections · Tab focus detail · [ ] jump section · Laws · Rules · Agents · Skills · Docs".to_string(),
                    Tab::Os => "↑/↓ operative systems · Enter master agent · T link Telegram bot · F5 refresh".to_string(),
                    Tab::Help => "↑/↓ scroll · Esc back to Sessions".to_string(),
                });
            }

            if should_refresh_preview_after_event(
                selected_before,
                app.selected,
                tab_before,
                app.tab,
                selected_session_before.as_deref(),
                app.selected_session()
                    .map(|entry| entry.session.name.as_str()),
            ) {
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
            // Self-healing refresh: a failure here used to be swallowed
            // silently (`let _ =`), freezing the list on stale state with no
            // recovery while the daemon kept running fine — the classic "my
            // sessions disappeared from the interface" report. Log it and
            // drop the cached daemon connection so the next tick redials.
            if let Err(e) = app.refresh().await {
                omega_core::tuilog::log(format!(
                    "refresh failed: {e:#} — resetting cached daemon connection"
                ));
                SessionManager::reset_cached().await;
            }
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
        let agent_enum = omega_core::agents::Agent::from_name(agent_name).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown agent: {}. Run `omega agents` to list options.",
                agent_name
            )
        })?;
        if !agent_enum.is_available() {
            eprintln!(
                "Warning: {} not detected on this system. Session will be created anyway.",
                agent_enum.display_name()
            );
        }
        let _session = mgr
            .create_session_with_agent(name, dir, agent_enum, prompt)
            .await?;
        println!("Agent: {}", agent_enum.display_name());
    } else {
        let default_agent = omega_core::agents::Agent::from_name(&config.agent_command)
            .unwrap_or(omega_core::agents::Agent::Codex);
        let _session = mgr
            .create_session_with_agent(name, dir, default_agent, prompt)
            .await?;
        println!("Agent: {} (OmegaOS default)", default_agent.display_name());
    }

    println!("Created session: {}", name);
    if let Some(ref files) = files {
        println!("  Scope claimed: {}", files.join(", "));
    }
    Ok(())
}

/// Bootstrap a new project via the workflow-driven /omega-new-project pipeline.
/// Mirrors the TUI `Action::CreateProject` path: resolve the category dir from
/// config (never a hardcoded ~/VibeCoding), create it, and spawn a Codex
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
    if resume {
        flags.push_str(" --resume");
    }
    if let Some(f) = from {
        flags.push_str(&format!(" --from={}", f));
    }
    if let Some(s) = skip {
        flags.push_str(&format!(" --skip={}", s));
    }
    if let Some(b) = budget {
        flags.push_str(&format!(" --budget={}", b));
    }
    if build {
        flags.push_str(" --build");
    }
    if dry_run {
        flags.push_str(" --dry-run");
    }

    let prompt = format!(
        "/omega-new-project {} {} {} {}{}",
        stack, category, name, group, flags
    );

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
    let agent = omega_core::agents::Agent::Codex;
    mgr.create_session_with_agent(&session, project_dir.to_str(), agent, Some(&prompt))
        .await?;
    println!(
        "New project '{}' ({}/{}) — bootstrap running in session '{}'",
        name, stack, category, session
    );
    println!("  dir: {}", project_dir.display());
    Ok(())
}

async fn cmd_install_bindings() -> Result<()> {
    // Option+Z / Option+/ have been REMOVED — they didn't toggle (popup spawned
    // a nested omega instead of returning to the main one). Use Tab-Tab in the
    // TUI for fullscreen and Ctrl+Space / prefix+z for popup entry.
    let popup_cmd = "display-popup -E -w 100% -h 100% \"omega menu\"";

    // Root-table bindings (no prefix required) — single reliable shortcut
    let root_bindings: Vec<(&str, &str)> = vec![("C-Space", "Open OmegaOS menu (Ctrl+Space)")];

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
            Ok(o) => failed.push(format!(
                "{}: {}",
                key,
                String::from_utf8_lossy(&o.stderr).trim()
            )),
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
            Ok(o) => failed.push(format!(
                "C-b {}: {}",
                key,
                String::from_utf8_lossy(&o.stderr).trim()
            )),
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

    // The persistent config: PREFER THE SHIPPED FILE, and never clobber a
    // richer one.
    //
    // This used to unconditionally overwrite ~/.omega/rmux.conf.omega with the
    // small inline stub below, "to keep in sync with this binary". That was
    // destructive, and on the documented path: install.sh copies the real
    // config/rmux.conf.omega (mouse on, forced terminal-features, 500k history,
    // the Omega chrome, the alternate-scroll fallback and the non-modal scroll
    // key table) and then prints "run 'omega install-bindings' to activate" —
    // so the activation step deleted everything it was supposed to activate.
    // A fresh user ended up with three popup bindings and no working mouse.
    //
    // Order now: the repo checkout's config wins; otherwise an existing file is
    // left untouched; the stub is only a last resort for a machine with neither.
    // Then source the result so "activate" really activates it.
    let omega_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".omega");
    std::fs::create_dir_all(&omega_dir)?;
    let conf_path = omega_dir.join("rmux.conf.omega");
    let shipped = resolve_omega_src().map(|src| src.join("config/rmux.conf.omega"));
    let mut wrote_shipped = false;
    if let Some(shipped) = shipped.filter(|p| p.is_file()) {
        // Same file? Nothing to do — do not rewrite it just to touch the mtime.
        let same = std::fs::read(&shipped).ok() == std::fs::read(&conf_path).ok();
        if !same {
            std::fs::copy(&shipped, &conf_path)?;
            println!("[+] Shipped rmux config installed → {}", conf_path.display());
        } else {
            println!("[+] rmux config already current → {}", conf_path.display());
        }
        wrote_shipped = true;
    } else if conf_path.is_file() {
        println!(
            "[i] Keeping existing {} (repo checkout not found — refusing to overwrite it)",
            conf_path.display()
        );
        wrote_shipped = true;
    }
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
    if !wrote_shipped {
        std::fs::write(&conf_path, content)?;
        println!(
            "[+] Minimal fallback config written to {} (no repo checkout found)",
            conf_path.display()
        );
    }
    // Apply it live — the whole point of "activate". Best-effort: a machine with
    // no running rmux server just gets it on the next session.
    let _ = std::process::Command::new("rmux")
        .arg("source-file")
        .arg(&conf_path)
        .output();

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
        println!(
            "\n[+] {} is now installed and on PATH.",
            agent.display_name()
        );
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
    // Canonicalize so "." / relative paths land on the same key the agents use.
    // Codex keys its trust table by the canonical path (on macOS /tmp/x is
    // stored as /private/tmp/x), so this is load-bearing, not cosmetic.
    let dir = dir.canonicalize().unwrap_or(dir);
    match omega_core::claude_trust::trust_dir(&dir) {
        Ok(true) => println!("trusted (claude): {}", dir.display()),
        Ok(false) => println!("already trusted (claude): {}", dir.display()),
        Err(e) => println!("trust-dir skipped, claude ({}): {}", dir.display(), e),
    }
    // Codex has the same blocking trust prompt in a different store — one
    // command trusts the folder for both agents (see codex_trust.rs).
    match omega_core::codex_trust::trust_dir(&dir) {
        Ok(true) => println!("trusted (codex): {}", dir.display()),
        Ok(false) => println!("already trusted (codex): {}", dir.display()),
        Err(e) => println!("trust-dir skipped, codex ({}): {}", dir.display(), e),
    }
    Ok(())
}

fn cmd_projects(json: bool) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home"));
    let projects = omega_core::projects::discover(&home);

    if json {
        println!("{}", serde_json::to_string_pretty(&projects)?);
        return Ok(());
    }

    if projects.is_empty() {
        println!("No projects discovered under {}", home.display());
        println!("Tip: a project is any directory with a .git or a build manifest");
        println!("(package.json, Cargo.toml, pyproject.toml, go.mod, …) up to 5 levels deep.");
        return Ok(());
    }

    println!("Discovered {} project(s), best first:\n", projects.len());
    for p in &projects {
        let stack = if p.stack.is_empty() {
            String::new()
        } else {
            format!("  [{}]", p.stack.join(", "))
        };
        let age = match p.last_active_days {
            Some(0) => "  · today".to_string(),
            Some(d) => format!("  · {}d", d),
            None => String::new(),
        };
        println!("  {}  ({}){}{}", p.name, p.container, stack, age);
    }
    Ok(())
}

#[derive(Subcommand)]
enum MarketingAction {
    /// List marketing-enabled projects and their status (content ✓, calendar
    /// posts, daily-engine on/off). Add `--json` for the machine feed that
    /// Telegram / Nova consume.
    List {
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
        /// Also fetch connected-account counts (calls omega-zernio; slower).
        #[arg(long)]
        accounts: bool,
    },
    /// Print the capabilities registry (capabilities.toml) grouped, with run
    /// commands. THE anti-forgetting command — everything the machine can do.
    Capabilities {
        /// Filter to a single group id (e.g. visual-image, publishing).
        #[arg(long)]
        group: Option<String>,
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Per-project layer + capability status: which groups are built / missing
    /// for this project (cross-references the registry against the filesystem).
    Status {
        /// Project name or slug (case-insensitive).
        project: String,
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
    },
    /// The single next-best action for a project + the exact command to run
    /// (deterministic rules over the project's marketing state).
    Next {
        /// Project name or slug (case-insensitive).
        project: String,
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Check integration keys/tools present + working (zernio, higgsfield,
    /// HeyGen, ElevenLabs, bun, ffmpeg) — OK/missing per dependency.
    Doctor {
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
    },
}

fn cmd_marketing(action: MarketingAction) -> Result<()> {
    match action {
        MarketingAction::List { json, accounts } => {
            let mut projects = omega_core::marketing::list_marketing_projects();
            if accounts {
                for p in projects.iter_mut() {
                    p.accounts = omega_core::marketing::project_accounts(&p.slug);
                }
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&projects)?);
                return Ok(());
            }

            if projects.is_empty() {
                println!("No marketing-enabled projects found.");
                println!("A project is marketing-enabled once it has a marketing/ directory");
                println!("(set OMEGA_STATION_DIR to scan a different projects root).");
                return Ok(());
            }

            println!("Marketing projects ({}):\n", projects.len());
            for p in &projects {
                let posts = if p.calendar_posts > 0 {
                    format!("  · {} posts", p.calendar_posts)
                } else if p.has_content {
                    "  · calendar (0 posts)".to_string()
                } else {
                    "  · no calendar".to_string()
                };
                let engine = if p.engine_on { "  · engine ON" } else { "" };
                let accts = match p.accounts {
                    Some(n) => format!("  · {} accounts", n),
                    None => String::new(),
                };
                println!("  {} {}{}{}{}", p.glyph(), p.name, posts, engine, accts);
                let ck = |b: bool| if b { "✓" } else { "·" };
                println!(
                    "      context {}  strategy {}  copy {}  visual {}  branding {}",
                    ck(p.has_context),
                    ck(p.has_strategy),
                    ck(p.has_copy),
                    ck(p.has_visual),
                    ck(p.has_branding)
                );
            }
            Ok(())
        }
        MarketingAction::Capabilities { group, json } => cmd_marketing_capabilities(group, json),
        MarketingAction::Status { project, json } => cmd_marketing_status(&project, json),
        MarketingAction::Next { project, json } => cmd_marketing_next(&project, json),
        MarketingAction::Doctor { json } => cmd_marketing_doctor(json),
    }
}

/// Resolve a project by name or slug (case-insensitive), fetching accounts so
/// next-best-action can reason about connectivity.
fn find_marketing_project(query: &str) -> Option<omega_core::marketing::MarketingProject> {
    let q = query.to_lowercase();
    let projects = omega_core::marketing::list_marketing_projects();
    projects
        .into_iter()
        .find(|p| p.name.to_lowercase() == q || p.slug.to_lowercase() == q)
}

fn cmd_marketing_capabilities(group: Option<String>, json: bool) -> Result<()> {
    let reg = match omega_core::marketing::load_capabilities()? {
        Some(r) => r,
        None => {
            if json {
                println!("null");
            } else {
                println!("No capabilities.toml found.");
                println!(
                    "Expected at tools/marketing-machine/capabilities.toml in the OmegaOS repo"
                );
                println!("(override with OMEGA_MKT_CAPS=/path/to/capabilities.toml).");
            }
            return Ok(());
        }
    };

    if json {
        // Optionally narrow to a group.
        if let Some(gid) = group.as_deref() {
            let caps: Vec<_> = reg.in_group(gid).into_iter().cloned().collect();
            println!("{}", serde_json::to_string_pretty(&caps)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&reg)?);
        }
        return Ok(());
    }

    let (built, partial, missing) = reg.status_counts();
    println!(
        "Marketing Machine — capabilities registry ({} capabilities: {} built · {} partial · {} missing)\n",
        reg.capabilities.len(),
        built,
        partial,
        missing
    );

    let glyph = |s: &str| match s {
        "built" => "🟢",
        "partial" => "🟡",
        "missing" => "🔴",
        _ => "⚪",
    };

    for grp in reg.groups_ordered() {
        if let Some(ref only) = group {
            if &grp.id != only {
                continue;
            }
        }
        let caps = reg.in_group(&grp.id);
        if caps.is_empty() {
            continue;
        }
        println!("── {} ({}) ──", grp.name, grp.id);
        for c in caps {
            let paid = if c.paid { " 💲" } else { "" };
            println!("  {} {:<6} {}{}", glyph(&c.status), c.id, c.name, paid);
            if !c.does.is_empty() {
                println!("        {}", c.does);
            }
            if !c.run.is_empty() {
                println!("        run: {}", c.run);
            }
        }
        println!();
    }

    if let Some(only) = group {
        if reg.in_group(&only).is_empty() {
            println!("(no capabilities in group '{}')", only);
        }
    }
    Ok(())
}

fn cmd_marketing_status(project: &str, json: bool) -> Result<()> {
    let mut p = match find_marketing_project(project) {
        Some(p) => p,
        None => {
            if json {
                println!("null");
            } else {
                println!("No marketing project matches '{}'.", project);
                println!("Run `omega marketing list` to see available projects.");
            }
            return Ok(());
        }
    };
    // Accounts inform the calendar/publishing view.
    p.accounts = omega_core::marketing::project_accounts(&p.slug);
    let groups = omega_core::marketing::project_group_status(&p);

    if json {
        let out = serde_json::json!({
            "project": p.name,
            "slug": p.slug,
            "path": p.path,
            "calendarPosts": p.calendar_posts,
            "engineOn": p.engine_on,
            "accounts": p.accounts,
            "groups": groups,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{} {}  ({})", p.glyph(), p.name, p.slug);
    println!("  path: {}", p.path.display());
    let accts = match p.accounts {
        Some(n) => n.to_string(),
        None => "unknown".to_string(),
    };
    println!(
        "  calendar posts: {}   engine: {}   connected accounts: {}\n",
        p.calendar_posts,
        if p.engine_on { "ON" } else { "off" },
        accts
    );
    println!("  Per-group status (against the capabilities registry):");
    for g in &groups {
        let mark = if g.present { "✓" } else { "✗" };
        println!("    {} {:<22} {}", mark, g.name, g.detail);
    }
    Ok(())
}

fn cmd_marketing_next(project: &str, json: bool) -> Result<()> {
    let mut p = match find_marketing_project(project) {
        Some(p) => p,
        None => {
            if json {
                println!("null");
            } else {
                println!("No marketing project matches '{}'.", project);
                println!("Run `omega marketing list` to see available projects.");
            }
            return Ok(());
        }
    };
    p.accounts = omega_core::marketing::project_accounts(&p.slug);
    let (id, why, cmd) = omega_core::marketing::next_best_action(&p);

    if json {
        let out = serde_json::json!({
            "project": p.name,
            "slug": p.slug,
            "nextBestAction": { "id": id, "why": why, "command": cmd },
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{} {} — next best action:\n", p.glyph(), p.name);
    println!("  ▸ {}", id);
    println!("    why: {}", why);
    println!("    run: {}", cmd);
    Ok(())
}

fn cmd_marketing_doctor(json: bool) -> Result<()> {
    use std::process::Command;

    // (name, ok, detail)
    let mut checks: Vec<(String, bool, String)> = Vec::new();

    // Read integrations.env once (values never printed — presence only).
    let env_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home"))
        .join(".omega")
        .join("secrets")
        .join("integrations.env");
    let env_raw = std::fs::read_to_string(&env_path).unwrap_or_default();
    let key_set = |name: &str| -> bool {
        env_raw.lines().any(|l| {
            let l = l.trim();
            if let Some(rest) = l.strip_prefix(name) {
                rest.starts_with('=') && rest.len() > 1 && !rest.trim_end().ends_with('=')
            } else {
                false
            }
        }) || std::env::var(name).map(|v| !v.is_empty()).unwrap_or(false)
    };
    let bin_present = |bin: &str| -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {}", bin))
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    // --- Binaries / tools ---
    checks.push((
        "bun".into(),
        bin_present("bun"),
        "video/engine runtime".into(),
    ));
    checks.push((
        "ffmpeg".into(),
        bin_present("ffmpeg"),
        "video mux + audio ducking".into(),
    ));
    let hf = bin_present("higgsfield");
    checks.push((
        "higgsfield CLI".into(),
        hf,
        "image/video/soul engine".into(),
    ));

    // --- zernio key ---
    checks.push((
        "ZERNIO_API_KEY".into(),
        key_set("ZERNIO_API_KEY"),
        "publishing to 15+ networks".into(),
    ));

    // --- higgsfield account status (live if the CLI + keys are present) ---
    let hf_keys = key_set("HIGGSFIELD_API_KEY_ID") && key_set("HIGGSFIELD_API_KEY_SECRET");
    checks.push((
        "HIGGSFIELD_API_KEY_ID/SECRET".into(),
        hf_keys,
        "higgsfield API credentials".into(),
    ));
    if hf && hf_keys {
        // Best-effort live check: `higgsfield account` — bounded, non-fatal.
        let ok = Command::new("higgsfield")
            .arg("account")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        checks.push((
            "higgsfield account".into(),
            ok,
            if ok {
                "credits/account reachable".into()
            } else {
                "CLI present but account call failed (login/credits?)".into()
            },
        ));
    }

    // --- HeyGen / ElevenLabs / ARTLIST / Tella ---
    checks.push((
        "HEYGEN_API_KEY".into(),
        key_set("HEYGEN_API_KEY"),
        "talking-head UGC avatars".into(),
    ));
    checks.push((
        "ELEVENLABS_API_KEY".into(),
        key_set("ELEVENLABS_API_KEY"),
        "branded VO + music beds".into(),
    ));
    checks.push((
        "ARTLIST_API_KEY".into(),
        key_set("ARTLIST_API_KEY"),
        "licensed music (no runner yet)".into(),
    ));
    checks.push((
        "TELLA_API_KEY".into(),
        key_set("TELLA_API_KEY"),
        "screen recording (no runner yet)".into(),
    ));

    if json {
        let arr: Vec<_> = checks
            .iter()
            .map(|(n, ok, d)| serde_json::json!({ "name": n, "ok": ok, "detail": d }))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "checks": arr }))?
        );
        return Ok(());
    }

    println!("Marketing Machine — doctor (integration keys/tools)\n");
    let mut missing = 0;
    for (name, ok, detail) in &checks {
        if *ok {
            println!("  ✓ {:<32} {}", name, detail);
        } else {
            missing += 1;
            println!("  ✗ {:<32} MISSING — {}", name, detail);
        }
    }
    println!();
    if missing == 0 {
        println!("All checked dependencies present.");
    } else {
        println!(
            "{} dependency/dependencies missing (keys live in ~/.omega/secrets/integrations.env).",
            missing
        );
    }
    Ok(())
}

/// The graph-driver surface. Thin: every decision belongs to
/// `omega_core::graph_executor` and `graph_risk`, this only runs what they
/// authorize and reports what happened.
#[derive(Subcommand)]
enum GraphAction {
    /// Drive the graph to a terminal outcome.
    Run {
        /// Path to the graph JSON.
        graph: String,
        /// Where the run state lives, so an interrupted run RESUMES instead of
        /// restarting. Defaults to `<graph>.state.json` beside the graph.
        #[arg(long)]
        state: Option<String>,
        /// Evaluate as an UNATTENDED run (nobody is watching), the mode a
        /// dispatched oracle or worker runs in. Default is attended.
        #[arg(long)]
        unattended: bool,
        /// Gate and print what WOULD run now, execute nothing, advance nothing.
        #[arg(long)]
        dry_run: bool,
        /// Backstop for a driver bug, never a substitute for the graph's own
        /// bounds, which already guarantee termination (R-LOOP).
        #[arg(long, default_value_t = 1000)]
        max_steps: usize,
    },
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
        /// Preview the block an agent would ACTUALLY receive for this mission:
        /// Laws and universal rules in full, domain rules indexed unless the
        /// mission text mentions their topic. Omit to print the full block.
        #[arg(long)]
        mission: Option<String>,
    },
}

#[derive(Subcommand)]
enum SkillsAction {
    /// Validate an owned skill root and print its deterministic catalog digest
    Validate {
        /// Explicit OmegaOS-owned skill root (defaults to ./skills, $OMEGA_SRC/skills, or ~/.omega/skills)
        #[arg(long)]
        root: Option<String>,
    },
    /// Compile the canonical JSON consumed by Atlas, RAG, and provider adapters
    Compile {
        /// Explicit OmegaOS-owned skill root
        #[arg(long)]
        root: Option<String>,
        /// Output JSON (defaults to ~/.omega/skill-catalog-v1.json)
        #[arg(long)]
        out: Option<String>,
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

/// The risk-gate surface. Every variant reads a graph document, asks
/// `omega_core::graph_risk` a question, and prints the answer: no judgement of
/// its own, so the CLI can never disagree with the module the executor consults.
#[derive(Subcommand)]
enum RiskGateAction {
    /// Show what the gate says about one node.
    Show {
        /// Path to the graph JSON.
        graph: String,
        /// Node id to evaluate.
        node: String,
        /// Path to the run-state JSON, where recorded approvals live. Omitted,
        /// a fresh state is seeded from the graph — i.e. no human decision on
        /// record, which is what makes the gate ASK rather than assume.
        #[arg(long)]
        state: Option<String>,
        /// Evaluate as an UNATTENDED run (nobody is watching), the mode a
        /// dispatched oracle or worker runs in. Default is attended.
        #[arg(long)]
        unattended: bool,
    },
    /// Record an attributed human APPROVAL for an escalated node into the run
    /// state, where the next `show` (and the executor) will honour it.
    Approve {
        /// Path to the graph JSON.
        graph: String,
        /// Node id to approve.
        node: String,
        /// WHO approves. An unattributed approval is refused by the core: an
        /// approval nobody signed is indistinguishable from one the process
        /// invented for itself.
        #[arg(long)]
        approver: String,
        /// Path to the run-state JSON to update. Created if absent.
        #[arg(long)]
        state: Option<String>,
    },
    /// Record an attributed human DENIAL, held to the same attribution standard:
    /// an operator reading the run later needs to know who blocked it.
    Deny {
        /// Path to the graph JSON.
        graph: String,
        /// Node id to deny.
        node: String,
        /// WHO denies.
        #[arg(long)]
        approver: String,
        /// Path to the run-state JSON to update. Created if absent.
        #[arg(long)]
        state: Option<String>,
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
        /// Bot token from @BotFather. May be OMITTED when the OMEGA_TG_TOKEN
        /// env var carries it (keeps the secret out of `ps`/argv — how the
        /// npm wizard invokes us); the first positional is then the chat id.
        /// A plain `#[arg(env)]` can't express that: clap fills positionals
        /// by argv index, so the lone <chat_id> would land in this slot —
        /// both positionals are Options and cmd_telegram shifts them.
        /// `allow_negative_numbers` so a leading negative chat id parses.
        #[arg(value_name = "BOT_TOKEN", allow_negative_numbers = true)]
        bot_token: Option<String>,
        /// Telegram chat id. Groups/supergroups have NEGATIVE ids
        /// (e.g. -1001234567) — accepted as-is, no quoting needed.
        #[arg(allow_negative_numbers = true)]
        chat_id: Option<i64>,
        /// Optional Telegram sender user_ids allowed to talk to the bot.
        /// When set, every message MUST come from one of these users; others
        /// are silently dropped. Recommended for shared chats.
        /// `allow_hyphen_values` (not `allow_negative_numbers`): a comma
        /// list with a leading negative id ("-100123,42") is not
        /// syntactically a number, so the negative-number exemption alone
        /// still rejects it (runtime-proven).
        #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
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
        TelegramAction::Setup {
            bot_token,
            chat_id,
            user_id,
            relay_session,
            label,
        } => {
            // fix7-T1: resolve the token/chat pair. Two accepted shapes:
            //   omega telegram setup <bot_token> <chat_id> …             (classic)
            //   OMEGA_TG_TOKEN=<tok> omega telegram setup <chat_id> …    (token off argv)
            // In the env form the single positional lands in `bot_token`
            // (clap assigns by index) — shift it into chat_id here.
            let env_token = std::env::var("OMEGA_TG_TOKEN")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let (bot_token, chat_id) = match (bot_token, chat_id, env_token) {
                // classic two-positional form — an explicit argv token wins over the env var
                (Some(tok), Some(id), _) => (tok, id),
                // env form: the one positional is the chat id
                (Some(first), None, Some(tok)) => {
                    let id = first.parse::<i64>().with_context(|| {
                        format!(
                            "OMEGA_TG_TOKEN is set, so the first positional must be the <chat_id> (got '{}')",
                            first
                        )
                    })?;
                    (tok, id)
                }
                (Some(_), None, None) => anyhow::bail!(
                    "missing <chat_id> — usage: OMEGA_TG_TOKEN=<token> omega telegram setup <chat_id>, \
                     or legacy: omega telegram setup <bot_token> <chat_id>"
                ),
                (None, _, Some(_)) => anyhow::bail!(
                    "missing <chat_id> — usage: OMEGA_TG_TOKEN=<token> omega telegram setup <chat_id>"
                ),
                (None, _, None) => anyhow::bail!(
                    "missing <bot_token> <chat_id> — pass both positionally, or set \
                     OMEGA_TG_TOKEN (keeps the token out of the process list) and pass only <chat_id>"
                ),
            };
            // The Bun bot REFUSES to serve with an empty allow-list (it controls
            // the whole machine — omega-tg-bot.ts hard-exits on bot_token set +
            // allow_user_ids empty). A private chat id IS the operator's user id,
            // so default the allow-list to it instead of writing a config the
            // bot will reject; only group ids (negative) can't be defaulted.
            let allow_user_ids = if user_id.is_empty() && chat_id > 0 {
                println!(
                    "[i] --user-id not given — allow-list defaulted to your chat id ({chat_id})"
                );
                vec![chat_id]
            } else {
                user_id
            };
            if allow_user_ids.is_empty() {
                println!(
                    "[!] No sender allow-list (group chat id {chat_id}): the bot will REFUSE to \
                     serve until you re-run setup with --user-id <your_id> (it controls this machine)"
                );
            }
            let cfg = OmegaTelegramConfig {
                bot_token,
                chat_id,
                allow_user_ids,
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
                println!("  Sender filter: NONE — bot refuses to serve until --user-id is set");
            } else {
                println!(
                    "  Sender filter: only user_ids {:?} accepted",
                    cfg.allow_user_ids
                );
            }
            // One poller per token (Telegram 409): when the installed service is
            // already polling, it re-reads telegram.toml within ~5s — telling the
            // user to ALSO `omega telegram run` would start a conflicting second
            // poller. Only suggest the foreground run when no service is up.
            match omega_core::service::tg_bot_status() {
                Some(s) if s == "active" => {
                    println!("\nThe bot service is running — it picks up this config within ~5s. Just message your bot.");
                }
                Some(other) => {
                    println!(
                        "\nBot service installed but {} — start it:  {}",
                        other,
                        omega_core::service::tg_bot_start_hint()
                    );
                }
                None => println!("\nRun the bot with:  omega telegram run"),
            }
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
                    println!("Run: OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <CHAT_ID> --user-id <CHAT_ID>");
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
            // IS Atlas, the 15 agents live in its system prompt, and it
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
            println!("◆ Launching OmegaOS Telegram bot (Bun) — Atlas + 15 agents");
            // exec() replaces this process; it only returns on failure.
            let err = std::process::Command::new("bun").arg(&bot_ts).exec();
            anyhow::bail!(
                "Failed to launch the Bun Telegram bot ({err}). Is `bun` installed and on PATH?"
            );
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
            // `auto_update` lives in config.toml (OmegaConfig), not in the
            // providers table every other key here belongs to. Intercepted so
            // the one command users are told about actually works.
            if key == "auto_update" {
                let cfg = omega_core::config::OmegaConfig::load().unwrap_or_default();
                println!("{}", cfg.auto_update.as_str());
                return Ok(());
            }
            let value = get_config_value(&cfg, &key)?;
            println!("{}", value);
        }
        ConfigAction::Set { key, value } => {
            if key == "auto_update" {
                use omega_core::config::{AutoUpdatePolicy, OmegaConfig};
                let policy = AutoUpdatePolicy::parse(&value);
                OmegaConfig::set_auto_update(policy)?;
                println!("[+] Set auto_update = {}", policy.as_str());
                println!(
                    "{}",
                    match policy {
                        AutoUpdatePolicy::Apply =>
                            "The daily 03:30 check will install updates automatically.",
                        AutoUpdatePolicy::Check =>
                            "The daily 03:30 check will alert you, and install nothing.",
                        AutoUpdatePolicy::Off => "The daily check will do nothing.",
                    }
                );
                return Ok(());
            }
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
        ("claude", "dangerously_skip_permissions") => {
            cfg.claude.dangerously_skip_permissions.to_string()
        }
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
    let snap = monitor::UsageSnapshot::read()
        .ok()
        .flatten()
        .unwrap_or_default();
    let accounts = monitor::list_accounts();
    let bot = monitor::aisb_bot_status();
    let tg = monitor::OmegaTelegramConfig::read();

    println!("─── Billing ───");
    println!(
        "  5h session:  {:.1}%  ({}/{})",
        snap.precise_5h(),
        snap.tokens_5h,
        snap.budget_5h
    );
    println!(
        "  Week:        {:.1}%  ({}/{})",
        snap.precise_week(),
        snap.tokens_7d,
        snap.budget_week
    );
    println!("  Account:     {} ({})", snap.active_account, snap.email);
    println!();
    println!("─── AISB Bot ───");
    println!("  Running:     {}", bot.bot_alive);
    println!("  Cache:       {:?}", bot.cache_status);
    println!();
    println!("─── Accounts ({}) ───", accounts.len());
    for acc in &accounts {
        let marker = if acc.is_active { "▶" } else { " " };
        println!(
            "  {} {}  {}",
            marker,
            acc.label,
            acc.email.as_deref().unwrap_or("")
        );
    }
    println!();
    println!("─── Omega Telegram ───");
    match tg {
        Some(c) => println!("  Configured: yes (enabled={}, relay={})", c.enabled, c.relay_session),
        None => println!("  Not configured. Run: OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <CHAT_ID> --user-id <CHAT_ID>"),
    }
    Ok(())
}

async fn cmd_master() -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;

    let agent = omega_core::agents::Agent::from_name(&config.aisb_agent)
        .unwrap_or(omega_core::agents::Agent::Codex);
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

/// `omega mouse-test` — ground-truth probe of the WHOLE input chain from the
/// terminal emulator down to this process (the same crossterm parser the TUI
/// uses). Recurring operator report "scroll ne marche pas" kept being blamed
/// on rmux/the TUI when the events never left the terminal (mosh swallows the
/// mouse handshake; some emulators translate wheel→arrows; Apple Terminal has
/// a View → Allow Mouse Reporting toggle). This makes the layer visible.
fn cmd_mouse_test() -> Result<()> {
    use crossterm::event::{Event, KeyCode, MouseEventKind};
    use std::io::Write;

    println!("◆ Mouse diagnostic — scroll the wheel / swipe, click, and drag IN THIS WINDOW.");
    println!("  Listening 10s (q to stop early)…\n");
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;

    let (mut wheel, mut click, mut drag, mut arrows, mut other_keys) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut out = std::io::stdout();
    while std::time::Instant::now() < deadline {
        if !crossterm::event::poll(std::time::Duration::from_millis(200))? {
            continue;
        }
        match crossterm::event::read()? {
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                    wheel += 1;
                    let _ = write!(out, "  ← wheel ({:?})\r\n", m.kind);
                }
                MouseEventKind::Down(b) => {
                    click += 1;
                    let _ = write!(out, "  ← click ({:?})\r\n", b);
                }
                MouseEventKind::Drag(_) => {
                    drag += 1;
                }
                _ => {}
            },
            Event::Key(k) => match k.code {
                KeyCode::Up | KeyCode::Down => {
                    arrows += 1;
                    let _ = write!(
                        out,
                        "  ← arrow key ({:?}) — wheel→arrow translation?\r\n",
                        k.code
                    );
                }
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c')
                    if k.modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break
                }
                _ => other_keys += 1,
            },
            _ => {}
        }
        let _ = out.flush();
    }

    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;

    println!("\n━━ Verdict ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  wheel events: {wheel} · clicks: {click} · drags: {drag} · arrow keys: {arrows} · other keys: {other_keys}");
    if wheel > 0 {
        println!("  ✓ Your terminal DOES send wheel events — mouse scroll works on this");
        println!("    connection. If the omega TUI still doesn't scroll, report that.");
    } else if click > 0 || drag > 0 {
        println!("  ◐ Clicks arrive but NO wheel events: your terminal doesn't report the");
        println!("    scroll wheel to apps. Use PgUp/PgDn (Mac: fn+↑ / fn+↓) in the TUI,");
        println!("    or a terminal with full mouse reporting (iTerm2, Ghostty, kitty).");
    } else if arrows > 0 {
        println!("  ◐ Your terminal translates the wheel into ARROW KEYS (alternate-screen");
        println!("    scroll). In the TUI those forward to the focused agent, not the view.");
        println!("    Use PgUp/PgDn (Mac: fn+↑ / fn+↓), or iTerm2/Ghostty/kitty for real");
        println!("    mouse reporting.");
    } else {
        println!("  ✗ NOTHING arrived: the mouse handshake never reached your terminal.");
        println!("    Typical causes: connected over mosh (it never forwards mouse modes —");
        println!("    use plain SSH), or mouse reporting disabled in the terminal");
        println!("    (Apple Terminal: View → Allow Mouse Reporting).");
    }
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
    println!("Junk sessions (rmux name ≠ sanitized slug — not created by omega):");
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
        let color = if agent.is_available() {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        };
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

/// What `omega kill` decided to do, resolved BEFORE anything is touched.
///
/// Splitting the decision from the execution is what makes the close-gate
/// testable at all: the alternative is a live rmux daemon plus a real oracle
/// with real workers, which is why the cascade was never covered and why the
/// scope-claim leak below survived so long.
#[derive(Debug, PartialEq, Eq)]
enum KillPlan {
    /// Nothing live under that name and nothing to cascade. A second `omega
    /// kill` must be a quiet exit 0, never `Session not found` — the stuck-oracle
    /// alert cron tells the operator to run `omega kill oracle-<key>`, and an
    /// operator who already ran it once was being handed a hard error for
    /// obeying the alert twice.
    AlreadyClosed,
    /// An oracle whose workers are still WORKING. Closing it strands them:
    /// they keep their scope claims, so the next `spawn-worker` on those files
    /// is rejected by a claim whose owner nobody can find any more.
    Refused { running: Vec<String> },
    /// Go ahead. `cascade` is killed first, the target last (the target may be
    /// the pane running this very command).
    Proceed { cascade: Vec<String> },
}

/// The close-gate, as a pure function of what is live.
///
/// `workers` is empty for a non-oracle target, which collapses this to
/// "proceed when live, already-closed when not" for a plain session.
fn decide_kill(
    target_live: bool,
    workers: &omega_core::oracle_lifecycle::LiveWorkers,
    force: bool,
) -> KillPlan {
    // Same gate, same reason, and deliberately the same wording as the
    // done_clean refusal in `cmd_done`: an oracle does not close while its
    // workers run. `omega kill` was the hole in that gate — it closed the
    // oracle unconditionally and left every worker alive with no parent.
    if !workers.running.is_empty() && !force {
        return KillPlan::Refused {
            running: workers.running.clone(),
        };
    }
    // Without --force only FINISHED workers cascade (they are what `cmd_done`
    // cascades too). With --force the running ones go down as well, because
    // the operator has now said so explicitly.
    let cascade = if force {
        workers.all()
    } else {
        workers.terminal.clone()
    };
    if !target_live && cascade.is_empty() {
        return KillPlan::AlreadyClosed;
    }
    KillPlan::Proceed { cascade }
}

/// Whether a worker's git worktree may be unregistered.
///
/// Losing a worker's commits is far worse than leaking a directory, so this
/// is deliberately conservative and every uncertain case keeps the worktree.
#[derive(Debug, PartialEq, Eq)]
enum WorktreeVerdict {
    /// Not an OmegaOS-created linked worktree. Never touched: an operator's
    /// own worktree, or a worker that ran in the shared checkout (spawn-worker
    /// falls back to the shared dir when `git worktree add` fails), and
    /// unregistering THAT would take the operator's own checkout down.
    NotOurs,
    /// Uncommitted or untracked files live in it. `git worktree remove` would
    /// need --force to delete them, which is exactly what must not happen here.
    Dirty,
    /// The branch carries commits the main worktree does not have yet, i.e.
    /// `omega-git-merge` never integrated it. Removing the worktree here would
    /// leave those commits reachable only from a branch nobody is looking at.
    Unmerged { commits: u32 },
    /// Registered, clean, and fully contained in the main worktree's HEAD.
    Removable,
}

/// Decide from already-gathered git output, so the rule is testable without a
/// repository: `git status --porcelain` (empty = clean) and the count from
/// `git rev-list --count <main-head>..HEAD`.
fn worktree_verdict(
    is_omega_linked_worktree: bool,
    porcelain_status: &str,
    unmerged_commits: u32,
) -> WorktreeVerdict {
    if !is_omega_linked_worktree {
        return WorktreeVerdict::NotOurs;
    }
    if !porcelain_status.trim().is_empty() {
        return WorktreeVerdict::Dirty;
    }
    if unmerged_commits > 0 {
        return WorktreeVerdict::Unmerged {
            commits: unmerged_commits,
        };
    }
    WorktreeVerdict::Removable
}

/// Run a git command and return its trimmed stdout, or None when git itself
/// failed. Every worktree probe below is advisory: an unreadable repository
/// must leave the worktree alone, never remove it on a guess.
fn git_probe(dir: &std::path::Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The branch slug `omega-git-branch.sh` derives from a worker's session name,
/// reproduced exactly: lowercase, spaces to `-`, drop everything outside
/// `[a-z0-9-]`, trim the edges. `_mk_branch` then builds
/// `omega/<slug>-<shortid>` and names the worktree directory after the branch
/// minus its `omega/` prefix, which is what makes a worker's tree findable
/// from its session name alone.
fn worker_branch_slug(session_name: &str) -> String {
    let s: String = session_name
        .chars()
        .map(|c| {
            if c == ' ' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    let trimmed = s.trim_matches('-');
    if trimmed.is_empty() {
        "worker".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Whether a worktree DIRECTORY name was generated for `slug`.
///
/// The tail after the slug must be a real `_mk_branch` suffix — an 8-hex
/// shortid, optionally plus the numeric `-1`, `-2`, … collision counter — and
/// not merely a longer worker name that happens to start the same way. Without
/// the shortid test, a worker `foo` would claim the tree of a worker
/// `foo-bar-1a2b3c4d`.
fn worktree_dir_belongs_to(dir_name: &str, slug: &str) -> bool {
    let is_short_id = |s: &str| s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit());
    let Some(tail) = dir_name
        .strip_prefix(slug)
        .and_then(|t| t.strip_prefix('-'))
    else {
        return false;
    };
    match tail.split_once('-') {
        None => is_short_id(tail),
        Some((id, n)) => is_short_id(id) && !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
    }
}

/// Every worktree under `$OMEGA_DIR/worktrees` that belongs to `worker_name`.
///
/// This exists because the session list CANNOT answer it: `list_sessions`
/// builds each entry with `OmegaSession::classify`, which leaves `working_dir`
/// at `None` and only fills `provider`, so reading a worker's directory off
/// its live session yields nothing and the whole cleanup below silently never
/// ran. The layout is walked instead, one level (the repo bucket
/// `worktrees/<repo>/`) and two, because both shapes exist on disk.
///
/// Over-matching is safe by construction here and under-matching is not: an
/// extra candidate still faces `worktree_verdict`, which keeps anything
/// carrying uncommitted or unmerged work, so the worst case is that a stale
/// already-merged tree is collected too. Missing the real one is what leaks.
fn worker_worktrees(omega_dir: &std::path::Path, worker_name: &str) -> Vec<std::path::PathBuf> {
    fn is_worker_worktree(p: &std::path::Path, slug: &str) -> bool {
        p.is_dir()
            && p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| worktree_dir_belongs_to(n, slug))
            // A LINKED worktree has a `.git` FILE; a main checkout has a `.git`
            // directory. Never hand a main checkout to the remover.
            && p.join(".git").is_file()
    }

    let slug = worker_branch_slug(worker_name);
    let mut found = Vec::new();
    let Ok(top) = std::fs::read_dir(omega_dir.join("worktrees")) else {
        return found;
    };
    for entry in top.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        if is_worker_worktree(&p, &slug) {
            found.push(p.clone());
        }
        if let Ok(inner) = std::fs::read_dir(&p) {
            for e in inner.flatten() {
                let q = e.path();
                if is_worker_worktree(&q, &slug) {
                    found.push(q);
                }
            }
        }
    }
    found
}

/// Unregister the git worktree a `--worktree` worker ran in, when and only
/// when it holds nothing the operator could still want.
///
/// The candidate directory comes from `worker_worktrees`, but the MAIN
/// worktree it is measured against is always resolved from `git worktree list
/// --porcelain` rather than assumed: `omega-git-branch.sh` suffixes the branch
/// (against existing refs) and the directory (against the filesystem)
/// INDEPENDENTLY, so the two can disagree and nothing here may depend on them
/// matching. Both guards below are re-checked on the real path before anything
/// is removed.
///
/// Prints one line per worktree it looked at, because the whole point of the
/// keep decision is that the operator learns where the unrecovered work is.
fn cleanup_worker_worktree(omega_dir: &std::path::Path, work_dir: &std::path::Path) {
    // A LINKED worktree has a `.git` FILE (`gitdir: …/.git/worktrees/<name>`);
    // the main checkout has a `.git` directory. That single test is what keeps
    // this off the operator's own repository.
    if !work_dir.join(".git").is_file() {
        return;
    }
    // Second guard: only trees OmegaOS itself created, under
    // `$OMEGA_DIR/worktrees/<repo-basename>/`. A linked worktree the operator
    // made by hand elsewhere is none of our business.
    if !work_dir.starts_with(omega_dir.join("worktrees")) {
        return;
    }

    // The first `worktree ` line of the porcelain listing is the MAIN
    // worktree — that is the tree the operator actually works in and therefore
    // the one a worker's commits have to be merged into to count as saved.
    // Parse the value as everything after "worktree ", never a whitespace
    // split: a path containing a space would otherwise be truncated, and a
    // truncated path is what gets handed to a remove command.
    let listing = match git_probe(work_dir, &["worktree", "list", "--porcelain"]) {
        Some(l) => l,
        None => return,
    };
    let (mut main_top, mut main_head) = (None, None);
    for line in listing.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            if main_top.is_none() {
                main_top = Some(p.to_string());
            }
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            if main_head.is_none() {
                main_head = Some(h.to_string());
            }
        }
    }
    let (Some(main_top), Some(main_head)) = (main_top, main_head) else {
        return;
    };

    let status = git_probe(work_dir, &["status", "--porcelain"]).unwrap_or_else(|| "?".to_string());
    // `<main-head>..HEAD` counts the commits this branch has that the main
    // worktree does not. Zero means `omega-git-merge` already integrated it.
    // An unreadable count is treated as "has work" (u32::MAX below would be a
    // lie about the number, so fall back to 1 and say so as Unmerged).
    let ahead = git_probe(
        work_dir,
        &["rev-list", "--count", &format!("{main_head}..HEAD")],
    )
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(1);
    let verdict = worktree_verdict(true, &status, ahead);
    let wt = work_dir.display();
    match verdict {
        WorktreeVerdict::Removable => {
            // No --force: git's own refusal on a dirty tree is kept as a second
            // net behind our own check. Run it from the MAIN worktree, since the
            // tree being removed is the one we would otherwise be standing in.
            let removed = git_probe(
                std::path::Path::new(&main_top),
                &["worktree", "remove", &work_dir.to_string_lossy()],
            )
            .is_some();
            // prune only unregisters worktrees whose directory is already gone,
            // so it is safe whether or not the remove above succeeded.
            let _ = git_probe(std::path::Path::new(&main_top), &["worktree", "prune"]);
            if removed {
                println!("  worktree removed: {wt}");
            } else {
                println!("  worktree KEPT (git refused to remove it): {wt}");
            }
        }
        WorktreeVerdict::Dirty => {
            println!("  worktree KEPT — uncommitted work still in it: {wt}");
        }
        WorktreeVerdict::Unmerged { commits } => {
            println!("  worktree KEPT — {commits} commit(s) not merged into the main tree: {wt}");
        }
        WorktreeVerdict::NotOurs => {}
    }
}

/// Drop the lifecycle state of a closed oracle.
///
/// Without this, patrol resurrects the session the operator just killed
/// (workers non-terminal + phase < 24h) and the stuck-alert cron keeps
/// watching a ghost — and `omega kill` is the close action that very alert
/// tells the operator to run. Idempotent: everything here is a remove that
/// tolerates an already-absent file.
fn clear_oracle_state(state_dir: &std::path::Path, name: &str) {
    let _ = omega_core::oracle_lifecycle::OracleState::remove(state_dir, name);
    let key = name.strip_prefix("oracle-").unwrap_or(name);
    for f in [
        format!("oracle-{key}.progress.json"),
        format!("oracle-{key}.stuck-alerted"),
        // Resurrect markers are stamped with the FULL session name after
        // an `oracle-` prefix (giving oracle-oracle-X) — remove both forms.
        format!("oracle-{key}.resurrect-attempt"),
        format!("oracle-{name}.resurrect-attempt"),
    ] {
        let _ = std::fs::remove_file(state_dir.join(f));
    }
}

/// `omega kill <session> [--force]` — a controlled mission closure.
///
/// Before this, killing an oracle killed exactly one pane and released exactly
/// one scope claim. Its workers stayed alive with no parent, and their claims
/// stayed on disk forever: the next `spawn-worker` touching those files was
/// then rejected by `claim_or_reject` against an owner whose session no longer
/// existed. That recurring failure is what this closure exists to end.
///
/// Every step is INDEPENDENT and best-effort. A pane that will not die must
/// not skip the scope release, and a git probe that fails must not skip the
/// state cleanup: a half-run closure leaks exactly the thing being fixed.
async fn cmd_kill(name: &str, force: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let omega_dir = config
        .state_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let mgr = SessionManager::connect().await?;
    // Snapshot the live sessions ONCE, before anything is killed: the working
    // dir recorded here is the only handle on a worker's worktree, and it is
    // gone the moment the session is.
    let live = mgr.list_sessions().await.unwrap_or_default();

    // Resolve the mission-key spelling FIRST. `omega kill dentistrygpt-3` used to
    // classify as a plain session, find nothing live, take the `AlreadyClosed`
    // branch and print a success line while closing nothing — the worst shape a
    // close command can have, because the operator walks away believing the
    // mission is shut and it is still running.
    let resolved = resolve_oracle_alias(
        name,
        &live.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
        &config.state_dir,
    );
    if resolved != name {
        println!("[i] {name} resolved to the oracle session {resolved}");
    }
    let name: &str = &resolved;
    let target_live = live.iter().any(|s| s.name == name);

    let is_oracle = omega_core::session::OmegaSession::classify(name).role
        == omega_core::session::SessionRole::Oracle;
    let workers = if is_oracle {
        omega_core::oracle_lifecycle::live_workers_of_oracle(&config.state_dir, name, &live)
    } else {
        omega_core::oracle_lifecycle::LiveWorkers::default()
    };

    let cascade = match decide_kill(target_live, &workers, force) {
        KillPlan::Refused { running } => {
            // Same shape as the done_clean refusal: name them, say why, and
            // give the two ways forward. Naming them matters — "some workers
            // are running" sends the operator hunting through `omega list`.
            anyhow::bail!(
                "kill REFUSED — {} worker(s) of this oracle are still running: {}.\n\
                 Closing the oracle now would strand them: they keep their scope claims, \
                 and the next `omega spawn-worker` on those files is rejected by a claim \
                 whose owner no longer exists (zombie-worker guard).\n\
                 Wait for their done signals (`omega workers`), close them explicitly \
                 (`omega kill <worker>`), or re-run with --force to take them down too.",
                running.len(),
                running.join(", ")
            );
        }
        KillPlan::AlreadyClosed => {
            // Still run the state cleanup: a session that died on its own (a
            // crash, an OOM, a killed pane) leaves exactly the claims and
            // markers this command exists to reclaim, and reclaiming them
            // twice is a no-op.
            let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, name);
            // …and the same is true of the worktree. A worker that died before
            // committing is the single most common way an unrecovered tree is
            // left behind, and its session is exactly the one that is already
            // gone by the time anyone runs this. Returning here without looking
            // would defeat the guard in the case it was written for: nothing is
            // removed unless it is clean AND merged, so this only ever collects
            // an empty tree or PRINTS where the unrecovered work is.
            if !is_oracle {
                for dir in worker_worktrees(&omega_dir, name) {
                    cleanup_worker_worktree(&omega_dir, &dir);
                }
            }
            if is_oracle {
                clear_oracle_state(&config.state_dir, name);
            }
            println!("Session {} is already closed — nothing live to kill.", name);
            return Ok(());
        }
        KillPlan::Proceed { cascade } => cascade,
    };

    // WORKERS FIRST, target last: the target may be the pane running this very
    // command, so anything after killing it may never execute.
    for w in &cascade {
        // The scope release comes BEFORE the kill on purpose. If the kill hangs
        // or the daemon drops the connection, the claim is already gone —
        // a leaked claim is the failure with the long tail, a surviving pane is
        // visible in `omega list` and the operator can retry.
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, w);
        for dir in worker_worktrees(&omega_dir, w) {
            cleanup_worker_worktree(&omega_dir, &dir);
        }
        match mgr.kill_session(w).await {
            Ok(()) => println!("  cascaded worker closed: {}", w),
            Err(e) => println!("  cascaded worker {} could not be killed ({})", w, e),
        }
    }

    let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, name);
    if is_oracle {
        clear_oracle_state(&config.state_dir, name);
    } else {
        // A worker killed directly gets the same worktree treatment as one
        // that was cascaded.
        for dir in worker_worktrees(&omega_dir, name) {
            cleanup_worker_worktree(&omega_dir, &dir);
        }
    }

    // Kept as the last line and byte-identical to what it always printed: the
    // Telegram bot renders this stdout verbatim in its session card.
    match mgr.kill_session(name).await {
        Ok(()) => println!("Killed session: {}", name),
        Err(e) => println!("Killed session: {} (pane cleanup reported: {})", name, e),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The reaper — closing a worker that already finished, without an operator
// ---------------------------------------------------------------------------

/// Whether a done status means the worker will not act again.
///
/// Deliberately WIDER than `DoneSignal::is_terminal` (`done_clean | failed`),
/// because the two answer different questions. Core asks whether the REPORT is
/// a final verdict; this asks whether the SESSION still has work to do. A
/// `blocked` worker has, by L3 and R-DESTRUCT, written its block-file and
/// stopped, so its pane is a zombie exactly like a `done_clean` one — and the
/// work it did not finish is safe regardless, because its worktree is dirty or
/// unmerged and the worktree guard KEEPS both.
///
/// `pending` is excluded, and that exclusion is the one that needs saying out
/// loud: `pending` is what the L4 close-gate writes over an unfinished plan,
/// and `cmd_progress` upgrades that signal back to `done_clean` on a later tick
/// (see `arms_gate_upgrade`). A session closed here could never produce that
/// tick, so reaping `pending` would freeze precisely the missions the upgrade
/// path exists to finish.
fn is_stop_status(status: DoneStatus) -> bool {
    match status {
        DoneStatus::DoneClean | DoneStatus::Failed | DoneStatus::Blocked => true,
        DoneStatus::Pending => false,
    }
}

/// What the reaper decided about ONE session, resolved before anything is
/// touched.
///
/// The distinction that carries the whole safety property is the first
/// variant: a worker with no done signal is STILL WORKING, and closing it
/// destroys in-flight work. Absence of a signal is never read as "probably
/// finished".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReapVerdict {
    /// No `worker-<session>.done.json` at all. Never touched, alive or dead: a
    /// live one is mid-task, and a DEAD one crashed before it could signal,
    /// whose worktree is then the only copy of whatever it had already done.
    /// Reclaiming that case stays the operator's explicit `omega kill`, which
    /// is a decision somebody made rather than one a sweep made for them.
    StillWorking,
    /// A signal that is not a stop (today only `pending` — see
    /// `is_stop_status`). Left alone for the same reason.
    NotTerminal,
    /// Terminal signal, session still live: close it.
    Reap,
    /// Terminal signal, session already gone. Not an error and not a second
    /// cascade: only the reclaim (scope claim, worktree) re-runs, and every one
    /// of those steps is a no-op the second time — `ScopeClaim::release` on an
    /// absent file, and `worker_worktrees` finding nothing once the tree is
    /// unregistered. This is what makes reaping twice identical to reaping once.
    AlreadyClosed,
}

/// How long a scope claim whose owning session is GONE is left alone before the
/// sweep reclaims it.
///
/// The window exists for one race: `claim_or_reject` writes the claim before the
/// worker's session is necessarily listed by the daemon, so a claim seconds old
/// with no live owner may be a worker that is about to appear rather than one
/// that died. Ten minutes is far longer than that gap and far shorter than the
/// weeks these claims currently survive.
const ORPHAN_CLAIM_GRACE_SECS: i64 = 600;

/// A scope claim whose owning session no longer exists.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OrphanClaim {
    session: String,
    files: Vec<String>,
    age_secs: i64,
}

/// Which scope claims the sweep may reclaim, as a pure function of the claims on
/// disk, the live session set, and the clock.
///
/// THE LEAK THIS CLOSES, measured on this box: five claims aged 17 to 25 days,
/// every one owned by a worker that died BEFORE writing a done signal. They are
/// invisible to `reap_verdict` for two independent reasons — the sweep only
/// enumerates LIVE worker sessions, and a signal-less worker is `StillWorking`,
/// which is never touched. Both rules are right for a worker that still exists.
/// Neither can ever fire for one that does not, so the claim stays on disk
/// forever and `claim_or_reject` keeps rejecting the next `spawn-worker` on those
/// files against an owner nobody can find (R-SCOPE).
///
/// THE NARROWING that keeps `StillWorking`'s safety property intact: this
/// reclaims the CLAIM only. The worktree of a worker that died mid-task may hold
/// the only copy of its work, so it is never removed here — `cmd_reap` prints its
/// path instead and leaves the recovery an operator decision, exactly as before.
fn plan_orphan_claims(
    claims: &[omega_core::scope::ScopeClaim],
    live: &[String],
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<OrphanClaim> {
    let live: std::collections::HashSet<&str> = live.iter().map(|s| s.as_str()).collect();
    let mut out: Vec<OrphanClaim> = claims
        .iter()
        .filter(|c| !live.contains(c.session.as_str()))
        .map(|c| OrphanClaim {
            session: c.session.clone(),
            files: c.files_owned.clone(),
            age_secs: (now - c.claimed_at).num_seconds(),
        })
        .filter(|o| o.age_secs >= ORPHAN_CLAIM_GRACE_SECS)
        .collect();
    out.sort_by(|a, b| a.session.cmp(&b.session));
    out
}

/// One session the reaper looked at, carrying everything the decision needs and
/// nothing it does not.
#[derive(Debug, Clone)]
struct ReapCandidate {
    session: String,
    /// Whether the session daemon still lists it.
    live: bool,
    /// The status in `worker-<session>.done.json`; `None` when there is no
    /// readable file.
    signal: Option<DoneStatus>,
}

/// The reaper's rule, as a pure function of the two facts it depends on.
fn reap_verdict(live: bool, signal: Option<DoneStatus>) -> ReapVerdict {
    let Some(status) = signal else {
        return ReapVerdict::StillWorking;
    };
    if !is_stop_status(status) {
        return ReapVerdict::NotTerminal;
    }
    if live {
        ReapVerdict::Reap
    } else {
        ReapVerdict::AlreadyClosed
    }
}

/// The reaper's decision over a whole candidate list.
///
/// Split from the execution for the same reason `decide_kill` is: the
/// alternative is a live rmux daemon plus real workers mid-task, which is
/// exactly the situation nobody can reproduce on demand — so the safety
/// property that matters most would be the one thing left untested.
fn plan_reap(candidates: &[ReapCandidate]) -> Vec<(String, ReapVerdict)> {
    candidates
        .iter()
        .map(|c| (c.session.clone(), reap_verdict(c.live, c.signal)))
        .collect()
}

/// The status in a worker's done signal, or `None` when it has not written one.
///
/// An UNREADABLE or malformed file reads as `None` — still working — which
/// keeps the reaper fail-closed: a corrupt signal is not evidence that a worker
/// finished, and guessing in that direction is what closes a live session.
fn done_status_of(state_dir: &std::path::Path, session: &str) -> Option<DoneStatus> {
    omega_core::done::DoneSignal::read(state_dir, session)
        .ok()
        .flatten()
        .map(|s| s.status)
}

/// `omega reap [<session>] [--dry-run]` — close the workers that already finished.
///
/// The zombie this ends, observed twice on this box: a worker wrote a terminal
/// `done.json` and its rmux session stayed OPEN, so the operator had to run
/// `omega kill` by hand for every one of them. What this does to a reaped
/// worker is byte-for-byte what that manual `omega kill` already did — release
/// the scope claim, unregister the worktree only when it holds nothing unsaved,
/// close the pane — so nothing about the closure semantics changes; only the
/// manual step disappears.
///
/// Every step is INDEPENDENT and best-effort. A pane that will not die must not
/// skip the scope release, and a git probe that fails must not skip the kill: a
/// half-run closure leaks exactly the thing this command exists to reclaim.
async fn cmd_reap(target: Option<&str>, dry_run: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let omega_dir = config
        .state_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    // Fail CLOSED when liveness cannot be established. Without the daemon every
    // session reads as dead, and a sweep would then release the scope claims of
    // workers that are still editing files — the exact R-SCOPE damage this
    // command exists to prevent, inverted.
    let mgr = SessionManager::connect().await.map_err(|e| {
        anyhow::anyhow!(
            "reap ABORTED — the session daemon is unreachable ({}), so a finished worker \
             cannot be told from a running one. Nothing was touched.",
            e
        )
    })?;
    let live = mgr.list_sessions().await.unwrap_or_default();

    let candidates: Vec<ReapCandidate> = match target {
        // A named target is examined whether or not it is still listed: the
        // reclaim half (scope claim, worktree) outlives the pane, and a worker
        // that died right after signalling is the common way it is left behind.
        Some(name) => vec![ReapCandidate {
            session: name.to_string(),
            live: live.iter().any(|s| s.name == name),
            signal: done_status_of(&config.state_dir, name),
        }],
        // The sweep looks at WORKERS only. An oracle's closure is a different
        // contract — `cmd_done` auto-closes a clean one and leaves a failed one
        // open on purpose so the operator can inspect it — and nothing here
        // changes it.
        None => live
            .iter()
            .filter(|s| s.role == omega_core::session::SessionRole::Worker)
            .map(|s| ReapCandidate {
                session: s.name.clone(),
                live: true,
                signal: done_status_of(&config.state_dir, &s.name),
            })
            .collect(),
    };

    let plan = plan_reap(&candidates);
    let mut reaped = 0usize;
    for (session, verdict) in &plan {
        match verdict {
            ReapVerdict::StillWorking => {
                println!("  {}: no done signal — still working, left alone", session);
            }
            ReapVerdict::NotTerminal => {
                println!("  {}: signal is not a stop — left alone", session);
            }
            ReapVerdict::Reap | ReapVerdict::AlreadyClosed => {
                let was_live = *verdict == ReapVerdict::Reap;
                if dry_run {
                    let state = if was_live { "live" } else { "already closed" };
                    println!(
                        "  {}: WOULD be reaped (terminal signal, {})",
                        session, state
                    );
                    continue;
                }
                reaped += 1;
                // The scope release comes FIRST, as in `cmd_kill`: a leaked
                // claim is the failure with the long tail (it rejects the next
                // `spawn-worker` on those files against an owner nobody can
                // find), where a surviving pane is visible in `omega list` and
                // the operator can retry.
                let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, session);
                // Only ever removes a tree that is clean AND merged; anything
                // holding uncommitted or unmerged work is KEPT and its path
                // printed, because losing a worker's work is far worse than
                // leaking a directory.
                for dir in worker_worktrees(&omega_dir, session) {
                    cleanup_worker_worktree(&omega_dir, &dir);
                }
                if !was_live {
                    println!("  {}: already closed — scope claim reclaimed", session);
                    continue;
                }
                match mgr.kill_session(session).await {
                    Ok(()) => println!("  {}: reaped (scope released, session closed)", session),
                    Err(e) => println!(
                        "  {}: scope released, but the session did not close ({})",
                        session, e
                    ),
                }
            }
        }
    }

    // ── ORPHAN SCOPE CLAIMS ──
    // Only on the SWEEP: a named target already gets its claim reclaimed above,
    // live or dead, and widening a targeted run into a global claim sweep would
    // make `omega reap <one-worker>` touch state the operator never named.
    let mut released = 0usize;
    let orphans = if target.is_none() {
        plan_orphan_claims(
            &omega_core::scope::ScopeClaim::read_all(&config.state_dir),
            &live.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            chrono::Utc::now(),
        )
    } else {
        Vec::new()
    };
    for o in &orphans {
        let days = o.age_secs / 86_400;
        if dry_run {
            println!(
                "  {}: WOULD release an orphan scope claim ({}d old, owner session gone) on {}",
                o.session,
                days,
                o.files.join(", ")
            );
            continue;
        }
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, &o.session);
        released += 1;
        println!(
            "  {}: orphan scope claim released ({}d old, owner session gone) on {}",
            o.session,
            days,
            o.files.join(", ")
        );
        // The worktree is NOT collected here — see `plan_orphan_claims`. A worker
        // that died before signalling may have its only copy of the work in
        // there, so its path is printed and the removal stays an operator call.
        for dir in worker_worktrees(&omega_dir, &o.session) {
            println!(
                "      worktree left in place (may hold unrecovered work): {}",
                dir.display()
            );
        }
    }

    // Always a quiet exit 0, including when there was nothing to do: the reaper
    // runs unattended (`omega done` schedules it, and a sweep can be crontab'd),
    // and a command that exits non-zero on "nothing to reap" turns a healthy
    // idle tick into an alert.
    if plan.is_empty() && orphans.is_empty() {
        println!("Nothing to reap — no worker session or orphan claim to reconcile.");
    } else {
        if !plan.is_empty() {
            println!("Reaped {} of {} session(s) examined.", reaped, plan.len());
        }
        if !orphans.is_empty() {
            println!(
                "Released {} of {} orphan scope claim(s).",
                if dry_run { 0 } else { released },
                orphans.len()
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The risk gate — presenting what omega_core::graph_risk decided
// ---------------------------------------------------------------------------

/// Load a graph document, or say which file and why it would not parse.
fn load_graph(path: &str) -> Result<omega_core::graph::Graph> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read the graph document {}", path))?;
    serde_json::from_str(&raw).with_context(|| format!("{} is not a readable graph", path))
}

/// Load a run state, or seed a fresh one from the graph.
///
/// A MISSING file is a legitimate first run and seeds an empty state. An
/// UNREADABLE one is an error rather than a silent reseed: a state document
/// that will not parse may hold a denial, and quietly replacing it with a blank
/// one would erase a human's refusal and let the gate ask again as if nothing
/// had ever been decided.
fn load_graph_state(
    path: Option<&str>,
    graph: &omega_core::graph::Graph,
) -> Result<omega_core::graph::GraphState> {
    let Some(path) = path else {
        return Ok(omega_core::graph::GraphState::for_graph(graph));
    };
    if !std::path::Path::new(path).exists() {
        return Ok(omega_core::graph::GraphState::for_graph(graph));
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read the run state {}", path))?;
    serde_json::from_str(&raw).with_context(|| format!("{} is not a readable run state", path))
}

/// `omega risk-gate <show|approve|deny>` — the operator's window onto the
/// R-DESTRUCT gate.
///
/// Thin on purpose, and the thinness is the point: `evaluate_gate`, `approve`
/// and `deny` decide, this formats. A CLI that re-derived "is this risky" would
/// be a second implementation of the rule, free to drift from the one the
/// executor actually consults, and the gate would then say different things
/// depending on who asked.
fn cmd_risk_gate(action: RiskGateAction) -> Result<()> {
    use omega_core::graph::NodeId;
    use omega_core::graph_risk::{ExecutionMode, GateDecision};

    match action {
        RiskGateAction::Show {
            graph,
            node,
            state,
            unattended,
        } => {
            let g = load_graph(&graph)?;
            let s = load_graph_state(state.as_deref(), &g)?;
            let mode = if unattended {
                ExecutionMode::Unattended
            } else {
                ExecutionMode::Attended
            };
            let id = NodeId::new(node);
            match omega_core::graph_risk::evaluate_gate(&g, &s, &id, mode) {
                GateDecision::Proceed => {
                    println!("PROCEED  node {} ({} run)", id, mode);
                }
                GateDecision::RequireApproval {
                    node,
                    risk,
                    reason,
                    what_is_lost,
                } => {
                    println!("HELD     node {} ({} run)", node, mode);
                    println!("  risk:         {}", risk);
                    println!("  reason:       {}", reason);
                    println!("  what is lost: {}", what_is_lost);
                    println!(
                        "  resolve with: omega risk-gate approve {} {} --approver <who>",
                        graph, node
                    );
                }
                // Not a softer hold: the gate could not establish what it is
                // being asked to approve, so there is nothing a human could
                // meaningfully consent to. Exit non-zero — a caller that treated
                // this as a pass would be proceeding on an unreadable rule.
                GateDecision::Refuse { node, reason } => {
                    anyhow::bail!("REFUSED  node {}: {}", node, reason);
                }
            }
            Ok(())
        }
        RiskGateAction::Approve {
            graph,
            node,
            approver,
            state,
        } => resolve_risk_gate(&graph, &node, &approver, state.as_deref(), true),
        RiskGateAction::Deny {
            graph,
            node,
            approver,
            state,
        } => resolve_risk_gate(&graph, &node, &approver, state.as_deref(), false),
    }
}

/// Record one human decision against a held node.
///
/// The escalation record is built by evaluating the gate in UNATTENDED mode,
/// which is the mode that holds the widest set (an `elevated` node proceeds
/// attended, so evaluating attended would report nothing to decide on exactly
/// the nodes a dispatched run escalates). The record therefore describes what
/// the unattended run would have blocked on, which is what the human is being
/// asked about.
fn resolve_risk_gate(
    graph: &str,
    node: &str,
    approver: &str,
    state_path: Option<&str>,
    approving: bool,
) -> Result<()> {
    use omega_core::graph::NodeId;
    use omega_core::graph_risk::{ExecutionMode, GateDecision};

    let g = load_graph(graph)?;
    let mut s = load_graph_state(state_path, &g)?;
    let id = NodeId::new(node);
    let decision = omega_core::graph_risk::evaluate_gate(&g, &s, &id, ExecutionMode::Unattended);
    let record = match decision {
        GateDecision::Refuse { node, reason } => {
            anyhow::bail!("REFUSED  node {}: {}", node, reason);
        }
        GateDecision::Proceed => {
            anyhow::bail!(
                "node {} needs no decision — the gate lets it proceed unattended. \
                 Nothing was recorded.",
                id
            );
        }
        held => match held.into_escalation(chrono::Utc::now()) {
            Some(record) => record,
            // Unreachable: `into_escalation` returns None only for Proceed and
            // Refuse, both handled above. Loud rather than silent if that ever
            // stops being true.
            None => anyhow::bail!("node {} produced no escalation record", id),
        },
    };

    // The attribution check belongs to the core and stays there: `approve`
    // refuses an empty or whitespace approver with a typed error, and that
    // refusal is surfaced verbatim rather than pre-empted by a check here that
    // could disagree with it.
    let resolution = if approving {
        omega_core::graph_risk::approve(record, approver)
    } else {
        omega_core::graph_risk::deny(record, approver)
    };
    let resolution = match resolution {
        Ok(r) => r,
        Err(e) => anyhow::bail!(
            "REFUSED by the risk gate: {}.\n\
             Pass a real --approver: an approval nobody signed is not consent, and \
             nothing was recorded.",
            e
        ),
    };

    omega_core::graph_risk::record_resolution(&mut s, &resolution);
    let verdict = if resolution.is_approved() {
        "APPROVED"
    } else {
        "DENIED"
    };
    println!(
        "{} node {} by {}",
        verdict, resolution.record.node, resolution.approver
    );
    println!("  what is lost: {}", resolution.record.what_is_lost);

    // `record_resolution` is in-memory by contract (a decision core that wrote
    // files could not be replayed), so persisting is this caller's job. Without
    // a --state path there is nowhere durable to put it, and an unsaved decision
    // that PRINTS as recorded is worse than one that refuses to pretend.
    match state_path {
        Some(path) => {
            let json = serde_json::to_string_pretty(&s)?;
            std::fs::write(path, json)
                .with_context(|| format!("cannot write the run state {}", path))?;
            println!("  recorded in: {}", path);
        }
        None => {
            println!(
                "  NOT RECORDED — pass --state <run-state.json> to persist this decision; \
                 the gate will ask again without it."
            );
        }
    }
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
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": e.to_string() })
            );
            std::process::exit(1);
        }
    }
}

/// `omega codex-login` — start the Codex device-code re-login and print the
/// URL + one-time code. See `omega_core::codex_login` for why this is not
/// shaped like the Claude flow (no code to paste back) and why it backs the
/// credentials up first (the flow logs you out the instant it starts).
async fn cmd_codex_login() -> Result<()> {
    // The engine is blocking (spawn + poll a log file) → keep it off the runtime.
    match tokio::task::spawn_blocking(omega_core::codex_login::start).await {
        Ok(Ok(d)) => {
            println!(
                "{}",
                serde_json::json!({ "ok": true, "url": d.url, "code": d.code, "pid": d.pid })
            );
            Ok(())
        }
        Ok(Err(e)) => {
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": e.to_string() })
            );
            std::process::exit(1);
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": format!("join failed: {}", e) })
            );
            std::process::exit(1);
        }
    }
}

/// `omega codex-login-status [--pid N]` — settle the device-code flow: report
/// success, or restore the pre-flow credentials when it was abandoned.
async fn cmd_codex_login_status(pid: Option<u32>) -> Result<()> {
    let result = tokio::task::spawn_blocking(move || omega_core::codex_login::finish(pid)).await?;
    println!("{}", codex_login_status_json(&result));
    if !result.flow_succeeded {
        std::process::exit(1);
    }
    Ok(())
}

async fn cmd_codex_login_abort(pid: u32) -> Result<()> {
    let result = tokio::task::spawn_blocking(move || omega_core::codex_login::abort(pid)).await?;
    println!("{}", codex_login_abort_json(&result));
    if matches!(
        result.status,
        omega_core::codex_login::LoginStatus::Unknown { .. }
    ) {
        std::process::exit(1);
    }
    Ok(())
}

fn codex_login_status_label(status: &omega_core::codex_login::LoginStatus) -> String {
    match status {
        omega_core::codex_login::LoginStatus::LoggedIn { mode } => {
            format!("logged in using {mode}")
        }
        omega_core::codex_login::LoginStatus::NotLoggedIn => "not logged in".to_string(),
        omega_core::codex_login::LoginStatus::Unknown { reason } => {
            format!("unknown: {reason}")
        }
    }
}

fn codex_login_status_json(result: &omega_core::codex_login::FinishResult) -> serde_json::Value {
    serde_json::json!({
        "ok": result.flow_succeeded,
        "status": codex_login_status_label(&result.status),
        "restored": result.restored
    })
}

fn codex_login_abort_json(result: &omega_core::codex_login::AbortResult) -> serde_json::Value {
    let command_ok = !matches!(
        &result.status,
        omega_core::codex_login::LoginStatus::Unknown { .. }
    );
    serde_json::json!({
        "ok": command_ok,
        "aborted": result.aborted,
        "status": codex_login_status_label(&result.status),
        "restored": result.restored
    })
}

/// `omega codex-reconcile [--json]` — authoritative credential-topology
/// reconciliation for installers and operators. Unlike startup auto-heal, an
/// actual error is returned to the caller as a non-zero exit.
async fn cmd_codex_reconcile(json: bool) -> Result<()> {
    let outcome =
        tokio::task::spawn_blocking(omega_core::codex_login::reconcile_on_startup).await?;
    match outcome {
        Ok(status) => {
            let status = match status {
                omega_core::codex_login::StartupReconcile::Reconciled => "reconciled",
                omega_core::codex_login::StartupReconcile::DeferredActiveFlow => {
                    "deferred_active_flow"
                }
                omega_core::codex_login::StartupReconcile::DeferredLocked => "deferred_locked",
            };
            if json {
                println!("{}", serde_json::json!({ "ok": true, "status": status }));
            } else {
                println!("Codex credential topology: {status}");
            }
            Ok(())
        }
        Err(error) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "ok": false, "error": error.to_string() })
                );
            } else {
                eprintln!("Codex credential reconciliation failed: {error:#}");
            }
            Err(error).context("reconciling Codex credential topology")
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
            println!(
                "{}",
                serde_json::json!({ "ok": false, "error": e.to_string() })
            );
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
    // load_strict: a malformed tracker must surface its parse error here — the
    // lenient load() reported a corrupt file as "no tracker", telling the
    // operator the plan doesn't exist while it sat there broken.
    let tracker = omega_core::planner::PlanTracker::load_strict(dir)
        .context("loading .planner/tracker.json")?
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
        Ok(()) => println!("\n[+] plan validation: OK (acyclic, no dangling deps, no trivial verify_commands, no dup ids, exact files_to_touch)"),
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
        agent: omega_core::agents::Agent::Codex,
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

/// A live "still working" beat for long, silent CLI waits: a rotating glyph,
/// growing dots and elapsed seconds on one rewritten line.
///
/// Why: `dispatch` blocks ~17s on the amplify Brain pass (a full opus call,
/// dispatch.rs:348) BEFORE the oracle exists, and printed nothing until
/// "◆ Oracle dispatched" — a measured 17s of pure silence on every mission
/// over 40 chars. The wait is inherent to the LLM call (sonnet is only ~2x
/// faster and yields a poorer brief), so we make it honest instead of shorter.
/// Mirrors the Telegram `brainReply` beat (omega-tg-bot.ts:1437).
///
/// stderr + TTY-only BY DESIGN: piped, scripted and bot-captured output stays
/// byte-identical to before.
///
/// KNOWN, ACCEPTED: tracing also writes stderr, so a log record emitted while
/// the beat is live lands on the beat's line (measured: 2 per dispatch — the
/// stale-done-signal WARN and the git-sync INFO). The record is never lost or
/// erased, only prefixed by the current frame. Fixing it properly means
/// wrapping the global tracing writer to erase the line before each record —
/// a 57-command blast radius for a one-command cosmetic, so it is deliberately
/// NOT done here.
struct Beat {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Beat {
    fn start(label: &'static str) -> Self {
        use std::io::IsTerminal;
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        if !std::io::stderr().is_terminal() {
            return Beat { stop, handle: None };
        }
        let flag = stop.clone();
        let handle = tokio::spawn(async move {
            use std::io::Write;
            use std::sync::atomic::Ordering;
            const FRAMES: [char; 4] = ['◐', '◓', '◑', '◒'];
            let t0 = std::time::Instant::now();
            let mut tick = 0usize;
            while !flag.load(Ordering::Relaxed) {
                let secs = t0.elapsed().as_secs();
                let dots = ".".repeat((secs as usize % 3) + 1);
                // \r + erase-line: rewrite in place, never scroll the pane.
                eprint!(
                    "\r\x1b[2K  {} {}{}  {}s",
                    FRAMES[tick % FRAMES.len()],
                    label,
                    dots,
                    secs
                );
                let _ = std::io::stderr().flush();
                tick += 1;
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
            // Clear our line here — inside the task — so we can never race a
            // half-written frame against the caller's output.
            eprint!("\r\x1b[2K");
            let _ = std::io::stderr().flush();
        });
        Beat {
            stop,
            handle: Some(handle),
        }
    }

    async fn stop(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
    }
}

/// Open the session journal for a dispatch — for a SPAWN only.
///
/// A FOLLOWUP GETS NO JOURNAL, and that is the fix. `SessionLog::create` names
/// its file `<session>-<first 8 hex of the ms timestamp>.jsonl`, an ~65s
/// bucket, so a followup landing on a live oracle either appended a SECOND
/// session header (with its message ids restarting at 1) into the journal of
/// the mission still running, or opened a near-empty second file that
/// `find_latest` — which sorts by mtime — then served as the whole answer to
/// `omega log <oracle>`. Either way the operator asking "what is this oracle
/// doing" stopped seeing the mission it is actually doing. The incident that
/// motivated the followup feature had its two dispatches 24 SECONDS apart:
/// same bucket, first shape.
///
/// The followup is not lost from the record: `deliver_followup` appends a
/// `MissionLog` event to the live oracle's own timeline (`omega timeline`),
/// which is the log that belongs to a running mission.
fn write_dispatch_session_log(
    config: &OmegaConfig,
    outcome: &omega_core::dispatch::DispatchOutcome,
    mission: &str,
) {
    // ASK THE PREDICATE, NOT THE VARIANT. `matches!(…, Followup)` here missed
    // `FollowupUnconfirmed` the day it was added — a delivery that landed in the
    // same live session and would have opened the same forbidden journal under
    // its name. `went_to_live_oracle()` covers both by construction.
    if outcome.delivery.went_to_live_oracle() {
        return;
    }
    let sessions_dir = config.state_dir.join("sessions");
    if let Ok(mut log) =
        omega_core::session_log::SessionLog::create(&sessions_dir, &outcome.oracle_name, ".")
    {
        let _ = log.append_message("system", &format!("Mission dispatched: {}", mission));
    }
}

async fn cmd_dispatch(
    project: &str,
    mission: &str,
    agent: Option<&str>,
    new_oracle: bool,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;
    let mgr = SessionManager::connect().await?;
    let dispatcher = omega_core::dispatch::Dispatcher::new(mgr, config.clone());

    // The beat must stop on the error path too, so bind the Result first
    // rather than `?`-ing straight through and leaving a live beat behind.
    let beat = Beat::start("briefing the oracle");
    let dispatched = dispatcher
        .dispatch_oracle_with_agent(project, mission, agent, new_oracle)
        .await;
    beat.stop().await;
    let outcome = dispatched?;

    // Session log — for a spawn only; see write_dispatch_session_log.
    write_dispatch_session_log(&config, &outcome, mission);

    // report_lines() owns the output contract: line 0 is always the canonical
    // "Oracle dispatched: <name>" the Telegram bridge parses, a followup adds
    // its note on a separate line, and the last line is the machine-readable
    // DISPATCH_DELIVERY=<tag>. See DispatchOutcome::report_lines.
    for line in outcome.report_lines() {
        println!("{}", line);
    }
    println!("  Mission: {}", mission);
    Ok(())
}

#[derive(Debug, Clone)]
struct V3WorkerAttempt {
    mission_id: omega_core::mission::MissionId,
    task_id: String,
    attempt_id: String,
    plan_revision: u64,
}

fn declared_verify_command(prompt: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = prompt.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let marker = lower
            .find("verify command:")
            .or_else(|| lower.find("verify-command:"));
        let Some(marker) = marker else {
            continue;
        };
        let colon = line[marker..].find(':').map(|offset| marker + offset)?;
        let mut command = line[colon + 1..].trim();
        if command.is_empty() {
            command = lines
                .iter()
                .skip(index + 1)
                .map(|candidate| candidate.trim())
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with("```"))?;
        }
        command = command
            .trim_start_matches("- ")
            .trim()
            .trim_matches('`')
            .trim();
        if command.is_empty()
            || command
                .chars()
                .any(|ch| matches!(ch, ';' | '&' | '|' | '<' | '>' | '`' | '$' | '\n' | '\r'))
        {
            return None;
        }
        let argv = shlex::split(command)?;
        if !argv.is_empty() {
            return Some(argv);
        }
    }
    None
}

fn declared_done_criteria(prompt: &str) -> Vec<String> {
    for line in prompt.lines() {
        let lower = line.to_lowercase();
        if let Some(marker) = lower
            .find("done criteria:")
            .or_else(|| lower.find("done-criteria:"))
        {
            if let Some(colon) = line[marker..].find(':').map(|offset| marker + offset) {
                let criterion = line[colon + 1..].trim().trim_start_matches("- ").trim();
                if !criterion.is_empty() {
                    return vec![criterion.to_string()];
                }
            }
        }
    }
    vec!["All Done Criteria frozen in the immutable worker prompt are satisfied".to_string()]
}

fn prepare_v3_worker_attempt(
    config: &OmegaConfig,
    oracle_session: Option<&str>,
    worker_name: &str,
    task: &str,
    prompt: &str,
    work_dir: &str,
    files: &[String],
    provider: omega_core::agents::Agent,
) -> Result<Option<V3WorkerAttempt>> {
    let Some(oracle_session) = oracle_session else {
        return Ok(None);
    };
    let Some(state) =
        omega_core::oracle_lifecycle::OracleState::read(&config.state_dir, oracle_session)?
    else {
        return Ok(None);
    };
    if state.mission_id.as_str().is_empty() {
        return Ok(None);
    }

    let ledger_path = config.state_dir.join("mission-engine-v3.sqlite3");
    if !ledger_path.exists() {
        return Ok(None);
    }
    let ledger = omega_core::mission_ledger::MissionLedger::open(&ledger_path)?;
    let Some(mut projection) = ledger.mission(&state.mission_id)? else {
        return Ok(None);
    };
    let argv = declared_verify_command(prompt).ok_or_else(|| {
        anyhow::anyhow!(
            "worker brief has no safe, directly executable `Verify Command:`; \
             shell operators are not accepted in immutable verifier contracts"
        )
    })?;
    let task_contract = omega_core::mission::TaskContract {
        schema_version: omega_core::mission::CONTRACT_SCHEMA_VERSION,
        task_id: omega_core::mission::TaskId::new(task),
        name: task.to_string(),
        prompt: prompt.to_string(),
        acceptance_criteria: declared_done_criteria(prompt),
        verifier_checks: vec![omega_core::mission::VerifierCheck {
            schema_version: omega_core::mission::CONTRACT_SCHEMA_VERSION,
            check_id: format!("verify-{task}"),
            kind: omega_core::mission::VerifierCheckKind::Command {
                argv,
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 120,
        }],
        required_capabilities: vec!["code_editing".to_string(), "tool_calling".to_string()],
        scope: files.to_vec(),
        risk: omega_core::routing::classify_mission(prompt).risk,
        retry_policy: omega_core::mission::RetryPolicy::default(),
        depends_on: Vec::new(),
    };

    let active_plan = ledger.active_plan(&state.mission_id)?;
    let (plan_revision, plan_to_append) = match active_plan {
        None => {
            let plan = omega_core::mission::PlanContract::new(
                state.mission_id.clone(),
                1,
                projection.version,
                vec![task_contract],
                vec!["independent_verification".to_string()],
                Vec::new(),
            )?;
            (1, Some(plan))
        }
        Some(plan) => {
            if let Some(existing) = plan
                .tasks
                .iter()
                .find(|existing| existing.task_id.as_str() == task)
            {
                if existing != &task_contract {
                    anyhow::bail!(
                        "task `{task}` already exists in immutable plan revision {} with a \
                         different contract; dispatch it under a new task id",
                        plan.revision
                    );
                }
                (plan.revision, None)
            } else {
                let protected: Vec<omega_core::mission::TaskId> =
                    plan.tasks.iter().map(|item| item.task_id.clone()).collect();
                let mut tasks = plan.tasks.clone();
                tasks.push(task_contract);
                let amended = plan.amend(plan.revision, projection.version, tasks, &protected)?;
                (amended.revision, Some(amended))
            }
        }
    };

    if let Some(plan) = plan_to_append {
        let mut event = omega_core::mission_ledger::AppendEvent::new(
            state.mission_id.clone(),
            projection.version,
            format!(
                "worker-plan:{}:{}:{}",
                state.mission_id.as_str(),
                task,
                plan.revision
            ),
            worker_name,
            if plan.revision == 1 {
                "plan_accepted"
            } else {
                "plan_amended"
            },
        );
        event.provider = Some(provider.name().to_string());
        event.payload = serde_json::to_value(&plan)?;
        event.plan = Some(plan);
        event.next_mission_state = match projection.state {
            omega_core::mission::MissionState::Classified
            | omega_core::mission::MissionState::Blocked => {
                Some(omega_core::mission::MissionState::Planned)
            }
            omega_core::mission::MissionState::Planned
            | omega_core::mission::MissionState::Running => None,
            current => anyhow::bail!(
                "mission {} cannot accept a worker plan while in state {:?}",
                state.mission_id.as_str(),
                current
            ),
        };
        projection = ledger.append(event)?.projection;
    }

    let attempt_id = format!(
        "attempt-{}-{}",
        worker_name,
        chrono::Utc::now().timestamp_micros()
    );
    let mut queued = omega_core::mission_ledger::AppendEvent::new(
        state.mission_id.clone(),
        projection.version,
        format!("worker-attempt:{attempt_id}:queued"),
        worker_name,
        "task_attempt_queued",
    );
    queued.provider = Some(provider.name().to_string());
    queued.payload = serde_json::json!({
        "worker": worker_name,
        "task": task,
        "working_dir": work_dir,
    });
    queued.task_attempt = Some(omega_core::mission_ledger::TaskAttemptMutation {
        task_id: task.to_string(),
        attempt_id: attempt_id.clone(),
        plan_revision,
        expected_version: 0,
        next_state: omega_core::mission::TaskAttemptState::Queued,
    });
    ledger.append(queued)?;

    Ok(Some(V3WorkerAttempt {
        mission_id: state.mission_id,
        task_id: task.to_string(),
        attempt_id,
        plan_revision,
    }))
}

fn transition_v3_worker_attempt(
    config: &OmegaConfig,
    worker_name: &str,
    attempt: &V3WorkerAttempt,
    next: omega_core::mission::TaskAttemptState,
) -> Result<()> {
    let ledger = omega_core::mission_ledger::MissionLedger::open(
        config.state_dir.join("mission-engine-v3.sqlite3"),
    )?;
    let projection = ledger
        .mission(&attempt.mission_id)?
        .ok_or_else(|| anyhow::anyhow!("mission disappeared before worker transition"))?;
    let task_projection = ledger
        .task_attempt(&attempt.attempt_id)?
        .ok_or_else(|| anyhow::anyhow!("task attempt disappeared before worker transition"))?;
    let next_label = format!("{next:?}").to_lowercase();
    let mut event = omega_core::mission_ledger::AppendEvent::new(
        attempt.mission_id.clone(),
        projection.version,
        format!("worker-attempt:{}:{next_label}", attempt.attempt_id),
        worker_name,
        format!("task_attempt_{next_label}"),
    );
    event.task_attempt = Some(omega_core::mission_ledger::TaskAttemptMutation {
        task_id: attempt.task_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        plan_revision: attempt.plan_revision,
        expected_version: task_projection.version,
        next_state: next,
    });
    if next == omega_core::mission::TaskAttemptState::Running
        && projection.state == omega_core::mission::MissionState::Planned
    {
        event.next_mission_state = Some(omega_core::mission::MissionState::Running);
    }
    ledger.append(event)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cmd_spawn_worker(
    task: &str,
    prompt: &str,
    dir: Option<&str>,
    project: Option<&str>,
    files: Option<Vec<String>>,
    force: bool,
    worktree: bool,
    agent_override: Option<&str>,
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

    let mut work_dir = dir.unwrap_or(".").to_string();
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

    // Clear any STALE lifecycle markers from a prior run under the same name.
    // Worker names are deterministic (`<project>-worker-<task>`) and the
    // done.json survives its session, so a leftover signal from a predecessor
    // would make patrol read THIS fresh worker as already done on its next
    // tick — pushing the OLD outcome to the oracle and reaping (killing) the
    // new session after the close grace. Mirror of the Executor-path clear in
    // orchestration.rs; the blocked/close markers go too, for the same reason.
    // Ordering: after BOTH gates (prompt + scope claim) — a rejected dispatch
    // must leave no side effects, and the scope gate is a rejection path too.
    // Guard: if a same-name session is STILL ALIVE, or its done.json is fresh
    // (a just-finished worker whose result patrol/oracle hasn't consumed yet —
    // LLM oracles do double-fire spawn-worker), refuse instead of silently
    // destroying the unconsumed outcome of live or just-completed work.
    let done_marker = config
        .state_dir
        .join(format!("worker-{}.done.json", worker_name));
    let refuse = |why: String| {
        // This dispatch claimed scope above — a refusal must release it, or the
        // rejected name holds its files hostage (no side effects on rejection).
        if files.is_some() {
            let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, &worker_name);
        }
        anyhow::anyhow!(why)
    };
    if mgr.capture_pane(&worker_name).await.is_ok() {
        return Err(refuse(format!(
            "worker session `{worker_name}` is still alive — not clobbering it. \
             Wait for it to finish (or `omega kill {worker_name}`) before re-dispatching."
        )));
    }
    if let Ok(meta) = std::fs::metadata(&done_marker) {
        let fresh = meta
            .modified()
            .ok()
            .and_then(|m| m.elapsed().ok())
            .is_some_and(|age| age.as_secs() < 120);
        if fresh {
            return Err(refuse(format!(
                "worker `{worker_name}` left a done.json less than 2 minutes old — its result \
                 may not be consumed yet. Re-dispatch after patrol's next tick (or remove \
                 {} to override).",
                done_marker.display()
            )));
        }
    }
    for marker in [
        format!("worker-{}.done.json", worker_name),
        format!("worker-blocked-{}.json", worker_name),
        format!("worker-close-{}.json", worker_name),
    ] {
        let _ = std::fs::remove_file(config.state_dir.join(marker));
    }

    // GIT SYNC PREFLIGHT (pull-before-work doctrine): make sure the worker —
    // and the worktree branched off this HEAD below — starts from the CURRENT
    // origin state, not a stale checkout that silently rebuilds or overwrites
    // what cloud sessions already pushed. ff-only on a clean tree; a dirty or
    // diverged dir is never touched, the drift is surfaced to the worker
    // prompt instead.
    let git_sync = omega_core::git_sync::pull_preflight(std::path::Path::new(&work_dir));
    eprintln!("[git-sync] {}: {}", work_dir, git_sync.describe());
    let git_sync_warning = git_sync.warning();

    // --worktree: give this worker its OWN git worktree (independent HEAD + working
    // tree) so concurrent workers never race on the shared checkout. node_modules/.env
    // are symlinked in by omega-git-branch so builds/tests still work. The oracle later
    // runs omega-git-merge to integrate the branch and remove the worktree. Best-effort:
    // fall back to the shared dir (with a warning) if the worktree can't be created.
    if worktree {
        let home = dirs::home_dir().unwrap_or_default();
        let script = home.join(".omega/bin/omega-git-branch.sh");
        let out = std::process::Command::new("bash")
            .arg(&script)
            .arg("worktree")
            .arg(&worker_name)
            .arg("") // base = current HEAD of the repo dir
            .arg(&work_dir)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let wt = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !wt.is_empty() && std::path::Path::new(&wt).is_dir() {
                    eprintln!("[+] worker isolated in worktree: {wt}");
                    work_dir = wt;
                } else {
                    eprintln!(
                        "[!] worktree create returned no path — running in shared dir {work_dir}"
                    );
                }
            }
            Ok(o) => eprintln!(
                "[!] worktree create failed ({}) — running in shared dir {work_dir}",
                String::from_utf8_lossy(&o.stderr).trim()
            ),
            Err(e) => {
                eprintln!("[!] worktree create error ({e}) — running in shared dir {work_dir}")
            }
        }
    }

    // THE FUNNEL — inject the Worker-scoped Laws + operational rules, exactly
    // like Dispatcher::dispatch_worker_with_context. Without this, a worker
    // spawned via the CLI (the live path oracles use) gets NO doctrine.
    let mut full_prompt = prompt.to_string();
    // SESSION IDENTITY — a worker must know its own deterministic name: it is the
    // join key for its rmux session, its Claude conversation (--name, resumable),
    // and every state file the engine polls (worker-<name>.done.json etc.). Without
    // this a worker only knows its name if the oracle happened to paste it.
    full_prompt.push_str(&format!(
        "\n\n## SESSION IDENTITY\nYou are worker `{worker_name}` — this exact string is your rmux session name, \
         your Claude conversation name (resumable via `claude --resume {worker_name}`), and the key for your \
         state files in ~/.omega/state/. Use it verbatim in every `omega done {worker_name} …` / \
         `omega progress {worker_name} …` call — never a paraphrase.\n"
    ));
    // Surface an unresolved git drift to the worker so it reconciles BEFORE
    // editing instead of working blind on a stale/diverged checkout.
    if let Some(warning) = &git_sync_warning {
        full_prompt.push_str(&format!(
            "\n\n## GIT SYNC\n{warning}\nReconcile (fetch/pull --ff-only on a clean tree) before touching any file.\n"
        ));
    }
    // The shape of the work, matched from the brief. A worker inside a
    // self-correcting loop, an audit dimension or a long-horizon slice needs to
    // know which it is — the stop condition is different in each, and a worker
    // that does not know when to stop either quits early or never quits.
    let shape = omega_core::mission_patterns::orchestration_block(&full_prompt);
    if !shape.is_empty() {
        full_prompt.push_str("\n\n");
        full_prompt.push_str(&shape);
    }

    let agent_ctx = omega_core::rules::agent_context_block_for_mission(
        omega_core::rules::RuleScope::Worker,
        &full_prompt,
    );
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
    let agent = match agent_override {
        Some(name) => {
            let resolved = omega_core::agents::Agent::from_name(name).ok_or_else(|| {
                anyhow::anyhow!("unknown agent '{name}' — expected one of: claude, codex, glm")
            })?;
            if !matches!(
                resolved,
                omega_core::agents::Agent::Claude
                    | omega_core::agents::Agent::Codex
                    | omega_core::agents::Agent::Glm
            ) {
                anyhow::bail!(
                    "worker agent '{name}' is not allowed: only claude, codex and glm carry \
                     the finish-guard hooks a detached worker needs"
                );
            }
            resolved
        }
        None => omega_core::agents::Agent::from_name(&config.agent_command)
            .unwrap_or(omega_core::agents::Agent::Codex),
    };
    omega_core::providers::ProvidersConfig::negotiate_provider(
        Some(agent.name()),
        &[
            omega_core::providers::ProviderCapability::Reasoning,
            omega_core::providers::ProviderCapability::CodeEditing,
            omega_core::providers::ProviderCapability::ToolCalling,
        ],
        &[omega_core::providers::ProviderCapability::Delegation],
    )
    .map_err(|error| anyhow::anyhow!("provider capability negotiation failed: {error}"))?;
    let v3_attempt = prepare_v3_worker_attempt(
        &config,
        oracle_session.as_deref(),
        &worker_name,
        task,
        &full_prompt,
        &work_dir,
        files.as_deref().unwrap_or(&[]),
        agent,
    )?;
    let spawn_result = if matches!(agent, omega_core::agents::Agent::Claude) {
        let mut opts = omega_core::agents::LaunchOptions::default();
        opts.permission_mode = Some("bypassPermissions".to_string());
        opts.disallowed_tools = Some("Bash(git push:*) Bash(rm:*) Bash(sudo:*)".to_string());
        // Claude-side session label (`--name`): mirror the rmux session name so the
        // conversation is addressable/resumable by the SAME deterministic identity
        // (`claude --resume <name>`, searchable in /resume) — oracles already get
        // this in dispatch_oracle; workers were anonymous on the Claude side.
        opts.session_name = Some(worker_name.clone());
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
        mgr.create_agent_session_with_opts(&worker_name, &work_dir, agent, Some(&full_prompt), opts)
            .await
    } else {
        mgr.create_session_with_agent(&worker_name, Some(&work_dir), agent, Some(&full_prompt))
            .await
    };
    if let Err(e) = spawn_result {
        if let Some(attempt) = &v3_attempt {
            let _ = transition_v3_worker_attempt(
                &config,
                &worker_name,
                attempt,
                omega_core::mission::TaskAttemptState::Cancelled,
            );
        }
        // Roll back the scope claim so a failed spawn doesn't lock files forever.
        let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, &worker_name);
        return Err(e);
    }
    if let Some(attempt) = &v3_attempt {
        if let Err(error) = transition_v3_worker_attempt(
            &config,
            &worker_name,
            attempt,
            omega_core::mission::TaskAttemptState::Running,
        ) {
            let _ = mgr.kill_session(&worker_name).await;
            let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, &worker_name);
            return Err(error.context(
                "worker spawn rolled back because the authoritative V3 running transition failed",
            ));
        }
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
        let mut state =
            omega_core::oracle_lifecycle::OracleState::read(&config.state_dir, oracle_name)
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
            attempt_id: v3_attempt
                .as_ref()
                .map(|attempt| attempt.attempt_id.clone()),
            plan_revision: v3_attempt.as_ref().map(|attempt| attempt.plan_revision),
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
        println!(
            "  -kill {} session(s): {}",
            targets.len(),
            targets.join(", ")
        );
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
        cmd.args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--max-time",
            "15",
        ]);
        cmd.args(auth);
        cmd.arg(url);
        let out = cmd.output().ok()?;
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<u32>()
            .ok()
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
                println!(
                    "  {}  {} {}",
                    e.at.format("%m-%d %H:%M:%S"),
                    e.marker,
                    e.text
                );
            }
            println!("\n{} event(s)", tl.events.len());
        }
        None => {
            println!("No OracleState timeline for '{}'.", oracle);
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

    // ── Loop-guard timeline (R-LOOP) ──
    // The append-only loop record: dispatches, contests with their thrash
    // count, wall-clock notes, gate verdicts, and any escalation to a human.
    // Distinct from the OracleState replay above (which is phase-centric) —
    // this is the bounded-retry/escalation view that mitigates comprehension
    // debt. Only printed when there is something to show.
    let loop_events = omega_core::loop_guard::MissionLog::read(&config.state_dir, oracle);
    if !loop_events.is_empty()
        || omega_core::loop_guard::EscalationRecord::read(&config.state_dir, oracle).is_some()
    {
        println!();
        print!(
            "{}",
            omega_core::loop_guard::MissionLog::render(&config.state_dir, oracle)
        );
    }
    Ok(())
}

async fn doctor_checks(config: &OmegaConfig, deep: bool) -> Vec<omega_core::doctor::Check> {
    let mut checks = omega_core::doctor::run_all(config).await;
    if deep {
        checks.push(omega_core::doctor::probe_codex_auth().await);
    }
    checks
}

async fn cmd_doctor(fix: bool, deep: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let mut checks = doctor_checks(&config, deep).await;
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
            checks = doctor_checks(&config, deep).await;
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

    println!(
        "\n  Next: `omega backup`  (writes ~/omega-backup-<ts>.tgz — scp it OFF this machine)."
    );
    Ok(())
}

/// `omega backup` — archive the irreproducible OmegaOS state. Projects are never
/// bundled (only reported); they belong to the user's own git.
fn cmd_backup(out: Option<String>, include_memory: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let report = omega_core::backup::run_backup(
        &config,
        out.map(std::path::PathBuf::from),
        include_memory,
        &ts,
    )?;

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
            let prompt = parts
                .get(1)
                .unwrap_or(&"Implement your assigned task")
                .to_string();
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
/// One plan task as stored in `oracle-<key>.progress.json`: `{t: title, s: status}`.
#[derive(Debug, PartialEq, Eq)]
struct PlanTask {
    title: String,
    status: String,
}

/// The status glyph, identical to the one the Telegram progress card renders
/// (`taskList` in omega-tg-bot.ts). Same plan, same symbols, whichever surface
/// the operator or the oracle is looking at.
fn plan_task_glyph(status: &str) -> char {
    match status {
        "done" => '✓',
        "fail" => '✗',
        "doing" => '▸',
        _ => '☐',
    }
}

/// Read the `tasks` array out of a progress document. Anything malformed is
/// skipped rather than fatal: this file is written by three producers (the
/// CLI, the Telegram bot, patrol) and a read-back that panics on one odd entry
/// is worse than a read-back that shows the rest.
fn parse_plan_tasks(doc: &serde_json::Value) -> Vec<PlanTask> {
    doc.get("tasks")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| {
                    Some(PlanTask {
                        title: t.get("t")?.as_str()?.to_string(),
                        status: t
                            .get("s")
                            .and_then(|v| v.as_str())
                            .unwrap_or("todo")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Render the plan the way an oracle needs to READ it after a compaction:
/// every task, its glyph, and the counts. Pure, so the format is pinned by a
/// unit test instead of by whoever last looked at a terminal.
fn render_plan_checklist(key: &str, tasks: &[PlanTask]) -> String {
    if tasks.is_empty() {
        return format!("oracle-{key}: no plan recorded (0 tasks).");
    }
    let done = tasks.iter().filter(|t| t.status == "done").count();
    let mut out = format!("oracle-{}: plan {}/{}\n", key, done, tasks.len());
    for t in tasks {
        out.push_str(&format!("{} {}\n", plan_task_glyph(&t.status), t.title));
    }
    out.pop();
    out
}

/// The `--status` vocabulary, validated against the ledger's own state machine.
///
/// It used to be written to the file verbatim, so a typo minted a status no
/// consumer knows: the Telegram card renders it as `todo`, the L4 gate counts it
/// as unfinished forever, and `OracleTodo::load` cannot parse it at all. Failing
/// here is the only way that stays a caller error instead of an unreadable
/// ledger.
fn parse_todo_status(status: &str) -> Result<omega_core::oracle_todo::TodoStatus> {
    use omega_core::oracle_todo::TodoStatus;
    Ok(match status {
        "todo" => TodoStatus::Todo,
        "doing" => TodoStatus::Doing,
        "done" => TodoStatus::Done,
        "fail" => TodoStatus::Fail,
        _ => anyhow::bail!(
            "Invalid task status: {}. Use: todo, doing, done, fail",
            status
        ),
    })
}

/// Whether this close arms the gate-pending upgrade — the flag that lets a
/// later `omega progress` tick, or patrol, rewrite the signal back to
/// `done_clean`.
///
/// A named predicate rather than an inline `&&` because the third argument is
/// the one that is easy to drop and impossible to notice. The flag arms TWO
/// upgraders and only one of them reads the ledger: patrol re-derives the L4
/// rule from the raw JSON and trusts the on-disk `done` / `total` counters
/// (omega-core/src/patrol.rs:1131-1146). So a file that is valid JSON but not a
/// valid PLAN — an `s` value this vocabulary does not know, an entry with no
/// `t` — is refused by every surface here and still reads `3 == 3` to patrol,
/// which would flip the honest refusal to `done_clean` within one cycle. That
/// case is never armed, and nothing legitimate is lost: `omega progress` exits
/// 1 on such a file, so there is no honest upgrade waiting to happen.
///
/// WHAT THIS PREDICATE DELIBERATELY DOES NOT COVER, so the next reader does not
/// mistake it for a complete guard. Patrol's upgrader does not read the quality
/// gate either — `GateResult` appears nowhere in patrol.rs — while the upgrader
/// in `cmd_progress` does. So a signal downgraded ONLY because the independent
/// gate is absent is still armed here, and patrol can clear that refusal on its
/// next minute without ever consulting the gate. Refusing to arm it would close
/// that hole from this file, and it is NOT done, for two reasons: the arming
/// rule predates this change byte for byte, and narrowing it would also delete
/// the legitimate upgrade for a gate that passes AFTER `omega done` — the
/// behaviour this crate is explicitly required to preserve. The honest fix is
/// one crate down, in patrol's own copy of the rule: read `GateResult` exactly
/// as `cmd_progress` does, or call `OracleTodo::is_complete` instead of
/// trusting the counters. Until then, a mission refused only by the gate can be
/// auto-accepted by patrol, and the gate's own refusal text is cleared with it.
fn arms_gate_upgrade(
    requested: omega_core::done::DoneStatus,
    final_status: omega_core::done::DoneStatus,
    ledger_unreadable: bool,
) -> bool {
    requested == omega_core::done::DoneStatus::DoneClean
        && final_status == omega_core::done::DoneStatus::Pending
        && !ledger_unreadable
}

/// The French refusal lines `cmd_done` records in `pending_actions`, derived
/// from the ledger rather than from a second reading of the raw JSON.
///
/// Same wording, same order (failures first, then what is still owed) and the
/// same ratio fallback as [`closure_verdict`], which `omega status` prints — the
/// two surfaces must never describe one plan differently.
fn l4_refusal_reasons(todo: &omega_core::oracle_todo::OracleTodo) -> Vec<String> {
    let mut reasons: Vec<String> = todo
        .failed()
        .iter()
        .map(|t| format!("échec: {}", t.title))
        .collect();
    reasons.extend(
        todo.unfinished()
            .iter()
            .map(|t| format!("non fait: {}", t.title)),
    );
    if reasons.is_empty() {
        // No titles to name: report the ratio, or the absence of a plan at all.
        let (done, total) = todo.counts();
        reasons.push(if total == 0 {
            "plan missionnel absent ou vide; acceptation impossible".to_string()
        } else {
            format!("plan {}/{} — pas 100% (L4)", done, total)
        });
    }
    reasons
}

/// `omega progress <session>` with no mutating flag: PRINT the plan, write
/// nothing.
///
/// It used to merge-write the file and print a bare `[+] progress 3/7`, so an
/// oracle resuming after a compaction had no way to read its own plan back —
/// the only surface that showed the task list was the Telegram card, which an
/// agent cannot see. Worse, the silent rewrite restamped `ts` on every look,
/// which is precisely the field patrol's stall detector reads: merely LOOKING
/// at a stalled mission made it look alive.
fn cmd_progress_readback(state_dir: &std::path::Path, session: &str, json: bool) -> Result<()> {
    let key = session.strip_prefix("oracle-").unwrap_or(session);
    let path = state_dir.join(format!("oracle-{}.progress.json", key));
    let doc: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let tasks = parse_plan_tasks(&doc);
    if json {
        let done = tasks.iter().filter(|t| t.status == "done").count();
        println!(
            "{}",
            serde_json::json!({
                "session": session,
                "oracle": format!("oracle-{key}"),
                "exists": path.exists(),
                "total": tasks.len(),
                "done": done,
                "ts": doc.get("ts").and_then(|v| v.as_str()),
                "tasks": tasks
                    .iter()
                    .map(|t| serde_json::json!({ "t": t.title, "s": t.status }))
                    .collect::<Vec<_>>(),
            })
        );
    } else if !path.exists() {
        println!("oracle-{key}: no plan file yet ({}).", path.display());
    } else {
        println!("{}", render_plan_checklist(key, &tasks));
        // This reader is TOLERANT by design — one bad entry must not blank the
        // read-back an oracle is resuming from — but every WRITER now fails
        // closed on the same file. Without this line the checklist above reads
        // like a healthy plan while `omega progress --task …` exits 1 and
        // `omega done done_clean` refuses, and a post-compaction resume is
        // exactly when an oracle consults this surface and exactly when it has
        // to learn its ledger is unwritable.
        if omega_core::oracle_todo::OracleTodo::load(state_dir, session).is_err() {
            println!(
                "[!] this plan file does not parse strictly — the checklist above skips what it \
                 could not read, and every WRITE to it will be refused until it is repaired. \
                 tasks[] entries need both `t` and `s`, and `s` must be todo|doing|done|fail."
            );
        }
    }
    Ok(())
}

fn cmd_progress(
    session: &str,
    plan: Option<&str>,
    task: Option<&str>,
    status: Option<&str>,
    json: bool,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    // READ-BACK: no --plan and no --task means the caller is asking WHAT the
    // plan is, not changing it. `--status` alone is deliberately counted as
    // non-mutating: it was already a no-op (the status is only ever read
    // inside `if let Some(t) = task`), so nothing that works today changes.
    if plan.is_none() && task.is_none() {
        return cmd_progress_readback(&config.state_dir, session, json);
    }
    let key = session.strip_prefix("oracle-").unwrap_or(session);
    // The ledger owns this file now. It resolves the SAME path, keeps the same
    // `{tasks:[{t,s}], done, total, ts}` keys, preserves every foreign field the
    // Telegram bot writes (chat/thread/msgId/bot/project/oracle/mission),
    // recomputes the counters from the task list and still publishes through
    // tmp+rename — three readers poll this file concurrently (patrol's stall
    // pass, the TUI worker bars, the Telegram card), so a torn read must stay
    // impossible.
    //
    // DIVERGENCE (core wins): a progress file that does not PARSE used to be
    // silently replaced by a fresh document; `OracleTodo::load` fails instead,
    // because starting from empty here is what lets the next write overwrite a
    // real plan with nothing.
    //
    // The BOUND, because fail-closed on a shared file is a one-way door: every
    // later `omega progress` on this oracle exits 1 until the file is readable
    // again, while the read-back path still prints a healthy-looking checklist
    // (it parses tolerantly, by design). Two ways in — a torn read of the
    // Telegram bot's non-atomic `persistMsgId` write, or a task carrying an `s`
    // value the OLD binary wrote verbatim before this vocabulary was validated.
    // So the error names the file and the repair, rather than leaving an
    // operator with a bare parse error and no way out.
    let mut todo =
        omega_core::oracle_todo::OracleTodo::load(&config.state_dir, session).map_err(|e| {
            anyhow::anyhow!(
                "{e}\nThe plan was NOT changed. Repair the file in place, NOW — the error above \
                 names the spot; tasks[] entries need both `t` and `s`, and `s` must be one of \
                 todo|doing|done|fail. `omega progress {session}` still prints what it holds. \
                 Repair rather than delete (the file also carries the Telegram card's \
                 chat/msgId), and do not leave it: once this oracle signals done, the bot \
                 finalizes its card and removes this file within seconds."
            )
        })?;
    if let Some(p) = plan {
        // DIVERGENCE (core wins): this used to reset every task to `todo`.
        // `set_plan` keeps the status, evidence and unknown keys of a title it
        // already knows (invariant 1) — an oracle re-stating its plan after a
        // compaction is describing the same mission, not restarting it — and it
        // folds a title named twice into one item.
        todo.set_plan(p.split('|').map(|t| t.trim()).filter(|t| !t.is_empty()));
    }
    if let Some(t) = task {
        // DIVERGENCE (core wins): the upsert is now a validated transition, so
        // `done -> todo` is REFUSED instead of silently walking a finished task
        // backwards (invariant 2). The refusal names the item, and the plan on
        // disk is left exactly as it was — `upsert` rejects before `save` runs.
        //
        // DIVERGENCE (core wins): a second `--status doing` now DEMOTES the
        // previously-doing item to `todo` (invariant 3) where the inline upsert
        // left both marked `doing`. "What am I doing" must have ONE answer after
        // a compaction, and the Telegram card rendered every one of them with the
        // in-progress marker.
        let st = parse_todo_status(status.unwrap_or("done"))?;
        todo.upsert(t, st, None).map_err(anyhow::Error::from)?;
    }
    todo.save(&config.state_dir, session)?;
    let (done, total) = todo.counts();
    println!("[+] progress {}/{} for oracle-{}", done, total, key);

    // L4 GATE RESOLUTION: the `omega done` oracle path downgrades done_clean →
    // pending while the plan is <100% (gate_pending=true) — but the oracle's own
    // final task ("report done") is by contract still unfinished at omega-done
    // time. When THIS progress tick completes the plan (100% done, no failure),
    // upgrade the stuck signal back to done_clean and auto-close the session,
    // mirroring the inline auto-close in cmd_done. Oracle sessions only.
    let gate_passed = omega_core::gate::GateResult::read(&config.state_dir, session)
        .ok()
        .flatten()
        .map(|g| g.overall_pass)
        .unwrap_or(false);
    // `is_complete` IS the old `total > 0 && done == total && no fail` test, read
    // off the ledger instead of re-derived from raw JSON.
    let plan_complete = todo.is_complete();
    if session.starts_with("oracle-")
        && task.is_some()
        && status.unwrap_or("done") == "done"
        && plan_complete
        && gate_passed
    {
        if let Ok(Some(mut osignal)) =
            omega_core::done::OracleDoneSignal::read(&config.state_dir, session)
        {
            if osignal.status == omega_core::done::DoneStatus::Pending && osignal.gate_pending {
                osignal.status = omega_core::done::DoneStatus::DoneClean;
                osignal.pending_actions.clear();
                osignal.gate_pending = false;
                osignal.finished_at = chrono::Utc::now();
                osignal.duration_secs = (osignal.finished_at - osignal.started_at)
                    .num_seconds()
                    .max(0) as u64;
                osignal.write(&config.state_dir)?;
                // The 1-min notifier cron may have already reported the transient
                // Pending state and written its per-path .notified marker — without
                // invalidating it, the corrected done_clean would NEVER be sent and
                // the operator's record would permanently say "mission incomplète".
                omega_core::done::OracleDoneSignal::invalidate_notified(&config.state_dir, session);
                println!(
                    "[+] L4 gate satisfied - done upgraded to done_clean, auto-closing session"
                );
                if let Ok(exe) = std::env::current_exe() {
                    // Session names are sanitized to [A-Za-z0-9._-] (no shell
                    // metachars), so this format is injection-safe.
                    //
                    // --force because this branch has already DECIDED to close:
                    // the plan is 100% and the independent gate accepted it. A
                    // straggler worker must be cascaded down with the oracle,
                    // not allowed to veto a close that the L4 gate just
                    // granted — vetoing it is how the oracle pane and every one
                    // of its workers stayed alive after a finished mission.
                    let _ = std::process::Command::new("bash")
                        .arg("-c")
                        .arg(format!(
                            "sleep 3; '{}' kill '{}' --force >/dev/null 2>&1",
                            exe.to_string_lossy(),
                            session
                        ))
                        .spawn();
                }
            }
        }
    } else if session.starts_with("oracle-") && plan_complete && !gate_passed {
        println!(
            "[-] Plan complete, but independent quality gate is not accepted; mission remains pending"
        );
    }
    Ok(())
}

fn v3_declared_artifacts(
    state_dir: &std::path::Path,
    session: &str,
) -> Result<Vec<omega_core::done::DoneArtifact>> {
    let Some((mission_id, task_id)) =
        omega_core::oracle_lifecycle::OracleState::read_all(state_dir)
            .into_iter()
            .find_map(|state| {
                state
                    .workers
                    .iter()
                    .find(|worker| worker.session_name == session)
                    .map(|worker| (state.mission_id.clone(), worker.task_id.clone()))
            })
    else {
        return Ok(Vec::new());
    };
    if mission_id.as_str().is_empty() {
        return Ok(Vec::new());
    }
    let ledger_path = state_dir.join("mission-engine-v3.sqlite3");
    if !ledger_path.exists() {
        return Ok(Vec::new());
    }
    let ledger = omega_core::mission_ledger::MissionLedger::open(ledger_path)?;
    let Some(plan) = ledger.active_plan(&mission_id)? else {
        return Ok(Vec::new());
    };
    let Some(task) = plan
        .tasks
        .into_iter()
        .find(|task| task.task_id.as_str() == task_id)
    else {
        return Ok(Vec::new());
    };
    Ok(task
        .verifier_checks
        .into_iter()
        .map(|check| match check.kind {
            omega_core::mission::VerifierCheckKind::Command {
                argv,
                expected_exit_code,
                ..
            } => omega_core::done::DoneArtifact::Command {
                cmd: argv.join(" "),
                exit_code: expected_exit_code,
            },
            omega_core::mission::VerifierCheckKind::Http {
                url,
                expected_status,
            } => omega_core::done::DoneArtifact::Url {
                url,
                expected_status,
            },
            omega_core::mission::VerifierCheckKind::FileExists { path } => {
                omega_core::done::DoneArtifact::FilePath { path }
            }
            omega_core::mission::VerifierCheckKind::GitObject { sha } => {
                omega_core::done::DoneArtifact::GitSha { sha, branch: None }
            }
        })
        .collect())
}

fn record_done_projection<T: serde::Serialize>(
    state_dir: &std::path::Path,
    session: &str,
    value: &T,
    idempotency_suffix: &str,
    kind: &str,
) -> Result<Option<omega_core::done::ProjectionProvenance>> {
    let ledger_path = state_dir.join("mission-engine-v3.sqlite3");
    if !ledger_path.exists() {
        return Ok(None);
    }

    let states = omega_core::oracle_lifecycle::OracleState::read_all(state_dir);
    let oracle = if omega_core::session::OmegaSession::classify(session).role
        == omega_core::session::SessionRole::Oracle
    {
        states.into_iter().find(|state| {
            state.oracle_name == session
                || state.oracle_name.strip_prefix("oracle-") == session.strip_prefix("oracle-")
        })
    } else {
        states.into_iter().find(|state| {
            state
                .workers
                .iter()
                .any(|worker| worker.session_name == session)
        })
    };
    let Some(oracle) = oracle.filter(|state| !state.mission_id.as_str().is_empty()) else {
        return Ok(None);
    };
    let worker_attempt = oracle
        .workers
        .iter()
        .find(|worker| worker.session_name == session)
        .and_then(|worker| {
            Some((
                worker.task_id.clone(),
                worker.attempt_id.clone()?,
                worker.plan_revision?,
            ))
        });

    let ledger = omega_core::mission_ledger::MissionLedger::open(&ledger_path)?;
    let Some(current) = ledger.mission(&oracle.mission_id)? else {
        // This is a legacy OracleState created before the V3 ledger existed.
        return Ok(None);
    };
    let mut event = omega_core::mission_ledger::AppendEvent::new(
        oracle.mission_id,
        current.version,
        format!("{kind}:{session}:{idempotency_suffix}"),
        session,
        kind,
    );
    event.provider = Some(OmegaConfig::load().unwrap_or_default().agent_command);
    event.correlation_id = Some(oracle.oracle_name);
    event.payload = serde_json::to_value(value)?;
    if let Some((task_id, attempt_id, plan_revision)) = worker_attempt {
        if let Some(task) = ledger.task_attempt(&attempt_id)? {
            if task.state == omega_core::mission::TaskAttemptState::Running {
                event.task_attempt = Some(omega_core::mission_ledger::TaskAttemptMutation {
                    task_id,
                    attempt_id,
                    plan_revision,
                    expected_version: task.version,
                    next_state: omega_core::mission::TaskAttemptState::CandidateDone,
                });
            }
        }
    }
    let appended = ledger.append(event)?;

    Ok(Some(omega_core::done::ProjectionProvenance {
        source: "mission-engine-v3.sqlite3".to_string(),
        event_id: appended.event.event_id,
        event_sequence: appended.event.sequence,
        mission_version: appended.projection.version,
        projection_hash: appended.projection.projection_hash,
    }))
}

fn finalize_v3_oracle_delivery(
    state_dir: &std::path::Path,
    session: &str,
) -> Result<Option<omega_core::done::ProjectionProvenance>> {
    let Some(state) = omega_core::oracle_lifecycle::OracleState::read(state_dir, session)? else {
        return Ok(None);
    };
    if state.mission_id.as_str().is_empty() {
        return Ok(None);
    }
    let ledger_path = state_dir.join("mission-engine-v3.sqlite3");
    if !ledger_path.exists() {
        return Ok(None);
    }
    let ledger = omega_core::mission_ledger::MissionLedger::open(ledger_path)?;
    if ledger.mission(&state.mission_id)?.is_none() {
        return Ok(None);
    }
    let Some(plan) = ledger.active_plan(&state.mission_id)? else {
        anyhow::bail!("V3 delivery refused: no accepted immutable plan");
    };
    if plan.tasks.is_empty() {
        anyhow::bail!("V3 delivery refused: the accepted plan is empty");
    }
    let attempts = ledger.task_attempts(&state.mission_id)?;
    let unaccepted: Vec<String> = plan
        .tasks
        .iter()
        .filter(|task| {
            !attempts.iter().any(|attempt| {
                attempt.task_id == task.task_id.as_str()
                    && attempt.state == omega_core::mission::TaskAttemptState::Accepted
            })
        })
        .map(|task| task.task_id.0.clone())
        .collect();
    if !unaccepted.is_empty() {
        anyhow::bail!(
            "V3 delivery refused: tasks lack an independently accepted attempt: {}",
            unaccepted.join(", ")
        );
    }

    let mut last = None;
    loop {
        let projection = ledger
            .mission(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("V3 mission projection disappeared"))?;
        let next = match projection.state {
            omega_core::mission::MissionState::Running => {
                omega_core::mission::MissionState::Verifying
            }
            omega_core::mission::MissionState::Verifying => {
                omega_core::mission::MissionState::Accepted
            }
            omega_core::mission::MissionState::Accepted => {
                omega_core::mission::MissionState::Reporting
            }
            omega_core::mission::MissionState::Reporting => {
                omega_core::mission::MissionState::Delivered
            }
            omega_core::mission::MissionState::Delivered => break,
            current => anyhow::bail!(
                "V3 delivery refused: mission is in non-deliverable state {:?}",
                current
            ),
        };
        let label = format!("{next:?}").to_lowercase();
        let mut event = omega_core::mission_ledger::AppendEvent::new(
            state.mission_id.clone(),
            projection.version,
            format!("oracle-delivery:{session}:{label}"),
            session,
            format!("mission_{label}"),
        );
        event.next_mission_state = Some(next);
        event.payload = serde_json::json!({
            "oracle": session,
            "source": "independent_gate_and_accepted_task_attempts",
        });
        last = Some(ledger.append(event)?);
    }

    let outcome = if let Some(outcome) = last {
        outcome
    } else {
        let projection = ledger
            .mission(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("V3 mission projection disappeared"))?;
        let event = ledger
            .events(&state.mission_id)?
            .into_iter()
            .last()
            .ok_or_else(|| anyhow::anyhow!("V3 mission has no provenance event"))?;
        omega_core::mission_ledger::AppendOutcome {
            event,
            projection,
            idempotent_replay: true,
        }
    };
    Ok(Some(omega_core::done::ProjectionProvenance {
        source: "mission-engine-v3.sqlite3".to_string(),
        event_id: outcome.event.event_id,
        event_sequence: outcome.event.sequence,
        mission_version: outcome.projection.version,
        projection_hash: outcome.projection.projection_hash,
    }))
}

async fn cmd_done(session: &str, status: &str, summary: &str, commit: Option<&str>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    config.ensure_dirs()?;

    let done_status = match status {
        "done_clean" => DoneStatus::DoneClean,
        "pending" => DoneStatus::Pending,
        "failed" => DoneStatus::Failed,
        "blocked" => DoneStatus::Blocked,
        _ => anyhow::bail!(
            "Invalid status: {}. Use: done_clean, pending, failed, blocked",
            status
        ),
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
        // unfinished. The verdict is the LEDGER's — `omega_core::oracle_todo` reads the
        // same oracle-<key>.progress.json, `honest_status` decides the downgrade and the
        // ledger's own accessors name what is missing — so the CLI no longer carries a
        // second reading of the same rule. The report then honestly shows incomplete
        // (no 92%-is-done).
        //
        // DIVERGENCE (core wins): `done` / `total` come from `counts()`, recomputed from
        // the task list, where the inline gate trusted the on-disk `done` / `total`
        // fields. A file whose counters disagree with its own tasks is exactly the drift
        // this module exists to end, and a stale counter reading 5/5 over an unfinished
        // list would have accepted the close.
        let mut final_status = done_status;
        let mut gate_pending: Vec<String> = Vec::new();
        let mut ledger: Option<omega_core::oracle_todo::OracleTodo> = None;
        // Set only when the plan file exists and does not PARSE. It is tracked
        // separately from the downgrade because it must NOT arm the gate-pending
        // upgrade — see where `osignal.gate_pending` is assigned.
        let mut ledger_unreadable = false;
        if final_status == omega_core::done::DoneStatus::DoneClean {
            match omega_core::oracle_todo::OracleTodo::load(&config.state_dir, session) {
                Ok(todo) => {
                    // DIVERGENCE (core wins): a plan carrying a FAILED item now reports
                    // `failed`, where the inline gate reported `pending` with
                    // `gate_pending` set. That flag exists so a later `omega progress`
                    // tick can flip the signal back to done_clean, and its branch
                    // requires a complete, failure-free plan — so while the failure
                    // stands the pending signal cannot resolve, and it describes the
                    // mission less honestly than `failed` does.
                    //
                    // The BOUND, stated rather than glossed, because it reaches past
                    // this file: a retried item CAN clear (`fail -> doing -> done`),
                    // and the old `pending` + `gate_pending` signal was upgraded back
                    // to done_clean by the very next progress tick. `failed` never is
                    // — the upgrade branch requires `Pending` — and, worse, the
                    // Telegram bot treats a non-gate-held terminal signal as final:
                    // it finalizes the card and DELETES oracle-<key>.progress.json
                    // (telegram-bot/omega-tg-bot.ts:1491 and :1530). A mission that
                    // recovers after that rebuilds its ledger from whatever it marks
                    // next, so it can close done_clean over a TRUNCATED plan (proved
                    // at runtime: 1/1, having lost the item it had already finished).
                    // Reporting a failure as a failure is the honest half of this
                    // trade; the bot half is a real regression and is reported to the
                    // operator rather than papered over here — `gate_pending` cannot
                    // be set for `failed` without holding a genuinely failed mission's
                    // card open forever, which is the worse of the two.
                    //
                    // `honest_status` never upgrades, so an explicit failed/blocked/
                    // pending request still passes through untouched.
                    let honest = omega_core::oracle_todo::honest_status(final_status, &todo);
                    if honest != final_status {
                        final_status = honest;
                        // DIVERGENCE (core wins): a MISSING progress file used to report
                        // "projection … illisible". The ledger reads absence as an empty
                        // plan, so it reports "plan missionnel absent ou vide" — the same
                        // refusal `omega status` already prints for that case.
                        gate_pending.extend(l4_refusal_reasons(&todo));
                    }
                    ledger = Some(todo);
                }
                // An UNPARSEABLE file is the only load error left, and it stays
                // fail-closed with the wording the bot and patrol already parse.
                //
                // THE COST OF THAT, recorded rather than discovered later: not
                // arming the upgrade (see `arms_gate_upgrade`) means the bot sees
                // no gate hold, finalizes the card and removes this progress file
                // on its next poll (telegram-bot/omega-tg-bot.ts:1491, :1530), so
                // the unreadable plan is GONE rather than waiting to be repaired.
                // The trade is still the right way round — patrol forging a
                // done_clean over a plan nobody could parse is worse than losing a
                // file that already failed to parse — but the repair hint
                // `cmd_progress` prints says "now" for this reason.
                Err(_) => {
                    final_status = omega_core::done::DoneStatus::Pending;
                    ledger_unreadable = true;
                    gate_pending.push(
                        "projection de plan absente ou illisible; acceptation impossible"
                            .to_string(),
                    );
                }
            }
        }
        if final_status == omega_core::done::DoneStatus::DoneClean {
            let gate_passed = omega_core::gate::GateResult::read(&config.state_dir, session)
                .ok()
                .flatten()
                .map(|g| g.overall_pass)
                .unwrap_or(false);
            if !gate_passed {
                final_status = omega_core::done::DoneStatus::Pending;
                gate_pending.push("quality gate indépendante absente ou non acceptée".to_string());
            }
        }
        // WORKER CLOSE-GATE + CASCADE (zombie-worker fix, dentistrygpt incident):
        // an oracle may NOT close itself while its workers still run, and a
        // clean close must take its FINISHED workers' sessions down with it —
        // until now the auto-close killed only the oracle pane, leaving every
        // worker session alive forever (no signal → no reaper).
        //
        // `evaluate_closure` owns that decision now; the CLI only executes the
        // ClosurePlan it hands back (kill the cascade, release the claims).
        let mut closure = omega_core::oracle_todo::ClosurePlan::default();
        if final_status == omega_core::done::DoneStatus::DoneClean {
            if let (Some(todo), Ok(live)) = (
                ledger.as_ref(),
                async { SessionManager::connect().await?.list_sessions().await }.await,
            ) {
                let lw = omega_core::oracle_lifecycle::live_workers_of_oracle(
                    &config.state_dir,
                    session,
                    &live,
                );
                // `None` for the existing signal, deliberately: `omega done` is
                // re-runnable here (a resumed oracle and patrol both re-issue it,
                // and the auto-close kill below must fire again if the first one
                // did not land), so the CLI does not take the module's
                // already-closed short circuit.
                match omega_core::oracle_todo::evaluate_closure(
                    todo,
                    &lw,
                    None,
                    omega_core::done::DoneStatus::DoneClean,
                ) {
                    Ok(plan) => closure = plan,
                    Err(omega_core::oracle_todo::ClosureRefusal::WorkersRunning(running)) => {
                        anyhow::bail!(
                            "done_clean REFUSED — {} worker(s) of this oracle still running: {}.\n\
                             An oracle cannot close while its workers run (zombie-worker guard).\n\
                             Wait for their done signals (omega workers), or close them explicitly \
                             (`omega kill <worker>`), then re-run `omega done`.",
                            running.len(),
                            running.join(", ")
                        );
                    }
                    // Unreachable: the L4 gate above already downgraded every
                    // plan-based refusal out of done_clean. Loud rather than silent
                    // if that ever stops being true.
                    Err(refusal) => anyhow::bail!("done_clean REFUSED — {}", refusal),
                }
            } else if ledger.is_none() {
                // Unreachable today: the only route to a still-clean `final_status`
                // is the `Ok(todo)` arm that sets `ledger`. Kept because a future
                // path reaching done_clean without a plan would skip the guard in
                // total silence.
                anyhow::bail!(
                    "done_clean REFUSED — the mission plan could not be loaded, so the \
                     zombie-worker guard cannot run. Re-run once `omega progress {}` reads \
                     the plan back cleanly.",
                    session
                );
            } else {
                // The REACHABLE gap, and it is pre-existing rather than introduced
                // here: the ledger loaded but the daemon did not answer, so the live
                // worker list is unknown. The old code skipped the guard here too —
                // silently, which is the part worth ending. Closing anyway is the
                // deliberate choice (a dead daemon must not strand an oracle that
                // finished), but an oracle that closes without ever checking its
                // workers has to SAY so: nothing is cascaded and no worker scope
                // claim is released on this path, so a leftover claim can reject the
                // next spawn-worker on the same files (R-SCOPE).
                println!(
                    "[!] session daemon unreachable — closing WITHOUT the zombie-worker \
                     check; no worker session was cascaded. Verify with `omega workers` \
                     and close any straggler explicitly (`omega kill <worker>`)."
                );
            }
        }
        let mut osignal =
            omega_core::done::OracleDoneSignal::new(key, project, final_status, summary);
        osignal.summary = summary.to_string();
        // Mark the L4-gate downgrade so `omega progress` / patrol can upgrade the
        // signal back to done_clean once the plan hits 100% (the oracle's own
        // final "report" task is unfinished at omega-done time by contract).
        //
        // NOT for an unreadable ledger — see `arms_gate_upgrade`, where that
        // exclusion is stated and tested.
        osignal.gate_pending = arms_gate_upgrade(done_status, final_status, ledger_unreadable);
        if osignal.gate_pending {
            // Arming the upgrade hands the verdict to a reader that does NOT use
            // the ledger: patrol recomputes nothing and trusts the on-disk `done`
            // / `total` (omega-core/src/patrol.rs:1131-1146). This gate has just
            // refused the close on the TASKS, so if those counters disagree with
            // the task list, patrol reads the stale pair, sees 2 == 2 over a plan
            // that is really 1 of 2, and upgrades a refusal it never re-derived.
            // `save` rewrites both counters from the tasks (and merges the bot's
            // keys back), so the numbers patrol is about to read are the ones this
            // gate just judged. Best-effort: a failed heal must not sink the
            // signal, it only leaves the pre-existing drift in place.
            if let Some(todo) = ledger.as_ref() {
                let _ = todo.save(&config.state_dir, session);
            }
        }
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
        osignal.projection = record_done_projection(
            &config.state_dir,
            session,
            &osignal,
            &osignal.finished_at.to_rfc3339(),
            "legacy_oracle_completion_candidate",
        )?;
        if final_status == omega_core::done::DoneStatus::DoneClean {
            osignal.projection =
                finalize_v3_oracle_delivery(&config.state_dir, session)?.or(osignal.projection);
        }
        osignal.write(&config.state_dir)?;
        // Release the scope claims on a clean close, mirroring the worker path.
        // `ClosurePlan::scopes_to_release` names the oracle PLUS every worker that
        // cascades with it; it is empty only when the daemon was unreachable, and the
        // oracle's own claim is still released in that case.
        if osignal.is_closeable() {
            if closure.scopes_to_release.is_empty() {
                let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, session);
            } else {
                for scope in &closure.scopes_to_release {
                    let _ = omega_core::scope::ScopeClaim::release(&config.state_dir, scope);
                }
            }
        }
        println!("[+] Oracle done signal written: oracle-{}.done.json", key);
        // AUDIT JOURNAL: append the mission outcome to the per-project audit log,
        // organized under ~/.omega/audit/<project>/audit.jsonl (governance trail — who
        // did what, when, with what result). Best-effort, never blocks the done signal.
        {
            let dir = config
                .state_dir
                .parent()
                .map(|p| p.join("audit").join(project));
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
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dir.join("audit.jsonl"))
                {
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
            // The finished workers die with their oracle; their scope claims were
            // released just above, with the oracle's, from the same ClosurePlan.
            if let Ok(exe) = std::env::current_exe() {
                // Session names are sanitized to [A-Za-z0-9._-] (no shell metachars),
                // so this format is injection-safe. Workers first, oracle last —
                // the oracle pane is the one running THIS command.
                let exe = exe.to_string_lossy();
                //
                // --force on the oracle kill: `omega kill` refuses by default
                // when a worker is still running, and that gate has ALREADY
                // been evaluated above (a running worker bailed out of this
                // whole branch). Without --force a worker that started or was
                // re-registered in the seconds between the two checks would
                // silently veto the auto-close and leave the oracle pane open
                // forever, which is a regression of the close contract, not a
                // safety win.
                let worker_kills: String = closure
                    .cascade_workers
                    .iter()
                    .map(|w| format!("'{}' kill '{}' >/dev/null 2>&1; ", exe, w))
                    .collect();
                let _ = std::process::Command::new("bash")
                    .arg("-c")
                    .arg(format!(
                        "sleep 3; {}'{}' kill '{}' --force >/dev/null 2>&1",
                        worker_kills, exe, session
                    ))
                    .spawn();
            }
        }
        return Ok(());
    }

    let mut signal = DoneSignal::new(session, done_status, summary);
    signal.commit = commit.map(|s| s.to_string());
    // A worker session represents one bounded task on the legacy projection.
    // The independent artifact gate still decides acceptance; this avoids the
    // ambiguous 0/0 count while keeping a no-op with zero evidence fail-closed.
    if signal.status == DoneStatus::DoneClean {
        signal.todos_total = 1;
        signal.todos_completed = 1;
    }

    // Candidate evidence only. Never add the current HEAD unconditionally:
    // an unchanged repository made a no-op worker look independently proven.
    // Explicit commits and actually changed paths are observations the patrol
    // can verify. Acceptance and lease release happen later, outside the
    // worker process.
    use omega_core::done::{CorroborationSource, DoneArtifact};
    signal
        .corroboration
        .push(CorroborationSource::WorkerSelfReport);
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(c) = commit.filter(|c| !c.trim().is_empty()) {
            signal.artifacts.push(DoneArtifact::GitSha {
                sha: c.to_string(),
                branch: None,
            });
        }

        let changed = std::process::Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=all"])
            .current_dir(&cwd)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let mut observed_paths = std::collections::BTreeSet::new();
        for line in changed.lines() {
            // Porcelain v1 has a two-byte status and one separating space.
            // Rename rows use `old -> new`; the destination is the artifact.
            let raw = line.get(3..).unwrap_or("").trim();
            let path = raw.rsplit_once(" -> ").map(|(_, new)| new).unwrap_or(raw);
            if !path.is_empty() {
                observed_paths.insert(path.trim_matches('"').to_string());
            }
        }
        for path in observed_paths {
            signal.artifacts.push(DoneArtifact::FilePath { path });
        }
        if signal
            .artifacts
            .iter()
            .any(|a| matches!(a, DoneArtifact::FilePath { .. }))
        {
            signal
                .corroboration
                .push(CorroborationSource::FilesystemCheck);
        }
    }
    for artifact in v3_declared_artifacts(&config.state_dir, session)? {
        signal.artifacts.push(artifact);
    }
    signal.projection = record_done_projection(
        &config.state_dir,
        session,
        &signal,
        &signal.finished_at.to_rfc3339(),
        "legacy_worker_completion_candidate",
    )?;
    signal.write(&config.state_dir)?;

    println!(
        "[+] Candidate completion written for: {} (scope remains held until independent acceptance)",
        session
    );

    // AUTO-REAP: a terminal signal closes its own session, without an operator.
    //
    // This is the zombie the reaper exists for, observed twice on this box: the
    // worker wrote this very file and its rmux session stayed OPEN, so the pane
    // had to be closed by hand. What the reap then does is byte-for-byte what
    // that manual `omega kill <worker>` already did, which is why the line above
    // is unchanged: the claim is released when the SESSION closes, exactly as
    // before, and the acceptance gate reads the done.json from the state dir,
    // which outlives the pane.
    //
    // Deferred and detached for the same reason the oracle auto-close is:
    // `omega done` normally runs INSIDE the pane being closed, so an inline kill
    // would take down the process still returning from this function. The reap
    // re-derives its decision from the file just written rather than being told
    // what to do, so it is still correct if this scheduling never fires (a later
    // sweep catches it) and a no-op if it fires twice.
    if is_stop_status(signal.status) {
        if let Ok(exe) = std::env::current_exe() {
            // Session names are sanitized to [A-Za-z0-9._-] (no shell
            // metachars), so this format is injection-safe.
            let exe = exe.to_string_lossy();
            let _ = std::process::Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "sleep 3; '{}' reap '{}' >/dev/null 2>&1",
                    exe, session
                ))
                .spawn();
            println!(
                "[+] terminal signal — session {} closes itself (omega reap); \
                 a worktree holding unsaved work is kept",
                session
            );
        }
    }
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
                    println!("  [{:?}] {}", event.event_type, event.payload);
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
    let pipeline =
        omega_core::ship::ShipPipeline::new(project_dir, config.state_dir.clone(), ship_config);

    if unfreeze {
        pipeline.unfreeze(project)?;
        println!("[+] Ship pipeline unfrozen for {}", project);
        return Ok(());
    }

    if pipeline.is_frozen(project) {
        println!(
            "[x] Ship pipeline is FROZEN for {}. Use --unfreeze to clear.",
            project
        );
        return Ok(());
    }

    println!("◆ Ship pipeline starting for {}...", project);
    let result = pipeline
        .execute(project, message, &Vec::<String>::new())
        .await;

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
            println!(
                "[x] Ship failed: {}",
                result.error.as_deref().unwrap_or("unknown")
            );
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
    let home = dirs::home_dir().unwrap_or_else(|| {
        std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    });
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
        println!(
            "Sessions: {} (◆{} ●{})",
            report.total_sessions, report.oracles, report.workers
        );
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

async fn cmd_gate(
    oracle: &str,
    mission: Option<&str>,
    accept: bool,
    approver: Option<&str>,
    evidence: Option<&str>,
) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();

    if accept {
        // Same alias tolerance as `omega status`: the operator reads the mission
        // key off an escalation and types that.
        let live: Vec<String> = match SessionManager::connect().await {
            Ok(m) => m
                .list_sessions()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.name)
                .collect(),
            Err(_) => Vec::new(),
        };
        let oracle = resolve_oracle_alias(oracle, &live, &config.state_dir);
        let result = omega_core::gate::GateResult::human_acceptance(
            &oracle,
            approver.unwrap_or_default(),
            evidence.unwrap_or_default(),
        )?;
        result.write(&config.state_dir)?;
        println!(
            "Gate ACCEPTED for {} by {} — {}",
            oracle,
            result.accepted_by.as_deref().unwrap_or("?"),
            result.accepted_evidence.as_deref().unwrap_or("")
        );
        println!("This is a human sign-off, not a graded pass: no rubric, consensus or");
        println!("adversarial check was run, and the record says so.");
        println!("Close the mission with:  omega done {oracle} done_clean \"<summary>\"");
        return Ok(());
    }
    if approver.is_some() || evidence.is_some() {
        anyhow::bail!("--approver / --evidence are only meaningful with --accept");
    }

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
        println!(
            "Rubric created for {}: {} criteria",
            oracle,
            rubric.criteria.len()
        );
        return Ok(());
    }

    // Prefer the ACTUAL gate result if the oracle has been graded — that is what
    // "check the gate" means. Falls back to showing the rubric (the criteria the
    // gate will grade against) when no result exists yet.
    let result_path = config
        .state_dir
        .join(format!("{}.gate-result.json", oracle));
    if result_path.exists() {
        let content = std::fs::read_to_string(&result_path)?;
        let r: omega_core::gate::GateResult = serde_json::from_str(&content)?;
        let mark = |b: bool| if b { "PASS" } else { "FAIL" };
        println!(
            "Gate result for {} — {} ({:.1}/100)",
            oracle,
            mark(r.overall_pass),
            r.score
        );
        println!(
            "  rubric={}  consensus={}  adversarial={}  regression={}",
            mark(r.rubric_pass),
            mark(r.consensus_pass),
            mark(r.adversarial_pass),
            mark(r.regression_pass)
        );
        println!(
            "  audit={}  token_budget={}  citation={}",
            mark(r.audit_pass),
            mark(r.token_budget_pass),
            mark(r.citation_pass)
        );
        return Ok(());
    }

    match omega_core::gate::Rubric::read(&config.state_dir, oracle)? {
        Some(rubric) => {
            println!(
                "No gate result yet for {} — showing the rubric it will grade against.",
                oracle
            );
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

/// Whether a mission could close right now, and if not, why not.
///
/// This is the SAME gate `cmd_done` applies, in the same order and with the
/// same wording, so `omega status` can never promise a close that `omega done`
/// then refuses. Keeping it a pure function is the only way to assert that
/// equivalence in a test.
#[derive(Debug, PartialEq, Eq)]
struct ClosureVerdict {
    refused: bool,
    reasons: Vec<String>,
}

fn closure_verdict(
    total: usize,
    done: usize,
    failed: &[String],
    unfinished: &[String],
    gate_passed: bool,
    running_workers: &[String],
) -> ClosureVerdict {
    let mut reasons = Vec::new();
    // (1) L4 completeness gate — cmd_done downgrades done_clean to pending here.
    if total == 0 {
        reasons.push("plan missionnel absent ou vide; acceptation impossible".to_string());
    } else if done < total || !failed.is_empty() {
        for f in failed {
            reasons.push(format!("échec: {f}"));
        }
        for u in unfinished {
            reasons.push(format!("non fait: {u}"));
        }
        if reasons.is_empty() {
            reasons.push(format!("plan {done}/{total} — pas 100% (L4)"));
        }
    }
    // (2) The independent quality gate. A self-graded mission is not accepted.
    if !gate_passed {
        reasons.push("quality gate indépendante absente ou non acceptée".to_string());
    }
    // (3) The zombie-worker guard, the only one that is a hard bail in cmd_done.
    if !running_workers.is_empty() {
        reasons.push(format!(
            "{} worker(s) still running: {}",
            running_workers.len(),
            running_workers.join(", ")
        ));
    }
    ClosureVerdict {
        refused: !reasons.is_empty(),
        reasons,
    }
}

/// The next command that actually clears each closure refusal.
///
/// `omega status` used to print the reasons and stop there, which reads as a
/// verdict with no appeal: the operator learns the mission cannot close and is
/// given nothing to type. Every refusal below HAS a remedy — the gate one is the
/// remedy nobody could guess, because `omega gate` only ever read the result and
/// the sole writer of one is `omega orchestrate`, a pipeline most missions never
/// run (a mission dispatched with `omega dispatch`, which is what the Telegram
/// bot does, could therefore never satisfy it).
///
/// Pure so the wording can be asserted against the reasons `closure_verdict`
/// actually produces, rather than drifting from them.
fn closure_remedies(verdict: &ClosureVerdict, session: &str) -> Vec<String> {
    if !verdict.refused {
        return Vec::new();
    }
    let mut out = Vec::new();
    let has = |needle: &str| verdict.reasons.iter().any(|r| r.contains(needle));

    if has("plan missionnel absent") || has("projection de plan") {
        out.push(format!(
            "persist the plan:      omega progress {session} --plan \"task a|task b|task c\""
        ));
    }
    if has("non fait:") || has("pas 100%") {
        out.push(format!(
            "close a finished task: omega progress {session} --task \"<title>\" --status done"
        ));
    }
    if has("échec:") {
        out.push(format!(
            "a failed task stays failed; correct it and re-run, or accept the mission as \
             non-clean:  omega done {session} pending \"<what remains and why>\""
        ));
    }
    if has("quality gate") {
        out.push(format!(
            "sign the gate off:     omega gate {session} --accept --approver \"<you>\" \
             --evidence \"<what you verified>\""
        ));
    }
    if has("still running") {
        out.push("account for the workers: omega workers   (then `omega kill <worker>`)".to_string());
    }
    // The LAST resort, and it is named as one. `omega kill` on an oracle is not
    // a neutral close: `clear_oracle_state` deletes `oracle-<key>.progress.json`,
    // so the ledger of everything the mission verified goes with the pane, and
    // the done signal is left exactly as it was. Recommending it as the easy way
    // out would trade a stuck mission for a destroyed record (R-DESTRUCT), so the
    // clean path above is offered first and this one says what it costs.
    out.push(format!(
        "last resort, DISCARDS the mission ledger (oracle-*.progress.json) and leaves \
         the done signal as-is:  omega kill {session}"
    ));
    out
}

/// Resolve an operator-typed name to the session that actually exists.
///
/// OmegaOS spells one mission two ways and the operator meets both: the state
/// files are keyed on the MISSION KEY (`dentistrygpt-3.mission-log.jsonl`,
/// `dentistrygpt-3.escalation.json`) while the pane, and therefore every CLI
/// lookup, is `oracle-dentistrygpt-3`. So the name printed in an escalation is
/// exactly the name the CLI rejects with a bare "Session not found", and the
/// operator has to guess the prefix at the moment something is already wrong.
///
/// A live exact match always wins, so this can never re-point a real session
/// that happens to share a suffix; the `oracle-` form is tried only when the
/// name as typed is not live.
fn resolve_oracle_alias(name: &str, live: &[String], state_dir: &std::path::Path) -> String {
    if live.iter().any(|s| s == name) {
        return name.to_string();
    }
    if name.starts_with("oracle-") {
        return name.to_string();
    }
    let prefixed = format!("oracle-{name}");
    if live.iter().any(|s| s == &prefixed) {
        return prefixed;
    }
    // Not live either way — fall back to the prefixed form only when this
    // mission left a record under it, so a genuinely unknown name still fails
    // with the message it always did.
    let known = state_dir
        .join(format!("oracle-{name}.progress.json"))
        .exists()
        || state_dir.join(format!("{prefixed}.state.json")).exists()
        || state_dir.join(format!("oracle-{name}.done.json")).exists();
    if known {
        prefixed
    } else {
        name.to_string()
    }
}

/// One oracle's headline, computed exactly the way `omega status` computes it
/// so the roster and the detail view can never disagree.
struct OracleRow {
    name: String,
    project: String,
    live: bool,
    phase: String,
    done: usize,
    total: usize,
    running: usize,
    terminal: usize,
    closeable: bool,
    first_reason: Option<String>,
    /// An unacknowledged operator escalation (`<key>.escalation.json`). It
    /// outlives the mission's own lifecycle object, so it is often the ONLY
    /// remaining trace that something went wrong here.
    escalation: Option<String>,
}

fn oracle_row(
    state_dir: &std::path::Path,
    name: &str,
    live_sessions: &[omega_core::session::OmegaSession],
) -> OracleRow {
    let key = name.strip_prefix("oracle-").unwrap_or(name);
    let state = omega_core::oracle_lifecycle::OracleState::read(state_dir, name)
        .ok()
        .flatten();
    let workers = omega_core::oracle_lifecycle::live_workers_of_oracle(state_dir, name, live_sessions);
    let doc: serde_json::Value =
        std::fs::read_to_string(state_dir.join(format!("oracle-{}.progress.json", key)))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));
    let tasks = parse_plan_tasks(&doc);
    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.status == "done").count();
    let failed: Vec<String> = tasks
        .iter()
        .filter(|t| t.status == "fail")
        .map(|t| t.title.clone())
        .collect();
    let unfinished: Vec<String> = tasks
        .iter()
        .filter(|t| t.status == "todo" || t.status == "doing")
        .map(|t| t.title.clone())
        .collect();
    let gate_passed = omega_core::gate::GateResult::read(state_dir, name)
        .ok()
        .flatten()
        .map(|g| g.overall_pass)
        .unwrap_or(false);
    let verdict = closure_verdict(
        total,
        done,
        &failed,
        &unfinished,
        gate_passed,
        &workers.running,
    );
    OracleRow {
        name: name.to_string(),
        project: state
            .as_ref()
            .map(|s| s.project.clone())
            .unwrap_or_else(|| {
                omega_core::session::OmegaSession::classify(name)
                    .project
                    .unwrap_or_default()
            }),
        live: live_sessions.iter().any(|s| s.name == name),
        phase: state
            .as_ref()
            .map(|s| s.phase.label().to_string())
            .unwrap_or_else(|| "?".to_string()),
        done,
        total,
        running: workers.running.len(),
        terminal: workers.terminal.len(),
        closeable: !verdict.refused,
        first_reason: verdict.reasons.first().cloned(),
        escalation: std::fs::read_to_string(state_dir.join(format!("{key}.escalation.json")))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| {
                v.get("detail")
                    .or_else(|| v.get("reason"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
            }),
    }
}

/// `omega oracles [--all]` — the roster.
///
/// WHY IT EXISTS: answering "what oracles are alive, and is any of them stuck"
/// took `omega list` (which shows only oracles with a progress file, as a bare
/// percentage), then one `omega status` per oracle, then a directory listing of
/// ~/.omega/state to find the ones with no pane at all. Three of them were stuck
/// on this box and none of it was visible in one place.
async fn cmd_oracles(all: bool) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let live_sessions = match SessionManager::connect().await {
        Ok(m) => m.list_sessions().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // The union of what is LIVE and what left a record: a crashed mission has no
    // pane and is exactly the one worth seeing.
    let mut names: Vec<String> = live_sessions
        .iter()
        .filter(|s| s.role == omega_core::session::SessionRole::Oracle)
        .map(|s| s.name.clone())
        .collect();
    if all {
        for st in omega_core::oracle_lifecycle::OracleState::read_all(&config.state_dir) {
            names.push(st.oracle_name);
        }
        // The GHOST MISSIONS, and they are the whole reason `--all` exists: a
        // pane-less oracle loses its `.state.json` to the 48h cleanup while its
        // escalation alarm survives forever, so the mission that most needs
        // looking at is precisely the one `read_all` can no longer see. The
        // ledger and the alarm outlive the lifecycle object — read those too.
        if let Ok(entries) = std::fs::read_dir(&config.state_dir) {
            for e in entries.flatten() {
                let Some(f) = e.file_name().to_str().map(|s| s.to_string()) else {
                    continue;
                };
                if let Some(key) = f.strip_suffix(".escalation.json") {
                    names.push(format!("oracle-{key}"));
                } else if let Some(rest) = f.strip_suffix(".progress.json") {
                    if rest.starts_with("oracle-") {
                        names.push(rest.to_string());
                    }
                }
            }
        }
    }
    names.sort();
    names.dedup();

    if names.is_empty() {
        println!("No oracle {}.", if all { "on record" } else { "live" });
        return Ok(());
    }

    let rows: Vec<OracleRow> = names
        .iter()
        .map(|n| oracle_row(&config.state_dir, n, &live_sessions))
        .collect();

    // Fixed columns, hard-truncated: a session name can be 50 chars and one long
    // row that wraps costs more than the characters it saves.
    fn fit(s: &str, w: usize) -> String {
        if s.chars().count() <= w {
            format!("{s:<w$}")
        } else {
            let keep: String = s.chars().take(w.saturating_sub(1)).collect();
            format!("{keep}…")
        }
    }
    println!(
        "{} {} {} {} {}",
        fit("ORACLE", 34),
        fit("STATE", 5),
        fit("PLAN", 7),
        fit("WORKERS", 8),
        "CLOSURE"
    );
    for r in &rows {
        let closure = if r.closeable {
            "closeable".to_string()
        } else {
            format!(
                "REFUSED — {}",
                r.first_reason.as_deref().unwrap_or("see omega status")
            )
        };
        println!(
            "{} {} {} {} {}",
            fit(&r.name, 34),
            fit(if r.live { "live" } else { "dead" }, 5),
            fit(&format!("{}/{}", r.done, r.total), 7),
            fit(&format!("{}r/{}t", r.running, r.terminal), 8),
            closure
        );
        if let Some(e) = &r.escalation {
            println!("{}⚠ {}", " ".repeat(35), e);
        }
    }
    let stuck = rows.iter().filter(|r| !r.closeable).count();
    let escalated = rows.iter().filter(|r| r.escalation.is_some()).count();
    if stuck > 0 || escalated > 0 {
        println!();
        if stuck > 0 {
            println!(
                "{stuck} of {} cannot close. `omega status <oracle>` prints the exact command for each.",
                rows.len()
            );
        }
        if escalated > 0 {
            println!(
                "{escalated} carry an unacknowledged escalation (the alarm outlives the mission)."
            );
        }
    }
    Ok(())
}

/// `omega workers [oracle]`.
///
/// Three separate refusal messages tell the operator to run this — the
/// done_clean refusal, the kill refusal, and the status remedy list — and until
/// now it did not exist, so the one instruction handed out at the moment a
/// mission is stuck exited 2 with "unrecognized subcommand".
async fn cmd_workers(oracle: Option<&str>) -> Result<()> {
    let config = OmegaConfig::load().unwrap_or_default();
    let live_sessions = match SessionManager::connect().await {
        Ok(m) => m.list_sessions().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let live_names: Vec<String> = live_sessions.iter().map(|s| s.name.clone()).collect();

    let targets: Vec<String> = match oracle {
        Some(o) => vec![resolve_oracle_alias(o, &live_names, &config.state_dir)],
        None => {
            let mut v: Vec<String> = live_sessions
                .iter()
                .filter(|s| s.role == omega_core::session::SessionRole::Oracle)
                .map(|s| s.name.clone())
                .collect();
            v.sort();
            v
        }
    };
    if targets.is_empty() {
        println!("No live oracle, so no worker to account for.");
        return Ok(());
    }
    for t in &targets {
        let w = omega_core::oracle_lifecycle::live_workers_of_oracle(
            &config.state_dir,
            t,
            &live_sessions,
        );
        println!(
            "─── {} ─── {} running, {} finished",
            t,
            w.running.len(),
            w.terminal.len()
        );
        for name in &w.running {
            println!("  ▸ {name}   still working — blocks this oracle's done_clean");
        }
        for name in &w.terminal {
            let status = omega_core::done::DoneSignal::read(&config.state_dir, name)
                .ok()
                .flatten()
                .map(|s| format!("{:?}", s.status))
                .unwrap_or_else(|| "terminal".to_string());
            println!("  ✓ {name}   {status} — cascades when the oracle closes");
        }
        if w.running.is_empty() && w.terminal.is_empty() {
            println!("  (none live)");
        }
    }
    Ok(())
}

/// `omega status <session> [--json]`.
///
/// A NON-oracle session keeps the original behaviour byte for byte (the last
/// 30 lines of its pane), because that is what every prompt template and the
/// `/omega-status` command tell an agent to read.
///
/// An ORACLE gets the lifecycle block FIRST, then the same pane tail. The old
/// output answered "what is it printing right now" but never "is this mission
/// closeable", so the operator had to reconstruct the close-gate by hand from
/// three state files, and usually reconstructed it wrong.
async fn cmd_status(name: &str, json: bool) -> Result<()> {
    // Resolve the mission-key spelling BEFORE classifying: `dentistrygpt-3`
    // classifies as a plain session, so it took the pane-capture branch and died
    // on "Session not found" while `oracle-dentistrygpt-3` printed a full report.
    let early_live: Vec<String> = match SessionManager::connect().await {
        Ok(m) => m
            .list_sessions()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.name)
            .collect(),
        Err(_) => Vec::new(),
    };
    let resolved = resolve_oracle_alias(
        name,
        &early_live,
        &OmegaConfig::load().unwrap_or_default().state_dir,
    );
    let name: &str = &resolved;

    let is_oracle = omega_core::session::OmegaSession::classify(name).role
        == omega_core::session::SessionRole::Oracle;
    if !is_oracle {
        let mgr = SessionManager::connect().await?;
        let content = mgr.capture_pane(name).await?;
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(30);
        for line in &lines[start..] {
            println!("{}", line);
        }
        return Ok(());
    }

    let config = OmegaConfig::load().unwrap_or_default();
    let key = name.strip_prefix("oracle-").unwrap_or(name);
    // Everything below is best-effort: a dead oracle whose daemon connection is
    // gone is exactly when the operator most needs to read its lifecycle, so a
    // failed connect must degrade to "no live data", never to an error.
    let mgr = SessionManager::connect().await.ok();
    let live = match &mgr {
        Some(m) => m.list_sessions().await.unwrap_or_default(),
        None => Vec::new(),
    };
    let session_live = live.iter().any(|s| s.name == name);
    let state = omega_core::oracle_lifecycle::OracleState::read(&config.state_dir, name)
        .ok()
        .flatten();
    let workers =
        omega_core::oracle_lifecycle::live_workers_of_oracle(&config.state_dir, name, &live);

    let doc: serde_json::Value = std::fs::read_to_string(
        config
            .state_dir
            .join(format!("oracle-{}.progress.json", key)),
    )
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_else(|| serde_json::json!({}));
    let tasks = parse_plan_tasks(&doc);
    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.status == "done").count();
    let failed: Vec<String> = tasks
        .iter()
        .filter(|t| t.status == "fail")
        .map(|t| t.title.clone())
        .collect();
    let unfinished: Vec<String> = tasks
        .iter()
        .filter(|t| t.status == "todo" || t.status == "doing")
        .map(|t| t.title.clone())
        .collect();
    let doing: Option<&PlanTask> = tasks.iter().find(|t| t.status == "doing");
    let gate_passed = omega_core::gate::GateResult::read(&config.state_dir, name)
        .ok()
        .flatten()
        .map(|g| g.overall_pass)
        .unwrap_or(false);
    let mut verdict = closure_verdict(
        total,
        done,
        &failed,
        &unfinished,
        gate_passed,
        &workers.running,
    );
    // `cmd_done` now derives its L4 gate from `OracleTodo::load`, which fails
    // CLOSED on a file this tolerant reader walks straight past (a task entry
    // with no `t`, a status no consumer knows). Without this check `omega status`
    // printed "closure allowed" over a file `omega done` then refuses to load —
    // exactly the promise `closure_verdict`'s own doc comment says it can never
    // make. The display above stays tolerant on purpose: a diagnostic command
    // must still SHOW the mess it is refusing to close over.
    if omega_core::oracle_todo::OracleTodo::load(&config.state_dir, name).is_err() {
        verdict.refused = true;
        // REPLACE, never append: `cmd_done` writes this one line and nothing else
        // for an unreadable plan (its own gate check is skipped once the status is
        // already Pending). Appending would leave `omega status` narrating the same
        // file with a longer list — counts read off a plan it just admitted it
        // cannot parse — which is the divergence between surfaces this change
        // exists to end.
        verdict.reasons =
            vec!["projection de plan absente ou illisible; acceptation impossible".to_string()];
    }
    let phase = state
        .as_ref()
        .map(|s| s.phase.label().to_string())
        .unwrap_or_else(|| "(no lifecycle state)".to_string());

    if json {
        println!(
            "{}",
            serde_json::json!({
                "session": name,
                "live": session_live,
                "phase": phase,
                "project": state.as_ref().map(|s| s.project.clone()),
                "plan": { "done": done, "total": total },
                "doing": doing.map(|t| t.title.clone()),
                "workers": { "running": workers.running, "terminal": workers.terminal },
                "gate_passed": gate_passed,
                "closeable": !verdict.refused,
                "refused_because": verdict.reasons,
            })
        );
        return Ok(());
    }

    println!("─── {} ───", name);
    println!(
        "  session   {}",
        if session_live { "live" } else { "not live" }
    );
    println!("  phase     {}", phase);
    println!("  plan      {}/{}", done, total);
    println!(
        "  doing     {}",
        doing.map(|t| t.title.as_str()).unwrap_or("(none)")
    );
    println!(
        "  workers   {} running, {} terminal",
        workers.running.len(),
        workers.terminal.len()
    );
    for w in &workers.running {
        println!("    ▸ {}", w);
    }
    for w in &workers.terminal {
        println!("    ✓ {}", w);
    }
    if verdict.refused {
        println!("  closure   REFUSED");
        for r in &verdict.reasons {
            println!("            - {}", r);
        }
        let remedies = closure_remedies(&verdict, name);
        if !remedies.is_empty() {
            println!("  next      to clear it:");
            for r in &remedies {
                println!("            {}", r);
            }
        }
    } else {
        println!("  closure   allowed (`omega done {} done_clean …`)", name);
    }

    // The pane tail still follows, unchanged, so an agent told to read
    // `omega status <session>` sees everything it used to see.
    if let Some(m) = &mgr {
        if let Ok(content) = m.capture_pane(name).await {
            println!("─── pane ───");
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(30);
            for line in &lines[start..] {
                println!("{}", line);
            }
        }
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

/// `omega stream <session> | <host>:<session> | list` — mirror a live rmux
/// session into a local viewer session that PULLS rendered screen snapshots.
///
/// Preflight before creation is the whole point: an unknown alias, a box that
/// does not answer, or a session that is not there each produce a named error
/// and a non-zero exit, never a viewer session that dies the instant it starts
/// (see omega_core::stream for the five constraints this obeys).
async fn cmd_stream(target: &str, detach: bool, interval: u32, lines: u32) -> Result<()> {
    use omega_core::stream::{self, ProbeOutcome};

    if target.eq_ignore_ascii_case("list") {
        return cmd_stream_list().await;
    }

    if interval == 0 {
        anyhow::bail!("--interval must be at least 1 second (0 would spin the loop at full CPU)");
    }
    if lines == 0 {
        anyhow::bail!("--lines must be at least 1 (0 would capture an empty screen)");
    }

    let t = stream::parse_target(target);

    // The coordinates land on the viewer's shell command line, so they are
    // refused rather than quoted when they are not slugs.
    if !stream::is_safe_coordinate(t.session()) {
        anyhow::bail!(
            "unsupported session name {:?} — rmux session names are slugs \
             ([A-Za-z0-9._-]); nothing else is streamable",
            t.session()
        );
    }
    if let Some(host) = t.host() {
        if !stream::is_safe_coordinate(host) {
            anyhow::bail!("unsupported ssh host alias {host:?} — use the alias from ~/.ssh/config");
        }
    }

    // PREFLIGHT 1 — an alias ssh actually knows.
    if let Some(host) = t.host() {
        let cfg = stream::read_ssh_config();
        let known = cfg.hosts.iter().any(|h| h == host);
        // An empty list means we could not enumerate, and an Include means the
        // list is only a lower bound. Neither is evidence the host is unknown,
        // so we let ssh be the judge instead of blocking a valid box.
        if !known && !cfg.hosts.is_empty() && !cfg.has_include {
            anyhow::bail!(
                "unknown ssh host alias {host:?}\n  \
                 ~/.ssh/config defines: {}\n  \
                 (hosts are ALIASES, never raw coordinates — add one to ~/.ssh/config first)",
                cfg.hosts.join(", ")
            );
        }
    }

    // PREFLIGHT 2 — the box answers, and the session is really on it.
    let box_label = t.host().unwrap_or("this box").to_string();
    let outcome = stream::probe_target(&t).await;
    let sessions = match &outcome {
        ProbeOutcome::Sessions(s) => s.clone(),
        other => anyhow::bail!(
            "cannot list sessions on {box_label}: {}\n  \
             (nothing was created — a viewer for an unreachable box would just render errors)",
            other.describe()
        ),
    };
    if !sessions.iter().any(|s| s == t.session()) {
        let listing = if sessions.is_empty() {
            "  (no rmux sessions at all on that box)".to_string()
        } else {
            sessions
                .iter()
                .map(|s| format!("  - {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        anyhow::bail!(
            "no session named {:?} on {box_label}. What IS there:\n{listing}",
            t.session()
        );
    }

    // IDEMPOTENT — one viewer per stream, always. Two pullers on one stream
    // interleave their snapshots into unreadable garbage.
    //
    // "Already streaming" is a claim about the SOURCE, so it is only ever made
    // after reading what the live viewer is actually pulling (resolve_viewer
    // parses its start command). Matching on the name alone is what once told
    // the operator they were watching B while the viewer rendered A.
    let viewer = match stream::resolve_viewer(&t).await {
        stream::ViewerChoice::Reuse(name) => {
            println!(
                "Already streaming {} in session {} — reusing it (a second puller would interleave).",
                t.label(),
                name
            );
            name
        }
        stream::ViewerChoice::Create(name) => {
            stream::create_viewer(&name, &t, interval, lines).await?;
            println!(
                "Streaming {} → session {} (rendered snapshot every {}s, {} lines).",
                t.label(),
                name,
                interval,
                lines
            );
            name
        }
        stream::ViewerChoice::Conflict { name, held_by } => {
            let holds = match held_by {
                Some(other) => format!("it is streaming {} instead", other.label()),
                None => "it is not a viewer we can read".to_string(),
            };
            anyhow::bail!(
                "the viewer name {name} is already taken and {holds}\n  \
                 (nothing was created — attaching you to another session's mirror is \
                 exactly the failure this check exists to prevent)\n  \
                 free it with: omega kill {name}"
            );
        }
    };

    stream_attach(&viewer, detach)
}

/// Attach to the viewer, or print exactly how to reach it when attaching from
/// here would be wrong.
///
/// rmux exports `RMUX` and `RMUX_PANE`, NOT `$TMUX` — testing `$TMUX` reports
/// "not in a multiplexer" from inside one, and attaching a session to itself
/// is how you nest a terminal inside a terminal.
fn stream_attach(viewer: &str, detach: bool) -> Result<()> {
    use std::io::IsTerminal;

    if detach {
        println!("  attach when you want it: rmux attach-session -t {viewer}");
        return Ok(());
    }
    if std::env::var_os("RMUX").is_some() {
        println!("  you are already inside rmux — switch to it: rmux switch-client -t {viewer}");
        return Ok(());
    }
    if !std::io::stdout().is_terminal() {
        println!("  no TTY here — attach from a terminal: rmux attach-session -t {viewer}");
        return Ok(());
    }

    let status = std::process::Command::new(omega_core::stream::rmux_bin())
        .args(["attach-session", "-t", viewer])
        .status()
        .context("spawning rmux attach-session")?;
    if !status.success() {
        anyhow::bail!(
            "could not attach to {viewer} (exit {:?}) — the viewer is running; \
             attach manually with: rmux attach-session -t {viewer}",
            status.code()
        );
    }
    Ok(())
}

/// `omega stream list` — what is watchable, on this box and on every ssh
/// alias, probed IN PARALLEL under a bounded timeout. A host that is down is
/// marked and skipped; it never holds the listing hostage.
async fn cmd_stream_list() -> Result<()> {
    use omega_core::stream::{self, ProbeOutcome};

    let cfg = stream::read_ssh_config();

    // Spawn every probe first, then collect: the whole listing costs one
    // timeout, not one per host.
    let mut probes: Vec<(Option<String>, tokio::task::JoinHandle<ProbeOutcome>)> = Vec::new();
    probes.push((None, tokio::spawn(async { stream::probe_host(None).await })));
    for host in &cfg.hosts {
        let host = host.clone();
        probes.push((
            Some(host.clone()),
            tokio::spawn(async move { stream::probe_host(Some(&host)).await }),
        ));
    }

    let mut results: Vec<(Option<String>, ProbeOutcome)> = Vec::new();
    for (host, handle) in probes {
        let outcome = handle.await.unwrap_or(ProbeOutcome::SpawnFailed {
            detail: "probe task panicked".to_string(),
        });
        results.push((host, outcome));
    }

    // The "already mirrored" index, read from what each live viewer is really
    // pulling rather than from its name. Name matching marked BOTH sources of a
    // colliding pair as mirrored while only one viewer existed.
    let mirrors = stream::live_viewer_index().await;

    for (host, outcome) in &results {
        let header = match host {
            None => "local".to_string(),
            Some(h) => format!("{h} (ssh)"),
        };
        println!("─── {} ───", header);
        match outcome {
            ProbeOutcome::Sessions(sessions) if sessions.is_empty() => {
                println!("  (no rmux sessions)");
            }
            ProbeOutcome::Sessions(sessions) => {
                for s in sessions {
                    let target = match host {
                        None => omega_core::stream::StreamTarget::Local { session: s.clone() },
                        Some(h) => omega_core::stream::StreamTarget::Remote {
                            host: h.clone(),
                            session: s.clone(),
                        },
                    };
                    let mirrored = mirrors.iter().any(|(_, t)| t == &target);
                    let note = if host.is_none() && s.starts_with("stream-") {
                        "  (a viewer)"
                    } else if mirrored {
                        "  (already streaming here)"
                    } else {
                        ""
                    };
                    println!("  ● {}{}", s, note);
                }
            }
            other => println!("  ✗ {}", other.describe()),
        }
        println!();
    }

    if cfg.hosts.is_empty() {
        println!("No ssh host aliases found in ~/.ssh/config — only this box was probed.");
    } else if cfg.has_include {
        println!(
            "note: ~/.ssh/config has an Include, so aliases defined in the included \
             files are not listed above (streaming them still works)."
        );
    }
    println!("Watch one:");
    println!("  omega stream <session>          a session on this box");
    println!("  omega stream <host>:<session>   a session on another box");
    Ok(())
}

/// `omega monitor <session> | <host>:<session> | list | classify`: the SESSION
/// monitor (see omega_core::session_monitor; the bare `omega monitor` is the
/// unrelated billing view and never reaches here).
///
/// Preflight before creation, exactly as `omega stream` does and for the same
/// reason: an unknown alias, a box that does not answer, or a session that is
/// not there each produce a named error and a non-zero exit, never a monitor
/// session that dies the instant it starts.
#[allow(clippy::too_many_arguments)]
async fn cmd_session_monitor(
    target: &str,
    detach: bool,
    interval: u32,
    lines: u32,
    work_probe: Option<&str>,
    work: Option<i64>,
    progress_probe: Option<&str>,
    audit_every: u32,
) -> Result<()> {
    use omega_core::session_monitor as monitor;
    use omega_core::stream::{self, ProbeOutcome};

    if target.eq_ignore_ascii_case("list") {
        return cmd_monitor_list().await;
    }
    if target.eq_ignore_ascii_case("classify") {
        return cmd_monitor_classify(work_probe, work).await;
    }

    if interval == 0 {
        anyhow::bail!("--interval must be at least 1 second (0 would spin the loop at full CPU)");
    }
    if lines == 0 {
        anyhow::bail!("--lines must be at least 1 (0 would capture an empty screen)");
    }

    let t = stream::parse_target(target);

    // The coordinates land on a shell command line, so they are refused rather
    // than quoted when they are not slugs.
    if !stream::is_safe_coordinate(t.session()) {
        anyhow::bail!(
            "unsupported session name {:?}: rmux session names are slugs \
             ([A-Za-z0-9._-]), and nothing else is monitorable",
            t.session()
        );
    }
    if let Some(host) = t.host() {
        if !stream::is_safe_coordinate(host) {
            anyhow::bail!("unsupported ssh host alias {host:?}, use the alias from ~/.ssh/config");
        }
    }

    // PREFLIGHT 1. An alias ssh actually knows. An empty list means we could
    // not enumerate and an Include means the list is a lower bound; neither is
    // evidence the host is unknown, so ssh gets to be the judge.
    if let Some(host) = t.host() {
        let cfg = stream::read_ssh_config();
        let known = cfg.hosts.iter().any(|h| h == host);
        if !known && !cfg.hosts.is_empty() && !cfg.has_include {
            anyhow::bail!(
                "unknown ssh host alias {host:?}\n  \
                 ~/.ssh/config defines: {}\n  \
                 (hosts are ALIASES, never raw coordinates: add one to ~/.ssh/config first)",
                cfg.hosts.join(", ")
            );
        }
    }

    // PREFLIGHT 2. The box answers, and the session is really on it.
    let box_label = t.host().unwrap_or("this box").to_string();
    let outcome = stream::probe_target(&t).await;
    let sessions = match &outcome {
        ProbeOutcome::Sessions(s) => s.clone(),
        other => anyhow::bail!(
            "cannot list sessions on {box_label}: {}\n  \
             (nothing was created, and a monitor of an unreachable box would just render errors)",
            other.describe()
        ),
    };
    if !sessions.iter().any(|s| s == t.session()) {
        let listing = if sessions.is_empty() {
            "  (no rmux sessions at all on that box)".to_string()
        } else {
            sessions
                .iter()
                .map(|s| format!("  - {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        anyhow::bail!(
            "no session named {:?} on {box_label}. What IS there:\n{listing}",
            t.session()
        );
    }

    // IDEMPOTENT. One monitor per target: two watchers on one session would
    // both nudge it, which turns one mechanical answer into a race.
    let viewer = monitor::monitor_viewer_name(&t);
    if stream::session_exists(&viewer).await {
        println!(
            "Already monitoring {} in session {}, reusing it (a second watcher would double every nudge).",
            t.label(),
            viewer
        );
    } else {
        monitor::create_monitor(
            &viewer,
            &t,
            interval,
            lines,
            work_probe.unwrap_or(""),
            progress_probe.unwrap_or(""),
            audit_every,
        )
        .await?;
        println!(
            "Monitoring {} → session {} (poll every {}s over {} lines, deep audit every {} polls).",
            t.label(),
            viewer,
            interval,
            lines,
            audit_every
        );
        if work_probe.is_none() {
            println!(
                "  note: no --work-probe, so a stop cannot be told from a block. Every stop \
                 will read as STALLED and be nudged, which is the safe direction (never stall \
                 silently) but not the informed one."
            );
        }
        if progress_probe.is_none() {
            println!(
                "  note: no --progress-probe, so the nudge budget has nothing to reset it and \
                 behaves as a flat cap."
            );
        }
    }

    stream_attach(&viewer, detach)
}

/// `omega monitor list`: what is watchable, on this box and on every ssh
/// alias, probed IN PARALLEL under a bounded timeout, plus which of them a
/// monitor is already watching. A host that is down is marked and skipped, so
/// it never holds the listing hostage.
async fn cmd_monitor_list() -> Result<()> {
    use omega_core::session_monitor as monitor;
    use omega_core::stream::{self, ProbeOutcome};

    let cfg = stream::read_ssh_config();

    // Spawn every probe first, then collect: the whole listing costs one
    // timeout, not one per host.
    let mut probes: Vec<(Option<String>, tokio::task::JoinHandle<ProbeOutcome>)> = Vec::new();
    probes.push((None, tokio::spawn(async { stream::probe_host(None).await })));
    for host in &cfg.hosts {
        let host = host.clone();
        probes.push((
            Some(host.clone()),
            tokio::spawn(async move { stream::probe_host(Some(&host)).await }),
        ));
    }

    let mut results: Vec<(Option<String>, ProbeOutcome)> = Vec::new();
    for (host, handle) in probes {
        let outcome = handle.await.unwrap_or(ProbeOutcome::SpawnFailed {
            detail: "probe task panicked".to_string(),
        });
        results.push((host, outcome));
    }

    // What each live monitor is REALLY watching, read from the command it was
    // started with rather than from its name. Matching on the name marks a
    // colliding pair as watched while only one watcher exists.
    let watched = monitor::live_monitor_index().await;

    for (host, outcome) in &results {
        let header = match host {
            None => "local".to_string(),
            Some(h) => format!("{h} (ssh)"),
        };
        println!("─── {} ───", header);
        match outcome {
            ProbeOutcome::Sessions(sessions) if sessions.is_empty() => {
                println!("  (no rmux sessions)");
            }
            ProbeOutcome::Sessions(sessions) => {
                for s in sessions {
                    let target = match host {
                        None => omega_core::stream::StreamTarget::Local { session: s.clone() },
                        Some(h) => omega_core::stream::StreamTarget::Remote {
                            host: h.clone(),
                            session: s.clone(),
                        },
                    };
                    let note = if host.is_none() && s.starts_with(monitor::MONITOR_PREFIX) {
                        "  (a monitor)"
                    } else if host.is_none() && s.starts_with("stream-") {
                        "  (a stream viewer)"
                    } else if watched.iter().any(|(_, t)| t == &target) {
                        "  (already monitored here)"
                    } else {
                        ""
                    };
                    println!("  ● {}{}", s, note);
                }
            }
            other => println!("  ✗ {}", other.describe()),
        }
        println!();
    }

    if cfg.hosts.is_empty() {
        println!("No ssh host aliases found in ~/.ssh/config, so only this box was probed.");
    } else if cfg.has_include {
        println!(
            "note: ~/.ssh/config has an Include, so aliases defined in the included \
             files are not listed above (monitoring them still works)."
        );
    }
    println!("Watch one:");
    println!("  omega monitor <session>          a session on this box");
    println!("  omega monitor <host>:<session>   a session on another box");
    Ok(())
}

/// `omega monitor classify`: the JUDGEMENT seam.
///
/// The shell loop captures the pane, pipes it here on stdin, and reads back one
/// word. Every rule that decides what a pane MEANS lives in Rust, where it is
/// unit-tested against real captures; the loop only polls, sleeps and sends.
/// The reference watcher put all of this in bash greps that could never be
/// tested, and that is the one thing this design deliberately improves on.
async fn cmd_monitor_classify(work_probe: Option<&str>, work: Option<i64>) -> Result<()> {
    use omega_core::session_monitor as monitor;
    use std::io::Read;

    let mut pane = String::new();
    std::io::stdin()
        .read_to_string(&mut pane)
        .context("reading the captured pane from stdin")?;

    // An already-measured count wins over a probe command. The watcher loop
    // measures work once per poll for its own decisions and passes the number
    // here, so re-running the probe would both cost a second ssh round trip
    // and risk classifying against a count the caller never saw.
    //
    // No probe, a failed probe and a hung probe all land on Unknown, and
    // Unknown is read as work. Never stall a build on a probe bug.
    let work = match work {
        Some(n) => monitor::WorkSignal::from_count(n),
        None => match work_probe {
            Some(cmd) => match monitor::run_probe(cmd, monitor::PROBE_TIMEOUT).await {
                Some(out) => monitor::WorkSignal::from_probe_output(&out),
                None => monitor::WorkSignal::Unknown,
            },
            None => monitor::WorkSignal::Unknown,
        },
    };

    println!("{}", monitor::classify(&pane, work).as_str());
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
                        println!(
                            "[{}] SESSION {} cwd={}",
                            h.timestamp.format("%H:%M:%S"),
                            h.session_name,
                            h.cwd
                        );
                    }
                    omega_core::session_log::SessionEntry::Message(m) => {
                        let preview: String = m.content.chars().take(80).collect();
                        println!(
                            "[{}] {} {}",
                            m.timestamp.format("%H:%M:%S"),
                            m.role,
                            preview
                        );
                    }
                    omega_core::session_log::SessionEntry::ToolCall(t) => {
                        println!("[{}] TOOL {}", t.timestamp.format("%H:%M:%S"), t.tool_name);
                    }
                    omega_core::session_log::SessionEntry::Done(d) => {
                        println!(
                            "[{}] DONE {} — {}",
                            d.timestamp.format("%H:%M:%S"),
                            d.status,
                            d.summary
                        );
                    }
                    omega_core::session_log::SessionEntry::Event(e) => {
                        println!(
                            "[{}] EVENT {}",
                            e.timestamp.format("%H:%M:%S"),
                            e.event_type
                        );
                    }
                    omega_core::session_log::SessionEntry::Compaction(c) => {
                        println!(
                            "[{}] COMPACT {} entries",
                            c.timestamp.format("%H:%M:%S"),
                            c.entries_compacted
                        );
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
    println!(
        "Recommended team:  {} agent(s)",
        decision.complexity.recommended_agents()
    );
    println!(
        "Estimated time:    ~{} min",
        decision.complexity.estimated_minutes()
    );
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
        _ => anyhow::bail!(
            "Unknown shell: {}. Use: bash, zsh, fish, elvish, powershell",
            shell
        ),
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
    println!(
        "[+] PDF generated: {} ({:.1} KB)",
        out,
        size as f64 / 1024.0
    );

    // Send via Telegram if requested
    if send_telegram {
        send_pdf_telegram(out, caption).await?;
    }

    Ok(())
}

fn find_pdfgen_dir(exe_dir: Option<&std::path::Path>) -> Result<std::path::PathBuf> {
    let omega_dir = omega_core::config::omega_dir();
    // 1. $OMEGA_DIR/skills/pdfgen (canonical installed location)
    let skills_dir = omega_dir.join("skills/pdfgen");
    if skills_dir.join("bin/pdfgen.ts").exists() {
        return Ok(skills_dir);
    }
    // 2. $OMEGA_DIR/pdfgen (legacy installed location)
    let user_dir = omega_dir.join("pdfgen");
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
        "PDF generator not found. Expected at tools/pdfgen/ or {}/pdfgen/.\n\
         Run `omega init` to set up, or copy the pdfgen/ directory manually.",
        omega_dir.display()
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

    let url = format!("https://api.telegram.org/bot{}/sendDocument", cfg.bot_token);

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
        form = form
            .text("caption", cap.to_string())
            .text("parse_mode", "HTML".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .context("sendDocument")?;
    if resp.status().is_success() {
        println!("[+] PDF sent via Telegram");
    } else {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Telegram sendDocument failed: {}", body);
    }

    Ok(())
}

fn cmd_rules(action: RulesAction) -> Result<()> {
    use omega_core::rules;
    match action {
        RulesAction::Context { scope, mission } => {
            let s = match scope.to_lowercase().as_str() {
                "master" | "atlas" | "director" => rules::RuleScope::Master,
                "worker" => rules::RuleScope::Worker,
                _ => rules::RuleScope::Oracle,
            };
            if let Some(m) = mission {
                print!("{}", rules::agent_context_block_for_mission(s, &m));
                return Ok(());
            }
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
            println!(
                "\nRules dir: {}/rules/",
                omega_core::config::omega_dir().display()
            );
            println!("Export:    omega rules export");
        }
        RulesAction::Export => {
            let rules_dir = omega_core::config::omega_dir().join("rules");
            let n = export_rules_to(&rules_dir, true)?;
            println!("\n{} rules exported to {}", n, rules_dir.display());
        }
    }
    Ok(())
}

fn resolve_skill_root(explicit: Option<&str>) -> Result<std::path::PathBuf> {
    if let Some(root) = explicit {
        return Ok(std::path::PathBuf::from(root));
    }
    let cwd_skills = std::env::current_dir()?.join("skills");
    if cwd_skills.is_dir() {
        return Ok(cwd_skills);
    }
    if let Ok(src) = std::env::var("OMEGA_SRC") {
        let candidate = std::path::PathBuf::from(src).join("skills");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Ok(omega_core::config::omega_dir().join("skills"))
}

fn cmd_skills(action: SkillsAction) -> Result<()> {
    use omega_core::skill_registry::{OwnedSkillRoot, SkillCatalogV1};

    let (root_arg, output) = match &action {
        SkillsAction::Validate { root } => (root.as_deref(), None),
        SkillsAction::Compile { root, out } => {
            let target = out
                .as_deref()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| omega_core::config::omega_dir().join("skill-catalog-v1.json"));
            (root.as_deref(), Some(target))
        }
    };
    let root = resolve_skill_root(root_arg)?;
    let catalog = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", &root)])?;

    println!(
        "[+] SkillCatalogV1: {} skills, {} warnings, sha256:{}",
        catalog.skills.len(),
        catalog.warnings.len(),
        catalog.content_digest
    );
    println!("    owned root: {}", root.display());
    for warning in catalog.warnings.iter().take(12) {
        println!(
            "    warning [{}] {}: {}",
            warning.code, warning.skill, warning.message
        );
    }
    if catalog.warnings.len() > 12 {
        println!(
            "    … {} additional migration warnings",
            catalog.warnings.len() - 12
        );
    }
    if let Some(path) = output {
        catalog.write_json(&path)?;
        println!("[+] Canonical catalog written: {}", path.display());
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
                        println!("  /{:<18} {} — {}", a.id, a.domain.label(), a.description);
                    }
                }
            }
        }
        AuditAction::Results { oracle } => {
            let config = OmegaConfig::load().unwrap_or_default();
            let path = config
                .state_dir
                .join(format!("{}.audit-report.json", oracle));
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
            println!(
                "  Phases: {}, Max score: /{}",
                skill.phases, skill.max_score
            );
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

/// Resolve the OmegaOS repo checkout that `omega sync` copies FROM.
///
/// The repo-sourced steps (OMEGA.md, agents/, tools/pdfgen) used bare
/// CWD-relative paths, so running `omega sync` from anywhere but the repo
/// root silently skipped them all. Resolution order: $OMEGA_SRC (install.sh's
/// source-dir convention), the CWD, then the known checkout locations (the
/// same candidate list doctor.rs uses for bot parity). A candidate counts
/// only if it actually looks like the repo (OMEGA.md + crates/omega-core).
/// `omega update [--check] [--dir <path>]` — bring this install up to date.
///
/// The update path already existed (`npx omega-os` re-runs `git pull --ff-only`
/// + `install.sh`) but it was unnamed, undocumented, and it *died* on a dirty or
/// diverged checkout with a raw git error. This is the same mechanism as a real
/// command that REFUSES rather than breaks: local work is never touched, never
/// stashed, never discarded — it is reported and the update stops.
///
/// `install.sh` is idempotent and guards every user file (`config.toml`,
/// `telegram.toml`, secrets, `projects.json`), so re-running it is safe.
fn cmd_update(check: bool, dir: Option<&str>) -> Result<()> {
    let src = match dir {
        Some(d) => {
            let p = std::path::PathBuf::from(d);
            if !p.join("OMEGA.md").is_file() {
                anyhow::bail!("{} is not an OmegaOS checkout (no OMEGA.md)", p.display());
            }
            p
        }
        None => resolve_omega_src().ok_or_else(|| {
            anyhow::anyhow!(
                "no OmegaOS checkout found.\n\
                 Looked at $OMEGA_SRC, the current directory, ~/Station/SideBusiness/OmegaOS, \
                 ~/Station/OmegaOS and ~/OmegaOS.\n\
                 Pass --dir <path>, or install fresh with:  npx omega-os"
            )
        })?,
    };

    let git = |args: &[&str]| -> String {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&src)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };

    if !src.join(".git").exists() {
        anyhow::bail!(
            "{} has no .git — it cannot be updated in place.\n\
             Reinstall with:  npx omega-os",
            src.display()
        );
    }

    println!("◆ OmegaOS checkout: {}", src.display());
    println!("  installed: v{}", env!("CARGO_PKG_VERSION"));

    let branch = {
        let b = git(&["rev-parse", "--abbrev-ref", "HEAD"]);
        if b.is_empty() || b == "HEAD" {
            "main".to_string()
        } else {
            b
        }
    };

    println!("  fetching origin/{}…", branch);
    let fetch = std::process::Command::new("git")
        .args(["fetch", "origin", &branch])
        .current_dir(&src)
        .output()?;
    if !fetch.status.success() {
        anyhow::bail!(
            "git fetch failed — check network/credentials:\n{}",
            String::from_utf8_lossy(&fetch.stderr).trim()
        );
    }

    let behind = git(&["rev-list", "--count", &format!("HEAD..origin/{}", branch)]);
    let ahead = git(&["rev-list", "--count", &format!("origin/{}..HEAD", branch)]);
    let dirty = !git(&["status", "--porcelain"]).is_empty();
    let behind_n: usize = behind.parse().unwrap_or(0);
    let ahead_n: usize = ahead.parse().unwrap_or(0);

    // Report the FULL state before deciding anything — a dirty tree is what
    // blocks an update, so it must be visible even when already up to date.
    //
    // "Nothing to pull" is NOT "nothing to install", and this interactive path
    // has to know that as surely as the cron does. HEAD can move without this
    // command ever fetching anything — you commit to the checkout yourself —
    // and then git is genuinely current while the BINARY is not. Reporting
    // "up to date" there is the exact lie the cron used to tell: measured on
    // the source box, `omega update --check` said "nothing changed" against a
    // binary thirty commits old.
    let head = git(&["rev-parse", "--short", "HEAD"]);
    let installed = {
        let cfg = omega_core::config::OmegaConfig::load().unwrap_or_default();
        omega_core::auto_update::AutoUpdateState::load(&cfg.state_dir).last_applied_commit
    };
    // `None` is unknown provenance, never a claim of staleness — an install
    // predating that record must not be reported as behind.
    let binary_stale = !head.is_empty() && installed.as_deref().is_some_and(|i| i != head.as_str());

    let up_to_date = behind_n == 0 && ahead_n == 0;
    if up_to_date {
        println!("  up to date with origin/{}", branch);
    } else {
        println!("  {} commit(s) behind, {} ahead", behind_n, ahead_n);
    }
    if binary_stale {
        println!(
            "  installed binary: built from {} — checkout HEAD is {}",
            installed.as_deref().unwrap_or("?"),
            head
        );
    }
    if dirty {
        println!("  local changes: present (never touched by update)");
    }

    if check {
        println!(
            "\n(--check: nothing changed){}",
            if dirty || ahead_n > 0 {
                " — an update would stop, see above"
            } else if behind_n > 0 {
                " — an update would fast-forward + reinstall"
            } else if binary_stale {
                " — an update would reinstall (the binary is behind the checkout)"
            } else {
                ""
            }
        );
        return Ok(());
    }

    if up_to_date && !dirty && !binary_stale {
        println!("✓ already up to date — nothing to do");
        return Ok(());
    }

    // REFUSE rather than clobber. `git pull --ff-only` would abort here anyway,
    // but with a raw git error and no way forward — say what to do instead.
    if dirty {
        anyhow::bail!(
            "local changes in {} — update stopped so nothing of yours is lost.\n\
             Commit or stash them, then re-run `omega update`:\n\
               git -C {} status\n\
               git -C {} stash",
            src.display(),
            src.display(),
            src.display()
        );
    }
    if ahead_n > 0 {
        anyhow::bail!(
            "your checkout has {} local commit(s) not on origin/{} — update stopped.\n\
             Push or rebase them first:\n\
               git -C {} log --oneline origin/{}..HEAD",
            ahead_n,
            branch,
            src.display(),
            branch
        );
    }

    if behind_n > 0 {
        println!("  fast-forwarding…");
        let ff = std::process::Command::new("git")
            .args(["merge", "--ff-only", &format!("origin/{}", branch)])
            .current_dir(&src)
            .output()?;
        if !ff.status.success() {
            anyhow::bail!(
                "fast-forward failed:\n{}",
                String::from_utf8_lossy(&ff.stderr).trim()
            );
        }
    }

    // install.sh rebuilds the binary from the pulled source and re-applies every
    // asset. It is idempotent and never clobbers user state.
    let installer = src.join("install.sh");
    if !installer.is_file() {
        anyhow::bail!("{} has no install.sh", src.display());
    }
    println!("  running install.sh (this rebuilds the binary from source)…\n");
    // OMEGA_FROM_SOURCE (install.sh's existing switch): `main` is normally
    // AHEAD of the latest release tag, and install.sh's "prefer source" gate
    // only triggers when a local target/release build already exists. Without
    // this, updating a fresh clone fetches the prebuilt artifact from that
    // OLDER tag and installs it over the source we just fast-forwarded — an
    // update that hands back older code (seen live 2026-07-16: the installed
    // binary knew 32 of 45 rules and exported stale doctrine over the current
    // one). Build from source so the binary matches the commit we pulled.
    let status = std::process::Command::new("bash")
        .arg(&installer)
        .current_dir(&src)
        .env("OMEGA_FROM_SOURCE", "1")
        .status()?;
    if !status.success() {
        anyhow::bail!("install.sh failed — your previous install is untouched");
    }

    println!("\n✓ OmegaOS updated. Restart a running TUI (Menu → R) to pick up the new binary.");
    Ok(())
}

/// The daily cron path: check, then install what is available — unattended.
///
/// Everything here is written for a run nobody is watching. It is quiet when
/// there is nothing to do (the normal day), it refuses rather than risks, and
/// every refusal is logged with its reason and — when the operator has to act —
/// pushed through the alert funnel. The decision itself lives in
/// `omega_core::auto_update::decide`, which is pure and unit-tested; this
/// function only gathers the facts and carries out the verdict.
// ---------------------------------------------------------------------------
// graph run — the driver the decision core was written for
// ---------------------------------------------------------------------------

/// What a node declares it wants run, if anything.
///
/// A node with no `command` is a HARD failure, never a silent success: a graph
/// whose nodes do nothing would otherwise report `Complete` and look like a
/// mission that ran, which is the worst possible answer to "did it work".
fn node_command(
    graph: &omega_core::graph::Graph,
    id: &omega_core::graph::NodeId,
) -> Option<String> {
    graph
        .node(id)?
        .extra
        .get("command")?
        .as_str()
        .map(|s| s.to_string())
}

/// Run one node's command and turn the process result into a report.
///
/// Exit 0 is the only success. Everything else carries a reason built from the
/// tail of stderr, because `advance` records that reason and a later
/// `Failed` outcome names the cause instead of telling the operator only that
/// something, somewhere, went wrong.
fn run_node(
    graph: &omega_core::graph::Graph,
    id: &omega_core::graph::NodeId,
    cwd: &std::path::Path,
) -> omega_core::graph_executor::NodeReport {
    use omega_core::graph_executor::NodeReport;

    let Some(command) = node_command(graph, id) else {
        return NodeReport::failed(
            id.clone(),
            "node declares no `command`, so the driver has nothing to run for it",
        );
    };

    match std::process::Command::new("bash")
        .arg("-c")
        .arg(&command)
        .current_dir(cwd)
        .output()
    {
        Ok(out) if out.status.success() => NodeReport::succeeded(id.clone()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let tail: String = stderr.lines().rev().take(3).collect::<Vec<_>>().join(" / ");
            NodeReport::failed(
                id.clone(),
                format!(
                    "exit {}: {}",
                    out.status.code().unwrap_or(-1),
                    if tail.is_empty() { "no stderr" } else { tail.trim() }
                ),
            )
        }
        Err(e) => NodeReport::failed(id.clone(), format!("could not spawn the command: {e}")),
    }
}

/// Drive a graph to a terminal outcome.
///
/// The loop is the one `docs/GRAPH-EXECUTION-LAYER.md` prescribes, with two
/// things the doc leaves to the caller made explicit here.
///
/// STATE IS PERSISTED AFTER EVERY STEP, before anything is dispatched. A run
/// killed mid-dispatch must resume knowing the node was attempted; writing
/// afterwards would hand a thrashing node a fresh retry budget on every crash,
/// which is exactly the unbounded loop the retry policy exists to prevent.
///
/// A HELD NODE STOPS THE RUN rather than being skipped. Skipping it would let
/// the graph converge around a step a human refused to authorize and report
/// `Complete` on a mission that never did the thing that mattered.
async fn cmd_graph_run(
    graph_path: &str,
    state_path: Option<&str>,
    unattended: bool,
    dry_run: bool,
    max_steps: usize,
) -> Result<()> {
    use omega_core::graph_executor::{advance, ExecutionOutcome, NodeReport};
    use omega_core::graph_risk::{evaluate_gate, ExecutionMode, GateDecision};

    let graph = load_graph(graph_path)?;
    graph
        .validate()
        .map_err(|e| anyhow::anyhow!("{} is not a runnable graph: {:?}", graph_path, e))?;

    // Default the state beside the graph so a resumed run needs no extra flag —
    // the common case is re-running the same command after an interruption.
    let default_state = format!("{}.state.json", graph_path);
    let state_path = state_path.unwrap_or(&default_state);
    let mut state = load_graph_state(Some(state_path), &graph)?;

    // Node commands run in the GRAPH's directory, not the caller's.
    //
    // A run whose steps resolve relative paths against whatever directory
    // somebody happened to invoke it from is not replayable, which is the one
    // property this whole layer is built to have. Found the hard way: a test
    // graph containing `echo dedupe: 4 -> 3` left a file named `3` in the repo
    // root, because the shell read the arrow as a redirect and the cwd was
    // wherever the command was typed. Anchoring to the graph makes a mission
    // self-contained and its side effects land where its author can see them.
    let graph_dir = std::path::Path::new(graph_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let mode = if unattended {
        ExecutionMode::Unattended
    } else {
        ExecutionMode::Attended
    };

    println!("◆ graph {} ({} nodes)", graph_path, graph.nodes.len());
    println!("  state: {}", state_path);
    println!("  mode:  {}", if unattended { "unattended" } else { "attended" });
    println!();

    let persist = |state: &omega_core::graph::GraphState| -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        let tmp = format!("{}.tmp", state_path);
        std::fs::write(&tmp, json).with_context(|| format!("writing {}", tmp))?;
        std::fs::rename(&tmp, state_path).with_context(|| format!("replacing {}", state_path))?;
        Ok(())
    };

    let mut reports: Vec<NodeReport> = Vec::new();

    for step_no in 1..=max_steps {
        let step = advance(&graph, &mut state, &reports)
            .map_err(|e| anyhow::anyhow!("executor refused the step: {:?}", e))?;
        persist(&state)?;
        reports.clear();

        for id in &step.retrying {
            println!("  [~] {} failed, retrying (budget left)", id.as_str());
        }
        for id in &step.exhausted {
            println!("  [x] {} failed terminally (retry budget spent)", id.as_str());
        }
        for id in &step.fallbacks {
            println!("  [>] fallback {} unlocked", id.as_str());
        }
        for (from, to) in &step.loops_taken {
            println!("  [o] loop edge {} -> {} traversed", from.as_str(), to.as_str());
        }

        match &step.outcome {
            ExecutionOutcome::Complete => {
                println!("\n✓ complete — every node settled, nothing failed unrecovered");
                return Ok(());
            }
            ExecutionOutcome::Blocked { unreachable } => {
                println!("\n✗ blocked — nothing can become ready. Unreachable:");
                for id in unreachable {
                    println!("    {}", id.as_str());
                }
                std::process::exit(1);
            }
            ExecutionOutcome::Failed { node, reason } => {
                println!("\n✗ failed — {} : {}", node.as_str(), reason);
                std::process::exit(1);
            }
            ExecutionOutcome::Progressing { ready } => {
                println!("  step {}: {} ready", step_no, ready.len());

                // GATE EVERY node before ANY of them runs. Gating inside the
                // dispatch loop would let the nodes ahead of a held one run
                // first, so a refusal would arrive after the run had already
                // moved — the approval would be for a state that no longer
                // exists.
                let mut authorized = Vec::new();
                for id in ready {
                    match evaluate_gate(&graph, &state, id, mode) {
                        GateDecision::Proceed => authorized.push(id.clone()),
                        GateDecision::RequireApproval {
                            node,
                            risk,
                            reason,
                            what_is_lost,
                        } => {
                            println!("\n⛔ HELD  {} ({:?})", node.as_str(), risk);
                            println!("   reason:       {}", reason);
                            println!("   what is lost: {}", what_is_lost);
                            if unattended {
                                // Unattended, a prompt is not a safety
                                // mechanism, it is a mission that stalls in
                                // silence. Leave the durable artifact instead.
                                let record = GateDecision::RequireApproval {
                                    node: node.clone(),
                                    risk,
                                    reason: reason.clone(),
                                    what_is_lost: what_is_lost.clone(),
                                }
                                .into_escalation(chrono::Utc::now());
                                if let Some(record) = record {
                                    let path = format!("{}.escalation.json", state_path);
                                    std::fs::write(&path, serde_json::to_string_pretty(&record)?)?;
                                    println!("   escalation written to {}", path);
                                }
                            }
                            println!(
                                "   resolve with: omega risk-gate approve {} {} --state {} --approver <who>",
                                graph_path,
                                node.as_str(),
                                state_path
                            );
                            std::process::exit(2);
                        }
                        GateDecision::Refuse { node, reason } => {
                            println!("\n⛔ REFUSED {} — {}", node.as_str(), reason);
                            println!("   the gate will not put this in front of a human as it stands; fix the graph");
                            std::process::exit(2);
                        }
                    }
                }

                if dry_run {
                    println!("\n(--dry-run) would run now, and nothing else was touched:");
                    for id in &authorized {
                        println!(
                            "    {}  {}",
                            id.as_str(),
                            node_command(&graph, id).unwrap_or_else(|| "<no command>".into())
                        );
                    }
                    return Ok(());
                }

                // The ready set is exactly what may run CONCURRENTLY — that is
                // the whole reason to express a mission as a graph rather than
                // a list, so running it serially here would throw the value
                // away while keeping the ceremony.
                reports = std::thread::scope(|scope| {
                    let handles: Vec<_> = authorized
                        .iter()
                        .map(|id| {
                            let graph = &graph;
                            let cwd = graph_dir.as_path();
                            scope.spawn(move || {
                                println!("    ▸ {}", id.as_str());
                                run_node(graph, id, cwd)
                            })
                        })
                        .collect();
                    handles
                        .into_iter()
                        .filter_map(|h| h.join().ok())
                        .collect::<Vec<_>>()
                });
            }
        }
    }

    anyhow::bail!(
        "stopped after {} steps without a terminal outcome — the graph's own bounds should have \
         ended it, so this is a driver bug worth reporting, not a graph to re-run",
        max_steps
    )
}

/// Stamp `auto-update.json` with the commit that was just installed.
///
/// Called by `install.sh` at the end of a successful install. Before this
/// existed, only the cron ever wrote that file, so every hand-run install left
/// it naming an older commit — and since the staleness check compares HEAD to
/// that field, a lie there is worse than an absence: it reports a stale binary
/// as current. Measured on the source box 2026-08-05, the field still named a
/// commit from five days and thirty commits earlier.
///
/// Deliberately quiet and non-fatal: an installer must not fail because it
/// could not write a bookkeeping file. A missing record degrades to "unknown
/// provenance", which `decide()` treats as "do not owe an install" rather than
/// as a rebuild loop.
/// Write every compiled rule to `rules_dir` as its own .md, and return how many.
///
/// Extracted so `omega rules export` and `omega reconcile` cannot drift: the
/// reconciler regenerating the doctrine differently from the command that
/// exports it is precisely the kind of silent divergence it exists to catch.
///
/// Idempotent: stale REGISTRY exports are pruned first so a re-export mirrors
/// the current registry exactly (no lingering old-id files when rules are
/// renamed or removed) — while .md files whose id is NOT in the compiled
/// registry survive: those are disk-only rules (install.sh copies repo rules/
/// before this export runs).
fn export_rules_to(rules_dir: &std::path::Path, verbose: bool) -> Result<usize> {
    use omega_core::rules::{self, RuleKind};
    std::fs::create_dir_all(rules_dir)?;

    // WRITE FIRST, PRUNE AFTER. Pruning first wipes every registered id and
    // trusts the write pass to put them back, which leaves a window where the
    // directory holds fewer rules than the registry — and anything reading
    // doctrine in that window sees a partial set. The parity test is one such
    // reader (it failed 2 in 25 under concurrent exports, measured), and an
    // agent being briefed while an install runs is the one that matters.
    let mut written: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    let all = rules::all_rules();
    for r in &all {
        let slug = r
            .title
            .to_lowercase()
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
        if verbose {
            println!("  [+] {}", fname);
        }
        written.insert(fname);
    }

    // Now the stale ones, sparing everything just written. A stale file and its
    // replacement share an ID and differ only in slug, so a reader in the
    // remaining window sees an extra retired FILE at worst, never a missing
    // current RULE.
    rules::prune_registered_exports_except(rules_dir, &written);
    Ok(all.len())
}

fn cmd_update_record_installed(dir: Option<&str>) -> Result<()> {
    use omega_core::auto_update::AutoUpdateState;

    let Some(src) = (match dir {
        Some(d) => Some(std::path::PathBuf::from(d)),
        None => resolve_omega_src(),
    }) else {
        eprintln!("omega update --record-installed: no OmegaOS checkout found, not recording");
        return Ok(());
    };

    let head = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&src)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if head.is_empty() {
        eprintln!(
            "omega update --record-installed: {} has no readable HEAD, not recording",
            src.display()
        );
        return Ok(());
    }

    let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
    let state_dir = config.state_dir.clone();
    let mut history = AutoUpdateState::load(&state_dir);
    history.record_success(&head, chrono::Utc::now());
    history.last_outcome = Some(format!("installed {} from {}", head, src.display()));

    match history.save(&state_dir) {
        Ok(()) => println!("  recorded installed commit: {}", head),
        Err(e) => eprintln!(
            "omega update --record-installed: could not write state ({e}) — \
             the staleness check will read this install as unknown provenance"
        ),
    }
    Ok(())
}

/// Bring everything AROUND the binary back in line after an update, then say
/// what is left that a human has to decide.
///
/// Ordered so the cheap deterministic repairs land before anything is judged:
/// a report written against un-regenerated doctrine would accuse the machine of
/// drift the reconciler was about to fix itself.
async fn cmd_reconcile(report_only: bool) -> Result<()> {
    let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
    let mut needs_human: Vec<String> = Vec::new();

    println!(
        "◆ OmegaOS reconcile{}",
        if report_only { " (report only)" } else { "" }
    );
    println!();

    // ---- Mechanical, fixed in place -------------------------------------
    println!("  Mechanical");
    if report_only {
        println!("    (skipped — report-only)");
    } else {
        // Doctrine on disk must mirror the registry compiled into THIS binary.
        // A new binary with the previous binary's rule files on disk is the
        // most common post-update drift, and the one no one ever notices.
        let rules_dir = omega_core::config::omega_dir().join("rules");
        match export_rules_to(&rules_dir, false) {
            Ok(n) => println!("    [+] doctrine re-exported ({} rules)", n),
            Err(e) => needs_human.push(format!("could not re-export doctrine: {e}")),
        }

        // Whatever installed this binary may not have recorded which commit it
        // was. Without that record nothing downstream can prove staleness.
        if let Err(e) = cmd_update_record_installed(None) {
            needs_human.push(format!("could not record the installed commit: {e}"));
        }

        // Workers that finished but whose pane was never closed.
        if let Err(e) = cmd_reap(None, false).await {
            needs_human.push(format!("reap failed: {e}"));
        }
    }
    println!();

    // ---- Judgement, reported only ---------------------------------------
    println!("  Needs a human");

    // Registered projects that no longer exist, or carry no canon doc. Both are
    // real, both are recoverable, and neither is safe to "fix" automatically:
    // deleting a registry entry and writing a project's CLAUDE.md are the
    // operator's calls.
    let registry = omega_core::project_manager::ProjectRegistry::load();
    for p in &registry.projects {
        if !p.path.exists() {
            needs_human.push(format!(
                "project '{}' is registered at {} which does not exist",
                p.name,
                p.path.display()
            ));
            continue;
        }
        let has_doc = ["CLAUDE.md", "AGENTS.md", "OMEGA.md"]
            .iter()
            .any(|f| p.path.join(f).is_file());
        if !has_doc {
            needs_human.push(format!(
                "project '{}' has no CLAUDE.md / AGENTS.md / OMEGA.md — agents get no project doctrine there",
                p.name
            ));
        }
    }

    // Sessions older than the binary were briefed by a PREVIOUS build: the
    // doctrine block is injected at spawn time, so a running session keeps
    // whatever was compiled in when it started. Restarting one mid-turn would
    // destroy work, so this is named, never acted on.
    //
    // The comparison is binary MTIME, which over-reports and must say so.
    // A rebuild that touched no rule leaves the doctrine byte-identical, yet
    // every session started before it still looks older — measured 2026-08-05,
    // a 29-line change to main.rs flagged nine sessions whose doctrine had not
    // moved at all. It never under-reports, which is the safe direction, but a
    // check that cries wolf is the one defect an alerting system cannot afford
    // (R-MONITOR): the operator learns to skip the line, and then it may as
    // well not exist. So the wording claims only what mtime can prove. Making
    // this exact would mean recording a doctrine fingerprint per install and
    // comparing against the last build whose fingerprint actually changed.
    // The cutoff is the last install whose DOCTRINE actually changed, not the
    // last install. Binary mtime over-reported badly: a 29-line CLI change,
    // touching no rule, flagged nine live sessions whose doctrine block was
    // byte-identical (measured 2026-08-05). A check that cries wolf teaches the
    // operator to skip the line, and then it may as well not exist (R-MONITOR).
    //
    // No recorded change means no install has yet observed a doctrine SHIFT on
    // this box, so there is nothing a session could be behind on. Falling back
    // to mtime here would reintroduce the false positive on exactly the
    // machines that just upgraded into this version.
    let history = omega_core::auto_update::AutoUpdateState::load(&config.state_dir);
    match history.doctrine_changed_at {
        Some(changed_at) => {
            let stale = sessions_older_than(changed_at);
            if !stale.is_empty() {
                needs_human.push(format!(
                    "{} session(s) predate the last doctrine change ({}) and were briefed with \
                     the rules as they stood before it (restart them once their work is done): {}",
                    stale.len(),
                    changed_at.format("%Y-%m-%d %H:%M UTC"),
                    stale.join(", ")
                ));
            }
        }
        None => {
            // Say it rather than stay silent: "no finding" and "cannot yet
            // tell" are different facts, and only one of them is reassuring.
            println!(
                "    [i] no doctrine change recorded yet on this box, so no session can be \
                 measured against one — the next install that moves a rule sets the mark"
            );
        }
    }

    // Everything doctor already knows how to see, folded in rather than
    // re-derived — including the binary-provenance check.
    for c in omega_core::doctor::run_all(&config).await {
        if c.health == omega_core::doctor::Health::Fail {
            needs_human.push(format!("doctor: {} — {}", c.name, c.detail));
        }
    }

    if needs_human.is_empty() {
        println!("    nothing — this install is coherent");
        println!();
        return Ok(());
    }
    for item in &needs_human {
        println!("    [!] {}", item);
    }
    println!();
    println!("  {} item(s) need a human.", needs_human.len());
    std::process::exit(1);
}

/// Live sessions created before `cutoff`, by name.
///
/// The creation time comes from rmux itself (`#{session_created}`, a unix
/// timestamp) rather than the session SDK, which does not carry one. A session
/// that cannot be read is deliberately NOT reported as stale: a false "restart
/// this" against a session doing real work is far worse than missing one.
fn sessions_older_than(cutoff: chrono::DateTime<chrono::Utc>) -> Vec<String> {
    let out = std::process::Command::new(omega_core::stream::rmux_bin())
        .args(["list-sessions", "-F", "#{session_name} #{session_created}"])
        .output();
    let Ok(out) = out else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let (name, ts) = line.rsplit_once(' ')?;
            let secs: i64 = ts.trim().parse().ok()?;
            let created = chrono::DateTime::from_timestamp(secs, 0)?;
            (created < cutoff).then(|| name.trim().to_string())
        })
        .collect()
}


async fn cmd_update_auto(dir: Option<&str>) -> Result<()> {
    use omega_core::auto_update::{decide, AutoUpdateState, CheckoutState, Decision, SkipReason};

    let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
    let state_dir = config.state_dir.clone();
    let stamp = chrono::Utc::now();

    // A line per run, timestamped, in the cron's own log.
    let say = |msg: &str| println!("[{}] auto-update: {}", stamp.to_rfc3339(), msg);

    if config.auto_update == omega_core::config::AutoUpdatePolicy::Off {
        say(&Decision::Disabled.describe());
        return Ok(());
    }

    // Single-flight. A rebuild can outlast a day if the machine is slow or the
    // network stalls; two installs writing the same binary is how you get a
    // half-written omega. A lock older than 6h is stale (killed cron) and is
    // taken over, so a crash can never wedge updates forever.
    let lock_path = config.locks_dir.join("auto-update.lock");
    std::fs::create_dir_all(&config.locks_dir).ok();
    // Claim it with create_new, which is atomic: checking then writing left a
    // window where two runs starting together both saw no lock and both
    // proceeded to rebuild the same binary.
    let claim = |path: &std::path::Path| -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        write!(f, "{}", std::process::id())
    };
    if claim(&lock_path).is_err() {
        // Someone holds it. Only a lock old enough to be a crashed run is
        // taken over — otherwise a slow-but-live rebuild would be trampled.
        let age = std::fs::metadata(&lock_path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|m| m.elapsed().ok())
            .unwrap_or_default();
        if age < std::time::Duration::from_secs(6 * 3600) {
            say("another update is already running — skipping this run");
            return Ok(());
        }
        say("clearing a stale update lock (older than 6h)");
        std::fs::remove_file(&lock_path).ok();
        if claim(&lock_path).is_err() {
            say("could not claim the update lock — skipping this run");
            return Ok(());
        }
    }
    // Released on every exit path below via this guard.
    struct LockGuard(std::path::PathBuf);
    impl Drop for LockGuard {
        fn drop(&mut self) {
            std::fs::remove_file(&self.0).ok();
        }
    }
    let _lock = LockGuard(lock_path);

    let src = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None => match resolve_omega_src() {
            Some(p) => p,
            None => {
                // Nothing to update from. Not an error worth alerting daily —
                // an npx install with no checkout is a legitimate setup.
                say("no OmegaOS checkout found — nothing to update (install with: npx omega-os)");
                return Ok(());
            }
        },
    };
    if !src.join(".git").exists() {
        say(&format!(
            "{} has no .git — cannot update in place",
            src.display()
        ));
        return Ok(());
    }

    let git = |args: &[&str]| -> String {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&src)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };

    let branch = {
        let b = git(&["rev-parse", "--abbrev-ref", "HEAD"]);
        if b.is_empty() || b == "HEAD" {
            "main".to_string()
        } else {
            b
        }
    };

    let fetch = std::process::Command::new("git")
        .args(["fetch", "origin", &branch])
        .current_dir(&src)
        .output()?;
    if !fetch.status.success() {
        // A flaky network is the single most likely daily failure. It is not
        // worth waking anyone: tomorrow's run retries by itself.
        say(&format!(
            "fetch failed (network/credentials) — retrying tomorrow: {}",
            String::from_utf8_lossy(&fetch.stderr).trim()
        ));
        return Ok(());
    }

    let checkout = CheckoutState {
        behind: git(&["rev-list", "--count", &format!("HEAD..origin/{}", branch)])
            .parse()
            .unwrap_or(0),
        ahead: git(&["rev-list", "--count", &format!("origin/{}..HEAD", branch)])
            .parse()
            .unwrap_or(0),
        dirty: !git(&["status", "--porcelain"]).is_empty(),
        target: git(&["rev-parse", "--short", &format!("origin/{}", branch)]),
        head: git(&["rev-parse", "--short", "HEAD"]),
    };

    let mut history = AutoUpdateState::load(&state_dir);
    history.last_check = Some(stamp);

    // Only ask whether an agent is busy when an update is actually pending —
    // capturing panes on every session costs real time, daily, for nothing.
    let working = if checkout.behind > 0 {
        busy_agent_session().await
    } else {
        None
    };

    let decision = decide(config.auto_update, &checkout, &history, working.as_deref());
    let summary = decision.describe();
    say(&summary);

    match decision {
        Decision::Disabled | Decision::UpToDate => {
            history.last_outcome = Some(summary);
            history.save(&state_dir).ok();
            Ok(())
        }
        Decision::NotifyOnly { behind, target } => {
            history.last_outcome = Some(summary);
            history.save(&state_dir).ok();
            alert(&format!(
                "⬆️ <b>OmegaOS update available</b>\n{} commit(s) behind (<code>{}</code>).\nInstall it with <code>omega update</code>.\n<i>auto_update = check</i>",
                behind, target
            ));
            Ok(())
        }
        Decision::Skip { reason } => {
            history.last_outcome = Some(reason.describe());
            history.save(&state_dir).ok();
            if reason.needs_human() {
                let extra = match &reason {
                    SkipReason::RepeatedFailure { .. } => {
                        "\nSee <code>~/.omega/logs/omega-auto-update.log</code>."
                    }
                    _ => "",
                };
                alert(&format!(
                    "⚠️ <b>OmegaOS auto-update skipped</b>\n{}{}",
                    reason.describe(),
                    extra
                ));
            }
            Ok(())
        }
        Decision::Apply { behind, target } => {
            let from = git(&["rev-parse", "--short", "HEAD"]);
            let ff = std::process::Command::new("git")
                .args(["merge", "--ff-only", &format!("origin/{}", branch)])
                .current_dir(&src)
                .output()?;
            if !ff.status.success() {
                history.record_failure(&target);
                history.last_outcome = Some("fast-forward failed".to_string());
                history.save(&state_dir).ok();
                say(&format!(
                    "fast-forward failed: {}",
                    String::from_utf8_lossy(&ff.stderr).trim()
                ));
                return Ok(());
            }

            say("running install.sh (rebuilds the binary from source)…");
            let status = std::process::Command::new("bash")
                .arg(src.join("install.sh"))
                .current_dir(&src)
                .env("OMEGA_FROM_SOURCE", "1")
                .status()?;

            if !status.success() {
                history.record_failure(&target);
                let failures = history.failures_for(&target);
                history.last_outcome = Some(format!("install.sh failed ({}x)", failures));
                history.save(&state_dir).ok();
                say(&format!(
                    "install.sh FAILED on {} (attempt {} of {}) — previous install untouched",
                    target,
                    failures,
                    omega_core::auto_update::FAILURE_CAP
                ));
                // Only shout once the cap is reached; a single transient build
                // failure retries itself tomorrow without waking anyone.
                if failures >= omega_core::auto_update::FAILURE_CAP {
                    alert(&format!(
                        "🛑 <b>OmegaOS auto-update is stuck</b>\ncommit <code>{}</code> failed to install {} times — not retrying.\nRun <code>omega update</code> by hand to see why.",
                        target, failures
                    ));
                }
                return Ok(());
            }

            history.record_success(&target, stamp);
            history.last_outcome = Some(format!("updated {} → {}", from, target));

            // This save is the ONLY thing that stops a nightly rebuild loop,
            // so its failure can no longer be swallowed.
            //
            // Since an install is now owed whenever HEAD is not the recorded
            // commit, a machine that installs successfully but cannot WRITE
            // that record owes the same install again tomorrow, and every
            // night after, forever. `record_failure` never fires — the install
            // succeeded — so FAILURE_CAP never engages and nothing bounds it.
            // And a state dir we cannot write is a state dir no bookkeeping
            // can bound, so the only honest ceiling is a human (R-LOOP).
            // This lands on every install that auto-updates, client boxes
            // included, which is exactly why it gets an alert and not a
            // swallowed `.ok()`.
            if let Err(e) = history.save(&state_dir) {
                say(&format!(
                    "installed {} but COULD NOT record it ({e}) — this box will reinstall \
                     every night until {} is writable",
                    target,
                    state_dir.display()
                ));
                alert(&format!(
                    "⚠️ <b>OmegaOS updated but cannot remember it</b>\ninstalled <code>{}</code>, but writing <code>{}</code> failed: {}\nUntil that is fixed this machine rebuilds itself every night.",
                    target,
                    state_dir.display(),
                    e
                ));
            }

            say(&format!(
                "updated {} → {} ({} commits)",
                from, target, behind
            ));

            // A new binary is not a coherent install. The doctrine on disk is
            // still the previous binary's, finished workers may still hold
            // panes, and every session running right now was briefed by the
            // binary we just replaced. Reconcile in its OWN detached session
            // rather than inline: the cron must not sit through it, and the
            // operator can read what it found with `omega stream`. Only ever
            // launched after an install that actually happened, so a machine
            // with nothing to update spawns nothing.
            let reconcile_report = spawn_reconcile_session();

            alert(&format!(
                "✅ <b>OmegaOS updated</b>\n<code>{}</code> → <code>{}</code> ({} commit(s)).\n{}\nRestart a running TUI (Menu → R) to pick up the new binary.",
                from, target, behind, reconcile_report
            ));
            Ok(())
        }
    }
}

/// Launch the post-update reconciliation in its own detached rmux session.
///
/// Detached on purpose: the caller is a cron at 03:30 that must not block, and
/// a session is watchable afterwards (`omega stream omega-reconcile`) whereas
/// inline output vanishes into a log nobody opens. Returns the sentence to put
/// in the operator's alert — never an error, because failing to launch the
/// tidy-up must not turn a SUCCESSFUL update into a reported failure.
fn spawn_reconcile_session() -> String {
    const NAME: &str = "omega-reconcile";
    let rmux = omega_core::stream::rmux_bin();

    // One reconciler at a time. A second one racing the first would re-export
    // the same doctrine underneath it and double-report every finding.
    let already = std::process::Command::new(&rmux)
        .args(["has-session", "-t", NAME])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if already {
        return format!(
            "Reconcile already running (<code>omega stream {}</code>).",
            NAME
        );
    }

    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "omega".to_string());
    // `read` holds the pane open so the report is still there to read; without
    // it the session dies the instant reconcile exits and the findings are lost.
    let cmd =
        format!("{exe} reconcile; echo; echo '[reconcile done — press Enter to close]'; read _");

    match std::process::Command::new(&rmux)
        .args(["new-session", "-d", "-s", NAME, &cmd])
        .status()
    {
        Ok(s) if s.success() => format!(
            "Reconciling this install in session <code>{}</code> — read it with <code>omega stream {}</code>.",
            NAME, NAME
        ),
        _ => format!(
            "Could not launch the reconcile session — run <code>omega reconcile</code> by hand."
        ),
    }
}

/// Name of an agent session that is mid-turn right now, if any.
///
/// Only Oracle/Worker sessions count: a Home shell sitting at a prompt is not
/// work, and deferring the update because someone left a shell open would mean
/// never updating. Any failure to look is treated as "not busy" — a broken
/// probe must not silently stop every update.
async fn busy_agent_session() -> Option<String> {
    use omega_core::session::{SessionManager, SessionRole};

    let mgr = SessionManager::connect_cached().await.ok()?;
    let sessions = mgr.list_sessions().await.ok()?;
    for session in sessions {
        if !matches!(session.role, SessionRole::Oracle | SessionRole::Worker) {
            continue;
        }
        if let Ok(pane) = mgr.capture_pane(&session.name).await {
            if omega_core::session_monitor::working_indicator_visible(&pane) {
                return Some(session.name);
            }
        }
    }
    None
}

/// Send an operational alert through the canonical funnel (the Telegram
/// "Alerts" topic). Best-effort by design: the update itself already happened,
/// and a machine with no Telegram configured must not see a failure here.
fn alert(html: &str) {
    let script = omega_core::config::omega_dir().join("bin/omega-alert-send.sh");
    if !script.is_file() {
        return;
    }
    let _ = std::process::Command::new("bash")
        .arg(script)
        .arg(html)
        .status();
}

/// Re-exported from omega-core so the CLI and `doctor` can never disagree about
/// where the checkout is — see `config::resolve_omega_src`.
fn resolve_omega_src() -> Option<std::path::PathBuf> {
    omega_core::config::resolve_omega_src()
}

/// Prune dangling omega-managed symlinks from a ~/.claude integration dir.
///
/// sync only ever CREATED links, so every rule rename/removal left its old
/// link pointing at a deleted ~/.omega file forever. Scope strictly to links
/// whose target is INSIDE ~/.omega — user-managed links are never touched.
/// `symlink_metadata` succeeding while `exists()` (which follows the link)
/// fails is the dangling test.
fn prune_dangling_omega_links(dir: &std::path::Path, omega_dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.file_type().is_symlink() {
            continue;
        }
        let Ok(target) = std::fs::read_link(&path) else {
            continue;
        };
        if target.starts_with(omega_dir) && !path.exists() && std::fs::remove_file(&path).is_ok() {
            println!("  [-] pruned dangling link: {}", path.display());
        }
    }
}

fn cmd_sync() -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let omega_dir = omega_core::config::omega_dir();

    // Ensure master dirs exist
    for sub in &[
        "rules",
        "agents",
        "agents/aisb",
        "skills",
        "hooks",
        "plugins",
        "docs",
        "projects",
        "state",
        "logs",
    ] {
        std::fs::create_dir_all(omega_dir.join(sub))?;
    }

    // Export rules unconditionally: Export now prunes only registry-owned .md
    // files (disk-only rules survive), so re-running sync refreshes renamed or
    // updated rules instead of silently keeping a stale set (the old
    // only-when-empty guard meant sync NEVER updated rules after first run).
    let rules_dir = omega_dir.join("rules");
    println!("Exporting rules...");
    cmd_rules(RulesAction::Export)?;

    // Resolve the repo checkout ONCE for every repo-sourced step below — the
    // old bare relative paths only worked with the repo root as CWD, so
    // `omega sync` from anywhere else silently skipped OMEGA.md/agents/pdfgen.
    let repo_src = resolve_omega_src();
    if repo_src.is_none() {
        println!("[i] OmegaOS repo checkout not found — skipping OMEGA.md/agents/pdfgen sync");
    }
    if let Some(src) = &repo_src {
        let root = src.join("skills");
        if root.is_dir() {
            // Reconcile the shipped native tree before compiling or linking the
            // catalog. No --delete: externally installed skills (for example
            // Agent Reach or a private pack) remain valid installed additions.
            let installed_skills = omega_dir.join("skills");
            let status = std::process::Command::new("rsync")
                .args(["-a", "--exclude=node_modules", "--exclude=.next"])
                .arg(format!("{}/", root.display()))
                .arg(format!("{}/", installed_skills.display()))
                .status()
                .context("failed to execute rsync while synchronizing native skills")?;
            if !status.success() {
                anyhow::bail!("native skill synchronization failed with status {}", status);
            }
            println!("[+] Native skills synced to {}", installed_skills.display());

            use omega_core::skill_registry::{OwnedSkillRoot, SkillCatalogV1};
            let catalog = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", &root)])?;
            let output = omega_dir.join("skill-catalog-v1.json");
            catalog.write_json(&output)?;
            println!(
                "[+] SkillCatalogV1: {} skills, sha256:{} → {}",
                catalog.skills.len(),
                catalog.content_digest,
                output.display()
            );
        }
    }

    // Copy OMEGA.md to ~/.omega/ (the dst is also the Codex symlink target below,
    // so it lives outside the repo-scoped block)
    let omega_md_dst = omega_dir.join("OMEGA.md");
    if let Some(src) = &repo_src {
        let omega_md_src = src.join("OMEGA.md");
        if omega_md_src.exists() {
            std::fs::copy(&omega_md_src, &omega_md_dst)?;
            println!("[+] OMEGA.md → {}", omega_md_dst.display());
        }
    }

    // Copy agents from repo if available
    if let Some(src) = &repo_src {
        let agents_src = src.join("agents");
        if agents_src.exists() {
            let agents_dst = omega_dir.join("agents");
            std::fs::create_dir_all(agents_dst.join("aisb"))?;
            for entry in std::fs::read_dir(&agents_src).into_iter().flatten() {
                let entry = entry?;
                let dst = agents_dst.join(entry.file_name());
                if entry.file_type()?.is_dir() {
                    // aisb/ subdirectory
                    for sub in std::fs::read_dir(entry.path()).into_iter().flatten() {
                        let sub = sub?;
                        if sub.file_name().to_string_lossy().ends_with(".md") {
                            std::fs::copy(
                                sub.path(),
                                agents_dst.join("aisb").join(sub.file_name()),
                            )?;
                        }
                    }
                } else if entry.file_name().to_string_lossy().ends_with(".md") {
                    std::fs::copy(entry.path(), &dst)?;
                }
            }
            println!("[+] Agents synced to {}", agents_dst.display());
        }
    }

    // Sync pdfgen from the repo — UNCONDITIONALLY, not only on first install.
    // R-PDF promises "`omega sync` re-links it": template/theme improvements
    // in tools/pdfgen must reach the installed copy, so an existing
    // bin/pdfgen.ts is no longer a skip condition.
    if let Some(src) = &repo_src {
        let skills_src = src.join("tools/pdfgen");
        let skills_dst = omega_dir.join("skills/pdfgen");
        if skills_src.exists() {
            std::fs::create_dir_all(&skills_dst)?;
            let status = std::process::Command::new("rsync")
                .args([
                    "-a",
                    "--exclude=node_modules",
                    "--exclude=.next",
                    "--exclude=output",
                ])
                .arg(format!("{}/", skills_src.display()))
                .arg(format!("{}/", skills_dst.display()))
                .status();
            if let Ok(s) = status {
                if s.success() {
                    println!("[+] PDF generator synced to {}", skills_dst.display());
                }
            }
        }
    }

    // ── Claude Code integration ──
    let claude_dir = home.join(".claude");
    if claude_dir.exists() {
        // Rules: symlink each omega rule with omega- prefix
        let claude_rules = claude_dir.join("rules");
        std::fs::create_dir_all(&claude_rules)?;
        // Prune BEFORE linking: a renamed/removed rule leaves its old
        // omega-* link dangling forever otherwise (sync only creates).
        prune_dangling_omega_links(&claude_rules, &omega_dir);
        for entry in std::fs::read_dir(&rules_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.ends_with(".md") {
                continue;
            }
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
        // Same prune as rules: a deleted ~/.omega/skills dir must take its
        // ~/.claude/skills link with it on the next sync.
        prune_dangling_omega_links(&claude_skills, &omega_dir);
        if skills_dir.exists() {
            for entry in std::fs::read_dir(&skills_dir)? {
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                let name = entry.file_name();
                let link = claude_skills.join(&name);
                if !link.exists() {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(entry.path(), &link)?;
                    println!("  [+] Claude skill: {}", name.to_string_lossy());
                }
                // Dedupe the slash-command menu: a skill is invocable as
                // /<name> by itself, so a bare ~/.claude/commands/<name>.md
                // stub for the SAME name lists the command twice (the /omg-*
                // alias stub keeps its own namespace and stays). Remove the
                // shadowed bare stub.
                let bare_stub = claude_dir
                    .join("commands")
                    .join(format!("{}.md", name.to_string_lossy()));
                if bare_stub.exists() {
                    let _ = std::fs::remove_file(&bare_stub);
                    println!(
                        "  [-] pruned duplicate command stub: {}.md (skill of the same name is linked)",
                        name.to_string_lossy()
                    );
                }
            }
        }
        println!("[+] Claude Code synced (rules + skills)");
    }

    // ── Gemini CLI integration ──
    let gemini_dir = home.join(".gemini");
    if gemini_dir.exists() {
        let gemini_md = gemini_dir.join("GEMINI.md");
        // Same reasoning as the Codex block below: import the generated
        // full-doctrine AGENTS.md, not the OMEGA.md summary.
        let agents_import = format!("@import {}", omega_dir.join("AGENTS.md").display());
        let omega_ref = format!("\n# OmegaOS\n{agents_import}\n");
        if gemini_md.exists() {
            let content = std::fs::read_to_string(&gemini_md)?;
            if content.contains("@import ~/.omega/OMEGA.md")
                || (content.contains("@import ~/.omega/AGENTS.md")
                    && !content.contains(&agents_import))
            {
                // Upgrade an install that predates the full-doctrine file.
                let upgraded = content
                    .replace("@import ~/.omega/OMEGA.md", &agents_import)
                    .replace("@import ~/.omega/AGENTS.md", &agents_import);
                std::fs::write(&gemini_md, upgraded)?;
                println!("[+] Gemini: GEMINI.md import upgraded to the full doctrine");
            } else if !content.contains("OmegaOS") {
                std::fs::write(&gemini_md, format!("{}{}", content, omega_ref))?;
                println!("[+] Gemini: appended OmegaOS reference to GEMINI.md");
            }
        } else {
            std::fs::write(&gemini_md, omega_ref)?;
            println!("[+] Gemini: created GEMINI.md → OmegaOS");
        }
    }

    // ── Codex integration ──
    //
    // Codex and Gemini load one global instructions file. Compile a compact,
    // provider-neutral kernel here; domain runbooks and selected skills remain
    // discoverable on demand. Injecting the historical full doctrine duplicated
    // 60-70 KB into every session and reduced adherence.
    let agents_full_dst = omega_dir.join("AGENTS.md");
    {
        let base = std::fs::read_to_string(&omega_md_dst).unwrap_or_default();
        let compiled =
            omega_core::rules::compile_rule_context(omega_core::rules::RuleScope::Worker, None)
                .map_err(|error| anyhow::anyhow!("rule context compile failed: {error}"))?;
        let generated = format!(
            "{}\n\n---\n\n# ACTIVE OMEGAOS POLICY KERNEL (generated by `omega sync`, do not edit)\n\n\
             Doctrine hash: `{}`. Full historical policy and runbooks remain available through \
             `omega rules list`, `omega rules context`, and `~/.omega/rules/`.\n\n{}",
            base.trim_end(),
            compiled.digest,
            compiled.markdown
        );
        std::fs::write(&agents_full_dst, generated)?;
        println!(
            "[+] AGENTS.md (OMEGA.md + compact policy kernel, {} bytes) → {}",
            compiled.bytes,
            agents_full_dst.display()
        );
    }

    let codex_dir = home.join(".codex");
    if codex_dir.exists() || std::fs::create_dir_all(&codex_dir).is_ok() {
        let agents_md = codex_dir.join("AGENTS.md");
        // Repoint an existing OMEGA.md-only link (or a stale copy) at the
        // full-doctrine file. A hand-written regular file is left alone.
        let is_omega_link = std::fs::read_link(&agents_md)
            .map(|t| t != agents_full_dst)
            .unwrap_or(false);
        if is_omega_link {
            let _ = std::fs::remove_file(&agents_md);
        }
        if !agents_md.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&agents_full_dst, &agents_md)?;
            println!(
                "[+] Codex: AGENTS.md → {} (compact policy kernel)",
                agents_full_dst.display()
            );
        }
    }

    // Codex activates reusable skills from the provider-neutral
    // ~/.agents/skills directory. Link every skill the canonical registry can
    // parse, including categorized/nested entries. Mentioning a slash command
    // in AGENTS.md alone does not make a skill discoverable by Codex.
    let codex_skills = home.join(".agents").join("skills");
    std::fs::create_dir_all(&codex_skills)?;
    prune_dangling_omega_links(&codex_skills, &omega_dir);
    let skills_dir = omega_dir.join("skills");
    if skills_dir.exists() {
        let registry = omega_core::skill_registry::SkillRegistry::discover(&skills_dir)?;
        for skill in registry.list() {
            let Some(skill_dir) = skill.path.parent() else {
                continue;
            };
            let link = codex_skills.join(&skill.name);
            if link.exists() {
                continue;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(skill_dir, &link)?;
            println!("  [+] Codex skill: ${}", skill.name);
        }
        println!(
            "[+] Codex skills synced: {} canonical entries → {}",
            registry.count(),
            codex_skills.display()
        );
    }

    println!(
        "\n[+] OmegaOS sync complete: all LLMs reference {}",
        omega_dir.display()
    );
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

#[cfg(test)]
mod phase1_tests {
    use super::*;

    #[test]
    fn same_index_session_identity_change_refreshes_preview() {
        assert!(should_refresh_preview_after_event(
            0,
            0,
            omega_tui::app::Tab::Sessions,
            omega_tui::app::Tab::Sessions,
            Some("post-refresh-session-a-e941357"),
            Some("post-refresh-session-b-e941357"),
        ));
    }

    #[test]
    fn codex_login_status_json_shape_is_stable() {
        let success = omega_core::codex_login::FinishResult {
            status: omega_core::codex_login::LoginStatus::LoggedIn {
                mode: "ChatGPT".to_string(),
            },
            restored: false,
            flow_succeeded: true,
        };
        assert_eq!(
            codex_login_status_json(&success),
            serde_json::json!({
                "ok": true,
                "status": "logged in using ChatGPT",
                "restored": false
            })
        );

        let restored_without_fresh_credential = omega_core::codex_login::FinishResult {
            status: omega_core::codex_login::LoginStatus::LoggedIn {
                mode: "API key".to_string(),
            },
            restored: true,
            flow_succeeded: false,
        };
        assert_eq!(
            codex_login_status_json(&restored_without_fresh_credential),
            serde_json::json!({
                "ok": false,
                "status": "logged in using API key",
                "restored": true
            })
        );

        let unknown = omega_core::codex_login::FinishResult {
            status: omega_core::codex_login::LoginStatus::Unknown {
                reason: "reason".to_string(),
            },
            restored: false,
            flow_succeeded: false,
        };
        assert_eq!(
            codex_login_status_json(&unknown),
            serde_json::json!({
                "ok": false,
                "status": "unknown: reason",
                "restored": false
            })
        );
    }

    #[test]
    fn codex_login_abort_json_reports_command_success_not_login_success() {
        let aborted = omega_core::codex_login::AbortResult {
            status: omega_core::codex_login::LoginStatus::LoggedIn {
                mode: "API key".to_string(),
            },
            restored: true,
            flow_succeeded: false,
            aborted: true,
        };
        assert_eq!(
            codex_login_abort_json(&aborted),
            serde_json::json!({
                "ok": true,
                "aborted": true,
                "status": "logged in using API key",
                "restored": true
            })
        );

        let unknown = omega_core::codex_login::AbortResult {
            status: omega_core::codex_login::LoginStatus::Unknown {
                reason: "PID identity changed".to_string(),
            },
            restored: false,
            flow_succeeded: false,
            aborted: false,
        };
        assert_eq!(
            codex_login_abort_json(&unknown),
            serde_json::json!({
                "ok": false,
                "aborted": false,
                "status": "unknown: PID identity changed",
                "restored": false
            })
        );

        let completed_before_abort = omega_core::codex_login::AbortResult {
            status: omega_core::codex_login::LoginStatus::LoggedIn {
                mode: "ChatGPT".to_string(),
            },
            restored: false,
            flow_succeeded: true,
            aborted: false,
        };
        assert_eq!(
            codex_login_abort_json(&completed_before_abort),
            serde_json::json!({
                "ok": true,
                "aborted": false,
                "status": "logged in using ChatGPT",
                "restored": false
            })
        );
    }

    #[test]
    fn explicit_codex_settlement_commands_own_startup_reconciliation() {
        assert!(command_owns_codex_reconciliation(&Some(
            Commands::CodexLoginStatus { pid: None }
        )));
        assert!(command_owns_codex_reconciliation(&Some(
            Commands::CodexReconcile { json: true }
        )));
        assert!(!command_owns_codex_reconciliation(&Some(
            Commands::Doctor {
                pre_reset: false,
                fix: false,
                deep: false,
            }
        )));
    }

    #[test]
    fn verifier_contract_parser_accepts_direct_argv_and_rejects_shell_operators() {
        assert_eq!(
            declared_verify_command(
                "Done Criteria: green\nVerify Command: cargo test -p omega-core"
            ),
            Some(vec![
                "cargo".to_string(),
                "test".to_string(),
                "-p".to_string(),
                "omega-core".to_string(),
            ])
        );
        assert!(declared_verify_command(
            "Done Criteria: green\nVerify Command: cargo test && curl example.test"
        )
        .is_none());
    }

    #[test]
    fn v3_worker_preparation_freezes_plan_and_queues_attempt_before_spawn() {
        let state_dir = std::env::temp_dir().join(format!(
            "omega-v3-worker-preparation-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_micros()
        ));
        std::fs::create_dir_all(&state_dir).unwrap();
        let mut config = OmegaConfig::default();
        config.state_dir = state_dir.clone();

        let mission =
            omega_core::mission::Mission::new("OmegaOS", "implement safely", state_dir.clone());
        let ledger = omega_core::mission_ledger::MissionLedger::open(
            state_dir.join("mission-engine-v3.sqlite3"),
        )
        .unwrap();
        ledger
            .create_mission(&mission, "test-create", "test")
            .unwrap();
        let mut classified = omega_core::mission_ledger::AppendEvent::new(
            mission.id.clone(),
            1,
            "test-classified",
            "test",
            "mission_classified",
        );
        classified.next_mission_state = Some(omega_core::mission::MissionState::Classified);
        ledger.append(classified).unwrap();
        let oracle_name = "oracle-OmegaOS-test";
        omega_core::oracle_lifecycle::OracleState::new(oracle_name, &mission)
            .write(&state_dir)
            .unwrap();

        let attempt = prepare_v3_worker_attempt(
            &config,
            Some(oracle_name),
            "OmegaOS-worker-core",
            "core",
            "Done Criteria: ledger is authoritative\nVerify Command: cargo check --workspace",
            state_dir.to_str().unwrap(),
            &["crates/omega-core".to_string()],
            omega_core::agents::Agent::Codex,
        )
        .unwrap()
        .unwrap();
        let plan = ledger.active_plan(&mission.id).unwrap().unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].verifier_checks.len(), 1);
        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            omega_core::mission::TaskAttemptState::Queued
        );
        assert_eq!(
            ledger.mission(&mission.id).unwrap().unwrap().state,
            omega_core::mission::MissionState::Planned
        );

        transition_v3_worker_attempt(
            &config,
            "OmegaOS-worker-core",
            &attempt,
            omega_core::mission::TaskAttemptState::Running,
        )
        .unwrap();
        assert_eq!(
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            omega_core::mission::TaskAttemptState::Running
        );
        assert_eq!(
            ledger.mission(&mission.id).unwrap().unwrap().state,
            omega_core::mission::MissionState::Running
        );
        std::fs::remove_dir_all(state_dir).unwrap();
    }
}

/// Unit tests for the ORACLE LIFECYCLE decisions of `omega kill`, `omega
/// progress` and `omega status`.
///
/// The whole reason those decisions were extracted into pure functions is
/// right here: exercising them through the commands would need a live rmux
/// daemon, a real oracle and real worker sessions, which is exactly why the
/// cascade, the close-gate and the worktree guard were never covered, and why
/// the scope-claim leak survived so long.
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use omega_core::oracle_lifecycle::LiveWorkers;

    fn workers(running: &[&str], terminal: &[&str]) -> LiveWorkers {
        LiveWorkers {
            running: running.iter().map(|s| s.to_string()).collect(),
            terminal: terminal.iter().map(|s| s.to_string()).collect(),
        }
    }

    // --- omega kill: which sessions cascade, and when a kill is refused -----

    #[test]
    fn kill_cascades_only_finished_workers_by_default() {
        let plan = decide_kill(true, &workers(&[], &["w-a", "w-b"]), false);
        assert_eq!(
            plan,
            KillPlan::Proceed {
                cascade: vec!["w-a".to_string(), "w-b".to_string()]
            }
        );
    }

    #[test]
    fn kill_is_refused_while_a_worker_is_still_running() {
        let plan = decide_kill(true, &workers(&["w-live"], &["w-done"]), false);
        assert_eq!(
            plan,
            KillPlan::Refused {
                running: vec!["w-live".to_string()]
            }
        );
    }

    #[test]
    fn force_cascades_running_workers_too() {
        let plan = decide_kill(true, &workers(&["w-live"], &["w-done"]), true);
        // Terminal first, then running — LiveWorkers::all()'s order.
        assert_eq!(
            plan,
            KillPlan::Proceed {
                cascade: vec!["w-done".to_string(), "w-live".to_string()]
            }
        );
    }

    #[test]
    fn second_kill_on_a_dead_session_is_already_closed_not_an_error() {
        assert_eq!(
            decide_kill(false, &LiveWorkers::default(), false),
            KillPlan::AlreadyClosed
        );
        // …and it stays a no-op with --force: nothing live means nothing to do.
        assert_eq!(
            decide_kill(false, &LiveWorkers::default(), true),
            KillPlan::AlreadyClosed
        );
    }

    #[test]
    fn a_dead_oracle_with_surviving_workers_still_cascades() {
        // The zombie leak itself: the oracle pane is gone but its finished
        // workers are alive holding scope claims. "Already closed" here would
        // leave them there forever.
        assert_eq!(
            decide_kill(false, &workers(&[], &["w-orphan"]), false),
            KillPlan::Proceed {
                cascade: vec!["w-orphan".to_string()]
            }
        );
    }

    #[test]
    fn a_plain_session_has_no_workers_and_never_refuses() {
        assert_eq!(
            decide_kill(true, &LiveWorkers::default(), false),
            KillPlan::Proceed { cascade: vec![] }
        );
    }

    // --- omega kill: whether a worktree is safe to remove -------------------

    #[test]
    fn worktree_with_uncommitted_work_is_kept() {
        assert_eq!(
            worktree_verdict(true, " M crates/omega-cli/src/main.rs\n", 0),
            WorktreeVerdict::Dirty
        );
        // Untracked-only counts as dirty too: those files are unrecoverable.
        assert_eq!(
            worktree_verdict(true, "?? notes.md\n", 0),
            WorktreeVerdict::Dirty
        );
    }

    #[test]
    fn worktree_with_unmerged_commits_is_kept() {
        assert_eq!(
            worktree_verdict(true, "", 3),
            WorktreeVerdict::Unmerged { commits: 3 }
        );
    }

    #[test]
    fn clean_and_merged_worktree_is_removable() {
        assert_eq!(
            worktree_verdict(true, "   \n", 0),
            WorktreeVerdict::Removable
        );
    }

    #[test]
    fn a_tree_omegaos_did_not_create_is_never_touched() {
        // Even perfectly clean and merged: it is not ours to unregister.
        assert_eq!(worktree_verdict(false, "", 0), WorktreeVerdict::NotOurs);
        assert_eq!(worktree_verdict(false, " M x", 9), WorktreeVerdict::NotOurs);
    }

    #[test]
    fn a_reaped_workers_dirty_worktree_is_kept() {
        // The reaper runs `cleanup_worker_worktree`, which is `omega kill`'s
        // remover and therefore governed by the same verdict: a `blocked` or
        // `failed` worker's tree is exactly the one that still holds unsaved
        // work, and it must survive its own reaping.
        //
        // Only the DECISION is unit-testable here. The keep-and-print branch
        // needs a real repository with a linked worktree, so the printing is
        // covered by `cleanup_worker_worktree`'s own guard rather than asserted.
        assert_eq!(
            worktree_verdict(true, " M crates/omega-cli/src/main.rs\n?? notes.md\n", 0),
            WorktreeVerdict::Dirty
        );
        assert_eq!(
            worktree_verdict(true, "", 2),
            WorktreeVerdict::Unmerged { commits: 2 }
        );
    }

    // --- omega reap: which finished workers get closed -----------------------

    fn candidate(session: &str, live: bool, signal: Option<DoneStatus>) -> ReapCandidate {
        ReapCandidate {
            session: session.to_string(),
            live,
            signal,
        }
    }

    #[test]
    fn a_worker_with_no_done_signal_is_never_reaped() {
        // THE safety property. A live worker with no signal is mid-task, and
        // closing it destroys in-flight work.
        assert_eq!(reap_verdict(true, None), ReapVerdict::StillWorking);
        // And a DEAD one with no signal is not reclaimed either: it crashed
        // before signalling, so its worktree is the only copy of what it had
        // done. That case stays the operator's explicit `omega kill`.
        assert_eq!(reap_verdict(false, None), ReapVerdict::StillWorking);
    }

    #[test]
    fn every_terminal_signal_closes_a_live_worker() {
        for status in [
            DoneStatus::DoneClean,
            DoneStatus::Failed,
            DoneStatus::Blocked,
        ] {
            assert_eq!(
                reap_verdict(true, Some(status)),
                ReapVerdict::Reap,
                "{:?} is a stop and must close its session",
                status
            );
        }
    }

    #[test]
    fn pending_is_not_a_stop_and_is_left_alone() {
        // `pending` is what the L4 close-gate writes over an unfinished plan,
        // and a later `omega progress` tick upgrades it back to done_clean.
        // Reaping it would kill the session that has to produce that tick.
        assert!(!is_stop_status(DoneStatus::Pending));
        assert_eq!(
            reap_verdict(true, Some(DoneStatus::Pending)),
            ReapVerdict::NotTerminal
        );
        assert_eq!(
            reap_verdict(false, Some(DoneStatus::Pending)),
            ReapVerdict::NotTerminal
        );
    }

    #[test]
    fn a_live_worker_is_never_closed_by_a_mixed_sweep() {
        // The realistic pane: two finished workers beside one that is still
        // editing files. The sweep must take exactly the two.
        let plan = plan_reap(&[
            candidate("w-done", true, Some(DoneStatus::DoneClean)),
            candidate("w-working", true, None),
            candidate("w-failed", true, Some(DoneStatus::Failed)),
            candidate("w-pending", true, Some(DoneStatus::Pending)),
        ]);
        assert_eq!(
            plan,
            vec![
                ("w-done".to_string(), ReapVerdict::Reap),
                ("w-working".to_string(), ReapVerdict::StillWorking),
                ("w-failed".to_string(), ReapVerdict::Reap),
                ("w-pending".to_string(), ReapVerdict::NotTerminal),
            ]
        );
    }

    #[test]
    fn reaping_twice_is_identical_to_reaping_once() {
        let first = plan_reap(&[
            candidate("w-done", true, Some(DoneStatus::DoneClean)),
            candidate("w-working", true, None),
        ]);
        assert_eq!(
            first,
            vec![
                ("w-done".to_string(), ReapVerdict::Reap),
                ("w-working".to_string(), ReapVerdict::StillWorking),
            ]
        );

        // Second pass over the SAME state dir, after the first closed w-done:
        // the signal file is still there, the session is not. The reaper must
        // report it closed and re-run only the reclaim (a `release` on an absent
        // claim and a `worker_worktrees` that finds nothing are both no-ops), and
        // it must NOT have acquired an opinion about the worker still running.
        let second = plan_reap(&[
            candidate("w-done", false, Some(DoneStatus::DoneClean)),
            candidate("w-working", true, None),
        ]);
        assert_eq!(
            second,
            vec![
                ("w-done".to_string(), ReapVerdict::AlreadyClosed),
                ("w-working".to_string(), ReapVerdict::StillWorking),
            ]
        );

        // And the sweep form converges to nothing: a closed session is no
        // longer in the live list, so it is not even a candidate.
        assert_eq!(plan_reap(&[]), vec![]);
    }

    #[test]
    fn an_already_closed_session_is_never_a_second_cascade() {
        // `AlreadyClosed` and `Reap` differ by exactly one step, the kill. Both
        // reclaim, neither errors — which is why running the reaper on a dead
        // session exits 0 rather than reporting "session not found", the same
        // contract `omega kill` already honours.
        assert_eq!(
            reap_verdict(false, Some(DoneStatus::DoneClean)),
            ReapVerdict::AlreadyClosed
        );
        assert_eq!(
            reap_verdict(false, Some(DoneStatus::Blocked)),
            ReapVerdict::AlreadyClosed
        );
    }

    // --- omega status: name resolution + actionable remedies ----------------

    #[test]
    fn the_mission_key_resolves_to_the_oracle_session() {
        // The incident: `omega status dentistrygpt-3` died on "Session not found"
        // while `omega status oracle-dentistrygpt-3` printed a full report — and
        // the bare key is the spelling the escalation file hands the operator.
        let tmp = std::env::temp_dir().join(format!("omega-alias-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let live = vec!["oracle-dentistrygpt-3".to_string()];
        assert_eq!(
            resolve_oracle_alias("dentistrygpt-3", &live, &tmp),
            "oracle-dentistrygpt-3"
        );
        // Idempotent on the already-correct spelling.
        assert_eq!(
            resolve_oracle_alias("oracle-dentistrygpt-3", &live, &tmp),
            "oracle-dentistrygpt-3"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_live_exact_match_always_beats_the_oracle_prefix() {
        // The property that keeps this safe: a real session named `foo` must
        // never be re-pointed at `oracle-foo` just because both exist.
        let tmp = std::env::temp_dir().join(format!("omega-alias-exact-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let live = vec!["foo".to_string(), "oracle-foo".to_string()];
        assert_eq!(resolve_oracle_alias("foo", &live, &tmp), "foo");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn an_unknown_name_is_left_alone_so_the_error_is_unchanged() {
        let tmp = std::env::temp_dir().join(format!("omega-alias-unk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::create_dir_all(&tmp);
        assert_eq!(resolve_oracle_alias("nope", &[], &tmp), "nope");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_dead_mission_still_resolves_from_its_record_on_disk() {
        // Reading a crashed mission is exactly when the operator needs this, and
        // the daemon lists nothing for it.
        let tmp = std::env::temp_dir().join(format!("omega-alias-dead-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("oracle-proj-9.progress.json"), "{}").unwrap();
        assert_eq!(resolve_oracle_alias("proj-9", &[], &tmp), "oracle-proj-9");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn every_closure_refusal_carries_a_next_command() {
        // The whole complaint in one assertion: a refusal that names no remedy
        // reads as a verdict with no appeal.
        let cases = [
            closure_verdict(0, 0, &[], &[], false, &[]),
            closure_verdict(2, 1, &[], &["b".into()], false, &[]),
            closure_verdict(2, 1, &["a".into()], &[], false, &[]),
            closure_verdict(1, 1, &[], &[], false, &[]),
            closure_verdict(1, 1, &[], &[], true, &["w".into()]),
        ];
        for v in &cases {
            assert!(v.refused, "case should be refused: {v:?}");
            let r = closure_remedies(v, "oracle-p-1");
            assert!(!r.is_empty(), "no remedy offered for {v:?}");
            assert!(
                r.iter().any(|s| s.contains("omega kill oracle-p-1")),
                "the operator's close-it-anyway button must always be offered: {r:?}"
            );
        }
    }

    #[test]
    fn the_gate_refusal_names_the_command_that_satisfies_it() {
        // The unguessable one: `omega gate` only ever READ a result, and the sole
        // writer was `omega orchestrate`.
        let v = closure_verdict(1, 1, &[], &[], false, &[]);
        let r = closure_remedies(&v, "oracle-p-1");
        assert!(
            r.iter().any(|s| s.contains("--accept") && s.contains("--approver")),
            "expected a signed gate acceptance, got {r:?}"
        );
    }

    #[test]
    fn a_closeable_mission_is_offered_no_remedies() {
        let v = closure_verdict(2, 2, &[], &[], true, &[]);
        assert!(!v.refused);
        assert_eq!(closure_remedies(&v, "oracle-p-1"), Vec::<String>::new());
    }

    // --- omega reap: orphan scope claims ------------------------------------

    fn claim(session: &str, files: &[&str], age_secs: i64) -> omega_core::scope::ScopeClaim {
        omega_core::scope::ScopeClaim {
            session: session.to_string(),
            files_owned: files.iter().map(|f| f.to_string()).collect(),
            claimed_at: chrono::Utc::now() - chrono::Duration::seconds(age_secs),
        }
    }

    #[test]
    fn a_claim_whose_owner_session_is_gone_is_reclaimed() {
        // The measured leak: five claims aged 17-25 days on this box, every owner
        // long dead, none of them reachable by `reap_verdict` (the sweep only
        // enumerates LIVE sessions, and a signal-less worker is `StillWorking`).
        let now = chrono::Utc::now();
        let claims = vec![claim("proj-worker-dead", &["convex/rag.ts"], 25 * 86_400)];
        let orphans = plan_orphan_claims(&claims, &[], now);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].session, "proj-worker-dead");
        assert_eq!(orphans[0].files, vec!["convex/rag.ts".to_string()]);
    }

    #[test]
    fn a_claim_whose_owner_is_still_live_is_never_touched() {
        // The property that must not regress: a live worker keeps its claim, or
        // the sweep becomes the R-SCOPE violation it exists to prevent.
        let now = chrono::Utc::now();
        let claims = vec![claim("proj-worker-alive", &["a.ts"], 30 * 86_400)];
        let live = vec!["proj-worker-alive".to_string()];
        assert_eq!(plan_orphan_claims(&claims, &live, now), vec![]);
    }

    #[test]
    fn a_fresh_claim_is_left_alone_even_with_no_live_owner() {
        // `claim_or_reject` writes the claim before the session is necessarily
        // listed, so a claim seconds old with no live owner may be a worker that
        // is about to appear. The grace window is what keeps the sweep off it.
        let now = chrono::Utc::now();
        let claims = vec![claim("proj-worker-starting", &["a.ts"], 5)];
        assert_eq!(plan_orphan_claims(&claims, &[], now), vec![]);

        // …and one second past the window it is reclaimable.
        let claims = vec![claim("proj-worker-starting", &["a.ts"], ORPHAN_CLAIM_GRACE_SECS + 1)];
        assert_eq!(plan_orphan_claims(&claims, &[], now).len(), 1);
    }

    #[test]
    fn an_oracles_own_claim_is_not_swept_while_the_oracle_lives() {
        // Oracles hold claims too, and the live set is checked by NAME, not by
        // role — an oracle mid-mission must not have its claim reclaimed under it.
        let now = chrono::Utc::now();
        let claims = vec![
            claim("oracle-proj-1", &["docs/"], 3 * 86_400),
            claim("proj-worker-dead", &["src/"], 3 * 86_400),
        ];
        let live = vec!["oracle-proj-1".to_string()];
        let orphans = plan_orphan_claims(&claims, &live, now);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].session, "proj-worker-dead");
    }

    // --- omega kill: finding the worktree a worker ran in -------------------

    #[test]
    fn the_branch_slug_matches_omega_git_branch_sh() {
        // The real pairing on disk: session `OmegaOS-worker-cli-lifecycle` ran
        // in `worktrees/OmegaOS/omegaos-worker-cli-lifecycle-40347920`.
        assert_eq!(
            worker_branch_slug("OmegaOS-worker-cli-lifecycle"),
            "omegaos-worker-cli-lifecycle"
        );
        // Spaces become dashes, everything outside [a-z0-9-] is dropped, and
        // the edges are trimmed — `tr`/`sed` in _mk_branch, in that order.
        assert_eq!(worker_branch_slug("Foo Bar_v2.1!"), "foo-barv21");
        assert_eq!(worker_branch_slug("--Edge--"), "edge");
        // A name with nothing usable left falls back, exactly like the script.
        assert_eq!(worker_branch_slug("!!!"), "worker");
    }

    #[test]
    fn a_worktree_dir_is_claimed_only_with_a_real_shortid_suffix() {
        let slug = "omegaos-worker-cli-lifecycle";
        assert!(worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle-40347920",
            slug
        ));
        // The `-1`, `-2`, … collision counter _mk_branch appends.
        assert!(worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle-40347920-2",
            slug
        ));
        // A LONGER worker name that merely starts the same way is NOT ours —
        // this is the case that would otherwise delete another worker's tree.
        assert!(!worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle-extra-1a2b3c4d",
            slug
        ));
        // No shortid at all, wrong length, or non-hex: never claimed.
        assert!(!worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle",
            slug
        ));
        assert!(!worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle-4034792",
            slug
        ));
        assert!(!worktree_dir_belongs_to(
            "omegaos-worker-cli-lifecycle-zzzzzzzz",
            slug
        ));
        // A different worker entirely.
        assert!(!worktree_dir_belongs_to(
            "dentistrygpt-worker-hor05-codeaudit-0cc9215d",
            slug
        ));
    }

    #[test]
    fn a_multi_dash_worker_name_still_resolves_its_tree() {
        // Real tree on disk; the slug itself carries dashes, so the shortid
        // test cannot just split on the last dash.
        let slug = worker_branch_slug("dentistrygpt-worker-bal01-debugaudit-harness-rerun");
        assert!(worktree_dir_belongs_to(
            "dentistrygpt-worker-bal01-debugaudit-harness-rerun-6ebe8107",
            &slug
        ));
    }

    #[test]
    fn the_worktree_walker_finds_a_real_tree_on_disk() {
        // Walks a real directory layout, because the bug this replaced was not
        // a wrong rule but a source that was always empty: the unit rules below
        // all passed while nothing ever called them.
        let omega_dir = std::env::temp_dir().join(format!(
            "omega-worktree-walk-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_micros()
        ));
        let root = omega_dir.join("worktrees");
        // Nested under a repo bucket, the shape spawn-worker actually creates.
        let mine = root.join("OmegaOS").join("omegaos-worker-cli-40347920");
        // A sibling worker: same bucket, different slug, must never be claimed.
        let other = root.join("OmegaOS").join("omegaos-worker-docs-40347920");
        // Flat at the top level, the older shape still present on disk.
        let flat = root.join("omegaos-worker-cli-aabbccdd");
        for d in [&mine, &other, &flat] {
            std::fs::create_dir_all(d).unwrap();
            // A LINKED worktree's `.git` is a FILE.
            std::fs::write(d.join(".git"), "gitdir: /somewhere/.git/worktrees/x").unwrap();
        }
        // A MAIN checkout (`.git` is a directory) is rejected even though its
        // name matches — this is the guard that keeps a real repo safe.
        let main_checkout = root.join("OmegaOS").join("omegaos-worker-cli-99887766");
        std::fs::create_dir_all(main_checkout.join(".git")).unwrap();

        let mut found = worker_worktrees(&omega_dir, "OmegaOS-worker-cli");
        found.sort();
        // Sorted: the repo bucket `OmegaOS/` sorts before the flat `omegaos-…`
        // entry, uppercase first.
        assert_eq!(found, vec![mine.clone(), flat.clone()]);
        assert!(!found.contains(&other));
        assert!(!found.contains(&main_checkout));

        // An unknown worker resolves to nothing rather than to someone else's.
        assert!(worker_worktrees(&omega_dir, "OmegaOS-worker-nope").is_empty());
        // A missing worktrees root is not an error.
        assert!(worker_worktrees(std::path::Path::new("/nonexistent-omega"), "x").is_empty());

        std::fs::remove_dir_all(&omega_dir).unwrap();
    }

    // --- omega progress: the read-back checklist ---------------------------

    #[test]
    fn checklist_glyphs_match_the_telegram_card() {
        assert_eq!(plan_task_glyph("done"), '✓');
        assert_eq!(plan_task_glyph("fail"), '✗');
        assert_eq!(plan_task_glyph("doing"), '▸');
        assert_eq!(plan_task_glyph("todo"), '☐');
        // Anything unknown falls back to todo, exactly like taskList() does.
        assert_eq!(plan_task_glyph("banana"), '☐');
    }

    #[test]
    fn checklist_renders_every_task_with_its_counts() {
        let doc = serde_json::json!({
            "tasks": [
                { "t": "audit code", "s": "done" },
                { "t": "fix N+1",    "s": "doing" },
                { "t": "merge",      "s": "todo" },
                { "t": "deploy",     "s": "fail" },
            ]
        });
        let tasks = parse_plan_tasks(&doc);
        assert_eq!(tasks.len(), 4);
        assert_eq!(
            render_plan_checklist("omegaos-3", &tasks),
            "oracle-omegaos-3: plan 1/4\n✓ audit code\n▸ fix N+1\n☐ merge\n✗ deploy"
        );
    }

    #[test]
    fn checklist_survives_a_malformed_task_entry() {
        // Three producers write this file; one bad entry must not blank the
        // whole read-back an oracle is resuming from.
        let doc = serde_json::json!({
            "tasks": [
                { "t": "good", "s": "done" },
                { "s": "done" },
                { "t": "no status" },
            ]
        });
        let tasks = parse_plan_tasks(&doc);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[1].status, "todo");
        assert_eq!(
            render_plan_checklist("k", &tasks),
            "oracle-k: plan 1/2\n✓ good\n☐ no status"
        );
    }

    #[test]
    fn an_empty_plan_says_so_instead_of_printing_nothing() {
        assert_eq!(
            render_plan_checklist("k", &[]),
            "oracle-k: no plan recorded (0 tasks)."
        );
        assert!(parse_plan_tasks(&serde_json::json!({})).is_empty());
    }

    // --- omega status: the closure verdict ---------------------------------

    #[test]
    fn closure_is_allowed_only_when_every_gate_is_green() {
        assert_eq!(
            closure_verdict(3, 3, &[], &[], true, &[]),
            ClosureVerdict {
                refused: false,
                reasons: vec![]
            }
        );
    }

    #[test]
    fn an_absent_plan_refuses_the_closure_with_cmd_dones_wording() {
        let v = closure_verdict(0, 0, &[], &[], true, &[]);
        assert!(v.refused);
        assert_eq!(
            v.reasons,
            vec!["plan missionnel absent ou vide; acceptation impossible".to_string()]
        );
    }

    #[test]
    fn an_unfinished_plan_names_the_tasks_that_are_not_done() {
        let v = closure_verdict(
            3,
            1,
            &["deploy".to_string()],
            &["merge".to_string()],
            true,
            &[],
        );
        assert!(v.refused);
        assert_eq!(
            v.reasons,
            vec!["échec: deploy".to_string(), "non fait: merge".to_string()]
        );
    }

    #[test]
    fn a_complete_plan_with_no_titles_still_reports_the_ratio() {
        // done < total but the titles were unreadable: never claim "all good".
        let v = closure_verdict(7, 4, &[], &[], true, &[]);
        assert_eq!(v.reasons, vec!["plan 4/7 — pas 100% (L4)".to_string()]);
    }

    #[test]
    fn a_missing_quality_gate_refuses_the_closure() {
        let v = closure_verdict(2, 2, &[], &[], false, &[]);
        assert!(v.refused);
        assert_eq!(
            v.reasons,
            vec!["quality gate indépendante absente ou non acceptée".to_string()]
        );
    }

    #[test]
    fn running_workers_refuse_the_closure_and_are_named() {
        let v = closure_verdict(
            1,
            1,
            &[],
            &[],
            true,
            &["w-a".to_string(), "w-b".to_string()],
        );
        assert!(v.refused);
        assert_eq!(v.reasons, vec!["2 worker(s) still running: w-a, w-b"]);
    }

    #[test]
    fn every_failing_gate_is_reported_at_once_not_one_per_run() {
        // An operator fixing them one at a time, one `omega status` per fix,
        // is the slow path this exists to avoid.
        let v = closure_verdict(2, 1, &[], &["merge".to_string()], false, &["w".to_string()]);
        assert_eq!(v.reasons.len(), 3);
    }

    // --- the ledger wiring: cmd_progress / cmd_done read omega_core --------

    fn ledger(items: &[(&str, &str)]) -> omega_core::oracle_todo::OracleTodo {
        let mut todo = omega_core::oracle_todo::OracleTodo::new("oracle-t");
        for (title, status) in items {
            todo.upsert(title, parse_todo_status(status).unwrap(), None)
                .unwrap();
        }
        todo
    }

    #[test]
    fn the_status_vocabulary_is_exactly_the_documented_four() {
        use omega_core::oracle_todo::TodoStatus;
        for (word, expected) in [
            ("todo", TodoStatus::Todo),
            ("doing", TodoStatus::Doing),
            ("done", TodoStatus::Done),
            ("fail", TodoStatus::Fail),
        ] {
            assert_eq!(parse_todo_status(word).unwrap(), expected);
        }
        // A typo used to be written to the file verbatim, minting a status no
        // consumer knows. It has to be a caller error, not an unreadable ledger.
        for bad in ["Done", "blocked", "in_progress", ""] {
            assert!(parse_todo_status(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn the_l4_refusal_reads_exactly_like_the_one_omega_status_prints() {
        // cmd_done now derives its refusal from the ledger while `omega status`
        // still derives it from closure_verdict. The two surfaces describe one
        // plan, so their wording and ORDER must not drift apart; the bot and the
        // patrol read both.
        let todo = ledger(&[
            ("audit", "done"),
            ("deploy", "doing"),
            ("deploy", "fail"),
            ("merge", "todo"),
        ]);
        let (done, total) = todo.counts();
        let reasons = l4_refusal_reasons(&todo);
        // Pinned literally as well as by equivalence: two empty vectors would
        // satisfy the comparison below without either surface saying anything.
        assert_eq!(reasons, vec!["échec: deploy", "non fait: merge"]);
        assert_eq!(
            reasons,
            closure_verdict(
                total as usize,
                done as usize,
                &["deploy".to_string()],
                &["merge".to_string()],
                true,
                &[],
            )
            .reasons
        );
    }

    #[test]
    fn an_absent_ledger_refuses_with_the_absent_plan_wording() {
        let reasons = l4_refusal_reasons(&omega_core::oracle_todo::OracleTodo::new("oracle-t"));
        assert_eq!(
            reasons,
            vec!["plan missionnel absent ou vide; acceptation impossible"]
        );
        assert_eq!(reasons, closure_verdict(0, 0, &[], &[], true, &[]).reasons);
    }

    #[test]
    fn a_second_doing_leaves_exactly_one_task_in_progress() {
        // The defect this wiring exists to fix: the inline upsert left BOTH
        // tasks `doing`, so "what am I doing" had two answers after a compaction.
        let todo = ledger(&[("a", "todo"), ("b", "todo"), ("a", "doing"), ("b", "doing")]);
        assert_eq!(todo.current().map(|t| t.title.as_str()), Some("b"));
        assert_eq!(todo.unfinished().len(), 2);
    }

    #[test]
    fn a_finished_task_cannot_be_walked_back_to_todo() {
        // The other defect: `done -> todo` exited 0 and silently rewrote the
        // ledger. It is now an error, and the plan is left untouched.
        let mut todo = ledger(&[("a", "done")]);
        assert!(todo
            .upsert("a", omega_core::oracle_todo::TodoStatus::Todo, None)
            .is_err());
        assert_eq!(todo.counts(), (1, 1));
    }

    #[test]
    fn an_unreadable_plan_never_arms_the_gate_upgrade() {
        use omega_core::done::DoneStatus::{Blocked, DoneClean, Failed, Pending};
        // The legitimate case: the oracle's own final "report" task is unfinished
        // at omega-done time by contract, so the next progress tick upgrades it.
        assert!(arms_gate_upgrade(DoneClean, Pending, false));
        // The case that must never arm: patrol's upgrader re-derives L4 from the
        // RAW json and trusts the on-disk counters, so a file that is valid JSON
        // but not a valid plan reads 3/3 to patrol while every CLI surface refuses
        // it. Arming it lets patrol flip an honest refusal to done_clean.
        assert!(!arms_gate_upgrade(DoneClean, Pending, true));
        // Nothing else arms it: no downgrade happened, or the request was already
        // an honest non-clean status.
        assert!(!arms_gate_upgrade(DoneClean, DoneClean, false));
        assert!(!arms_gate_upgrade(DoneClean, Failed, false));
        assert!(!arms_gate_upgrade(Pending, Pending, false));
        assert!(!arms_gate_upgrade(Blocked, Pending, false));
    }

    #[test]
    fn the_l4_gate_downgrades_a_dishonest_clean_close_and_upgrades_nothing() {
        use omega_core::done::DoneStatus;
        use omega_core::oracle_todo::honest_status;
        // Incomplete -> pending, so the `omega progress` tick can still upgrade it.
        assert_eq!(
            honest_status(DoneStatus::DoneClean, &ledger(&[("a", "todo")])),
            DoneStatus::Pending
        );
        // A failure is reported as one, never dressed up as pending.
        assert_eq!(
            honest_status(
                DoneStatus::DoneClean,
                &ledger(&[("a", "doing"), ("a", "fail")])
            ),
            DoneStatus::Failed
        );
        // 100% and failure-free is the only clean close.
        assert_eq!(
            honest_status(DoneStatus::DoneClean, &ledger(&[("a", "done")])),
            DoneStatus::DoneClean
        );
        // An honest non-clean status is never rewritten into something nicer.
        assert_eq!(
            honest_status(DoneStatus::Failed, &ledger(&[("a", "done")])),
            DoneStatus::Failed
        );
    }
}
