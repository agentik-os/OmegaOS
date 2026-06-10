//! Mission types — the unit of work that flows through OmegaOS.
//!
//! A Mission is what the human (or Master AISB) hands to the orchestrator.
//! It is then classified, decomposed into a Plan, and executed by Workers.

use crate::routing::Complexity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique mission identifier — short hex string from timestamp + random.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MissionId(pub String);

impl MissionId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mixed = nanos ^ counter.wrapping_mul(0x9E3779B97F4A7C15);

        Self(format!(
            "m-{:08x}{:04x}",
            (mixed >> 16) as u32,
            (mixed & 0xffff) as u16
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MissionId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unit of work submitted by the human or AISB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: MissionId,
    pub project: String,
    pub text: String,
    pub working_dir: PathBuf,
    pub created_at: DateTime<Utc>,
    pub created_by: MissionSource,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionSource {
    Human,
    AisbMaster,
    Oracle,
    Scheduled,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Mission {
    pub fn new(project: impl Into<String>, text: impl Into<String>, working_dir: PathBuf) -> Self {
        Self {
            id: MissionId::new(),
            project: project.into(),
            text: text.into(),
            working_dir,
            created_at: Utc::now(),
            created_by: MissionSource::Human,
            priority: Priority::Normal,
        }
    }

    pub fn from_aisb_master(project: impl Into<String>, text: impl Into<String>, working_dir: PathBuf) -> Self {
        let mut m = Self::new(project, text, working_dir);
        m.created_by = MissionSource::AisbMaster;
        m
    }

    pub fn with_priority(mut self, p: Priority) -> Self {
        self.priority = p;
        self
    }
}

/// A decomposition of a Mission into executable tasks.
/// Built by Oracle/KEYMAKER for COMPLEX or EPIC missions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub mission_id: MissionId,
    pub complexity: Complexity,
    pub strategy: PlanStrategy,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStrategy {
    /// Single worker handles the whole mission
    Direct,
    /// Oracle decomposes and dispatches sequentially
    Sequential,
    /// Multiple workers run in parallel with shared file scopes
    Parallel,
    /// Team in split panes coordinated by a lead
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub files_owned: Vec<String>,
    pub depends_on: Vec<String>,
    pub agent: String,
    #[serde(default)]
    pub estimated_minutes: u32,
}

impl Task {
    pub fn new(id: impl Into<String>, name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            prompt: prompt.into(),
            files_owned: Vec::new(),
            depends_on: Vec::new(),
            agent: "claude".to_string(),
            estimated_minutes: 15,
        }
    }
}

/// Outcome of a Mission after execution + verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub mission_id: MissionId,
    pub status: OutcomeStatus,
    pub workers: Vec<WorkerResult>,
    pub gate: Option<crate::gate::GateResult>,
    pub audit_recommendations: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub summary: String,
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        matches!(self.status, OutcomeStatus::Success)
    }

    pub fn duration_secs(&self) -> u64 {
        (self.finished_at - self.started_at).num_seconds().max(0) as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    Success,
    PartialSuccess,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub task_id: String,
    pub session_name: String,
    pub status: crate::done::DoneStatus,
    pub summary: String,
    pub commit: Option<String>,
    pub duration_secs: u64,
}

// NOTE: a `MissionTracker` (missions.json — "persists active missions for
// AISB visibility") used to live here. It had ZERO callers — no dispatch
// path ever tracked, nothing ever completed, missions.json never existed
// on any install — and there is no mission QUEUE anywhere (dispatch_oracle
// always reserves + spawns immediately). Deleted as dead code rather than
// speculatively wired; if a real queue ever lands, gate dispatch on
// `OracleRegistry::count_active(project)` and persist the overflow there.
