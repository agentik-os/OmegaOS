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

    // 3. Doctrine integrity (6 Laws + 22 operational rules — R-SKILLPUB added
    // 2026-06-07; bump EXPECTED_OPS whenever rules.rs ships a new rule).
    const EXPECTED_LAWS: usize = 6;
    const EXPECTED_OPS: usize = 22;
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
    //     user is also excluded (pgrep -u scopes to this user). fix8-T1: a
    //     headless Mac (SSH/cron — no GUI launchctl domain) cannot see the
    //     agent-bot labels at all, so the exclusion list is unavailable there;
    //     report info and skip the duplicate check instead of counting every
    //     agent bot as a duplicate.
    {
        let uid = current_uid();
        let verdict = if cfg!(target_os = "macos") {
            let exclusion = launchctl_gui_domain_accessible(&uid).then(agent_bot_pids_darwin);
            poller_verdict(&raw_tg_bot_pids(&uid), exclusion.as_ref())
        } else {
            // Linux: the /proc-based exclusion is always available — same
            // main_bot_pollers count as before.
            let count = main_bot_pollers(&uid).len();
            if count > 1 {
                PollerVerdict::Duplicates(count)
            } else {
                PollerVerdict::Single(count)
            }
        };
        match verdict {
            PollerVerdict::Duplicates(count) => checks.push(Check::warn(
                "telegram poller",
                format!(
                    "multiple Telegram pollers ({}) — duplicate messages; keep only {}",
                    count,
                    crate::service::tg_bot_service_desc()
                ),
            )),
            PollerVerdict::Single(count) => {
                checks.push(Check::ok("telegram poller", format!("{} poller", count)))
            }
            PollerVerdict::Undeterminable => checks.push(Check::ok(
                "telegram poller",
                "poller ownership undeterminable on this host — no GUI launchctl domain; skipping duplicate check",
            )),
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

/// fix9: whether a command line is a GENUINE `bun … omega-tg-bot.ts`
/// invocation — argv[0] is the `bun` (or `…/bun`) executable AND a later token's
/// path basename is exactly `omega-tg-bot.ts`. Rejects command lines that merely
/// CONTAIN both tokens incidentally — e.g. a claude oracle/worker launcher whose
/// own argv[0] is `bash`/`claude` and that mentions `bun` and `omega-tg-bot.ts`
/// only inside a later argv element (a `--brief` string, $PATH, or quoted docs).
/// The loose `bun.*omega-tg-bot\.ts` pgrep pattern counted those as pollers and
/// `--fix` would have KILLED them (live agent sessions).
///
/// Requiring the bun executable to be argv[0] is the argv boundary expressed on
/// the space-joined cmdline: argv[0] is a path with no internal spaces, so it is
/// exactly the first whitespace token. A launcher's argv[0] is never `bun`, so
/// an embedded bot command — even a bare, undecorated one — can never re-admit
/// the false positive. (The brief recommended "any token == bun"; first-token is
/// strictly safer and still matches every documented case.) Agent bots run the
/// IDENTICAL `bun …/omega-tg-bot.ts` argv and pass here too; they are excluded
/// later via /proc environ (OMEGA_AGENT_BOT=), not by this predicate.
fn cmdline_is_tg_bot(cmdline: &str) -> bool {
    let mut tokens = cmdline.split_whitespace();
    let Some(exe) = tokens.next() else {
        return false;
    };
    if exe != "bun" && !exe.ends_with("/bun") {
        return false;
    }
    tokens.any(|t| t.rsplit('/').next() == Some("omega-tg-bot.ts"))
}

/// Raw `bun … omega-tg-bot.ts` PIDs for this user (pgrep), BEFORE any
/// agent-bot exclusion — both the main bot and agent bots match. See
/// `main_bot_pollers` / `poller_verdict` for the exclusion step.
fn raw_tg_bot_pids(uid: &str) -> Vec<u32> {
    // Tight pattern: require a `bun` executable token (start-of-line or after a
    // path separator) immediately followed by args reaching omega-tg-bot.ts —
    // NOT just `bun` and `omega-tg-bot.ts` anywhere on the line.
    let Ok(out) = std::process::Command::new("pgrep")
        .args(["-u", uid, "-f", r"(^|/)bun .*omega-tg-bot\.ts"])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .filter(|pid| {
            // Belt-and-suspenders (fix9): on Linux confirm each pid's real argv
            // is a genuine bun+script invocation by reading /proc/<pid>/cmdline
            // (NUL-separated argv) — platform pgrep quirks can never re-admit
            // the false positive that `--fix` would KILL. macOS has no /proc, so
            // the tightened pgrep match stands there (agent bots excluded later
            // via launchctl).
            if cfg!(target_os = "macos") {
                return true;
            }
            match std::fs::read(format!("/proc/{}/cmdline", pid)) {
                Ok(raw) => {
                    let cmdline = raw
                        .split(|b| *b == 0)
                        .map(String::from_utf8_lossy)
                        .collect::<Vec<_>>()
                        .join(" ");
                    cmdline_is_tg_bot(&cmdline)
                }
                // pid vanished between pgrep and read — drop it.
                Err(_) => false,
            }
        })
        .collect()
}

/// fix8-T1: whether this process can see the user's GUI launchctl domain.
/// Headless sessions (SSH/cron — no Aqua session) cannot: `launchctl print
/// gui/<uid>` fails there and/or `launchctl list` yields nothing. Without the
/// GUI domain the os.omega.tg-agent-* labels are invisible, so agent bots can
/// NOT be excluded from a poller count — callers must skip the duplicate
/// check rather than count agent bots as duplicates.
fn launchctl_gui_domain_accessible(uid: &str) -> bool {
    let print_ok = std::process::Command::new("launchctl")
        .args(["print", &format!("gui/{}", uid)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let list_nonempty = std::process::Command::new("launchctl")
        .arg("list")
        .output()
        .map(|o| o.status.success() && o.stdout.iter().any(|b| !b.is_ascii_whitespace()))
        .unwrap_or(false);
    print_ok && list_nonempty
}

/// Outcome of the duplicate-poller decision (see `poller_verdict`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PollerVerdict {
    /// 0 or 1 main poller — healthy.
    Single(usize),
    /// More than one main poller after agent-bot exclusion — warn.
    Duplicates(usize),
    /// Multiple pids but no exclusion list (headless Mac: GUI launchctl
    /// domain inaccessible) — the extras may all be legitimate agent bots; info.
    Undeterminable,
}

/// Pure duplicate-poller decision (fix8-T1) — unit-testable without launchctl.
/// `agent_exclusion` is the set of known-legitimate agent-bot pids, or None
/// when the platform cannot provide one. Without an exclusion list a multi-pid
/// count is undeterminable and must NEVER be reported as duplicates.
fn poller_verdict(
    raw_pids: &[u32],
    agent_exclusion: Option<&std::collections::HashSet<u32>>,
) -> PollerVerdict {
    match agent_exclusion {
        Some(excl) => {
            let count = raw_pids.iter().filter(|p| !excl.contains(p)).count();
            if count > 1 {
                PollerVerdict::Duplicates(count)
            } else {
                PollerVerdict::Single(count)
            }
        }
        None if raw_pids.len() > 1 => PollerVerdict::Undeterminable,
        None => PollerVerdict::Single(raw_pids.len()),
    }
}

/// This user's `bun … omega-tg-bot.ts` PIDs that are NOT agent bots. Agent
/// bots (omega-tg-agent-* / os.omega.tg-agent-*) run the SAME script but are
/// legitimate separate services and must NOT be killed. Platform-aware
/// exclusion (fix7-T3): Linux reads OMEGA_AGENT_BOT= from /proc/<pid>/environ;
/// macOS matches the pid against the os.omega.tg-agent-* launchd jobs.
fn main_bot_pollers(uid: &str) -> Vec<u32> {
    let agent_pids = if cfg!(target_os = "macos") {
        agent_bot_pids_darwin()
    } else {
        Default::default()
    };
    raw_tg_bot_pids(uid)
        .into_iter()
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

    // fix8-T1: the pure decision behind doctor check #11 (telegram poller).
    #[test]
    fn poller_verdict_excludes_agent_bots() {
        let excl: std::collections::HashSet<u32> = [20, 30].into_iter().collect();
        assert_eq!(poller_verdict(&[10, 20, 30], Some(&excl)), PollerVerdict::Single(1));
        assert_eq!(poller_verdict(&[10, 11, 20], Some(&excl)), PollerVerdict::Duplicates(2));
        assert_eq!(poller_verdict(&[], Some(&excl)), PollerVerdict::Single(0));
    }

    #[test]
    fn poller_verdict_without_exclusion_never_warns() {
        // Headless Mac: no GUI launchctl domain → no exclusion list. Several
        // pids may all be legitimate agent bots — undeterminable, never
        // Duplicates (the fix8-T1 false-warning bug).
        assert_eq!(poller_verdict(&[10, 20, 30], None), PollerVerdict::Undeterminable);
        assert_eq!(poller_verdict(&[10], None), PollerVerdict::Single(1));
        assert_eq!(poller_verdict(&[], None), PollerVerdict::Single(0));
    }

    // fix9: the pure predicate behind raw_tg_bot_pids' /proc/<pid>/cmdline
    // filter (doctor check #11 telegram poller). The loose `bun.*omega-tg-bot
    // \.ts` pattern false-matched claude launchers; `--fix` would KILL them.
    #[test]
    fn cmdline_is_tg_bot_rejects_claude_launcher() {
        // `.bun/bin` in $PATH + `omega-tg-bot.ts` inside a --brief string — the
        // documented false positive. argv[0] is `bash`, not the bun exe → false.
        let line = r#"bash -c export PATH="/home/vibe/.bun/bin:/x"; claude --brief "edit telegram-bot/omega-tg-bot.ts""#;
        assert!(!cmdline_is_tg_bot(line));
    }

    #[test]
    fn cmdline_is_tg_bot_accepts_real_bot() {
        // The genuine systemd poller: `…/bun` executable + script basename.
        assert!(cmdline_is_tg_bot(
            "/usr/local/bin/bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts"
        ));
    }

    #[test]
    fn cmdline_is_tg_bot_accepts_agent_bot() {
        // Agent bots run the IDENTICAL bun+script command — true here; they are
        // excluded later by /proc environ (OMEGA_AGENT_BOT), not this predicate.
        assert!(cmdline_is_tg_bot(
            "bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts --agent tg-agent-7"
        ));
    }

    #[test]
    fn cmdline_is_tg_bot_rejects_embedded_bare_bot_command() {
        // Robustness: a launcher (argv[0]=bash) whose --brief embeds the BARE,
        // undecorated real bot command as a substring. An "any token == bun"
        // check would false-match (and --fix would KILL this claude session);
        // requiring argv[0] to be the bun executable rejects it.
        assert!(!cmdline_is_tg_bot(
            "bash -c claude --brief run /usr/local/bin/bun /home/vibe/.omega/telegram-bot/omega-tg-bot.ts"
        ));
    }

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
