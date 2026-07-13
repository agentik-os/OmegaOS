//! Pre-trust a project folder in `~/.codex/config.toml` so Codex never shows
//! the "Do you trust the contents of this directory?" prompt for it.
//!
//! Why this exists: a Codex pane launched by omega is detached — nobody is
//! watching it at boot. Codex blocks on the trust prompt before it renders
//! anything, so the pane looks dead and the session is unusable until someone
//! finds it and presses Enter. Claude has the same failure mode and dodges it
//! via `claude_trust.rs`; this is the Codex half of that pattern, called from
//! the same `omega trust-dir` entry point.
//!
//! Codex keys the trust table by the CANONICAL path (on macOS `/tmp/x` is
//! stored as `/private/tmp/x`), so callers must canonicalize before calling —
//! `cmd_trust_dir` does. The config is co-owned by the Codex desktop app and
//! holds MCP servers, plugins and desktop settings, so we edit it surgically
//! with toml_edit rather than round-tripping it through a value model.

use std::path::Path;
use toml_edit::{value, DocumentMut, Item, Table};

/// Mark `dir` as trusted in `~/.codex/config.toml` (atomic temp+rename).
///
/// Returns `Ok(true)` if the file was updated, `Ok(false)` if the folder was
/// already trusted. A corrupt/unparseable config aborts with `Err` and leaves
/// the file untouched — the worst case is the prompt showing once, never a
/// clobbered Codex config.
pub fn trust_dir(dir: &Path) -> std::io::Result<bool> {
    let home = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no home dir"))?;
    let codex_home = std::env::var("CODEX_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| home.join(".codex"));
    let cfg_path = codex_home.join("config.toml");

    let mut doc: DocumentMut = if cfg_path.exists() {
        std::fs::read_to_string(&cfg_path)?
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    } else {
        DocumentMut::new()
    };

    // `projects` is a table of quoted absolute paths → { trust_level }.
    let projects = doc.entry("projects").or_insert_with(|| {
        let mut t = Table::new();
        // Implicit: render as [projects."/path"] headers, not a [projects]
        // header followed by orphan sub-tables.
        t.set_implicit(true);
        Item::Table(t)
    });
    let projects = projects.as_table_mut().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "`projects` is not a table")
    })?;
    projects.set_implicit(true);

    let key = dir.to_string_lossy().to_string();
    let entry = projects
        .entry(&key)
        .or_insert_with(|| Item::Table(Table::new()));
    let entry = entry.as_table_mut().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "project entry is not a table",
        )
    })?;

    if entry.get("trust_level").and_then(|v| v.as_str()) == Some("trusted") {
        return Ok(false);
    }
    entry["trust_level"] = value("trusted");

    if let Some(parent) = cfg_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Atomic write: temp file in the same dir + rename, so the Codex app never
    // reads a torn config and a crash never truncates the real one.
    let tmp = cfg_path.with_extension(format!("toml.trust-{}", std::process::id()));
    std::fs::write(&tmp, doc.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, &cfg_path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use toml_edit::{value, DocumentMut, Item, Table};

    /// The edit must add the trust entry WITHOUT disturbing the MCP servers,
    /// plugins and desktop settings the Codex app owns in the same file.
    fn trust_in(src: &str, path: &str) -> String {
        let mut doc: DocumentMut = src.parse().unwrap();
        let projects = doc.entry("projects").or_insert_with(|| {
            let mut t = Table::new();
            t.set_implicit(true);
            Item::Table(t)
        });
        let projects = projects.as_table_mut().unwrap();
        projects.set_implicit(true);
        let entry = projects
            .entry(path)
            .or_insert_with(|| Item::Table(Table::new()));
        entry.as_table_mut().unwrap()["trust_level"] = value("trusted");
        doc.to_string()
    }

    #[test]
    fn preserves_existing_config_and_adds_trust() {
        let src = "model = \"gpt-5.6-terra\"\n\n[mcp_servers.node_repl]\ncommand = \"node_repl\"\n";
        let out = trust_in(src, "/private/tmp/proj");
        assert!(out.contains("model = \"gpt-5.6-terra\""));
        assert!(out.contains("[mcp_servers.node_repl]"));
        assert!(out.contains("command = \"node_repl\""));
        let doc: DocumentMut = out.parse().unwrap();
        assert_eq!(
            doc["projects"]["/private/tmp/proj"]["trust_level"].as_str(),
            Some("trusted")
        );
    }

    #[test]
    fn upgrades_an_untrusted_entry_in_place() {
        let src = "[projects.\"/private/tmp/proj\"]\ntrust_level = \"untrusted\"\n";
        let out = trust_in(src, "/private/tmp/proj");
        let doc: DocumentMut = out.parse().unwrap();
        assert_eq!(
            doc["projects"]["/private/tmp/proj"]["trust_level"].as_str(),
            Some("trusted")
        );
        assert!(!out.contains("untrusted"));
    }
}
