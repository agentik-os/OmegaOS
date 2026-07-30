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
    /// Runtime used when a caller does not provide an explicit agent.
    ///
    /// Missing fields use Codex. An explicit serialized value is always
    /// preserved, including `claude`: config loading has no provenance proving
    /// whether an old value was untouched or deliberately chosen.
    #[serde(default = "default_agent_command")]
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
    /// What the daily update cron is allowed to do (`omega update --auto`).
    /// Default `Apply` — the check runs every day and the update is installed.
    #[serde(default)]
    pub auto_update: AutoUpdatePolicy,
    pub telegram: Option<TelegramConfig>,
}

/// How far the daily update cron may go on its own.
///
/// This is the ONE switch between "OmegaOS keeps itself current" and "OmegaOS
/// tells me and I decide". It exists because auto-applying code pulled from a
/// remote is a real trust decision, not a preference: `Apply` means whoever
/// controls the repo controls this machine's OmegaOS — the same trust the
/// installer already required, now renewed daily instead of once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AutoUpdatePolicy {
    /// Check daily and install what is available. The default.
    #[default]
    Apply,
    /// Check daily, alert when an update exists, install nothing.
    Check,
    /// Do nothing at all — the cron exits immediately.
    Off,
}

impl AutoUpdatePolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Apply => "apply",
            Self::Check => "check",
            Self::Off => "off",
        }
    }

    /// Parse a user-typed value (config file, `omega config set`). Unknown text
    /// is NOT silently treated as `off` — a typo must not quietly stop updates,
    /// so it falls back to the default.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "check" | "notify" | "check-only" => Self::Check,
            "off" | "false" | "no" | "disabled" | "never" => Self::Off,
            _ => Self::Apply,
        }
    }
}

fn default_agent_command() -> String {
    "codex".to_string()
}

fn default_aisb_agent() -> String {
    "codex".to_string()
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
            // Codex is the default runtime for every new session whose caller
            // does not deliberately choose another provider. Deserialization
            // preserves every explicit value, including a legacy `claude`.
            agent_command: default_agent_command(),
            // This field is provider-specific. Codex reads its model/effort
            // from ~/.codex/config.toml; "opus" remains the Claude default for
            // missions that explicitly select Claude.
            default_model: "opus".to_string(),
            aisb_agent: default_aisb_agent(),
            auto_spawn_master: default_auto_master(),
            auto_naming: default_auto_naming(),
            session_shortcuts: default_session_shortcuts(),
            theme: default_theme(),
            theme_background: default_theme_background(),
            timezone: None,
            auto_update: AutoUpdatePolicy::default(),
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

    /// Persist the auto-update policy into config.toml.
    ///
    /// Rewrites ONLY that one key and leaves every other byte — including the
    /// user's comments and formatting — untouched. Serializing the whole struct
    /// back would be shorter and would silently strip a hand-written config
    /// down to whatever the struct happens to model.
    ///
    /// The key must be written ABOVE the first `[table]` header, or TOML parses
    /// it as a member of that table instead of a top-level key.
    pub fn set_auto_update(policy: AutoUpdatePolicy) -> Result<()> {
        let path = Self::config_path();
        Self::set_auto_update_at(&path, policy)
    }

    fn set_auto_update_at(path: &Path, policy: AutoUpdatePolicy) -> Result<()> {
        let line = format!("auto_update = \"{}\"", policy.as_str());
        let existing = std::fs::read_to_string(path).unwrap_or_default();

        let mut out: Vec<String> = Vec::new();
        let mut written = false;
        for raw in existing.lines() {
            let trimmed = raw.trim_start();
            if trimmed.starts_with("auto_update") && trimmed.contains('=') && !written {
                out.push(line.clone());
                written = true;
                continue;
            }
            // First table header: the last point where a top-level key is still
            // top-level. Insert here if we have not written the key yet.
            if !written && trimmed.starts_with('[') {
                out.push(line.clone());
                out.push(String::new());
                written = true;
            }
            out.push(raw.to_string());
        }
        if !written {
            out.push(line);
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut body = out.join("\n");
        body.push('\n');
        std::fs::write(path, body)?;
        Ok(())
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
    fn auto_update_defaults_to_apply_so_installs_stay_current() {
        assert_eq!(OmegaConfig::default().auto_update, AutoUpdatePolicy::Apply);
        // And an empty config.toml must not silently turn updates off.
        let parsed: OmegaConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.auto_update, AutoUpdatePolicy::Apply);
    }

    #[test]
    fn auto_update_round_trips_through_config_toml() {
        for policy in [
            AutoUpdatePolicy::Off,
            AutoUpdatePolicy::Check,
            AutoUpdatePolicy::Apply,
        ] {
            let toml_str = format!("auto_update = \"{}\"", policy.as_str());
            let parsed: OmegaConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.auto_update, policy);
        }
    }

    #[test]
    fn setting_the_policy_preserves_the_rest_of_the_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            "# my notes\nagent_command = \"claude\"\n\n[telegram]\nbot_token = \"t\"\nchat_id = 42\n",
        )
        .unwrap();

        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Off).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert!(back.contains("# my notes"), "comments survive");
        assert!(back.contains("chat_id = 42"), "telegram survives");
        assert!(back.contains("auto_update = \"off\""));

        // And it must still parse, with the key top-level rather than swallowed
        // into [telegram].
        let cfg: OmegaConfig = toml::from_str(&back).unwrap();
        assert_eq!(cfg.auto_update, AutoUpdatePolicy::Off);
        assert!(cfg.telegram.is_some());
    }

    #[test]
    fn setting_the_policy_twice_replaces_rather_than_appends() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Off).unwrap();
        OmegaConfig::set_auto_update_at(&path, AutoUpdatePolicy::Check).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert_eq!(back.matches("auto_update").count(), 1, "one key, not two");
        let cfg: OmegaConfig = toml::from_str(&back).unwrap();
        assert_eq!(cfg.auto_update, AutoUpdatePolicy::Check);
    }

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
        assert_eq!(cfg.aisb_agent, "codex");
        assert_eq!(cfg.default_model, "opus");
    }

    #[test]
    fn fresh_and_missing_agent_configuration_defaults_to_codex() {
        let fresh = OmegaConfig::default();
        assert_eq!(fresh.agent_command, "codex");
        assert_eq!(fresh.aisb_agent, "codex");

        // A partial pre-existing config with no agent field adopts the new
        // default without requiring a rewrite of the operator's file.
        let missing: OmegaConfig = toml::from_str("default_model = \"sonnet\"\n").unwrap();
        assert_eq!(missing.agent_command, "codex");
        assert_eq!(missing.aisb_agent, "codex");
        assert_eq!(missing.default_model, "sonnet");
    }

    #[test]
    fn explicit_agent_choices_are_never_migrated_by_config_loading() {
        for explicit in ["claude", "gemini", "codex"] {
            let body = format!("agent_command = \"{explicit}\"\naisb_agent = \"{explicit}\"\n");
            let parsed: OmegaConfig = toml::from_str(&body).unwrap();
            assert_eq!(parsed.agent_command, explicit);
            assert_eq!(parsed.aisb_agent, explicit);
        }

        // `load_from` has no provenance that could distinguish an untouched
        // legacy "claude" from a deliberate operator choice, so it preserves it.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, "agent_command = \"claude\"\n").unwrap();
        let loaded = OmegaConfig::load_from(&path).unwrap();
        assert_eq!(loaded.agent_command, "claude");
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
        assert_eq!(cfg.aisb_agent, "codex");
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
