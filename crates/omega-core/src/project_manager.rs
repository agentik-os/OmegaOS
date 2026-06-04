//! Project CRUD — create, register, discover, persist projects.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedProject {
    pub name: String,
    pub path: PathBuf,
    pub icon: Option<String>,
    pub telegram_topic_id: Option<i64>,
    pub oracle_session: Option<String>,
    pub git_email: Option<String>,
    pub created_at: String,
    /// Per-project Telegram visibility toggle. `None` or `Some(true)` = enabled
    /// (the project gets a forum topic on `/sync` and shows normally in the Atlas
    /// bot); `Some(false)` = disabled (sync skips/removes its topic, the bot marks
    /// it 🔕 but keeps it listed). Default ON preserves existing behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram: Option<bool>,
}

impl ManagedProject {
    /// Whether this project participates in Telegram (topic sync + Atlas display).
    /// Enabled unless the toggle was explicitly set to `false`.
    pub fn telegram_enabled(&self) -> bool {
        self.telegram != Some(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ManagedProject>,
}

impl ProjectRegistry {
    pub fn registry_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("projects.json")
    }

    pub fn load() -> Self {
        let path = Self::registry_path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json).context("writing projects.json")?;
        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<&ManagedProject> {
        let lower = name.to_lowercase();
        self.projects.iter().find(|p| p.name.to_lowercase() == lower)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&ManagedProject> {
        self.projects.iter().find(|p| p.path == path)
    }

    pub fn add(&mut self, project: ManagedProject) {
        if self.find_by_path(&project.path).is_none() {
            self.projects.push(project);
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let lower = name.to_lowercase();
        let before = self.projects.len();
        self.projects.retain(|p| p.name.to_lowercase() != lower);
        self.projects.len() < before
    }

    /// Set a project's Telegram toggle (topic sync + Atlas visibility). Returns
    /// `true` if a project with that name existed and was updated.
    pub fn set_telegram(&mut self, name: &str, enabled: bool) -> bool {
        let lower = name.to_lowercase();
        if let Some(p) = self
            .projects
            .iter_mut()
            .find(|p| p.name.to_lowercase() == lower)
        {
            p.telegram = Some(enabled);
            true
        } else {
            false
        }
    }
}

const SCAFFOLD_DIRS: &[&str] = &[
    "docs",
    "docs/FEATURES",
    ".planner",
    ".oracles",
    ".omega",
];

const CLAUDE_MD_TEMPLATE: &str = r#"# {name}

## Project

- **Path:** {path}
- **Created:** {date}

## Development Rules

- Law 1: Code lies. Only runtime tells the truth.
- Law 2: Researcher, not sycophant. Challenge flawed premises.
- Every feature must be verified with live runtime evidence before merge.
"#;

const VISION_TEMPLATE: &str = r#"# Vision — {name}

> Define the emotional foundation and product identity here.

## Internal Compass

_One sentence that acts as a decision filter for everything._

## Soul Statement

_What is this product, at its core?_

## Design Principles

1.
2.
3.

## Personas

### Primary

- **Name:**
- **Role:**
- **Pain:**
- **Goal:**

## What This Is NOT

-
"#;

const PRD_TEMPLATE: &str = r#"# PRD — {name}

> Product Requirements Document

## Overview

_What problem does this solve and for whom?_

## Goals

1.
2.
3.

## Features

| ID | Feature | Priority | Status |
|----|---------|----------|--------|
| F-001 | | P0 | planned |

## Technical Constraints

-

## Success Metrics

-
"#;

pub fn create_project(name: &str, location: &Path, icon: Option<&str>) -> Result<ManagedProject> {
    let project_path = location.join(name);
    std::fs::create_dir_all(&project_path)
        .with_context(|| format!("creating project dir at {}", project_path.display()))?;

    for dir in SCAFFOLD_DIRS {
        std::fs::create_dir_all(project_path.join(dir))?;
    }

    let date = chrono::Utc::now().to_rfc3339();

    let claude_md = CLAUDE_MD_TEMPLATE
        .replace("{name}", name)
        .replace("{path}", &project_path.to_string_lossy())
        .replace("{date}", &date);
    std::fs::write(project_path.join("CLAUDE.md"), claude_md)?;

    let vision = VISION_TEMPLATE.replace("{name}", name);
    std::fs::write(project_path.join("docs/VISION.md"), vision)?;

    let prd = PRD_TEMPLATE.replace("{name}", name);
    std::fs::write(project_path.join("docs/PRD.md"), prd)?;

    // git init if not already a repo
    if !project_path.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project_path)
            .output();
    }

    let project = ManagedProject {
        name: name.to_string(),
        path: project_path,
        icon: icon.map(|s| s.to_string()),
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: date,
        telegram: None,
    };

    // Persist to the registry so /projects sees it.
    let mut registry = ProjectRegistry::load();
    registry.projects.retain(|p| p.name != name);
    registry.projects.push(project.clone());
    registry.save()?;

    Ok(project)
}

pub fn add_existing_project(path: &Path) -> Result<ManagedProject> {
    anyhow::ensure!(path.exists(), "path does not exist: {}", path.display());
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project = ManagedProject {
        name: name.clone(),
        path: path.to_path_buf(),
        icon: None,
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        telegram: None,
    };

    // Persist to the registry so /projects sees it next time.
    let mut registry = ProjectRegistry::load();
    // De-dup: replace any existing entry with the same name OR path.
    let path_str = path.display().to_string();
    registry.projects.retain(|p| p.name != name && p.path.display().to_string() != path_str);
    registry.projects.push(project.clone());
    registry.save()?;

    Ok(project)
}

pub fn scan_directory(root: &Path) -> Vec<ManagedProject> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let is_project = p.join(".git").exists()
            || p.join("package.json").exists()
            || p.join("Cargo.toml").exists()
            || p.join("pyproject.toml").exists()
            || p.join("go.mod").exists();
        if is_project {
            found.push(ManagedProject {
                name: name.to_string(),
                path: p,
                icon: None,
                telegram_topic_id: None,
                oracle_session: None,
                git_email: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                telegram: None,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_register() {
        let tmp = std::env::temp_dir().join("omega-pm-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let project = create_project("TestApp", &tmp, Some("🚀")).unwrap();
        assert_eq!(project.name, "TestApp");
        assert!(project.path.join("CLAUDE.md").exists());
        assert!(project.path.join("docs/VISION.md").exists());
        assert!(project.path.join("docs/PRD.md").exists());
        assert!(project.path.join(".planner").exists());

        let mut registry = ProjectRegistry::default();
        registry.add(project.clone());
        assert_eq!(registry.projects.len(), 1);
        assert!(registry.find("testapp").is_some());

        // duplicate add is a no-op
        registry.add(project);
        assert_eq!(registry.projects.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn scan_finds_projects() {
        let tmp = std::env::temp_dir().join("omega-scan-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("project-a/.git")).unwrap();
        std::fs::create_dir_all(tmp.join("project-b")).unwrap();
        std::fs::write(tmp.join("project-b/package.json"), "{}").unwrap();
        std::fs::create_dir_all(tmp.join("not-a-project")).unwrap();

        let found = scan_directory(&tmp);
        assert_eq!(found.len(), 2);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
