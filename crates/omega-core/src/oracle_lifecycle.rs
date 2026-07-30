//! Oracle Lifecycle — 5-step workflow state machine.
//!
//! Implements the VPS Oracle pattern: Analyze → Dispatch → Monitor → Verify → Report.
//! Oracles are on-demand project managers: they decompose, delegate, monitor, and verify.
//! They NEVER edit code directly — all code changes go through workers.

use crate::done::DoneStatus;
use crate::mission::{Mission, MissionId, WorkerResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Oracle State Machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OraclePhase {
    /// Initial: reading CLAUDE.md, decomposing, defining success criteria
    Analyze,
    /// Dispatching workers via structured context template
    Dispatch,
    /// Monitoring worker panes, stall detection, inbox events
    Monitor,
    /// All workers done — running quality gate + verification
    Verify,
    /// Writing signal file, reporting results
    Report,
    /// Terminal: oracle has completed its mission
    Done,
}

impl OraclePhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Analyze => "ANALYSE",
            Self::Dispatch => "DISPATCH",
            Self::Monitor => "MONITORING",
            Self::Verify => "VERIFY",
            Self::Report => "REPORT",
            Self::Done => "DONE",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}

// ---------------------------------------------------------------------------
// Oracle State — persisted to state dir
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleState {
    pub oracle_name: String,
    pub project: String,
    pub mission_id: MissionId,
    pub mission_text: String,
    pub working_dir: PathBuf,
    pub phase: OraclePhase,
    pub workers: Vec<WorkerEntry>,
    pub god_mode: Option<GodModeState>,
    pub ship_requested: bool,
    pub started_at: DateTime<Utc>,
    pub phase_entered_at: DateTime<Utc>,
    #[serde(default)]
    pub phase_history: Vec<PhaseTransition>,
    /// N10: timestamp at which this oracle reached a real *closeable* state —
    /// i.e. it wrote a genuine done-signal / finished its mission, NOT merely
    /// that `all_workers_terminal()` became true. An oracle is reusable
    /// (`dispatch::find_available`) ONLY when this is set. `None` means the
    /// oracle is still live work, even if every worker is in a terminal status.
    #[serde(default)]
    pub closeable_since: Option<DateTime<Utc>>,
    /// Claude `--session-id` UUID for this oracle, persisted for the record.
    /// NOTE: every dispatch/resurrect mints a FRESH id (resolve_session_id —
    /// reusing one collides: `--session-id` CREATES, it does not resume), so
    /// this field is bookkeeping, not lineage. `None` for states written
    /// before this field existed.
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerEntry {
    pub session_name: String,
    pub task_id: String,
    pub task_name: String,
    /// V3 task-attempt identity. Absent on pre-ledger workers.
    #[serde(default)]
    pub attempt_id: Option<String>,
    /// Immutable plan revision that authorized this attempt.
    #[serde(default)]
    pub plan_revision: Option<u64>,
    pub files_owned: Vec<String>,
    pub dispatched_at: DateTime<Utc>,
    pub status: WorkerEntryStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerEntryStatus {
    Running,
    DoneClean,
    Pending,
    Failed,
    Blocked,
    Stalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from: OraclePhase,
    pub to: OraclePhase,
    pub at: DateTime<Utc>,
    pub reason: String,
}

impl OracleState {
    pub fn new(oracle_name: &str, mission: &Mission) -> Self {
        let now = Utc::now();
        Self {
            oracle_name: oracle_name.to_string(),
            project: mission.project.clone(),
            mission_id: mission.id.clone(),
            mission_text: mission.text.clone(),
            working_dir: mission.working_dir.clone(),
            phase: OraclePhase::Analyze,
            workers: Vec::new(),
            god_mode: None,
            ship_requested: false,
            started_at: now,
            phase_entered_at: now,
            phase_history: Vec::new(),
            closeable_since: None,
            session_id: None,
        }
    }

    /// Minimal state for a live oracle that has no persisted mission yet.
    /// Lets `spawn-worker` ALWAYS record the worker→oracle link (the session
    /// menu nests workers under their governing oracle from it) even when the
    /// oracle never wrote a full `OracleState`. Placeholders are overwritten by
    /// a later full `new()` / `transition()`.
    pub fn new_minimal(oracle_name: &str, project: &str, working_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            oracle_name: oracle_name.to_string(),
            project: project.to_string(),
            mission_id: MissionId(String::new()),
            mission_text: String::new(),
            working_dir,
            // A placeholder oracle has not declared a mission yet, so the only
            // truthful initial phase is the first one (`Analyze`, matching
            // `new()`). `Dispatch` falsely implies workers are already being
            // dispatched; a later full `new()`/`transition()` sets the real phase.
            phase: OraclePhase::Analyze,
            workers: Vec::new(),
            god_mode: None,
            ship_requested: false,
            started_at: now,
            phase_entered_at: now,
            phase_history: Vec::new(),
            closeable_since: None,
            session_id: None,
        }
    }

    pub fn transition(&mut self, to: OraclePhase, reason: &str) {
        let from = self.phase;
        self.phase_history.push(PhaseTransition {
            from,
            to,
            at: Utc::now(),
            reason: reason.to_string(),
        });
        self.phase = to;
        self.phase_entered_at = Utc::now();
        tracing::info!(
            oracle = %self.oracle_name,
            from = ?from,
            to = ?to,
            reason = %reason,
            "Oracle phase transition"
        );
    }

    pub fn register_worker(&mut self, entry: WorkerEntry) {
        // Upsert — never DUPLICATE the same session (retry / concurrent
        // dispatch would otherwise break terminal detection), but a
        // RE-DISPATCH under the same deterministic worker name must refresh
        // the entry: patrol's worker freshness guard dates done signals
        // against `dispatched_at`, and keeping the old timestamp would let
        // the PREVIOUS mission's stale done.json pass as fresh and
        // insta-finish (then reap) the new worker.
        if let Some(w) = self
            .workers
            .iter_mut()
            .find(|w| w.session_name == entry.session_name)
        {
            *w = entry;
            return;
        }
        self.workers.push(entry);
    }

    pub fn update_worker_status(&mut self, session: &str, status: WorkerEntryStatus) {
        if let Some(w) = self.workers.iter_mut().find(|w| w.session_name == session) {
            w.status = status;
        }
    }

    pub fn all_workers_terminal(&self) -> bool {
        // Blocked + Stalled are terminal too — a worker in either state will not
        // progress on its own, so the oracle must move on (to Verify/Report)
        // instead of hanging in Monitor forever waiting for done_clean/failed.
        !self.workers.is_empty()
            && self.workers.iter().all(|w| {
                matches!(
                    w.status,
                    WorkerEntryStatus::DoneClean
                        | WorkerEntryStatus::Failed
                        | WorkerEntryStatus::Blocked
                        | WorkerEntryStatus::Stalled
                )
            })
    }

    /// N10: mark this oracle as having reached a real, closeable done-state
    /// (it wrote a genuine done-signal / finished its mission). Idempotent —
    /// the first timestamp wins so the close-grace clock isn't reset on every
    /// patrol tick. Reaching the terminal `Done` phase implies this too.
    pub fn mark_closeable(&mut self) {
        if self.closeable_since.is_none() {
            self.closeable_since = Some(Utc::now());
        }
    }

    /// N10: an oracle has a real done-signal / is in a closeable state when its
    /// phase is terminal `Done` OR `mark_closeable` was recorded. This is the
    /// authoritative "safe to reuse / close" predicate — STRICTLY stronger than
    /// `all_workers_terminal()`, which only means no worker will progress on its
    /// own (the oracle may still owe Verify/Report work).
    pub fn is_closeable(&self) -> bool {
        self.phase.is_terminal() || self.closeable_since.is_some()
    }

    /// Convenience alias for `is_closeable` matching the N10 wording — true when
    /// the oracle has written a real done-signal / reached a closeable state.
    pub fn has_done_signal(&self) -> bool {
        self.is_closeable()
    }

    pub fn any_worker_failed(&self) -> bool {
        self.workers
            .iter()
            .any(|w| w.status == WorkerEntryStatus::Failed)
    }

    pub fn running_workers(&self) -> Vec<&WorkerEntry> {
        self.workers
            .iter()
            .filter(|w| w.status == WorkerEntryStatus::Running)
            .collect()
    }

    pub fn duration_secs(&self) -> u64 {
        (Utc::now() - self.started_at).num_seconds().max(0) as u64
    }

    /// Filename key — the oracle name minus a single leading `oracle-`
    /// prefix, the SAME rule as `OracleDoneSignal::oracle_key`. Every
    /// caller holds the FULL session name (`oracle-X`), and formatting it
    /// straight into `oracle-{}.state.json` produced double-prefixed files
    /// (`oracle-oracle-X.state.json`) that the shell-side consumers
    /// (stuck-oracle-alert derives its done/progress probe paths from the
    /// state basename) then mis-keyed — finished oracles were never
    /// recognized as done and false stall alerts fired forever.
    fn state_key(oracle_name: &str) -> &str {
        oracle_name.strip_prefix("oracle-").unwrap_or(oracle_name)
    }

    /// Persist oracle state to disk for patrol/AISB visibility.
    pub fn write(&self, state_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let key = Self::state_key(&self.oracle_name);
        let path = state_dir.join(format!("oracle-{}.state.json", key));
        let tmp = state_dir.join(format!(".oracle-{}.state.json.tmp", key));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        // Migration: drop the legacy double-prefixed twin so `read_all`
        // never yields two states for one oracle.
        let legacy = state_dir.join(format!("oracle-{}.state.json", self.oracle_name));
        if legacy != path {
            let _ = std::fs::remove_file(&legacy);
        }
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle_name: &str) -> Result<Option<Self>> {
        let key = Self::state_key(oracle_name);
        let path = state_dir.join(format!("oracle-{}.state.json", key));
        if !path.exists() {
            // Lazy migration: a state written by a pre-normalization binary
            // lives at the double-prefixed name. Rename it into place so
            // every later read, read_all, and the shell consumers see the
            // canonical file; if the rename fails (perms/race), read the
            // legacy file in place rather than dropping live state.
            let legacy = state_dir.join(format!("oracle-oracle-{}.state.json", key));
            if !legacy.exists() {
                return Ok(None);
            }
            if std::fs::rename(&legacy, &path).is_err() {
                let content = std::fs::read_to_string(&legacy)?;
                return Ok(Some(serde_json::from_str(&content)?));
            }
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("oracle-") && name.ends_with(".state.json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(state) = serde_json::from_str::<OracleState>(&content) {
                                results.push(state);
                            }
                        }
                    }
                }
            }
        }
        results
    }

    pub fn remove(state_dir: &Path, oracle_name: &str) -> Result<()> {
        let key = Self::state_key(oracle_name);
        let path = state_dir.join(format!("oracle-{}.state.json", key));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        // Remove a lingering legacy double-prefixed file too.
        let legacy = state_dir.join(format!("oracle-oracle-{}.state.json", key));
        if legacy.exists() {
            std::fs::remove_file(&legacy)?;
        }
        Ok(())
    }
}

/// True when a worker entry will not progress on its own (the per-entry twin
/// of `OracleState::all_workers_terminal`).
pub fn worker_entry_terminal(status: WorkerEntryStatus) -> bool {
    matches!(
        status,
        WorkerEntryStatus::DoneClean
            | WorkerEntryStatus::Failed
            | WorkerEntryStatus::Blocked
            | WorkerEntryStatus::Stalled
    )
}

/// An oracle's LIVE worker sessions, split by whether they are finished.
#[derive(Debug, Default)]
pub struct LiveWorkers {
    /// Live session but the worker is finished (terminal registry status OR a
    /// written done-signal) — safe to cascade-close together with the oracle.
    pub terminal: Vec<String>,
    /// Live and still working — these BLOCK an oracle's done_clean (an oracle
    /// may not close itself while its workers run).
    pub running: Vec<String>,
}

impl LiveWorkers {
    pub fn all(&self) -> Vec<String> {
        self.terminal.iter().chain(self.running.iter()).cloned().collect()
    }
}

/// Resolve the live worker sessions governed by `oracle_name`.
///
/// Authoritative source: the oracle's own `OracleState.workers` registry.
/// Fallback (state file GC'd or never written — the exact shape of the
/// dentistrygpt orphan incident): live `<project>-worker-*` sessions of the
/// oracle's project that no OTHER oracle's state claims. Without the
/// fallback, a lost state file silently exempts every worker from the
/// close-gate and the cascade, recreating the zombie leak this exists to fix.
pub fn live_workers_of_oracle(
    state_dir: &Path,
    oracle_name: &str,
    live_sessions: &[crate::session::OmegaSession],
) -> LiveWorkers {
    use crate::session::{OmegaSession, SessionRole};

    let live: std::collections::HashSet<&str> =
        live_sessions.iter().map(|s| s.name.as_str()).collect();
    let all_states = OracleState::read_all(state_dir);
    let own_state = all_states.iter().find(|s| s.oracle_name == oracle_name);

    // A worker is "finished" when its registry status is terminal or it wrote
    // ANY done-signal (the signal marks the end of its run, whatever the
    // verdict; spawn-time stale-clear guarantees the file postdates dispatch).
    let finished = |name: &str, entry_status: Option<WorkerEntryStatus>| -> bool {
        if entry_status.is_some_and(worker_entry_terminal) {
            return true;
        }
        matches!(crate::done::DoneSignal::read(state_dir, name), Ok(Some(_)))
    };

    let mut out = LiveWorkers::default();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(state) = own_state {
        for w in &state.workers {
            if !live.contains(w.session_name.as_str()) {
                continue;
            }
            seen.insert(w.session_name.clone());
            if finished(&w.session_name, Some(w.status)) {
                out.terminal.push(w.session_name.clone());
            } else {
                out.running.push(w.session_name.clone());
            }
        }
    }

    // Project-name fallback for unregistered live workers.
    if let Some(project) = OmegaSession::classify(oracle_name).project {
        let owned_elsewhere: std::collections::HashSet<&str> = all_states
            .iter()
            .filter(|s| s.oracle_name != oracle_name)
            .flat_map(|s| s.workers.iter().map(|w| w.session_name.as_str()))
            .collect();
        for s in live_sessions {
            if s.role != SessionRole::Worker
                || s.project.as_deref() != Some(project.as_str())
                || seen.contains(&s.name)
                || owned_elsewhere.contains(s.name.as_str())
            {
                continue;
            }
            if finished(&s.name, None) {
                out.terminal.push(s.name.clone());
            } else {
                out.running.push(s.name.clone());
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// God Mode State Machine: WORK → VERIFYING → DONE (loop on failure)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodModeState {
    pub phase: GodModePhase,
    pub iteration: u32,
    pub max_iterations: u32,
    pub plan_phases_total: u32,
    pub plan_phases_completed: u32,
    pub previous_issues: Vec<String>,
    pub entered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GodModePhase {
    /// Planning: /planner generates task DAG
    Planning,
    /// Executing plan phases sequentially, workers per phase
    Working,
    /// Post-work: running /debugaudit verification
    Verifying,
    /// All phases done + verification passed
    Done,
}

impl GodModeState {
    pub fn new(max_iterations: u32) -> Self {
        Self {
            phase: GodModePhase::Planning,
            iteration: 1,
            max_iterations,
            plan_phases_total: 0,
            plan_phases_completed: 0,
            previous_issues: Vec::new(),
            entered_at: Utc::now(),
        }
    }

    pub fn transition(&mut self, to: GodModePhase) {
        self.phase = to;
        self.entered_at = Utc::now();
    }

    pub fn start_new_iteration(&mut self) -> bool {
        if self.iteration >= self.max_iterations {
            return false;
        }
        self.iteration += 1;
        self.phase = GodModePhase::Working;
        self.entered_at = Utc::now();
        true
    }

    pub fn record_issue(&mut self, issue: String) {
        self.previous_issues.push(issue);
    }
}

// ---------------------------------------------------------------------------
// Oracle Prompt Generator — per-project system prompt with context
// ---------------------------------------------------------------------------

pub struct OraclePromptGenerator;

impl OraclePromptGenerator {
    /// Generate the system prompt for an oracle session.
    ///
    /// Single source of truth: renders the shared `agents/oracle.md` v2
    /// template (installed `~/.omega/agents/oracle.md`, falling back to the
    /// repo copy) with the {{PROJECT}}/{{WORKDIR}}/{{SESSION}} placeholders,
    /// prepends the amplified mission as the Layer-3 body, and appends the
    /// per-mission dynamic blocks (ship / god mode). When the template file is
    /// missing (pre-install), falls back to a self-contained inline prompt so
    /// the oracle is never left without an identity.
    pub fn generate(
        project: &str,
        working_dir: &Path,
        oracle_name: &str,
        mission: &str,
        ship: bool,
        god_mode: bool,
    ) -> String {
        let mut prompt = String::with_capacity(4096);

        // Layer 3 — the mission body comes first so it's unmissable.
        prompt.push_str(&format!("## Mission\n{}\n\n---\n\n", mission));

        // Layer 3b — HOW to run this particular mission. Sits right under the
        // mission and above the standing identity, because the shape of the
        // work is mission-specific while the identity is not. Without it, an
        // oracle given "audit this" and one given "build this" received the
        // same generic advice and both defaulted to doing the work themselves
        // instead of spawning and supervising workers.
        prompt.push_str(&crate::mission_patterns::orchestration_block(mission));
        prompt.push_str("\n---\n\n");

        // Layer 2 — the shared v2 identity/protocol template.
        match Self::load_template(project, working_dir, oracle_name) {
            Some(tpl) => prompt.push_str(&tpl),
            None => Self::push_inline_fallback(&mut prompt, project, working_dir, oracle_name),
        }

        // Per-mission dynamic blocks.
        if ship {
            prompt.push_str(
                "## Ship Pipeline\n\
                 This mission requires shipping. After verification:\n\
                 build → gitleaks → commit → push → deploy → verify deploy.\n\
                 Use `omega ship` when ready.\n\n",
            );
        }

        // God Mode
        if god_mode {
            prompt.push_str(
                "## GOD MODE Active\n\
                 Multi-phase autonomous execution:\n\
                 1. PLAN: Generate task DAG with /planner\n\
                 2. EXECUTE: Phase by phase — dispatch workers, monitor, verify per phase\n\
                 3. VERIFY: Run /debugaudit after each phase, fix-loop until clean\n\
                 4. COMPLETE: All phases done + final verification → write signal file\n\
                 Pre-check: if 'Not Done' items exist from prior iteration → CONTINUE (don't ask)\n\n",
            );
        }

        prompt
    }

    /// Load + render the shared `agents/oracle.md` v2 template. Prefers the
    /// installed copy, falls back to the repo copy. Returns None if neither
    /// exists (the caller then uses the inline fallback).
    fn load_template(project: &str, working_dir: &Path, oracle_name: &str) -> Option<String> {
        let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
        let candidates = [
            home.join(".omega/agents/oracle.md"),
            std::path::PathBuf::from("agents/oracle.md"),
        ];
        let template = candidates.iter().find_map(|p| std::fs::read_to_string(p).ok())?;
        Some(
            template
                .replace("{{PROJECT}}", project)
                .replace("{{WORKDIR}}", &working_dir.display().to_string())
                .replace("{{SESSION}}", oracle_name),
        )
    }

    /// Self-contained inline identity/protocol, used only when the shared v2
    /// template file is missing (pre-install). Kept terse but complete.
    fn push_inline_fallback(
        prompt: &mut String,
        project: &str,
        working_dir: &Path,
        oracle_name: &str,
    ) {
        prompt.push_str(&format!(
            "## Project: {} ({})\n## Role: ORACLE\n## Session: {}\n\n\
             You are the Oracle — a PROJECT MANAGER, never edit code directly. Analyze,\n\
             decompose, dispatch workers, monitor, verify, report.\n\n\
             ## Three Laws\n\
             1. Code lies. Only runtime tells the truth.\n\
             2. Be a researcher, not a sycophant. Challenge flawed premises.\n\
             3. Decide and proceed, never wait.\n\n\
             ## Workflow\n\
             ANALYZE → DISPATCH (`omega spawn-worker <task> \"<prompt>\" --dir <dir> --files a,b`) → MONITOR\n\
             (`omega status`, inbox) → VERIFY (build + tests + audit + ground-truth) → REPORT\n\
             (`omega done {} done_clean \"<summary>\"`).\n\n\
             ## Quality Gate (before done)\n\
             - All workers done_clean AND survived the ground-truth gate\n\
             - Build = 0 errors, no runtime errors\n\
             - `omega gate {}` criteria satisfied\n",
            project,
            working_dir.display(),
            oracle_name,
            oracle_name,
            oracle_name,
        ));
    }

    /// Detect if a mission text implies shipping.
    pub fn should_ship(text: &str) -> bool {
        let lower = text.to_lowercase();
        let ship_keywords = [
            "ship",
            "deploy",
            "push",
            "merge",
            "en prod",
            "envoie en prod",
            "livre",
        ];
        ship_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Detect if a mission text requests god mode.
    pub fn is_god_mode(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("god mode") || lower.contains("godmode") || lower.contains("/godmode")
    }
}

// ---------------------------------------------------------------------------
// Signal File — oracle writes result, watcher detects completion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSignalFile {
    pub oracle: String,
    pub project: String,
    pub status: SignalStatus,
    pub build: SignalBuild,
    pub summary: String,
    pub not_done: Vec<String>,
    pub next_steps: Vec<String>,
    pub worker_results: Vec<WorkerResult>,
    pub duration_secs: u64,
    pub ship: Option<ShipResult>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalStatus {
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalBuild {
    Pass,
    Fail,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipResult {
    pub result: ShipOutcome,
    pub commit: Option<String>,
    pub deploy_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipOutcome {
    Ok,
    Failed,
    Skipped,
    Frozen,
}

impl OracleSignalFile {
    pub fn from_oracle_state(state: &OracleState, build: SignalBuild) -> Self {
        let worker_results: Vec<WorkerResult> = state
            .workers
            .iter()
            .map(|w| WorkerResult {
                task_id: w.task_id.clone(),
                session_name: w.session_name.clone(),
                status: match w.status {
                    WorkerEntryStatus::DoneClean => DoneStatus::DoneClean,
                    WorkerEntryStatus::Pending => DoneStatus::Pending,
                    WorkerEntryStatus::Failed => DoneStatus::Failed,
                    WorkerEntryStatus::Blocked => DoneStatus::Blocked,
                    _ => DoneStatus::Pending,
                },
                summary: String::new(),
                commit: None,
                duration_secs: 0,
            })
            .collect();

        let status = if state.any_worker_failed() || build == SignalBuild::Fail {
            SignalStatus::Failed
        } else {
            SignalStatus::Done
        };

        Self {
            oracle: state.oracle_name.clone(),
            project: state.project.clone(),
            status,
            build,
            summary: String::new(),
            not_done: Vec::new(),
            next_steps: Vec::new(),
            worker_results,
            duration_secs: state.duration_secs(),
            ship: None,
            created_at: Utc::now(),
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(state_dir)?;
        let path = state_dir.join(format!("oracle-{}.result.json", self.oracle));
        let tmp = state_dir.join(format!(".oracle-{}.result.json.tmp", self.oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(path)
    }

    /// Write a markdown signal file (human-readable, for backward compat with VPS watcher).
    pub fn write_markdown(&self, state_dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(state_dir)?;
        let path = state_dir.join(format!("oracle-result-{}.md", self.project));

        let status_str = match self.status {
            SignalStatus::Done => "DONE",
            SignalStatus::Failed => "FAILED",
        };
        let build_str = match self.build {
            SignalBuild::Pass => "PASS",
            SignalBuild::Fail => "FAIL",
            SignalBuild::Skipped => "SKIPPED",
        };

        let mut md = format!(
            "# Oracle Report -- {}\nPROJECT:{}\nSTATUS:{}\nBUILD:{}\n",
            self.project, self.project, status_str, build_str
        );

        md.push_str("## Resume\n");
        if self.summary.is_empty() {
            md.push_str("Mission completed.\n");
        } else {
            md.push_str(&self.summary);
            md.push('\n');
        }

        if !self.not_done.is_empty() {
            md.push_str("## Not Done\n");
            for item in &self.not_done {
                md.push_str(&format!("- {}\n", item));
            }
        }

        if !self.next_steps.is_empty() {
            md.push_str("## Next Steps\n");
            for step in &self.next_steps {
                md.push_str(&format!("- {}\n", step));
            }
        }

        std::fs::write(&path, &md)?;
        Ok(path)
    }

    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("oracle-{}.result.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }
}

// ---------------------------------------------------------------------------
// Signal File Watcher — poll for new oracle result files
// ---------------------------------------------------------------------------

pub struct SignalWatcher {
    state_dir: PathBuf,
    known: HashMap<String, DateTime<Utc>>,
}

impl SignalWatcher {
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            state_dir,
            known: HashMap::new(),
        }
    }

    /// Scan for new or updated signal files since last check.
    /// Returns list of (oracle_name, signal_file) pairs that are new.
    pub fn poll(&mut self) -> Result<Vec<(String, OracleSignalFile)>> {
        let mut new_signals = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("oracle-") && name.ends_with(".result.json") {
                        let oracle = name
                            .strip_prefix("oracle-")
                            .and_then(|s| s.strip_suffix(".result.json"))
                            .unwrap_or("")
                            .to_string();

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(signal) =
                                serde_json::from_str::<OracleSignalFile>(&content)
                            {
                                let is_new = self
                                    .known
                                    .get(&oracle)
                                    .map(|prev| signal.created_at > *prev)
                                    .unwrap_or(true);

                                if is_new {
                                    self.known.insert(oracle.clone(), signal.created_at);
                                    new_signals.push((oracle, signal));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(new_signals)
    }
}

// ---------------------------------------------------------------------------
// Worker Stall Detector — captures pane output, detects idle prompt
// ---------------------------------------------------------------------------

/// Thresholds for stall detection, matching the VPS Shadow Manager signals.
pub struct StallThresholds {
    /// Seconds of idle at prompt before sending a nudge
    pub nudge_after_secs: u64,
    /// Seconds of idle at prompt before escalating to oracle
    pub escalate_after_secs: u64,
}

impl Default for StallThresholds {
    fn default() -> Self {
        Self {
            nudge_after_secs: 30,
            escalate_after_secs: 300,
        }
    }
}

pub struct WorkerStallDetector {
    thresholds: StallThresholds,
    last_active: HashMap<String, std::time::Instant>,
    idle_since: HashMap<String, std::time::Instant>,
}

#[derive(Debug, Clone)]
pub enum StallAction {
    Active,
    Nudge { session: String, idle_secs: u64 },
    Escalate { session: String, idle_secs: u64 },
}

impl WorkerStallDetector {
    pub fn new(thresholds: StallThresholds) -> Self {
        Self {
            thresholds,
            last_active: HashMap::new(),
            idle_since: HashMap::new(),
        }
    }

    /// Analyze pane content to determine if the worker is idle.
    /// Returns the appropriate stall action.
    pub fn check(&mut self, session: &str, pane_content: &str) -> StallAction {
        let is_idle = Self::detect_idle_prompt(pane_content);
        let now = std::time::Instant::now();

        if is_idle {
            let idle_start = self.idle_since.entry(session.to_string()).or_insert(now);
            let idle_secs = now.duration_since(*idle_start).as_secs();

            if idle_secs >= self.thresholds.escalate_after_secs {
                return StallAction::Escalate {
                    session: session.to_string(),
                    idle_secs,
                };
            } else if idle_secs >= self.thresholds.nudge_after_secs {
                return StallAction::Nudge {
                    session: session.to_string(),
                    idle_secs,
                };
            }
        } else {
            // Worker is active — reset idle tracking
            self.idle_since.remove(session);
            self.last_active.insert(session.to_string(), now);
        }

        StallAction::Active
    }

    /// Detect if pane shows an idle prompt (worker finished or stuck).
    /// Checks for common prompt patterns: `❯`, `$`, `>` at end of visible output.
    fn detect_idle_prompt(content: &str) -> bool {
        let trimmed = content.trim();
        // N11: an EMPTY pane capture is NOT proof of an idle prompt — it usually
        // means the capture raced a redraw or the session is mid-spawn. Treating
        // it as idle was a false-positive that mis-escalated live workers. Empty
        // => not idle.
        if trimmed.is_empty() {
            return false;
        }

        // Check the last non-empty line, trimming trailing whitespace
        let last_line = trimmed.lines().last().unwrap_or("");
        let last_trimmed = last_line.trim_end();

        // Claude Code / zsh prompt indicator. '❯' is a prompt glyph that
        // virtually never terminates a code/markup line, so a line ending in it
        // is an idle prompt (unchanged behaviour). The ambiguous case the N11
        // exclusions target is the ASCII '>' below.
        if last_trimmed.ends_with('❯') {
            return true;
        }

        // Standard shell prompt patterns (with or without trailing space)
        if last_trimmed.ends_with('$')
            || last_trimmed.ends_with('%')
            || last_trimmed.ends_with('#')
        {
            return true;
        }

        // Bare '>' / '❯' prompt — but ONLY when it's a standalone prompt, never
        // when the line ends in code/markup punctuation that merely closes with
        // '>': '=>', '->', '/>', '});', generic-type '<T>', JSX/HTML tags, etc.
        // Those are output, not an idle prompt.
        if last_trimmed.ends_with('>') {
            return is_standalone_prompt(last_trimmed, '>');
        }

        false
    }

    /// Remove tracking for a session (e.g., after it's killed).
    pub fn forget(&mut self, session: &str) {
        self.last_active.remove(session);
        self.idle_since.remove(session);
    }
}

/// True only when `line` is a *standalone* prompt ending in `glyph` (one of
/// `>` / `❯`), not a code/markup line that merely happens to end with it.
///
/// Excludes the common code/markup line-endings that close with `>`: `=>`,
/// `->`, `/>`, `<T>` / `<Foo>` generics, JSX/HTML tags, and a `});`-style
/// statement that still ends with `>` after a fat-arrow body. A genuine idle
/// prompt is a short line: either just the glyph, or a shell-style
/// `user@host …>` style prefix followed by the glyph.
fn is_standalone_prompt(line: &str, glyph: char) -> bool {
    let line = line.trim_end();
    if !line.ends_with(glyph) {
        return false;
    }
    // The character immediately before the prompt glyph decides intent.
    // Code/markup endings put punctuation right before the closing '>'.
    let before = line[..line.len() - glyph.len_utf8()].trim_end_matches(' ');
    if let Some(prev) = before.chars().last() {
        // '=>' '->' '/>' and tag/generic closers like 'foo>' are NOT prompts;
        // a real prompt has whitespace (or nothing) before the glyph.
        if matches!(prev, '=' | '-' | '/' | ')' | ';' | '"' | '\'' | '`') {
            return false;
        }
        // An alphanumeric immediately before '>' is a closing tag / generic
        // (e.g. `</div>`, `Vec<T>`), not a bare prompt.
        if prev.is_alphanumeric() {
            return false;
        }
    }
    // Markup/JSX tag lines also typically OPEN with '<' somewhere — a line that
    // both contains '<' and ends with '>' is overwhelmingly a tag, not a prompt.
    if glyph == '>' && before.contains('<') {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Oracle Registry — tracks all active oracles across projects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRegistry {
    pub oracles: Vec<OracleRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRegistryEntry {
    pub oracle_name: String,
    pub project: String,
    pub session_name: String,
    pub status: OracleRegistryStatus,
    pub spawned_at: DateTime<Utc>,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleRegistryStatus {
    Active,
    Idle,
    Done,
    Dead,
}

impl OracleRegistry {
    pub fn load(state_dir: &Path) -> Self {
        let path = state_dir.join("oracle-registry.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(reg) = serde_json::from_str::<Self>(&content) {
                    return reg;
                }
            }
        }
        Self {
            oracles: Vec::new(),
        }
    }

    pub fn save(&self, state_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let path = state_dir.join("oracle-registry.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Atomically reserve + register an Active oracle name for `project`,
    /// serialized across processes by an exclusive advisory lock (fs2, the same
    /// pattern as scope.rs / inbox.rs). Without it two concurrent dispatches
    /// both load the registry, both compute the SAME next_oracle_name, and both
    /// save — colliding on one name and losing a registration. The lock guards
    /// a fresh reload + name pick + register + save, so each caller sees the
    /// other's reservation. `preferred` (a caller-verified live idle oracle to
    /// reuse) is honored only if still Idle in the reloaded registry. The lock
    /// guards only synchronous IO — do any async liveness check BEFORE calling.
    pub fn reserve_oracle(
        state_dir: &Path,
        project: &str,
        preferred: Option<&str>,
    ) -> Result<String> {
        std::fs::create_dir_all(state_dir)?;
        let lockfile = std::fs::File::create(state_dir.join(".oracle-registry.lock"))?;
        lockfile.lock_exclusive()?;

        // Reload under the lock so a concurrent reservation is merged, not lost.
        let mut reg = Self::load(state_dir);
        let name = match preferred {
            Some(p)
                if reg
                    .oracles
                    .iter()
                    .any(|e| e.oracle_name == p && e.status == OracleRegistryStatus::Idle) =>
            {
                p.to_string()
            }
            _ => reg.next_oracle_name(project),
        };
        reg.register(OracleRegistryEntry {
            oracle_name: name.clone(),
            project: project.to_string(),
            session_name: name.clone(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        });
        reg.save(state_dir)?;
        Ok(name)
        // lockfile drops here -> advisory lock released
    }

    pub fn register(&mut self, entry: OracleRegistryEntry) {
        // Replace existing entry for same oracle name
        self.oracles.retain(|e| e.oracle_name != entry.oracle_name);
        self.oracles.push(entry);
    }

    /// Apply mutations to the registry atomically, under the same exclusive
    /// advisory lock as `reserve_oracle`/`register_resurrected`. Patrol's
    /// previous pattern — load at tick start, mutate across awaits, save at
    /// the end — clobbered any oracle a concurrent locked dispatch
    /// registered mid-tick: the lost entry took its `spawned_at` with it,
    /// the freshness guard then treated EVERY signal of that oracle as
    /// stale (never upgraded, never reaped), and `next_oracle_name` could
    /// re-issue its name while the session was live. The closure runs on a
    /// FRESH reload while the lock is held — keep it synchronous and quick.
    pub fn update_locked<F>(state_dir: &Path, mutate: F) -> Result<()>
    where
        F: FnOnce(&mut OracleRegistry),
    {
        std::fs::create_dir_all(state_dir)?;
        let lockfile = std::fs::File::create(state_dir.join(".oracle-registry.lock"))?;
        lockfile.lock_exclusive()?;
        let mut reg = Self::load(state_dir);
        mutate(&mut reg);
        reg.save(state_dir)
        // lockfile drops here -> advisory lock released
    }

    /// Re-register a resurrected oracle as Active with a FRESH `spawned_at`,
    /// under the same exclusive lock as reserve_oracle. The resurrect path
    /// previously never re-registered (the dead entry had been purged by
    /// cleanup), so the resurrected oracle was invisible to the registry and —
    /// critically — patrol had no `spawned_at` to date the session's done
    /// signal against: its freshness guard then treats every signal as stale.
    pub fn register_resurrected(
        state_dir: &Path,
        oracle_name: &str,
        project: &str,
    ) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let lockfile = std::fs::File::create(state_dir.join(".oracle-registry.lock"))?;
        lockfile.lock_exclusive()?;
        let mut reg = Self::load(state_dir);
        reg.register(OracleRegistryEntry {
            oracle_name: oracle_name.to_string(),
            project: project.to_string(),
            session_name: oracle_name.to_string(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        });
        reg.save(state_dir)
        // lockfile drops here -> advisory lock released
    }

    pub fn find_available(&self, project: &str) -> Option<&OracleRegistryEntry> {
        self.oracles
            .iter()
            .find(|e| e.project == project && e.status == OracleRegistryStatus::Idle)
    }

    pub fn find_active(&self, project: &str) -> Vec<&OracleRegistryEntry> {
        self.oracles
            .iter()
            .filter(|e| e.project == project && e.status == OracleRegistryStatus::Active)
            .collect()
    }

    pub fn count_active(&self, project: &str) -> usize {
        self.oracles
            .iter()
            .filter(|e| {
                e.project == project
                    && matches!(
                        e.status,
                        OracleRegistryStatus::Active | OracleRegistryStatus::Idle
                    )
            })
            .count()
    }

    pub fn mark_status(&mut self, oracle_name: &str, status: OracleRegistryStatus) {
        if let Some(entry) = self.oracles.iter_mut().find(|e| e.oracle_name == oracle_name) {
            entry.status = status;
        }
    }

    /// Remove dead entries (sessions that no longer exist in rmux).
    /// `Done` entries are purged too once their session is gone — retaining
    /// them forever grew `next_oracle_name`'s index without bound and
    /// contradicted the dispatch-time "Dead-purged → name re-issued"
    /// contract (the stale-signal clear exists precisely because names
    /// recycle). A freshly-spawned entry is exempt for a short grace
    /// window: the caller's session list may be a snapshot taken BEFORE
    /// the spawn (patrol snapshots sessions at tick start), and purging it
    /// would erase the `spawned_at` the freshness guard depends on.
    pub fn cleanup(&mut self, live_sessions: &[String]) {
        const SPAWN_GRACE_SECS: i64 = 120;
        let now = Utc::now();
        for entry in &mut self.oracles {
            if !live_sessions.contains(&entry.session_name)
                && (now - entry.spawned_at).num_seconds() > SPAWN_GRACE_SECS
            {
                entry.status = OracleRegistryStatus::Dead;
            }
        }
        self.oracles
            .retain(|e| e.status != OracleRegistryStatus::Dead);
    }

    /// Next available oracle name for a project (oracle-Proj, oracle-Proj-2, ...).
    pub fn next_oracle_name(&self, project: &str) -> String {
        let existing: Vec<_> = self
            .oracles
            .iter()
            .filter(|e| e.project == project)
            .collect();

        if existing.is_empty() {
            return format!("oracle-{}", project);
        }

        let max_idx = existing
            .iter()
            .filter_map(|e| {
                e.oracle_name
                    .strip_prefix(&format!("oracle-{}-", project))
                    .and_then(|s| s.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(1);

        format!("oracle-{}-{}", project, max_idx + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stall_detector_idle_prompt() {
        assert!(WorkerStallDetector::detect_idle_prompt("some output\n❯"));
        assert!(WorkerStallDetector::detect_idle_prompt("some output\n❯ "));
        assert!(WorkerStallDetector::detect_idle_prompt("hacker@vps ~ $ "));
        // N11: an empty pane capture is NOT an idle prompt (was a false-positive).
        assert!(!WorkerStallDetector::detect_idle_prompt(""));
        assert!(!WorkerStallDetector::detect_idle_prompt("Thinking..."));
        assert!(!WorkerStallDetector::detect_idle_prompt("Writing file.rs"));
        assert!(!WorkerStallDetector::detect_idle_prompt("Running tests..."));
    }

    #[test]
    fn idle_prompt_excludes_code_and_markup_endings() {
        // N11: a line that ends in '>' but is code/markup is NOT an idle prompt.
        assert!(!WorkerStallDetector::detect_idle_prompt("let f = |x| x => x + 1"));
        assert!(!WorkerStallDetector::detect_idle_prompt("fn foo() -> u32 {\nreturn 1 ->"));
        assert!(!WorkerStallDetector::detect_idle_prompt("<input type=\"text\" />"));
        assert!(!WorkerStallDetector::detect_idle_prompt("    </div>"));
        assert!(!WorkerStallDetector::detect_idle_prompt("let v: Vec<T>"));
        assert!(!WorkerStallDetector::detect_idle_prompt("arr.map(x => x);"));
        // A genuine bare '>' prompt still counts.
        assert!(WorkerStallDetector::detect_idle_prompt("output\n>"));
        assert!(WorkerStallDetector::detect_idle_prompt("output\n> "));
    }

    #[test]
    fn oracle_phase_transitions() {
        let mission = Mission::new("TestProject", "Fix a bug", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-TestProject", &mission);

        assert_eq!(state.phase, OraclePhase::Analyze);
        state.transition(OraclePhase::Dispatch, "Analysis complete");
        assert_eq!(state.phase, OraclePhase::Dispatch);
        assert_eq!(state.phase_history.len(), 1);
    }

    #[test]
    fn god_mode_iteration_limit() {
        let mut gm = GodModeState::new(3);
        assert!(gm.start_new_iteration()); // iter 2
        assert!(gm.start_new_iteration()); // iter 3
        assert!(!gm.start_new_iteration()); // at max, returns false
    }

    #[test]
    fn signal_file_markdown_format() {
        let mission = Mission::new("MyProject", "Do something", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-MyProject", &mission);
        let mut signal = OracleSignalFile::from_oracle_state(&state, SignalBuild::Pass);
        signal.summary = "Fixed the auth flow.".to_string();
        signal.next_steps = vec!["Deploy to prod".to_string()];

        let tmp = tempfile::TempDir::new().unwrap();
        let path = signal.write_markdown(tmp.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(content.contains("PROJECT:MyProject"));
        assert!(content.contains("STATUS:DONE"));
        assert!(content.contains("BUILD:PASS"));
        assert!(content.contains("Fixed the auth flow."));
        assert!(content.contains("Deploy to prod"));
    }

    #[test]
    fn oracle_registry_next_name() {
        let mut reg = OracleRegistry {
            oracles: Vec::new(),
        };

        assert_eq!(reg.next_oracle_name("Kommu"), "oracle-Kommu");

        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-Kommu".to_string(),
            project: "Kommu".to_string(),
            session_name: "oracle-Kommu".to_string(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: Vec::new(),
        });

        assert_eq!(reg.next_oracle_name("Kommu"), "oracle-Kommu-2");
    }

    #[test]
    fn reserve_oracle_concurrent_names_are_unique() {
        use std::collections::HashSet;
        use std::sync::Arc;
        use std::thread;
        const N: usize = 8;

        let tmp = tempfile::TempDir::new().unwrap();
        let dir = Arc::new(tmp.path().to_path_buf());

        // N threads race to reserve an oracle for the same project at once.
        let mut handles = Vec::new();
        for _ in 0..N {
            let dir = Arc::clone(&dir);
            handles.push(thread::spawn(move || {
                OracleRegistry::reserve_oracle(dir.as_path(), "Race", None).unwrap()
            }));
        }
        let names: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // The exclusive lock serializes name allocation, so every concurrent
        // reservation gets a DISTINCT name (pre-fix they all read the same stale
        // registry and collided on oracle-Race / -2 …).
        let unique: HashSet<&String> = names.iter().collect();
        assert_eq!(unique.len(), N, "concurrent reservations must be unique: {:?}", names);

        // All N registrations survive (none clobbered by a racing save).
        let reg = OracleRegistry::load(dir.as_path());
        assert_eq!(reg.oracles.iter().filter(|e| e.project == "Race").count(), N);
    }

    #[test]
    fn oracle_state_filename_is_prefix_normalized() {
        // Callers hold the FULL session name (`oracle-X`); the file on disk
        // must be single-prefixed (`oracle-X.state.json`), never
        // `oracle-oracle-X.state.json` — same rule as OracleDoneSignal.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-Acme-2", &mission);
        state.write(dir).unwrap();

        assert!(dir.join("oracle-Acme-2.state.json").exists());
        assert!(!dir.join("oracle-oracle-Acme-2.state.json").exists());

        // Both name forms resolve the same file.
        assert!(OracleState::read(dir, "oracle-Acme-2").unwrap().is_some());
        assert!(OracleState::read(dir, "Acme-2").unwrap().is_some());
    }

    #[test]
    fn oracle_state_read_migrates_legacy_double_prefix() {
        // A state written by a pre-normalization binary lives at the
        // double-prefixed name; read() must find it AND rename it into place.
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let state = OracleState::new("oracle-Acme", &mission);
        let legacy = dir.join("oracle-oracle-Acme.state.json");
        std::fs::write(&legacy, serde_json::to_string_pretty(&state).unwrap()).unwrap();

        let read = OracleState::read(dir, "oracle-Acme").unwrap();
        assert!(read.is_some(), "legacy state must remain readable");
        assert!(dir.join("oracle-Acme.state.json").exists(), "must migrate in place");
        assert!(!legacy.exists(), "legacy file must be renamed away");
    }

    #[test]
    fn register_worker_upsert_refreshes_dispatched_at() {
        // A re-dispatch under the same deterministic worker name must
        // refresh dispatched_at (the worker freshness guard dates done
        // signals against it) without duplicating the entry.
        let mission = Mission::new("Acme", "do it", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-Acme", &mission);
        let t0 = Utc::now() - chrono::Duration::hours(2);
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-x".into(),
            task_id: "t1".into(),
            task_name: "x".into(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec![],
            dispatched_at: t0,
            status: WorkerEntryStatus::DoneClean,
        });
        let t1 = Utc::now();
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-x".into(),
            task_id: "t1".into(),
            task_name: "x".into(),
            attempt_id: None,
            plan_revision: None,
            files_owned: vec![],
            dispatched_at: t1,
            status: WorkerEntryStatus::Running,
        });
        assert_eq!(state.workers.len(), 1, "no duplicate entry");
        assert_eq!(state.workers[0].dispatched_at, t1);
        assert_eq!(state.workers[0].status, WorkerEntryStatus::Running);
    }

    #[test]
    fn cleanup_purges_done_entries_with_dead_sessions_after_grace() {
        let mut reg = OracleRegistry { oracles: Vec::new() };
        let old = Utc::now() - chrono::Duration::hours(1);
        // Done + dead session + past the spawn grace → purged.
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-A".into(),
            project: "A".into(),
            session_name: "oracle-A".into(),
            status: OracleRegistryStatus::Done,
            spawned_at: old,
            files_owned: vec![],
        });
        // Active + dead but JUST spawned → kept (snapshot may predate spawn).
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-B".into(),
            project: "B".into(),
            session_name: "oracle-B".into(),
            status: OracleRegistryStatus::Active,
            spawned_at: Utc::now(),
            files_owned: vec![],
        });
        // Done + LIVE session → kept.
        reg.register(OracleRegistryEntry {
            oracle_name: "oracle-C".into(),
            project: "C".into(),
            session_name: "oracle-C".into(),
            status: OracleRegistryStatus::Done,
            spawned_at: old,
            files_owned: vec![],
        });
        reg.cleanup(&["oracle-C".to_string()]);
        let names: Vec<&str> = reg.oracles.iter().map(|e| e.oracle_name.as_str()).collect();
        assert!(!names.contains(&"oracle-A"), "Done+dead+aged must be purged");
        assert!(names.contains(&"oracle-B"), "fresh spawn must survive a stale snapshot");
        assert!(names.contains(&"oracle-C"), "Done+live must be retained");
    }

    #[test]
    fn update_locked_persists_mutations() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        OracleRegistry::reserve_oracle(dir, "Acme", None).unwrap();
        OracleRegistry::update_locked(dir, |reg| {
            reg.mark_status("oracle-Acme", OracleRegistryStatus::Done);
        })
        .unwrap();
        let reg = OracleRegistry::load(dir);
        assert_eq!(reg.oracles[0].status, OracleRegistryStatus::Done);
    }

    #[test]
    fn ship_detection() {
        assert!(OraclePromptGenerator::should_ship("Fix auth and deploy"));
        assert!(OraclePromptGenerator::should_ship("envoie en prod"));
        assert!(OraclePromptGenerator::should_ship("ship the feature"));
        assert!(!OraclePromptGenerator::should_ship("just fix the bug"));
    }

    #[test]
    fn god_mode_detection() {
        assert!(OraclePromptGenerator::is_god_mode("Run in god mode"));
        assert!(OraclePromptGenerator::is_god_mode("/godmode fix everything"));
        assert!(!OraclePromptGenerator::is_god_mode("Fix the login flow"));
    }

    // The operator's stated objective: oracles must LAUNCH worker sessions they
    // supervise, not do the work themselves. That only happens if the generated
    // prompt actually says so, per mission. This test fails the moment the
    // orchestration block stops reaching a real oracle prompt.
    #[test]
    fn the_generated_oracle_prompt_carries_the_mission_shape() {
        let prompt = OraclePromptGenerator::generate(
            "camelia",
            std::path::Path::new("/tmp/camelia"),
            "oracle-camelia-1",
            "audite le code et corrige ce que tu trouves en parallele",
            false,
            false,
        );
        assert!(prompt.contains("How to run THIS mission"), "shape block missing");
        assert!(prompt.contains("P-AUDIT"), "an audit mission must match the audit shape");
        assert!(prompt.contains("P-PARALLEL"), "\"en parallele\" must match the parallel shape");
        assert!(prompt.contains("spawn-worker"), "it must tell the oracle to dispatch");
        assert!(prompt.contains("Done when"), "it must carry a stop condition");
        // And the mission itself still leads.
        assert!(prompt.starts_with("## Mission"));
    }

    /// A mission that matches nothing still gets the orchestration floor —
    /// never a prompt with no shape at all.
    #[test]
    fn an_unclassifiable_mission_still_gets_a_shape() {
        let prompt = OraclePromptGenerator::generate(
            "p",
            std::path::Path::new("/tmp/p"),
            "oracle-p-1",
            "do the thing",
            false,
            false,
        );
        assert!(prompt.contains("How to run THIS mission"));
        assert!(prompt.contains("spawn-worker"));
    }
}
