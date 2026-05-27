//! Project auto-discovery — finds project containers and their children
//! on the user's machine without any hard-coded paths.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredProject {
    pub name: String,
    pub path: PathBuf,
    pub container: String,
    pub stack: Vec<String>,
}

/// Heuristic project-container names — directories that typically hold
/// many sub-projects. Case-insensitive match against directory names.
const CONTAINER_NAMES: &[&str] = &[
    "projects", "project", "code", "dev", "develop", "development",
    "work", "works", "repos", "repositories", "src", "sources",
    "vibecoding", "personal", "perso", "pro", "clients", "client",
    "freelance", "agentik",
];

/// Look at the file structure to identify what kind of project this is.
fn detect_stack(path: &std::path::Path) -> Vec<String> {
    let mut stack = Vec::new();
    if path.join("package.json").exists() {
        if path.join("next.config.js").exists() || path.join("next.config.ts").exists() {
            stack.push("Next.js".to_string());
        } else if path.join("vite.config.ts").exists() || path.join("vite.config.js").exists() {
            stack.push("Vite".to_string());
        } else {
            stack.push("Node".to_string());
        }
    }
    if path.join("Cargo.toml").exists() {
        stack.push("Rust".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        stack.push("Python".to_string());
    }
    if path.join("go.mod").exists() {
        stack.push("Go".to_string());
    }
    if path.join("Gemfile").exists() {
        stack.push("Ruby".to_string());
    }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        stack.push("Java".to_string());
    }
    if path.join(".git").exists() && stack.is_empty() {
        stack.push("Git repo".to_string());
    }
    stack
}

fn is_project_dir(path: &std::path::Path) -> bool {
    path.join(".git").exists()
        || path.join("package.json").exists()
        || path.join("Cargo.toml").exists()
        || path.join("pyproject.toml").exists()
        || path.join("go.mod").exists()
        || path.join("Gemfile").exists()
        || path.join("composer.json").exists()
        || path.join("pubspec.yaml").exists()
        || path.join("pom.xml").exists()
}

/// Discover all candidate project containers under `$HOME` and list their
/// projects. Skips hidden dirs, `node_modules`, `target`, etc.
pub fn discover(home: &std::path::Path) -> Vec<DiscoveredProject> {
    let mut found = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // First pass: find container directories (depth ≤ 2)
    let mut containers: Vec<PathBuf> = Vec::new();
    walk_for_containers(home, 0, 2, &mut containers, &mut visited);

    // Second pass: for each container, list its project children
    for container in containers {
        let container_name = container
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "?".to_string());

        if let Ok(entries) = std::fs::read_dir(&container) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let name = p.file_name().map(|n| n.to_string_lossy().to_string());
                let Some(name) = name else { continue };
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }

                if is_project_dir(&p) {
                    found.push(DiscoveredProject {
                        name,
                        container: container_name.clone(),
                        stack: detect_stack(&p),
                        path: p,
                    });
                } else {
                    // Could be a nested container — check one level deeper
                    if let Ok(grand) = std::fs::read_dir(&p) {
                        for g in grand.flatten() {
                            let gp = g.path();
                            if !gp.is_dir() { continue; }
                            let gname = gp.file_name().map(|n| n.to_string_lossy().to_string());
                            let Some(gname) = gname else { continue };
                            if gname.starts_with('.') { continue; }
                            if is_project_dir(&gp) {
                                found.push(DiscoveredProject {
                                    name: gname,
                                    container: format!("{}/{}", container_name, name),
                                    stack: detect_stack(&gp),
                                    path: gp,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // De-duplicate by path
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found.dedup_by(|a, b| a.path == b.path);

    found
}

fn walk_for_containers(
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<PathBuf>,
    visited: &mut std::collections::HashSet<PathBuf>,
) {
    if depth > max_depth {
        return;
    }
    let Ok(canon) = dir.canonicalize() else { return };
    if !visited.insert(canon.clone()) {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let name = p.file_name().map(|n| n.to_string_lossy().to_string());
        let Some(name) = name else { continue };
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        let lower = name.to_lowercase();
        if CONTAINER_NAMES.iter().any(|c| lower == *c || lower.contains(c)) {
            // Looks like a container — but verify it has at least one project child
            if has_project_children(&p, 2) {
                out.push(p.clone());
            }
        }

        // Recurse one level deeper
        walk_for_containers(&p, depth + 1, max_depth, out, visited);
    }
}

fn has_project_children(dir: &std::path::Path, min: usize) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    let mut count = 0;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() && is_project_dir(&p) {
            count += 1;
            if count >= min {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dir_returns_no_projects() {
        let tmp = std::env::temp_dir().join("omega-test-empty");
        let _ = std::fs::create_dir_all(&tmp);
        let projects = discover(&tmp);
        assert!(projects.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
