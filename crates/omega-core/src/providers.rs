//! Provider configuration — typed, persistent, shared across all sessions.
//!
//! Saved to `~/.omega/providers.toml`. When OmegaOS spawns an agent session
//! (claude/codex/gemini/pi/glm), the relevant env vars from this file are
//! injected so every session uses the same credentials and model defaults.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub claude: ClaudeConfig,
    #[serde(default)]
    pub codex: CodexConfig,
    #[serde(default)]
    pub gemini: GeminiConfig,
    #[serde(default)]
    pub pi: PiConfig,
    #[serde(default)]
    pub glm: GlmConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// "opus" | "sonnet" | "haiku" | full model id like "claude-opus-4-7"
    #[serde(default)]
    pub model: String,
    /// "low" | "medium" | "high" | "max" — Claude effort level
    #[serde(default)]
    pub effort: String,
    /// Anthropic API key (only used if not via OAuth)
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub dangerously_skip_permissions: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodexConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeminiConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiConfig {
    /// "openrouter" | "anthropic" | "openai"
    #[serde(default)]
    pub provider: String,
    /// e.g. "anthropic/claude-sonnet-4.6"
    #[serde(default)]
    pub model: String,
    /// Path to an extension file passed via -e
    #[serde(default)]
    pub extension: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlmConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

impl ProvidersConfig {
    fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("providers.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).context("serializing providers config")?;
        std::fs::write(&path, content).context("writing providers config")?;
        Ok(())
    }

    /// Build the env vars that should be injected into every agent session.
    pub fn env_vars(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if !self.claude.api_key.is_empty() {
            out.push(("ANTHROPIC_API_KEY".to_string(), self.claude.api_key.clone()));
        }
        if !self.codex.api_key.is_empty() {
            out.push(("OPENAI_API_KEY".to_string(), self.codex.api_key.clone()));
        }
        if !self.codex.base_url.is_empty() {
            out.push(("OPENAI_BASE_URL".to_string(), self.codex.base_url.clone()));
        }
        if !self.gemini.api_key.is_empty() {
            out.push(("GOOGLE_API_KEY".to_string(), self.gemini.api_key.clone()));
            out.push(("GEMINI_API_KEY".to_string(), self.gemini.api_key.clone()));
        }
        if !self.glm.api_key.is_empty() {
            out.push(("GLM_API_KEY".to_string(), self.glm.api_key.clone()));
        }
        out
    }
}
