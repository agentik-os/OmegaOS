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
    /// Single-key rmux session shortcuts in the Sessions tab (x kill, r rename,
    /// b background, . cleanup, …). Default ON. When OFF, the Sessions tab takes
    /// only arrow navigation + Enter/Esc/Tab — so no action fires by accident.
    /// (Arrow-only navigation is always enforced there; j/k never move the cursor.)
    #[serde(default = "default_session_shortcuts")]
    pub session_shortcuts: bool,
    /// TUI color theme slug (Settings → Theme). The registry of valid slugs
    /// lives in omega-tui's theme module; an unknown slug falls back to the
    /// default "omega" palette at load time.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Paint the active theme's background color across the whole TUI frame
    /// (Settings → Theme). Default ON. When OFF the terminal's own background
    /// shows through (transparency, background images, user terminal colors),
    /// while accent/text colors still follow the theme.
    #[serde(default = "default_theme_background")]
    pub theme_background: bool,
    /// IANA timezone for the on-screen clock (e.g. "Europe/Paris", "America/New_York").
    /// Persisted timestamps stay UTC — this localizes ONLY the displayed clock,
    /// because a headless VPS runs in UTC and the operator usually does not.
    /// `None`/empty → fall back to `$TZ`, then the system local zone.
    #[serde(default)]
    pub timezone: Option<String>,
    pub telegram: Option<TelegramConfig>,
}

fn default_aisb_agent() -> String {
    "claude".to_string()
}

fn default_auto_master() -> bool {
    // Retired: the brain is the Atlas Telegram bot (omega-tg-bot.ts), not the legacy
    // `aisb-master` rmux session (a Telegram-conversation viewer). Don't auto-spawn it.
    false
}

fn default_auto_naming() -> bool {
    true
}

fn default_session_shortcuts() -> bool {
    true
}

fn default_theme() -> String {
    "omega".to_string()
}

fn default_theme_background() -> bool {
    true
}

/// Auto-detect the user's project root: the first existing common work
/// container under $HOME, else ~/projects. Cross-user — NO ~/VibeCoding
/// hardcode, so a fresh install adapts to whatever layout the user has.
fn default_projects_dir() -> PathBuf {
    // `home_dir()` already consults $HOME on Unix; if it still fails the home is
    // genuinely unresolvable. Don't return the bare world-writable /tmp as a
    // PERSISTENT projects root (collision + data-loss-on-reboot risk) — scope it
    // to a per-user subdir under the temp dir as a last resort, and warn loudly
    // so the operator knows their projects are not landing under $HOME.
    let home = dirs::home_dir().unwrap_or_else(|| {
        let user = std::env::var("USER").unwrap_or_else(|_| "omega".to_string());
        let fallback = std::env::temp_dir().join(format!("omega-projects-{user}"));
        eprintln!(
            "omega: could not resolve a home directory ($HOME unset) — falling back \
             to a temp projects root at {}. This is EPHEMERAL; set $HOME or \
             projects_dir in config.toml for persistent storage.",
            fallback.display()
        );
        fallback
    });
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

/// The OmegaOS system root — the single source of truth for where all state,
/// config, credentials, agents, skills, and logs live.
///
/// Resolution order (first hit wins):
///   1. `$OMEGA_DIR`            — explicit override (install.sh already honors it;
///                                this makes the binary agree, fixing a long-standing
///                                install.sh-vs-binary split).
///   2. `$HOME/OmegaOS/System`  — the consolidated 3-folder layout, IF it exists
///                                (a fresh install or the migration creates it).
///   3. `$HOME/.omega`          — the legacy dotfolder, for machines not migrated.
///
/// Every other path in the codebase derives from this, so relocating the whole
/// system is one env var or one migration, not an N-site rewrite.
pub fn omega_dir() -> PathBuf {
    if let Ok(d) = std::env::var("OMEGA_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let consolidated = home.join("OmegaOS").join("System");
    if consolidated.is_dir() {
        return consolidated;
    }
    home.join(".omega")
}

impl Default for OmegaConfig {
    fn default() -> Self {
        let omega_dir = omega_dir();
        Self {
            state_dir: omega_dir.join("state"),
            logs_dir: omega_dir.join("logs"),
            locks_dir: omega_dir.join("locks"),
            projects_dir: default_projects_dir(),
            projects: Vec::new(),
            // Operator directive: Codex/Sol is the default coding agent everywhere
            // (oracles, workers, orchestrate, team, patrol re-spawn, and the plain
            // Telegram dispatch all resolve THIS field when no --agent is given).
            // Claude stays available (per-mission `--agent claude`, the /duo binome,
            // the agent picker) and remains the safe fallback when an agent name is
            // misconfigured or Codex is absent — see the parse-fallbacks below.
            agent_command: "codex".to_string(),
            default_model: "opus".to_string(),
            aisb_agent: default_aisb_agent(),
            auto_spawn_master: default_auto_master(),
            auto_naming: default_auto_naming(),
            session_shortcuts: default_session_shortcuts(),
            theme: default_theme(),
            theme_background: default_theme_background(),
            timezone: None,
            telegram: None,
        }
    }
}

impl OmegaConfig {
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path())
    }

    /// Load from an explicit path (testable without touching $OMEGA_DIR).
    ///
    /// A malformed existing config.toml is NOT discarded silently: almost every
    /// caller does `load().unwrap_or_default()`, so a parse error would quietly
    /// revert to defaults — dropping the telegram token, project list, model,
    /// timezone, etc. with no signal at all. On a parse failure we preserve the
    /// offending file (single `.corrupt` copy so the operator can recover their
    /// values) and log a loud error, then still return Err so the few callers
    /// that propagate it fail loudly too.
    fn load_from(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(config_path)?;
        match toml::from_str(&content) {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                let backup = config_path.with_file_name(format!(
                    "{}.corrupt",
                    config_path
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                ));
                let _ = std::fs::copy(config_path, &backup);
                // tracing is often not wired up in the TUI menu or the background
                // daemon, so a parse failure would otherwise be invisible. Mirror
                // the loud error to stderr so the operator always sees that their
                // settings (telegram token / projects / model / timezone) are not
                // being applied and where to recover them.
                eprintln!(
                    "omega: config parse FAILURE at {} — running on hardcoded \
                     DEFAULTS; your settings there are not applied. Original \
                     preserved at {} (fix or restore it): {}",
                    config_path.display(),
                    backup.display(),
                    e
                );
                tracing::error!(
                    path = %config_path.display(),
                    backup = %backup.display(),
                    error = %e,
                    "config.toml failed to parse — running on DEFAULTS; original preserved at \
                     .corrupt. Fix or restore it: telegram token / projects / model are lost \
                     until then."
                );
                Err(e.into())
            }
        }
    }

    pub fn config_path() -> PathBuf {
        omega_dir().join("config.toml")
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
        assert_eq!(cfg.agent_command, "codex");
        assert_eq!(cfg.default_model, "opus");
    }

    #[test]
    fn theme_background_defaults_on_and_round_trips() {
        // Pre-existing configs (no theme_background key) must keep painting the
        // background — default ON, not serde's bool default (false).
        let cfg: OmegaConfig = toml::from_str("").unwrap();
        assert!(cfg.theme_background);
        // An explicit OFF survives a save/load round-trip.
        let mut cfg = OmegaConfig::default();
        cfg.theme_background = false;
        let reloaded: OmegaConfig =
            toml::from_str(&toml::to_string_pretty(&cfg).unwrap()).unwrap();
        assert!(!reloaded.theme_background);
    }

    #[test]
    fn partial_config_overrides_only_its_keys() {
        // A partial config overrides its keys and keeps defaults for the rest,
        // instead of nuking the whole file.
        let cfg: OmegaConfig = toml::from_str("default_model = \"sonnet\"\n").unwrap();
        assert_eq!(cfg.default_model, "sonnet"); // overridden
        assert_eq!(cfg.agent_command, "codex"); // still the default, not empty
    }

    #[test]
    fn malformed_config_surfaces_error_and_preserves_original() {
        // A corrupt config.toml must NOT be silently swallowed into defaults:
        // load_from returns Err (so callers can choose) and copies the original
        // to .corrupt so the operator can recover their telegram token / projects.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "default_model = \"opus\"\nbroken = = =\n[[[").unwrap();

        let r = OmegaConfig::load_from(&path);
        assert!(r.is_err(), "malformed config must surface an error, not default silently");

        let backup = tmp.path().join("config.toml.corrupt");
        assert!(backup.exists(), "the unparseable original must be preserved at .corrupt");
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            std::fs::read_to_string(&path).unwrap(),
            "the .corrupt backup must be a faithful copy of the original"
        );
    }

    #[test]
    fn absent_config_is_default_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = OmegaConfig::load_from(&tmp.path().join("nonexistent.toml")).unwrap();
        assert_eq!(cfg.agent_command, "codex");
    }

    #[test]
    fn omega_dir_honors_explicit_env_override() {
        // $OMEGA_DIR wins over everything — this is what lets the whole system
        // relocate with one env var (and makes the binary agree with install.sh).
        // Serialize the env mutation; cargo runs tests in parallel.
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("OMEGA_DIR").ok();
        std::env::set_var("OMEGA_DIR", "/tmp/omega-test-root");
        assert_eq!(omega_dir(), PathBuf::from("/tmp/omega-test-root"));
        // empty value is ignored (falls through to a real default, never "")
        std::env::set_var("OMEGA_DIR", "");
        assert_ne!(omega_dir(), PathBuf::from(""));
        match prev {
            Some(v) => std::env::set_var("OMEGA_DIR", v),
            None => std::env::remove_var("OMEGA_DIR"),
        }
    }
}
