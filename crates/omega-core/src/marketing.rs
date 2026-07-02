//! Marketing projects — discovery + fast filesystem status for the Marketing
//! tab (TUI), the `omega marketing` CLI, and the Telegram `nav:marketing` hub.
//!
//! A project is "marketing-enabled" when it has a `<path>/marketing/` directory
//! (created by the marketing machine + zernio + higgsfield pipeline). This
//! module answers TWO questions cheaply, with NO network:
//!   1. which projects have marketing? (`list_marketing_projects`)
//!   2. what's their at-a-glance status? (content ✓, calendar post count,
//!      daily-engine on/off)
//! Connected-account counts touch `omega-zernio` and are therefore ON-DEMAND
//! only (`project_accounts`) — never fetched per-frame.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A project with a `marketing/` directory, plus fast filesystem status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingProject {
    /// Display name (the registry name, else the directory name).
    pub name: String,
    /// Slug = directory name lowercased — the id zernio/higgsfield use.
    pub slug: String,
    /// Absolute project root (the parent of `marketing/`).
    pub path: PathBuf,
    /// `marketing/05-calendar/calendar-90d.json` (or `calendar.json`) exists.
    pub has_content: bool,
    /// Total posts counted across days in the calendar json (0 on any failure).
    pub calendar_posts: usize,
    /// The daily-publishing engine is wired (cron line present OR the
    /// `marketing/04-publishing/daily-engine/` dir exists).
    pub engine_on: bool,
    /// Connected social accounts — `None` in the list (populated on-demand only
    /// via `project_accounts`, never per-frame).
    pub accounts: Option<usize>,
}

/// The projects root to scan for `marketing/` subdirs. Configurable via
/// `OMEGA_STATION_DIR`; defaults to `~/Station/SideBusiness` (the operator's
/// station root), falling back to `~/Station` then `$HOME`.
fn station_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_STATION_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let side = home.join("Station").join("SideBusiness");
    if side.is_dir() {
        return side;
    }
    let station = home.join("Station");
    if station.is_dir() {
        return station;
    }
    home
}

/// True if `<path>/marketing/` is a directory.
fn has_marketing_dir(path: &Path) -> bool {
    path.join("marketing").is_dir()
}

/// The calendar json for a project, if either canonical name exists.
/// Prefers `calendar-90d.json`, then `calendar.json`.
fn calendar_json_path(marketing: &Path) -> Option<PathBuf> {
    let cal_dir = marketing.join("05-calendar");
    for name in ["calendar-90d.json", "calendar.json"] {
        let p = cal_dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Best-effort count of posts across days in a calendar json. Returns 0 on any
/// parse failure. Handles the common shapes:
///   { "days": [ { "posts": [...] }, ... ] }
///   { "<date>": { "posts": [...] }, ... }  (map of days)
///   [ { "posts": [...] }, ... ]            (array of days)
///   { "posts": [...] }                     (flat)
fn count_calendar_posts(json_path: &Path) -> usize {
    let Ok(raw) = std::fs::read_to_string(json_path) else {
        return 0;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return 0;
    };

    fn posts_in(day: &serde_json::Value) -> usize {
        match day.get("posts") {
            Some(serde_json::Value::Array(a)) => a.len(),
            _ => 0,
        }
    }

    match &val {
        // Flat: a single object carrying "posts".
        serde_json::Value::Object(map) if map.contains_key("posts") => posts_in(&val),
        // { "days": [ ... ] }
        serde_json::Value::Object(map) if map.get("days").map(|d| d.is_array()).unwrap_or(false) => {
            map["days"]
                .as_array()
                .map(|days| days.iter().map(posts_in).sum())
                .unwrap_or(0)
        }
        // Map of days → each value is a day object.
        serde_json::Value::Object(map) => map.values().map(posts_in).sum(),
        // Array of day objects.
        serde_json::Value::Array(days) => days.iter().map(posts_in).sum(),
        _ => 0,
    }
}

/// Whether the daily-publishing engine is wired for this project. Fast + local:
///   • the `marketing/04-publishing/daily-engine/` dir exists, OR
///   • a `crontab -l` line mentions this project's daily-engine (matched on the
///     slug + "daily-engine"). The crontab is read ONCE per call.
fn engine_on(marketing: &Path, slug: &str, crontab: &str) -> bool {
    if marketing.join("04-publishing").join("daily-engine").is_dir() {
        return true;
    }
    let needle = slug.to_lowercase();
    crontab.lines().any(|line| {
        let l = line.to_lowercase();
        l.contains("daily-engine") && (l.contains(&needle) || l.contains(&format!("/{needle}/")))
    })
}

/// Read `crontab -l` once (empty string on any failure / no crontab).
fn read_crontab() -> String {
    std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Build a `MarketingProject` from a discovered root. `crontab` is passed in so
/// the (potentially slow) `crontab -l` is read once for the whole list.
fn build(name: String, path: PathBuf, crontab: &str) -> MarketingProject {
    let slug = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| name.to_lowercase());
    let marketing = path.join("marketing");
    let cal = calendar_json_path(&marketing);
    let has_content = cal.is_some();
    let calendar_posts = cal.as_deref().map(count_calendar_posts).unwrap_or(0);
    let engine_on = engine_on(&marketing, &slug, crontab);
    MarketingProject {
        name,
        slug,
        path,
        has_content,
        calendar_posts,
        engine_on,
        accounts: None,
    }
}

/// List marketing-enabled projects: the UNION of the project registry and a
/// filesystem scan of the station root, deduped by name (case-insensitive).
/// Fast, local-only — safe to call on tab entry / F5.
pub fn list_marketing_projects() -> Vec<MarketingProject> {
    let crontab = read_crontab();
    let mut out: Vec<MarketingProject> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push = |name: String, path: PathBuf| {
        let key = name.to_lowercase();
        if seen.insert(key) {
            out.push(build(name, path, &crontab));
        }
    };

    // (a) Registry projects whose path/marketing/ exists.
    let registry = crate::project_manager::ProjectRegistry::load();
    for p in registry.projects {
        if has_marketing_dir(&p.path) {
            push(p.name, p.path);
        }
    }

    // (b) Filesystem scan: immediate subdirs of the station root with marketing/.
    let root = station_dir();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && has_marketing_dir(&path) {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if !name.is_empty() {
                    push(name, path);
                }
            }
        }
    }

    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Connected-account count for a project — shells out to `omega-zernio`.
/// ON-DEMAND ONLY (detail pane / CLI), never per-frame. Bounded so a hung
/// zernio can't stall the UI: a background thread with a short join timeout.
/// Returns `None` if zernio is absent, errors, times out, or yields nothing.
pub fn project_accounts(slug: &str) -> Option<usize> {
    let slug = slug.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // Prefer JSON (count array/object entries); fall back to line count.
        let json = std::process::Command::new("omega-zernio")
            .args(["accounts", &slug, "--json"])
            .output();
        let count = match json {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                parse_accounts_json(&text).or_else(|| {
                    // Not JSON — fall through to line count below.
                    None
                })
            }
            _ => None,
        };
        let count = count.or_else(|| {
            let plain = std::process::Command::new("omega-zernio")
                .args(["accounts", &slug])
                .output()
                .ok()?;
            if !plain.status.success() {
                return None;
            }
            let text = String::from_utf8_lossy(&plain.stdout);
            let n = text.lines().filter(|l| !l.trim().is_empty()).count();
            Some(n)
        });
        let _ = tx.send(count);
    });
    rx.recv_timeout(std::time::Duration::from_secs(4))
        .ok()
        .flatten()
}

/// Count entries in a zernio `accounts --json` payload. Accepts an array, an
/// object with an `accounts` array, or a bare object (keyed by account).
fn parse_accounts_json(text: &str) -> Option<usize> {
    let val = serde_json::from_str::<serde_json::Value>(text.trim()).ok()?;
    match val {
        serde_json::Value::Array(a) => Some(a.len()),
        serde_json::Value::Object(ref map) => {
            if let Some(serde_json::Value::Array(a)) = map.get("accounts") {
                Some(a.len())
            } else {
                Some(map.len())
            }
        }
        _ => None,
    }
}

/// Convenience paths surfaced in the detail pane.
impl MarketingProject {
    pub fn marketing_dir(&self) -> PathBuf {
        self.path.join("marketing")
    }
    /// Human-readable path to the 90-day calendar markdown (may not exist).
    pub fn calendar_md(&self) -> PathBuf {
        self.marketing_dir()
            .join("05-calendar")
            .join("calendar-90d.md")
    }
    /// Status glyph: 🟢 content ✓ + engine on / 🟡 content ✓ engine off / ⚪ empty.
    pub fn glyph(&self) -> &'static str {
        match (self.has_content, self.engine_on) {
            (true, true) => "🟢",
            (true, false) => "🟡",
            (false, _) => "⚪",
        }
    }
}
