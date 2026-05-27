use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus, WorkerBlocked};
use crate::inbox::{Inbox, InboxEvent};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;

const STALL_THRESHOLD_SECS: i64 = 900; // 15 minutes without progress = stalled

#[derive(Debug)]
pub struct PatrolReport {
    pub total_sessions: usize,
    pub oracles: usize,
    pub workers: usize,
    pub done_workers: Vec<String>,
    pub stalled_workers: Vec<String>,
    pub blocked_workers: Vec<String>,
    pub orphaned_sessions: Vec<String>,
    pub actions_taken: Vec<String>,
}

pub struct Patrol {
    config: OmegaConfig,
}

impl Patrol {
    pub fn new(config: OmegaConfig) -> Self {
        Self { config }
    }

    pub async fn run_once(&self) -> Result<PatrolReport> {
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
            actions_taken: Vec::new(),
        };

        // Collect oracle names for inbox event delivery
        let oracle_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();

        // Check for done workers — push events to parent oracle inbox
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(done) = DoneSignal::read(&self.config.state_dir, &session.name)? {
                    report.done_workers.push(session.name.clone());

                    // Push worker_done event to the parent oracle's inbox
                    if let Some(oracle) = self.find_parent_oracle(&session.name, &oracle_sessions) {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        let status_str = match done.status {
                            DoneStatus::DoneClean => "done_clean",
                            DoneStatus::Pending => "pending",
                            DoneStatus::Failed => "failed",
                            DoneStatus::Blocked => "blocked",
                        };
                        let _ = inbox.push(&InboxEvent::worker_done(&session.name, status_str));
                    }

                    if done.status == DoneStatus::DoneClean {
                        let _ = ScopeClaim::release(&self.config.state_dir, &session.name);
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

        // Stall detection: workers with progress but no advancement
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
                                    "Stall detected: {} (idle {}s)",
                                    session.name, idle_secs
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Check for orphaned sessions (exist in rmux but no matching done.json or progress)
        for session in &sessions {
            if session.role == SessionRole::Worker {
                let has_done =
                    DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                let has_progress = crate::progress::ProgressInfo::read(
                    &self.config.state_dir,
                    &session.name,
                )
                .is_some();

                if !has_done && !has_progress {
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

        self.log_patrol(&report)?;

        Ok(report)
    }

    fn find_parent_oracle<'a>(
        &self,
        worker_name: &str,
        oracles: &'a [&crate::session::OmegaSession],
    ) -> Option<&'a crate::session::OmegaSession> {
        // Match worker to oracle by project name
        let worker_session = crate::session::OmegaSession::classify(worker_name);
        let worker_project = worker_session.project.as_deref()?;

        oracles
            .iter()
            .find(|o| o.project.as_deref() == Some(worker_project))
            .copied()
    }

    pub async fn run_loop(&self, interval: Duration) -> Result<()> {
        tracing::info!(interval_secs = interval.as_secs(), "Patrol daemon started");
        loop {
            match self.run_once().await {
                Ok(report) => {
                    tracing::info!(
                        sessions = report.total_sessions,
                        done = report.done_workers.len(),
                        orphaned = report.orphaned_sessions.len(),
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
            "[{}] sessions={} oracles={} workers={} done={} stalled={} blocked={} orphaned={} actions={}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S"),
            report.total_sessions,
            report.oracles,
            report.workers,
            report.done_workers.len(),
            report.stalled_workers.len(),
            report.blocked_workers.len(),
            report.orphaned_sessions.len(),
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
