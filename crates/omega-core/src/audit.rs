//! Quality Arsenal audit registry — typed catalogue of the 23 Gestalt-Popper
//! forensic audits that form OmegaOS's quality infrastructure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditDomain {
    Code,
    Flow,
    Design,
    Runtime,
    Feature,
    Performance,
    Security,
    Accessibility,
    Seo,
    Data,
    Api,
    Copy,
    Dx,
    Motion,
    Automation,
    Logic,
    Retention,
    Observability,
    Dependencies,
    Localization,
    Release,
    Privacy,
}

impl AuditDomain {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Code => "Code",
            Self::Flow => "Flow",
            Self::Design => "Design",
            Self::Runtime => "Runtime",
            Self::Feature => "Feature",
            Self::Performance => "Performance",
            Self::Security => "Security",
            Self::Accessibility => "Accessibility",
            Self::Seo => "SEO",
            Self::Data => "Data",
            Self::Api => "API",
            Self::Copy => "Copy",
            Self::Dx => "DX",
            Self::Motion => "Motion",
            Self::Automation => "Automation",
            Self::Logic => "Logic",
            Self::Retention => "Retention",
            Self::Observability => "Observability",
            Self::Dependencies => "Dependencies",
            Self::Localization => "Localization",
            Self::Release => "Release",
            Self::Privacy => "Privacy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditSkill {
    pub id: &'static str,
    pub name: &'static str,
    pub domain: AuditDomain,
    pub phases: u32,
    pub max_score: u32,
    pub normalized_max: u32,
    pub description: &'static str,
    pub triggers: &'static [&'static str],
    pub skill_path: &'static str,
    pub read_only: bool,
}

impl AuditSkill {
    pub fn normalized_score(&self, raw: f32) -> f32 {
        if self.max_score == 0 {
            return 0.0;
        }
        (raw / self.max_score as f32) * self.normalized_max as f32
    }

    pub fn matches_text(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.triggers.iter().any(|t| lower.contains(t))
    }
}

pub fn all_audits() -> Vec<AuditSkill> {
    vec![
        AuditSkill {
            id: "codeaudit",
            name: "Code Architecture",
            domain: AuditDomain::Code,
            phases: 23,
            max_score: 420,
            normalized_max: 100,
            description: "Is the code SOLID?",
            triggers: &["code", "code audit", "audit code"],
            skill_path: "skills/audits/codeaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "flowaudit",
            name: "User Flows",
            domain: AuditDomain::Flow,
            phases: 20,
            max_score: 400,
            normalized_max: 100,
            description: "Does the experience WORK?",
            triggers: &["flow", "user flow", "parcours"],
            skill_path: "skills/audits/flowaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "uiuxaudit",
            name: "Design Quality",
            domain: AuditDomain::Design,
            phases: 23,
            max_score: 420,
            normalized_max: 100,
            description: "Is the interface BEAUTIFUL?",
            triggers: &["ux", "ui", "design audit"],
            skill_path: "skills/audits/uiuxaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "debugaudit",
            name: "Runtime Bugs",
            domain: AuditDomain::Runtime,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "What is BROKEN right now?",
            triggers: &["debug", "runtime bug", "chaos"],
            skill_path: "skills/audits/debugaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "featureaudit",
            name: "Feature Completeness",
            domain: AuditDomain::Feature,
            phases: 16,
            max_score: 320,
            normalized_max: 100,
            description: "Is the product COMPLETE?",
            triggers: &["feature", "completeness"],
            skill_path: "skills/audits/featureaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "perfaudit",
            name: "Performance",
            domain: AuditDomain::Performance,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is it FAST enough?",
            triggers: &["perf", "performance", "core web vitals"],
            skill_path: "skills/audits/perfaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "secaudit",
            name: "Security",
            domain: AuditDomain::Security,
            phases: 20,
            max_score: 400,
            normalized_max: 100,
            description: "Is it SECURE?",
            triggers: &["sec", "security", "owasp", "vulnerab"],
            skill_path: "skills/audits/secaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "a11yaudit",
            name: "Accessibility",
            domain: AuditDomain::Accessibility,
            phases: 16,
            max_score: 320,
            normalized_max: 100,
            description: "Is it ACCESSIBLE?",
            triggers: &["a11y", "accessibility", "wcag"],
            skill_path: "skills/audits/a11yaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "seoaudit",
            name: "SEO",
            domain: AuditDomain::Seo,
            phases: 20,
            max_score: 400,
            normalized_max: 100,
            description: "Is it DISCOVERABLE?",
            triggers: &["seo", "crawlability"],
            skill_path: "skills/audits/seoaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "dataaudit",
            name: "Data Integrity",
            domain: AuditDomain::Data,
            phases: 16,
            max_score: 320,
            normalized_max: 100,
            description: "Is the data INTACT?",
            triggers: &["data integrity", "schema", "data audit"],
            skill_path: "skills/audits/dataaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "apiaudit",
            name: "API Quality",
            domain: AuditDomain::Api,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is the API SOLID?",
            triggers: &["api audit", "api contracts"],
            skill_path: "skills/audits/apiaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "copyaudit",
            name: "Copy & Messaging",
            domain: AuditDomain::Copy,
            phases: 14,
            max_score: 280,
            normalized_max: 100,
            description: "Is the copy CLEAR?",
            triggers: &["copy", "messaging"],
            skill_path: "skills/audits/copyaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "dxaudit",
            name: "Developer Experience",
            domain: AuditDomain::Dx,
            phases: 16,
            max_score: 320,
            normalized_max: 100,
            description: "Is the DX SMOOTH?",
            triggers: &["dx", "developer experience"],
            skill_path: "skills/audits/dxaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "motionaudit",
            name: "Motion Design",
            domain: AuditDomain::Motion,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is the motion PURPOSEFUL?",
            triggers: &["motion", "animation"],
            skill_path: "skills/audits/motionaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "automationaudit",
            name: "Automation",
            domain: AuditDomain::Automation,
            phases: 22,
            max_score: 330,
            normalized_max: 100,
            description: "Is automation RELIABLE?",
            triggers: &["automation", "cron", "scripts"],
            skill_path: "skills/audits/automationaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "logicaudit",
            name: "Systems Logic",
            domain: AuditDomain::Logic,
            phases: 20,
            max_score: 360,
            normalized_max: 100,
            description: "Is the logic OPTIMAL?",
            triggers: &["logic", "optimize logic"],
            skill_path: "skills/audits/logicaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "retentionaudit",
            name: "Retention (READ-ONLY)",
            domain: AuditDomain::Retention,
            phases: 20,
            max_score: 400,
            normalized_max: 100,
            description: "What FEATURES are missing?",
            triggers: &["retention", "feature opportunities"],
            skill_path: "skills/audits/retentionaudit/SKILL.md",
            read_only: true,
        },
        AuditSkill {
            id: "observabilityaudit",
            name: "Observability",
            domain: AuditDomain::Observability,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Can we SEE what it does in prod?",
            triggers: &["observability", "logging audit", "tracing audit", "metrics audit", "alerting audit", "slo audit", "can we debug prod"],
            skill_path: "skills/audits/observabilityaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "depaudit",
            name: "Dependency & Supply-Chain",
            domain: AuditDomain::Dependencies,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is the supply chain SAFE?",
            triggers: &["dep", "depaudit", "dependency audit", "supply chain", "license audit", "lockfile integrity", "sbom"],
            skill_path: "skills/audits/depaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "i18naudit",
            name: "i18n & Localization",
            domain: AuditDomain::Localization,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is it WORLD-READY?",
            triggers: &["i18n", "internationalization", "localization", "l10n", "translation audit", "hardcoded strings", "rtl audit"],
            skill_path: "skills/audits/i18naudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "releaseaudit",
            name: "Release & Shipping-Safety",
            domain: AuditDomain::Release,
            phases: 20,
            max_score: 400,
            normalized_max: 100,
            description: "Is shipping SAFE + reversible?",
            triggers: &["release", "release audit", "ci/cd audit", "pipeline audit", "deploy safety", "rollback", "migration safety", "release readiness"],
            skill_path: "skills/audits/releaseaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "privacyaudit",
            name: "Privacy & Data-Protection",
            domain: AuditDomain::Privacy,
            phases: 18,
            max_score: 360,
            normalized_max: 100,
            description: "Is user data HANDLED LAWFULLY?",
            triggers: &["privacy", "gdpr", "ccpa", "pii", "consent", "cookie compliance", "data retention", "dsar"],
            skill_path: "skills/audits/privacyaudit/SKILL.md",
            read_only: false,
        },
        AuditSkill {
            id: "refontaudit",
            name: "Dashboard Refonte",
            domain: AuditDomain::Design,
            phases: 25,
            max_score: 540,
            normalized_max: 100,
            description: "Should the dashboard be REDESIGNED (Linear/Vercel-grade)?",
            triggers: &["refonte", "redesign dashboard", "comme linear", "comme vercel", "dashboard pro", "dashboard senior"],
            skill_path: "skills/audits/refontaudit/SKILL.md",
            read_only: false,
        },
    ]
}

/// Tokenize text into lowercase alphanumeric words (splitting on any
/// non-alphanumeric character).
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// True if `phrase` (one or more words) appears as a contiguous run of whole
/// words inside `words`. Single-word phrases match a single token; multi-word
/// phrases must match consecutively.
fn phrase_matches(words: &[String], phrase: &str) -> bool {
    let needle = tokenize(phrase);
    if needle.is_empty() {
        return false;
    }
    words.windows(needle.len()).any(|w| w == needle.as_slice())
}

/// Select audits relevant to a mission based on keyword matching.
pub fn select_audits(mission_text: &str, _files: &[String]) -> Vec<&'static str> {
    static AUDITS: std::sync::OnceLock<Vec<AuditSkill>> = std::sync::OnceLock::new();
    let audits = AUDITS.get_or_init(all_audits);
    let lower = mission_text.to_lowercase();
    let words = tokenize(mission_text);

    // "full audit" / "audit complet" → every registered audit (all 23)
    if lower.contains("full audit") || lower.contains("audit complet") || lower.contains("toutes les audits") {
        return audits.iter().map(|a| a.id).collect();
    }

    let mut selected: Vec<&'static str> = audits
        .iter()
        .filter(|a| a.triggers.iter().any(|t| phrase_matches(&words, t)))
        .map(|a| a.id)
        .collect();

    // Composite bundles: security → secaudit + apiaudit + dataaudit
    if phrase_matches(&words, "security") || phrase_matches(&words, "auth") {
        for extra in &["secaudit", "apiaudit", "dataaudit"] {
            if !selected.contains(extra) {
                selected.push(extra);
            }
        }
    }

    // UI changes → uiuxaudit + a11yaudit + motionaudit
    if phrase_matches(&words, "ui") || phrase_matches(&words, "ux") || phrase_matches(&words, "design") {
        for extra in &["uiuxaudit", "a11yaudit", "motionaudit"] {
            if !selected.contains(extra) {
                selected.push(extra);
            }
        }
    }

    // Default end-of-mission minimum quality gate
    if selected.is_empty() {
        selected.push("codeaudit");
        selected.push("debugaudit");
    }

    selected
}

/// Select all audits for a specific domain.
pub fn select_audits_for_domain(domain: AuditDomain) -> Vec<AuditSkill> {
    all_audits().into_iter().filter(|a| a.domain == domain).collect()
}

/// Find a single audit by id.
pub fn find_audit(id: &str) -> Option<AuditSkill> {
    all_audits().into_iter().find(|a| a.id == id)
}

// ── Result types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditVerdict {
    Pass,
    NeedsWork,
    Fail,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub audit_id: String,
    pub raw_score: f32,
    pub max_score: u32,
    pub normalized_score: f32,
    pub confidence: AuditConfidence,
    pub verdict: AuditVerdict,
    pub findings_count: u32,
    pub critical_findings: u32,
    pub worker_session: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl AuditResult {
    pub fn new(audit_id: &str, raw_score: f32, max_score: u32) -> Self {
        let normalized = if max_score > 0 {
            (raw_score / max_score as f32) * 100.0
        } else {
            0.0
        };
        let verdict = if normalized >= 80.0 {
            AuditVerdict::Pass
        } else if normalized >= 50.0 {
            AuditVerdict::NeedsWork
        } else {
            AuditVerdict::Fail
        };
        Self {
            audit_id: audit_id.to_string(),
            raw_score,
            max_score,
            normalized_score: normalized,
            confidence: AuditConfidence::Medium,
            verdict,
            findings_count: 0,
            critical_findings: 0,
            worker_session: None,
            completed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub mission_id: String,
    pub audits: Vec<AuditResult>,
    pub overall_score: f32,
    pub overall_verdict: AuditVerdict,
    pub created_at: DateTime<Utc>,
}

impl AuditReport {
    pub fn from_results(mission_id: &str, audits: Vec<AuditResult>) -> Self {
        let overall_score = if audits.is_empty() {
            0.0
        } else {
            audits.iter().map(|a| a.normalized_score).sum::<f32>() / audits.len() as f32
        };
        let overall_verdict = if audits.is_empty() {
            AuditVerdict::Aborted
        } else if audits.iter().all(|a| a.verdict == AuditVerdict::Pass) {
            AuditVerdict::Pass
        } else if audits.iter().any(|a| a.verdict == AuditVerdict::Fail) {
            AuditVerdict::Fail
        } else {
            AuditVerdict::NeedsWork
        };
        Self {
            mission_id: mission_id.to_string(),
            audits,
            overall_score,
            overall_verdict,
            created_at: Utc::now(),
        }
    }

    pub fn all_pass(&self, threshold: f32) -> bool {
        self.audits.iter().all(|a| a.normalized_score >= threshold)
    }

    /// Format the report as Telegram-ready HTML using the formatting engine.
    pub fn to_telegram_html(&self) -> String {
        use crate::formatting;

        let verdict_icon = match self.overall_verdict {
            AuditVerdict::Pass => "✅",
            AuditVerdict::NeedsWork => "🟡",
            AuditVerdict::Fail => "🔴",
            AuditVerdict::Aborted => "⚫",
        };

        let mut out = format!(
            "{} <b>Quality Report — {}</b>\n\
             <b>Overall:</b> <code>{:.0}/100</code>\n\
             ━━━━━━━━━━\n",
            verdict_icon,
            formatting::escape_html(&self.mission_id),
            self.overall_score,
        );

        let scores: Vec<(&str, f32, f32)> = self
            .audits
            .iter()
            .map(|a| {
                let name = find_audit(&a.audit_id)
                    .map(|s| s.name)
                    .unwrap_or("Unknown");
                (name, a.raw_score, a.max_score as f32)
            })
            .collect();

        out.push_str(&formatting::format_audit_scores(&scores));
        out
    }
}

impl AuditSkill {
    /// Generate a dispatch prompt for invoking this audit in a worker session.
    /// The first line is always the skill slash command.
    pub fn dispatch_prompt(&self, scope: &AuditScope) -> String {
        let mut prompt = format!("/{}", self.id);

        if let Some(url) = &scope.url {
            prompt.push_str(&format!(" --url={}", url));
        }
        if !scope.files.is_empty() {
            prompt.push_str(&format!(" --files={}", scope.files.join(",")));
        }
        if let Some(desc) = &scope.description {
            prompt.push_str(&format!(" --scope=\"{}\"", desc));
        }

        prompt.push_str("\n\ndo not expand beyond this scope");
        prompt
    }
}

/// Scoping parameters for a targeted audit.
#[derive(Debug, Clone, Default)]
pub struct AuditScope {
    pub url: Option<String>,
    pub files: Vec<String>,
    pub description: Option<String>,
}

/// Build a list of dispatch prompts for all audits selected for a mission.
pub fn build_dispatch_prompts(mission_text: &str, scope: &AuditScope) -> Vec<(String, String)> {
    let selected_ids = select_audits(mission_text, &scope.files.iter().map(|s| s.clone()).collect::<Vec<_>>());
    selected_ids
        .into_iter()
        .filter_map(|id| {
            find_audit(id).map(|audit| {
                let prompt = audit.dispatch_prompt(scope);
                (id.to_string(), prompt)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_23_audits() {
        assert_eq!(all_audits().len(), 23);
    }

    #[test]
    fn every_audit_has_metadata() {
        for a in all_audits() {
            assert!(!a.id.is_empty());
            assert!(!a.name.is_empty());
            assert!(!a.description.is_empty());
            assert!(a.phases > 0);
            assert!(a.max_score > 0);
            assert_eq!(a.normalized_max, 100);
            assert!(!a.triggers.is_empty());
            assert!(!a.skill_path.is_empty());
        }
    }

    #[test]
    fn unique_ids() {
        let audits = all_audits();
        let ids: Vec<&str> = audits.iter().map(|a| a.id).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }

    #[test]
    fn retentionaudit_is_read_only() {
        let audits = all_audits();
        let retention = audits.iter().find(|a| a.id == "retentionaudit").unwrap();
        assert!(retention.read_only);
        let non_readonly = audits.iter().filter(|a| a.id != "retentionaudit");
        for a in non_readonly {
            assert!(!a.read_only, "{} should not be read_only", a.id);
        }
    }

    #[test]
    fn select_full_audit_returns_all() {
        let selected = select_audits("full audit of the project", &[]);
        assert_eq!(selected.len(), 23);
    }

    #[test]
    fn select_security_mission() {
        let selected = select_audits("fix auth flow security", &[]);
        assert!(selected.contains(&"secaudit"));
        assert!(selected.contains(&"apiaudit"));
        assert!(selected.contains(&"dataaudit"));
    }

    #[test]
    fn select_default_minimum() {
        let selected = select_audits("do something generic", &[]);
        assert!(selected.contains(&"codeaudit"));
        assert!(selected.contains(&"debugaudit"));
    }

    #[test]
    fn select_ui_mission() {
        let selected = select_audits("redesign the UI dashboard", &[]);
        assert!(selected.contains(&"uiuxaudit"));
        assert!(selected.contains(&"a11yaudit"));
        assert!(selected.contains(&"motionaudit"));
    }

    #[test]
    fn select_audits_for_domain_works() {
        let code_audits = select_audits_for_domain(AuditDomain::Code);
        assert_eq!(code_audits.len(), 1);
        assert_eq!(code_audits[0].id, "codeaudit");
    }

    #[test]
    fn find_audit_works() {
        assert!(find_audit("codeaudit").is_some());
        assert!(find_audit("nonexistent").is_none());
    }

    #[test]
    fn audit_result_scoring() {
        let r = AuditResult::new("codeaudit", 336.0, 420);
        assert!((r.normalized_score - 80.0).abs() < 0.1);
        assert_eq!(r.verdict, AuditVerdict::Pass);
    }

    #[test]
    fn audit_report_all_pass() {
        let results = vec![
            AuditResult::new("codeaudit", 336.0, 420),
            AuditResult::new("debugaudit", 288.0, 360),
        ];
        let report = AuditReport::from_results("m-test", results);
        assert!(report.all_pass(70.0));
        assert_eq!(report.overall_verdict, AuditVerdict::Pass);
    }

    #[test]
    fn normalized_score_calculation() {
        let skill = find_audit("codeaudit").unwrap();
        let score = skill.normalized_score(210.0);
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn audit_report_telegram_html() {
        let results = vec![
            AuditResult::new("codeaudit", 336.0, 420),
            AuditResult::new("secaudit", 200.0, 400),
        ];
        let report = AuditReport::from_results("m-test", results);
        let html = report.to_telegram_html();
        assert!(html.contains("Quality Report"));
        assert!(html.contains("m-test"));
    }

    #[test]
    fn dispatch_prompt_with_scope() {
        let audit = find_audit("codeaudit").unwrap();
        let scope = AuditScope {
            url: Some("https://app.example.com/dashboard".to_string()),
            files: vec!["src/auth.rs".to_string()],
            description: Some("auth module only".to_string()),
        };
        let prompt = audit.dispatch_prompt(&scope);
        assert!(prompt.starts_with("/codeaudit"));
        assert!(prompt.contains("--url="));
        assert!(prompt.contains("--files="));
        assert!(prompt.contains("--scope="));
        assert!(prompt.contains("do not expand beyond this scope"));
    }

    #[test]
    fn dispatch_prompt_minimal_scope() {
        let audit = find_audit("debugaudit").unwrap();
        let scope = AuditScope::default();
        let prompt = audit.dispatch_prompt(&scope);
        assert!(prompt.starts_with("/debugaudit"));
        assert!(!prompt.contains("--url="));
        assert!(!prompt.contains("--files="));
    }

    #[test]
    fn build_dispatch_prompts_security() {
        let scope = AuditScope {
            url: Some("https://app.example.com".to_string()),
            ..Default::default()
        };
        let prompts = build_dispatch_prompts("fix auth flow security", &scope);
        assert!(prompts.iter().any(|(id, _)| id == "secaudit"));
        assert!(prompts.iter().any(|(id, _)| id == "apiaudit"));
        for (_, prompt) in &prompts {
            assert!(prompt.contains("--url="));
        }
    }
}
