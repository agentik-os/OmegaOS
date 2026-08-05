//! Project CRUD — create, register, discover, persist projects.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedProject {
    pub name: String,
    pub path: PathBuf,
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
    /// Thematic category used to group the Projects tab. An explicit value wins;
    /// when unset it is derived per-machine from the project's own location. See
    /// `display_category`. Kept out of the serialized form when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl ManagedProject {
    /// Whether this project participates in Telegram (topic sync + Atlas display).
    /// Enabled unless the toggle was explicitly set to `false`.
    pub fn telegram_enabled(&self) -> bool {
        self.telegram != Some(false)
    }

    /// Thematic category for the Projects-tab grouping. An explicit `category`
    /// (non-empty) wins; otherwise it is derived from the project's own location
    /// on THIS machine — the first path component under the user's configured
    /// projects root (`OmegaConfig::projects_dir`). A project that sits directly
    /// under the root (no category folder) or outside it is `"Other"`.
    ///
    /// Machine-agnostic by design: it mirrors whatever top-level folders the user
    /// organizes their projects into (customers/ side-business/ tools/ …, or any
    /// custom layout), never a hardcoded taxonomy. Pass the root from
    /// `OmegaConfig::projects_dir`.
    pub fn display_category(&self, projects_root: &Path) -> String {
        if let Some(c) = self.category.as_deref() {
            let c = c.trim();
            if !c.is_empty() {
                return c.to_string();
            }
        }
        // The first folder under the projects root, but only when the project
        // actually lives INSIDE a category folder (≥2 remaining components:
        // <category>/<project>[/…]). A bare child of the root has no category.
        if let Ok(rel) = self.path.strip_prefix(projects_root) {
            let mut comps = rel.components();
            if let Some(first) = comps.next() {
                if comps.next().is_some() {
                    return first.as_os_str().to_string_lossy().to_string();
                }
            }
        }
        "Other".to_string()
    }

    /// Sort key for the Projects-tab category sections: named categories
    /// alphabetically (case-insensitive), `"Other"` always last. Deliberately
    /// order-agnostic so it fits any user's folder taxonomy.
    pub fn category_rank(cat: &str) -> (u8, String) {
        if cat == "Other" {
            (1, String::new())
        } else {
            (0, cat.to_lowercase())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ManagedProject>,
    /// True when this registry came from a file that exists but could not be
    /// read or parsed. Such a registry is empty-looking but MUST NOT be saved:
    /// writing it back would erase the user's real project list. Never
    /// serialized — it is a property of this load, not of the data.
    #[serde(skip)]
    pub poisoned: bool,
}

impl ProjectRegistry {
    pub fn registry_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("projects.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::registry_path())
    }

    /// Load the registry, treating "file is absent" and "file is unreadable"
    /// as DIFFERENT things.
    ///
    /// This used to be `.ok().and_then(parse).unwrap_or_default()`, which
    /// collapsed both into an empty registry. Callers then add one project and
    /// `save()` — so a single transient read failure silently replaced every
    /// project the user had with one entry. That is exactly what happened on
    /// 2026-07-29: a registry of 25 projects became 1.
    ///
    /// Absent → empty is correct (first run). Present-but-unreadable → we
    /// preserve the file, shout on stderr, and return a POISONED registry that
    /// `save()` refuses to persist, so the damage cannot be written back.
    pub fn load_from(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(mut reg) => {
                    reg.poisoned = false;
                    reg
                }
                Err(e) => {
                    let backup = path.with_extension("json.unreadable");
                    let _ = std::fs::copy(path, &backup);
                    eprintln!(
                        "omega: projects.json could not be parsed ({e}). Your project list is \
                         NOT lost — the file is preserved at {}. Refusing to overwrite it.",
                        backup.display()
                    );
                    Self { projects: Vec::new(), poisoned: true }
                }
            },
            Err(e) => {
                eprintln!(
                    "omega: projects.json could not be read ({e}). Refusing to overwrite it."
                );
                Self { projects: Vec::new(), poisoned: true }
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::registry_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        // A registry we failed to read is never written back. Persisting it
        // would turn a transient read error into permanent data loss.
        anyhow::ensure!(
            !self.poisoned,
            "refusing to save a projects registry that failed to load — \
             the existing file is preserved"
        );
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        // Write-then-rename: a concurrent reader must never observe a
        // half-written file. A torn read is what poisons a load in the first
        // place, and rename(2) is atomic on the same filesystem.
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json).context("writing projects.json")?;
        std::fs::rename(&tmp, path).context("replacing projects.json")?;
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

/// Scaffold a project on disk and describe it — WITHOUT touching the registry.
///
/// Split out from `create_project` so the scaffolding can be tested without
/// writing the machine's real `~/.omega/projects.json`. The old test called
/// `create_project` directly and rewrote the operator's registry every time the
/// suite ran; on 2026-07-29 that replaced 25 projects with one.
pub fn scaffold_project(name: &str, location: &Path) -> Result<ManagedProject> {
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
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: date,
        telegram: None,
        category: None,
    };

    Ok(project)
}

/// Scaffold a project AND register it, so `/projects` sees it. This is the
/// production entry point; tests use `scaffold_project` instead.
pub fn create_project(name: &str, location: &Path) -> Result<ManagedProject> {
    let project = scaffold_project(name, location)?;
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
        telegram_topic_id: None,
        oracle_session: Some(format!("oracle-{}", name)),
        git_email: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        telegram: None,
        category: None,
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
                        telegram_topic_id: None,
                oracle_session: None,
                git_email: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                telegram: None,
                category: None,
            });
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: this test used to call `create_project()`, which persists to the
    // REAL ~/.omega/projects.json. Running the suite therefore rewrote the
    // machine's own project registry — on 2026-07-29 it replaced 25 projects
    // with a single "TestApp". It now exercises the scaffolding and the
    // registry separately, and never touches the user's registry.
    #[test]
    fn scaffolding_creates_the_expected_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();
        assert_eq!(project.name, "TestApp");
        assert!(project.path.join("CLAUDE.md").exists());
        assert!(project.path.join("docs/VISION.md").exists());
        assert!(project.path.join("docs/PRD.md").exists());
        assert!(project.path.join(".planner").exists());
    }

    #[test]
    fn registry_add_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let project = scaffold_project("TestApp", tmp.path()).unwrap();

        let mut registry = ProjectRegistry::default();
        registry.add(project.clone());
        assert_eq!(registry.projects.len(), 1);
        assert!(registry.find("testapp").is_some());

        registry.add(project);
        assert_eq!(registry.projects.len(), 1, "duplicate add is a no-op");
    }

    /// The data-loss guard: a registry that failed to load must never be
    /// written back over the file it failed to read.
    #[test]
    fn an_unreadable_registry_is_never_saved_over() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        std::fs::write(&path, "{ this is not json").unwrap();

        let reg = ProjectRegistry::load_from(&path);
        assert!(reg.poisoned, "an unparseable registry must be marked poisoned");
        assert!(reg.projects.is_empty());
        assert!(
            reg.save_to(&path).is_err(),
            "saving a poisoned registry would erase the user's projects"
        );
        // The original bytes survive, and a copy is preserved for recovery.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ this is not json");
        assert!(path.with_extension("json.unreadable").exists());
    }

    /// …but a genuinely absent registry is just a first run.
    #[test]
    fn an_absent_registry_is_a_clean_start() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let reg = ProjectRegistry::load_from(&path);
        assert!(!reg.poisoned);
        assert!(reg.save_to(&path).is_ok(), "a first run must be able to save");
    }

    /// A save must be atomic — a concurrent reader seeing a half-written file
    /// is what poisons a load and starts the whole failure chain.
    #[test]
    fn save_round_trips_and_leaves_no_temp_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("projects.json");
        let scaffold_dir = tempfile::tempdir().unwrap();
        let mut reg = ProjectRegistry::default();
        reg.add(scaffold_project("A", scaffold_dir.path()).unwrap());
        reg.save_to(&path).unwrap();

        let back = ProjectRegistry::load_from(&path);
        assert_eq!(back.projects.len(), 1);
        assert!(!back.poisoned);
        assert!(!path.with_extension("json.tmp").exists(), "temp file must be renamed away");
    }

    #[test]
    fn telegram_toggle_roundtrip() {
        // Absent field → None → enabled (default ON, backward-compatible).
        let legacy = r#"{"projects":[{"name":"A","path":"/p/a","telegram_topic_id":null,"oracle_session":null,"git_email":null,"created_at":"x"}]}"#;
        let reg: ProjectRegistry = serde_json::from_str(legacy).unwrap();
        assert!(reg.find("A").unwrap().telegram_enabled(), "absent telegram must default to enabled");

        // Set OFF, serialize, and confirm it persists as `false` across a round-trip.
        let mut reg2 = reg.clone();
        assert!(reg2.set_telegram("a", false));
        assert!(!reg2.find("A").unwrap().telegram_enabled());
        let json = serde_json::to_string(&reg2).unwrap();
        assert!(json.contains("\"telegram\":false"), "OFF must serialize: {json}");
        let reg3: ProjectRegistry = serde_json::from_str(&json).unwrap();
        assert!(!reg3.find("A").unwrap().telegram_enabled(), "OFF must survive round-trip");

        // Flip back ON → enabled again, and the field is no longer `false`.
        let mut reg4 = reg3.clone();
        assert!(reg4.set_telegram("a", true));
        assert!(reg4.find("A").unwrap().telegram_enabled());
        assert!(!serde_json::to_string(&reg4).unwrap().contains("\"telegram\":false"));
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

    fn proj(path: &str, category: Option<&str>) -> ManagedProject {
        ManagedProject {
            name: "x".into(),
            path: PathBuf::from(path),
                telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: "t".into(),
            telegram: None,
            category: category.map(|s| s.into()),
        }
    }

    #[test]
    fn category_is_derived_from_the_users_own_layout() {
        // Machine-agnostic: the category is the first folder under the user's
        // configured projects root — whatever it is named — NOT a fixed taxonomy.
        let root = Path::new("/home/alice/dev");
        assert_eq!(proj("/home/alice/dev/customers/acme", None).display_category(root), "customers");
        assert_eq!(proj("/home/alice/dev/tools/mylib/crate", None).display_category(root), "tools");
        // A different user with a different root and different folder names.
        let root2 = Path::new("/Users/bob/Code");
        assert_eq!(proj("/Users/bob/Code/Clients/foo", None).display_category(root2), "Clients");
    }

    #[test]
    fn category_falls_back_to_other() {
        let root = Path::new("/home/alice/dev");
        // Directly under the root (no category folder) → Other.
        assert_eq!(proj("/home/alice/dev/loose-project", None).display_category(root), "Other");
        // Outside the root entirely → Other.
        assert_eq!(proj("/somewhere/else/proj", None).display_category(root), "Other");
    }

    #[test]
    fn explicit_category_overrides_derivation() {
        let root = Path::new("/home/alice/dev");
        // Even though the path would derive "side-business", the pin wins.
        assert_eq!(
            proj("/home/alice/dev/side-business/omega", Some("Framework")).display_category(root),
            "Framework"
        );
        // Blank/whitespace category is ignored → falls back to derivation.
        assert_eq!(proj("/home/alice/dev/tools/x", Some("  ")).display_category(root), "tools");
    }

    #[test]
    fn category_rank_sorts_named_alpha_other_last() {
        let mut cats = vec!["Partners", "Other", "framework", "Nova"];
        cats.sort_by_key(|c| ManagedProject::category_rank(c));
        assert_eq!(cats, vec!["framework", "Nova", "Partners", "Other"]);
    }
}
