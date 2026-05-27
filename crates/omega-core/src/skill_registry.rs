//! Skill registry — discovers and manages skills from ~/.omega/skills/.
//!
//! Skills are markdown-based instruction files that agents invoke by name.
//! Each skill directory contains a SKILL.md with frontmatter (name, description,
//! triggers, phases, scoring) and the full protocol body.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A discovered skill with parsed metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub triggers: Vec<String>,
    pub phases: Option<u32>,
    pub max_score: Option<u32>,
    pub read_only: bool,
    pub category: SkillCategory,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    Audit,
    Build,
    Design,
    Orchestration,
    Marketing,
    Utility,
    Custom,
}

impl SkillCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Audit => "Audit",
            Self::Build => "Build",
            Self::Design => "Design",
            Self::Orchestration => "Orchestration",
            Self::Marketing => "Marketing",
            Self::Utility => "Utility",
            Self::Custom => "Custom",
        }
    }

    fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("audit") {
            Self::Audit
        } else if path_str.contains("build") || path_str.contains("deploy") {
            Self::Build
        } else if path_str.contains("design") || path_str.contains("ui") {
            Self::Design
        } else if path_str.contains("orchestr") || path_str.contains("team") {
            Self::Orchestration
        } else if path_str.contains("market") || path_str.contains("seo") || path_str.contains("ads") {
            Self::Marketing
        } else {
            Self::Custom
        }
    }
}

/// The skill registry: discovers, indexes, and looks up skills.
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    skills_dir: PathBuf,
}

impl SkillRegistry {
    /// Create a registry from the default skills directory (~/.omega/skills/).
    pub fn discover_default() -> Result<Self> {
        let home = dirs::home_dir().context("no home directory")?;
        let skills_dir = home.join(".omega").join("skills");
        Self::discover(&skills_dir)
    }

    /// Create a registry by scanning a specific directory.
    pub fn discover(skills_dir: &Path) -> Result<Self> {
        let mut skills = HashMap::new();

        if !skills_dir.exists() {
            tracing::debug!(path = %skills_dir.display(), "skills directory does not exist");
            return Ok(Self {
                skills,
                skills_dir: skills_dir.to_path_buf(),
            });
        }

        let entries = std::fs::read_dir(skills_dir)
            .with_context(|| format!("reading {}", skills_dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let skill_file = path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }

            match parse_skill_file(&skill_file) {
                Ok(skill) => {
                    tracing::debug!(name = %skill.name, "discovered skill");
                    skills.insert(skill.name.clone(), skill);
                }
                Err(e) => {
                    tracing::warn!(path = %skill_file.display(), error = %e, "failed to parse skill");
                }
            }
        }

        tracing::info!(count = skills.len(), dir = %skills_dir.display(), "skill discovery complete");

        Ok(Self {
            skills,
            skills_dir: skills_dir.to_path_buf(),
        })
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        let mut skills: Vec<_> = self.skills.values().collect();
        skills.sort_by_key(|s| &s.name);
        skills
    }

    pub fn list_by_category(&self, category: SkillCategory) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    pub fn find_by_trigger(&self, text: &str) -> Vec<&Skill> {
        let lower = text.to_lowercase();
        self.skills
            .values()
            .filter(|s| s.triggers.iter().any(|t| lower.contains(&t.to_lowercase())))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Register a skill programmatically (for built-in audit skills).
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Register all 17 Quality Arsenal audits from the audit registry.
    pub fn register_audits(&mut self) {
        for audit in crate::audit::all_audits() {
            let skill = Skill {
                name: audit.id.to_string(),
                description: audit.description.to_string(),
                path: self.skills_dir.join(audit.skill_path),
                triggers: audit.triggers.iter().map(|t| t.to_string()).collect(),
                phases: Some(audit.phases),
                max_score: Some(audit.max_score),
                read_only: audit.read_only,
                category: SkillCategory::Audit,
                discovered_at: Utc::now(),
            };
            self.skills.insert(skill.name.clone(), skill);
        }
    }
}

/// Parse a SKILL.md file to extract skill metadata from YAML-style frontmatter.
fn parse_skill_file(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    let mut name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut description = String::new();
    let mut triggers = Vec::new();
    let mut phases = None;
    let mut max_score = None;
    let mut read_only = false;

    // Parse YAML-like frontmatter between --- markers
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;
    let mut frontmatter_seen = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !frontmatter_seen {
                in_frontmatter = true;
                frontmatter_seen = true;
                continue;
            } else {
                break;
            }
        }

        if in_frontmatter {
            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "name" => name = val.to_string(),
                    "description" => description = val.to_string(),
                    "read_only" | "readOnly" => read_only = val == "true",
                    "phases" => phases = val.parse().ok(),
                    "max_score" | "maxScore" => max_score = val.parse().ok(),
                    _ => {}
                }
            }
        }
    }

    // If no frontmatter, try to extract name from first heading
    if description.is_empty() {
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                description = trimmed[2..].to_string();
                break;
            }
        }
    }

    // Parse trigger lines (common pattern: "trigger:" or "Use when user says")
    for line in &lines {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("trigger") || trimmed.contains("use when user says") {
            if let Some(quotes_content) = extract_quoted_strings(line) {
                triggers.extend(quotes_content);
            }
        }
    }

    if triggers.is_empty() {
        triggers.push(name.clone());
    }

    let category = SkillCategory::from_path(path);

    Ok(Skill {
        name,
        description,
        path: path.to_path_buf(),
        triggers,
        phases,
        max_score,
        read_only,
        category,
        discovered_at: Utc::now(),
    })
}

fn extract_quoted_strings(text: &str) -> Option<Vec<String>> {
    let mut strings = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();
    let mut quote_char = '"';

    for ch in text.chars() {
        if !in_quote && (ch == '"' || ch == '\'') {
            in_quote = true;
            quote_char = ch;
            current.clear();
        } else if in_quote && ch == quote_char {
            in_quote = false;
            if !current.is_empty() {
                strings.push(current.clone());
            }
        } else if in_quote {
            current.push(ch);
        }
    }

    if strings.is_empty() {
        None
    } else {
        Some(strings)
    }
}

/// Audit orchestrator: auto-selects relevant audits for a mission.
#[derive(Debug, Clone)]
pub struct AuditOrchestrator {
    registry: SkillRegistry,
}

impl AuditOrchestrator {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Select audits relevant to a mission based on keywords.
    /// Returns skill names to invoke.
    pub fn select_for_mission(&self, mission_text: &str) -> Vec<String> {
        let ids = crate::audit::select_audits(mission_text, &[]);
        ids.into_iter().map(|id| id.to_string()).collect()
    }

    /// Select all audits (full audit mode).
    pub fn select_all(&self) -> Vec<String> {
        crate::audit::all_audits()
            .into_iter()
            .map(|a| a.id.to_string())
            .collect()
    }

    /// Get the full Skill metadata for an audit by id.
    pub fn get_audit_skill(&self, id: &str) -> Option<&Skill> {
        self.registry.get(id)
    }
}

/// Audit tracker: monitors freshness and scores across audits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTracker {
    pub entries: HashMap<String, AuditTrackerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrackerEntry {
    pub audit_id: String,
    pub project: String,
    pub last_run: DateTime<Utc>,
    pub last_score: f32,
    pub max_score: u32,
    pub normalized_score: f32,
    pub trend: ScoreTrend,
    pub run_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTrend {
    Improving,
    Stable,
    Declining,
    New,
}

impl AuditTracker {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Load tracker state from disk.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading tracker at {}", path.display()))?;
        serde_json::from_str(&content).context("parsing audit tracker")
    }

    /// Save tracker state to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).with_context(|| format!("writing tracker to {}", path.display()))
    }

    /// Record a new audit result.
    pub fn record(&mut self, project: &str, audit_id: &str, score: f32, max_score: u32) {
        let key = format!("{}:{}", project, audit_id);
        let normalized = if max_score > 0 {
            (score / max_score as f32) * 100.0
        } else {
            0.0
        };

        let entry = self.entries.entry(key).or_insert_with(|| AuditTrackerEntry {
            audit_id: audit_id.to_string(),
            project: project.to_string(),
            last_run: Utc::now(),
            last_score: 0.0,
            max_score,
            normalized_score: 0.0,
            trend: ScoreTrend::New,
            run_count: 0,
        });

        let prev_score = entry.normalized_score;
        entry.last_run = Utc::now();
        entry.last_score = score;
        entry.max_score = max_score;
        entry.normalized_score = normalized;
        entry.run_count += 1;

        entry.trend = if entry.run_count <= 1 {
            ScoreTrend::New
        } else if normalized > prev_score + 2.0 {
            ScoreTrend::Improving
        } else if normalized < prev_score - 2.0 {
            ScoreTrend::Declining
        } else {
            ScoreTrend::Stable
        };
    }

    /// Get stale audits (older than `max_age`).
    pub fn stale_audits(&self, max_age: chrono::Duration) -> Vec<&AuditTrackerEntry> {
        let cutoff = Utc::now() - max_age;
        self.entries
            .values()
            .filter(|e| e.last_run < cutoff)
            .collect()
    }

    /// Get scores for a specific project.
    pub fn project_scores(&self, project: &str) -> Vec<&AuditTrackerEntry> {
        self.entries
            .values()
            .filter(|e| e.project == project)
            .collect()
    }

    /// Get the latest entry for a specific audit on a project.
    pub fn latest(&self, project: &str, audit_id: &str) -> Option<&AuditTrackerEntry> {
        let key = format!("{}:{}", project, audit_id);
        self.entries.get(&key)
    }
}

impl Default for AuditTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn skill_category_from_path() {
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/audits/codeaudit/SKILL.md")),
            SkillCategory::Audit
        );
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/deploy/SKILL.md")),
            SkillCategory::Build
        );
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/random/SKILL.md")),
            SkillCategory::Custom
        );
    }

    #[test]
    fn parse_skill_with_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(
            &skill_file,
            "---\nname: test-skill\ndescription: A test skill\nphases: 10\nmax_score: 200\n---\n# Test Skill\nBody here.",
        )
        .unwrap();

        let skill = parse_skill_file(&skill_file).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.phases, Some(10));
        assert_eq!(skill.max_score, Some(200));
    }

    #[test]
    fn parse_skill_without_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(&skill_file, "# My Great Skill\nDoes things.").unwrap();

        let skill = parse_skill_file(&skill_file).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "My Great Skill");
    }

    #[test]
    fn registry_discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::discover(dir.path()).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn registry_discover_with_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("alpha");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: alpha\ndescription: Alpha skill\n---\n",
        )
        .unwrap();

        let registry = SkillRegistry::discover(dir.path()).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.get("alpha").is_some());
    }

    #[test]
    fn registry_register_audits() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        assert!(registry.count() >= 17);
        assert!(registry.get("codeaudit").is_some());
        assert!(registry.get("retentionaudit").is_some());
    }

    #[test]
    fn registry_find_by_trigger() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        let matches = registry.find_by_trigger("fix the security vulnerability");
        assert!(matches.iter().any(|s| s.name == "secaudit"));
    }

    #[test]
    fn audit_tracker_record_and_query() {
        let mut tracker = AuditTracker::new();
        tracker.record("Causio", "codeaudit", 336.0, 420);
        assert_eq!(tracker.entries.len(), 1);

        let entry = tracker.latest("Causio", "codeaudit").unwrap();
        assert_eq!(entry.run_count, 1);
        assert!((entry.normalized_score - 80.0).abs() < 0.1);
        assert_eq!(entry.trend, ScoreTrend::New);
    }

    #[test]
    fn audit_tracker_trend_detection() {
        let mut tracker = AuditTracker::new();
        tracker.record("P", "codeaudit", 200.0, 420);
        tracker.record("P", "codeaudit", 336.0, 420);
        let entry = tracker.latest("P", "codeaudit").unwrap();
        assert_eq!(entry.trend, ScoreTrend::Improving);
        assert_eq!(entry.run_count, 2);
    }

    #[test]
    fn audit_tracker_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tracker.json");

        let mut tracker = AuditTracker::new();
        tracker.record("Causio", "codeaudit", 336.0, 420);
        tracker.save(&path).unwrap();

        let loaded = AuditTracker::load(&path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[test]
    fn audit_orchestrator_selects_audits() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        let orchestrator = AuditOrchestrator::new(registry);

        let selected = orchestrator.select_for_mission("fix auth flow security");
        assert!(selected.contains(&"secaudit".to_string()));
    }

    #[test]
    fn audit_orchestrator_select_all() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::discover(dir.path()).unwrap();
        let orchestrator = AuditOrchestrator::new(registry);
        assert_eq!(orchestrator.select_all().len(), 17);
    }

    #[test]
    fn extract_quoted_strings_works() {
        let result = extract_quoted_strings(r#"trigger: "foo", "bar baz""#).unwrap();
        assert_eq!(result, vec!["foo", "bar baz"]);
    }

    #[test]
    fn extract_quoted_strings_single_quotes() {
        let result = extract_quoted_strings("trigger: 'hello'").unwrap();
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn extract_quoted_strings_empty() {
        assert!(extract_quoted_strings("no quotes here").is_none());
    }
}
