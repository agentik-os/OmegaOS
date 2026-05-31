//! `omega doctor` — a one-shot health check of the whole stack. The first
//! command to run after a fresh install / VPS reset: it tells you, in one
//! screen, whether the daemon, doctrine, agent CLI, Telegram service, secrets,
//! and resources are all in order. Adapts to the host — a missing systemd or
//! crontab is a soft warning, never a hard error.

use crate::config::OmegaConfig;
use crate::session::SessionManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Ok,
    Warn,
    Fail,
}

impl Health {
    pub fn glyph(&self) -> &'static str {
        match self {
            Health::Ok => "✓",
            Health::Warn => "⚠",
            Health::Fail => "✗",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub health: Health,
    pub detail: String,
}

impl Check {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Ok, detail: detail.into() }
    }
    fn warn(name: &str, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Warn, detail: detail.into() }
    }
    fn fail(name: &str, detail: impl Into<String>) -> Self {
        Self { name: name.into(), health: Health::Fail, detail: detail.into() }
    }
}

/// Run a `systemctl --user` query, returning its trimmed stdout (or None if
/// systemd / the unit isn't available — a soft condition, not an error).
fn systemctl_user(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn ram_available_mb() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemAvailable:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<u64>().ok())
        })
        .map(|kb| kb / 1024)
        .unwrap_or(0)
}

/// Run every health check. Each is independent and never panics.
pub async fn run_all(config: &OmegaConfig) -> Vec<Check> {
    let mut checks = Vec::new();

    // 1. Binary version.
    checks.push(Check::ok("binary", format!("omega {}", env!("CARGO_PKG_VERSION"))));

    // 2. rmux daemon reachable.
    match SessionManager::connect().await {
        Ok(mgr) => {
            let n = mgr.list_sessions().await.map(|s| s.len()).unwrap_or(0);
            checks.push(Check::ok("rmux daemon", format!("connected, {} live session(s)", n)));
        }
        Err(e) => checks.push(Check::fail("rmux daemon", format!("unreachable: {}", e))),
    }

    // 3. Doctrine integrity (6 Laws + 20 operational rules).
    let laws = crate::rules::laws().len();
    let ops = crate::rules::operational_rules().len();
    if laws == 6 && ops == 20 {
        checks.push(Check::ok("doctrine", format!("{} Laws + {} Rules", laws, ops)));
    } else {
        checks.push(Check::warn(
            "doctrine",
            format!("{} Laws + {} Rules (expected 6 + 20)", laws, ops),
        ));
    }

    // 4. Agent CLI available.
    match crate::agents::Agent::from_name(&config.agent_command) {
        Some(agent) if agent.is_available() => {
            checks.push(Check::ok("agent CLI", format!("{} available", agent.name())))
        }
        Some(agent) => {
            let hint = agent.install_command().map(|c| format!(" — {}", c)).unwrap_or_default();
            checks.push(Check::warn("agent CLI", format!("{} not on PATH{}", agent.name(), hint)))
        }
        None => checks.push(Check::warn(
            "agent CLI",
            format!("unknown agent '{}'", config.agent_command),
        )),
    }

    // 5. State dir writable.
    let probe = config.state_dir.join(".doctor-probe");
    match std::fs::write(&probe, b"ok").and_then(|_| std::fs::remove_file(&probe)) {
        Ok(()) => checks.push(Check::ok("state dir", config.state_dir.display().to_string())),
        Err(e) => checks.push(Check::fail(
            "state dir",
            format!("{} not writable: {}", config.state_dir.display(), e),
        )),
    }

    // 6. Telegram service (systemd — soft if absent).
    match systemctl_user(&["is-active", "omega-telegram.service"]) {
        Some(s) if s == "active" => {
            checks.push(Check::ok("telegram service", "omega-telegram.service active"))
        }
        Some(other) => checks.push(Check::warn(
            "telegram service",
            format!("omega-telegram.service {} (start: systemctl --user start omega-telegram)", other),
        )),
        None => checks.push(Check::warn(
            "telegram service",
            "systemd user unit not found (optional)",
        )),
    }

    // 7. Secrets present (~/.omega exists + non-empty).
    let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
    match omega_dir {
        Some(dir) if dir.exists() => {
            let has_any = std::fs::read_dir(&dir)
                .map(|mut e| e.next().is_some())
                .unwrap_or(false);
            if has_any {
                checks.push(Check::ok("secrets dir", format!("{} present", dir.display())));
            } else {
                checks.push(Check::warn("secrets dir", format!("{} empty", dir.display())));
            }
        }
        _ => checks.push(Check::warn("secrets dir", "~/.omega not found")),
    }

    // 8. Memory headroom.
    let ram = ram_available_mb();
    if ram == 0 {
        checks.push(Check::warn("memory", "/proc/meminfo unreadable"));
    } else if ram < 400 {
        checks.push(Check::warn(
            "memory",
            format!("{}MB available — low (try 'omega cleanup --yes')", ram),
        ));
    } else {
        checks.push(Check::ok("memory", format!("{}MB available", ram)));
    }

    checks
}

/// Worst health across all checks — drives the process exit code.
pub fn overall(checks: &[Check]) -> Health {
    if checks.iter().any(|c| c.health == Health::Fail) {
        Health::Fail
    } else if checks.iter().any(|c| c.health == Health::Warn) {
        Health::Warn
    } else {
        Health::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overall_is_worst_of() {
        let ok = vec![Check::ok("a", "x")];
        assert_eq!(overall(&ok), Health::Ok);
        let warn = vec![Check::ok("a", "x"), Check::warn("b", "y")];
        assert_eq!(overall(&warn), Health::Warn);
        let fail = vec![Check::warn("b", "y"), Check::fail("c", "z")];
        assert_eq!(overall(&fail), Health::Fail);
    }
}
