use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneSignal {
    pub session: String,
    pub status: DoneStatus,
    pub summary: String,
    #[serde(default)]
    pub commit: Option<String>,
    pub finished_at: DateTime<Utc>,
    #[serde(default)]
    pub todos_total: u32,
    #[serde(default)]
    pub todos_completed: u32,
    #[serde(default)]
    pub pending_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoneStatus {
    DoneClean,
    Pending,
    Failed,
    Blocked,
}

impl DoneSignal {
    pub fn new(session: &str, status: DoneStatus, summary: &str) -> Self {
        Self {
            session: session.to_string(),
            status,
            summary: summary.to_string(),
            commit: None,
            finished_at: Utc::now(),
            todos_total: 0,
            todos_completed: 0,
            pending_actions: Vec::new(),
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("worker-{}.done.json", self.session));
        let tmp_path = state_dir.join(format!(".worker-{}.done.json.tmp", self.session));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp_path, &content)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("worker-{}.done.json", session));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    pub fn is_complete(&self) -> bool {
        self.status == DoneStatus::DoneClean && self.todos_completed >= self.todos_total
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            DoneStatus::DoneClean | DoneStatus::Failed
        )
    }

    /// Read all done signals in state directory.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("worker-") && name.ends_with(".done.json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(signal) = serde_json::from_str::<Self>(&content) {
                                results.push(signal);
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Clean up done signal file after processing.
    pub fn remove(state_dir: &Path, session: &str) -> Result<()> {
        let path = state_dir.join(format!("worker-{}.done.json", session));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// A structured record for when a worker is blocked but still executing a fallback.
/// Mirrors the live system's worker-blocked-*.json protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerBlocked {
    pub session: String,
    pub blocked_at: DateTime<Utc>,
    pub question: String,
    pub best_guess: String,
    pub fallback_action: String,
    pub can_resume_without_answer: bool,
}

impl WorkerBlocked {
    pub fn new(
        session: &str,
        question: &str,
        best_guess: &str,
        fallback_action: &str,
    ) -> Self {
        Self {
            session: session.to_string(),
            blocked_at: Utc::now(),
            question: question.to_string(),
            best_guess: best_guess.to_string(),
            fallback_action: fallback_action.to_string(),
            can_resume_without_answer: true,
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("worker-blocked-{}.json", self.session));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("worker-blocked-{}.json", session));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    pub fn clear(state_dir: &Path, session: &str) -> Result<()> {
        let path = state_dir.join(format!("worker-blocked-{}.json", session));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Read all blocked signals in state directory.
    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("worker-blocked-") && name.ends_with(".json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(blocked) = serde_json::from_str::<Self>(&content) {
                                results.push(blocked);
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

/// Oracle-level done signal — written when an oracle completes its mission.
/// Mirrors the VPS oracle-*.done.json schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleDoneSignal {
    pub oracle: String,
    pub project: String,
    pub status: DoneStatus,
    pub mission: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_secs: u64,
    pub summary: String,
    pub ship: Option<OracleShipResult>,
    #[serde(default)]
    pub pending_actions: Vec<String>,
    #[serde(default)]
    pub lifecycle: OracleLifecycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleShipResult {
    pub requested: bool,
    pub result: String,
    pub commit: Option<String>,
    pub push_url: Option<String>,
    pub deploy_url: Option<String>,
    pub deploy_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleLifecycle {
    Persistent,
    Ephemeral,
}

impl Default for OracleLifecycle {
    fn default() -> Self {
        Self::Ephemeral
    }
}

impl OracleDoneSignal {
    pub fn new(oracle: &str, project: &str, status: DoneStatus, mission: &str) -> Self {
        let now = Utc::now();
        Self {
            oracle: oracle.to_string(),
            project: project.to_string(),
            status,
            mission: mission.to_string(),
            started_at: now,
            finished_at: now,
            duration_secs: 0,
            summary: String::new(),
            ship: None,
            pending_actions: Vec::new(),
            lifecycle: OracleLifecycle::Ephemeral,
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("oracle-{}.done.json", self.oracle));
        let tmp = state_dir.join(format!(".oracle-{}.done.json.tmp", self.oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("oracle-{}.done.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    pub fn is_closeable(&self) -> bool {
        self.status == DoneStatus::DoneClean && self.pending_actions.is_empty()
    }
}
