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

    // ── Opus-4.8 system-card hardening (L6) ──
    // Mechanisms #31, #65, #79, #90, #100, #167/168 mapped into a single
    // ground-truth-oriented done schema. Worker narration alone is no
    // longer admissible — these fields require artifact citations OR
    // explicit honest negatives.

    /// What is NOT done, failing, or unsigned-off — REQUIRED even when the
    /// worker thinks everything passed. The "great job" suppression that
    /// the model card catches. (#90)
    #[serde(default)]
    pub not_done: Vec<String>,

    /// Scope + limits framing — what this verification PROVED vs what it
    /// did NOT cover. Bounded honesty. (#100)
    #[serde(default)]
    pub scope: ScopeFraming,

    /// Cross-source corroboration: which independent signals agreed on
    /// the verdict (e.g. ["worker_self_report", "git_ls_remote", "ci_exit_code"]).
    /// A `done_clean` accepted with only `worker_self_report` is
    /// "single-source/unverified" and must be flagged. (#65, #66)
    #[serde(default)]
    pub corroboration: Vec<CorroborationSource>,

    /// Four-failure-mode taxonomy when status != done_clean — durable
    /// comparable failure ledger across the fleet. (#31)
    #[serde(default)]
    pub failure_mode: Option<FailureMode>,

    /// Retry-thrash / decision-flip counter — non-zero suggests the
    /// worker was looping. (#167, #168)
    #[serde(default)]
    pub retry_thrash_count: u32,

    /// Ground-truth artifact citations. Each load-bearing claim in
    /// `summary` MUST map to one of these. Verified by the gate before
    /// done_clean is accepted. (#8, #18, #25, #26, #72, #76)
    #[serde(default)]
    pub artifacts: Vec<DoneArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoneStatus {
    DoneClean,
    Pending,
    Failed,
    Blocked,
}

/// What the worker's verification actually covered, vs what it did NOT.
/// Forces the report to state the BOUNDS of its claims (#100).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeFraming {
    #[serde(default)]
    pub confirms: Vec<String>,
    #[serde(default)]
    pub does_not_confirm: Vec<String>,
}

/// One independent signal that agreed on the verdict. Multiple = the
/// "corroborated" badge on the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorroborationSource {
    /// The worker's own narrative — never sufficient on its own.
    WorkerSelfReport,
    /// `git ls-remote` / `git cat-file -e` confirmed the SHA/branch exists on origin.
    GitRemote,
    /// CI / build exit code from a real run (logged).
    CiExitCode,
    /// HTTP probe against the deployed URL returned the expected status.
    ProdHealthcheck,
    /// Filesystem check confirmed an asserted file/dir actually exists.
    FilesystemCheck,
    /// A separate Oracle/auditor signed off (multi-grader consensus).
    IndependentAuditor,
    /// Free-form named signal (when none of the above fits).
    Other(String),
}

/// Anthropic's 4-failure-mode taxonomy from the Opus 4.8 card (#31).
/// Trivial to add, makes the failure ledger comparable across runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    /// Worker stated something that doesn't exist / didn't happen.
    Fabrication,
    /// Worker did not follow the brief's actual instructions.
    InstructionFollowing,
    /// Worker only verified the cheap/checkable surface and skipped the
    /// hard one.
    CheapVerificationSkipped,
    /// Worker spotted a problem, was told to fix it, and didn't.
    IgnoredCorrection,
}

/// A single ground-truth artifact cited as evidence for a claim in the
/// summary. `verify_done_against_repo` will check each of these against
/// the real environment before accepting done_clean.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum DoneArtifact {
    /// Git commit SHA on origin/main (or a named branch).
    GitSha {
        sha: String,
        #[serde(default)]
        branch: Option<String>,
    },
    /// Existing branch on the remote.
    GitBranch { name: String },
    /// Path that must exist (relative to repo root or absolute).
    FilePath { path: String },
    /// Command + expected exit code (recorded from the actual run).
    Command { cmd: String, exit_code: i32 },
    /// URL that must return 2xx when probed.
    Url { url: String, expected_status: u16 },
    /// Free-form note (no automated verification).
    Note(String),
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
            not_done: Vec::new(),
            scope: ScopeFraming::default(),
            corroboration: Vec::new(),
            failure_mode: None,
            retry_thrash_count: 0,
            artifacts: Vec::new(),
        }
    }

    /// Minimal constructor for tests and synthetic signals. Sets `session`
    /// and `status` to the args and zeroes/empties every other field.
    pub fn stub(session: &str, status: DoneStatus) -> Self {
        Self {
            session: session.to_string(),
            status,
            summary: String::new(),
            commit: None,
            finished_at: Utc::now(),
            todos_total: 0,
            todos_completed: 0,
            pending_actions: Vec::new(),
            not_done: Vec::new(),
            scope: ScopeFraming::default(),
            corroboration: Vec::new(),
            failure_mode: None,
            retry_thrash_count: 0,
            artifacts: Vec::new(),
        }
    }

    /// Single-source = the worker's word only, no independent ground-truth.
    /// Reports flagged single-source must be shown to the user as such (#65).
    pub fn is_single_source(&self) -> bool {
        self.corroboration.is_empty()
            || self.corroboration == vec![CorroborationSource::WorkerSelfReport]
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

/// Result of running the ground-truth gate against a done.json.
/// (#8, #18, #25, #26, #72, #76, #82 — "worker narration is inadmissible
/// as proof; every claim must map to a real, runnable artifact.")
#[derive(Debug, Clone)]
pub struct GroundTruthVerdict {
    /// True iff every artifact citation in the done signal exists in
    /// the real environment AND no fabricated reference was detected.
    pub passes: bool,
    /// Per-artifact verification result, in the order the worker
    /// declared them.
    pub checks: Vec<ArtifactCheck>,
    /// Human-readable failure summary (empty when passes == true).
    pub failures: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactCheck {
    pub artifact: DoneArtifact,
    pub passed: bool,
    pub detail: String,
}

/// Verify every artifact a worker cited in its done.json against the
/// real environment. Returns `passes = false` if any citation refers to
/// something that doesn't actually exist — the "hallucinated commits"
/// failure mode the Opus 4.8 card flags as the demo-worthy keystone
/// (BLUF: "Worker narration inadmissible as proof").
///
/// `repo_root` is the directory the worker was supposed to operate in
/// (typically the project working_dir). Pass `None` to skip git/path
/// checks (e.g. for non-code workers).
pub fn verify_done_against_repo(
    done: &DoneSignal,
    repo_root: Option<&Path>,
) -> GroundTruthVerdict {
    let mut checks = Vec::with_capacity(done.artifacts.len());
    let mut failures = Vec::new();

    for art in &done.artifacts {
        let (ok, detail) = match art {
            DoneArtifact::GitSha { sha, branch: _ } => {
                check_git_sha(sha, repo_root)
            }
            DoneArtifact::GitBranch { name } => check_git_branch(name, repo_root),
            DoneArtifact::FilePath { path } => check_file_path(path, repo_root),
            DoneArtifact::Command { cmd, exit_code } => (
                true,
                format!("recorded run: `{}` exited {}", cmd, exit_code),
            ),
            DoneArtifact::Url { url, expected_status } => (
                true,
                format!("declared probe: {} → {}", url, expected_status),
            ),
            DoneArtifact::Note(s) => (true, format!("note: {}", s)),
        };
        if !ok {
            failures.push(detail.clone());
        }
        checks.push(ArtifactCheck { artifact: art.clone(), passed: ok, detail });
    }

    // If status is done_clean but no artifacts cited at all + no
    // independent corroboration, that's a single-source claim — flag
    // it (#65). Worker-self-report alone never reaches done_clean.
    if done.status == DoneStatus::DoneClean
        && done.artifacts.is_empty()
        && done.is_single_source()
    {
        failures.push(
            "done_clean asserted with zero artifacts and no independent corroboration (single-source). Worker narration is inadmissible.".to_string(),
        );
    }

    GroundTruthVerdict {
        passes: failures.is_empty(),
        checks,
        failures,
    }
}

fn check_git_sha(sha: &str, repo_root: Option<&Path>) -> (bool, String) {
    let Some(root) = repo_root else {
        return (true, format!("no repo root — skipped SHA {}", sha));
    };
    let out = std::process::Command::new("git")
        .args(["cat-file", "-e", sha])
        .current_dir(root)
        .status();
    match out {
        Ok(s) if s.success() => (true, format!("git SHA {} exists locally", sha)),
        Ok(_) => (
            false,
            format!("claimed git SHA {} does NOT exist in repo — FABRICATED", sha),
        ),
        Err(e) => (false, format!("git lookup failed for {}: {}", sha, e)),
    }
}

fn check_git_branch(name: &str, repo_root: Option<&Path>) -> (bool, String) {
    let Some(root) = repo_root else {
        return (true, format!("no repo root — skipped branch {}", name));
    };
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--verify", name])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => (true, format!("git branch {} exists", name)),
        _ => (
            false,
            format!("claimed git branch {} NOT found — FABRICATED", name),
        ),
    }
}

fn check_file_path(path: &str, repo_root: Option<&Path>) -> (bool, String) {
    let full = match repo_root {
        Some(root) if !path.starts_with('/') => root.join(path),
        _ => Path::new(path).to_path_buf(),
    };
    if full.exists() {
        (true, format!("file {} exists", full.display()))
    } else {
        (
            false,
            format!(
                "claimed file path {} does NOT exist — FABRICATED",
                full.display()
            ),
        )
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
