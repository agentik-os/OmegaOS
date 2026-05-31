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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardian_min_one_attempt() {
        let g = Guardian::new(0);
        assert_eq!(g.max_attempts, 1);
    }
}
