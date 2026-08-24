//! Worker spawn contract: project cwd + record-only Verify Command.
//!
//! Live hole (Gareth 2026-08-24): `omega spawn-worker` used `dir.unwrap_or(".")`.
//! rmux treats `.` as the *daemon* cwd (often `$HOME`), so Claude wrote
//! `/Users/hacker/CLAUDE_OK.txt` instead of the project file. Always pass an
//! absolute existing directory.
//!
//! Second hole: the parent eval'd `Verify Command:` at spawn
//! (`(eval):1: no such file or directory: CLAUDE_OK.txt`). The contract is
//! recorded for later verification. It is never executed at spawn.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::session::expand_user_path;

/// Resolve the directory a worker pane must start in.
///
/// `--dir` wins (tilde-expanded). Otherwise the oracle's persisted
/// `working_dir`. Never return a relative `.` for rmux.
pub fn resolve_worker_working_dir(
    dir_flag: Option<&str>,
    oracle_working_dir: Option<&Path>,
    process_cwd: &Path,
) -> Result<PathBuf> {
    let dir_flag = dir_flag.filter(|raw| {
        let trimmed = raw.trim();
        trimmed != "." && trimmed != "./"
    });
    let candidate = match dir_flag {
        Some(raw) => {
            let expanded = expand_user_path(raw);
            if expanded.is_absolute() {
                expanded
            } else {
                process_cwd.join(expanded)
            }
        }
        None => match oracle_working_dir {
            Some(oracle_dir)
                if !oracle_dir.as_os_str().is_empty() && oracle_dir.as_os_str() != "." =>
            {
                if oracle_dir.is_absolute() {
                    oracle_dir.to_path_buf()
                } else {
                    process_cwd.join(oracle_dir)
                }
            }
            _ => process_cwd.to_path_buf(),
        },
    };
    let canon = candidate.canonicalize().map_err(|error| {
        anyhow::anyhow!(
            "worker working_dir '{}' does not exist (resolved: {}): {error}. \
             Workers must start in the project --dir / oracle working_dir.",
            dir_flag.unwrap_or("<oracle working_dir or process cwd>"),
            candidate.display()
        )
    })?;
    if !canon.is_dir() {
        bail!("worker working_dir {} is not a directory", canon.display());
    }
    Ok(canon)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifySpec {
    FileExists { path: String },
    Command { argv: Vec<String> },
}

fn extract_verify_line(prompt: &str) -> Option<String> {
    let lines: Vec<&str> = prompt.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let Some(marker) = lower
            .find("verify command:")
            .or_else(|| lower.find("verify-command:"))
        else {
            continue;
        };
        let Some(colon) = line[marker..].find(':').map(|offset| marker + offset) else {
            continue;
        };
        let mut command = line[colon + 1..].trim();
        if command.is_empty() {
            command = lines
                .iter()
                .skip(index + 1)
                .map(|candidate| candidate.trim())
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with("```"))?;
        }
        command = command
            .trim_start_matches("- ")
            .trim()
            .trim_matches('`')
            .trim();
        if !command.is_empty() {
            return Some(command.to_string());
        }
    }
    None
}

fn looks_like_artifact_path(token: &str) -> bool {
    if token.contains('/') || token.contains('\\') {
        return true;
    }
    Path::new(token).extension().is_some()
}

/// Parse the oracle-authored Verify Command. Never execute it.
pub fn parse_verify_contract(prompt: &str) -> Option<VerifySpec> {
    let command = extract_verify_line(prompt)?;
    if command
        .chars()
        .any(|ch| matches!(ch, ';' | '&' | '|' | '<' | '>' | '`' | '$' | '\n' | '\r'))
    {
        return None;
    }
    let argv: Vec<String> = command
        .split_whitespace()
        .map(str::to_string)
        .filter(|part| !part.is_empty())
        .collect();
    if argv.is_empty() {
        return None;
    }
    if argv.len() == 1 && looks_like_artifact_path(&argv[0]) {
        return Some(VerifySpec::FileExists {
            path: argv[0].clone(),
        });
    }
    Some(VerifySpec::Command { argv })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_dir_uses_oracle_project_not_dot() {
        let project = tempfile::TempDir::new().unwrap();
        let home = tempfile::TempDir::new().unwrap();
        let got = resolve_worker_working_dir(None, Some(project.path()), home.path()).unwrap();
        assert_eq!(got, project.path().canonicalize().unwrap());
        assert_ne!(
            got,
            home.path().canonicalize().unwrap(),
            "a worker must not inherit $HOME because rmux `.` is the daemon cwd"
        );
    }

    #[test]
    fn relative_dot_dir_prefers_oracle_project() {
        let project = tempfile::TempDir::new().unwrap();
        let home = tempfile::TempDir::new().unwrap();
        let got = resolve_worker_working_dir(Some("."), Some(project.path()), home.path()).unwrap();
        assert_eq!(got, project.path().canonicalize().unwrap());
        let no_oracle = resolve_worker_working_dir(Some("."), None, project.path()).unwrap();
        assert_eq!(no_oracle, project.path().canonicalize().unwrap());
    }

    #[test]
    fn missing_dir_is_a_hard_error() {
        let cwd = tempfile::TempDir::new().unwrap();
        let err = resolve_worker_working_dir(Some("no-such-worker-dir"), None, cwd.path())
            .expect_err("missing --dir must fail before spawn");
        assert!(err.to_string().contains("does not exist"), "{err}");
    }

    #[test]
    fn bare_artifact_verify_is_file_exists_not_a_shell_command() {
        let spec = parse_verify_contract(
            "Write CLAUDE_OK.txt\nDone Criteria: file exists\nVerify Command: CLAUDE_OK.txt",
        )
        .unwrap();
        assert_eq!(
            spec,
            VerifySpec::FileExists {
                path: "CLAUDE_OK.txt".into()
            }
        );
    }

    #[test]
    fn runtime_check_verify_is_recorded_argv() {
        let spec =
            parse_verify_contract("Done Criteria: green\nVerify Command: test -f CLAUDE_OK.txt")
                .unwrap();
        assert_eq!(
            spec,
            VerifySpec::Command {
                argv: vec!["test".into(), "-f".into(), "CLAUDE_OK.txt".into()]
            }
        );
    }

    #[test]
    fn shell_operators_are_refused_not_evald() {
        assert!(
            parse_verify_contract("Verify Command: test -f CLAUDE_OK.txt && echo ok").is_none()
        );
    }
}
