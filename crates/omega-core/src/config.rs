use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaConfig {
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub locks_dir: PathBuf,
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
                Some("clients") => ProjectCategory::Client,
                Some("1-life") => ProjectCategory::Personal,
                _ => ProjectCategory::Work,
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
