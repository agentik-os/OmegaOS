use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;

#[derive(Debug)]
pub struct PatrolReport {
    pub total_sessions: usize,
    pub oracles: usize,
    pub workers: usize,
    pub done_workers: Vec<String>,
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
            orphaned_sessions: Vec::new(),
            actions_taken: Vec::new(),
        };

        // Check for done workers
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(done) = DoneSignal::read(&self.config.state_dir, &session.name)? {
                    report.done_workers.push(session.name.clone());

                    if done.status == DoneStatus::DoneClean {
                        // Release scope claim
                        let _ = ScopeClaim::release(&self.config.state_dir, &session.name);
                        report
                            .actions_taken
                            .push(format!("Released scope for {}", session.name));
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
                    // Could be orphaned — check if pane has content
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

        // Log patrol run
        self.log_patrol(&report)?;

        Ok(report)
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
            "[{}] sessions={} oracles={} workers={} done={} orphaned={} actions={}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S"),
            report.total_sessions,
            report.oracles,
            report.workers,
            report.done_workers.len(),
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
