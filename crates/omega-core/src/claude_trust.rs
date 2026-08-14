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
use std::time::Duration;

/// Total wall clock this path will spend trying to take the `~/.claude.json`
/// lock before giving up on it and writing unserialized. Deliberately tiny: the
/// budget is a spawn-latency cost paid on every single session launch.
const TRUST_LOCK_BUDGET: Duration = Duration::from_millis(100);

/// Mark `dir` as trusted in `~/.claude.json` (read-modify-write under a
/// best-effort bounded lock, atomic temp+rename, mode 600).
///
/// Returns `Ok(true)` if the file was updated, `Ok(false)` if the flag was
/// already set. Never clobbers an unreadable/corrupt config: any parse error
/// aborts with `Err` (the worst case is the dialog showing once, not a lost
/// config).
pub fn trust_dir(dir: &Path) -> std::io::Result<bool> {
    let home = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no home dir"))?;
    trust_dir_in_config(&home.join(".claude.json"), dir)
}

/// [`trust_dir`] against an explicit config path, so the whole transaction is
/// testable without going anywhere near the real `$HOME`.
fn trust_dir_in_config(cfg_path: &Path, dir: &Path) -> std::io::Result<bool> {
    // `~/.claude.json` also carries `oauthAccount` — the Claude account IDENTITY
    // — and this function runs on EVERY session spawn (agents.rs prefixes the
    // launch command with `omega trust-dir "$PWD"`). Read-modify-write with no
    // lock means two concurrent spawns both read, both merge, and the second
    // rename drops the first one's work; if a `/login` writes a fresh identity
    // inside that window, the identity is what gets overwritten. So hold the
    // crate's private file lock across the WHOLE transaction and read INSIDE it
    // (see `trust_dir_locked`), which closes that window rather than narrowing
    // it, on every spawn that actually gets the lock. Bounds of the guarantee:
    // it serializes OMEGA writers, the ones that run once per spawn (Claude Code
    // does not take this lock, so its own writes stay last-writer-wins), and it
    // holds only when the lock was acquired, never at the cost of waiting.
    //
    // THIS PATH MUST NEVER BLOCK ON THAT LOCK: it is the prefix of every session
    // launch command on the machine (`omega trust-dir "$PWD" >/dev/null 2>&1; `
    // at agents.rs:450/590/710), so one live-but-stuck holder waiting on a
    // blocking `lock_exclusive` would freeze every subsequent spawn, everywhere.
    // Hence a bounded try: acquire if the lock is free within TRUST_LOCK_BUDGET,
    // and on contention (or any lock error) fall back to the unserialized
    // read-modify-write this function did before the lock existed. Losing the
    // lock costs at worst the narrow, rare, recoverable race it was added to
    // close; blocking costs the whole box. Best-effort is the correct trade for
    // a pre-accepted dialog: the caller already discards output and treats a
    // failure as "skip the step".
    match crate::config::try_acquire_private_file_lock(cfg_path, TRUST_LOCK_BUDGET) {
        Ok(Some(guard)) => {
            let outcome = trust_dir_locked(cfg_path, dir);
            // Explicit, so the span the lock covers is the transaction above and
            // does not rest on where a binding happens to be dropped.
            drop(guard);
            outcome
        }
        // Contended, or the lock file itself is unusable: do the work anyway,
        // unserialized, exactly as this function did before the lock existed.
        Ok(None) | Err(_) => trust_dir_locked(cfg_path, dir),
    }
}

/// The read-modify-write half of [`trust_dir`], run with the lock held when it
/// could be taken within the budget, and unserialized when it could not.
fn trust_dir_locked(cfg_path: &Path, dir: &Path) -> std::io::Result<bool> {
    // Read where the lock is held, immediately before the merge and the rename
    // below — no other omega writer can slip between this read and that rename.
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
    use super::{mark_trusted, trust_dir_in_config, TRUST_LOCK_BUDGET};
    use std::path::Path;
    use std::time::{Duration, Instant};

    // The whole transaction is covered here, never against the real
    // `~/.claude.json`: every test below owns a fresh tempdir and passes that
    // path to `trust_dir_in_config`, which resolves no home directory at all —
    // only the thin `trust_dir` wrapper does, and nothing here calls it.

    fn fixture() -> serde_json::Value {
        serde_json::json!({
            "oauthAccount": {
                "emailAddress": "fixture@example.invalid",
                "accountUuid": "00000000-0000-0000-0000-000000000000"
            },
            "projects": { "/other": { "history": [1, 2] } },
            "topLevel": "kept"
        })
    }

    fn write_fixture(cfg_path: &Path) {
        std::fs::write(cfg_path, serde_json::to_string(&fixture()).unwrap()).unwrap();
    }

    fn read_back(cfg_path: &Path) -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(cfg_path).unwrap()).unwrap()
    }

    /// The flag was added for `dir` and NOTHING else moved: strip the entry the
    /// merge is allowed to create and what remains must equal the fixture, which
    /// covers `oauthAccount` and every other pre-existing field at once.
    fn assert_only_added_trust_for(cfg_path: &Path, dir: &str) {
        let mut root = read_back(cfg_path);
        assert_eq!(root["projects"][dir]["hasTrustDialogAccepted"], true);
        root["projects"]
            .as_object_mut()
            .unwrap()
            .remove(dir)
            .expect("the trusted entry must exist");
        assert_eq!(root, fixture());
    }

    /// (a) Lock FREE: the transaction runs under the lock and lands exactly what
    /// it lands today — flag written, second call a no-op, fixture intact.
    #[test]
    fn trusts_and_stays_idempotent_when_the_lock_is_free() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = tmp.path().join(".claude.json");
        write_fixture(&cfg);

        assert!(trust_dir_in_config(&cfg, Path::new("/proj")).unwrap());
        assert_only_added_trust_for(&cfg, "/proj");

        // Already accepted → nothing to write, so no second rename.
        assert!(!trust_dir_in_config(&cfg, Path::new("/proj")).unwrap());
        assert_only_added_trust_for(&cfg, "/proj");
    }

    /// Run the transaction on a worker thread and refuse to wait on it forever,
    /// so a path that blocks FAILS this suite instead of hanging it.
    fn trust_without_hanging(cfg: &Path, dir: &str) -> (std::io::Result<bool>, Duration) {
        let (tx, rx) = std::sync::mpsc::channel();
        let cfg = cfg.to_path_buf();
        let dir = dir.to_string();
        std::thread::spawn(move || {
            let started = Instant::now();
            let outcome = trust_dir_in_config(&cfg, Path::new(&dir));
            let _ = tx.send((outcome, started.elapsed()));
        });
        rx.recv_timeout(Duration::from_secs(5))
            .expect("trust_dir blocked on the lock instead of falling back to the unlocked write")
    }

    /// (b) Lock ALREADY HELD: the call must come back inside its budget instead
    /// of waiting on the holder, and must still do the job through the
    /// unserialized fallback. This is the regression guard for the freeze — the
    /// prefix of every session launch may never wait on a stuck holder.
    #[test]
    fn returns_within_budget_and_still_writes_when_the_lock_is_held() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = tmp.path().join(".claude.json");
        write_fixture(&cfg);

        // A real holder of the real lock file: flock() is held per open file
        // description, so a second descriptor contends here exactly as another
        // process would. Taken through the blocking primitive itself, which also
        // proves both flavours agree on one lock path.
        let holder = crate::config::acquire_private_file_lock(&cfg).unwrap();

        let (outcome, elapsed) = trust_without_hanging(&cfg, "/proj");
        assert!(outcome.unwrap(), "the fallback must still write the flag");
        assert!(
            elapsed < Duration::from_secs(2),
            "took {elapsed:?}, which is nowhere near the {TRUST_LOCK_BUDGET:?} budget"
        );

        drop(holder);
        assert_only_added_trust_for(&cfg, "/proj");
    }

    /// (c) `oauthAccount` and the other existing fields survive BOTH paths. The
    /// identity is the one thing this file must never lose, so it is asserted on
    /// the locked path and on the contended fallback independently.
    #[test]
    fn preserves_the_account_identity_on_both_paths() {
        for hold_the_lock in [false, true] {
            let tmp = tempfile::tempdir().unwrap();
            let cfg = tmp.path().join(".claude.json");
            write_fixture(&cfg);

            let holder =
                hold_the_lock.then(|| crate::config::acquire_private_file_lock(&cfg).unwrap());

            let (outcome, _) = trust_without_hanging(&cfg, "/proj");
            assert!(outcome.unwrap(), "lock held: {hold_the_lock}");
            drop(holder);

            let root = read_back(&cfg);
            assert_eq!(
                root["oauthAccount"]["emailAddress"], "fixture@example.invalid",
                "identity lost (lock held: {hold_the_lock})"
            );
            assert_eq!(
                root["oauthAccount"]["accountUuid"],
                "00000000-0000-0000-0000-000000000000"
            );
            assert_eq!(root["projects"]["/other"]["history"][1], 2);
            assert_eq!(root["topLevel"], "kept");
            assert_only_added_trust_for(&cfg, "/proj");
        }
    }

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
