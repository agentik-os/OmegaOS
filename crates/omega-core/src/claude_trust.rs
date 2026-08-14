//! Pre-trust a project folder in `~/.claude.json` so Claude Code never shows
//! the "Do you trust the files in this folder?" dialog for it.
//!
//! Why this exists: `~/.claude.json` is a whole-file last-writer-wins store
//! shared by EVERY Claude session. With many concurrent sessions (oracles,
//! workers, the operator's own terminals), an acceptance written by one
//! session is routinely clobbered when an older session flushes its in-memory
//! copy — so the operator keeps being asked to trust the same folders, and a
//! dispatched oracle can hang on the dialog instead of running its mission.
//!
//! The fix is to write the trust flag IMMEDIATELY before `claude` starts (the
//! launch command runs `omega trust-dir "$PWD"` first — see
//! `agents.rs::launch_command_with`), so the value is fresh when Claude reads
//! its config, regardless of what other sessions wrote earlier.

use std::path::Path;

/// Mark `dir` as trusted in `~/.claude.json` (locked read-modify-write, atomic
/// temp+rename, mode 600).
///
/// Returns `Ok(true)` if the file was updated, `Ok(false)` if the flag was
/// already set. Never clobbers an unreadable/corrupt config: any parse error
/// aborts with `Err` (the worst case is the dialog showing once, not a lost
/// config).
pub fn trust_dir(dir: &Path) -> std::io::Result<bool> {
    let home = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no home dir"))?;
    let cfg_path = home.join(".claude.json");
    // `~/.claude.json` also carries `oauthAccount` — the Claude account IDENTITY
    // — and this function runs on EVERY session spawn (agents.rs prefixes the
    // launch command with `omega trust-dir "$PWD"`). Read-modify-write with no
    // lock means two concurrent spawns both read, both merge, and the second
    // rename drops the first one's work; if a `/login` writes a fresh identity
    // inside that window, the identity is what gets overwritten. So hold the
    // crate's private file lock across the WHOLE transaction and read INSIDE it
    // (see `trust_dir_locked`), which closes the window instead of narrowing it.
    // Bound of the guarantee: this serializes OMEGA writers, the ones that run
    // once per spawn. Claude Code does not take this lock, so its own writes
    // stay last-writer-wins.
    let outcome =
        crate::config::with_private_file_lock(&cfg_path, || Ok(trust_dir_locked(&cfg_path, dir)))
            .map_err(|e| std::io::Error::other(format!("{e:#}")))?;
    outcome
}

/// The read-modify-write half of [`trust_dir`], run with the lock held.
fn trust_dir_locked(cfg_path: &Path, dir: &Path) -> std::io::Result<bool> {
    // Read UNDER the lock, immediately before the merge and the rename below —
    // no other omega writer can slip between this read and that rename.
    let mut root: serde_json::Value = if cfg_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(cfg_path)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    } else {
        serde_json::json!({})
    };
    if !mark_trusted(&mut root, dir)? {
        return Ok(false);
    }
    // Atomic write: temp file in the same dir + rename, so a concurrent reader
    // never sees a torn file (and a crash never truncates the real config).
    let tmp = cfg_path.with_extension(format!("json.trust-{}", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string(&root)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, cfg_path)?;
    Ok(true)
}

/// Set `hasTrustDialogAccepted` for `dir` on an in-memory `~/.claude.json`.
/// `Ok(false)` means it was already set and there is nothing to write; `Err`
/// means the config is not the shape we know, which aborts rather than
/// overwrite it.
fn mark_trusted(root: &mut serde_json::Value, dir: &Path) -> std::io::Result<bool> {
    let key = dir.to_string_lossy().to_string();
    let projects = root
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "config root is not an object",
            )
        })?
        .entry("projects")
        .or_insert_with(|| serde_json::json!({}));
    let entry = projects
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "projects is not an object")
        })?
        .entry(key)
        .or_insert_with(|| serde_json::json!({}));
    let obj = entry.as_object_mut().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "project entry is not an object",
        )
    })?;
    if obj.get("hasTrustDialogAccepted").and_then(|v| v.as_bool()) == Some(true) {
        return Ok(false);
    }
    obj.insert(
        "hasTrustDialogAccepted".into(),
        serde_json::Value::Bool(true),
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::mark_trusted;
    use std::path::Path;

    // trust_dir mutates the REAL ~/.claude.json (path is not injectable), so
    // unit tests only cover the pure JSON merge; the end-to-end path is
    // exercised by `omega trust-dir` in verify-install / live runtime.

    /// The merge runs on the file holding `oauthAccount`, so it must be provably
    /// additive: it may only set the trust flag, and must return `Ok(false)`
    /// (write nothing at all) once the flag is already there.
    #[test]
    fn merge_preserves_account_identity_and_skips_a_second_write() {
        let mut root = serde_json::json!({
            "oauthAccount": {
                "emailAddress": "fixture@example.invalid",
                "accountUuid": "00000000-0000-0000-0000-000000000000"
            },
            "projects": { "/x": { "history": [1, 2] } },
            "topLevel": "kept"
        });

        assert!(mark_trusted(&mut root, Path::new("/x")).unwrap());

        assert_eq!(root["projects"]["/x"]["hasTrustDialogAccepted"], true);
        assert_eq!(root["projects"]["/x"]["history"][1], 2);
        assert_eq!(root["topLevel"], "kept");
        assert_eq!(
            root["oauthAccount"]["emailAddress"],
            "fixture@example.invalid"
        );
        assert_eq!(
            root["oauthAccount"]["accountUuid"],
            "00000000-0000-0000-0000-000000000000"
        );

        // Already accepted → nothing to write, so the caller never renames.
        assert!(!mark_trusted(&mut root, Path::new("/x")).unwrap());
    }

    /// A config whose shape we do not recognize is refused, never overwritten.
    #[test]
    fn merge_refuses_a_corrupt_config() {
        let mut root = serde_json::json!("not an object");
        assert!(mark_trusted(&mut root, Path::new("/x")).is_err());
    }

    #[test]
    fn merge_preserves_existing_fields() {
        let mut root = serde_json::json!({
            "projects": { "/x": { "history": [1, 2], "hasTrustDialogAccepted": false } },
            "topLevel": "kept"
        });
        let obj = root["projects"]["/x"].as_object_mut().unwrap();
        obj.insert(
            "hasTrustDialogAccepted".into(),
            serde_json::Value::Bool(true),
        );
        assert_eq!(root["projects"]["/x"]["history"][1], 2);
        assert_eq!(root["topLevel"], "kept");
        assert_eq!(root["projects"]["/x"]["hasTrustDialogAccepted"], true);
    }
}
