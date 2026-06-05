//! Implementation planner — DAG-enforced sequential execution with verification gates.
//!
//! Fixes three bugs in the VPS planner:
//! 1. Steps are vague → workers interpret freely → skip ahead
//! 2. No DAG enforcement → workers jump to "interesting" steps
//! 3. No verify gate → "done" claimed without proof

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    pub attempt: u8,
    pub status: StepStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
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
        }
    }

    pub fn tracker_path(project_dir: &Path) -> PathBuf {
        project_dir.join(".planner").join("tracker.json")
    }

    pub fn load(project_dir: &Path) -> Option<Self> {
        let path = Self::tracker_path(project_dir);
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Strict structural validation of a loaded plan. The engine REFUSES to run a
    /// plan that would silently mis-sequence or fake-complete — the exact failure
    /// modes that let a "build" finish with nothing actually working:
    ///   - **dangling `depends_on`** (a typo'd dep id → `deps_satisfied` is never
    ///     true → that step blocks FOREVER, silently);
    ///   - **duplicate `step_id`s** (mark_done/ready_steps act on the wrong step);
    ///   - **empty or trivial `verify_command`** (`true` / `:` / bare `echo` →
    ///     the Guardian's re-run passes unconditionally, so the step is marked Done
    ///     without any proof — the #1 cause of "done but broken").
    /// Returns every issue at once (don't make the operator fix one at a time).
    pub fn validate(&self) -> Result<()> {
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
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join("tracker.json"), json)?;
        Ok(())
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
            bail!(
                "phase {} already has 25 steps (max per phase)",
                phase.name
            );
        }

        // Validate dependencies exist
        for dep in &step.depends_on {
            if !self.steps.iter().any(|s| s.step_id == *dep)
                && !step.step_id.eq(dep)
            {
                bail!(
                    "step {} depends on unknown step {}",
                    step.step_id,
                    dep
                );
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
        self.steps.iter().find(|step| {
            step.status == StepStatus::Pending && self.deps_satisfied(&step.step_id)
        })
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
            bail!(
                "cannot start {} — dependencies not satisfied",
                step_id
            );
        }
        let step = self
            .get_step_mut(step_id)
            .context("step not found")?;
        if step.status != StepStatus::Pending {
            bail!("step {} is not pending (status: {})", step_id, step.status.label());
        }
        step.status = StepStatus::InProgress;
        step.started_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Mark a step as done. In a real execution, the verify_command would be
    /// run first — the caller is responsible for checking that.
    pub fn mark_done(&mut self, step_id: &str) -> Result<()> {
        let step = self
            .get_step_mut(step_id)
            .context("step not found")?;
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
        let step = self
            .get_step_mut(step_id)
            .context("step not found")?;
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
        let done = self.steps.iter().filter(|s| s.status == StepStatus::Done).count();
        let in_progress = self.steps.iter().filter(|s| s.status == StepStatus::InProgress).count();
        let failed = self.steps.iter().filter(|s| s.status == StepStatus::Failed).count();
        let blocked = self.steps.iter().filter(|s| {
            s.status == StepStatus::Pending && !self.deps_satisfied(&s.step_id)
        }).count();
        let ready = self.steps.iter().filter(|s| {
            s.status == StepStatus::Pending && self.deps_satisfied(&s.step_id)
        }).count();

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
            if !visited.contains(&step.step_id) {
                if self.has_cycle(&step.step_id, &mut visited, &mut in_stack) {
                    return false;
                }
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
                    .verify("npx convex dev --once 2>&1 | grep -v ERROR")
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
        assert!(t.validate().is_err(), "trivial `true` verify must be rejected");
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
            step("S-026", pid).title("Too many").criteria("ok").verify("true").build(),
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
            .add_step(pid, step("S-001", pid).title("A").criteria("ok").verify("true").build())
            .unwrap();
        let result = tracker.add_step(
            pid,
            step("S-001", pid).title("B").criteria("ok").verify("true").build(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn step_file_generation() {
        let tmp = std::env::temp_dir().join("omega-planner-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let tracker = sample_tracker();
        tracker.write_step_file("STEP-001", &tmp).unwrap();
        assert!(tmp.join(".planner/STEP-001-create-schema.md").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn persistence() {
        let tmp = std::env::temp_dir().join("omega-planner-persist");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let tracker = sample_tracker();
        tracker.save(&tmp).unwrap();

        let loaded = PlanTracker::load(&tmp).unwrap();
        assert_eq!(loaded.steps.len(), 3);
        assert_eq!(loaded.project, "TestProject");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn wave_and_attempt_defaults() {
        let s = step("STEP-001", 1).title("X").criteria("ok").verify("true").build();
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
        t.add_step(p, step("A", p).title("a").files(&["a.rs"]).criteria("ok").verify("true").build()).unwrap();
        t.add_step(p, step("B", p).title("b").files(&["b.rs"]).criteria("ok").verify("true").build()).unwrap();
        t.add_step(p, step("C", p).title("c").files(&["a.rs"]).criteria("ok").verify("true").build()).unwrap();
        let ready: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
        assert_eq!(ready, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn ready_steps_cap() {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        for i in 0..5 {
            t.add_step(p, step(&format!("S{i}"), p).title("x")
                .files(&[&format!("f{i}.rs")]).criteria("ok").verify("true").build()).unwrap();
        }
        assert_eq!(t.ready_steps(2).len(), 2);
    }

    #[test]
    fn ready_steps_holds_audit_until_impl_done() {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(p, step("IMPL", p).title("impl").files(&["x.rs"]).criteria("ok").verify("true").build()).unwrap();
        t.add_step(p, step("AUD", p).title("audit").files(&["y.rs"]).criteria("ok").verify("true").wave(Wave::Audit).build()).unwrap();
        let ready: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
        assert_eq!(ready, vec!["IMPL".to_string()]);
        t.start_step("IMPL").unwrap();
        t.mark_done("IMPL").unwrap();
        let ready2: Vec<_> = t.ready_steps(10).iter().map(|s| s.step_id.clone()).collect();
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
