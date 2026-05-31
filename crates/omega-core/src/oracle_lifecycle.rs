//! Oracle Lifecycle — 5-step workflow state machine.
//!
//! Implements the VPS Oracle pattern: Analyze → Dispatch → Monitor → Verify → Report.
//! Oracles are on-demand project managers: they decompose, delegate, monitor, and verify.
//! They NEVER edit code directly — all code changes go through workers.

use crate::done::DoneStatus;
use crate::mission::{Mission, MissionId, WorkerResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerEntry {
    pub session_name: String,
    pub task_id: String,
    pub task_name: String,
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
        // Idempotent — never double-register the same session (retry / concurrent
        // dispatch would otherwise duplicate the entry and break terminal detection).
        if self.workers.iter().any(|w| w.session_name == entry.session_name) {
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

    /// Persist oracle state to disk for patrol/AISB visibility.
    pub fn write(&self, state_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let path = state_dir.join(format!("oracle-{}.state.json", self.oracle_name));
        let tmp = state_dir.join(format!(".oracle-{}.state.json.tmp", self.oracle_name));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle_name: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("oracle-{}.state.json", oracle_name));
        if !path.exists() {
            return Ok(None);
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
        let path = state_dir.join(format!("oracle-{}.state.json", oracle_name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
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
        if trimmed.is_empty() {
            return true;
        }

        // Check the last non-empty line, trimming trailing whitespace
        let last_line = trimmed.lines().last().unwrap_or("");
        let last_trimmed = last_line.trim_end();

        // Claude Code prompt indicators
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

        // Prompt ending with "> " or bare ">"
        if last_trimmed.ends_with('>') {
            return true;
        }

        false
    }

    /// Remove tracking for a session (e.g., after it's killed).
    pub fn forget(&mut self, session: &str) {
        self.last_active.remove(session);
        self.idle_since.remove(session);
    }
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

    pub fn register(&mut self, entry: OracleRegistryEntry) {
        // Replace existing entry for same oracle name
        self.oracles.retain(|e| e.oracle_name != entry.oracle_name);
        self.oracles.push(entry);
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
    pub fn cleanup(&mut self, live_sessions: &[String]) {
        for entry in &mut self.oracles {
            if entry.status != OracleRegistryStatus::Done
                && !live_sessions.contains(&entry.session_name)
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
        assert!(WorkerStallDetector::detect_idle_prompt(""));
        assert!(!WorkerStallDetector::detect_idle_prompt("Thinking..."));
        assert!(!WorkerStallDetector::detect_idle_prompt("Writing file.rs"));
        assert!(!WorkerStallDetector::detect_idle_prompt("Running tests..."));
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
}
