//! Provider configuration — typed, persistent, shared across all sessions.
//!
//! Saved to `~/.omega/providers.toml`. When OmegaOS spawns an agent session
//! (claude/codex/gemini/glm), the relevant env vars from this file are
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
    pub glm: GlmConfig,
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
    #[serde(default)]
    pub pi: PiConfig,
    #[serde(default)]
    pub hermes: HermesConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiConfig {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    /// Pi routes through OpenRouter; this key is injected as OPENROUTER_API_KEY
    /// into the Pi pane (see agents.rs `provider_env_prefix`).
    #[serde(default)]
    pub api_key: String,
    // NOTE: a `pi.extension` field was removed (2026-06) — it was an orphan with
    // get/set arms but NO consumer anywhere in the launch path, and the Pi CLI
    // (pi.dev) exposes no documented `--extension` flag to wire it to. Per L2
    // (no fabricated confidence) we removed the dead field rather than invent a
    // consumer. `#[serde(default)]` on the struct means old providers.toml files
    // carrying an `extension = "..."` line still deserialize (the key is ignored).
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HermesConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// "fable" | "opus" | "sonnet" | "haiku" | full model id like "claude-fable-5"
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
pub struct GlmConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

impl ProvidersConfig {
    fn path() -> PathBuf {
        crate::config::omega_dir().join("providers.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        // providers.toml holds per-provider api_key secrets. save() writes it
        // 0o600, but perms can drift (backup restore, manual copy, errant chmod).
        // Before trusting the secrets, verify the file is owner-only; if it leaked
        // group/world access, warn (observability) and best-effort re-tighten via
        // the shared chmod_600 impl. Best-effort (not fatal): a recoverable perms
        // drift shouldn't brick a running system — mirrors CredentialStore::new().
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let mode = meta.permissions().mode();
                if mode & 0o077 != 0 {
                    tracing::warn!(
                        "providers.toml ({}) has lax permissions ({:o}); contains \
                         api_key secrets — re-tightening to 0600",
                        path.display(),
                        mode & 0o777
                    );
                    let _ = crate::credentials::chmod_600(&path);
                }
            }
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
        // providers.toml holds per-provider api_key secrets — lock it to the
        // owner (was written with the default umask, i.e. world/group-readable
        // on a multi-user host). Shares credentials::chmod_600 (one impl).
        crate::credentials::chmod_600(&path)
            .with_context(|| format!("chmod 600 {}", path.display()))?;
        Ok(())
    }

    /// The stored API key for a provider (empty string if none / unknown).
    pub fn api_key_for(&self, provider: &str) -> &str {
        match provider {
            "claude" => &self.claude.api_key,
            "codex" => &self.codex.api_key,
            "gemini" => &self.gemini.api_key,
            "glm" => &self.glm.api_key,
            "openrouter" => &self.openrouter.api_key,
            _ => "",
        }
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
        if !self.openrouter.api_key.is_empty() {
            out.push((
                "OPENROUTER_API_KEY".to_string(),
                self.openrouter.api_key.clone(),
            ));
        }
        if !self.openrouter.base_url.is_empty() {
            out.push((
                "OPENROUTER_BASE_URL".to_string(),
                self.openrouter.base_url.clone(),
            ));
        }
        out
    }

    // ───────── Provider catalog (static metadata) ─────────

    /// All known providers in canonical order. Static slice — safe to expose.
    pub fn all_providers() -> Vec<&'static str> {
        vec!["claude", "codex", "gemini", "glm", "openrouter"]
    }

    pub fn default_provider() -> &'static str {
        "claude"
    }

    /// Default model id for a provider (used when /model <provider> has no
    /// model arg).
    pub fn default_model(provider: &str) -> &'static str {
        match provider {
            "claude" => "opus",
            "codex" => "gpt-5.5-codex",
            "gemini" => "gemini-3.1-pro",
            "glm" => "glm-5.1",
            // Operator directive 2026-07-24: Claude Opus 5 is THE default brain
            // everywhere a tier has not been deliberately pinned (R-MODEL).
            "openrouter" | "pi" | "hermes" => "anthropic/claude-opus-5",
            _ => "",
        }
    }

    /// Available models for a provider (used to list options in /model UI).
    pub fn models_for(provider: &str) -> Vec<&'static str> {
        match provider {
            // "opus" is first = the default, and resolves to claude-opus-5[1m]
            // at dispatch time (dispatch::resolve_model_flag). The explicit
            // ids sit beside the aliases so a session can be pinned exactly.
            "claude" => vec![
                "opus",
                "claude-opus-5",
                "sonnet",
                "claude-sonnet-5",
                "haiku",
                "fable",
            ],
            // June 2026: gpt-5.5-codex = Codex default; gpt-5.2-codex stays the
            // API-key-only fallback (5.5 needs ChatGPT sign-in).
            "codex" => vec!["gpt-5.5-codex", "gpt-5.5", "gpt-5.2-codex"],
            // 3.1+ line uses bare ids (no -preview); 2.5-pro kept as fallback.
            "gemini" => vec!["gemini-3.1-pro", "gemini-3.1-flash", "gemini-3.5-flash", "gemini-2.5-pro"],
            "glm" => vec!["glm-5.1", "glm-5", "glm-4.6"],
            // Pi and Hermes both route through OpenRouter, so they share the
            // same curated OpenRouter model IDs — this gives them an arrow-key
            // picker (no typing) instead of the free-text fallback.
            "pi" | "hermes" | "openrouter" => vec![
                "anthropic/claude-opus-5",
                "anthropic/claude-sonnet-5",
                "anthropic/claude-sonnet-4.6",
                "anthropic/claude-opus-4.8",
                "openai/gpt-5.5",
                "google/gemini-3.1-pro-preview",
                "z-ai/glm-5.1",
                "deepseek/deepseek-chat",
            ],
            _ => vec![],
        }
    }

    /// Auth type for a provider: "oauth" | "api_key" | "config".
    pub fn auth_type(provider: &str) -> &'static str {
        match provider {
            "claude" | "gemini" => "oauth",
            "codex" | "glm" | "openrouter" => "api_key",
            _ => "unknown",
        }
    }

    /// Relative cred file path under ~/.omega/.
    pub fn cred_file(provider: &str) -> String {
        format!("credentials/{}.json", provider)
    }

    /// True if the provider exists in the catalog.
    pub fn is_known(provider: &str) -> bool {
        Self::all_providers().iter().any(|p| *p == provider)
    }
}

/// Track the per-Telegram-chat active model selection.
/// Persisted to `~/.omega/state/telegram-active-model.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModel {
    #[serde(default = "default_active_provider")]
    pub active_provider: String,
    #[serde(default = "default_active_model")]
    pub active_model: String,
}

fn default_active_provider() -> String {
    ProvidersConfig::default_provider().to_string()
}

fn default_active_model() -> String {
    ProvidersConfig::default_model(ProvidersConfig::default_provider()).to_string()
}

impl Default for ActiveModel {
    fn default() -> Self {
        Self {
            active_provider: default_active_provider(),
            active_model: default_active_model(),
        }
    }
}

impl ActiveModel {
    fn path() -> PathBuf {
        crate::config::omega_dir()
            .join("state")
            .join("telegram-active-model.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Self>(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json).context("writing active-model state")?;
        Ok(())
    }

    /// Set the active provider+model. If `model` is empty, falls back to the
    /// provider's default model.
    pub fn set(provider: &str, model: Option<&str>) -> Result<Self> {
        if !ProvidersConfig::is_known(provider) {
            anyhow::bail!("unknown provider: {}", provider);
        }
        let resolved_model = match model {
            Some(m) if !m.is_empty() => m.to_string(),
            _ => ProvidersConfig::default_model(provider).to_string(),
        };
        let new = Self {
            active_provider: provider.to_string(),
            active_model: resolved_model,
        };
        new.save()?;
        Ok(new)
    }
}
