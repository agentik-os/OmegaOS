//! Pre-reset backup of the irreproducible OmegaOS state.
//!
//! What this DOES back up: `~/.omega/` (secrets, credentials, provisioning
//! tokens, config, projects registry) + the user's crontab. Optionally the
//! claude-mem memory store (`~/.claude/projects`, opt-in — it is large).
//!
//! What this DELIBERATELY does NOT touch: the user's project repositories. Those
//! live in the user's own git / GitHub and are never bundled into anything
//! OmegaOS-owned. `omega backup` only *reports* which repos have uncommitted or
//! unpushed work so the user pushes them to their OWN remotes — keeping personal
//! projects strictly separate from OmegaOS. This separation is intentional:
//! OmegaOS is a public agentic-coding OS; it backs up its own state, never your
//! projects or credentials into its domain.
//!
//! Cross-user: every path is resolved from `$HOME` / config, never hardcoded.

use crate::config::OmegaConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// A repository with work that isn't safely on a remote yet.
#[derive(Debug, Clone)]
pub struct RepoRisk {
    pub path: PathBuf,
    /// Uncommitted changes in the working tree.
    pub dirty: bool,
    /// Commits ahead of upstream, or no upstream configured at all.
    pub unpushed: bool,
    /// `true` when the repo has no upstream branch (work exists on no remote).
    pub no_upstream: bool,
}

/// Result of a backup run.
#[derive(Debug, Clone)]
pub struct BackupReport {
    pub archive: PathBuf,
    pub archive_bytes: u64,
    pub included: Vec<String>,
    pub memory_included: bool,
    pub at_risk: Vec<RepoRisk>,
    /// Set when `~/.omega` is itself a symlink (legacy migration) — the real
    /// secrets live at this target and were dereferenced into the archive.
    pub omega_symlink_target: Option<PathBuf>,
}

/// Read-only state for `doctor --pre-reset` (no archive written).
#[derive(Debug, Clone)]
pub struct PreResetReport {
    pub omega_present: bool,
    pub secret_files: Vec<String>,
    pub memory_mb: Option<u64>,
    pub crontab_lines: Option<usize>,
    pub projects_dir: PathBuf,
    pub at_risk: Vec<RepoRisk>,
    pub projects_scanned: usize,
}

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// Scan every discovered project repo (read-only) for uncommitted / unpushed
/// work. Never mutates anything. Repos with nothing at risk are omitted.
pub fn scan_projects(config: &OmegaConfig) -> (Vec<RepoRisk>, usize) {
    // Category dirs under the user's resolved projects root — cross-user.
    let roots: Vec<PathBuf> = ["work", "clients", "1-life"]
        .iter()
        .map(|c| config.projects_dir.join(c))
        .filter(|p| p.is_dir())
        .collect();
    // Also scan the projects_dir itself in case the user keeps a flat layout.
    let mut scan: Vec<&Path> = roots.iter().map(|p| p.as_path()).collect();
    if config.projects_dir.is_dir() {
        scan.push(config.projects_dir.as_path());
    }

    let projects = OmegaConfig::discover_projects(&scan);
    let scanned = projects.len();
    let mut at_risk = Vec::new();
    for p in &projects {
        if !p.path.join(".git").exists() {
            continue;
        }
        let dirty = git_dirty(&p.path);
        let no_upstream = !git_has_upstream(&p.path);
        let unpushed = no_upstream || git_ahead(&p.path);
        if dirty || unpushed {
            at_risk.push(RepoRisk {
                path: p.path.clone(),
                dirty,
                unpushed,
                no_upstream,
            });
        }
    }
    (at_risk, scanned)
}

fn git_dirty(repo: &Path) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn git_has_upstream(repo: &Path) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--abbrev-ref", "@{u}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_ahead(repo: &Path) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-list", "--count", "@{u}..HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().parse::<u64>().unwrap_or(0) > 0)
        .unwrap_or(false)
}

/// Gather the read-only pre-reset readiness report.
pub fn pre_reset_report(config: &OmegaConfig) -> PreResetReport {
    let omega_dir = home().join(".omega");
    let omega_present = omega_dir.is_dir();

    // Which known secret/config files are present (names only — never values).
    let candidates = [
        "credentials/claude.json",
        "credentials/codex.json",
        "credentials/gemini.json",
        "provisioning/services.env",
        "provisioning/clerk-pool.env",
        "config/vercel-tokens.json",
        "config/git-identities.json",
        "telegram.toml",
        "projects.json",
    ];
    let secret_files = candidates
        .iter()
        .filter(|f| omega_dir.join(f).exists())
        .map(|f| f.to_string())
        .collect();

    let memory_mb = dir_size_mb(&home().join(".claude/projects"));
    let crontab_lines = Command::new("crontab")
        .arg("-l")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count());

    let (at_risk, projects_scanned) = scan_projects(config);
    PreResetReport {
        omega_present,
        secret_files,
        memory_mb,
        crontab_lines,
        projects_dir: config.projects_dir.clone(),
        at_risk,
        projects_scanned,
    }
}

/// Run the backup: archive `~/.omega` (+ captured crontab), optionally the
/// memory store, and return the report (including the read-only at-risk repo
/// scan). Projects are NEVER included in the archive.
pub fn run_backup(
    config: &OmegaConfig,
    out: Option<PathBuf>,
    include_memory: bool,
    timestamp: &str,
) -> Result<BackupReport> {
    let home = home();
    let omega_dir = home.join(".omega");
    anyhow::ensure!(
        omega_dir.is_dir(),
        "~/.omega not found — nothing to back up (is OmegaOS installed?)"
    );

    // Capture the crontab INTO ~/.omega so it lands in the archive.
    if let Ok(o) = Command::new("crontab").arg("-l").output() {
        if o.status.success() {
            let _ = std::fs::write(omega_dir.join("crontab.bak"), &o.stdout);
        }
    }

    let archive =
        out.unwrap_or_else(|| home.join(format!("omega-backup-{}.tgz", timestamp)));

    // Relative paths tar'd from $HOME. `.omega` is the OmegaOS-owned state.
    // Memory is opt-in (large). No project files, ever.
    let mut included = vec![".omega".to_string()];
    let memory_rel = ".claude/projects";
    let memory_included = include_memory && home.join(memory_rel).is_dir();
    if memory_included {
        included.push(memory_rel.to_string());
    }

    // `~/.omega` may itself be a symlink (legacy migration → e.g. ~/.aisb). Tar
    // WITHOUT -h would archive the dangling link and capture ZERO secrets. We
    // pass -h (--dereference) so the real files are always captured, and report
    // the target so the user knows their layout is non-standard.
    let omega_symlink_target = std::fs::symlink_metadata(&omega_dir)
        .ok()
        .filter(|m| m.file_type().is_symlink())
        .and_then(|_| std::fs::read_link(&omega_dir).ok());

    // Exclude regenerable bulk (logs/state/locks/*.log) so a legacy symlinked
    // ~/.omega doesn't drag gigabytes of logs into the archive. Secrets,
    // credentials, provisioning, and config are always kept.
    let status = Command::new("tar")
        .args(["czhf"])
        .arg(&archive)
        .arg("-C")
        .arg(&home)
        .args([
            // Regenerable OmegaOS runtime state.
            "--exclude=.omega/logs",
            "--exclude=.omega/state",
            "--exclude=.omega/locks",
            "--exclude=*.log",
            // Regenerable dev/dep artifacts (matter only for legacy symlinked
            // layouts; a clean ~/.omega has none of these). Never secrets.
            "--exclude=node_modules",
            "--exclude=__pycache__",
            "--exclude=.venv",
            "--exclude=target",
            "--exclude=.next",
        ])
        .args(&included)
        .status()
        .context("running tar (is it installed?)")?;
    anyhow::ensure!(status.success(), "tar exited with failure");

    let archive_bytes = std::fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
    let (at_risk, _) = scan_projects(config);

    Ok(BackupReport {
        archive,
        archive_bytes,
        included,
        memory_included,
        at_risk,
        omega_symlink_target,
    })
}

fn dir_size_mb(path: &Path) -> Option<u64> {
    if !path.is_dir() {
        return None;
    }
    let out = Command::new("du").args(["-sm"]).arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout)
        .ok()?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()
}
