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
            StepStatus::Pending => "⬚",
            StepStatus::InProgress => "⏳",
            StepStatus::Done => "✅",
            StepStatus::Blocked => "🚫",
            StepStatus::Failed => "❌",
        }
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

    /// Mark a step as failed.
    pub fn mark_failed(&mut self, step_id: &str) -> Result<()> {
        let step = self
            .get_step_mut(step_id)
            .context("step not found")?;
        step.status = StepStatus::Failed;
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
}
