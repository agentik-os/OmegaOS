//! Centralized credential storage for OmegaOS.
//!
//! All provider credentials live in `~/.omega/credentials/`. Each provider has:
//!   - `~/.omega/credentials/<provider>.json`            — active credentials
//!   - `~/.omega/credentials/accounts/<provider>-<name>.json` — saved accounts
//!
//! Legacy LLM CLIs (claude, codex, gemini) expect their credentials at known
//! paths (~/.claude/.credentials.json, ~/.codex/auth.json, etc.). We make those
//! paths symlinks to the canonical files under ~/.omega/credentials/ so:
//!   - The CLI still finds its creds.
//!   - OmegaOS owns the source of truth.
//!   - Account switching = atomic rewrite of the canonical file (symlink stays).
//!
//! ## Relationship to `account.rs` (KNOWN DESIGN-DEBT)
//! `account.rs` is a SEPARATE, Claude-only surface for the `/account` billing +
//! switch view (keyed by `~/.claude/accounts/accounts-meta.json`). The two share
//! the active Claude file (account.rs writes via `oauth::credentials_path()` =
//! this store's `claude.json`), so the ACTIVE account is consistent — but the two
//! SAVED-profile lists are not unified (a Claude account saved here via `/model`
//! is not listed by `/account`, and vice versa). See the account.rs module doc for
//! the full boundary + why a merge is deferred. Keep both lists in sync if you
//! change Claude account-switching.

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::providers::ProvidersConfig;

/// Secrets-at-rest: chmod a file to 0600 (owner read/write only). No-op target
/// must already exist. Called before the atomic rename on every credential write
/// so a multi-user host never sees another user's OAuth tokens / API keys.
/// `pub(crate)` so other secret-writers (e.g. providers.toml) share one impl
/// instead of duplicating the unix/non-unix split.
#[cfg(unix)]
pub(crate) fn chmod_600(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}
#[cfg(not(unix))]
pub(crate) fn chmod_600(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Manager for ~/.omega/credentials/ — every provider's creds + saved accounts.
#[derive(Debug, Clone)]
pub struct CredentialStore {
    base_dir: PathBuf,
}

impl CredentialStore {
    pub fn new() -> Result<Self> {
        let base = omega_dir().join("credentials");
        std::fs::create_dir_all(base.join("accounts"))
            .with_context(|| format!("creating {}", base.display()))?;
        // The credentials tree holds OAuth tokens + API keys — owner-only (0700)
        // so a multi-user host can't traverse into another user's secrets.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o700));
            let _ = std::fs::set_permissions(base.join("accounts"), std::fs::Permissions::from_mode(0o700));
        }
        Ok(Self { base_dir: base })
    }

    /// Path to the active credentials file for a provider.
    pub fn active_path(&self, provider: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", provider))
    }

    /// Path to a named saved account for a provider.
    pub fn account_path(&self, provider: &str, name: &str) -> PathBuf {
        self.base_dir
            .join("accounts")
            .join(format!("{}-{}.json", provider, sanitize(name)))
    }

    /// Read the active credentials for a provider as JSON.
    pub fn read(&self, provider: &str) -> Result<Value> {
        let path = self.active_path(provider);
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let v: Value = serde_json::from_str(&raw)
            .with_context(|| format!("parsing {}", path.display()))?;
        Ok(v)
    }

    /// Write the active credentials for a provider. Atomic via tmp+rename.
    pub fn write(&self, provider: &str, data: &Value) -> Result<()> {
        let path = self.active_path(provider);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(data)?;
        std::fs::write(&tmp, json).with_context(|| format!("writing {}", tmp.display()))?;
        // Secrets at rest: 0600 on the tmp BEFORE rename (rename adopts the tmp's
        // mode, so chmod-after-rename would still leave a world-readable window).
        chmod_600(&tmp).with_context(|| format!("chmod 600 {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    }

    /// List saved account names for a provider (just the `<name>` part).
    pub fn list_accounts(&self, provider: &str) -> Vec<String> {
        let accounts = self.base_dir.join("accounts");
        let prefix = format!("{}-", provider);
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&accounts) else {
            return out;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Some(name) = stem.strip_prefix(&prefix) {
                out.push(name.to_string());
            }
        }
        out.sort();
        out
    }

    /// Switch the active provider credentials to a named saved account.
    /// Copies the account file over the active file (atomic via tmp+rename).
    pub fn switch_account(&self, provider: &str, name: &str) -> Result<()> {
        let from = self.account_path(provider, name);
        if !from.exists() {
            anyhow::bail!(
                "account not found: {} (looked at {})",
                name,
                from.display()
            );
        }
        let to = self.active_path(provider);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let tmp = to.with_extension("json.switching");
        std::fs::copy(&from, &tmp)
            .with_context(|| format!("copy {} -> {}", from.display(), tmp.display()))?;
        chmod_600(&tmp).with_context(|| format!("chmod 600 {}", tmp.display()))?;
        std::fs::rename(&tmp, &to)
            .with_context(|| format!("rename {} -> {}", tmp.display(), to.display()))?;
        Ok(())
    }

    /// Save the currently active credentials as a named account.
    pub fn save_as_account(&self, provider: &str, name: &str) -> Result<()> {
        let from = self.active_path(provider);
        if !from.exists() {
            anyhow::bail!(
                "no active credentials for {} (no file at {})",
                provider,
                from.display()
            );
        }
        let to = self.account_path(provider, name);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::copy(&from, &to)
            .with_context(|| format!("copy {} -> {}", from.display(), to.display()))?;
        chmod_600(&to).with_context(|| format!("chmod 600 {}", to.display()))?;
        Ok(())
    }

    /// Ensure the legacy path for a provider exists as a symlink to the
    /// canonical file in ~/.omega/credentials/. Creates parent dirs as needed.
    /// No-op if the legacy path already points to the canonical file.
    pub fn ensure_legacy_symlink(&self, provider: &str) -> Result<()> {
        let Some(legacy) = legacy_path_for(provider) else {
            return Ok(());
        };
        let canonical = self.active_path(provider);
        if let Some(parent) = legacy.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // If legacy is already a symlink pointing at the right target, done.
        if let Ok(meta) = std::fs::symlink_metadata(&legacy) {
            if meta.file_type().is_symlink() {
                if let Ok(target) = std::fs::read_link(&legacy) {
                    if target == canonical {
                        return Ok(());
                    }
                }
                // wrong target — replace
                std::fs::remove_file(&legacy).ok();
            } else if meta.is_file() {
                // Real file at legacy path. If canonical doesn't exist, migrate.
                if !canonical.exists() {
                    if let Some(p) = canonical.parent() {
                        std::fs::create_dir_all(p).ok();
                    }
                    std::fs::rename(&legacy, &canonical).with_context(|| {
                        format!(
                            "migrating {} -> {}",
                            legacy.display(),
                            canonical.display()
                        )
                    })?;
                } else {
                    // Both exist — keep canonical, back up + remove legacy.
                    let backup = legacy.with_extension("json.pre-omega");
                    let _ = std::fs::rename(&legacy, &backup);
                }
            }
        }

        // At this point legacy does not exist. Create symlink if canonical exists.
        if canonical.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&canonical, &legacy).with_context(|| {
                format!(
                    "symlink {} -> {}",
                    legacy.display(),
                    canonical.display()
                )
            })?;
            #[cfg(not(unix))]
            std::fs::copy(&canonical, &legacy)?;
        }
        Ok(())
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Map a provider id to the legacy path its CLI expects.
/// Returns None for providers without a fixed legacy location (api-key only).
pub fn legacy_path_for(provider: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match provider {
        "claude" => Some(home.join(".claude").join(".credentials.json")),
        "codex" => Some(home.join(".codex").join("auth.json")),
        "gemini" => Some(home.join(".config").join("gemini").join("oauth_creds.json")),
        // api-key providers store config in providers.toml; no legacy path.
        _ => None,
    }
}

/// Convenience: list providers known to ProvidersConfig.
pub fn known_providers() -> Vec<&'static str> {
    ProvidersConfig::all_providers()
}

/// Delegates to the single canonical resolver (config::omega_dir) so credentials
/// honor $OMEGA_DIR + the consolidated ~/OmegaOS/System layout, instead of
/// hardcoding ~/.omega and diverging from where config.toml actually lives.
fn omega_dir() -> PathBuf {
    crate::config::omega_dir()
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{Mutex, MutexGuard};

    // fresh_store mutates the process-global HOME, and cargo runs tests in
    // parallel, so without this lock two tests clobber each other's HOME and a
    // legacy symlink can land in the wrong tmp dir. Serialize HOME-touching tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn omega_dir_override_relocates_credentials() {
        // $OMEGA_DIR must relocate the WHOLE secret tree, not just config.toml.
        // credentials now share config::omega_dir() (was: a hardcoded ~/.omega),
        // so setting the override lands the credential store under it.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var("OMEGA_DIR").ok();
        std::env::set_var("OMEGA_DIR", tmp.path());
        let store = CredentialStore::new().unwrap();
        assert!(
            store.base_dir().starts_with(tmp.path()),
            "credentials base {:?} must be under $OMEGA_DIR {:?}",
            store.base_dir(),
            tmp.path()
        );
        match prev {
            Some(v) => std::env::set_var("OMEGA_DIR", v),
            None => std::env::remove_var("OMEGA_DIR"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn secret_writes_are_owner_only_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        // write() — the active-creds path
        store.write("codex", &json!({"refresh_token": "sk-secret-xyz"})).unwrap();
        let mode = std::fs::metadata(store.active_path("codex")).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "active creds must be 0600, got {:o}", mode & 0o777);
        // save_as_account() — the saved-profile copy
        store.save_as_account("codex", "work").unwrap();
        let amode = std::fs::metadata(store.account_path("codex", "work")).unwrap().permissions().mode();
        assert_eq!(amode & 0o077, 0, "saved account must be 0600, got {:o}", amode & 0o777);
        // switch_account() — the copy-over-active path
        store.write("codex", &json!({"refresh_token": "v2"})).unwrap();
        store.switch_account("codex", "work").unwrap();
        let smode = std::fs::metadata(store.active_path("codex")).unwrap().permissions().mode();
        assert_eq!(smode & 0o077, 0, "switched creds must be 0600, got {:o}", smode & 0o777);
    }

    fn fresh_store(tmp: &Path) -> (CredentialStore, MutexGuard<'static, ()>) {
        let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("HOME", tmp);
        // Force re-eval of omega_dir
        std::fs::create_dir_all(tmp.join(".omega").join("credentials").join("accounts")).unwrap();
        (CredentialStore::new().unwrap(), guard)
    }

    #[test]
    fn write_read_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        let data = json!({"apiKey": "sk-abc123", "model": "gpt-5"});
        store.write("codex", &data).unwrap();
        let read = store.read("codex").unwrap();
        assert_eq!(read, data);
    }

    #[test]
    fn save_and_switch_account() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store.write("codex", &json!({"k": "v1"})).unwrap();
        store.save_as_account("codex", "work").unwrap();
        store.write("codex", &json!({"k": "v2"})).unwrap();
        store.save_as_account("codex", "perso").unwrap();
        let mut accs = store.list_accounts("codex");
        accs.sort();
        assert_eq!(accs, vec!["perso", "work"]);

        store.switch_account("codex", "work").unwrap();
        let read = store.read("codex").unwrap();
        assert_eq!(read["k"], "v1");
    }

    #[test]
    fn ensure_legacy_symlink_creates_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store.write("claude", &json!({"claudeAiOauth": {}})).unwrap();
        store.ensure_legacy_symlink("claude").unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        let meta = std::fs::symlink_metadata(&legacy).unwrap();
        assert!(meta.file_type().is_symlink());
    }

    #[test]
    fn ensure_legacy_migrates_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        // Pre-create a real file at the legacy path with NO canonical file.
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let legacy = tmp.path().join(".claude").join(".credentials.json");
        std::fs::write(&legacy, "{\"k\":1}").unwrap();
        let (store, _env) = fresh_store(tmp.path());
        store.ensure_legacy_symlink("claude").unwrap();
        // canonical should now exist
        assert!(store.active_path("claude").exists());
        // legacy should now be a symlink
        let meta = std::fs::symlink_metadata(&legacy).unwrap();
        assert!(meta.file_type().is_symlink());
    }

    #[test]
    fn sanitize_strips_bad_chars() {
        assert_eq!(sanitize("hello world"), "hello_world");
        assert_eq!(sanitize("ok-name_123"), "ok-name_123");
    }
}
