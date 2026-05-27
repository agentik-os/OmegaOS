//! Oracle inbox — JSONL event queue for oracle-worker communication.
//!
//! Replaces tmux send-keys nudges with file-based events. Workers/patrol
//! push events; oracles drain on their own schedule. Pure file IO.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxEvent {
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    WorkerDone,
    WorkerStalled,
    WorkerBlocked,
    ShipFailed,
    ShipSuccess,
    AuditComplete,
    GateResult,
}

impl InboxEvent {
    pub fn worker_done(session: &str, status: &str) -> Self {
        Self {
            event_type: EventType::WorkerDone,
            payload: serde_json::json!({
                "session": session,
                "status": status,
            }),
            timestamp: Utc::now(),
        }
    }

    pub fn worker_stalled(session: &str, idle_secs: u64) -> Self {
        Self {
            event_type: EventType::WorkerStalled,
            payload: serde_json::json!({
                "session": session,
                "idle_secs": idle_secs,
            }),
            timestamp: Utc::now(),
        }
    }

    pub fn worker_blocked(session: &str, reason: &str) -> Self {
        Self {
            event_type: EventType::WorkerBlocked,
            payload: serde_json::json!({
                "session": session,
                "reason": reason,
            }),
            timestamp: Utc::now(),
        }
    }

    pub fn ship_failed(project: &str, step: &str, error: &str) -> Self {
        Self {
            event_type: EventType::ShipFailed,
            payload: serde_json::json!({
                "project": project,
                "step": step,
                "error": error,
            }),
            timestamp: Utc::now(),
        }
    }

    pub fn ship_success(project: &str, commit: &str, deploy_url: Option<&str>) -> Self {
        Self {
            event_type: EventType::ShipSuccess,
            payload: serde_json::json!({
                "project": project,
                "commit": commit,
                "deploy_url": deploy_url,
            }),
            timestamp: Utc::now(),
        }
    }
}

pub struct Inbox {
    path: PathBuf,
}

impl Inbox {
    fn inbox_path(state_dir: &Path, oracle: &str) -> PathBuf {
        state_dir.join(format!("oracle-{}.inbox.jsonl", oracle))
    }

    pub fn for_oracle(state_dir: &Path, oracle: &str) -> Self {
        Self {
            path: Self::inbox_path(state_dir, oracle),
        }
    }

    pub fn push(&self, event: &InboxEvent) -> Result<()> {
        let line = serde_json::to_string(event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn peek(&self) -> Result<Vec<InboxEvent>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(&self.path)?;
        let reader = std::io::BufReader::new(file);
        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(event) = serde_json::from_str::<InboxEvent>(trimmed) {
                events.push(event);
            }
        }
        Ok(events)
    }

    pub fn drain(&self) -> Result<Vec<InboxEvent>> {
        let events = self.peek()?;
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(events)
    }

    pub fn count(&self) -> Result<usize> {
        if !self.path.exists() {
            return Ok(0);
        }
        let file = std::fs::File::open(&self.path)?;
        let reader = std::io::BufReader::new(file);
        Ok(reader.lines().filter_map(|l| l.ok()).filter(|l| !l.trim().is_empty()).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn push_peek_drain_cycle() {
        let tmp = TempDir::new().unwrap();
        let inbox = Inbox::for_oracle(tmp.path(), "test-proj");

        assert_eq!(inbox.count().unwrap(), 0);
        assert!(inbox.peek().unwrap().is_empty());

        inbox
            .push(&InboxEvent::worker_done("worker-1", "done_clean"))
            .unwrap();
        inbox
            .push(&InboxEvent::worker_stalled("worker-2", 600))
            .unwrap();

        assert_eq!(inbox.count().unwrap(), 2);
        assert_eq!(inbox.peek().unwrap().len(), 2);

        let events = inbox.drain().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::WorkerDone);
        assert_eq!(events[1].event_type, EventType::WorkerStalled);

        assert_eq!(inbox.count().unwrap(), 0);
    }
}
