//! Codex (OpenAI/ChatGPT) headless re-login — the twin of [`crate::oauth`] for
//! the Codex agent, shared by the CLI, the TUI and the Telegram bridge.
//!
//! # Why this is NOT a copy of the Claude flow
//!
//! Claude: `/login` prints an authorize URL, the operator authorizes, pastes a
//! code back, and that code finishes the exchange (`claude-login-code`).
//!
//! Codex: `codex login --device-auth` prints a STATIC url + a one-time code,
//! and then POLLS by itself until the operator approves in the browser. There
//! is nothing to paste back, so the second step is a STATUS QUERY, not a code
//! submission.
//!
//! # The hazard this module exists to contain (measured, 2026-07-17)
//!
//! `codex login --device-auth` DELETES `~/.codex/auth.json` the moment it
//! starts — BEFORE the operator has approved anything, and even if they never
//! do. Observed live: a working `Logged in using ChatGPT` became `Not logged
//! in` within seconds of launching the flow, with the file gone.
//!
//! That makes an unguarded "Codex login" button a trap: one distracted tap
//! from a phone, the flow is abandoned, and the operator is silently logged
//! out — taking the `/duo` binome down with it. So every start() backs the
//! credentials up first, and finish() restores them when the flow did not land.
//!
//! The `--device-auth` flag itself is UNDOCUMENTED in `codex login --help`
//! (the description is empty, codex v0.144.5). It is load-bearing here and can
//! break on a Codex upgrade — parse failures degrade to a clear error rather
//! than a silent half-login.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Where the device-code flow sends the operator. Static — only the code rotates.
pub const DEVICE_URL: &str = "https://auth.openai.com/codex/device";

fn codex_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".codex")
}
fn auth_path() -> PathBuf {
    codex_dir().join("auth.json")
}
fn omega_state() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".omega/state")
}
/// The pre-flow credential backup. Lives under ~/.omega/state (gitignored, and
/// NOT under ~/.codex where the flow itself reaches in).
fn backup_path() -> PathBuf {
    omega_state().join("codex-auth.backup.json")
}
fn log_path() -> PathBuf {
    omega_state().join("codex-login.log")
}

/// The device-code challenge to show the operator.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceLogin {
    pub url: String,
    pub code: String,
    /// PID of the waiting `codex login --device-auth` process.
    pub pid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum LoginStatus {
    /// `Logged in using ChatGPT` / `Logged in using an API key`.
    LoggedIn { mode: String },
    NotLoggedIn,
}

/// Strip ANSI SGR sequences — codex colorizes the code and the URL, so the raw
/// log is not directly parseable.
fn strip_ansi(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == 0x1b {
            // Skip up to and including the final byte of the escape sequence.
            i += 1;
            if i < b.len() && b[i] == b'[' {
                i += 1;
                while i < b.len() && !(b[i] as char).is_ascii_alphabetic() {
                    i += 1;
                }
            }
            i += 1;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

/// Pull the one-time code out of the flow's output. Codex renders it as
/// `XXXX-XXXXX` (measured: `B6BU-SV81P`) on its own line. Matched structurally
/// rather than by surrounding prose, so a copy tweak upstream does not break it.
fn parse_code(out: &str) -> Option<String> {
    for tok in strip_ansi(out).split_whitespace() {
        let t = tok.trim();
        let Some((a, b)) = t.split_once('-') else {
            continue;
        };
        let ok = |s: &str, n: usize| {
            s.len() == n && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        };
        if ok(a, 4) && ok(b, 5) {
            return Some(t.to_string());
        }
    }
    None
}

/// Back the live credentials up so an abandoned flow is recoverable.
///
/// A missing auth.json means we are ALREADY logged out — most likely because an
/// earlier flow ate it and never settled. Any existing backup is then the last
/// known-good credential and the operator's only way back, so it is deliberately
/// LEFT ALONE: clearing it here would destroy the very safety net this module
/// exists to hold. Returns whether a fresh backup was taken.
fn backup_credentials() -> Result<bool> {
    std::fs::create_dir_all(omega_state()).ok();
    if !auth_path().exists() {
        return Ok(false);
    }
    std::fs::copy(auth_path(), backup_path()).context("backing up ~/.codex/auth.json")?;
    Ok(true)
}

/// Put the pre-flow credentials back, restoring 0600 (the file holds live
/// OAuth tokens).
fn restore_credentials() -> Result<bool> {
    if !backup_path().exists() {
        return Ok(false);
    }
    std::fs::create_dir_all(codex_dir()).ok();
    std::fs::copy(backup_path(), auth_path()).context("restoring ~/.codex/auth.json")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(auth_path(), std::fs::Permissions::from_mode(0o600));
    }
    Ok(true)
}

/// `codex login status` — the single source of truth for whether we are in.
pub fn status() -> LoginStatus {
    let out = std::process::Command::new("codex")
        .args(["login", "status"])
        .output();
    let text = match out {
        Ok(o) => format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
        Err(_) => return LoginStatus::NotLoggedIn,
    };
    let clean = strip_ansi(&text);
    match clean.split_once("Logged in using") {
        Some((_, rest)) => LoginStatus::LoggedIn {
            mode: rest.trim().lines().next().unwrap_or("?").trim().to_string(),
        },
        None => LoginStatus::NotLoggedIn,
    }
}

/// Start the device-code flow: back the credentials up, spawn the waiting
/// `codex login --device-auth`, and return the URL + one-time code to show the
/// operator. The child KEEPS RUNNING after we return — it is what polls for the
/// approval — so the caller must later call [`finish`] to settle the outcome.
///
/// If the code never appears (codex changed its output, or the CLI is missing),
/// the flow is killed and the credentials are put straight back: we never leave
/// the operator logged out because of a parse failure.
pub fn start() -> Result<DeviceLogin> {
    backup_credentials()?;
    std::fs::create_dir_all(omega_state()).ok();
    let log = std::fs::File::create(log_path()).context("creating the codex-login log")?;
    let err_log = log.try_clone()?;

    let mut child = std::process::Command::new("codex")
        .args(["login", "--device-auth"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log))
        .stderr(std::process::Stdio::from(err_log))
        .spawn()
        .context("spawning `codex login --device-auth` (is the codex CLI on PATH?)")?;

    // The code lands within ~1s; allow generously for a slow box before giving up.
    let deadline = Instant::now() + Duration::from_secs(25);
    while Instant::now() < deadline {
        if let Ok(out) = std::fs::read_to_string(log_path()) {
            if let Some(code) = parse_code(&out) {
                return Ok(DeviceLogin {
                    url: DEVICE_URL.to_string(),
                    code,
                    pid: child.id(),
                });
            }
        }
        // A flow that exits before printing a code has failed outright.
        if matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    let _ = child.kill();
    let _ = child.wait();
    // Restore unconditionally, not just when WE took the backup: an earlier
    // flow may have left one behind, and it is still the best credential we
    // know of. No-ops when there is nothing to restore.
    restore_credentials()?;
    let tail = std::fs::read_to_string(log_path())
        .map(|s| strip_ansi(&s).trim().chars().rev().take(300).collect::<String>())
        .unwrap_or_default()
        .chars()
        .rev()
        .collect::<String>();
    anyhow::bail!(
        "codex never printed a device code — flow aborted and previous login restored. Output: {}",
        tail
    )
}

/// Settle a started flow. Logged in → the flow landed, drop the backup. Still
/// out → the operator abandoned it (or it expired), so put the pre-flow
/// credentials back and kill the waiting process.
///
/// Returns the status it settled on, plus whether a restore happened.
pub fn finish(pid: Option<u32>) -> (LoginStatus, bool) {
    let st = status();
    if let LoginStatus::LoggedIn { .. } = st {
        let _ = std::fs::remove_file(backup_path());
        return (st, false);
    }
    if let Some(p) = pid {
        // Only ever signal the pid the caller got from start().
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(p.to_string())
            .output();
    }
    let restored = restore_credentials().unwrap_or(false);
    let settled = status();
    // Once auth.json holds the credentials again the backup has done its job —
    // drop it rather than leave a second copy of live OAuth tokens on disk. Kept
    // untouched if the restore did NOT land, since it is then still the only way back.
    if matches!(settled, LoginStatus::LoggedIn { .. }) {
        let _ = std::fs::remove_file(backup_path());
    }
    (settled, restored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_the_sgr_codex_wraps_its_code_in() {
        assert_eq!(strip_ansi("\x1b[94mB6BU-SV81P\x1b[0m"), "B6BU-SV81P");
    }

    /// The real captured output (codex v0.144.5), colour codes and all.
    #[test]
    fn parses_the_code_from_real_device_auth_output() {
        let out = "Welcome to Codex [v\x1b[90m0.144.5\x1b[0m]\n\n\
                   1. Open this link in your browser and sign in to your account\n   \
                   \x1b[94mhttps://auth.openai.com/codex/device\x1b[0m\n\n\
                   2. Enter this one-time code \x1b[90m(expires in 15 minutes)\x1b[0m\n   \
                   \x1b[94mB6BU-SV81P\x1b[0m\n";
        assert_eq!(parse_code(out).as_deref(), Some("B6BU-SV81P"));
    }

    #[test]
    fn no_code_before_the_flow_prints_one() {
        assert_eq!(parse_code("Welcome to Codex\nFollow these steps:"), None);
    }

    /// The version banner (`v0.144.5`) and the URL must not be mistaken for a code.
    #[test]
    fn does_not_match_the_banner_or_the_url() {
        assert_eq!(parse_code("[v0.144.5] https://auth.openai.com/codex/device"), None);
    }
}
