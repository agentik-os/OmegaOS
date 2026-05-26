use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeClaim {
    pub session: String,
    pub files_owned: Vec<String>,
    pub claimed_at: DateTime<Utc>,
}

impl ScopeClaim {
    pub fn new(session: &str, files: Vec<String>) -> Self {
        Self {
            session: session.to_string(),
            files_owned: files,
            claimed_at: Utc::now(),
        }
    }

    pub fn write(&self, state_dir: &Path) -> Result<()> {
        let path = state_dir.join(format!("scope-{}.json", self.session));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn read(state_dir: &Path, session: &str) -> Option<Self> {
        let path = state_dir.join(format!("scope-{}.json", session));
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn release(state_dir: &Path, session: &str) -> Result<()> {
        let path = state_dir.join(format!("scope-{}.json", session));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut claims = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("scope-") && name.ends_with(".json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(claim) = serde_json::from_str::<ScopeClaim>(&content) {
                                claims.push(claim);
                            }
                        }
                    }
                }
            }
        }
        claims
    }
}

pub fn check_conflicts(
    state_dir: &Path,
    session: &str,
    files: &[String],
) -> Result<Vec<ScopeConflict>> {
    let existing = ScopeClaim::read_all(state_dir);
    let requested: HashSet<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut conflicts = Vec::new();

    for claim in &existing {
        if claim.session == session {
            continue;
        }
        let claimed: HashSet<&str> = claim.files_owned.iter().map(|s| s.as_str()).collect();
        let overlap: Vec<String> = requested
            .intersection(&claimed)
            .map(|s| s.to_string())
            .collect();
        if !overlap.is_empty() {
            conflicts.push(ScopeConflict {
                blocking_session: claim.session.clone(),
                overlapping_files: overlap,
            });
        }
    }

    Ok(conflicts)
}

pub fn claim_or_reject(
    state_dir: &Path,
    session: &str,
    files: Vec<String>,
) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    let conflicts = check_conflicts(state_dir, session, &files)?;
    if !conflicts.is_empty() {
        let details: Vec<String> = conflicts
            .iter()
            .map(|c| {
                format!(
                    "  {} owns: {}",
                    c.blocking_session,
                    c.overlapping_files.join(", ")
                )
            })
            .collect();
        bail!(
            "Scope conflict — files already claimed:\n{}",
            details.join("\n")
        );
    }

    let claim = ScopeClaim::new(session, files);
    claim.write(state_dir)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ScopeConflict {
    pub blocking_session: String,
    pub overlapping_files: Vec<String>,
}
