use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
    Epic,
}

impl Complexity {
    pub fn label(&self) -> &'static str {
        match self {
            Complexity::Simple => "SIMPLE",
            Complexity::Medium => "MEDIUM",
            Complexity::Complex => "COMPLEX",
            Complexity::Epic => "EPIC",
        }
    }

    pub fn recommended_agents(&self) -> usize {
        match self {
            Complexity::Simple => 1,
            Complexity::Medium => 1,
            Complexity::Complex => 3,
            Complexity::Epic => 5,
        }
    }

    pub fn estimated_minutes(&self) -> u32 {
        match self {
            Complexity::Simple => 5,
            Complexity::Medium => 20,
            Complexity::Complex => 60,
            Complexity::Epic => 240,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub complexity: Complexity,
    pub reasoning: Vec<String>,
    pub suggested_agent: String,
    pub decompose: bool,
    pub use_team: bool,
    pub use_quality_gate: bool,
}

pub fn classify_mission(mission: &str) -> RoutingDecision {
    let lower = mission.to_lowercase();
    let mut reasoning = Vec::new();
    let mut score: i32 = 0;

    let epic_signals = [
        "build", "create", "implement entire", "redesign", "rewrite",
        "migrate", "refactor everything", "full audit", "complete overhaul",
        "from scratch", "ground up",
    ];
    let complex_signals = [
        "audit", "implement", "refactor", "design", "architect",
        "multi-step", "multiple files", "across", "integrate", "across multiple",
    ];
    let medium_signals = [
        "fix", "add feature", "update", "modify", "improve", "enhance",
        "rewrite function", "change", "adjust",
    ];
    let simple_signals = [
        "rename", "typo", "format", "lint", "comment", "remove",
        "delete unused", "quick fix", "one-line", "fix typo",
    ];

    for sig in &epic_signals {
        if lower.contains(sig) {
            score += 4;
            reasoning.push(format!("Epic signal: '{}'", sig));
            break;
        }
    }
    for sig in &complex_signals {
        if lower.contains(sig) {
            score += 2;
            reasoning.push(format!("Complex signal: '{}'", sig));
            break;
        }
    }
    for sig in &medium_signals {
        if lower.contains(sig) {
            score += 1;
            reasoning.push(format!("Medium signal: '{}'", sig));
            break;
        }
    }
    for sig in &simple_signals {
        if lower.contains(sig) {
            score -= 1;
            reasoning.push(format!("Simple signal: '{}'", sig));
            break;
        }
    }

    let word_count = mission.split_whitespace().count();
    if word_count > 50 {
        score += 2;
        reasoning.push(format!("Long mission ({} words)", word_count));
    } else if word_count > 20 {
        score += 1;
        reasoning.push(format!("Medium-length mission ({} words)", word_count));
    } else if word_count < 5 {
        score -= 1;
        reasoning.push(format!("Very short mission ({} words)", word_count));
    }

    let file_mentions = lower.matches("file").count() + lower.matches(".rs").count()
        + lower.matches(".py").count() + lower.matches(".ts").count();
    if file_mentions > 3 {
        score += 1;
        reasoning.push(format!("Multiple file references ({})", file_mentions));
    }

    if lower.contains("audit") {
        reasoning.push("Audit detected — recommend quality gate".to_string());
    }

    let complexity = if score >= 5 {
        Complexity::Epic
    } else if score >= 3 {
        Complexity::Complex
    } else if score >= 1 {
        Complexity::Medium
    } else {
        Complexity::Simple
    };

    let suggested_agent = match complexity {
        Complexity::Simple => "morpheus",
        Complexity::Medium => "morpheus",
        Complexity::Complex => "oracle",
        Complexity::Epic => "oracle",
    };

    RoutingDecision {
        complexity,
        reasoning,
        suggested_agent: suggested_agent.to_string(),
        decompose: matches!(complexity, Complexity::Complex | Complexity::Epic),
        use_team: matches!(complexity, Complexity::Epic),
        use_quality_gate: matches!(complexity, Complexity::Complex | Complexity::Epic)
            || lower.contains("audit"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_typo_is_simple() {
        let d = classify_mission("fix typo in README");
        assert_eq!(d.complexity, Complexity::Simple);
    }

    #[test]
    fn build_auth_is_complex_or_epic() {
        let d = classify_mission("build the entire authentication system with OAuth2 and JWT refresh tokens");
        assert!(matches!(d.complexity, Complexity::Complex | Complexity::Epic));
    }

    #[test]
    fn audit_triggers_quality_gate() {
        let d = classify_mission("audit the security of the API endpoints");
        assert!(d.use_quality_gate);
    }
}
