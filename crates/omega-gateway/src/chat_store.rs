//! Persisted chat metadata + transcripts.
//!
//! Storage layout under the gateway data dir:
//! `<gateway_dir>/chats/<id>/meta.json`        (0600, ChatMeta)
//! `<gateway_dir>/chats/<id>/transcript.jsonl`  (0600, one ChatMessage per line)
//! Each `<gateway_dir>/chats/<id>/` dir is hardened to 0700.

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::{ChatAgent, ChatMeta, ChatMessage};
use crate::util::random_hex;
use std::path::{Path, PathBuf};

pub struct ChatStore {
    chats_dir: PathBuf,
}

impl ChatStore {
    /// Opens (creating if needed) the chat store rooted at `<gateway_dir>/chats`.
    pub fn open(gateway_dir: &Path) -> Self {
        let chats_dir = gateway_dir.join("chats");
        std::fs::create_dir_all(&chats_dir).ok();
        harden_dir(&chats_dir);
        Self { chats_dir }
    }

    fn dir_for(&self, id: &str) -> PathBuf {
        self.chats_dir.join(id)
    }

    fn meta_path(&self, id: &str) -> PathBuf {
        self.dir_for(id).join("meta.json")
    }

    fn transcript_path(&self, id: &str) -> PathBuf {
        self.dir_for(id).join("transcript.jsonl")
    }

    fn write_meta(&self, meta: &ChatMeta) {
        let dir = self.dir_for(&meta.id);
        std::fs::create_dir_all(&dir).ok();
        harden_dir(&dir);
        let path = self.meta_path(&meta.id);
        match serde_json::to_string_pretty(meta) {
            Ok(text) => {
                // Atomic write: write to a temp sibling then rename, so a crash
                // mid-write or a concurrent turn never leaves a torn meta.json
                // that get() would read as None (the chat would transiently
                // vanish from list()). rename(2) is atomic on the same fs.
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
            Err(e) => tracing::error!("failed to serialize chat meta {}: {e}", meta.id),
        }
    }

    /// Creates a new chat, persists its metadata, and returns it.
    pub fn create(&self, agent: ChatAgent, cwd: String, title: Option<String>) -> ChatMeta {
        let now = chrono::Utc::now().to_rfc3339();
        let meta = ChatMeta {
            id: random_hex(8),
            title,
            agent,
            cwd,
            created_at: now.clone(),
            updated_at: now,
            provider_session_id: None,
        };
        self.write_meta(&meta);
        meta
    }

    /// All known chats, most recently updated first.
    pub fn list(&self) -> Vec<ChatMeta> {
        let mut metas = Vec::new();
        let Ok(entries) = std::fs::read_dir(&self.chats_dir) else {
            return metas;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            if !file_type.is_dir() {
                continue;
            }
            let Some(id) = entry.file_name().to_str().map(str::to_string) else { continue };
            if let Some(meta) = self.get(&id) {
                metas.push(meta);
            }
        }
        metas.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        metas
    }

    /// Reads one chat's metadata by id.
    pub fn get(&self, id: &str) -> Option<ChatMeta> {
        let text = std::fs::read_to_string(self.meta_path(id)).ok()?;
        match serde_json::from_str(&text) {
            Ok(meta) => Some(meta),
            Err(e) => {
                tracing::warn!("corrupted meta.json for chat {id}: {e}");
                None
            }
        }
    }

    /// Appends a message to the chat's transcript and bumps `updated_at`.
    pub fn append_message(&self, id: &str, msg: &ChatMessage) {
        let dir = self.dir_for(id);
        std::fs::create_dir_all(&dir).ok();
        harden_dir(&dir);
        let path = self.transcript_path(id);
        let Ok(line) = serde_json::to_string(msg) else {
            tracing::error!("failed to serialize chat message for {id}");
            return;
        };
        use std::io::Write;
        let file = std::fs::OpenOptions::new().create(true).append(true).open(&path);
        match file {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{line}") {
                    tracing::error!("failed to append to {}: {e}", path.display());
                }
                harden_file(&path);
            }
            Err(e) => tracing::error!("failed to open {}: {e}", path.display()),
        }

        if let Some(mut meta) = self.get(id) {
            meta.updated_at = chrono::Utc::now().to_rfc3339();
            self.write_meta(&meta);
        }
    }

    /// The full transcript for a chat, in append order. Empty if the chat is unknown.
    pub fn transcript(&self, id: &str) -> Vec<ChatMessage> {
        let Ok(text) = std::fs::read_to_string(self.transcript_path(id)) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|line| match serde_json::from_str(line) {
                Ok(msg) => Some(msg),
                Err(e) => {
                    tracing::warn!("skipping corrupted transcript line for chat {id}: {e}");
                    None
                }
            })
            .collect()
    }

    /// Records the provider (claude/codex) resume session id for a chat.
    pub fn set_provider_session(&self, id: &str, provider_session_id: &str) {
        if let Some(mut meta) = self.get(id) {
            meta.provider_session_id = Some(provider_session_id.to_string());
            meta.updated_at = chrono::Utc::now().to_rfc3339();
            self.write_meta(&meta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp/proj".to_string(), Some("hi".to_string()));

        let fetched = store.get(&meta.id).expect("chat should exist");
        assert_eq!(fetched.id, meta.id);
        assert_eq!(fetched.title.as_deref(), Some("hi"));
        assert_eq!(fetched.agent, ChatAgent::Claude);
        assert_eq!(fetched.cwd, "/tmp/proj");
        assert!(fetched.provider_session_id.is_none());
    }

    #[test]
    fn get_unknown_id_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn append_two_messages_returns_both_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Codex, "/tmp".to_string(), None);

        store.append_message(
            &meta.id,
            &ChatMessage { role: "user".to_string(), text: "first".to_string(), ts: "t1".to_string() },
        );
        store.append_message(
            &meta.id,
            &ChatMessage { role: "assistant".to_string(), text: "second".to_string(), ts: "t2".to_string() },
        );

        let transcript = store.transcript(&meta.id);
        assert_eq!(transcript.len(), 2);
        assert_eq!(transcript[0].text, "first");
        assert_eq!(transcript[1].text, "second");
    }

    #[test]
    fn transcript_of_unknown_chat_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.transcript("nonexistent").is_empty());
    }

    #[test]
    fn create_bumps_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.list().is_empty());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None);
        let listed = store.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, meta.id);
    }

    #[test]
    fn list_sorted_updated_at_desc() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let a = store.create(ChatAgent::Claude, "/tmp".to_string(), None);
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = store.create(ChatAgent::Claude, "/tmp".to_string(), None);
        std::thread::sleep(std::time::Duration::from_millis(5));
        // bump a's updated_at past b's by appending to it
        store.append_message(
            &a.id,
            &ChatMessage { role: "user".to_string(), text: "bump".to_string(), ts: "t".to_string() },
        );

        let listed = store.list();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, a.id, "most recently updated chat comes first");
        assert_eq!(listed[1].id, b.id);
    }

    #[test]
    fn set_provider_session_persists() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None);
        assert!(meta.provider_session_id.is_none());

        store.set_provider_session(&meta.id, "claude-session-abc");

        let fetched = store.get(&meta.id).unwrap();
        assert_eq!(fetched.provider_session_id.as_deref(), Some("claude-session-abc"));
    }

    #[test]
    fn persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let meta = {
            let store = ChatStore::open(dir.path());
            let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), Some("persisted".to_string()));
            store.append_message(
                &meta.id,
                &ChatMessage { role: "user".to_string(), text: "hello".to_string(), ts: "t1".to_string() },
            );
            meta
        };

        // fresh store over the same dir
        let store2 = ChatStore::open(dir.path());
        let fetched = store2.get(&meta.id).expect("chat should survive reopen");
        assert_eq!(fetched.title.as_deref(), Some("persisted"));
        let transcript = store2.transcript(&meta.id);
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].text, "hello");
        assert_eq!(store2.list().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn chat_dir_is_0700_and_meta_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None);

        let chat_dir = dir.path().join("chats").join(&meta.id);
        let dir_mode = std::fs::metadata(&chat_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "chat dir must be 0700");

        let meta_mode = std::fs::metadata(chat_dir.join("meta.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(meta_mode, 0o600, "meta.json must be 0600");
    }

    #[cfg(unix)]
    #[test]
    fn transcript_file_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None);
        store.append_message(
            &meta.id,
            &ChatMessage { role: "user".to_string(), text: "hi".to_string(), ts: "t".to_string() },
        );

        let transcript_path = dir.path().join("chats").join(&meta.id).join("transcript.jsonl");
        let mode = std::fs::metadata(&transcript_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "transcript.jsonl must be 0600");
    }
}
