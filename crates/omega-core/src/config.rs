use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
// Every field falls back to the manual `Default` impl below, so a partial or
// even empty config.toml deserializes cleanly instead of silently discarding
// the whole file (`OmegaConfig::load()` used to Err on any missing field).
#[serde(default)]
pub struct OmegaConfig {
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub locks_dir: PathBuf,
    /// Root under which project categories (work/clients/...) live. Auto-detected
    /// per-user (see `default_projects_dir`) — never a hardcoded ~/VibeCoding.
    /// Override in ~/.omega/config.toml.
    #[serde(default = "default_projects_dir")]
    pub projects_dir: PathBuf,
    pub projects: Vec<ProjectConfig>,
    pub agent_command: String,
    pub default_model: String,
    #[serde(default = "default_aisb_agent")]
    pub aisb_agent: String,
    #[serde(default = "default_auto_master")]
    pub auto_spawn_master: bool,
    #[serde(default = "default_auto_naming")]
    pub auto_naming: bool,
    pub telegram: Option<TelegramConfig>,
}

fn default_aisb_agent() -> String {
    "claude".to_string()
}

fn default_auto_master() -> bool {
    true
}

fn default_auto_naming() -> bool {
    true
}

/// Auto-detect the user's project root: the first existing common work
/// container under $HOME, else ~/projects. Cross-user — NO ~/VibeCoding
/// hardcode, so a fresh install adapts to whatever layout the user has.
fn default_projects_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    for cand in ["VibeCoding", "projects", "Projects", "code", "dev", "work"] {
        let p = home.join(cand);
        if p.is_dir() {
            return p;
        }
    }
    home.join("projects")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub category: ProjectCategory,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectCategory {
    Work,
    Client,
    Personal,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: i64,
}

impl Default for OmegaConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let omega_dir = home.join(".omega");
        Self {
            state_dir: omega_dir.join("state"),
            logs_dir: omega_dir.join("logs"),
            locks_dir: omega_dir.join("locks"),
            projects_dir: default_projects_dir(),
            projects: Vec::new(),
            agent_command: "claude".to_string(),
            default_model: "opus".to_string(),
            aisb_agent: default_aisb_agent(),
            auto_spawn_master: default_auto_master(),
            auto_naming: default_auto_naming(),
            telegram: None,
        }
    }
}

impl OmegaConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".omega")
            .join("config.toml")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.state_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        std::fs::create_dir_all(&self.locks_dir)?;
        Ok(())
    }

    /// Resolve a project category (customer/side-business/tools/personal/...)
    /// to an absolute directory under `projects_dir`. Cross-user — no hardcoded
    /// paths.
    pub fn resolve_category_path(&self, category: &str) -> PathBuf {
        // New installs use customer / side-business / tools. Older layouts used
        // clients / work / 1-life. Prefer an existing conventional dir so we
        // don't fragment a user's tree, then fall back to the new canonical name.
        let candidates: &[&str] = match category {
            "customer" | "customers" | "client" | "clients" => &["customers", "clients"],
            "side-business" | "side_business" | "sidebusiness" | "work" | "works" => {
                &["side-business", "work"]
            }
            "tool" | "tools" => &["tools"],
            "personal" | "life" | "1-life" => &["1-life", "personal"],
            other => return self.projects_dir.join(other),
        };
        candidates
            .iter()
            .map(|c| self.projects_dir.join(c))
            .find(|p| p.is_dir())
            .unwrap_or_else(|| self.projects_dir.join(candidates[0]))
    }

    pub fn find_project(&self, name: &str) -> Option<&ProjectConfig> {
        let lower = name.to_lowercase();
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
    }

    pub fn discover_projects(scan_dirs: &[&Path]) -> Vec<ProjectConfig> {
        let mut projects = Vec::new();
        for dir in scan_dirs {
            if !dir.exists() {
                continue;
            }
            let category = match dir.file_name().and_then(|n| n.to_str()) {
                Some("clients") | Some("customers") => ProjectCategory::Client,
                Some("1-life") | Some("personal") => ProjectCategory::Personal,
                _ => ProjectCategory::Work, // work, side-business, tools, ...
            };
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let is_project = path.join(".git").exists()
                        || path.join("package.json").exists()
                        || path.join("Cargo.toml").exists()
                        || path.join("pyproject.toml").exists()
                        || path.join("go.mod").exists();
                    if is_project {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            projects.push(ProjectConfig {
                                name: name.to_string(),
                                path,
                                category: category.clone(),
                                icon: None,
                            });
                        }
                    }
                }
            }
        }
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        projects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_category_path_is_under_projects_dir_no_vibecoding() {
        // Fresh layout (no category dir exists yet) → the new canonical names.
        let mut c = OmegaConfig::default();
        c.projects_dir = PathBuf::from("/home/someuser/projects");
        assert_eq!(
            c.resolve_category_path("customer"),
            PathBuf::from("/home/someuser/projects/customers")
        );
        assert_eq!(
            c.resolve_category_path("side-business"),
            PathBuf::from("/home/someuser/projects/side-business")
        );
        assert_eq!(
            c.resolve_category_path("tools"),
            PathBuf::from("/home/someuser/projects/tools")
        );
        assert_eq!(
            c.resolve_category_path("1-life"),
            PathBuf::from("/home/someuser/projects/1-life")
        );
        // Old aliases fall to the new canonical when no legacy dir exists.
        assert_eq!(
            c.resolve_category_path("client"),
            PathBuf::from("/home/someuser/projects/customers")
        );
        assert_eq!(
            c.resolve_category_path("work"),
            PathBuf::from("/home/someuser/projects/side-business")
        );
        // Unknown category → uses its own name as the subdir.
        assert_eq!(
            c.resolve_category_path("research"),
            PathBuf::from("/home/someuser/projects/research")
        );
        // Never a hardcoded ~/VibeCoding.
        assert!(c
            .resolve_category_path("customer")
            .starts_with("/home/someuser/projects"));
    }

    #[test]
    fn resolve_category_path_prefers_existing_legacy_dir() {
        // Backward-compat: a machine already using clients/ + work/ keeps them
        // instead of fragmenting into customers/ + side-business/.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("clients")).unwrap();
        std::fs::create_dir_all(tmp.path().join("work")).unwrap();
        let mut c = OmegaConfig::default();
        c.projects_dir = tmp.path().to_path_buf();
        assert_eq!(c.resolve_category_path("customer"), tmp.path().join("clients"));
        assert_eq!(c.resolve_category_path("side-business"), tmp.path().join("work"));
    }

    #[test]
    fn shipped_default_toml_deserializes() {
        // The shipped template MUST parse into OmegaConfig — it used to fail and
        // silently discard the whole config on a fresh install.
        let cfg: OmegaConfig = toml::from_str(include_str!("../../../config/default.toml"))
            .expect("config/default.toml must deserialize into OmegaConfig");
        assert_eq!(cfg.agent_command, "claude");
        assert_eq!(cfg.default_model, "opus");
    }

    #[test]
    fn partial_config_overrides_only_its_keys() {
        // A partial config overrides its keys and keeps defaults for the rest,
        // instead of nuking the whole file.
        let cfg: OmegaConfig = toml::from_str("default_model = \"sonnet\"\n").unwrap();
        assert_eq!(cfg.default_model, "sonnet"); // overridden
        assert_eq!(cfg.agent_command, "claude"); // still the default, not empty
    }
}
