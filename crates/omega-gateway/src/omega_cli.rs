//! Generic subprocess wrapper around the `omega` CLI binary.
//!
//! This module mirrors the SHAPE of `rmux.rs` (argv-only `Command::new`, an
//! env-var override for the binary path) but deliberately deviates from its
//! error-handling: `rmux::run()` treats a non-zero exit as an `Err`, but for
//! `omega` a non-zero exit (e.g. "unknown project") is a NORMAL outcome the
//! caller inspects via `CommandOutput::success` — never an error. Only a
//! genuine spawn failure (binary missing/not executable) returns `Err`.
//!
//! This module has no route handlers and no knowledge of dispatch/oracle
//! semantics — it is a thin "run the omega binary with these args, hand back
//! stdout/stderr/success" primitive that Tasks 7 and 8 build typed logic on
//! top of.

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Resolves the `omega` binary path: `OMEGA_BIN` env override, else
/// `~/.local/bin/omega` (confirmed via `which omega` to match the real
/// installed path on this box, matching every other OmegaOS component's
/// PATH convention — same shape as `rmux::rmux_bin()`).
pub fn omega_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir().expect("no home dir").join(".local/bin/omega")
}

/// Captured output of an `omega` subprocess invocation.
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Runs `omega <args>` and captures stdout/stderr separately. A non-zero
/// exit is a normal outcome reflected in `CommandOutput::success`, never an
/// `Err` — only a spawn failure (binary missing/not executable) errors.
pub fn run(args: &[&str]) -> Result<CommandOutput> {
    let out = Command::new(omega_bin()).args(args).output()?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        success: out.status.success(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both tests mutate the process-global OMEGA_BIN env var, so they must
    // never run concurrently with each other. Acquire this lock at the start
    // of each test to serialize them regardless of the test harness's thread
    // count (same pattern as tests/sessions_test.rs's OMEGA_RMUX_BIN LOCK).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Writes an executable fake `omega` script and points OMEGA_BIN at it.
    fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("omega");
        std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::env::set_var("OMEGA_BIN", &path);
    }

    #[test]
    fn success_captures_stdout() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(
            dir.path(),
            r#"printf 'oracle-Verba-1\nworker-a\n'; printf 'note: nothing wrong' >&2; exit 0"#,
        );
        let out = run(&["projects", "--json"]).unwrap();
        assert_eq!(out.stdout, "oracle-Verba-1\nworker-a\n");
        assert!(out.success);
    }

    #[test]
    fn nonzero_exit_is_not_an_error() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        install_fake_omega(dir.path(), "echo 'unknown project' >&2; exit 1");
        let out = run(&["projects", "--json"]).unwrap();
        assert!(!out.success);
        assert!(out.stderr.contains("unknown project"));
    }
}
