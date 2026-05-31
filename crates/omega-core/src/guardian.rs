//! Guardian — verified completion. A worker's own `done.json` is an input,
//! never the verdict (R-VERIFY). Before any step is marked Done, the Guardian
//! independently proves it.
//!
//! Tier 1 (M1, always): re-run the step's `verify_command` via IntentVerifier.
//! Tier 2 (follow-on): adversarial consensus via gate.rs for high-stakes steps.

use crate::planner::PlanStep;
use crate::verifier::{IntentSpec, IntentVerifier};
use std::path::Path;

/// Outcome of an independent verification of a completed step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Proven done — safe to mark the step Done.
    Pass,
    /// Not proven, but attempts remain — re-dispatch with this feedback.
    Retry { feedback: String },
    /// Not proven and out of attempts — mark the step Failed.
    Fail { reason: String },
}

pub struct Guardian {
    max_attempts: u8,
}

impl Guardian {
    pub fn new(max_attempts: u8) -> Self {
        Self { max_attempts: max_attempts.max(1) }
    }

    /// Tier 1 — deterministic. Re-run the step's verify_command in the project
    /// dir, independent of the worker's claim. `attempt` is 1-based.
    pub async fn verify(&self, step: &PlanStep, project_dir: &Path, attempt: u8) -> Verdict {
        if step.verify_command.trim().is_empty() {
            return Verdict::Fail {
                reason: format!("step {} has an empty verify_command", step.step_id),
            };
        }
        let intent = IntentSpec {
            action: step.title.clone(),
            target: step.files_to_touch.join(","),
            success_criteria: vec![step.done_criteria.clone()],
            verify_commands: vec![step.verify_command.clone()],
        };
        let verifier = IntentVerifier::default();
        match verifier.verify_once(&intent, project_dir, attempt as u32).await {
            Ok(result) => {
                let proven = !result.command_results.is_empty()
                    && result.command_results.iter().all(|c| c.passed);
                if proven {
                    Verdict::Pass
                } else if attempt < self.max_attempts {
                    let feedback = result
                        .command_results
                        .iter()
                        .map(|c| format!("$ {}\nexit {}\n{}", c.command, c.exit_code, c.stderr.trim()))
                        .collect::<Vec<_>>()
                        .join("\n---\n");
                    Verdict::Retry { feedback }
                } else {
                    Verdict::Fail {
                        reason: format!(
                            "verify_command failed after {} attempt(s): {}",
                            attempt, step.verify_command
                        ),
                    }
                }
            }
            Err(e) => Verdict::Fail {
                reason: format!("guardian could not run verify_command: {e}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::step;

    fn done_step(verify: &str) -> PlanStep {
        step("STEP-001", 1)
            .title("Create thing")
            .description("make the thing")
            .files(&["thing.rs"])
            .criteria("compiles")
            .verify(verify)
            .build()
    }

    #[test]
    fn guardian_min_one_attempt() {
        let g = Guardian::new(0);
        assert_eq!(g.max_attempts, 1);
    }

    #[tokio::test]
    async fn tier1_pass_when_command_exits_zero() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("true");
        assert_eq!(g.verify(&s, dir.path(), 1).await, Verdict::Pass);
    }

    #[tokio::test]
    async fn tier1_retry_then_fail_when_command_exits_nonzero() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("false");
        assert!(matches!(g.verify(&s, dir.path(), 1).await, Verdict::Retry { .. }));
        assert!(matches!(g.verify(&s, dir.path(), 2).await, Verdict::Fail { .. }));
    }

    #[tokio::test]
    async fn tier1_fail_on_empty_verify_command() {
        let g = Guardian::new(2);
        let dir = tempfile::tempdir().unwrap();
        let s = done_step("   ");
        assert!(matches!(g.verify(&s, dir.path(), 1).await, Verdict::Fail { .. }));
    }
}
