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
    pub audit_skills: Vec<AuditSkill>,
}

/// Maps audit keywords to specific forensic skill invocations.
/// Mirrors the live system's Quality Arsenal (see `crate::audit::all_audits`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSkill {
    pub skill: String,
    pub trigger: String,
}

/// Detect audit keywords and return matching skills.
fn detect_audit_skills(text: &str) -> Vec<AuditSkill> {
    let lower = text.to_lowercase();
    let mut skills = Vec::new();

    let audit_table: &[(&[&str], &str)] = &[
        (&["ux", "ui", "design audit", "audit visuel", "audit design", "ui/ux"], "uiuxaudit"),
        (&["refonte", "refontaudit", "redesign dashboard"], "refontaudit"),
        (&["flow", "user flow", "parcours", "audit flow"], "flowaudit"),
        (&["code audit", "code quality", "audit code"], "codeaudit"),
        (&["perf", "performance", "core web vitals", "audit perf"], "perfaudit"),
        (&["security", "vulnerab", "owasp", "audit sec"], "secaudit"),
        (&["a11y", "accessibility", "wcag"], "a11yaudit"),
        (&["seo", "audit seo", "crawlability"], "seoaudit"),
        (&["feature audit", "completeness", "audit feature"], "featureaudit"),
        (&["copy audit", "messaging audit", "audit copy"], "copyaudit"),
        (&["dx audit", "developer experience", "audit dx"], "dxaudit"),
        (&["motion audit", "animation audit", "audit motion"], "motionaudit"),
        (&["data integrity", "data audit", "audit data", "schema audit"], "dataaudit"),
        (&["api audit", "audit api", "api contracts"], "apiaudit"),
        (&["debugaudit", "runtime bug", "debug audit"], "debugaudit"),
        (&["automation", "cron", "crontab", "scripts audit", "daemon health"], "automationaudit"),
        (&["logic", "optimize logic", "system optimization", "architecture logic"], "logicaudit"),
        (&["retention", "retentionaudit", "feature opportunities", "make it sticky"], "retentionaudit"),
    ];

    for (triggers, skill) in audit_table {
        for trigger in *triggers {
            if lower.contains(trigger) {
                skills.push(AuditSkill {
                    skill: skill.to_string(),
                    trigger: trigger.to_string(),
                });
                break;
            }
        }
    }

    // "full audit" is authoritative: it means run EVERY audit, regardless of
    // any skills already detected above. Source the full list from the audit
    // registry (single source of truth) so routing never drifts out of sync
    // with the Quality Arsenal as audits are added or removed.
    if lower.contains("full audit") || lower.contains("audit complet") || lower.contains("toutes les audits")
    {
        skills.clear();
        for audit in crate::audit::all_audits() {
            skills.push(AuditSkill {
                skill: audit.id.to_string(),
                trigger: "full audit".to_string(),
            });
        }
    }

    skills
}

pub fn classify_mission(mission: &str) -> RoutingDecision {
    // An empty or whitespace-only mission is a user/intake error (typo, empty
    // API body). Reject it explicitly rather than scoring "" as a Simple task
    // and wasting an agent on an empty objective. Signature stays infallible
    // (8 call sites treat it as such); the rejection is surfaced via reasoning.
    if mission.trim().is_empty() {
        return RoutingDecision {
            complexity: Complexity::Simple,
            reasoning: vec!["INVALID: mission text is empty — nothing to route".to_string()],
            suggested_agent: "morpheus".to_string(),
            decompose: false,
            use_team: false,
            use_quality_gate: false,
            audit_skills: Vec::new(),
        };
    }

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

    let audit_skills = detect_audit_skills(mission);
    if !audit_skills.is_empty() {
        reasoning.push(format!(
            "Audit skills detected: {}",
            audit_skills.iter().map(|a| a.skill.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    RoutingDecision {
        complexity,
        reasoning,
        suggested_agent: suggested_agent.to_string(),
        decompose: matches!(complexity, Complexity::Complex | Complexity::Epic),
        use_team: matches!(complexity, Complexity::Epic),
        use_quality_gate: matches!(complexity, Complexity::Complex | Complexity::Epic)
            || lower.contains("audit")
            || !audit_skills.is_empty(),
        audit_skills,
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

    #[test]
    fn empty_mission_is_rejected() {
        let d = classify_mission("   ");
        assert!(d.audit_skills.is_empty());
        assert!(!d.use_quality_gate);
        assert!(d.reasoning.iter().any(|r| r.contains("INVALID")));
    }

    #[test]
    fn full_audit_runs_entire_registry() {
        let d = classify_mission("please run a full audit on everything");
        // "full audit" is authoritative and sourced from the registry.
        assert_eq!(d.audit_skills.len(), crate::audit::all_audits().len());
        assert!(d.audit_skills.iter().all(|a| a.trigger == "full audit"));
    }

    #[test]
    fn full_audit_overrides_partial_detection() {
        // Even when specific audits are named, "full audit" expands to all.
        let d = classify_mission("do a security and seo full audit");
        assert_eq!(d.audit_skills.len(), crate::audit::all_audits().len());
    }
}
