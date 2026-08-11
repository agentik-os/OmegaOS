//! Provider config: `GET /v1/config` (redacted read) and `PUT /v1/config`
//! (validated single key/value write) — wave7 task C.
//!
//! `omega_core::providers::ProvidersConfig::{load,save}` are the exact
//! in-process typed store `omega config`/`cmd_config`
//! (`crates/omega-cli/src/main.rs`) already use — no CLI subprocess for
//! this task; `ProvidersConfig::path()` resolves via
//! `omega_core::config::omega_dir()`, which honors `$OMEGA_DIR`, so a
//! hermetic test overrides that env var (same pattern
//! `tests/oracle_ops_test.rs` already uses for `timeline`/`gate`).
//!
//! SECURITY DECISION (documented in `.superpowers/sdd/progress.md`'s Task C
//! ground-truth section, load-bearing): `omega config show` LEAKS EVERY
//! PROVIDER API KEY IN PLAINTEXT — confirmed live. `GET /v1/config` NEVER
//! does that: every provider's `api_key` is redacted to `api_key_set: bool`
//! (non-empty vs empty), never the secret itself, even though the caller is
//! already device-authenticated. Rationale: a stolen/leaked device token
//! would otherwise hand over EVERY provider credential on the box in one
//! GET — a blast radius wildly disproportionate to what an app UI needs (it
//! needs to know a key IS configured, never the value). `PUT /v1/config`
//! still accepts a write to `*.api_key` (write-only — a client can SET a
//! key it already possesses, it can just never READ one back over this
//! API), validated against the SAME `(provider, field)` allowlist
//! [`apply_config_value`] carries — a byte-for-byte hand-kept twin of
//! `omega-cli::set_config_value`'s match arms (omega-cli is a BINARY crate,
//! not a library this crate can import; the same "twin" shape
//! `routes_agents.rs`/`routes_audit.rs` already carry for their own
//! CLI-mirroring logic).

use crate::protocol::{
    ClaudeConfigEntry, CodexConfigEntry, ConfigResponse, ConfigSetRequest, GeminiConfigEntry,
    GlmConfigEntry, HermesConfigEntry, OpenRouterConfigEntry, PiConfigEntry,
};
use axum::http::StatusCode;
use axum::Json;
use omega_core::providers::ProvidersConfig;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg.into() })))
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": msg.to_string() })))
}

/// Cap on `PUT /v1/config`'s `value` — generous enough for any real model
/// id / effort level / base URL / API key a provider issues, small enough
/// that a runaway payload can't bloat `providers.toml` unbounded (same
/// defensive posture `routes_session_org::MAX_LABEL_LEN` documents).
const MAX_CONFIG_VALUE_LEN: usize = 4096;

fn to_response(cfg: &ProvidersConfig) -> ConfigResponse {
    ConfigResponse {
        claude: ClaudeConfigEntry {
            model: cfg.claude.model.clone(),
            effort: cfg.claude.effort.clone(),
            api_key_set: !cfg.claude.api_key.is_empty(),
            dangerously_skip_permissions: cfg.claude.dangerously_skip_permissions,
        },
        codex: CodexConfigEntry {
            model: cfg.codex.model.clone(),
            api_key_set: !cfg.codex.api_key.is_empty(),
            base_url: cfg.codex.base_url.clone(),
        },
        gemini: GeminiConfigEntry {
            model: cfg.gemini.model.clone(),
            api_key_set: !cfg.gemini.api_key.is_empty(),
        },
        glm: GlmConfigEntry {
            model: cfg.glm.model.clone(),
            api_key_set: !cfg.glm.api_key.is_empty(),
        },
        openrouter: OpenRouterConfigEntry {
            model: cfg.openrouter.model.clone(),
            api_key_set: !cfg.openrouter.api_key.is_empty(),
            base_url: cfg.openrouter.base_url.clone(),
        },
        pi: PiConfigEntry {
            provider: cfg.pi.provider.clone(),
            model: cfg.pi.model.clone(),
            api_key_set: !cfg.pi.api_key.is_empty(),
        },
        hermes: HermesConfigEntry {
            model: cfg.hermes.model.clone(),
            api_key_set: !cfg.hermes.api_key.is_empty(),
        },
    }
}

/// `GET /v1/config` — see this module's doc comment.
pub async fn get() -> Json<ConfigResponse> {
    let cfg = tokio::task::spawn_blocking(ProvidersConfig::load).await.unwrap_or_default();
    Json(to_response(&cfg))
}

/// The exact `(provider, field)` allowlist `omega-cli::set_config_value`
/// (`crates/omega-cli/src/main.rs` ~line 4473) uses — kept byte-for-byte in
/// sync by hand (see this module's doc comment for why it can't be
/// imported). An unrecognized `(provider, field)` pair is `Err`, never a
/// silent no-op.
///
/// DELIBERATE STRICTNESS BEYOND THE CLI TWIN: the CLI's own
/// `dangerously_skip_permissions` arm is `value.parse().unwrap_or(false)`
/// (a malformed value silently becomes `false`); this gateway endpoint
/// rejects a malformed boolean with a 400 instead. A network API caller
/// gets no visual confirmation of what was actually persisted the way a CLI
/// operator watching the terminal does, so a silent "your typo became
/// false" is a worse failure mode here than a loud 400 — a judgment call,
/// not a functional deviation for any value that already parses.
fn apply_config_value(cfg: &mut ProvidersConfig, key: &str, value: &str) -> Result<(), String> {
    let mut parts = key.splitn(2, '.');
    let provider = parts.next().filter(|s| !s.is_empty()).ok_or("missing provider")?;
    let field = parts.next().filter(|s| !s.is_empty()).ok_or("missing field (use provider.field)")?;
    match (provider, field) {
        ("claude", "model") => cfg.claude.model = value.to_string(),
        ("claude", "effort") => cfg.claude.effort = value.to_string(),
        ("claude", "api_key") => cfg.claude.api_key = value.to_string(),
        ("claude", "dangerously_skip_permissions") => {
            cfg.claude.dangerously_skip_permissions = value
                .parse()
                .map_err(|_| "dangerously_skip_permissions must be 'true' or 'false'".to_string())?;
        }
        ("codex", "model") => cfg.codex.model = value.to_string(),
        ("codex", "api_key") => cfg.codex.api_key = value.to_string(),
        ("codex", "base_url") => cfg.codex.base_url = value.to_string(),
        ("gemini", "model") => cfg.gemini.model = value.to_string(),
        ("gemini", "api_key") => cfg.gemini.api_key = value.to_string(),
        ("pi", "provider") => cfg.pi.provider = value.to_string(),
        ("pi", "model") => cfg.pi.model = value.to_string(),
        ("pi", "api_key") => cfg.pi.api_key = value.to_string(),
        ("glm", "model") => cfg.glm.model = value.to_string(),
        ("glm", "api_key") => cfg.glm.api_key = value.to_string(),
        ("hermes", "model") => cfg.hermes.model = value.to_string(),
        ("hermes", "api_key") => cfg.hermes.api_key = value.to_string(),
        _ => return Err(format!("unknown key: {key}")),
    }
    Ok(())
}

/// `PUT /v1/config` — validates `{key, value}` against
/// [`apply_config_value`]'s allowlist, applies it to the shared
/// `providers.toml` (which affects every session's future spawn, not just
/// the caller's), and returns the fresh redacted [`ConfigResponse`] so the
/// caller can confirm the write without ever reading a secret back.
pub async fn set(Json(req): Json<ConfigSetRequest>) -> Result<Json<ConfigResponse>, ApiError> {
    if req.key.trim().is_empty() {
        return Err(bad_request("key is required"));
    }
    if req.key.contains('\0') || req.value.contains('\0') {
        return Err(bad_request("key/value must not contain a NUL byte"));
    }
    if req.value.len() > MAX_CONFIG_VALUE_LEN {
        return Err(bad_request(format!("value too long (max {MAX_CONFIG_VALUE_LEN} bytes)")));
    }

    let key = req.key.clone();
    let value = req.value.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<ProvidersConfig, String> {
        let mut cfg = ProvidersConfig::load();
        apply_config_value(&mut cfg, &key, &value)?;
        cfg.save().map_err(|e| e.to_string())?;
        Ok(cfg)
    })
    .await
    .map_err(|e| internal(format!("config task panicked: {e}")))?;

    match result {
        Ok(cfg) => Ok(Json(to_response(&cfg))),
        Err(msg) => Err(bad_request(msg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_config_value_rejects_unknown_key() {
        let mut cfg = ProvidersConfig::default();
        let err = apply_config_value(&mut cfg, "claude.nonexistent", "x").unwrap_err();
        assert!(err.contains("unknown key"));
    }

    #[test]
    fn apply_config_value_rejects_missing_field() {
        let mut cfg = ProvidersConfig::default();
        let err = apply_config_value(&mut cfg, "claude", "x").unwrap_err();
        assert!(err.contains("missing field"));
    }

    #[test]
    fn apply_config_value_rejects_malformed_bool() {
        let mut cfg = ProvidersConfig::default();
        let err =
            apply_config_value(&mut cfg, "claude.dangerously_skip_permissions", "yes").unwrap_err();
        assert!(err.contains("true"));
    }

    #[test]
    fn apply_config_value_sets_known_key() {
        let mut cfg = ProvidersConfig::default();
        apply_config_value(&mut cfg, "claude.model", "opus").unwrap();
        assert_eq!(cfg.claude.model, "opus");
    }

    #[test]
    fn to_response_never_carries_api_key_bytes() {
        let mut cfg = ProvidersConfig::default();
        cfg.claude.api_key = "sk-super-secret".to_string();
        let resp = to_response(&cfg);
        assert!(resp.claude.api_key_set);
        // The struct itself has no field capable of carrying the secret —
        // this assertion is here as a documented tripwire: if a future edit
        // ever adds a raw `api_key` field to `ClaudeConfigEntry`, this test
        // still passes (it doesn't reference such a field), but the crate's
        // schema_test + adversarial review are the real backstop for that.
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("sk-super-secret"));
    }
}
