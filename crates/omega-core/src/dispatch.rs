use crate::config::OmegaConfig;
use crate::done::DoneSignal;
use crate::oracle_lifecycle::{OraclePromptGenerator, OracleRegistry, OracleState};
use crate::routing;
use crate::session::SessionManager;
use anyhow::{bail, Result};
use std::path::Path;
use std::time::Duration;

/// N20: Claude's `/goal` rejects conditions longer than ~4000 chars; an
/// over-long goal silently aborts the whole dispatch. We cap at 4000.
const MAX_GOAL_LEN: usize = 4000;

/// Map a config `default_model` alias to the explicit model name Claude's CLI
/// pins with `--model`. "opus" → "claude-opus-4-8"; any other value (including
/// a full model name like "claude-opus-4-8" or a bare alias such as "sonnet")
/// is passed through verbatim — the CLI accepts both aliases and full names.
fn resolve_model_flag(default_model: &str) -> String {
    match default_model {
        "opus" => "claude-opus-4-8".to_string(),
        other => other.to_string(),
    }
}

/// Generate a fresh RFC-4122 v4-formatted UUID string for Claude's
/// `--session-id` flag (which validates the value as a UUID). We have no
/// `uuid` crate dependency, so we mix two u64s of time + atomic-counter
/// entropy (the same scheme as `MissionId`) into 128 bits and stamp the
/// version (4) and variant (10xx) nibbles per the spec.
fn gen_session_uuid() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let pid = std::process::id() as u64;

    let hi = nanos ^ counter.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let lo = (pid.wrapping_mul(0xA24B_AED4_963E_E407))
        ^ nanos.rotate_left(32)
        ^ counter.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);

    // 16 bytes from the two words.
    let mut b = [0u8; 16];
    b[..8].copy_from_slice(&hi.to_be_bytes());
    b[8..].copy_from_slice(&lo.to_be_bytes());
    // Version 4 (random): top nibble of byte 6.
    b[6] = (b[6] & 0x0f) | 0x40;
    // Variant 10xx: top bits of byte 8.
    b[8] = (b[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

/// Mint a FRESH `--session-id` for an oracle dispatch and persist it.
///
/// CRITICAL: `claude --session-id <uuid>` CREATES a session with that exact id and
/// fails hard ("Session ID … is already in use") if one already exists. Reusing a
/// persisted id on a re-dispatch / idle-reuse / resurrect therefore collides and the
/// oracle pane never launches Claude (it drops to a bare shell with the error). A
/// dispatch is a NEW mission = a NEW conversation, so we ALWAYS mint a fresh UUID
/// (which `gen_session_uuid` guarantees is unique) and overwrite the persisted one
/// for the record. Best-effort — a persistence failure still returns a usable id.
fn resolve_session_id(
    state_dir: &Path,
    oracle_name: &str,
    project: &str,
    working_dir: &Path,
) -> String {
    let id = gen_session_uuid();
    let mut state = OracleState::read(state_dir, oracle_name)
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            OracleState::new_minimal(oracle_name, project, working_dir.to_path_buf())
        });
    state.session_id = Some(id.clone());
    if let Err(e) = state.write(state_dir) {
        tracing::warn!(oracle = %oracle_name, error = %e, "failed to persist session_id");
    }
    id
}

/// Structured context for worker dispatch — ensures every worker gets
/// the information it needs to be fully autonomous (Third Law compliant).
///
/// Mirrors the VPS Fresh Context Template:
/// Mission, Purpose, Context, What's Done, Current Task, Done Criteria,
/// Verify Command, Key Decisions, Files in Scope, Relevant Memories.
#[derive(Debug, Clone, Default)]
pub struct WorkerContext {
    pub mission: String,
    pub purpose: Option<String>,
    pub project: Option<String>,
    pub working_dir: Option<String>,
    pub done_criteria: String,
    pub verify_command: Option<String>,
    pub files_owned: Vec<String>,
    pub context_notes: Vec<String>,
    pub what_done: Vec<String>,
    pub key_decisions: Vec<String>,
    pub git_branch: Option<String>,
    pub git_recent_commits: Vec<String>,
}

impl WorkerContext {
    pub fn format_prompt(&self, worker_name: &str) -> String {
        let mut prompt = String::with_capacity(2048);
        prompt.push_str("[DISPATCHED] You are an autonomous worker. Third Law: decide and proceed, never wait.\n\n");

        prompt.push_str(&format!("## Mission\n{}\n\n", self.mission));

        if let Some(ref purpose) = self.purpose {
            prompt.push_str(&format!("## Purpose\n{}\n\n", purpose));
        }

        if let Some(ref project) = self.project {
            let dir_str = self.working_dir.as_deref().unwrap_or(".");
            prompt.push_str(&format!("## Context\nProject: {} ({})\n", project, dir_str));
            if let Some(ref branch) = self.git_branch {
                prompt.push_str(&format!("Branch: {}\n", branch));
            }
            if !self.git_recent_commits.is_empty() {
                prompt.push_str("Recent commits:\n");
                for c in &self.git_recent_commits {
                    prompt.push_str(&format!("  {}\n", c));
                }
            }
            prompt.push('\n');
        }

        if !self.what_done.is_empty() {
            prompt.push_str("## What's Done\n");
            for item in &self.what_done {
                prompt.push_str(&format!("- {}\n", item));
            }
            prompt.push('\n');
        }

        if !self.context_notes.is_empty() {
            prompt.push_str("## Current Task\n");
            for note in &self.context_notes {
                prompt.push_str(&format!("- {}\n", note));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!("## Done Criteria\n{}\n\n", self.done_criteria));

        if let Some(ref verify) = self.verify_command {
            prompt.push_str(&format!("## Verify Command\n```bash\n{}\n```\n\n", verify));
        }

        if !self.files_owned.is_empty() {
            prompt.push_str(&format!(
                "## Files in Scope\n{}\nOnly modify files in your scope.\n\n",
                self.files_owned.join(", ")
            ));
        }

        if !self.key_decisions.is_empty() {
            prompt.push_str("## Key Decisions\n");
            for d in &self.key_decisions {
                prompt.push_str(&format!("- {}\n", d));
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!(
            "## Completion\nWhen done: `omega done {} done_clean \"<summary>\"`\n\
             If blocked: `omega done {} blocked \"<what's blocking>\"`\n\
             If failed: `omega done {} failed \"<what went wrong>\"`\n",
            worker_name, worker_name, worker_name
        ));

        prompt
    }

    /// Collect git context from a working directory.
    pub fn with_git_context(mut self, working_dir: &Path) -> Self {
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                self.git_branch =
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        if let Ok(output) = std::process::Command::new("git")
            .args(["log", "--oneline", "-5"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                self.git_recent_commits = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|l| l.to_string())
                    .collect();
            }
        }

        self
    }
}

pub struct Dispatcher {
    session_mgr: SessionManager,
    config: OmegaConfig,
}

impl Dispatcher {
    pub fn new(session_mgr: SessionManager, config: OmegaConfig) -> Self {
        Self {
            session_mgr,
            config,
        }
    }

    pub async fn dispatch_oracle(
        &self,
        project: &str,
        mission: &str,
    ) -> Result<String> {
        // An oracle is scoped to a DECLARED project. A project not present in the
        // config may still be auto-discovered under the user's projects root —
        // `omega projects` lists those — so fall back to that same discovery walk
        // before failing. A genuinely-unknown name (typo) is a configuration
        // error: fail loud instead of silently spawning in an arbitrary CWD,
        // which would break scope isolation and run code in an unexpected dir.
        let work_dir = match self.config.find_project(project) {
            Some(pc) => pc.path.to_string_lossy().to_string(),
            None => {
                let lower = project.to_lowercase();
                // SSOT: resolve from the shared ProjectRegistry (~/.omega/projects.json) —
                // the SAME source the TUI Project menu + Telegram read — then fall back to
                // a $HOME discovery walk. This is why a Telegram-added project dispatches.
                let from_registry = crate::project_manager::ProjectRegistry::load()
                    .projects
                    .into_iter()
                    .find(|p| p.name.to_lowercase() == lower)
                    .map(|p| p.path.to_string_lossy().to_string());
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/home"));
                from_registry
                    .or_else(|| {
                        crate::projects::discover(&home)
                            .into_iter()
                            .find(|p| p.name.to_lowercase() == lower)
                            .map(|p| p.path.to_string_lossy().to_string())
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("project '{}' not found in registry or config", project)
                    })?
            }
        };
        let work_path = std::path::PathBuf::from(&work_dir);

        // Oracle naming + idle-reuse. A registry entry is NOT proof of life — an
        // idle oracle may have crashed or been killed in rmux — so verify the
        // reuse candidate against live rmux sessions first (async). Then hand
        // the verified candidate to reserve_oracle, which does the name pick +
        // registration under an exclusive lock: two concurrent dispatches can no
        // longer both compute the same next name and clobber each other's save.
        let live_names = self.session_mgr.list_sessions().await.unwrap_or_default();
        let preferred: Option<String> = OracleRegistry::load(&self.config.state_dir)
            .find_available(project)
            .map(|idle| idle.oracle_name.clone())
            .filter(|name| live_names.iter().any(|s| &s.name == name))
            // N10: a registry-Idle oracle is only safe to REUSE once it has
            // reached a real closeable done-state (is_closeable/has_done_signal
            // on OracleState — strictly stronger than all_workers_terminal()).
            // An Idle oracle still owing Verify/Report work must NOT be reused.
            .filter(|name| {
                OracleState::read(&self.config.state_dir, name)
                    .ok()
                    .flatten()
                    .map(|st| st.is_closeable())
                    .unwrap_or(false)
            });
        let oracle_name =
            OracleRegistry::reserve_oracle(&self.config.state_dir, project, preferred.as_deref())?;

        // Classification + ship/god-mode detection run on the RAW message —
        // keyword signals ("ship", "god mode") must not be lost to
        // restructuring.
        let decision = routing::classify_mission(mission);
        let ship = OraclePromptGenerator::should_ship(mission);
        let god_mode = OraclePromptGenerator::is_god_mode(mission);

        // Amplify the raw message into a structured ## Mission/Context/Tasks/
        // Success Criteria/Constraints brief BEFORE it becomes the oracle's
        // mission body. Skip-gated + cached; falls back to raw on failure.
        // (blocking subprocess → spawn_blocking)
        let amplified = {
            let raw = mission.to_string();
            let proj = project.to_string();
            let wd = work_dir.clone();
            tokio::task::spawn_blocking(move || crate::amplify::amplify_mission(&raw, &proj, &wd))
                .await
                .unwrap_or_else(|_| mission.to_string())
        };

        // Generate structured oracle prompt
        let mut prompt = OraclePromptGenerator::generate(
            project,
            &work_path,
            &oracle_name,
            &amplified,
            ship,
            god_mode,
        );

        // Append detected audit skills
        if !decision.audit_skills.is_empty() {
            prompt.push_str(&format!(
                "\n## Detected Audit Skills\n{}\nDispatch each as a separate worker with `/skillname` on line 1.\n",
                decision.audit_skills.iter()
                    .map(|a| format!("- /{} (triggered by '{}')", a.skill, a.trigger))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Append complexity hint
        prompt.push_str(&format!("\n## Complexity: {:?}\n", decision.complexity));

        // THE FUNNEL — every dispatched agent (any LLM backend) MUST receive
        // its role-scoped Laws + operational rules via this single call.
        // This closes the gap where CLI/RPC-dispatched oracles previously
        // launched without their inviolable Laws.
        let ctx = crate::rules::agent_context_block(crate::rules::RuleScope::Oracle);
        if !ctx.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&ctx);
        }

        // Claude-only smart spawn (2026-w20 features): /goal + --effort +
        // budget caps. Gemini/Codex/GLM/Pi/Hermes fall back to the bare
        // launcher with the same prompt.
        let agent = crate::agents::Agent::from_name(&self.config.agent_command)
            .unwrap_or(crate::agents::Agent::Claude);
        if matches!(agent, crate::agents::Agent::Claude) {
            let mut opts = crate::agents::LaunchOptions::default();
            // Ultracode posture: the oracle is the strategic brain — it reasons
            // hard on every mission. Floor raised to high; Complex/Epic go xhigh/max.
            // (Model is Opus 4.8 via the default config; effort is the reasoning depth.)
            opts.effort = Some(match decision.complexity {
                routing::Complexity::Simple => "high".to_string(),
                routing::Complexity::Medium => "xhigh".to_string(),
                routing::Complexity::Complex => "xhigh".to_string(),
                routing::Complexity::Epic => "max".to_string(),
            });
            // Pin the model explicitly so the spawned oracle never silently
            // drifts onto the CLI's default. "opus" → claude-opus-4-8.
            opts.model = Some(resolve_model_flag(&self.config.default_model));
            // N5: --max-budget-usd is a no-op for interactive spawned sessions
            // (the flag only bounds non-interactive `-p` runs), so we do NOT
            // set it here and make no cost guarantee. max_turns still bounds
            // runaway loops. Real out-of-band budget enforcement is deferred.
            opts.max_turns = Some(match decision.complexity {
                routing::Complexity::Simple => 15,
                routing::Complexity::Medium => 50,
                routing::Complexity::Complex => 150,
                routing::Complexity::Epic => 400,
            });
            opts.session_name = Some(oracle_name.clone());
            // ── Oracle role (Lane A, interactive TTY) ────────────────────
            // Per-role LaunchOptions: an oracle is the strategic brain on an
            // ATTACHABLE pane, so every flag below is interactive-safe (no
            // --print / stream-json). It gets the full interactive posture:
            //   * permission-mode "auto" — auto-approve safe ops while keeping
            //     the pane interactive (replaces blanket skip-perms; see
            //     agents.rs:234). NOT a hermetic worker, so no disallowed_tools.
            //   * a persisted --session-id UUID so a daemon restart / resurrect
            //     resumes the SAME conversation instead of orphaning it.
            //   * --debug-file under ~/.omega/state for post-mortem (keeps TTY).
            //   * --exclude-dynamic-system-prompt-sections — cross-session
            //     prompt-cache reuse; SAFE because we inject via
            //     --append-system-prompt-file, not --system-prompt.
            // NOTE on --bare: deliberately NOT set for oracles. --bare flips
            // auth to API-key-only and disables CLAUDE.md autodiscovery — an
            // oracle depends on both, so bare is reserved for hermetic worker
            // roles (spawned elsewhere via spawn-worker), never the oracle.
            // A dispatched oracle is AUTONOMOUS (L3: decide and proceed, never wait).
            // It must BUILD a todo plan and then EXECUTE it without pausing for human
            // approval — so we do NOT use `--permission-mode plan` (that gate stops on
            // an interactive pane waiting for the operator to accept the plan, the exact
            // friction the operator rejects). The "plan" is a working method enforced by
            // the oracle doctrine (build the todo list, finish 100%), NOT a permission
            // gate. Leave permission_mode unset → the base command keeps
            // --dangerously-skip-permissions, so the oracle plans-and-proceeds fully
            // autonomously across every complexity tier.
            opts.permission_mode = None;
            // --brief enables the SendUserMessage agent→user tool so the oracle can
            // push a structured note to the human (oracle-only; workers stay silent).
            opts.brief = true;
            // --verbose: full tool/log visibility on the oracle's attachable pane.
            opts.verbose = true;
            // Wire OmegaOS tools as MCP servers for the oracle. NOT strict (the
            // oracle keeps access to user/project .mcp.json too); strict_mcp_config
            // is reserved for hermetic workers. Best-effort: a write failure logs
            // and the oracle still launches without the extra servers.
            match crate::mcp_servers::generate_mcp_config(&self.config, &oracle_name) {
                Ok(json) => {
                    let path = self
                        .config
                        .state_dir
                        .join(format!("{}.mcp.json", oracle_name));
                    match std::fs::write(&path, json) {
                        Ok(()) => {
                            opts.mcp_config = Some(vec![path.to_string_lossy().to_string()]);
                        }
                        Err(e) => tracing::warn!(
                            oracle = %oracle_name, error = %e,
                            "failed to write oracle mcp-config — launching without it"
                        ),
                    }
                }
                Err(e) => tracing::warn!(
                    oracle = %oracle_name, error = %e,
                    "failed to generate oracle mcp-config — launching without it"
                ),
            }
            opts.exclude_dynamic_prompt_sections = true;
            opts.session_id = Some(resolve_session_id(
                &self.config.state_dir,
                &oracle_name,
                project,
                &work_path,
            ));
            opts.debug_file = Some(
                self.config
                    .state_dir
                    .join(format!("{}.debug.log", oracle_name))
                    .to_string_lossy()
                    .to_string(),
            );
            // /goal — auto-derived success criteria. The oracle loops
            // until its own .done.json is written with status=done_clean
            // OR the build is green, depending on mission type.
            let goal = format!(
                "mission complete for project {} — .done.json written with status=done_clean and either no code changes OR `cd {} && npm run build` (or the project's build script) exits zero",
                project, work_dir
            );
            // N20: Claude's /goal rejects conditions over ~4000 chars and the
            // whole dispatch silently fails (the 30638-char bug). Guard the
            // length: drop the /goal injection rather than ship a body the
            // CLI will reject. The oracle still has its full prompt + done.json
            // contract; it just won't auto-loop on an over-long goal.
            if goal.len() > MAX_GOAL_LEN {
                tracing::warn!(
                    oracle = %oracle_name,
                    goal_len = goal.len(),
                    max = MAX_GOAL_LEN,
                    "goal_condition exceeds /goal length limit — dropping the /goal injection"
                );
            } else {
                opts.goal_condition = Some(goal);
            }

            self.session_mgr
                .create_agent_session_with_opts(
                    &oracle_name,
                    &work_dir,
                    agent,
                    Some(&prompt),
                    opts,
                )
                .await?;
        } else {
            self.session_mgr
                .create_agent_session(
                    &oracle_name,
                    &work_dir,
                    &self.config.agent_command,
                    Some(&prompt),
                )
                .await?;
        }

        // (The oracle was already registered Active under the lock by
        // reserve_oracle above; a failed spawn self-heals via patrol cleanup,
        // which marks registry entries with no live rmux session Dead.)

        tracing::info!(
            oracle = %oracle_name,
            project = %project,
            complexity = ?decision.complexity,
            audits = decision.audit_skills.len(),
            ship = %ship,
            god_mode = %god_mode,
            "Oracle dispatched"
        );
        // AUDIT JOURNAL: record the dispatch under ~/.omega/audit/<project>/ (best-effort).
        {
            let dir = self.config.state_dir.parent().map(|p| p.join("audit").join(project));
            if let Some(dir) = dir {
                let _ = std::fs::create_dir_all(&dir);
                let line = format!(
                    "{{\"ts\":\"{}\",\"event\":\"dispatch\",\"oracle\":\"{}\",\"complexity\":\"{:?}\",\"mission\":{}}}\n",
                    chrono::Utc::now().to_rfc3339(),
                    oracle_name,
                    decision.complexity,
                    serde_json::to_string(&mission.chars().take(500).collect::<String>()).unwrap_or_else(|_| "\"\"".into()),
                );
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("audit.jsonl")) {
                    let _ = f.write_all(line.as_bytes());
                }
            }
        }
        Ok(oracle_name)
    }

    /// Re-spawn a crashed oracle from its persisted OracleState — survives a
    /// daemon restart or an accidental kill. Returns whether it was actually
    /// resurrected, was already alive, or had no saved state.
    pub async fn resurrect_oracle(&self, oracle_name: &str) -> Result<ResurrectOutcome> {
        let state = match OracleState::read(&self.config.state_dir, oracle_name)? {
            Some(s) => s,
            None => return Ok(ResurrectOutcome::NotFound),
        };
        let alive = self
            .session_mgr
            .list_sessions()
            .await
            .unwrap_or_default()
            .iter()
            .any(|s| s.name == oracle_name);
        if alive {
            return Ok(ResurrectOutcome::AlreadyAlive);
        }

        let mut prompt = build_resume_prompt(&state);
        // THE FUNNEL — a resurrected oracle gets its Oracle-scoped doctrine too.
        let ctx = crate::rules::agent_context_block(crate::rules::RuleScope::Oracle);
        if !ctx.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&ctx);
        }

        let work_dir = state.working_dir.to_string_lossy().to_string();
        let agent = crate::agents::Agent::from_name(&self.config.agent_command)
            .unwrap_or(crate::agents::Agent::Claude);
        if matches!(agent, crate::agents::Agent::Claude) {
            let mut opts = crate::agents::LaunchOptions::default();
            opts.effort = Some("xhigh".to_string());
            opts.model = Some(resolve_model_flag(&self.config.default_model));
            opts.session_name = Some(oracle_name.to_string());
            // Resurrect path: same interactive oracle posture as a fresh
            // dispatch, plus --fork-session so the resumed run forks to a NEW
            // session instead of mutating the crashed original. The base
            // --session-id is the SAME persisted UUID (resolve_session_id reuses
            // state.session_id), so the fork derives from the right lineage.
            opts.permission_mode = Some("auto".to_string());
            opts.exclude_dynamic_prompt_sections = true;
            opts.fork_session = true;
            opts.session_id = Some(resolve_session_id(
                &self.config.state_dir,
                oracle_name,
                &state.project,
                &state.working_dir,
            ));
            opts.debug_file = Some(
                self.config
                    .state_dir
                    .join(format!("{}.debug.log", oracle_name))
                    .to_string_lossy()
                    .to_string(),
            );
            let goal = format!(
                "mission complete for project {} — .done.json written with status=done_clean",
                state.project
            );
            if goal.len() > MAX_GOAL_LEN {
                tracing::warn!(
                    oracle = %oracle_name,
                    goal_len = goal.len(),
                    max = MAX_GOAL_LEN,
                    "goal_condition exceeds /goal length limit — dropping the /goal injection"
                );
            } else {
                opts.goal_condition = Some(goal);
            }
            self.session_mgr
                .create_agent_session_with_opts(oracle_name, &work_dir, agent, Some(&prompt), opts)
                .await?;
        } else {
            self.session_mgr
                .create_agent_session(
                    oracle_name,
                    &work_dir,
                    &self.config.agent_command,
                    Some(&prompt),
                )
                .await?;
        }
        Ok(ResurrectOutcome::Resurrected)
    }

    /// Oracle names that have a persisted OracleState but no live session —
    /// candidates for `omega resurrect`.
    pub async fn dead_oracles(&self) -> Vec<String> {
        let alive: Vec<String> = self
            .session_mgr
            .list_sessions()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.name)
            .collect();
        OracleState::read_all(&self.config.state_dir)
            .into_iter()
            .filter(|st| !alive.contains(&st.oracle_name))
            .map(|st| st.oracle_name)
            .collect()
    }

    pub async fn wait_for_done(
        &self,
        session_name: &str,
        timeout: Duration,
    ) -> Result<DoneSignal> {
        let done_path = self
            .config
            .state_dir
            .join(format!("worker-{}.done.json", session_name));

        let start = std::time::Instant::now();
        loop {
            if done_path.exists() {
                let content = std::fs::read_to_string(&done_path)?;
                return Ok(serde_json::from_str(&content)?);
            }
            if start.elapsed() > timeout {
                bail!("Timeout waiting for done signal from {}", session_name);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    pub fn session_manager(&self) -> &SessionManager {
        &self.session_mgr
    }
}

/// Outcome of a [`Dispatcher::resurrect_oracle`] attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResurrectOutcome {
    Resurrected,
    AlreadyAlive,
    NotFound,
}

/// Build the resume prompt for a resurrected oracle from its persisted state —
/// mission + last phase + the workers it had already dispatched, with a strong
/// "don't duplicate completed work" instruction.
fn build_resume_prompt(state: &OracleState) -> String {
    let mut p = String::new();
    p.push_str(
        "[RESURRECTED] Your oracle session crashed or was killed; your state was \
         persisted. Resume exactly where you left off — do NOT restart the mission \
         from scratch.\n\n",
    );
    p.push_str(&format!("## Project\n{}\n\n", state.project));
    p.push_str(&format!("## Mission\n{}\n\n", state.mission_text));
    p.push_str(&format!(
        "## Last phase\n{:?} — re-assess, then continue.\n\n",
        state.phase
    ));
    if state.workers.is_empty() {
        p.push_str("## Workers\nNone dispatched yet.\n\n");
    } else {
        p.push_str("## Workers already dispatched\n");
        for w in &state.workers {
            p.push_str(&format!(
                "- '{}' [{:?}] — session {}\n",
                w.task_name, w.status, w.session_name
            ));
        }
        p.push_str(
            "\nBefore re-dispatching: check each worker's session + done.json. \
             Do NOT duplicate completed work.\n\n",
        );
    }
    p.push_str(
        "## Resume\nVerify what's already done (workers' done.json + git state), \
         continue to completion, then write your own .done.json.\n",
    );
    p
}

#[cfg(test)]
mod resurrect_tests {
    use super::*;
    use crate::mission::Mission;
    use crate::oracle_lifecycle::{OracleState, WorkerEntry, WorkerEntryStatus};
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn resume_prompt_carries_mission_workers_and_no_dupe_warning() {
        let mission = Mission::new("Acme", "ship the feature", PathBuf::from("/tmp"));
        let mut state = OracleState::new("oracle-Acme-1", &mission);
        state.register_worker(WorkerEntry {
            session_name: "Acme-worker-auth".into(),
            task_id: "t1".into(),
            task_name: "auth".into(),
            files_owned: vec![],
            dispatched_at: Utc::now(),
            status: WorkerEntryStatus::DoneClean,
        });
        let p = build_resume_prompt(&state);
        assert!(p.contains("[RESURRECTED]"));
        assert!(p.contains("ship the feature"));
        assert!(p.contains("auth"));
        assert!(p.contains("Do NOT duplicate completed work"));
    }
}
