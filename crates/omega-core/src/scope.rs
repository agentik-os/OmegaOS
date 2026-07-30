use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use fs2::FileExt;
use globset::Glob;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeClaim {
    pub session: String,
    pub files_owned: Vec<String>,
    pub claimed_at: DateTime<Utc>,
}

impl ScopeClaim {
    pub fn new(session: &str, files: Vec<String>) -> Self {
        let mut files_owned: Vec<String> = files
            .into_iter()
            .map(|file| normalize_scope_selector(&file))
            .filter(|file| !file.is_empty())
            .collect();
        files_owned.sort();
        files_owned.dedup();
        Self {
            session: session.to_string(),
            files_owned,
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
    let requested: Vec<String> = files
        .iter()
        .map(|file| normalize_scope_selector(file))
        .filter(|file| !file.is_empty())
        .collect();
    let mut conflicts = Vec::new();

    for claim in &existing {
        if claim.session == session {
            continue;
        }
        let claimed: Vec<String> = claim
            .files_owned
            .iter()
            .map(|file| normalize_scope_selector(file))
            .collect();
        let mut overlap = Vec::new();
        for request in &requested {
            for owner in &claimed {
                if selectors_overlap(request, owner) {
                    overlap.push(format!("{request} <-> {owner}"));
                }
            }
        }
        overlap.sort();
        overlap.dedup();
        if !overlap.is_empty() {
            conflicts.push(ScopeConflict {
                blocking_session: claim.session.clone(),
                overlapping_files: overlap,
            });
        }
    }

    Ok(conflicts)
}

pub fn claim_or_reject(state_dir: &Path, session: &str, files: Vec<String>) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(state_dir)?;

    // Serialize the check-then-write critical section behind an exclusive
    // advisory file lock to close the TOCTOU race: without it two concurrent
    // claimers can both pass check_conflicts then both write overlapping claims.
    // The lock auto-releases when `lockfile` drops at the end of this function.
    let lockfile = File::create(state_dir.join(".scope.lock"))?;
    lockfile.lock_exclusive()?;

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

/// Lexically normalize a scope selector without touching the filesystem.
///
/// This makes aliases such as `./src/lib.rs`, `src//lib.rs`, and
/// `src/./lib.rs` compare identically while preserving glob metacharacters.
/// Leading `..` components are retained so a caller cannot smuggle an
/// out-of-scope path into the same identifier as an in-tree path.
pub fn normalize_scope_selector(raw: &str) -> String {
    let slash_normalized = raw.trim().replace('\\', "/");
    if slash_normalized.is_empty() {
        return String::new();
    }

    let absolute = slash_normalized.starts_with('/');
    let mut parts: Vec<String> = Vec::new();
    for component in Path::new(&slash_normalized).components() {
        match component {
            Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().to_string())
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                if parts.last().is_some_and(|part| part != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..".to_string());
                }
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

/// Conservative selector intersection.
///
/// Exact paths conflict when one is the other or one is a directory parent.
/// Literal/glob pairs use `globset`; two globs use their static prefixes. The
/// latter intentionally prefers a false conflict over allowing two writers.
pub fn selectors_overlap(left: &str, right: &str) -> bool {
    let left = normalize_scope_selector(left);
    let right = normalize_scope_selector(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right || path_contains(&left, &right) || path_contains(&right, &left) {
        return true;
    }

    let left_glob = has_glob_meta(&left);
    let right_glob = has_glob_meta(&right);
    match (left_glob, right_glob) {
        (true, false) => {
            glob_matches(&left, &right) || path_contains(&right, &static_glob_prefix(&left))
        }
        (false, true) => {
            glob_matches(&right, &left) || path_contains(&left, &static_glob_prefix(&right))
        }
        (true, true) => {
            let left_prefix = static_glob_prefix(&left);
            let right_prefix = static_glob_prefix(&right);
            left_prefix.is_empty()
                || right_prefix.is_empty()
                || path_contains(&left_prefix, &right_prefix)
                || path_contains(&right_prefix, &left_prefix)
        }
        (false, false) => false,
    }
}

fn path_contains(parent: &str, child: &str) -> bool {
    !parent.is_empty()
        && (parent == child
            || child
                .strip_prefix(parent)
                .is_some_and(|suffix| suffix.starts_with('/')))
}

fn has_glob_meta(selector: &str) -> bool {
    selector
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b'{'))
}

fn static_glob_prefix(selector: &str) -> String {
    let meta = selector
        .char_indices()
        .find_map(|(index, ch)| "*?[{".contains(ch).then_some(index))
        .unwrap_or(selector.len());
    selector[..meta].trim_end_matches('/').to_string()
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    Glob::new(pattern)
        .map(|glob| glob.compile_matcher().is_match(candidate))
        .unwrap_or(true)
}

/// Durable scope lease data carried alongside a ledger lease.
///
/// The `fencing_token` must be compared against the ledger's current token
/// before every mutation. A lease id or owner name alone is insufficient
/// because an expired owner can reacquire the same resource (the ABA case).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FencedScopeLease {
    pub lease_id: String,
    pub resource: String,
    pub owner: String,
    pub selectors: Vec<String>,
    pub fencing_token: u64,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl FencedScopeLease {
    pub fn new(
        lease_id: impl Into<String>,
        resource: impl Into<String>,
        owner: impl Into<String>,
        selectors: Vec<String>,
        fencing_token: u64,
        ttl: Duration,
    ) -> Self {
        let acquired_at = Utc::now();
        let mut selectors: Vec<String> = selectors
            .into_iter()
            .map(|selector| normalize_scope_selector(&selector))
            .filter(|selector| !selector.is_empty())
            .collect();
        selectors.sort();
        selectors.dedup();
        Self {
            lease_id: lease_id.into(),
            resource: resource.into(),
            owner: owner.into(),
            selectors,
            fencing_token,
            acquired_at,
            expires_at: acquired_at + ttl,
        }
    }

    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at > now
    }

    pub fn accepts_fence(&self, supplied_token: u64, now: DateTime<Utc>) -> bool {
        self.is_active_at(now) && supplied_token == self.fencing_token
    }
}

#[cfg(test)]
mod scope_v3_tests {
    use super::*;

    #[test]
    fn lexical_aliases_normalize_to_one_selector() {
        assert_eq!(
            normalize_scope_selector("./crates//omega-core/src/./lib.rs"),
            "crates/omega-core/src/lib.rs"
        );
        assert!(selectors_overlap("./src/lib.rs", "src/lib.rs"));
    }

    #[test]
    fn directory_parent_conflicts_with_child() {
        assert!(selectors_overlap(
            "crates/omega-core",
            "crates/omega-core/src/lib.rs"
        ));
        assert!(!selectors_overlap(
            "crates/omega",
            "crates/omega-core/src/lib.rs"
        ));
    }

    #[test]
    fn glob_conflicts_with_matching_file_and_directory() {
        assert!(selectors_overlap(
            "crates/*/src/**",
            "crates/omega-core/src/lib.rs"
        ));
        assert!(selectors_overlap(
            "crates/omega-core/**",
            "crates/omega-core"
        ));
        assert!(!selectors_overlap(
            "docs/**/*.md",
            "crates/omega-core/src/lib.rs"
        ));
    }

    #[test]
    fn old_and_new_claim_aliases_conflict() {
        let dir = tempfile::tempdir().unwrap();
        ScopeClaim {
            session: "owner".to_string(),
            files_owned: vec!["./src/**".to_string()],
            claimed_at: Utc::now(),
        }
        .write(dir.path())
        .unwrap();
        let conflicts =
            check_conflicts(dir.path(), "challenger", &["src/lib.rs".to_string()]).unwrap();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn expired_or_old_fence_is_rejected() {
        let lease = FencedScopeLease::new(
            "lease-1",
            "repo:scope",
            "worker-a",
            vec!["src/**".to_string()],
            42,
            Duration::seconds(5),
        );
        assert!(!lease.accepts_fence(41, Utc::now()));
        assert!(lease.accepts_fence(42, Utc::now()));
        assert!(!lease.accepts_fence(42, lease.expires_at));
    }
}
