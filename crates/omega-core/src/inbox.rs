//! Oracle inbox — JSONL event queue for oracle-worker communication.
//!
//! Replaces tmux send-keys nudges with file-based events. Workers/patrol
//! push events; oracles drain on their own schedule. Pure file IO.

use anyhow::Result;
use chrono::{DateTime, Utc};
use fs2::FileExt;
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

    /// Exclusive advisory lock guarding the push/drain critical sections, held
    /// for the lifetime of the returned handle (drop = unlock). Without it
    /// drain()'s peek-then-remove races a concurrent push(): an event appended
    /// between the read and the unlink is deleted unread. Read-only peek takes
    /// a shared lock; drain calls the unlocked parser
    /// while already holding this exclusive lock. This makes malformed JSON a
    /// durable corruption signal instead of confusing a concurrent append
    /// with corruption. Mirrors scope.rs's `.scope.lock` pattern.
    fn lock(&self) -> Result<std::fs::File> {
        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.with_extension("lock"))?;
        f.lock_exclusive()?;
        Ok(f)
    }

    fn lock_shared(&self) -> Result<std::fs::File> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.with_extension("lock"))?;
        FileExt::lock_shared(&file)?;
        Ok(file)
    }

    pub fn push(&self, event: &InboxEvent) -> Result<()> {
        let _lock = self.lock()?;
        let line = serde_json::to_string(event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    fn peek_unlocked(&self) -> Result<Vec<InboxEvent>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(&self.path)?;
        let reader = std::io::BufReader::new(file);
        let mut events = Vec::new();
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let event = serde_json::from_str::<InboxEvent>(trimmed).map_err(|error| {
                anyhow::anyhow!(
                    "corrupt inbox event at {} line {}: {}",
                    self.path.display(),
                    index + 1,
                    error
                )
            })?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn peek(&self) -> Result<Vec<InboxEvent>> {
        // Synchronize with push/drain so strict parsing never mistakes a
        // concurrent partial append for durable corruption.
        let _lock = self.lock_shared()?;
        self.peek_unlocked()
    }

    pub fn drain(&self) -> Result<Vec<InboxEvent>> {
        // Hold the exclusive lock across peek+remove so a concurrent push can't
        // append an event into the window between the read and the unlink (which
        // remove_file would then delete unread). peek() is called lock-free here
        // on purpose — we already hold the lock.
        let _lock = self.lock()?;
        let events = self.peek_unlocked()?;
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(events)
    }

    pub fn count(&self) -> Result<usize> {
        Ok(self.peek()?.len())
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

    // Regression for the push/drain race: pushers append while the main thread
    // drains, exercising exactly the window remove_file used to delete unread.
    // The exclusive lock makes push and drain mutually exclusive, so every
    // pushed event is either drained or still pending — never lost. Invariant:
    // total drained == total pushed.
    #[test]
    fn concurrent_push_drain_loses_no_events() {
        use std::sync::Arc;
        use std::thread;
        const THREADS: usize = 4;
        const PER: usize = 25;

        let tmp = TempDir::new().unwrap();
        let dir = Arc::new(tmp.path().to_path_buf());

        let mut handles = Vec::new();
        for t in 0..THREADS {
            let dir = Arc::clone(&dir);
            handles.push(thread::spawn(move || {
                let inbox = Inbox::for_oracle(&dir, "race");
                for i in 0..PER {
                    inbox
                        .push(&InboxEvent::worker_done(
                            &format!("w-{t}-{i}"),
                            "done_clean",
                        ))
                        .unwrap();
                }
            }));
        }

        // Drain in parallel with the live pushers (the race window), then drain
        // once more after they all finish to collect the tail.
        let inbox = Inbox::for_oracle(&dir, "race");
        let mut collected = 0;
        for h in handles {
            collected += inbox.drain().unwrap().len();
            h.join().unwrap();
        }
        collected += inbox.drain().unwrap().len();

        assert_eq!(
            collected,
            THREADS * PER,
            "lock must lose no event across push/drain"
        );
    }

    #[test]
    fn corrupt_line_is_an_error_not_a_false_empty_or_ack() {
        let tmp = TempDir::new().unwrap();
        let inbox = Inbox::for_oracle(tmp.path(), "strict");
        inbox
            .push(&InboxEvent::worker_done("worker-1", "done_clean"))
            .unwrap();
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(tmp.path().join("oracle-strict.inbox.jsonl"))
            .unwrap();
        writeln!(file, "{{not-json").unwrap();

        assert!(inbox.peek().is_err());
        assert!(inbox.count().is_err());
        assert!(inbox.drain().is_err());
        assert!(tmp.path().join("oracle-strict.inbox.jsonl").exists());
    }
}
