use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus, OracleDoneSignal, WorkerBlocked};
use crate::inbox::{Inbox, InboxEvent};
use crate::oracle_lifecycle::{
    OracleRegistry, OracleRegistryStatus, OracleState, SignalWatcher,
    WorkerEntryStatus, WorkerStallDetector, StallAction, StallThresholds,
};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;

const STALL_THRESHOLD_SECS: i64 = 900; // 15 minutes without progress = stalled (file-based)

#[derive(Debug)]
pub struct PatrolReport {
    pub total_sessions: usize,
    pub oracles: usize,
    pub workers: usize,
    pub done_workers: Vec<String>,
    pub stalled_workers: Vec<String>,
    pub blocked_workers: Vec<String>,
    pub orphaned_sessions: Vec<String>,
    pub done_oracles: Vec<String>,
    pub actions_taken: Vec<String>,
}

pub struct Patrol {
    config: OmegaConfig,
    stall_detector: WorkerStallDetector,
    signal_watcher: SignalWatcher,
}

impl Patrol {
    pub fn new(config: OmegaConfig) -> Self {
        let signal_watcher = SignalWatcher::new(config.state_dir.clone());
        Self {
            stall_detector: WorkerStallDetector::new(StallThresholds::default()),
            signal_watcher,
            config,
        }
    }

    pub async fn run_once(&mut self) -> Result<PatrolReport> {
        // Heartbeat — proves the patrol actually fired. Lets the user (and
        // `omega doctor`) verify the self-improvement loop is alive rather
        // than silently dead (the failure mode of the old Smith agent).
        let hb = self.config.state_dir.join("patrol-heartbeat.txt");
        let _ = std::fs::create_dir_all(&self.config.state_dir);
        let _ = std::fs::write(&hb, Utc::now().to_rfc3339());

        let mgr = SessionManager::connect().await?;
        let sessions = mgr.list_sessions().await?;

        let mut report = PatrolReport {
            total_sessions: sessions.len(),
            oracles: sessions.iter().filter(|s| s.role == SessionRole::Oracle).count(),
            workers: sessions.iter().filter(|s| s.role == SessionRole::Worker).count(),
            done_workers: Vec::new(),
            stalled_workers: Vec::new(),
            blocked_workers: Vec::new(),
            orphaned_sessions: Vec::new(),
            done_oracles: Vec::new(),
            actions_taken: Vec::new(),
        };

        let oracle_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();

        // ── Worker patrol: done signals ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(done) = DoneSignal::read(&self.config.state_dir, &session.name)? {
                    report.done_workers.push(session.name.clone());

                    if let Some(oracle) = self.find_parent_oracle(&session.name, &oracle_sessions) {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        let status_str = match done.status {
                            DoneStatus::DoneClean => "done_clean",
                            DoneStatus::Pending => "pending",
                            DoneStatus::Failed => "failed",
                            DoneStatus::Blocked => "blocked",
                        };
                        let _ = inbox.push(&InboxEvent::worker_done(&session.name, status_str));

                        // Update oracle state with worker completion
                        if let Ok(Some(mut oracle_state)) =
                            OracleState::read(&self.config.state_dir, &oracle.name)
                        {
                            let ws = match done.status {
                                DoneStatus::DoneClean => WorkerEntryStatus::DoneClean,
                                DoneStatus::Pending => WorkerEntryStatus::Pending,
                                DoneStatus::Failed => WorkerEntryStatus::Failed,
                                DoneStatus::Blocked => WorkerEntryStatus::Blocked,
                            };
                            oracle_state.update_worker_status(&session.name, ws);
                            let _ = oracle_state.write(&self.config.state_dir);
                        }
                    }

                    if done.status == DoneStatus::DoneClean {
                        let _ = ScopeClaim::release(&self.config.state_dir, &session.name);
                        self.stall_detector.forget(&session.name);
                        report
                            .actions_taken
                            .push(format!("Released scope for {}", session.name));
                    }
                }

                // Check for blocked workers
                if let Ok(Some(blocked)) =
                    WorkerBlocked::read(&self.config.state_dir, &session.name)
                {
                    report.blocked_workers.push(session.name.clone());
                    if let Some(oracle) = self.find_parent_oracle(&session.name, &oracle_sessions) {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        let _ = inbox.push(&InboxEvent::worker_blocked(
                            &session.name,
                            &blocked.question,
                        ));
                    }
                }
            }
        }

        // ── Worker patrol: pane-based stall detection (30s nudge / 5min escalate) ──
        for session in &sessions {
            if session.role != SessionRole::Worker {
                continue;
            }
            let has_done = DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
            if has_done {
                continue;
            }

            match mgr.capture_pane(&session.name).await {
                Ok(content) => {
                    let action = self.stall_detector.check(&session.name, &content);
                    match action {
                        StallAction::Nudge { ref session, idle_secs } => {
                            tracing::info!(worker = %session, idle_secs, "Worker idle — nudge");
                            // Send a nudge via the session pane
                            let _ = mgr
                                .send_text(
                                    session,
                                    "You appear idle. Continue your mission or report done.",
                                )
                                .await;
                            report.actions_taken.push(format!(
                                "Nudged {} (idle {}s)",
                                session, idle_secs
                            ));
                        }
                        StallAction::Escalate { ref session, idle_secs } => {
                            report.stalled_workers.push(session.clone());
                            if let Some(oracle) =
                                self.find_parent_oracle(session, &oracle_sessions)
                            {
                                let inbox =
                                    Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                let _ = inbox.push(&InboxEvent::worker_stalled(
                                    session,
                                    idle_secs,
                                ));
                            }
                            report.actions_taken.push(format!(
                                "Escalated stall: {} (idle {}s)",
                                session, idle_secs
                            ));
                        }
                        StallAction::Active => {}
                    }
                }
                Err(_) => {
                    // Can't capture pane — session might be dead
                    report.orphaned_sessions.push(session.name.clone());
                }
            }
        }

        // ── Worker patrol: file-based stall detection (progress files) ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(progress) =
                    crate::progress::ProgressInfo::read(&self.config.state_dir, &session.name)
                {
                    let has_done =
                        DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                    if !has_done
                        && progress.todos_completed < progress.todos_total
                        && !progress.blocked
                    {
                        if let Some(last_update) = progress.last_updated {
                            let idle_secs = (Utc::now() - last_update).num_seconds();
                            if idle_secs > STALL_THRESHOLD_SECS {
                                if !report.stalled_workers.contains(&session.name) {
                                    report.stalled_workers.push(session.name.clone());
                                    if let Some(oracle) =
                                        self.find_parent_oracle(&session.name, &oracle_sessions)
                                    {
                                        let inbox =
                                            Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                        let _ = inbox.push(&InboxEvent::worker_stalled(
                                            &session.name,
                                            idle_secs as u64,
                                        ));
                                    }
                                    report.actions_taken.push(format!(
                                        "Stall detected (progress): {} (idle {}s)",
                                        session.name, idle_secs
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Orphan detection: sessions with no done/progress and empty pane ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                let has_done =
                    DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                let has_progress = crate::progress::ProgressInfo::read(
                    &self.config.state_dir,
                    &session.name,
                )
                .is_some();

                if !has_done && !has_progress && !report.orphaned_sessions.contains(&session.name) {
                    match mgr.capture_pane(&session.name).await {
                        Ok(content) => {
                            let trimmed = content.trim();
                            if trimmed.is_empty() || trimmed.lines().count() <= 1 {
                                report.orphaned_sessions.push(session.name.clone());
                            }
                        }
                        Err(_) => {
                            report.orphaned_sessions.push(session.name.clone());
                        }
                    }
                }
            }
        }

        // ── Oracle patrol: check done signals + registry cleanup ──
        self.patrol_oracles(&mgr, &sessions, &mut report).await?;

        // ── Signal file watcher: detect new oracle result files ──
        if let Ok(new_signals) = self.signal_watcher.poll() {
            for (oracle_name, signal) in &new_signals {
                report.actions_taken.push(format!(
                    "Signal file detected: {} (status: {:?})",
                    oracle_name, signal.status
                ));
            }
        }

        self.log_patrol(&report)?;

        Ok(report)
    }

    /// Patrol oracle sessions: check for done oracles, update registry, handle close.
    async fn patrol_oracles(
        &self,
        _mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_names: Vec<String> = sessions.iter().map(|s| s.name.clone()).collect();
        let mut registry = OracleRegistry::load(&self.config.state_dir);

        // Cleanup dead entries from registry
        registry.cleanup(&live_names);

        for session in sessions {
            if session.role != SessionRole::Oracle {
                continue;
            }

            // Check oracle done signal
            if let Ok(Some(done)) =
                OracleDoneSignal::read(&self.config.state_dir, &session.name)
            {
                if done.is_closeable() {
                    report.done_oracles.push(session.name.clone());
                    registry.mark_status(&session.name, OracleRegistryStatus::Done);
                    // Self-improvement: auto-dispatch the curator worker
                    // ONCE per done oracle. The marker file prevents
                    // re-triggering after the curator already ran.
                    let _ = self.maybe_trigger_curator(&session.name);
                }
            }

            // Check oracle state for all-workers-terminal
            if let Ok(Some(oracle_state)) =
                OracleState::read(&self.config.state_dir, &session.name)
            {
                if oracle_state.all_workers_terminal()
                    && !report.done_oracles.contains(&session.name)
                {
                    // All workers are done but oracle hasn't written done signal yet — mark idle
                    registry.mark_status(&session.name, OracleRegistryStatus::Idle);
                }
            }
        }

        let _ = registry.save(&self.config.state_dir);
        Ok(())
    }

    /// Self-improvement hook: when an oracle's done.json flips to a
    /// closeable status, spawn a curator worker that reads the trajectory
    /// + done.json and proposes NEW_SKILL / EDIT_SKILL / NEW_RULE /
    /// NEW_MEMORY items. Output lands in
    /// `~/.omega/state/curator/<oracle>-<timestamp>.md`.
    ///
    /// Idempotent: marker file `~/.omega/state/curator-triggered/<oracle>.flag`
    /// prevents re-trigger on subsequent patrol ticks.
    fn maybe_trigger_curator(&self, oracle_name: &str) -> Result<()> {
        let flag_dir = self.config.state_dir.join("curator-triggered");
        let flag = flag_dir.join(format!("{}.flag", oracle_name));
        if flag.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(&flag_dir)?;
        std::fs::create_dir_all(self.config.state_dir.join("curator"))?;
        std::fs::write(&flag, Utc::now().to_rfc3339())?;

        // Spawn the curator as a detached rmux session so its output is
        // visible in `omega menu`. The session name is prefixed with
        // "curator-" so it's grouped distinctly in the TUI session list.
        let curator_session = format!("curator-{}", oracle_name);
        let done_path = self
            .config
            .state_dir
            .join(format!("oracle-{}.done.json", oracle_name));
        let prompt = format!(
            "/omega-curate {}",
            done_path.to_string_lossy()
        );
        // Use claude --print --dangerously-skip-permissions for a
        // non-interactive one-shot. The session's output goes to its
        // pane (capturable) AND the curator skill writes its report
        // markdown to ~/.omega/state/curator/.
        // Manual shell-escape: wrap in single quotes and escape any
        // internal single quotes. Keeps us dependency-free.
        let escaped = prompt.replace('\'', r"'\''");
        let cmd = format!(
            "claude --print --dangerously-skip-permissions '{}' ; exec bash",
            escaped
        );
        let mgr_dispatch = std::process::Command::new("rmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &curator_session,
                "bash",
                "-c",
                &cmd,
            ])
            .status();
        match mgr_dispatch {
            Ok(s) if s.success() => {
                tracing::info!(
                    oracle = %oracle_name,
                    curator = %curator_session,
                    "curator dispatched"
                );
            }
            _ => {
                tracing::warn!(oracle = %oracle_name, "curator dispatch failed");
            }
        }
        Ok(())
    }

    fn find_parent_oracle<'a>(
        &self,
        worker_name: &str,
        oracles: &'a [&crate::session::OmegaSession],
    ) -> Option<&'a crate::session::OmegaSession> {
        let worker_session = crate::session::OmegaSession::classify(worker_name);
        let worker_project = worker_session.project.as_deref()?;

        oracles
            .iter()
            .find(|o| o.project.as_deref() == Some(worker_project))
            .copied()
    }

    pub async fn run_loop(&mut self, interval: Duration) -> Result<()> {
        tracing::info!(interval_secs = interval.as_secs(), "Patrol daemon started");
        loop {
            match self.run_once().await {
                Ok(report) => {
                    tracing::info!(
                        sessions = report.total_sessions,
                        done_workers = report.done_workers.len(),
                        stalled = report.stalled_workers.len(),
                        done_oracles = report.done_oracles.len(),
                        orphaned = report.orphaned_sessions.len(),
                        actions = report.actions_taken.len(),
                        "Patrol tick"
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Patrol tick failed");
                }
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn log_patrol(&self, report: &PatrolReport) -> Result<()> {
        let log_line = format!(
            "[{}] sessions={} oracles={} workers={} done_w={} stalled={} blocked={} orphaned={} done_o={} actions={}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S"),
            report.total_sessions,
            report.oracles,
            report.workers,
            report.done_workers.len(),
            report.stalled_workers.len(),
            report.blocked_workers.len(),
            report.orphaned_sessions.len(),
            report.done_oracles.len(),
            report.actions_taken.len(),
        );

        let log_path = self.config.logs_dir.join("patrol.log");
        std::fs::create_dir_all(&self.config.logs_dir)?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(log_line.as_bytes())?;
        Ok(())
    }
}
