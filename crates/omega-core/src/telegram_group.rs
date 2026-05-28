//! OmegaOS Telegram supergroup + per-project topics.
//!
//! When a supergroup is set up via `/setupgroup <id>` from the bot, the
//! config is stored here. The bridge then auto-creates a forum topic per
//! project on first dispatch, maps `project_name → message_thread_id`,
//! and routes that project's oracle reports (PDF + text) to its topic.
//!
//! Storage: `~/.omega/telegram-group.toml` — simple, human-readable,
//! re-setupable (running /setupgroup again overwrites it).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramGroupConfig {
    /// Supergroup chat_id (negative integer, e.g. -1001234567890).
    pub group_id: i64,
    /// Human-friendly group name (cached from getChat for the bridge's
    /// status display — not required for routing).
    #[serde(default)]
    pub group_name: String,
    /// project_name → message_thread_id (Telegram forum topic id).
    #[serde(default)]
    pub topics: BTreeMap<String, i64>,
    /// ISO timestamp of last successful setup.
    #[serde(default)]
    pub setup_at: String,
}

impl TelegramGroupConfig {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/home/hacker"))
            .join(".omega/telegram-group.toml")
    }

    /// Load the saved group config. Returns None if no group has ever
    /// been set up (clean install, fresh user).
    pub fn load() -> Option<Self> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Persist to ~/.omega/telegram-group.toml.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn topic_for(&self, project: &str) -> Option<i64> {
        self.topics.get(project).copied()
    }

    pub fn set_topic(&mut self, project: &str, topic_id: i64) {
        self.topics.insert(project.to_string(), topic_id);
    }

    /// Reverse lookup: which project owns this forum topic? Lets the bridge
    /// route a message typed in a topic straight to that project's oracle.
    pub fn project_for_topic(&self, thread_id: i64) -> Option<String> {
        self.topics
            .iter()
            .find(|(_, &t)| t == thread_id)
            .map(|(p, _)| p.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut cfg = TelegramGroupConfig {
            group_id: -1001234567890,
            group_name: "OmegaOS Projects".to_string(),
            topics: BTreeMap::new(),
            setup_at: "2026-05-28T18:00:00Z".to_string(),
        };
        cfg.set_topic("DentistryGPT", 42);
        cfg.set_topic("Causio", 47);
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let back: TelegramGroupConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.group_id, -1001234567890);
        assert_eq!(back.topic_for("Causio"), Some(47));
        assert_eq!(back.topic_for("Missing"), None);
    }

    #[test]
    fn reverse_topic_lookup() {
        let mut cfg = TelegramGroupConfig::default();
        cfg.set_topic("DentistryGPT", 6);
        cfg.set_topic("OmegaOS", 7);
        // a message in thread 7 → routes to OmegaOS' oracle
        assert_eq!(cfg.project_for_topic(7).as_deref(), Some("OmegaOS"));
        assert_eq!(cfg.project_for_topic(6).as_deref(), Some("DentistryGPT"));
        // General topic / unmapped thread → None → falls through to Master
        assert_eq!(cfg.project_for_topic(999), None);
    }
}
