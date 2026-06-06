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

    // 3. Doctrine integrity (6 Laws + 21 operational rules — R-PDF added
    // 2026-06-05; bump EXPECTED_OPS whenever rules.rs ships a new rule).
    const EXPECTED_LAWS: usize = 6;
    const EXPECTED_OPS: usize = 21;
    let laws = crate::rules::laws().len();
    let ops = crate::rules::operational_rules().len();
    if laws == EXPECTED_LAWS && ops == EXPECTED_OPS {
        checks.push(Check::ok("doctrine", format!("{} Laws + {} Rules", laws, ops)));
    } else {
        checks.push(Check::warn(
            "doctrine",
            format!("{} Laws + {} Rules (expected {} + {})", laws, ops, EXPECTED_LAWS, EXPECTED_OPS),
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

    // 6. Telegram service (systemd on Linux, launchd on macOS — soft if absent).
    match crate::service::tg_bot_status() {
        Some(s) if s == "active" => {
            checks.push(Check::ok("telegram service", "omega-tg-bot active"))
        }
        Some(other) => checks.push(Check::warn(
            "telegram service",
            format!("omega-tg-bot {} (start: {})", other, crate::service::tg_bot_start_hint()),
        )),
        None => checks.push(Check::warn(
            "telegram service",
            "user service not found (optional)",
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

    // 11. Single Telegram MAIN-bot poller — two main pollers mean duplicate
    //     messages / getUpdates 409s; only the service manager should run it.
    //     Agent bots (omega-tg-agent-* / os.omega.tg-agent-*) run the SAME
    //     script but are legitimate separate services (own token, per-project)
    //     — `main_bot_pollers` excludes them platform-aware (/proc environ on
    //     Linux, launchd labels on macOS) so they don't inflate the count into
    //     a false "duplicate pollers" warning. A co-tenant's bot under another
    //     user is also excluded (pgrep -u scopes to this user).
    {
        let count = main_bot_pollers(&current_uid()).len();
        if count > 1 {
            checks.push(Check::warn(
                "telegram poller",
                format!(
                    "multiple Telegram pollers ({}) — duplicate messages; keep only {}",
                    count,
                    crate::service::tg_bot_service_desc()
                ),
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

    // 13. Telegram bot parity: the LIVE bot (~/.omega/telegram-bot/omega-tg-bot.ts,
    //     run by the systemd service) must match the source in the OmegaOS repo. A
    //     concurrent linter/edit can revert one side, silently dropping shipped
    //     features. Warn if they diverge so the operator redeploys.
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home"));
        let live = home.join(".omega/telegram-bot/omega-tg-bot.ts");
        let candidates = [
            home.join("Station/SideBusiness/OmegaOS/telegram-bot/omega-tg-bot.ts"),
            home.join("Station/OmegaOS/telegram-bot/omega-tg-bot.ts"),
            home.join("OmegaOS/telegram-bot/omega-tg-bot.ts"),
            std::path::PathBuf::from("/tmp/omega-build/telegram-bot/omega-tg-bot.ts"),
        ];
        match std::fs::read(&live) {
            Ok(live_bytes) => {
                let repo = candidates.iter().find(|p| p.exists());
                match repo.and_then(|p| std::fs::read(p).ok()) {
                    Some(repo_bytes) if repo_bytes == live_bytes => {
                        checks.push(Check::ok("telegram bot parity", "live bot == repo source"))
                    }
                    Some(_) => checks.push(Check::warn(
                        "telegram bot parity",
                        format!(
                            "live bot DIFFERS from repo — redeploy: cp <repo>/telegram-bot/omega-tg-bot.ts ~/.omega/telegram-bot/ && {}",
                            crate::service::tg_bot_restart_hint()
                        ),
                    )),
                    None => checks.push(Check::ok(
                        "telegram bot parity",
                        "live bot present (repo source not found to compare)",
                    )),
                }
            }
            Err(_) => checks.push(Check::warn("telegram bot parity", "live bot missing from ~/.omega/telegram-bot")),
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

// ───────── auto-fix (omega doctor --fix / the self-heal daemon) ─────────

fn current_uid() -> String {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// The service-managed main-bot PID (the ONE poller we must keep): systemd
/// MainPID on Linux, the launchd job pid on macOS (fix7-T3 — this healer was
/// systemd-only and fail-safe-skipped on every Mac). 0/None if the service
/// isn't running under its manager.
fn service_main_pid() -> Option<u32> {
    if cfg!(target_os = "macos") {
        // `launchctl print gui/<uid>/os.omega.tg-bot` emits a `pid = <N>`
        // line while the job is running (absent when stopped).
        let target = format!("gui/{}/{}", current_uid(), crate::service::TG_BOT_LAUNCHD_LABEL);
        let out = std::process::Command::new("launchctl")
            .args(["print", &target])
            .output()
            .ok()?;
        if !out.status.success() {
            return None; // LaunchAgent not bootstrapped (or no launchd)
        }
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|l| l.trim().strip_prefix("pid =").and_then(|v| v.trim().parse::<u32>().ok()))
            .filter(|p| *p != 0)
    } else {
        let out = systemctl_user(&["show", "omega-tg-bot.service", "-p", "MainPID", "--value"])?;
        out.trim().parse::<u32>().ok().filter(|p| *p != 0)
    }
}

/// PIDs of legitimate agent bots on macOS, where /proc doesn't exist. Every
/// agent bot is a launchd job labelled `os.omega.tg-agent-<id>` (see
/// spawnAgentBot in omega-tg-bot.ts), so collect their pids from
/// `launchctl list` (columns: PID Status Label; "-" when not running).
/// NOTE a command-line check (`ps -o command=`) can NOT replace this: agent
/// bots run the IDENTICAL `bun … omega-tg-bot.ts` command as the main bot
/// and differ only by environment / launchd label.
fn agent_bot_pids_darwin() -> std::collections::HashSet<u32> {
    let Ok(out) = std::process::Command::new("launchctl").arg("list").output() else {
        return Default::default();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut cols = l.split_whitespace();
            let pid = cols.next()?.parse::<u32>().ok()?;
            let _status = cols.next()?;
            let label = cols.next()?;
            label.starts_with("os.omega.tg-agent-").then_some(pid)
        })
        .collect()
}

/// This user's `bun … omega-tg-bot.ts` PIDs that are NOT agent bots. Agent
/// bots (omega-tg-agent-* / os.omega.tg-agent-*) run the SAME script but are
/// legitimate separate services and must NOT be killed. Platform-aware
/// exclusion (fix7-T3): Linux reads OMEGA_AGENT_BOT= from /proc/<pid>/environ;
/// macOS matches the pid against the os.omega.tg-agent-* launchd jobs.
fn main_bot_pollers(uid: &str) -> Vec<u32> {
    let Ok(out) = std::process::Command::new("pgrep")
        .args(["-u", uid, "-f", r"bun.*omega-tg-bot\.ts"])
        .output()
    else {
        return Vec::new();
    };
    let agent_pids = if cfg!(target_os = "macos") {
        agent_bot_pids_darwin()
    } else {
        Default::default()
    };
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .filter(|pid| {
            if cfg!(target_os = "macos") {
                return !agent_pids.contains(pid);
            }
            // Skip agent bots: their /proc/<pid>/environ contains OMEGA_AGENT_BOT=.
            let environ = std::fs::read(format!("/proc/{}/environ", pid)).unwrap_or_default();
            !environ
                .windows(b"OMEGA_AGENT_BOT=".len())
                .any(|w| w == b"OMEGA_AGENT_BOT=")
        })
        .collect()
}

/// Kill orphan duplicate pollers of the MAIN bot, keeping the service-managed one.
fn fix_duplicate_pollers() -> Vec<String> {
    let uid = current_uid();
    let pollers = main_bot_pollers(&uid);
    if pollers.len() <= 1 {
        return Vec::new();
    }
    // SAFETY: only kill duplicates when we can positively identify the ONE to
    // keep (the service-managed PID). If the service manager can't tell us, do
    // nothing rather than risk killing the live bot.
    let Some(keep_pid) = service_main_pid() else {
        return vec![format!(
            "duplicate pollers found but the service-managed PID is unknown — skipped (manual: keep only {})",
            crate::service::tg_bot_service_desc()
        )];
    };
    let keep = Some(keep_pid);
    let mut log = Vec::new();
    for pid in pollers {
        if Some(pid) == keep {
            continue; // the canonical, service-managed poller
        }
        let ok = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            log.push(format!(
                "killed duplicate Telegram poller pid {}{}",
                pid,
                keep.map(|k| format!(" (kept service-managed {})", k)).unwrap_or_default()
            ));
        }
    }
    log
}

fn fix_restart_tg_service() -> Vec<String> {
    if crate::service::tg_bot_restart() {
        vec!["restarted omega-tg-bot service".into()]
    } else {
        Vec::new()
    }
}

fn fix_refresh_usage() -> Vec<String> {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("omega"));
    let ok = std::process::Command::new(exe)
        .args(["usage", "--check"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        vec!["refreshed usage cache (omega usage --check)".into()]
    } else {
        Vec::new()
    }
}

fn fix_refresh_oauth() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let script = home.join(".omega/bin/omega-token-refresh.sh");
    if !script.exists() {
        return Vec::new();
    }
    let ok = std::process::Command::new("bash")
        .arg(&script)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        vec!["ran omega-token-refresh.sh (oauth)".into()]
    } else {
        Vec::new()
    }
}

/// Apply safe, mechanical fixes for the warnings/fails we know how to resolve
/// (duplicate pollers, dead Telegram service, stale usage cache, expired oauth).
/// Returns one log line per action taken. Never panics; anything it can't fix is
/// left for an oracle (the /status "Fix it" button) or the operator.
pub fn auto_fix(checks: &[Check]) -> Vec<String> {
    let mut log = Vec::new();
    for c in checks.iter().filter(|c| c.health != Health::Ok) {
        match c.name.as_str() {
            "telegram poller" => log.extend(fix_duplicate_pollers()),
            "telegram service" => log.extend(fix_restart_tg_service()),
            "usage cache" => log.extend(fix_refresh_usage()),
            "claude oauth" => log.extend(fix_refresh_oauth()),
            _ => {}
        }
    }
    log
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
