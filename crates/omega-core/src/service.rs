//! Control of the omega-tg-bot user service, cross-platform: a systemd
//! `--user` unit on Linux, a launchd LaunchAgent on macOS. One tiny helper so
//! `omega doctor` and the TUI Telegram wizard share the same logic instead of
//! each hardcoding `systemctl` (which made every macOS status query a false
//! failure and every piece of advice dead).
//!
//! The launchd label and plist path MUST match what install.sh writes
//! (`LA_LABEL="os.omega.tg-bot"`, `$HOME/Library/LaunchAgents/$LA_LABEL.plist`).

pub const TG_BOT_SYSTEMD_UNIT: &str = "omega-tg-bot.service";
pub const TG_BOT_LAUNCHD_LABEL: &str = "os.omega.tg-bot";

fn is_darwin() -> bool {
    cfg!(target_os = "macos")
}

fn uid() -> String {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn launchd_target() -> String {
    format!("gui/{}/{}", uid(), TG_BOT_LAUNCHD_LABEL)
}

fn launchd_plist() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", TG_BOT_LAUNCHD_LABEL))
}

/// Status of the tg-bot service. `Some("active")` when it is running,
/// `Some(<state>)` when the service exists but isn't running, `None` when the
/// service manager / unit is absent (a soft condition, not an error).
pub fn tg_bot_status() -> Option<String> {
    if is_darwin() {
        let out = std::process::Command::new("launchctl")
            .args(["print", &launchd_target()])
            .output()
            .ok()?;
        if !out.status.success() {
            return None; // LaunchAgent not bootstrapped (or no launchd)
        }
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("state = running") {
            Some("active".to_string())
        } else {
            Some("loaded, not running".to_string())
        }
    } else {
        // `is-active` prints the state (active/inactive/failed/…) even on a
        // non-zero exit; empty stdout means systemd itself is unavailable.
        let out = std::process::Command::new("systemctl")
            .args(["--user", "is-active", TG_BOT_SYSTEMD_UNIT])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

/// Enable + start the tg-bot service. Returns true when the service is up.
pub fn tg_bot_start() -> bool {
    if is_darwin() {
        // bootstrap is best-effort (fails harmlessly when already loaded);
        // kickstart then (re)launches the job. Mirrors install.sh.
        let _ = std::process::Command::new("launchctl")
            .args(["bootstrap", &format!("gui/{}", uid())])
            .arg(launchd_plist())
            .status();
        std::process::Command::new("launchctl")
            .args(["kickstart", &launchd_target()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("systemctl")
            .args(["--user", "enable", "--now", TG_BOT_SYSTEMD_UNIT])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Restart the tg-bot service. Returns true on success.
pub fn tg_bot_restart() -> bool {
    if is_darwin() {
        std::process::Command::new("launchctl")
            .args(["kickstart", "-k", &launchd_target()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("systemctl")
            .args(["--user", "restart", TG_BOT_SYSTEMD_UNIT])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// The one-liner the operator should run to start the service by hand.
pub fn tg_bot_start_hint() -> String {
    if is_darwin() {
        format!(
            "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/{label}.plist; launchctl kickstart gui/$(id -u)/{label}",
            label = TG_BOT_LAUNCHD_LABEL
        )
    } else {
        format!("systemctl --user enable --now {}", TG_BOT_SYSTEMD_UNIT)
    }
}

/// The one-liner the operator should run to restart the service by hand.
pub fn tg_bot_restart_hint() -> String {
    if is_darwin() {
        format!("launchctl kickstart -k gui/$(id -u)/{}", TG_BOT_LAUNCHD_LABEL)
    } else {
        format!("systemctl --user restart {}", TG_BOT_SYSTEMD_UNIT)
    }
}
