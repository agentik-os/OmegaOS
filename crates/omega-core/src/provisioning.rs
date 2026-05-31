//! Provisioning credential store — `~/.omega/provisioning/services.env`.
//!
//! The TUI Monitor tab has a Telegram-style wizard that asks for the
//! provisioning tokens (Vercel / Convex / GitHub / Stripe) and writes them here
//! via [`update_services_env`]. The `/omega-new-project` skill sources this file
//! to auto-create + wire external services for a fresh project.
//!
//! Write semantics are deliberately NON-DESTRUCTIVE: only non-empty updates are
//! applied, every comment / mode line / untouched key is preserved verbatim, and
//! a blank value never wipes an existing token (so re-running the wizard and
//! skipping a field keeps the old value). Atomic write, `0600` perms.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// `~/.omega/provisioning/services.env`.
pub fn services_env_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".omega/provisioning/services.env")
}

/// Current value of an `export KEY="..."` line, or `None` if unset/empty.
pub fn read_value(key: &str) -> Option<String> {
    let content = std::fs::read_to_string(services_env_path()).ok()?;
    for line in content.lines() {
        if let Some((k, v)) = parse_export(line) {
            if k == key {
                return if v.is_empty() { None } else { Some(v) };
            }
        }
    }
    None
}

/// Merge `updates` (key, value) into services.env. Empty values are skipped
/// (no-op, never overwrites). Preserves comments, ordering, and untouched keys.
/// Creates the file if absent. Atomic + `0600`.
pub fn update_services_env(updates: &[(String, String)]) -> Result<()> {
    update_services_env_at(&services_env_path(), updates)
}

/// Path-parameterised core of [`update_services_env`] (testable).
fn update_services_env_at(path: &std::path::Path, updates: &[(String, String)]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Only non-empty updates apply — a blank field leaves the existing value.
    let map: HashMap<&str, &str> = updates
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| (k.as_str(), v.trim()))
        .collect();

    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = String::new();

    for line in existing.lines() {
        if let Some((k, _)) = parse_export(line) {
            if let Some(newv) = map.get(k.as_str()) {
                out.push_str(&format!("export {}={}\n", k, shell_quote(newv)));
                seen.insert(k);
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    // Keys updated but not present in the file → append them.
    for (k, v) in &map {
        if !seen.contains(*k) {
            out.push_str(&format!("export {}={}\n", k, shell_quote(v)));
        }
    }

    let tmp = path.with_extension("env.tmp");
    std::fs::write(&tmp, &out)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// ─── Credential groups (multi-account, per-client) ──────────────────────────
//
// A client project needs its OWN accounts (Vercel/Convex/Clerk/Stripe/GitHub).
// Each group is a full credential set stored at
// `~/.omega/provisioning/groups/<slug>.env`. The `default` group IS the shared
// `services.env` (personal / works projects). At new-project time the user
// picks an existing group or creates a new one; the chosen group is recorded
// on the project so push / deploy / update all use the same account.

/// `~/.omega/provisioning/groups/` — one `<slug>.env` per client credential set.
pub fn groups_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".omega/provisioning/groups")
}

/// Sanitize a free-text group/client name into a safe filename stem.
pub fn sanitize_group(name: &str) -> String {
    let slug: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_string()
}

/// Resolve a group name to its env file. `default` / empty → the shared
/// `services.env`; any other name → `groups/<slug>.env`.
pub fn group_env_path(group: &str) -> PathBuf {
    let g = group.trim();
    if g.is_empty() || g.eq_ignore_ascii_case("default") {
        services_env_path()
    } else {
        groups_dir().join(format!("{}.env", sanitize_group(g)))
    }
}

/// List credential groups — `default` (the shared set) always first, then each
/// `groups/<slug>.env` by name.
pub fn list_groups() -> Vec<String> {
    let mut groups = vec!["default".to_string()];
    if let Ok(entries) = std::fs::read_dir(groups_dir()) {
        let mut named: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("env") {
                    p.file_stem().and_then(|s| s.to_str()).map(str::to_string)
                } else {
                    None
                }
            })
            .collect();
        named.sort();
        groups.extend(named);
    }
    groups
}

/// Read a key from a specific group's env (`None` if unset/empty/missing).
pub fn read_value_in(group: &str, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(group_env_path(group)).ok()?;
    for line in content.lines() {
        if let Some((k, v)) = parse_export(line) {
            if k == key {
                return if v.is_empty() { None } else { Some(v) };
            }
        }
    }
    None
}

/// Merge updates into a specific group's env (non-destructive, atomic, 0600).
pub fn update_group_env(group: &str, updates: &[(String, String)]) -> Result<()> {
    update_services_env_at(&group_env_path(group), updates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_preserves_comments_and_skips_blanks() {
        let dir = std::env::temp_dir().join(format!("omega-prov-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("services.env");
        std::fs::write(
            &path,
            "# header comment\nexport VERCEL_TOKEN=\"\"\nexport STRIPE_MODE=\"single\"\n",
        )
        .unwrap();

        // Set Vercel, skip Stripe (blank), add a key not yet present.
        update_services_env_at(
            &path,
            &[
                ("VERCEL_TOKEN".into(), "vc_live_123".into()),
                ("STRIPE_SECRET_KEY".into(), "".into()), // blank → no-op
                ("CONVEX_TEAM_TOKEN".into(), "cv_abc".into()), // new key → appended
            ],
        )
        .unwrap();

        let out = std::fs::read_to_string(&path).unwrap();
        assert!(out.contains("# header comment"), "comment preserved");
        assert!(out.contains("export VERCEL_TOKEN=\"vc_live_123\""), "vercel set");
        assert!(out.contains("export STRIPE_MODE=\"single\""), "mode line preserved");
        assert!(!out.contains("STRIPE_SECRET_KEY"), "blank not written");
        assert!(out.contains("export CONVEX_TEAM_TOKEN=\"cv_abc\""), "new key appended");

        // Re-run with all blanks → existing values must survive untouched.
        update_services_env_at(&path, &[("VERCEL_TOKEN".into(), "".into())]).unwrap();
        let out2 = std::fs::read_to_string(&path).unwrap();
        assert!(out2.contains("export VERCEL_TOKEN=\"vc_live_123\""), "blank kept old value");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn credential_groups_resolve_and_sanitize() {
        // default / empty → the shared services.env (back-compat).
        assert_eq!(group_env_path("default"), services_env_path());
        assert_eq!(group_env_path(""), services_env_path());
        // a named client → groups/<slug>.env
        let p = group_env_path("Acme Corp!");
        assert!(
            p.to_string_lossy().ends_with("groups/acme-corp.env"),
            "got {}",
            p.display()
        );
        assert_eq!(sanitize_group("Acme Corp!"), "acme-corp");
        assert_eq!(sanitize_group("  Big_Client-2  "), "big_client-2");
    }
}

/// Parse `export KEY="value"` / `export KEY=value` → (KEY, value). `None` for any
/// line that isn't an env-var export (comments, blanks, prose all fall through).
fn parse_export(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("export ")?;
    let (k, v) = rest.split_once('=')?;
    let key = k.trim().to_string();
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        return None;
    }
    let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
    Some((key, val))
}

/// Double-quote a value for a sourceable env file, escaping `\` and `"`.
fn shell_quote(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}
