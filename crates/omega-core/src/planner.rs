//! Implementation planner — DAG-enforced sequential execution with verification gates.
//!
//! Fixes three bugs in the VPS planner:
//! 1. Steps are vague → workers interpret freely → skip ahead
//! 2. No DAG enforcement → workers jump to "interesting" steps
//! 3. No verify gate → "done" claimed without proof

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_TRACKER_BYTES: u64 = 8 * 1024 * 1024;
const TRACKER_LOCK: &str = ".tracker.lock";
static TRACKER_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct PlanSourceState {
    revision: u64,
    digest: String,
}

/// A single implementation step with full context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub phase: usize,
    pub title: String,
    pub description: String,
    pub files_to_touch: Vec<String>,
    pub done_criteria: String,
    pub verify_command: String,
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub wave: Option<Wave>,
    // The planner SKILL tells the emitting LLM that the default-able fields
    // accept `null`; #[serde(default)] alone covers only a MISSING key and
    // rejects an explicit `"attempt": null` for u8 — which used to brick the
    // whole plan with a misleading "no tracker" error. Accept null as 0.
    #[serde(default, deserialize_with = "null_as_default")]
    pub attempt: u8,
    /// Optional per-step worker-timeout override (minutes). Heavy steps
    /// (forensic audits, the /omg-acceptance browser sweep) routinely outlive
    /// the engine's default worker timeout; the executor honors this first.
    #[serde(default)]
    pub timeout_mins: Option<u64>,
    /// Why the previous attempt failed verification (Guardian evidence or the
    /// worker's own blocked/failed note). Engine-written on retry; render_brief
    /// delivers it to the retry worker so retries are never blind.
    #[serde(default)]
    pub last_feedback: Option<String>,
    pub status: StepStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Deserialize an explicit JSON `null` as the type's default (serde's
/// `#[serde(default)]` handles only an absent key, not `null`).
fn null_as_default<'de, D, T>(de: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(de)?.unwrap_or_default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Failed,
}

impl StepStatus {
    pub fn label(&self) -> &'static str {
        match self {
            StepStatus::Pending => "pending",
            StepStatus::InProgress => "in_progress",
            StepStatus::Done => "done",
            StepStatus::Blocked => "blocked",
            StepStatus::Failed => "failed",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            StepStatus::Pending => "[ ]",
            StepStatus::InProgress => "[~]",
            StepStatus::Done => "[+]",
            StepStatus::Blocked => "[!]",
            StepStatus::Failed => "[x]",
        }
    }
}

/// Optional scheduling tier. The DAG (`depends_on`) is the enforced order;
/// `Wave` is sugar that lets the driver hold terminal tiers (audit/deploy)
/// until all implementation work is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wave {
    Foundation,
    W1,
    W2,
    W3,
    Audit,
    Deploy,
}

impl Wave {
    /// Scheduling-tier ordinal. NOTE: `W1`/`W2`/`W3` are intentionally peers
    /// (all ordinal 1) — intra-implementation order is enforced by the DAG
    /// (`depends_on`), not by wave. Only terminal tiers (`Audit`=2, `Deploy`=3)
    /// use the ordinal, via `wave_open`, to wait for all lower tiers.
    pub fn ordinal(&self) -> u8 {
        match self {
            Wave::Foundation => 0,
            Wave::W1 => 1,
            Wave::W2 => 1,
            Wave::W3 => 1,
            Wave::Audit => 2,
            Wave::Deploy => 3,
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Wave::Audit | Wave::Deploy)
    }
}

/// A phase groups up to 25 related steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPhase {
    pub id: usize,
    pub name: String,
    pub goal: String,
    pub step_ids: Vec<String>,
}

/// The full plan tracker — persisted to .planner/tracker.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTracker {
    pub project: String,
    pub total_phases: usize,
    pub active_phase: usize,
    pub planner_version: String,
    pub generated_at: String,
    pub phases: Vec<PlanPhase>,
    pub steps: Vec<PlanStep>,
    /// Monotonic on-disk generation used by the save-time compare-and-swap.
    /// Kept private so callers cannot forge the generation they are expected
    /// to have observed through `load_strict`.
    #[serde(default, rename = "_omega_revision")]
    storage_revision: u64,
    /// Exact generation and byte digest observed by this in-memory instance.
    /// This is deliberately not serialized. Two processes loading generation
    /// N receive independent snapshots; after one publishes N+1, the other's
    /// save is rejected instead of silently winning last-writer-wins.
    #[serde(skip)]
    source_state: RefCell<Option<PlanSourceState>>,
}

impl PlanTracker {
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
            total_phases: 0,
            active_phase: 1,
            planner_version: "7.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            phases: Vec::new(),
            steps: Vec::new(),
            storage_revision: 0,
            source_state: RefCell::new(None),
        }
    }

    pub fn tracker_path(project_dir: &Path) -> PathBuf {
        project_dir.join(".planner").join("tracker.json")
    }

    /// Lenient loader for display surfaces (TUI): any failure is just "no plan".
    pub fn load(project_dir: &Path) -> Option<Self> {
        Self::load_strict(project_dir).ok().flatten()
    }

    /// Strict loader for the engine/CLI: distinguishes "no tracker file"
    /// (Ok(None)) from "tracker exists but is malformed" (Err carrying the
    /// serde detail). Swallowing parse errors into None made the engine report
    /// a corrupt tracker as "no .planner/tracker.json" — the operator was told
    /// the plan doesn't exist while it sat there with invisible schema drift.
    pub fn load_strict(project_dir: &Path) -> Result<Option<Self>> {
        Self::load_secure_document(project_dir)
    }

    /// Strict structural validation of a loaded plan. The engine REFUSES to run a
    /// plan that would silently mis-sequence or fake-complete — the exact failure
    /// modes that let a "build" finish with nothing actually working:
    ///   - **dangling `depends_on`** (a typo'd dep id → `deps_satisfied` is never
    ///     true → that step blocks FOREVER, silently);
    ///   - **duplicate `step_id`s** (mark_done/ready_steps act on the wrong step);
    ///   - **empty or trivial `verify_command`** (`true` / `:` / bare `echo` →
    ///     the Guardian's re-run passes unconditionally, so the step is marked Done
    ///     without any proof — the #1 cause of "done but broken");
    ///   - **empty or directory `files_to_touch`** (the ready-set disjointness
    ///     test and the executor's scope claim are both vacuous for an empty
    ///     list, so two such steps could run in parallel on the same files —
    ///     violating R-SCOPE — and a directory entry like `src/` is not a
    ///     claimable scope). STRICT-only: legitimately file-less steps exist
    ///     (a deploy that only runs `vercel --prod`, an audit sweep), so at
    ///     RUN time these demote to loud warnings instead of refusals;
    ///   - **dependency cycles** (no step in the cycle can ever become ready —
    ///     the run deadlocks; `plan-status` advertises this validate() as the
    ///     pre-flight gate that "proves no cycle", so the check lives HERE,
    ///     not only in plan-run).
    ///     Returns every issue at once (don't make the operator fix one at a time).
    ///
    /// Two strictness levels:
    ///   - `validate()` — authoring-time gate (plan-status): everything above
    ///     is a hard error, on every step.
    ///   - `validate_for_run()` — execution gate (plan-run): structural issues
    ///     (cycles, dangling/dup/self deps) stay hard for ALL steps; per-step
    ///     content issues are only enforced on steps that still have work to do
    ///     (Pending/InProgress) — a tracker mid-flight from an older schema
    ///     must remain RESUMABLE (its Done/Failed history is not future work) —
    ///     and the files_to_touch checks demote to warnings.
    pub fn validate(&self) -> Result<()> {
        self.validate_impl(true)
    }

    pub fn validate_for_run(&self) -> Result<()> {
        self.validate_impl(false)
    }

    fn validate_impl(&self, strict: bool) -> Result<()> {
        use std::collections::HashSet;
        let ids: HashSet<&str> = self.steps.iter().map(|s| s.step_id.as_str()).collect();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut errs: Vec<String> = Vec::new();
        if self.steps.is_empty() {
            bail!("plan has zero steps — refusing to run an empty plan");
        }
        for s in &self.steps {
            if !seen.insert(s.step_id.as_str()) {
                errs.push(format!("duplicate step_id `{}`", s.step_id));
            }
            for dep in &s.depends_on {
                if !ids.contains(dep.as_str()) {
                    errs.push(format!(
                        "step `{}` depends on unknown step `{}` — dangling dep, it would block forever",
                        s.step_id, dep
                    ));
                }
                if dep == &s.step_id {
                    errs.push(format!("step `{}` depends on itself", s.step_id));
                }
            }
            // Content checks: at run time, only steps with REMAINING work are
            // gated — terminal steps are history, and refusing to resume a
            // half-built project over them would brick it.
            let content_gated =
                strict || matches!(s.status, StepStatus::Pending | StepStatus::InProgress);
            if !content_gated {
                continue;
            }
            let v = s.verify_command.trim();
            let trivial = v.is_empty()
                || v == "true"
                || v == ":"
                || v.split_whitespace().next() == Some("echo");
            if trivial {
                errs.push(format!(
                    "step `{}` has a trivial verify_command `{}` — it fake-passes; give a real check (build / test / typecheck / `jq -e .score>=N`)",
                    s.step_id, v
                ));
            }
            if s.files_to_touch.is_empty() {
                let msg = format!(
                    "step `{}` has empty files_to_touch — list ≥1 exact file path (an empty list skips the scope claim and parallel disjointness entirely)",
                    s.step_id
                );
                if strict {
                    errs.push(msg);
                } else {
                    tracing::warn!("plan validation: {msg}");
                }
            }
            for f in &s.files_to_touch {
                if f.trim().is_empty() || f.ends_with('/') {
                    let msg = format!(
                        "step `{}` files_to_touch entry `{}` is not an exact file path — directories (`src/`) are not a claimable scope",
                        s.step_id, f
                    );
                    if strict {
                        errs.push(msg);
                    } else {
                        tracing::warn!("plan validation: {msg}");
                    }
                }
            }
        }
        // Cycle detection: one fresh DFS per start node so the error can name a
        // step actually inside the cycle. Report once — the named step is the
        // thread to pull; listing every member adds noise, not signal.
        for s in &self.steps {
            let mut visited = HashSet::new();
            let mut in_stack = HashSet::new();
            if self.has_cycle(&s.step_id, &mut visited, &mut in_stack) {
                errs.push(format!("dependency cycle involving step `{}`", s.step_id));
                break;
            }
        }
        if !errs.is_empty() {
            bail!(
                "plan validation failed ({} issue(s)) — refusing to run:\n  - {}",
                errs.len(),
                errs.join("\n  - ")
            );
        }
        Ok(())
    }

    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let dir = project_dir.join(".planner");
        crate::scope::ensure_private_state_dir(&dir)
            .with_context(|| format!("preparing private planner state at {}", dir.display()))?;
        let _lock = crate::scope::lock_private_state_file(&dir, TRACKER_LOCK)?;
        self.save_locked(project_dir)?;
        Ok(())
    }

    fn save_locked(&self, project_dir: &Path) -> Result<()> {
        let path = Self::tracker_path(project_dir);
        let current = Self::load_secure_document(project_dir)?;
        let expected = self.source_state.borrow().clone();

        match (expected.as_ref(), current.as_ref()) {
            (Some(expected), Some(observed))
                if observed.storage_revision == expected.revision
                    && observed
                        .source_state
                        .borrow()
                        .as_ref()
                        .is_some_and(|source| {
                            source.digest.as_str() == expected.digest.as_str()
                        }) => {}
            (None, None) if self.storage_revision == 0 => {}
            (Some(_), None) => {
                bail!("planner tracker disappeared before compare-and-swap; reload before saving")
            }
            (None, Some(_)) => {
                bail!("planner tracker already exists; reload before replacing it")
            }
            (None, None) => {
                bail!(
                    "detached planner tracker carries revision {}; reload before saving",
                    self.storage_revision
                )
            }
            _ => bail!("stale planner tracker write refused: generation or content digest changed"),
        }

        let current_revision = current
            .as_ref()
            .map(|tracker| tracker.storage_revision)
            .unwrap_or(0);
        let next_revision = current_revision
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("planner tracker revision overflow"))?;
        let mut published = self.clone();
        published.storage_revision = next_revision;
        published.source_state.replace(None);
        let bytes = serde_json::to_vec_pretty(&published)?;
        if bytes.len() as u64 > MAX_TRACKER_BYTES {
            bail!(
                "planner tracker is {} bytes; maximum is {} bytes",
                bytes.len(),
                MAX_TRACKER_BYTES
            );
        }
        let expected_digest = tracker_digest(&bytes);
        atomic_write_tracker(&path, &bytes)?;

        let observed = Self::load_secure_document(project_dir)?
            .ok_or_else(|| anyhow::anyhow!("planner tracker vanished after publish"))?;
        let observed_source = observed.source_state.borrow().clone().ok_or_else(|| {
            anyhow::anyhow!("planner tracker lost its source generation after publish")
        })?;
        if observed.storage_revision != next_revision || observed_source.digest != expected_digest {
            bail!("planner tracker changed while being published");
        }
        self.source_state.replace(Some(PlanSourceState {
            revision: next_revision,
            digest: expected_digest,
        }));
        Ok(())
    }

    fn load_secure_document(project_dir: &Path) -> Result<Option<Self>> {
        let dir = project_dir.join(".planner");
        match std::fs::symlink_metadata(&dir) {
            Ok(_) => crate::scope::ensure_private_state_dir(&dir).with_context(|| {
                format!("validating private planner state at {}", dir.display())
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(error).with_context(|| format!("inspecting {}", dir.display()))
            }
        }

        let path = Self::tracker_path(project_dir);
        let Some(bytes) = read_tracker_bytes(&path)? else {
            return Ok(None);
        };
        let digest = tracker_digest(&bytes);
        let tracker: Self = serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "{} exists but failed to parse — fix the JSON/schema",
                path.display()
            )
        })?;
        tracker.source_state.replace(Some(PlanSourceState {
            revision: tracker.storage_revision,
            digest,
        }));
        Ok(Some(tracker))
    }

    pub fn add_phase(&mut self, name: &str, goal: &str) -> usize {
        let id = self.phases.len() + 1;
        self.phases.push(PlanPhase {
            id,
            name: name.to_string(),
            goal: goal.to_string(),
            step_ids: Vec::new(),
        });
        self.total_phases = self.phases.len();
        id
    }

    pub fn add_step(&mut self, phase_id: usize, step: PlanStep) -> Result<()> {
        // Validate phase exists
        let phase = self
            .phases
            .iter_mut()
            .find(|p| p.id == phase_id)
            .context("phase not found")?;

        if phase.step_ids.len() >= 25 {
            bail!("phase {} already has 25 steps (max per phase)", phase.name);
        }

        // Validate dependencies exist
        for dep in &step.depends_on {
            if !self.steps.iter().any(|s| s.step_id == *dep) && !step.step_id.eq(dep) {
                bail!("step {} depends on unknown step {}", step.step_id, dep);
            }
        }

        // Validate no duplicate step IDs
        if self.steps.iter().any(|s| s.step_id == step.step_id) {
            bail!("duplicate step_id: {}", step.step_id);
        }

        phase.step_ids.push(step.step_id.clone());
        self.steps.push(step);
        Ok(())
    }

    /// Get the step by ID.
    pub fn get_step(&self, step_id: &str) -> Option<&PlanStep> {
        self.steps.iter().find(|s| s.step_id == step_id)
    }

    /// Get a mutable step by ID.
    pub fn get_step_mut(&mut self, step_id: &str) -> Option<&mut PlanStep> {
        self.steps.iter_mut().find(|s| s.step_id == step_id)
    }

    /// Returns the next unblocked step whose dependencies are ALL done.
    /// This is the ONLY way to get the next step — enforces the DAG.
    pub fn next_step(&self) -> Option<&PlanStep> {
        self.steps
            .iter()
            .find(|step| step.status == StepStatus::Pending && self.deps_satisfied(&step.step_id))
    }

    /// All pending steps whose deps are Done, whose wave is open, and whose
    /// files_to_touch are pairwise disjoint within the returned set — capped.
    /// This is the ONLY selector the driver uses. Parallel-safe.
    pub fn ready_steps(&self, cap: usize) -> Vec<&PlanStep> {
        let mut out: Vec<&PlanStep> = Vec::new();
        let mut claimed: Vec<&str> = Vec::new();
        for s in &self.steps {
            if out.len() >= cap {
                break;
            }
            if s.status != StepStatus::Pending {
                continue;
            }
            if !self.deps_satisfied(&s.step_id) {
                continue;
            }
            if !self.wave_open(s) {
                continue;
            }
            let disjoint = s
                .files_to_touch
                .iter()
                .all(|f| !claimed.contains(&f.as_str()));
            if !disjoint {
                continue;
            }
            for f in &s.files_to_touch {
                claimed.push(f.as_str());
            }
            out.push(s);
        }
        out
    }

    /// A terminal-tier step (audit/deploy) is withheld until every step of a
    /// strictly-lower wave ordinal is Done. Non-waved steps are gated only by
    /// the DAG (always "open" here).
    fn wave_open(&self, step: &PlanStep) -> bool {
        let Some(w) = step.wave else {
            return true;
        };
        if !w.is_terminal() {
            return true;
        }
        let my = w.ordinal();
        self.steps.iter().all(|other| {
            let other_ord = other.wave.map(|x| x.ordinal()).unwrap_or(0);
            other_ord >= my || other.status == StepStatus::Done
        })
    }

    /// Check if all dependencies for a step are satisfied (status == Done).
    pub fn deps_satisfied(&self, step_id: &str) -> bool {
        let Some(step) = self.get_step(step_id) else {
            return false;
        };
        step.depends_on.iter().all(|dep_id| {
            self.get_step(dep_id)
                .map(|d| d.status == StepStatus::Done)
                .unwrap_or(false)
        })
    }

    /// Mark a step as in-progress. Fails if deps aren't met.
    pub fn start_step(&mut self, step_id: &str) -> Result<()> {
        if !self.deps_satisfied(step_id) {
            bail!("cannot start {} — dependencies not satisfied", step_id);
        }
        let step = self.get_step_mut(step_id).context("step not found")?;
        if step.status != StepStatus::Pending {
            bail!(
                "step {} is not pending (status: {})",
                step_id,
                step.status.label()
            );
        }
        step.status = StepStatus::InProgress;
        step.started_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Mark a step as done. In a real execution, the verify_command would be
    /// run first — the caller is responsible for checking that.
    pub fn mark_done(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        if step.status != StepStatus::InProgress {
            bail!(
                "step {} is not in_progress (status: {})",
                step_id,
                step.status.label()
            );
        }
        step.status = StepStatus::Done;
        step.completed_at = Some(chrono::Utc::now().to_rfc3339());

        // Auto-advance active phase if all steps in current phase are done
        self.maybe_advance_phase();
        Ok(())
    }

    /// Mark a step as failed. Only an in-progress step can fail (mirrors
    /// `mark_done`'s guard) — a step cannot fail without having been attempted.
    pub fn mark_failed(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        if step.status != StepStatus::InProgress {
            bail!(
                "step {} is not in_progress (status: {})",
                step_id,
                step.status.label()
            );
        }
        step.status = StepStatus::Failed;
        step.completed_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Increment the retry counter for a step.
    pub fn bump_attempt(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        step.attempt = step.attempt.saturating_add(1);
        Ok(())
    }

    /// Return an in-progress step to pending so it can be re-dispatched.
    /// Only an in-progress step may be reset — guards against resetting a
    /// Done/Failed step (which would re-run work without an audit trail).
    pub fn reset_to_pending(&mut self, step_id: &str) -> Result<()> {
        let step = self.get_step_mut(step_id).context("step not found")?;
        if step.status != StepStatus::InProgress {
            bail!(
                "step {} is not in_progress (status: {}), cannot reset to pending",
                step_id,
                step.status.label()
            );
        }
        step.status = StepStatus::Pending;
        step.started_at = None;
        Ok(())
    }

    fn maybe_advance_phase(&mut self) {
        if let Some(phase) = self.phases.iter().find(|p| p.id == self.active_phase) {
            let all_done = phase.step_ids.iter().all(|sid| {
                self.get_step(sid)
                    .map(|s| s.status == StepStatus::Done)
                    .unwrap_or(false)
            });
            if all_done && self.active_phase < self.total_phases {
                self.active_phase += 1;
            }
        }
    }

    /// Progress summary.
    pub fn status(&self) -> PlanStatus {
        let total = self.steps.len();
        let done = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Done)
            .count();
        let in_progress = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::InProgress)
            .count();
        let failed = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count();
        let blocked = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending && !self.deps_satisfied(&s.step_id))
            .count();
        let ready = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending && self.deps_satisfied(&s.step_id))
            .count();

        PlanStatus {
            total,
            done,
            in_progress,
            failed,
            blocked,
            ready,
            active_phase: self.active_phase,
            total_phases: self.total_phases,
        }
    }

    /// Write a detailed step file to .planner/STEP-XXX-title.md
    pub fn write_step_file(&self, step_id: &str, project_dir: &Path) -> Result<()> {
        let step = self.get_step(step_id).context("step not found")?;
        let dir = project_dir.join(".planner");
        std::fs::create_dir_all(&dir)?;

        let slug: String = step
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        let slug = slug.trim_matches('-');
        let filename = format!("{}-{}.md", step.step_id, slug);

        let deps_str = if step.depends_on.is_empty() {
            "  (none)".to_string()
        } else {
            step.depends_on
                .iter()
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let files_str = step
            .files_to_touch
            .iter()
            .map(|f| format!("  - {}", f))
            .collect::<Vec<_>>()
            .join("\n");

        let content = format!(
            r#"# {step_id}: {title}

## Phase {phase}

## Description

{description}

## Files to Touch

{files}

## Dependencies

{deps}

## Done Criteria

{criteria}

## Verify Command

```bash
{verify}
```

## Status: {status}
"#,
            step_id = step.step_id,
            title = step.title,
            phase = step.phase,
            description = step.description,
            files = files_str,
            deps = deps_str,
            criteria = step.done_criteria,
            verify = step.verify_command,
            status = step.status.label(),
        );

        std::fs::write(dir.join(filename), content)?;
        Ok(())
    }

    /// Detect cycle in the dependency DAG. Returns true if the graph is acyclic.
    pub fn is_acyclic(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        for step in &self.steps {
            if !visited.contains(&step.step_id)
                && self.has_cycle(&step.step_id, &mut visited, &mut in_stack)
            {
                return false;
            }
        }
        true
    }

    fn has_cycle(
        &self,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());

        if let Some(step) = self.get_step(node) {
            for dep in &step.depends_on {
                if !visited.contains(dep.as_str()) {
                    if self.has_cycle(dep, visited, in_stack) {
                        return true;
                    }
                } else if in_stack.contains(dep.as_str()) {
                    return true;
                }
            }
        }

        in_stack.remove(node);
        false
    }
}

fn tracker_digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Read a tracker through a no-follow descriptor and cap allocation before
/// parsing. The path and the opened descriptor must keep the same inode for
/// the whole read; hard links and foreign-owned files are never authority.
fn read_tracker_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
    let before = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("inspecting {}", path.display())),
    };
    if !before.file_type().is_file() {
        bail!(
            "refusing non-regular planner tracker {} (symlinks are not trusted)",
            path.display()
        );
    }
    if before.len() > MAX_TRACKER_BYTES {
        bail!(
            "planner tracker {} is {} bytes; maximum is {} bytes",
            path.display(),
            before.len(),
            MAX_TRACKER_BYTES
        );
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(planner_no_follow_flag());
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "opening planner tracker {} without symlink following",
            path.display()
        )
    })?;
    let opened = file
        .metadata()
        .with_context(|| format!("inspecting opened planner tracker {}", path.display()))?;
    validate_tracker_file_identity(path, &before, &opened)?;
    if opened.len() > MAX_TRACKER_BYTES {
        bail!(
            "planner tracker {} is {} bytes; maximum is {} bytes",
            path.display(),
            opened.len(),
            MAX_TRACKER_BYTES
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if opened.mode() & 0o777 != 0o600 {
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| {
                    format!(
                        "setting owner-only mode on planner tracker {}",
                        path.display()
                    )
                })?;
        }
    }

    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_TRACKER_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("reading planner tracker {}", path.display()))?;
    if bytes.len() as u64 > MAX_TRACKER_BYTES {
        bail!(
            "planner tracker {} exceeded the {} byte read limit",
            path.display(),
            MAX_TRACKER_BYTES
        );
    }

    let after = std::fs::symlink_metadata(path)
        .with_context(|| format!("re-checking planner tracker {}", path.display()))?;
    let final_opened = file
        .metadata()
        .with_context(|| format!("re-checking opened planner tracker {}", path.display()))?;
    validate_tracker_file_identity(path, &after, &final_opened)?;
    if opened.len() != final_opened.len() || bytes.len() as u64 != final_opened.len() {
        bail!(
            "planner tracker {} changed while being read",
            path.display()
        );
    }
    Ok(Some(bytes))
}

fn validate_tracker_file_identity(
    path: &Path,
    path_metadata: &std::fs::Metadata,
    opened_metadata: &std::fs::Metadata,
) -> Result<()> {
    if !path_metadata.file_type().is_file() || !opened_metadata.file_type().is_file() {
        bail!("planner tracker {} is not a regular file", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
        {
            bail!(
                "planner tracker {} changed identity while being opened",
                path.display()
            );
        }
        if opened_metadata.nlink() != 1 {
            bail!(
                "planner tracker {} has {} hard links; expected exactly one",
                path.display(),
                opened_metadata.nlink()
            );
        }
        let current_uid = planner_effective_uid();
        if opened_metadata.uid() != current_uid {
            bail!(
                "planner tracker {} is owned by uid {}, current uid is {}",
                path.display(),
                opened_metadata.uid(),
                current_uid
            );
        }
    }
    Ok(())
}

/// Publish beside the destination, sync the staged file, atomically rename it,
/// then sync the already-validated parent directory. A stale staged file from
/// a killed writer is harmless: every attempt uses `create_new` and a fresh
/// nonce, while the last fully renamed tracker remains the recovery point.
fn atomic_write_tracker(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    crate::scope::ensure_private_state_dir(parent)?;
    let directory = std::fs::File::open(parent)
        .with_context(|| format!("opening planner state directory {}", parent.display()))?;
    validate_tracker_parent_identity(parent, &directory)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tracker.json");
    let (staged, mut file) = (0..128)
        .find_map(|_| {
            let serial = TRACKER_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let candidate = parent.join(format!(
                ".{filename}.omega-tmp-{}-{timestamp}-{serial}",
                std::process::id()
            ));
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&candidate) {
                Ok(file) => Some(Ok((candidate, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error).with_context(|| {
                    format!("creating staged planner tracker {}", candidate.display())
                })),
            }
        })
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("could not allocate a unique staged planner tracker"))?;

    let result = (|| -> Result<()> {
        file.write_all(bytes)
            .with_context(|| format!("writing staged planner tracker {}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| {
                    format!(
                        "setting owner-only mode on staged tracker {}",
                        staged.display()
                    )
                })?;
        }
        file.sync_all()
            .with_context(|| format!("syncing staged planner tracker {}", staged.display()))?;
        drop(file);
        std::fs::rename(&staged, path).with_context(|| {
            format!(
                "atomically replacing planner tracker {} with {}",
                path.display(),
                staged.display()
            )
        })?;
        validate_tracker_parent_identity(parent, &directory)?;
        directory
            .sync_all()
            .with_context(|| format!("syncing planner state directory {}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

fn validate_tracker_parent_identity(path: &Path, opened: &std::fs::File) -> Result<()> {
    let path_metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting planner state directory {}", path.display()))?;
    let opened_metadata = opened.metadata().with_context(|| {
        format!(
            "inspecting opened planner state directory {}",
            path.display()
        )
    })?;
    if !path_metadata.file_type().is_dir()
        || path_metadata.file_type().is_symlink()
        || !opened_metadata.file_type().is_dir()
    {
        bail!(
            "planner state parent {} is not a real directory",
            path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
        {
            bail!(
                "planner state parent {} changed identity during publish",
                path.display()
            );
        }
        let current_uid = planner_effective_uid();
        if opened_metadata.uid() != current_uid {
            bail!(
                "planner state parent {} is owned by uid {}, current uid is {}",
                path.display(),
                opened_metadata.uid(),
                current_uid
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn planner_effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: geteuid has no arguments or preconditions and returns the
    // effective uid of this process.
    unsafe { geteuid() }
}

#[cfg(all(unix, any(target_os = "linux", target_os = "android")))]
fn planner_no_follow_flag() -> i32 {
    0o400000
}

#[cfg(all(
    unix,
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )
))]
fn planner_no_follow_flag() -> i32 {
    0x100
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))
))]
fn planner_no_follow_flag() -> i32 {
    0
}

#[derive(Debug, Clone)]
pub struct PlanStatus {
    pub total: usize,
    pub done: usize,
    pub in_progress: usize,
    pub failed: usize,
    pub blocked: usize,
    pub ready: usize,
    pub active_phase: usize,
    pub total_phases: usize,
}

impl PlanStatus {
    pub fn progress_pct(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.done as f32 / self.total as f32 * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.done == self.total && self.total > 0
    }
}

/// Helper to create a step with a builder pattern.
pub fn step(id: &str, phase: usize) -> PlanStepBuilder {
    PlanStepBuilder {
        step_id: id.to_string(),
        phase,
        title: String::new(),
        description: String::new(),
        files_to_touch: Vec::new(),
        done_criteria: String::new(),
        verify_command: String::new(),
        depends_on: Vec::new(),
        wave: None,
    }
}

pub struct PlanStepBuilder {
    step_id: String,
    phase: usize,
    title: String,
    description: String,
    files_to_touch: Vec<String>,
    done_criteria: String,
    verify_command: String,
    depends_on: Vec<String>,
    wave: Option<Wave>,
}

impl PlanStepBuilder {
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.to_string();
        self
    }
    pub fn description(mut self, d: &str) -> Self {
        self.description = d.to_string();
        self
    }
    pub fn files(mut self, f: &[&str]) -> Self {
        self.files_to_touch = f.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn criteria(mut self, c: &str) -> Self {
        self.done_criteria = c.to_string();
        self
    }
    pub fn verify(mut self, v: &str) -> Self {
        self.verify_command = v.to_string();
        self
    }
    pub fn depends(mut self, deps: &[&str]) -> Self {
        self.depends_on = deps.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn wave(mut self, w: Wave) -> Self {
        self.wave = Some(w);
        self
    }
    pub fn build(self) -> PlanStep {
        PlanStep {
            step_id: self.step_id,
            phase: self.phase,
            title: self.title,
            description: self.description,
            files_to_touch: self.files_to_touch,
            done_criteria: self.done_criteria,
            verify_command: self.verify_command,
            depends_on: self.depends_on,
            wave: self.wave,
            attempt: 0,
            timeout_mins: None,
            last_feedback: None,
            status: StepStatus::Pending,
            started_at: None,
            completed_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tracker() -> PlanTracker {
        let mut tracker = PlanTracker::new("TestProject");
        let phase_id = tracker.add_phase("Foundation", "Schema + auth");

        tracker
            .add_step(
                phase_id,
                step("STEP-001", phase_id)
                    .title("Create schema")
                    .description("Define the database schema with users table")
                    .files(&["convex/schema.ts"])
                    .criteria("npx convex dev starts without schema errors")
                    // Exit-code only — piping through `grep -v` swallows the
                    // command's exit code AND fake-passes (any non-matching
                    // line satisfies it even when an ERROR line is present).
                    .verify("npx convex dev --once")
                    .build(),
            )
            .unwrap();

        tracker
            .add_step(
                phase_id,
                step("STEP-002", phase_id)
                    .title("Auth middleware")
                    .description("Create Clerk middleware at src/middleware.ts")
                    .files(&["src/middleware.ts"])
                    .criteria("npm run build exits 0")
                    .verify("npm run build")
                    .depends(&["STEP-001"])
                    .build(),
            )
            .unwrap();

        tracker
            .add_step(
                phase_id,
                step("STEP-003", phase_id)
                    .title("Dashboard layout")
                    .description("Create the main dashboard layout")
                    .files(&["src/app/dashboard/layout.tsx"])
                    .criteria("dashboard renders without errors")
                    .verify("npm run build")
                    .depends(&["STEP-002"])
                    .build(),
            )
            .unwrap();

        tracker
    }

    #[test]
    fn validate_catches_malformed_plans() {
        // A well-formed plan passes.
        assert!(sample_tracker().validate().is_ok());

        // Trivial verify_command (a loaded JSON can carry this; add_step can't catch
        // a post-hoc mutation). It must be refused so the step can't fake-pass.
        let mut t = sample_tracker();
        t.steps[0].verify_command = "true".to_string();
        assert!(
            t.validate().is_err(),
            "trivial `true` verify must be rejected"
        );
        t.steps[0].verify_command = "echo done".to_string();
        assert!(t.validate().is_err(), "echo-only verify must be rejected");
        t.steps[0].verify_command = String::new();
        assert!(t.validate().is_err(), "empty verify must be rejected");

        // Dangling dependency → would block forever; refuse it.
        let mut t = sample_tracker();
        t.steps[1].depends_on = vec!["STEP-999".to_string()];
        assert!(t.validate().is_err(), "dangling dep must be rejected");

        // Duplicate step_id.
        let mut t = sample_tracker();
        let dup = t.steps[0].clone();
        t.steps.push(dup);
        assert!(t.validate().is_err(), "duplicate step_id must be rejected");

        // Dependency cycle (add_step can't create one; a loaded JSON can).
        // plan-status advertises validate() as the gate that proves no cycle.
        let mut t = sample_tracker();
        t.steps[0].depends_on = vec!["STEP-002".to_string()];
        let err = t.validate().unwrap_err().to_string();
        assert!(err.contains("cycle"), "cyclic plan must be rejected: {err}");

        // Empty files_to_touch → scope claim + parallel disjointness are
        // vacuous (R-SCOPE violation); the SKILL promises this is rejected.
        let mut t = sample_tracker();
        t.steps[0].files_to_touch = vec![];
        assert!(
            t.validate().is_err(),
            "empty files_to_touch must be rejected"
        );

        // Directory entries are not a claimable scope.
        let mut t = sample_tracker();
        t.steps[0].files_to_touch = vec!["src/".to_string()];
        assert!(
            t.validate().is_err(),
            "directory files_to_touch must be rejected"
        );
    }

    #[test]
    fn attempt_null_deserializes_as_zero() {
        // SKILL.md tells the planner LLM the default-able fields accept null;
        // #[serde(default)] alone rejected an explicit "attempt": null.
        let mut t = sample_tracker();
        t.steps[0].attempt = 7; // ensure the round-trip really reads the null
        let mut v: serde_json::Value = serde_json::to_value(&t).unwrap();
        v["steps"][0]["attempt"] = serde_json::Value::Null;
        let loaded: PlanTracker = serde_json::from_value(v).unwrap();
        assert_eq!(loaded.steps[0].attempt, 0);
    }

    #[test]
    fn load_strict_surfaces_parse_error() {
        let tmp = tempfile::tempdir().unwrap();
        // Absent file → Ok(None), NOT an error.
        assert!(PlanTracker::load_strict(tmp.path()).unwrap().is_none());
        // Malformed file → Err naming the path, never "no tracker".
        std::fs::create_dir_all(tmp.path().join(".planner")).unwrap();
        std::fs::write(tmp.path().join(".planner/tracker.json"), "{not json").unwrap();
        let err = PlanTracker::load_strict(tmp.path()).unwrap_err();
        assert!(
            format!("{err:#}").contains("failed to parse"),
            "got: {err:#}"
        );
        // The lenient loader still degrades to None for display surfaces.
        assert!(PlanTracker::load(tmp.path()).is_none());
    }

    #[test]
    fn corrupt_live_tracker_is_never_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let planner_dir = tmp.path().join(".planner");
        std::fs::create_dir_all(&planner_dir).unwrap();
        let path = PlanTracker::tracker_path(tmp.path());
        std::fs::write(&path, b"{partial").unwrap();
        let before = std::fs::read(&path).unwrap();

        let replacement = sample_tracker();
        let error = replacement.save(tmp.path()).unwrap_err();
        assert!(
            format!("{error:#}").contains("failed to parse"),
            "unexpected error: {error:#}"
        );
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    #[test]
    fn repeated_saves_advance_revision_without_reloading() {
        let tmp = tempfile::tempdir().unwrap();
        let mut tracker = sample_tracker();
        tracker.save(tmp.path()).unwrap();
        tracker.start_step("STEP-001").unwrap();
        tracker.save(tmp.path()).unwrap();

        let loaded = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert_eq!(loaded.storage_revision, 2);
        assert_eq!(
            loaded.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress
        );
        assert_eq!(
            tracker
                .source_state
                .borrow()
                .as_ref()
                .map(|source| source.revision),
            Some(2)
        );
    }

    #[test]
    fn stale_loaded_tracker_fails_compare_and_swap() {
        let tmp = tempfile::tempdir().unwrap();
        sample_tracker().save(tmp.path()).unwrap();
        let mut first = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        let mut stale = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        first.project = "first-writer".to_string();
        stale.project = "stale-writer".to_string();

        first.save(tmp.path()).unwrap();
        let error = stale.save(tmp.path()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("stale planner tracker write refused"),
            "unexpected error: {error:#}"
        );
        let current = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert_eq!(current.project, "first-writer");
        assert_eq!(current.storage_revision, 2);
    }

    #[test]
    fn concurrent_writers_commit_exactly_one_generation() {
        let tmp = tempfile::tempdir().unwrap();
        sample_tracker().save(tmp.path()).unwrap();
        let mut first = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        let mut second = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        first.project = "concurrent-a".to_string();
        second.project = "concurrent-b".to_string();

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let first_dir = tmp.path().to_path_buf();
        let first_barrier = barrier.clone();
        let first_writer = std::thread::spawn(move || {
            first_barrier.wait();
            first.save(&first_dir).map_err(|error| format!("{error:#}"))
        });
        let second_dir = tmp.path().to_path_buf();
        let second_barrier = barrier.clone();
        let second_writer = std::thread::spawn(move || {
            second_barrier.wait();
            second
                .save(&second_dir)
                .map_err(|error| format!("{error:#}"))
        });
        barrier.wait();

        let first_result = first_writer.join().unwrap();
        let second_result = second_writer.join().unwrap();
        assert_ne!(first_result.is_ok(), second_result.is_ok());
        let rejection = first_result.err().or_else(|| second_result.err()).unwrap();
        assert!(rejection.contains("stale planner tracker write refused"));
        let current = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert!(matches!(
            current.project.as_str(),
            "concurrent-a" | "concurrent-b"
        ));
        assert_eq!(current.storage_revision, 2);
    }

    #[test]
    fn save_waits_for_the_interprocess_tracker_lock() {
        let tmp = tempfile::tempdir().unwrap();
        sample_tracker().save(tmp.path()).unwrap();
        let planner_dir = tmp.path().join(".planner");
        let held = crate::scope::lock_private_state_file(&planner_dir, TRACKER_LOCK).unwrap();
        let mut contender = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        contender.project = "after-lock".to_string();
        let project_dir = tmp.path().to_path_buf();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let writer = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            done_tx.send(contender.save(&project_dir)).unwrap();
        });
        started_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(
            done_rx
                .recv_timeout(std::time::Duration::from_millis(150))
                .is_err(),
            "save completed while the tracker lock was still held"
        );
        drop(held);
        done_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
            .unwrap();
        writer.join().unwrap();
        assert_eq!(
            PlanTracker::load_strict(tmp.path())
                .unwrap()
                .unwrap()
                .project,
            "after-lock"
        );
    }

    #[test]
    fn interrupted_staged_write_does_not_hide_the_last_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let tracker = sample_tracker();
        tracker.save(tmp.path()).unwrap();
        let staged = tmp
            .path()
            .join(".planner/.tracker.json.omega-tmp-interrupted");
        std::fs::write(&staged, b"{half-written").unwrap();

        let mut resumed = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert_eq!(resumed.project, "TestProject");
        resumed.project = "resumed".to_string();
        resumed.save(tmp.path()).unwrap();
        let current = PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert_eq!(current.project, "resumed");
        assert_eq!(current.storage_revision, 2);
        assert!(
            staged.exists(),
            "uncommitted staging data is ignored, not trusted"
        );
    }

    #[test]
    fn oversized_tracker_is_rejected_before_allocation_or_parse() {
        let tmp = tempfile::tempdir().unwrap();
        let planner_dir = tmp.path().join(".planner");
        std::fs::create_dir_all(&planner_dir).unwrap();
        let file = std::fs::File::create(PlanTracker::tracker_path(tmp.path())).unwrap();
        file.set_len(MAX_TRACKER_BYTES + 1).unwrap();
        let error = PlanTracker::load_strict(tmp.path()).unwrap_err();
        assert!(
            error.to_string().contains("maximum is"),
            "unexpected error: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tracker_rejects_symlinks_hardlinks_and_non_regular_paths() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();

        let symlink_project = root.path().join("symlink-project");
        std::fs::create_dir_all(symlink_project.join(".planner")).unwrap();
        let sentinel = root.path().join("sentinel");
        std::fs::write(&sentinel, b"sentinel").unwrap();
        symlink(&sentinel, PlanTracker::tracker_path(&symlink_project)).unwrap();
        assert!(PlanTracker::load_strict(&symlink_project).is_err());
        assert!(sample_tracker().save(&symlink_project).is_err());
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"sentinel");

        let hardlink_project = root.path().join("hardlink-project");
        std::fs::create_dir_all(hardlink_project.join(".planner")).unwrap();
        let hard_target = root.path().join("hard-target");
        std::fs::write(
            &hard_target,
            serde_json::to_vec_pretty(&sample_tracker()).unwrap(),
        )
        .unwrap();
        std::fs::hard_link(&hard_target, PlanTracker::tracker_path(&hardlink_project)).unwrap();
        assert!(PlanTracker::load_strict(&hardlink_project).is_err());
        assert!(sample_tracker().save(&hardlink_project).is_err());

        let directory_project = root.path().join("directory-project");
        std::fs::create_dir_all(PlanTracker::tracker_path(&directory_project)).unwrap();
        assert!(PlanTracker::load_strict(&directory_project).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn planner_parent_and_tracker_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let planner_dir = tmp.path().join(".planner");
        std::fs::create_dir_all(&planner_dir).unwrap();
        std::fs::set_permissions(&planner_dir, std::fs::Permissions::from_mode(0o777)).unwrap();
        let tracker_path = PlanTracker::tracker_path(tmp.path());
        std::fs::write(
            &tracker_path,
            serde_json::to_vec_pretty(&sample_tracker()).unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&tracker_path, std::fs::Permissions::from_mode(0o666)).unwrap();

        PlanTracker::load_strict(tmp.path()).unwrap().unwrap();
        assert_eq!(
            std::fs::metadata(&planner_dir)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(&tracker_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn dag_enforcement() {
        let tracker = sample_tracker();

        // Only STEP-001 should be available (no deps)
        let next = tracker.next_step().unwrap();
        assert_eq!(next.step_id, "STEP-001");

        // STEP-002 deps not satisfied
        assert!(!tracker.deps_satisfied("STEP-002"));
    }

    #[test]
    fn sequential_execution() {
        let mut tracker = sample_tracker();

        // Start and complete STEP-001
        tracker.start_step("STEP-001").unwrap();
        assert!(tracker.start_step("STEP-002").is_err()); // deps not met
        tracker.mark_done("STEP-001").unwrap();

        // Now STEP-002 is available
        let next = tracker.next_step().unwrap();
        assert_eq!(next.step_id, "STEP-002");

        tracker.start_step("STEP-002").unwrap();
        tracker.mark_done("STEP-002").unwrap();

        let next = tracker.next_step().unwrap();
        assert_eq!(next.step_id, "STEP-003");
    }

    #[test]
    fn cannot_skip_steps() {
        let mut tracker = sample_tracker();

        // Trying to start STEP-003 directly should fail
        assert!(tracker.start_step("STEP-003").is_err());
    }

    #[test]
    fn progress_tracking() {
        let mut tracker = sample_tracker();

        let status = tracker.status();
        assert_eq!(status.total, 3);
        assert_eq!(status.done, 0);
        assert_eq!(status.ready, 1); // only STEP-001

        tracker.start_step("STEP-001").unwrap();
        tracker.mark_done("STEP-001").unwrap();

        let status = tracker.status();
        assert_eq!(status.done, 1);
        assert!((status.progress_pct() - 33.3).abs() < 1.0);
    }

    #[test]
    fn max_25_per_phase() {
        let mut tracker = PlanTracker::new("Big");
        let pid = tracker.add_phase("Huge", "Too many steps");
        for i in 1..=25 {
            tracker
                .add_step(
                    pid,
                    step(&format!("S-{:03}", i), pid)
                        .title(&format!("Step {}", i))
                        .criteria("ok")
                        .verify("true")
                        .build(),
                )
                .unwrap();
        }
        // 26th should fail
        let result = tracker.add_step(
            pid,
            step("S-026", pid)
                .title("Too many")
                .criteria("ok")
                .verify("true")
                .build(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn acyclic_check() {
        let tracker = sample_tracker();
        assert!(tracker.is_acyclic());
    }

    #[test]
    fn duplicate_step_rejected() {
        let mut tracker = PlanTracker::new("Dup");
        let pid = tracker.add_phase("P1", "g");
        tracker
            .add_step(
                pid,
                step("S-001", pid)
                    .title("A")
                    .criteria("ok")
                    .verify("true")
                    .build(),
            )
            .unwrap();
        let result = tracker.add_step(
            pid,
            step("S-001", pid)
                .title("B")
                .criteria("ok")
                .verify("true")
                .build(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn step_file_generation() {
        let tmp = tempfile::tempdir().unwrap();
        let tracker = sample_tracker();
        tracker.write_step_file("STEP-001", tmp.path()).unwrap();
        assert!(tmp
            .path()
            .join(".planner/STEP-001-create-schema.md")
            .exists());
    }

    #[test]
    fn persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let tracker = sample_tracker();
        tracker.save(tmp.path()).unwrap();

        let loaded = PlanTracker::load(tmp.path()).unwrap();
        assert_eq!(loaded.steps.len(), 3);
        assert_eq!(loaded.project, "TestProject");
    }

    #[test]
    fn wave_and_attempt_defaults() {
        let s = step("STEP-001", 1)
            .title("X")
            .criteria("ok")
            .verify("true")
            .build();
        assert_eq!(s.attempt, 0);
        assert!(s.wave.is_none());
    }

    #[test]
    fn wave_ordinal_ordering() {
        assert!(Wave::Foundation.ordinal() < Wave::W1.ordinal());
        assert!(Wave::W1.ordinal() < Wave::Audit.ordinal());
        assert!(Wave::Audit.ordinal() < Wave::Deploy.ordinal());
    }

    #[test]
    fn ready_steps_respects_dag() {
        let tracker = sample_tracker();
        let ready = tracker.ready_steps(10);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].step_id, "STEP-001");
    }

    #[test]
    fn ready_steps_parallel_disjoint_files() {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("A", p)
                .title("a")
                .files(&["a.rs"])
                .criteria("ok")
                .verify("true")
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("B", p)
                .title("b")
                .files(&["b.rs"])
                .criteria("ok")
                .verify("true")
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("C", p)
                .title("c")
                .files(&["a.rs"])
                .criteria("ok")
                .verify("true")
                .build(),
        )
        .unwrap();
        let ready: Vec<_> = t
            .ready_steps(10)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();
        assert_eq!(ready, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn ready_steps_cap() {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        for i in 0..5 {
            t.add_step(
                p,
                step(&format!("S{i}"), p)
                    .title("x")
                    .files(&[&format!("f{i}.rs")])
                    .criteria("ok")
                    .verify("true")
                    .build(),
            )
            .unwrap();
        }
        assert_eq!(t.ready_steps(2).len(), 2);
    }

    #[test]
    fn ready_steps_holds_audit_until_impl_done() {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("IMPL", p)
                .title("impl")
                .files(&["x.rs"])
                .criteria("ok")
                .verify("true")
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("AUD", p)
                .title("audit")
                .files(&["y.rs"])
                .criteria("ok")
                .verify("true")
                .wave(Wave::Audit)
                .build(),
        )
        .unwrap();
        let ready: Vec<_> = t
            .ready_steps(10)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();
        assert_eq!(ready, vec!["IMPL".to_string()]);
        t.start_step("IMPL").unwrap();
        t.mark_done("IMPL").unwrap();
        let ready2: Vec<_> = t
            .ready_steps(10)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();
        assert_eq!(ready2, vec!["AUD".to_string()]);
    }

    #[test]
    fn retry_resets_to_pending_and_bumps_attempt() {
        let mut t = sample_tracker();
        t.start_step("STEP-001").unwrap();
        t.bump_attempt("STEP-001").unwrap();
        t.reset_to_pending("STEP-001").unwrap();
        let s = t.get_step("STEP-001").unwrap();
        assert_eq!(s.attempt, 1);
        assert_eq!(s.status, StepStatus::Pending);
        assert_eq!(t.ready_steps(10)[0].step_id, "STEP-001");
    }
}
