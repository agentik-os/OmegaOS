use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    pub mission: String,
    pub criteria: Vec<RubricCriterion>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub id: String,
    pub description: String,
    pub weight: f32,
    pub category: CriterionCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionCategory {
    Functional,
    Quality,
    Performance,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeResult {
    pub criterion_id: String,
    pub verdict: GradeVerdict,
    pub confidence: f32,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradeVerdict {
    Satisfied,
    NeedsRevision,
    Unmet,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub oracle: String,
    pub timestamp: DateTime<Utc>,
    pub rubric_pass: bool,
    pub consensus_pass: bool,
    pub adversarial_pass: bool,
    pub regression_pass: bool,
    pub audit_results: Vec<crate::audit::AuditResult>,
    pub audit_pass: bool,
    pub overall_pass: bool,
    pub score: f32,
    pub details: GateDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDetails {
    pub grades: Vec<GradeResult>,
    pub consensus_votes: Vec<ConsensusVote>,
    pub adversarial_challenges: Vec<AdversarialChallenge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub grader: String,
    pub verdict: GradeVerdict,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdversarialChallenge {
    pub challenge: String,
    pub result: ChallengeResult,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeResult {
    DefectFound,
    NoDefect,
    Inconclusive,
}

impl Rubric {
    pub fn new(mission: &str, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            mission: mission.to_string(),
            criteria,
            created_at: Utc::now(),
        }
    }

    pub fn write(&self, state_dir: &Path, oracle: &str) -> Result<()> {
        let path = state_dir.join(format!("{}.rubric.json", oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, oracle: &str) -> Result<Option<Self>> {
        let path = state_dir.join(format!("{}.rubric.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }
}

impl GateResult {
    pub fn evaluate(
        rubric: &Rubric,
        grades: Vec<GradeResult>,
        consensus_votes: Vec<ConsensusVote>,
        adversarial_challenges: Vec<AdversarialChallenge>,
    ) -> Self {
        let rubric_pass = grades
            .iter()
            .all(|g| g.verdict == GradeVerdict::Satisfied);

        let satisfied_votes = consensus_votes
            .iter()
            .filter(|v| v.verdict == GradeVerdict::Satisfied)
            .count();
        let consensus_pass = consensus_votes.is_empty()
            || satisfied_votes * 3 >= consensus_votes.len() * 2;

        let defects = adversarial_challenges
            .iter()
            .filter(|c| matches!(c.result, ChallengeResult::DefectFound))
            .count();
        let adversarial_pass = defects == 0;

        let regression_pass = true; // TODO: compare with prior iteration

        let total_weight: f32 = rubric.criteria.iter().map(|c| c.weight).sum();
        let earned_weight: f32 = grades
            .iter()
            .filter(|g| g.verdict == GradeVerdict::Satisfied)
            .filter_map(|g| {
                rubric
                    .criteria
                    .iter()
                    .find(|c| c.id == g.criterion_id)
                    .map(|c| c.weight)
            })
            .sum();

        let score = if total_weight > 0.0 {
            (earned_weight / total_weight) * 100.0
        } else {
            0.0
        };

        // audit_pass defaults to true (no audits run yet at gate-evaluate time);
        // the orchestrator populates audit_results post-gate and can flip this.
        let audit_pass = true;
        let overall_pass = rubric_pass && consensus_pass && adversarial_pass && regression_pass && audit_pass;

        Self {
            oracle: String::new(),
            timestamp: Utc::now(),
            rubric_pass,
            consensus_pass,
            adversarial_pass,
            regression_pass,
            audit_results: Vec::new(),
            audit_pass,
            overall_pass,
            score,
            details: GateDetails {
                grades,
                consensus_votes,
                adversarial_challenges,
            },
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("{}.gate-result.json", self.oracle));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
