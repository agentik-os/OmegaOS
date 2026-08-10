use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub bind: String,
    pub stream_interval_ms: u64,
    pub stream_lines: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self { bind: "127.0.0.1:4477".into(), stream_interval_ms: 1000, stream_lines: 200 }
    }
}

impl GatewayConfig {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("gateway.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!("invalid {}: {e}; using defaults", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}

pub fn gateway_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_GATEWAY_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir().expect("no home dir").join(".omega").join("gateway")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = GatewayConfig::load(dir.path());
        assert_eq!(cfg.bind, "127.0.0.1:4477");
        assert_eq!(cfg.stream_interval_ms, 1000);
        assert_eq!(cfg.stream_lines, 200);
    }

    #[test]
    fn file_overrides_partial_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("gateway.toml"), "bind = \"127.0.0.1:9999\"\n").unwrap();
        let cfg = GatewayConfig::load(dir.path());
        assert_eq!(cfg.bind, "127.0.0.1:9999");
        assert_eq!(cfg.stream_lines, 200);
    }

    #[test]
    fn env_overrides_gateway_dir() {
        std::env::set_var("OMEGA_GATEWAY_DIR", "/tmp/omega-gw-test");
        assert_eq!(gateway_dir(), PathBuf::from("/tmp/omega-gw-test"));
        std::env::remove_var("OMEGA_GATEWAY_DIR");
    }
}
