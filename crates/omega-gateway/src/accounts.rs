//! Isolated per-account credential slots + metadata registry.
//!
//! Storage layout under the gateway data dir:
//! `<gateway_dir>/accounts/accounts.json`  (0600, registry: `Vec<Account>` — METADATA only)
//! `<gateway_dir>/accounts/<slug>/`        (0700, one hardened dir per slot — the
//!                                          credential material a provider CLI writes
//!                                          there is opaque to this module and never
//!                                          touched or serialized by it)

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::{Account, AccountKind};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Slug charset: `[a-z0-9-]{1,32}`, non-empty. Deliberately excludes `.`/`/`
/// so a slug can never traverse out of the accounts dir (e.g. `../x`).
pub fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub struct AccountStore {
    accounts_dir: PathBuf,
}

impl AccountStore {
    /// Opens (creating if needed) the account store rooted at `<gateway_dir>/accounts`.
    pub fn open(gateway_dir: &Path) -> Self {
        let accounts_dir = gateway_dir.join("accounts");
        std::fs::create_dir_all(&accounts_dir).ok();
        harden_dir(&accounts_dir);
        Self { accounts_dir }
    }

    fn registry_path(&self) -> PathBuf {
        self.accounts_dir.join("accounts.json")
    }

    /// The hardened slot directory for `slug` — where a provider CLI's
    /// credential material for that slot lives. This module never reads or
    /// writes inside it beyond creating/removing the directory itself.
    pub fn slot_dir(&self, slug: &str) -> PathBuf {
        self.accounts_dir.join(slug)
    }

    fn read_registry(&self) -> Vec<Account> {
        let Ok(text) = std::fs::read_to_string(self.registry_path()) else {
            return Vec::new();
        };
        match serde_json::from_str(&text) {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!("corrupted accounts.json: {e}");
                Vec::new()
            }
        }
    }

    fn write_registry(&self, accounts: &[Account]) {
        let path = self.registry_path();
        match serde_json::to_string_pretty(accounts) {
            Ok(text) => {
                // Atomic write: temp sibling + rename, same discipline as
                // chat_store's meta.json so a crash mid-write never leaves a
                // torn accounts.json.
                let tmp = path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp, text) {
                    tracing::error!("failed to write {}: {e}", tmp.display());
                } else {
                    harden_file(&tmp);
                    if let Err(e) = std::fs::rename(&tmp, &path) {
                        tracing::error!("failed to rename {} -> {}: {e}", tmp.display(), path.display());
                    } else {
                        harden_file(&path);
                    }
                }
            }
            Err(e) => tracing::error!("failed to serialize accounts registry: {e}"),
        }
    }

    /// Creates a new isolated credential slot: validates the slug, makes its
    /// hardened (0700) directory, and persists the metadata registry. The
    /// first account created for a given `kind` becomes that kind's default.
    pub fn create_slot(&self, slug: &str, label: &str, kind: AccountKind) -> Result<Account> {
        if !valid_slug(slug) {
            bail!("invalid slug: {slug:?} (must be [a-z0-9-]{{1,32}})");
        }
        let mut accounts = self.read_registry();
        if accounts.iter().any(|a| a.slug == slug) {
            bail!("account slug already exists: {slug}");
        }
        let is_default = !accounts.iter().any(|a| a.kind == kind);

        let dir = self.slot_dir(slug);
        std::fs::create_dir_all(&dir)?;
        harden_dir(&dir);

        let account = Account {
            slug: slug.to_string(),
            label: label.to_string(),
            kind,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_default,
        };
        accounts.push(account.clone());
        self.write_registry(&accounts);
        Ok(account)
    }

    /// All known accounts (any kind), in registry order.
    pub fn list(&self) -> Vec<Account> {
        self.read_registry()
    }

    /// Reads one account's metadata by slug.
    pub fn get(&self, slug: &str) -> Option<Account> {
        self.read_registry().into_iter().find(|a| a.slug == slug)
    }

    /// Removes an account: deletes its registry entry AND its slot directory
    /// (credentials and all). Returns false if the slug was unknown.
    pub fn remove(&self, slug: &str) -> bool {
        let mut accounts = self.read_registry();
        let before = accounts.len();
        accounts.retain(|a| a.slug != slug);
        if accounts.len() == before {
            return false;
        }
        self.write_registry(&accounts);
        if let Err(e) = std::fs::remove_dir_all(self.slot_dir(slug)) {
            tracing::warn!("failed to remove slot dir for {slug}: {e}");
        }
        true
    }

    /// Makes `slug` the default for its kind, clearing the default flag on
    /// every other account of the SAME kind (accounts of other kinds are
    /// untouched). Returns false if the slug is unknown.
    pub fn set_default(&self, slug: &str) -> bool {
        let mut accounts = self.read_registry();
        let Some(kind) = accounts.iter().find(|a| a.slug == slug).map(|a| a.kind) else {
            return false;
        };
        for a in accounts.iter_mut().filter(|a| a.kind == kind) {
            a.is_default = a.slug == slug;
        }
        self.write_registry(&accounts);
        true
    }

    /// The current default account for `kind`, if any.
    pub fn default_for(&self, kind: AccountKind) -> Option<Account> {
        self.read_registry().into_iter().find(|a| a.kind == kind && a.is_default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_slug_rejects_bad_input() {
        assert!(!valid_slug("../x"), "path traversal must be rejected");
        assert!(!valid_slug("UP"), "uppercase must be rejected");
        assert!(!valid_slug(""), "empty must be rejected");
        assert!(!valid_slug(&"a".repeat(33)), ">32 chars must be rejected");
    }

    #[test]
    fn valid_slug_accepts_good_input() {
        assert!(valid_slug("work-1"));
        assert!(valid_slug("a"));
        assert!(valid_slug(&"a".repeat(32)));
    }

    #[test]
    fn create_slot_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let account = store.create_slot("work-1", "Work account", AccountKind::Claude).unwrap();

        assert_eq!(account.slug, "work-1");
        assert_eq!(account.label, "Work account");
        assert_eq!(account.kind, AccountKind::Claude);
        assert!(account.is_default, "first account of a kind is its default");

        let fetched = store.get("work-1").expect("account should exist");
        assert_eq!(fetched.slug, account.slug);
        assert_eq!(fetched.label, account.label);
    }

    #[test]
    fn create_slot_rejects_invalid_slug() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.create_slot("../x", "bad", AccountKind::Claude).is_err());
        assert!(store.create_slot("UP", "bad", AccountKind::Claude).is_err());
    }

    #[test]
    fn create_slot_rejects_duplicate_slug() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store.create_slot("work-1", "Work", AccountKind::Claude).unwrap();
        assert!(store.create_slot("work-1", "Again", AccountKind::Claude).is_err());
    }

    #[test]
    fn get_unknown_slug_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn list_returns_all_accounts() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.list().is_empty());
        store.create_slot("a", "A", AccountKind::Claude).unwrap();
        store.create_slot("b", "B", AccountKind::Codex).unwrap();
        let listed = store.list();
        assert_eq!(listed.len(), 2);
    }

    #[test]
    fn first_of_kind_is_default_independently_per_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let claude1 = store.create_slot("c1", "Claude 1", AccountKind::Claude).unwrap();
        assert!(claude1.is_default);

        let codex1 = store.create_slot("x1", "Codex 1", AccountKind::Codex).unwrap();
        assert!(codex1.is_default, "first Codex account is its own kind's default");

        let claude2 = store.create_slot("c2", "Claude 2", AccountKind::Claude).unwrap();
        assert!(!claude2.is_default, "second Claude account is not default");
    }

    #[test]
    fn set_default_moves_default_within_kind_other_kind_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store.create_slot("c1", "Claude 1", AccountKind::Claude).unwrap();
        store.create_slot("c2", "Claude 2", AccountKind::Claude).unwrap();
        let codex1 = store.create_slot("x1", "Codex 1", AccountKind::Codex).unwrap();

        assert!(store.set_default("c2"));

        let c1 = store.get("c1").unwrap();
        let c2 = store.get("c2").unwrap();
        let x1 = store.get("x1").unwrap();
        assert!(!c1.is_default, "loser is cleared");
        assert!(c2.is_default, "winner is set");
        assert!(x1.is_default, "other kind's default untouched");
        assert_eq!(codex1.slug, x1.slug);
    }

    #[test]
    fn set_default_unknown_slug_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(!store.set_default("nonexistent"));
    }

    #[test]
    fn default_for_returns_current_default_per_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.default_for(AccountKind::Claude).is_none());
        store.create_slot("c1", "Claude 1", AccountKind::Claude).unwrap();
        store.create_slot("c2", "Claude 2", AccountKind::Claude).unwrap();
        store.set_default("c2");
        assert_eq!(store.default_for(AccountKind::Claude).unwrap().slug, "c2");
    }

    #[test]
    fn remove_deletes_dir_and_registry_entry() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store.create_slot("work-1", "Work", AccountKind::Claude).unwrap();
        let slot_dir = store.slot_dir("work-1");
        assert!(slot_dir.exists());

        assert!(store.remove("work-1"));
        assert!(store.get("work-1").is_none());
        assert!(!slot_dir.exists(), "slot dir must be deleted");
    }

    #[test]
    fn remove_unknown_slug_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(!store.remove("nonexistent"));
    }

    #[test]
    fn persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = AccountStore::open(dir.path());
            store.create_slot("work-1", "Work", AccountKind::Claude).unwrap();
            store.create_slot("work-2", "Work 2", AccountKind::Claude).unwrap();
            store.set_default("work-2");
        }

        let store2 = AccountStore::open(dir.path());
        assert_eq!(store2.list().len(), 2);
        assert_eq!(store2.default_for(AccountKind::Claude).unwrap().slug, "work-2");
    }

    #[cfg(unix)]
    #[test]
    fn slot_dir_is_0700_and_registry_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store.create_slot("work-1", "Work", AccountKind::Claude).unwrap();

        let slot_dir = dir.path().join("accounts").join("work-1");
        let dir_mode = std::fs::metadata(&slot_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "slot dir must be 0700");

        let registry_path = dir.path().join("accounts").join("accounts.json");
        let registry_mode = std::fs::metadata(&registry_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(registry_mode, 0o600, "accounts.json must be 0600");
    }
}
