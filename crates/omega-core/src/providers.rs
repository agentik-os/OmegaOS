//! Provider configuration — typed, persistent, shared across all sessions.
//!
//! Saved to `~/.omega/providers.toml`. When OmegaOS spawns an agent session
//! (claude/codex/gemini/glm), the relevant env vars from this file are
//! injected so every session uses the same credentials and model defaults.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
    #[serde(default)]
    pub kimi: KimiConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiConfig {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    /// Pi routes through OpenRouter; this key is injected as OPENROUTER_API_KEY
    /// into the Pi pane through the typed rmux process environment.
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
    /// Extra writable roots granted to Codex in addition to the project and
    /// OmegaOS's state/lock directories. Values must be absolute paths.
    #[serde(default)]
    pub additional_writable_dirs: Vec<String>,
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
    /// Explicit high-risk opt-in for the Claude-backed GLM adapter. The safe
    /// default is Claude Code's `auto` permission mode.
    #[serde(default)]
    pub dangerously_skip_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KimiConfig {
    /// Kimi model id or configured alias. OAuth users may leave this empty and
    /// let Kimi Code use its own `default_model`.
    pub model: String,
    /// Direct API credential. Kimi Code only accepts this from its explicit
    /// `KIMI_MODEL_*` override channel, not from a plain `KIMI_API_KEY` export.
    pub api_key: String,
    pub base_url: String,
    pub provider_type: String,
}

impl Default for KimiConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            api_key: String::new(),
            base_url: String::new(),
            provider_type: "kimi".to_string(),
        }
    }
}

impl ProvidersConfig {
    pub fn path() -> PathBuf {
        crate::config::omega_dir().join("providers.toml")
    }

    /// Compatibility loader for read-only UI/diagnostic surfaces. Runtime
    /// authority must use [`Self::try_load`]. Mutations remain fail-closed in
    /// [`Self::save`], even if a legacy caller obtained this fallback value.
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(error) => {
                eprintln!(
                    "omega: provider config unavailable; diagnostic defaults only, agent launch is blocked: {error:#}"
                );
                tracing::error!(error = %error, "provider config unavailable; diagnostic defaults only");
                Self::default()
            }
        }
    }

    /// Strict provider configuration loader for every launch/mutation path.
    pub fn try_load() -> Result<Self> {
        let path = Self::path();
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let Some(raw) = crate::config::read_private_optional_string(path)? else {
            return Ok(Self::default());
        };
        let config: Self = toml::from_str(&raw)
            .with_context(|| format!("parsing provider config {}", path.display()))?;
        config
            .validate()
            .with_context(|| format!("validating provider config {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        self.validate()
            .with_context(|| format!("validating provider config for {}", path.display()))?;
        if crate::config::read_private_optional(path)?.is_some() {
            let _ = Self::load_from(path).context(
                "refusing to overwrite an unreadable or invalid existing provider config",
            )?;
        }
        let content = toml::to_string_pretty(self).context("serializing providers config")?;
        crate::config::atomic_write_private(path, content.as_bytes())
            .context("writing providers config")
    }

    pub(crate) fn validate(&self) -> Result<()> {
        match self.kimi.provider_type.as_str() {
            "kimi" | "anthropic" | "openai" => {}
            other => anyhow::bail!(
                "invalid kimi.provider_type {other:?}; expected kimi, anthropic, or openai"
            ),
        }
        for path in &self.codex.additional_writable_dirs {
            if !Path::new(path).is_absolute() {
                anyhow::bail!("codex.additional_writable_dirs entry must be absolute: {path:?}");
            }
            if path.chars().any(char::is_control) {
                anyhow::bail!("codex.additional_writable_dirs entry contains control characters");
            }
        }
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
            "pi" => &self.pi.api_key,
            "hermes" => &self.hermes.api_key,
            "kimi" => &self.kimi.api_key,
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
        // Current Kimi Code deliberately ignores plain KIMI_API_KEY exports.
        // Its documented ephemeral-provider channel requires NAME + API_KEY;
        // only emit the complete pair so an incomplete override never bricks an
        // otherwise healthy OAuth session.
        if !self.kimi.api_key.is_empty() {
            let model = if self.kimi.model.is_empty() {
                Self::default_model("kimi")
            } else {
                &self.kimi.model
            };
            out.push(("KIMI_MODEL_NAME".to_string(), model.to_string()));
            out.push(("KIMI_MODEL_API_KEY".to_string(), self.kimi.api_key.clone()));
            out.push((
                "KIMI_MODEL_PROVIDER_TYPE".to_string(),
                self.kimi.provider_type.clone(),
            ));
            if !self.kimi.base_url.is_empty() {
                out.push((
                    "KIMI_MODEL_BASE_URL".to_string(),
                    self.kimi.base_url.clone(),
                ));
            }
        }
        out
    }

    // ───────── Provider catalog (static metadata) ─────────

    /// All known providers in canonical order. Static slice — safe to expose.
    pub fn all_providers() -> Vec<&'static str> {
        vec![
            "claude",
            "codex",
            "gemini",
            "glm",
            "openrouter",
            "pi",
            "hermes",
            "kimi",
            "shell",
        ]
    }

    pub fn default_provider() -> &'static str {
        "codex"
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
            "kimi" => "kimi-for-coding",
            "shell" => "",
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
            "gemini" => vec![
                "gemini-3.1-pro",
                "gemini-3.1-flash",
                "gemini-3.5-flash",
                "gemini-2.5-pro",
            ],
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
            "kimi" => vec![
                "kimi-for-coding",
                "kimi-for-coding-highspeed",
                "k3",
                "k3-256k",
                "kimi-k2.6",
            ],
            "shell" => vec![],
            _ => vec![],
        }
    }

    /// Auth type for a provider: "oauth" | "api_key" | "config".
    pub fn auth_type(provider: &str) -> &'static str {
        match provider {
            "claude" | "gemini" => "oauth",
            "codex" | "glm" | "openrouter" | "pi" | "hermes" => "api_key",
            "kimi" => "oauth_or_api_key",
            "shell" => "local",
            _ => "unknown",
        }
    }

    /// Relative cred file path under ~/.omega/.
    pub fn cred_file(provider: &str) -> String {
        format!("credentials/{}.json", provider)
    }

    /// True if the provider exists in the catalog.
    pub fn is_known(provider: &str) -> bool {
        Self::all_providers().contains(&provider)
    }

    /// Static execution capabilities used by the router before a session is
    /// spawned. This describes the OmegaOS adapter, not a marketing claim made
    /// by a model vendor.
    pub fn capabilities_for(provider: &str) -> Option<BTreeSet<ProviderCapability>> {
        let capabilities = match provider {
            "claude" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "codex" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
            ][..],
            "gemini" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "glm" | "openrouter" | "pi" | "hermes" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
            ][..],
            "kimi" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "shell" => &[
                ProviderCapability::LocalExecution,
                ProviderCapability::DeterministicCommands,
            ][..],
            _ => return None,
        };
        Some(capabilities.iter().copied().collect())
    }

    /// Negotiate required and optional capabilities before spawn.
    ///
    /// A named provider is never silently replaced: a mismatch is returned to
    /// the caller. Without a preference, candidates missing any required
    /// capability are excluded and optional capability coverage is used only
    /// as a deterministic tie-breaker.
    pub fn negotiate_provider(
        preferred: Option<&str>,
        required: &[ProviderCapability],
        optional: &[ProviderCapability],
    ) -> std::result::Result<ProviderSelection, ProviderNegotiationError> {
        if let Some(provider) = preferred {
            let capabilities = Self::capabilities_for(provider)
                .ok_or_else(|| ProviderNegotiationError::UnknownProvider(provider.to_string()))?;
            let missing_required = missing_capabilities(required, &capabilities);
            if !missing_required.is_empty() {
                return Err(ProviderNegotiationError::RequiredCapabilitiesMissing {
                    provider: provider.to_string(),
                    missing: missing_required,
                });
            }
            return Ok(ProviderSelection::new(
                provider,
                capabilities,
                optional,
                false,
            ));
        }

        let mut candidates: Vec<ProviderSelection> = Self::all_providers()
            .into_iter()
            .filter_map(|provider| {
                let capabilities = Self::capabilities_for(provider)?;
                missing_capabilities(required, &capabilities)
                    .is_empty()
                    .then(|| ProviderSelection::new(provider, capabilities, optional, true))
            })
            .collect();
        candidates.sort_by(|left, right| {
            right
                .optional_capabilities_satisfied
                .len()
                .cmp(&left.optional_capabilities_satisfied.len())
                .then_with(|| provider_rank(&left.provider).cmp(&provider_rank(&right.provider)))
        });
        candidates
            .into_iter()
            .next()
            .ok_or_else(|| ProviderNegotiationError::NoProviderSatisfies {
                required: required.to_vec(),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    Reasoning,
    CodeEditing,
    ToolCalling,
    Delegation,
    Vision,
    LongContext,
    LocalExecution,
    DeterministicCommands,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub provider: String,
    pub capabilities: BTreeSet<ProviderCapability>,
    pub optional_capabilities_satisfied: Vec<ProviderCapability>,
    /// True when OmegaOS selected a provider from capabilities rather than
    /// honoring an explicit caller preference.
    pub automatically_selected: bool,
}

impl ProviderSelection {
    fn new(
        provider: &str,
        capabilities: BTreeSet<ProviderCapability>,
        optional: &[ProviderCapability],
        automatically_selected: bool,
    ) -> Self {
        let optional_capabilities_satisfied = optional
            .iter()
            .copied()
            .filter(|capability| capabilities.contains(capability))
            .collect();
        Self {
            provider: provider.to_string(),
            capabilities,
            optional_capabilities_satisfied,
            automatically_selected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderNegotiationError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
    #[error("provider {provider} is missing required capabilities: {missing:?}")]
    RequiredCapabilitiesMissing {
        provider: String,
        missing: Vec<ProviderCapability>,
    },
    #[error("no provider satisfies required capabilities: {required:?}")]
    NoProviderSatisfies { required: Vec<ProviderCapability> },
}

fn missing_capabilities(
    required: &[ProviderCapability],
    available: &BTreeSet<ProviderCapability>,
) -> Vec<ProviderCapability> {
    required
        .iter()
        .copied()
        .filter(|capability| !available.contains(capability))
        .collect()
}

fn provider_rank(provider: &str) -> usize {
    ProvidersConfig::all_providers()
        .iter()
        .position(|candidate| *candidate == provider)
        .unwrap_or(usize::MAX)
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
        match Self::try_load() {
            Ok(model) => model,
            Err(error) => {
                eprintln!(
                    "omega: active-model state unavailable; diagnostic default only: {error:#}"
                );
                tracing::error!(error = %error, "active-model state unavailable");
                Self::default()
            }
        }
    }

    pub fn try_load() -> Result<Self> {
        let path = Self::path();
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let Some(raw) = crate::config::read_private_optional_string(path)? else {
            return Ok(Self::default());
        };
        let model: Self = serde_json::from_str(&raw)
            .with_context(|| format!("parsing active-model state {}", path.display()))?;
        model
            .validate()
            .with_context(|| format!("validating active-model state {}", path.display()))?;
        Ok(model)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        self.validate()
            .with_context(|| format!("validating active-model state for {}", path.display()))?;
        if crate::config::read_private_optional(path)?.is_some() {
            let _ = Self::load_from(path)
                .context("refusing to overwrite unreadable or invalid active-model state")?;
        }
        let json = serde_json::to_string_pretty(self)?;
        crate::config::atomic_write_private(path, json.as_bytes())
            .context("writing active-model state")
    }

    fn validate(&self) -> Result<()> {
        if !ProvidersConfig::is_known(&self.active_provider) {
            anyhow::bail!("unknown active provider: {}", self.active_provider);
        }
        if self.active_provider != "shell" && self.active_model.trim().is_empty() {
            anyhow::bail!(
                "active model is empty for provider {}",
                self.active_provider
            );
        }
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

#[cfg(test)]
mod provider_capability_tests {
    use super::*;

    #[test]
    fn catalog_covers_every_agent_adapter() {
        for provider in [
            "claude",
            "codex",
            "gemini",
            "glm",
            "openrouter",
            "pi",
            "hermes",
            "kimi",
            "shell",
        ] {
            assert!(ProvidersConfig::is_known(provider), "{provider}");
            assert!(
                ProvidersConfig::capabilities_for(provider).is_some(),
                "{provider}"
            );
        }
    }

    #[test]
    fn kimi_is_first_class_and_uses_the_supported_override_channel() {
        assert!(ProvidersConfig::is_known("kimi"));
        assert_eq!(ProvidersConfig::default_model("kimi"), "kimi-for-coding");
        assert!(ProvidersConfig::models_for("kimi").contains(&"k3"));
        assert_eq!(ProvidersConfig::auth_type("kimi"), "oauth_or_api_key");

        let config = ProvidersConfig {
            kimi: KimiConfig {
                api_key: "secret".to_string(),
                base_url: "https://api.moonshot.ai/v1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let env: std::collections::BTreeMap<_, _> = config.env_vars().into_iter().collect();
        assert_eq!(
            env.get("KIMI_MODEL_NAME").map(String::as_str),
            Some("kimi-for-coding")
        );
        assert_eq!(
            env.get("KIMI_MODEL_API_KEY").map(String::as_str),
            Some("secret")
        );
        assert_eq!(
            env.get("KIMI_MODEL_PROVIDER_TYPE").map(String::as_str),
            Some("kimi")
        );
        assert!(!env.contains_key("KIMI_API_KEY"));
        assert!(!env.contains_key("MOONSHOT_API_KEY"));
    }

    #[test]
    fn malformed_provider_config_fails_and_cannot_be_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        let corrupt = b"[codex\napi_key = 'secret'\n";
        std::fs::write(&path, corrupt).unwrap();
        assert!(ProvidersConfig::load_from(&path).is_err());

        let result = ProvidersConfig::default().save_to(&path);
        assert!(
            result.is_err(),
            "mutation must not replace corrupt provider config"
        );
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[cfg(unix)]
    #[test]
    fn provider_config_rejects_symlinks_dangling_symlinks_and_hardlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.toml");
        let authority = tmp.path().join("providers.toml");
        std::fs::write(&victim, "[codex]\nmodel = \"gpt-5.5-codex\"\n").unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &authority).unwrap();
        assert!(ProvidersConfig::load_from(&authority).is_err());
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode
        );

        std::fs::remove_file(&authority).unwrap();
        symlink(tmp.path().join("missing.toml"), &authority).unwrap();
        assert!(ProvidersConfig::load_from(&authority).is_err());

        std::fs::remove_file(&authority).unwrap();
        std::fs::hard_link(&victim, &authority).unwrap();
        let error = ProvidersConfig::load_from(&authority).unwrap_err();
        assert!(error.to_string().contains("hard links"), "{error:#}");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected authority hardlink must not chmod the shared inode"
        );
    }

    #[test]
    fn provider_save_is_atomic_private_and_validates_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        let mut config = ProvidersConfig {
            codex: CodexConfig {
                api_key: "secret".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        config.save_to(&path).unwrap();
        let loaded = ProvidersConfig::load_from(&path).unwrap();
        assert_eq!(loaded.codex.api_key, "secret");
        assert_eq!(
            std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().contains("omega-tmp"))
                .count(),
            0
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        config.codex.additional_writable_dirs = vec!["relative/path".to_string()];
        assert!(config.save_to(&path).is_err());
    }

    #[test]
    fn invalid_kimi_provider_type_is_rejected() {
        let config = ProvidersConfig {
            kimi: KimiConfig {
                provider_type: "kimi; touch /tmp/nope".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn malformed_active_model_is_not_defaulted_or_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("active-model.json");
        let corrupt = b"{not-json";
        std::fs::write(&path, corrupt).unwrap();
        assert!(ActiveModel::load_from(&path).is_err());
        assert!(ActiveModel::default().save_to(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[cfg(unix)]
    #[test]
    fn active_model_rejects_symlinks_dangling_symlinks_and_hardlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.json");
        let authority = tmp.path().join("active-model.json");
        std::fs::write(
            &victim,
            r#"{"active_provider":"codex","active_model":"gpt-5.5-codex"}"#,
        )
        .unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &authority).unwrap();
        assert!(ActiveModel::load_from(&authority).is_err());
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode
        );

        std::fs::remove_file(&authority).unwrap();
        symlink(tmp.path().join("missing.json"), &authority).unwrap();
        assert!(ActiveModel::load_from(&authority).is_err());

        std::fs::remove_file(&authority).unwrap();
        std::fs::hard_link(&victim, &authority).unwrap();
        let error = ActiveModel::load_from(&authority).unwrap_err();
        assert!(error.to_string().contains("hard links"), "{error:#}");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected authority hardlink must not chmod the shared inode"
        );
    }

    #[test]
    fn active_model_rejects_unknown_provider_and_writes_privately() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("active-model.json");
        let unknown = ActiveModel {
            active_provider: "codxe".to_string(),
            active_model: "gpt".to_string(),
        };
        assert!(unknown.save_to(&path).is_err());

        ActiveModel::default().save_to(&path).unwrap();
        assert_eq!(
            ActiveModel::load_from(&path).unwrap().active_provider,
            "codex"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn required_capability_mismatch_fails_before_spawn() {
        let result = ProvidersConfig::negotiate_provider(
            Some("shell"),
            &[ProviderCapability::CodeEditing],
            &[],
        );
        assert!(matches!(
            result,
            Err(ProviderNegotiationError::RequiredCapabilitiesMissing {
                provider,
                missing,
            }) if provider == "shell" && missing == vec![ProviderCapability::CodeEditing]
        ));
    }

    #[test]
    fn optional_capabilities_rank_without_becoming_requirements() {
        let selection = ProvidersConfig::negotiate_provider(
            None,
            &[ProviderCapability::Reasoning],
            &[
                ProviderCapability::Delegation,
                ProviderCapability::LongContext,
            ],
        )
        .unwrap();
        assert_eq!(selection.provider, "claude");
        assert_eq!(selection.optional_capabilities_satisfied.len(), 2);
        assert!(selection.automatically_selected);
    }

    #[test]
    fn impossible_required_set_returns_no_provider() {
        let result = ProvidersConfig::negotiate_provider(
            None,
            &[
                ProviderCapability::CodeEditing,
                ProviderCapability::DeterministicCommands,
            ],
            &[],
        );
        assert!(matches!(
            result,
            Err(ProviderNegotiationError::NoProviderSatisfies { .. })
        ));
    }
}
