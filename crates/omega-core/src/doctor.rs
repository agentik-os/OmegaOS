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
            Health::Ok => "[+]",
            Health::Warn => "[!]",
            Health::Fail => "[x]",
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

/// The rmux daemon socket path, if present — `$RMUX_SOCKET` override, else the
/// conventional `/tmp/rmux-<uid>/default`.
fn rmux_socket_path() -> Option<std::path::PathBuf> {
    if let Ok(s) = std::env::var("RMUX_SOCKET") {
        let p = std::path::PathBuf::from(s);
        if p.exists() {
            return Some(p);
        }
    }
    for e in std::fs::read_dir("/tmp").ok()?.flatten() {
        if e.file_name().to_string_lossy().starts_with("rmux-") {
            let sock = e.path().join("default");
            if sock.exists() {
                return Some(sock);
            }
        }
    }
    None
}

/// Claude Code hooks: scripts present under `~/.omega/hooks` AND registered in
/// `~/.claude/settings.json` (PostToolUse track-tool-use + Stop stop-verify).
fn check_hooks(config: &OmegaConfig) -> Check {
    let hooks_dir = config
        .state_dir
        .parent()
        .map(|p| p.join("hooks"))
        .unwrap_or_else(|| std::path::PathBuf::from("/nonexistent"));
    let scripts_present = hooks_dir.join("track-tool-use.sh").exists()
        && hooks_dir.join("stop-verify-hook.sh").exists();

    let registered = dirs::home_dir()
        .map(|h| h.join(".claude/settings.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.contains("track-tool-use") && s.contains("stop-verify"))
        .unwrap_or(false);

    match (scripts_present, registered) {
        (true, true) => Check::ok("hooks", "track + verify present, registered in settings.json"),
        (true, false) => Check::warn(
            "hooks",
            "scripts present but not registered in settings.json (re-run install.sh; needs jq)",
        ),
        (false, _) => Check::warn(
            "hooks",
            format!("hook scripts missing from {}", hooks_dir.display()),
        ),
    }
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

    // 2b. rmux socket file present (distinct from the daemon RPC above —
    // catches a dead daemon that left a stale or missing socket).
    match rmux_socket_path() {
        Some(p) => checks.push(Check::ok("rmux socket", p.display().to_string())),
        None => checks.push(Check::warn(
            "rmux socket",
            "no socket under /tmp/rmux-*/ or $RMUX_SOCKET (daemon down?)",
        )),
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
    match systemctl_user(&["is-active", "omega-tg-bot.service"]) {
        Some(s) if s == "active" => {
            checks.push(Check::ok("telegram service", "omega-tg-bot.service active"))
        }
        Some(other) => checks.push(Check::warn(
            "telegram service",
            format!("omega-tg-bot.service {} (start: systemctl --user start omega-tg-bot)", other),
        )),
        None => checks.push(Check::warn(
            "telegram service",
            "systemd user unit not found (optional)",
        )),
    }

    // 6b. Claude Code hooks installed + registered.
    checks.push(check_hooks(config));

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

    // 9. Usage cache (drives the TUI token toolbar; refreshed by the
    //    `omega usage` cron). Soft — a blank toolbar is not a failure.
    {
        let usage = config.state_dir.join("usage.json");
        match std::fs::metadata(&usage).and_then(|m| m.modified()) {
            Ok(mtime) => {
                let age = std::time::SystemTime::now()
                    .duration_since(mtime)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mins = age / 60;
                if age > 900 {
                    checks.push(Check::warn(
                        "usage cache",
                        format!("usage cache stale ({} min) — omega usage cron may be failing", mins),
                    ));
                } else {
                    checks.push(Check::ok("usage cache", format!("usage cache {} min old", mins)));
                }
            }
            Err(_) => checks.push(Check::warn(
                "usage cache",
                "usage.json missing — TUI token toolbar blank (cron not run)",
            )),
        }
    }

    // 10. Claude OAuth credential — the agent CLI needs a live token.
    {
        let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
        let cred = omega_dir.map(|d| d.join("credentials/claude.json"));
        match cred.and_then(|p| std::fs::read_to_string(&p).ok()) {
            Some(content) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i128)
                    .unwrap_or(0);
                let expires = serde_json::from_str::<serde_json::Value>(&content)
                    .ok()
                    .and_then(|v| {
                        // The Claude credential nests the token under `claudeAiOauth`;
                        // fall back to a top-level field for older/alternate formats.
                        v.get("claudeAiOauth")
                            .and_then(|o| o.get("expiresAt"))
                            .or_else(|| v.get("expiresAt"))
                            .and_then(|e| e.as_i64())
                            .map(|n| n as i128)
                    });
                match expires {
                    Some(exp) if exp < now_ms => checks.push(Check::warn(
                        "claude oauth",
                        "Claude OAuth expired — refresh required",
                    )),
                    Some(_) => {
                        checks.push(Check::ok("claude oauth", "Claude OAuth valid"))
                    }
                    None => checks.push(Check::warn(
                        "claude oauth",
                        "claude.json missing/unreadable — agent CLI will fail",
                    )),
                }
            }
            None => checks.push(Check::warn(
                "claude oauth",
                "claude.json missing/unreadable — agent CLI will fail",
            )),
        }
    }

    // 11. Single Telegram poller — two `omega-tg-bot.ts` processes mean
    //     duplicate messages / getUpdates 409s. Only systemd should run it.
    {
        // Scope to THIS user's bun bot: a co-tenant's bot (a different token under
        // another user) and the doctor's own shell must not inflate the count —
        // only a genuine duplicate poller of this install should warn.
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty());
        let mut pgrep = std::process::Command::new("pgrep");
        if let Some(ref u) = uid {
            pgrep.args(["-u", u]);
        }
        let count = pgrep
            .args(["-fc", r"bun.*omega-tg-bot\.ts"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
            .unwrap_or(0);
        if count > 1 {
            checks.push(Check::warn(
                "telegram poller",
                format!("multiple Telegram pollers ({}) — duplicate messages; keep only systemd omega-tg-bot.service", count),
            ));
        } else {
            checks.push(Check::ok("telegram poller", format!("{} poller", count)));
        }
    }

    // 12. Provisioning tokens — warn only if the file exists but ALL deploy
    //     tokens are blank (deploys will fail). Absent file → skip silently.
    {
        let omega_dir = config.state_dir.parent().map(|p| p.to_path_buf());
        let svc = omega_dir.map(|d| d.join("provisioning/services.env"));
        if let Some(content) = svc.and_then(|p| std::fs::read_to_string(&p).ok()) {
            let pairs: std::collections::HashMap<String, bool> = content
                .lines()
                .filter_map(parse_export_kv)
                .map(|(k, v)| (k, !v.is_empty()))
                .collect();
            let watched = ["VERCEL_TOKEN", "CONVEX_TEAM_TOKEN", "GITHUB_TOKEN", "STRIPE_SECRET_KEY"];
            let set: Vec<&str> = watched
                .iter()
                .copied()
                .filter(|k| *pairs.get(*k).unwrap_or(&false))
                .collect();
            if set.is_empty() {
                checks.push(Check::warn(
                    "provisioning",
                    "provisioning tokens blank — fill via TUI Provisioning to enable deploys",
                ));
            } else {
                checks.push(Check::ok(
                    "provisioning",
                    format!("provisioning: {}", set.join(", ")),
                ));
            }
        }
    }

    checks
}

/// Minimal `export KEY="value"` / `export KEY=value` parse → (KEY, value).
/// Mirrors provisioning::parse_export; kept local to avoid a cross-module
/// `pub` just for the doctor check.
fn parse_export_kv(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("export ")?;
    let (k, v) = rest.split_once('=')?;
    let key = k.trim().to_string();
    if key.is_empty()
        || !key.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        return None;
    }
    let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
    Some((key, val))
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
