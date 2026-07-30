use crate::config::OmegaConfig;
use crate::done::DoneSignal;
use crate::oracle_lifecycle::{
    OraclePromptGenerator, OracleRegistry, OracleRegistryEntry, OracleRegistryStatus, OracleState,
};
use crate::routing;
use crate::session::SessionManager;
use anyhow::{bail, Result};
use std::path::Path;
use std::time::Duration;

/// N20: Claude's `/goal` rejects conditions longer than ~4000 chars; an
/// over-long goal silently aborts the whole dispatch. We cap at 4000.
const MAX_GOAL_LEN: usize = 4000;

/// How many times to look for a typeable composer before giving up on a
/// followup. A freshly-spawned oracle needs a few seconds to attach its agent
/// TUI to the pane, and the incident this exists for saw two dispatches land
/// 24 SECONDS apart — well inside that window.
const FOLLOWUP_PANE_ATTEMPTS: usize = 5;

/// Wait between composer probes. `ATTEMPTS * INTERVAL` is the whole bound:
/// past it we spawn instead of typing into an unknown pane.
const FOLLOWUP_PANE_INTERVAL: Duration = Duration::from_secs(2);

/// How many times to look for evidence that the paste was ACCEPTED before
/// refusing to report a success.
const FOLLOWUP_CONFIRM_ATTEMPTS: usize = 3;

/// Wait between acceptance probes.
const FOLLOWUP_CONFIRM_INTERVAL: Duration = Duration::from_secs(1);

/// Where a dispatch should be delivered. Computed by [`route_dispatch`] as a
/// PURE function of the registry, the live rmux session list, and each oracle's
/// closeable state — which is the only reason the decision is unit-testable
/// with zero rmux and zero daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchRoute {
    /// Spawn a session. `preferred` is the pre-existing idle-reuse candidate
    /// (an idle + live + closeable oracle whose NAME is recycled); `None` means
    /// the registry allocates the next name.
    Spawn { preferred: Option<String> },
    /// Deliver into the live oracle named here — its mission is still running,
    /// so its conversation already carries the context the operator is adding
    /// to. No new session, no new mission, no new state.
    Followup { oracle: String },
}

/// How a dispatch was actually delivered. Carried back to the CLI so the
/// operator (and the Telegram bridge) can see whether a sibling oracle was
/// created or a live one was reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDelivery {
    /// A session was spawned (possibly under a recycled idle name).
    Spawned,
    /// The mission text was typed into an already-live oracle.
    Followup,
    /// A followup was the right call, but the target pane never presented a
    /// typeable composer inside the bound, so a new oracle was spawned rather
    /// than risk the text landing in a shell.
    SpawnedPaneNotReady,
}

/// The result of a dispatch: which oracle owns the mission, and how the text
/// got there.
#[derive(Debug, Clone)]
pub struct DispatchOutcome {
    pub oracle_name: String,
    pub delivery: DispatchDelivery,
}

impl DispatchOutcome {
    /// The operator-facing report lines.
    ///
    /// LINE 0 IS A CONTRACT, NOT A COSMETIC CHOICE. The Telegram bridge shells
    /// out to `omega dispatch` and recovers the oracle name by matching
    /// `/Oracle dispatched:?\s*(oracle-[A-Za-z0-9._-]+)/` against stdout
    /// (telegram-bot/omega-tg-bot.ts:1273). A followup that announced itself
    /// with a different label would parse as a FAILED dispatch and the bridge
    /// would lose the thread — no progress card, no report. So every path
    /// prints the canonical line, and the followup says so on an ADDITIONAL
    /// line that the regex simply does not care about.
    pub fn report_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("◆ Oracle dispatched: {}", self.oracle_name)];
        match self.delivery {
            DispatchDelivery::Spawned => {}
            DispatchDelivery::Followup => lines.push(
                "  (suivi: route dans l oracle vivant, aucun nouvel oracle cree)".to_string(),
            ),
            DispatchDelivery::SpawnedPaneNotReady => lines.push(
                "  (suivi impossible: le pane de l oracle vivant n etait pas pret, \
                 nouvel oracle spawne)"
                    .to_string(),
            ),
        }
        lines.push(format!("DISPATCH_DELIVERY={}", self.delivery.tag()));
        lines
    }
}

impl DispatchDelivery {
    /// THE MACHINE-READABLE HALF OF THE OUTPUT CONTRACT, and the reason the
    /// French line above is not it: a consumer parsing prose breaks on the day
    /// somebody rewords it. Exactly one line of the form
    /// `DISPATCH_DELIVERY=<tag>` is printed on every SUCCESS path, and these
    /// three tags are the whole vocabulary.
    ///
    /// Line 0 stays the canonical `Oracle dispatched: <name>` — the Telegram
    /// bridge's regex owns that line and must keep matching it (see
    /// [`DispatchOutcome::report_lines`]).
    pub fn tag(&self) -> &'static str {
        match self {
            DispatchDelivery::Spawned => "spawned",
            DispatchDelivery::Followup => "followup",
            DispatchDelivery::SpawnedPaneNotReady => "spawned_pane_not_ready",
        }
    }
}

/// The env var that switches followup routing on.
pub const FOLLOWUP_ROUTING_ENV: &str = "OMEGA_FOLLOWUP_ROUTING";

/// Is followup routing switched on for this process?
///
/// SHIPPED OPT-IN, DELIBERATELY. The routing DECISION ([`route_dispatch`]) is
/// sound, but the DELIVERY half was audited and found to accept panes that are
/// not the agent's composer — including a live bash shell left behind by
/// `bash -c '<agent> …; exec bash'` after the agent dies, where every line of
/// the mission would execute as a command. Until the probe is proven against
/// those shapes, the whole branch stays behind an explicit flag: with the
/// variable absent, `omega dispatch` behaves exactly as it did before the
/// followup merge and always spawns.
pub fn followup_routing_enabled() -> bool {
    followup_routing_enabled_from(std::env::var(FOLLOWUP_ROUTING_ENV).ok().as_deref())
}

/// The pure half of [`followup_routing_enabled`], so the parsing is testable
/// without mutating the process environment from a parallel test thread.
fn followup_routing_enabled_from(raw: Option<&str>) -> bool {
    matches!(
        raw.map(str::trim),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// Decide where a mission for `project` goes: into a live oracle as a followup,
/// or into a spawn.
///
/// THE STATUS FIELD LIES, IN BOTH DIRECTIONS, and this function is written
/// around that. Observed in runtime during the incident that motivated it:
/// `oracle-dentistrygpt` sat at `active` with its mission already finished,
/// while `oracle-dentistrygpt-2` sat at `idle` while actively working. So
/// liveness is NEVER read off `status` — it is the intersection of the registry
/// and the LIVE rmux session list, qualified by the oracle's own closeable
/// state.
///
/// `closeable(name)` returns `Some(true)` when that oracle's `OracleState` says
/// its mission is over, `Some(false)` when it is still owed work, and `None`
/// when no state could be read. `None` is deliberately NOT a followup target:
/// every dispatched oracle writes a state file, so an unreadable one means we
/// cannot prove there is a live mission to add to, and the safe answer is the
/// pre-existing behavior (spawn).
///
/// NOTE on `is_closeable()`: the real incident states all carried
/// `phase: analyze` + `closeable_since: null`, so the predicate is false for
/// nearly every live oracle. That is FINE here and is the point: both a busy
/// oracle and a live not-yet-closed one are good followup targets, because in
/// both cases the conversation is alive and holds the context.
///
/// `has_done_signal(name)` is the THIRD condition and it is a hard veto. An
/// oracle with an `oracle-<key>.done.json` on disk is queued for reaping:
/// patrol kills the session AND its workers `ORACLE_CLOSE_GRACE_SECS` = 120s
/// after any closeable signal (patrol.rs:29, patrol.rs:1174-1236), so a followup
/// dropped there dies with the session. Clearing the signal to keep the session
/// alive was considered and REJECTED: the signal is keyed by NAME with no
/// mission id, so clearing it can destroy a report that was never delivered to
/// the operator. A finished oracle is simply not a followup target — we spawn.
pub fn route_dispatch<F, G>(
    entries: &[OracleRegistryEntry],
    project: &str,
    live_sessions: &[String],
    closeable: F,
    has_done_signal: G,
    force_new: bool,
) -> DispatchRoute
where
    F: Fn(&str) -> Option<bool>,
    G: Fn(&str) -> bool,
{
    // `--new` is the operator's explicit escape hatch: it skips ONLY the
    // followup branch. The idle-name recycling below is untouched, because
    // recycling a finished oracle's name still produces a brand-new mission.
    if !force_new {
        let mut candidates: Vec<&OracleRegistryEntry> = entries
            .iter()
            .filter(|e| e.project == project)
            .filter(|e| live_sessions.iter().any(|s| s == &e.session_name))
            .filter(|e| closeable(&e.oracle_name) == Some(false))
            .filter(|e| !has_done_signal(&e.oracle_name))
            .collect();
        // Freshest wins. With several live oracles on one project (the incident
        // had four), the most recently spawned is the one whose conversation the
        // operator is following up on.
        candidates.sort_by_key(|e| e.spawned_at);
        if let Some(target) = candidates.last() {
            return DispatchRoute::Followup {
                oracle: target.oracle_name.clone(),
            };
        }
    }

    // ── Pre-existing idle-reuse, semantics preserved verbatim ──────────────
    // First Idle entry for the project, kept only if it is really alive in rmux
    // AND has reached a genuine closeable done-state (N10). An Idle oracle that
    // still owes Verify/Report work is not reusable.
    let preferred = entries
        .iter()
        .find(|e| e.project == project && e.status == OracleRegistryStatus::Idle)
        .map(|e| e.oracle_name.clone())
        .filter(|name| live_sessions.iter().any(|s| s == name))
        .filter(|name| closeable(name) == Some(true));
    DispatchRoute::Spawn { preferred }
}

/// THE PRODUCTION `closeable` ADAPTER: does this oracle's own state say its
/// mission is over? `None` when no state could be read, which
/// [`route_dispatch`] treats as "prove nothing, spawn".
///
/// A free function, not a closure at the call site, because the audit's F12 is
/// exactly that the adapters were never executed by a test: every routing test
/// injected a hand-written constant, so inverting this predicate left all of
/// them green while re-delivering the incident.
pub fn oracle_is_closeable(state_dir: &Path, name: &str) -> Option<bool> {
    OracleState::read(state_dir, name)
        .ok()
        .flatten()
        .map(|st| st.is_closeable())
}

/// THE PRODUCTION `has_done_signal` ADAPTER: is this oracle queued for reaping?
pub fn oracle_has_done_signal(state_dir: &Path, name: &str) -> bool {
    crate::done::OracleDoneSignal::read(state_dir, name)
        .ok()
        .flatten()
        .is_some()
}

/// THE ONE WIRING of [`route_dispatch`] to the disk. Every caller — the
/// dispatch itself, the re-check immediately before the keystroke, and the
/// fallback that needs the idle-recycling candidate — goes through here, so
/// there is a single reading of the registry and of both adapters.
pub fn route_now(
    state_dir: &Path,
    project: &str,
    live_sessions: &[String],
    force_new: bool,
) -> DispatchRoute {
    route_dispatch(
        &OracleRegistry::load(state_dir).oracles,
        project,
        live_sessions,
        |name| oracle_is_closeable(state_dir, name),
        |name| oracle_has_done_signal(state_dir, name),
        force_new,
    )
}

/// Is this pane safe to type a followup into?
///
/// THE FAILURE THIS PREVENTS IS NOT HYPOTHETICAL, AND THE FIRST VERSION OF THIS
/// PREDICATE DID NOT PREVENT IT. It asked only for a rule-then-marker pair
/// ANYWHERE in the pane plus the absence of one known question modal, and a
/// forensic audit reproduced three panes that pass that test and must not:
///
///  1. A LIVE BASH SHELL. Every oracle runs as `bash -c '<agent> …; exec bash'`
///     (agents.rs:452), so an agent that dies — crash, `--max-turns`, budget,
///     auth, a reused session id — leaves the session UP, its last frame on
///     screen with the composer in it, and a shell prompt underneath. The
///     mission body would have executed there as command lines.
///  2. A MODAL THE BLACKLIST DOES NOT KNOW: the same question modal with its
///     hint hard-wrapped onto two lines (a narrow pane in a split layout), or a
///     numbered permission dialog, which draws no hint at all. The Enter that
///     submits a paste picks the highlighted option instead.
///  3. A COMPOSER HOLDING THE OPERATOR'S DRAFT. The paste does not clear (it
///     must not — that is the operator's text), so it CONCATENATES and submits
///     both as one turn.
///
/// So the evidence required is now POSITIVE and specific: a live-tail composer,
/// EMPTY, with the agent's own status bar under it, no selection list on
/// screen, and nothing typeable below it. Not-ready falls back to a spawn,
/// which is the whole reason a strict predicate is cheap here.
///
/// The shapes stay in `session_monitor` rather than being re-derived — that
/// module carries seven real false-positive scars — and only the POLICY (which
/// shapes are refused, and that a refusal means spawn) lives here.
pub fn pane_ready_for_followup(pane: &str) -> bool {
    crate::session_monitor::composer_ready_for_paste(pane)
        && !crate::session_monitor::question_ui_visible(pane)
}

/// Is there unsubmitted text sitting in the composer right now?
///
/// After a paste this is what tells "buffered but never submitted" apart from
/// "taken as a turn". No composer at all (the agent is redrawing, or took over
/// the screen) is NOT text held: there is nothing left holding our paste.
fn composer_holds_text(pane: &str) -> bool {
    match crate::session_monitor::find_live_composer_marker(pane) {
        Some(marker) => pane
            .lines()
            .nth(marker)
            .is_some_and(|line| !crate::session_monitor::composer_is_empty(line)),
        None => false,
    }
}

/// Map a config `default_model` alias to the explicit model name Claude's CLI
/// pins with `--model`. The default alias "opus" resolves to the 1M-context
/// Opus 5 variant ("claude-opus-5[1m]") so every dispatched session gets the
/// large context window without the config having to spell it out. "fable" →
/// "claude-fable-5"; any other value (including a full model name like
/// "claude-opus-4-8" or a bare alias such as "sonnet") is passed through
/// verbatim — the CLI accepts aliases, full names, and the "[1m]" suffix.
fn resolve_model_flag(default_model: &str) -> String {
    match default_model {
        "fable" => "claude-fable-5".to_string(),
        "opus" => "claude-opus-5[1m]".to_string(),
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

    /// Dispatch using the configured default agent (`config.agent_command`).
    pub async fn dispatch_oracle(&self, project: &str, mission: &str) -> Result<String> {
        self.dispatch_oracle_with_agent(project, mission, None, false)
            .await
            .map(|outcome| outcome.oracle_name)
    }

    /// Probe the pane until it presents a typeable composer, bounded by
    /// `FOLLOWUP_PANE_ATTEMPTS * FOLLOWUP_PANE_INTERVAL`. Returns whether it is
    /// safe to type. A capture error is treated as not-ready: we never type into
    /// a pane we could not read.
    async fn wait_for_typeable_pane(&self, session: &str) -> bool {
        for attempt in 1..=FOLLOWUP_PANE_ATTEMPTS {
            match self.session_mgr.capture_pane(session).await {
                Ok(pane) if pane_ready_for_followup(&pane) => return true,
                Ok(_) => tracing::debug!(
                    session = %session, attempt,
                    "followup target pane has no typeable composer yet"
                ),
                Err(e) => tracing::debug!(
                    session = %session, attempt, error = %e,
                    "failed to capture followup target pane"
                ),
            }
            if attempt < FOLLOWUP_PANE_ATTEMPTS {
                tokio::time::sleep(FOLLOWUP_PANE_INTERVAL).await;
            }
        }
        false
    }

    /// Is `oracle` STILL the followup target, read fresh off the disk and off
    /// the live session list?
    ///
    /// The probe above can sleep for eight seconds, and the fast path that ends
    /// an oracle is not patrol's 120s reap but the inline auto-close of
    /// `omega done`, which runs in seconds. So every condition the route was
    /// built on is re-read here, immediately before the keystroke, through the
    /// SAME production wiring that built it.
    async fn still_the_followup_target(&self, oracle: &str, project: &str) -> bool {
        let live: Vec<String> = self
            .session_mgr
            .list_sessions()
            .await
            .unwrap_or_default()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        matches!(
            route_now(&self.config.state_dir, project, &live, false),
            DispatchRoute::Followup { oracle: ref target } if target == oracle
        )
    }

    /// Was the paste ACCEPTED by the agent?
    ///
    /// A send that returns `Ok` proves the bytes reached the PTY and nothing
    /// more. The failure this exists for: the session dies in the seconds after
    /// a successful send, the mission goes down with it, and the operator reads
    /// `◆ Oracle dispatched:` — a success report for a mission that no longer
    /// exists anywhere.
    ///
    /// WHAT THIS PROVES, EXACTLY, and no more: the session still answers a
    /// capture, its pane MOVED, and the composer no longer holds the text.
    /// A capture that fails is the dead-session case and reads as NOT accepted.
    /// The pane still holding the paste means it was buffered but never
    /// submitted, so we keep looking until the bound. It does not prove the
    /// agent understood the mission — nothing observable from outside could.
    async fn confirm_followup_accepted(&self, oracle: &str, before: &str) -> bool {
        for attempt in 1..=FOLLOWUP_CONFIRM_ATTEMPTS {
            tokio::time::sleep(FOLLOWUP_CONFIRM_INTERVAL).await;
            let pane = match self.session_mgr.capture_pane(oracle).await {
                Ok(pane) => pane,
                Err(e) => {
                    tracing::warn!(
                        oracle = %oracle, attempt, error = %e,
                        "followup target stopped answering after the send — the mission went \
                         down with the session, refusing to report a success"
                    );
                    return false;
                }
            };
            if pane == before {
                continue; // nothing moved yet
            }
            if composer_holds_text(&pane) {
                continue; // buffered, not submitted
            }
            return true;
        }
        tracing::warn!(
            oracle = %oracle, attempts = FOLLOWUP_CONFIRM_ATTEMPTS,
            "followup paste was never confirmed as accepted — refusing to report a success"
        );
        false
    }

    /// Deliver `mission` into an ALREADY-LIVE oracle instead of spawning a
    /// sibling. Returns `Ok(false)` for EVERY way this can fail to deliver —
    /// pane never typeable, target no longer valid, send error, acceptance
    /// never confirmed — which tells the caller to fall through to the normal
    /// spawn path. That is deliberate: this function has no failure mode that
    /// is better served by killing the whole dispatch than by spawning, and a
    /// `?` here used to do exactly that, leaving the operator's mission
    /// undelivered anywhere while the neighbouring not-ready case fell back
    /// cleanly.
    ///
    /// Deliberately does NOT create a `Mission` in the ledger, does NOT write an
    /// `OracleState`, does NOT clear the done signal, and does NOT clear the
    /// `MissionLog` or the gate counters: one oracle is one mission is one
    /// state.json, and a followup is an addition to the mission already running
    /// there, not a second mission wearing the same name. Every one of those
    /// mutations lives on the spawn path AFTER this returns, and each would
    /// corrupt the running mission — `OracleState::new()` would wipe the live
    /// worker roster, `MissionLog::clear` would erase the R-LOOP timeline and
    /// reset the bounded-retry counters, and clearing the name-keyed done signal
    /// could destroy an undelivered report.
    ///
    /// It DOES append one `MissionLog::event` to the live oracle's timeline
    /// once the followup is confirmed — an append, by design, so `omega
    /// timeline` shows the operator that the running mission grew (loop_guard.rs
    /// :226-245: pure append, no rotation, no thrash or gate counter touched).
    async fn deliver_followup(&self, oracle: &str, project: &str, mission: &str) -> Result<bool> {
        if !self.wait_for_typeable_pane(oracle).await {
            tracing::warn!(
                oracle = %oracle, project = %project,
                attempts = FOLLOWUP_PANE_ATTEMPTS,
                "followup target never showed a typeable composer — spawning instead of typing \
                 into an unknown pane"
            );
            return Ok(false);
        }

        // The probe may have slept for eight seconds. Re-read the route, then
        // take one last look at the pane: both conditions must hold at the
        // moment of the keystroke, not at the moment of the decision.
        if !self.still_the_followup_target(oracle, project).await {
            tracing::warn!(
                oracle = %oracle, project = %project,
                "followup target stopped qualifying while we waited for its composer \
                 (closed, signalled, or gone) — spawning instead"
            );
            return Ok(false);
        }
        let before = match self.session_mgr.capture_pane(oracle).await {
            Ok(pane) if pane_ready_for_followup(&pane) => pane,
            Ok(_) => {
                tracing::warn!(
                    oracle = %oracle,
                    "followup target's composer stopped being typeable at the last look — \
                     spawning instead"
                );
                return Ok(false);
            }
            Err(e) => {
                tracing::warn!(
                    oracle = %oracle, error = %e,
                    "could not re-read the followup target's pane before typing — spawning instead"
                );
                return Ok(false);
            }
        };

        // BRACKETED PASTE, not a line-wise send. A mission body is multi-line
        // (it carries "## Attached files" and paths); `send_text` submits at the
        // first newline and would fracture one mission into a dozen half-turns.
        // `send_paste_then_submit` wraps the block in \e[200~ ... \e[201~ so the
        // TUI buffers it as ONE paste, then sends Enter as a separate key. It
        // chunks the body and replays the whole block on a stale pane, so a
        // failure can never leave a live composer stuck between the paste
        // markers eating the operator's keystrokes.
        if let Err(e) = self
            .session_mgr
            .send_paste_then_submit(oracle, mission)
            .await
        {
            tracing::warn!(
                oracle = %oracle, project = %project, error = %e,
                "failed to type the followup into the live oracle — spawning instead of failing \
                 the dispatch"
            );
            return Ok(false);
        }

        // NO CONFIRMATION, NO SUCCESS. An unconfirmed followup falls back to a
        // spawn and is reported as such: the operator gets an oracle that
        // certainly holds the mission, instead of a success line for a mission
        // that may have died with its session. The residual cost of the
        // fallback is a mission that landed twice, which is visible and
        // recoverable; the cost of the alternative is silence.
        if !self.confirm_followup_accepted(oracle, &before).await {
            return Ok(false);
        }

        crate::loop_guard::MissionLog::event(
            &self.config.state_dir,
            oracle,
            "followup",
            &format!(
                "followup delivered into the live mission: {}",
                mission.chars().take(140).collect::<String>()
            ),
        );
        tracing::info!(
            oracle = %oracle, project = %project,
            "Followup routed into a live oracle — no sibling spawned"
        );
        Ok(true)
    }

    /// Dispatch, optionally overriding the agent for THIS mission only.
    ///
    /// `agent_override` is the per-mission provider pick (e.g. the operator
    /// asking Atlas for "this mission on Codex"). `None` keeps the configured
    /// default, so the global `agent_command` stays the fallback rather than
    /// something every caller has to know about.
    pub async fn dispatch_oracle_with_agent(
        &self,
        project: &str,
        mission: &str,
        agent_override: Option<&str>,
        force_new: bool,
    ) -> Result<DispatchOutcome> {
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
        let live: Vec<String> = live_names.iter().map(|s| s.name.clone()).collect();
        let state_dir = self.config.state_dir.clone();

        // ── FOLLOWUP ROUTING — decided and RETURNED before any mutation ──────
        // Everything below this block mutates the mission of whatever name it
        // resolves: the stale-signal clear, MissionLog::clear +
        // clear_gate_attempt, the ledger Mission, and OracleState::new(). Run any
        // of them against a LIVE oracle and you corrupt the mission it is
        // currently running (wiped worker roster, erased R-LOOP timeline, reset
        // thrash counters, possibly a destroyed undelivered report). So the
        // followup path is a straight-line probe → deliver → return, and it
        // touches none of them.
        //
        // OPT-IN: with OMEGA_FOLLOWUP_ROUTING unset the followup branch is
        // skipped exactly like `--new` does, which is the pre-merge behavior.
        let followup_allowed = followup_routing_enabled();
        let route = route_now(&state_dir, project, &live, force_new || !followup_allowed);

        let mut pane_not_ready = false;
        let preferred: Option<String> = match route {
            DispatchRoute::Followup { oracle } => {
                if self.deliver_followup(&oracle, project, mission).await? {
                    return Ok(DispatchOutcome {
                        oracle_name: oracle,
                        delivery: DispatchDelivery::Followup,
                    });
                }
                // The followup could not be delivered (pane never typeable,
                // target stopped qualifying, send failed, or acceptance was
                // never confirmed). Fall through to a normal spawn — a followup
                // that lands in a shell, or is lost, is worse than the sibling
                // oracle we are trying to avoid.
                pane_not_ready = true;
                // …but do NOT throw away the idle-recycling candidate on the
                // way out. Asking the SAME router with `force_new` skips only
                // the followup branch and returns the `preferred` name the
                // pre-existing idle-reuse rules would have picked, which is
                // what the commit message claimed was left unchanged and what
                // hardcoding `None` here quietly broke. The session list is
                // re-read because the probe may have slept for eight seconds.
                let live_now: Vec<String> = self
                    .session_mgr
                    .list_sessions()
                    .await
                    .unwrap_or_default()
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();
                match route_now(&state_dir, project, &live_now, true) {
                    DispatchRoute::Spawn { preferred } => preferred,
                    // Unreachable: `force_new` skips the followup branch.
                    DispatchRoute::Followup { .. } => None,
                }
            }
            DispatchRoute::Spawn { preferred } => preferred,
        };

        let oracle_name =
            OracleRegistry::reserve_oracle(&self.config.state_dir, project, preferred.as_deref())?;

        // Clear any STALE done signal from a PRIOR mission under this name —
        // the oracle mirror of the worker-side clear (c1f0858). Oracle names
        // recycle (the registry entry of an auto-closed oracle is Dead-purged,
        // so next_oracle_name re-issues the base name) and nothing else ever
        // deletes oracle-<key>.done.json: a leftover closeable signal would
        // make patrol's reap kill the brand-new oracle within one tick, and a
        // leftover .notified marker would silently suppress its real report.
        if crate::done::OracleDoneSignal::clear(&self.config.state_dir, &oracle_name) {
            tracing::warn!(
                oracle = %oracle_name,
                "cleared stale done signal from a prior mission before dispatch"
            );
        }
        // A recycled name must start with a fresh loop timeline (R-LOOP): drop
        // the prior mission's log, escalation record, and bounded-retry markers
        // so `omega log` never mixes two missions and a stale escalation never
        // haunts the new one.
        crate::loop_guard::MissionLog::clear(&self.config.state_dir, &oracle_name);
        crate::loop_guard::clear_gate_attempt(&self.config.state_dir, &oracle_name);
        crate::loop_guard::MissionLog::event(
            &self.config.state_dir,
            &oracle_name,
            "dispatch",
            &format!(
                "mission dispatched: {}",
                mission.chars().take(140).collect::<String>()
            ),
        );

        // Classification + ship/god-mode detection run on the RAW message —
        // keyword signals ("ship", "god mode") must not be lost to
        // restructuring.
        let decision = routing::classify_mission(mission);
        let mission_record =
            crate::mission::Mission::new(project, mission, work_path.clone());
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        ledger.create_mission(
            &mission_record,
            &format!("dispatch:{}:create", mission_record.id.as_str()),
            "omega-dispatch",
        )?;
        let mut classified = crate::mission_ledger::AppendEvent::new(
            mission_record.id.clone(),
            1,
            format!("dispatch:{}:classified", mission_record.id.as_str()),
            "omega-dispatch",
            "mission_classified",
        );
        classified.next_mission_state = Some(crate::mission::MissionState::Classified);
        classified.payload = serde_json::to_value(&decision)?;
        ledger.append(classified)?;

        // The legacy OracleState is now a projection carrying the same stable
        // mission identity. It remains for existing readers during migration,
        // but is never allowed to invent an empty mission for this path.
        let oracle_state = OracleState::new(&oracle_name, &mission_record);
        oracle_state.write(&self.config.state_dir)?;

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
        prompt.push_str(&format!(
            "\n## Mission Identity\nmission_id: `{}`\nrouter_version: `{}`\n\
             This identity is stable. Session names and JSON files are projections only.\n",
            mission_record.id.as_str(),
            decision.router_version
        ));

        // GIT SYNC PREFLIGHT (pull-before-work doctrine, runtime-enforced):
        // every mission starts from the CURRENT origin state — the dispatcher
        // fetches + ff-only-pulls the project dir (clean tree only; dirty or
        // diverged is surfaced, never touched) and tells the oracle the
        // outcome so it never assumes its checkout is fresh. (blocking git
        // subprocesses → spawn_blocking, same pattern as amplify above)
        let git_sync = {
            let wp = work_path.clone();
            tokio::task::spawn_blocking(move || crate::git_sync::pull_preflight(&wp))
                .await
                .unwrap_or(crate::git_sync::GitSyncOutcome::FetchFailed)
        };
        tracing::info!(project = %project, outcome = %git_sync.describe(), "dispatch git-sync preflight");
        prompt.push_str(&format!(
            "\n## Git Sync (runtime preflight)\n{}{}\nRe-run `git fetch origin && git pull --ff-only` (clean tree only) before EVERY merge, ship, or deploy phase — other sessions push while you work.\n",
            git_sync.describe(),
            git_sync.warning().map(|w| format!("\n{w}")).unwrap_or_default()
        ));

        // THE FUNNEL — every dispatched agent (any LLM backend) MUST receive
        // its role-scoped Laws + operational rules via this single call.
        // This closes the gap where CLI/RPC-dispatched oracles previously
        // launched without their inviolable Laws.
        // Narrowed to THIS mission (rules::agent_context_block_for_mission):
        // universal rules + Laws in full, domain rules indexed unless the
        // mission mentions their topic. Nothing is hidden, only un-inlined.
        let ctx = crate::rules::agent_context_block_for_mission(
            crate::rules::RuleScope::Oracle,
            &prompt,
        );
        if !ctx.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&ctx);
        }

        // Claude-only smart spawn (2026-w20 features): /goal + --effort +
        // budget caps. Gemini/GLM/Pi/Hermes fall back to the bare launcher
        // with the same prompt; Codex gets its own parity lane below.
        //
        // The per-mission override wins over the configured default; an
        // unknown name is a caller error, so fail loud rather than silently
        // dispatching the mission onto the wrong provider.
        let agent = match agent_override {
            Some(name) => crate::agents::Agent::from_name(name).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown agent '{}' — expected one of: claude, codex, gemini, pi, hermes, glm, shell",
                    name
                )
            })?,
            None => crate::agents::Agent::from_name(&self.config.agent_command)
                .unwrap_or(crate::agents::Agent::Codex),
        };
        let mut required = vec![
            crate::providers::ProviderCapability::Reasoning,
            crate::providers::ProviderCapability::ToolCalling,
        ];
        if !mission.to_lowercase().contains("read-only")
            && !mission.to_lowercase().contains("lecture seule")
        {
            required.push(crate::providers::ProviderCapability::CodeEditing);
        }
        if matches!(
            decision.topology,
            routing::RoutingTopology::ManagerTools
                | routing::RoutingTopology::ParallelWorkers
                | routing::RoutingTopology::Council
        ) {
            required.push(crate::providers::ProviderCapability::Delegation);
        }
        crate::providers::ProvidersConfig::negotiate_provider(
            Some(agent.name()),
            &required,
            &[crate::providers::ProviderCapability::LongContext],
        )
        .map_err(|error| anyhow::anyhow!("provider capability negotiation failed: {error}"))?;
        if matches!(agent, crate::agents::Agent::Claude) {
            let mut opts = crate::agents::LaunchOptions::default();
            // Ultracode posture: the oracle is the strategic brain — it reasons
            // hard on every mission. Floor raised to high; Complex/Epic go xhigh/max.
            // (Model is Opus 5 via the default config; effort is the reasoning depth.)
            opts.effort = Some(match decision.complexity {
                routing::Complexity::Simple => "high".to_string(),
                routing::Complexity::Medium => "xhigh".to_string(),
                routing::Complexity::Complex => "xhigh".to_string(),
                routing::Complexity::Epic => "max".to_string(),
            });
            // Pin the model explicitly so the spawned oracle never silently
            // drifts onto the CLI's default. "opus" → claude-opus-5[1m].
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
            // Non-Claude oracles (Codex/GLM/Gemini/Pi/Hermes).
            //
            // They still get the FULL prompt — mission + git-sync preflight +
            // the role-scoped Laws/Rules funnel above — because the doctrine is
            // plain text, not a Claude flag. What they do NOT get is /goal:
            // it is a Claude Code slash command with no equivalent elsewhere,
            // so the mission runs one-shot and is verified afterwards rather
            // than self-looping.
            //
            // Model and reasoning effort are deliberately NOT injected here.
            // Codex reads its own ~/.codex/config.toml (the operator's SSOT,
            // e.g. gpt-5.6-sol at `ultra`); forcing providers.toml's value on
            // top of it would silently DOWNGRADE the oracle rather than pin it.
            //
            // Use the RESOLVED agent, not config.agent_command: with a
            // per-mission --agent override those two differ, and reading the
            // config here would silently dispatch onto the wrong provider.
            self.session_mgr
                .create_agent_session(&oracle_name, &work_dir, agent.name(), Some(&prompt))
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
        Ok(DispatchOutcome {
            oracle_name,
            delivery: if pane_not_ready {
                DispatchDelivery::SpawnedPaneNotReady
            } else {
                DispatchDelivery::Spawned
            },
        })
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

        // A FINISHED oracle (closeable done signal) must not be resurrected —
        // the mission is over and its record may still be awaiting the
        // notifier. Same guard as patrol's auto-resurrect path.
        if let Ok(Some(done)) =
            crate::done::OracleDoneSignal::read(&self.config.state_dir, oracle_name)
        {
            if done.is_closeable() {
                return Ok(ResurrectOutcome::Finished);
            }
        }

        // Clear any STALE done signal left by the dead incarnation (same
        // rationale as the dispatch-time clear above): a closeable signal with
        // an old finished_at would make patrol's reap murder the resurrected
        // session within 60-120s, and the name would stay bricked on every
        // retry. The resurrected oracle writes its OWN fresh signal at the end.
        if crate::done::OracleDoneSignal::clear(&self.config.state_dir, oracle_name) {
            tracing::warn!(
                oracle = %oracle_name,
                "cleared stale done signal from the prior incarnation before resurrect"
            );
        }
        // Re-register as Active with a fresh spawned_at — the dead entry was
        // purged by registry cleanup, and patrol's freshness guard needs a
        // spawn time to date this session's future done signal against.
        let _ = OracleRegistry::register_resurrected(
            &self.config.state_dir,
            oracle_name,
            &state.project,
        );

        let mut prompt = build_resume_prompt(&state);
        // THE FUNNEL — a resurrected oracle gets its Oracle-scoped doctrine too.
        // Narrowed to THIS mission (rules::agent_context_block_for_mission):
        // universal rules + Laws in full, domain rules indexed unless the
        // mission mentions their topic. Nothing is hidden, only un-inlined.
        let ctx = crate::rules::agent_context_block_for_mission(
            crate::rules::RuleScope::Oracle,
            &prompt,
        );
        if !ctx.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&ctx);
        }

        let work_dir = state.working_dir.to_string_lossy().to_string();
        let agent = crate::agents::Agent::from_name(&self.config.agent_command)
            .unwrap_or(crate::agents::Agent::Codex);
        crate::providers::ProvidersConfig::negotiate_provider(
            Some(agent.name()),
            &[
                crate::providers::ProviderCapability::Reasoning,
                crate::providers::ProviderCapability::CodeEditing,
                crate::providers::ProviderCapability::ToolCalling,
                crate::providers::ProviderCapability::Delegation,
            ],
            &[],
        )
        .map_err(|error| anyhow::anyhow!("provider capability negotiation failed: {error}"))?;
        if matches!(agent, crate::agents::Agent::Claude) {
            let mut opts = crate::agents::LaunchOptions::default();
            opts.effort = Some("xhigh".to_string());
            opts.model = Some(resolve_model_flag(&self.config.default_model));
            opts.session_name = Some(oracle_name.to_string());
            // Resurrect path: same interactive oracle posture as a fresh
            // dispatch. NOTE: this is a FRESH conversation, not a lineage fork —
            // resolve_session_id always mints a new UUID (see its doc: reusing a
            // persisted id collides and the pane never launches Claude), and
            // `--fork-session` only forks when RESUMING an existing session, so
            // passing it alongside a fresh --session-id was a silent no-op. The
            // crashed oracle's context is rebuilt from the mission brief +
            // on-disk state instead.
            // A resurrected oracle is AUTONOMOUS exactly like a fresh dispatch
            // (None → --dangerously-skip-permissions): never gate on the operator.
            // ("auto" used to prompt on risky ops — the exact friction the operator
            // rejects: every OmegaOS session must run fully bypass-permissions.)
            opts.permission_mode = None;
            opts.exclude_dynamic_prompt_sections = true;
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
    /// The oracle already finished cleanly (closeable done signal) — nothing
    /// to resume. Mirrors patrol's auto-resurrect guard; without it a no-arg
    /// `omega resurrect` swept every finished oracle (OracleState is never
    /// deleted), wiped its done record via the stale-signal clear, and
    /// pointlessly re-ran completed missions.
    Finished,
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
mod followup_routing_tests {
    use super::*;
    use crate::oracle_lifecycle::{OracleRegistryEntry, OracleRegistryStatus};
    use chrono::{Duration as ChronoDuration, Utc};

    fn entry(name: &str, project: &str, status: OracleRegistryStatus, age_secs: i64) -> OracleRegistryEntry {
        OracleRegistryEntry {
            oracle_name: name.to_string(),
            project: project.to_string(),
            session_name: name.to_string(),
            status,
            spawned_at: Utc::now() - ChronoDuration::seconds(age_secs),
            files_owned: Vec::new(),
        }
    }

    /// No done signal anywhere.
    fn no_signal(_: &str) -> bool {
        false
    }

    /// THE CORE FIX. An oracle that is alive in rmux with a mission still
    /// running receives the followup, and NO new name is allocated.
    ///
    /// The status field is deliberately `Idle` here: during the real incident
    /// the actively-working `oracle-dentistrygpt-2` was registered `idle`, so a
    /// routing decision that trusted `status` would have spawned a sibling. This
    /// test fails if anyone reintroduces that dependency.
    #[test]
    fn live_working_oracle_receives_the_followup() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Idle,
            30,
        )];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-dentistrygpt".to_string()],
            |_| Some(false), // mission still running
            no_signal,
            false,
        );
        assert_eq!(
            route,
            DispatchRoute::Followup {
                oracle: "oracle-dentistrygpt".to_string()
            },
            "a live oracle with an unfinished mission must absorb the followup, not get a sibling"
        );
    }

    /// `--new` is the escape hatch: the same live working oracle is bypassed.
    #[test]
    fn force_new_respawns_even_while_an_oracle_works() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Active,
            30,
        )];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-dentistrygpt".to_string()],
            |_| Some(false),
            no_signal,
            true, // --new
        );
        assert_eq!(
            route,
            DispatchRoute::Spawn { preferred: None },
            "--new must spawn a fresh oracle even when one is working"
        );
    }

    /// FM3: a done signal on disk means patrol reaps the session within
    /// ORACLE_CLOSE_GRACE_SECS (120s, patrol.rs:29). A followup delivered there
    /// dies with the session, so a signalled oracle is never a target.
    #[test]
    fn oracle_with_a_done_signal_is_not_a_followup_target() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Active,
            30,
        )];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-dentistrygpt".to_string()],
            |_| Some(false),
            |_| true, // done signal present
            false,
        );
        assert!(
            matches!(route, DispatchRoute::Spawn { .. }),
            "an oracle queued for reaping must not receive a followup"
        );
    }

    /// A registry entry is not proof of life: with no live rmux session the old
    /// behavior (spawn) is preserved.
    #[test]
    fn dead_oracle_still_spawns() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Active,
            30,
        )];
        let route = route_dispatch(&entries, "dentistrygpt", &[], |_| Some(false), no_signal, false);
        assert_eq!(route, DispatchRoute::Spawn { preferred: None });
    }

    /// The pre-existing idle-reuse path is untouched: an Idle + live + closeable
    /// oracle still has its NAME recycled.
    #[test]
    fn idle_closeable_oracle_name_is_still_recycled() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Idle,
            30,
        )];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-dentistrygpt".to_string()],
            |_| Some(true), // mission finished
            no_signal,
            false,
        );
        assert_eq!(
            route,
            DispatchRoute::Spawn {
                preferred: Some("oracle-dentistrygpt".to_string())
            },
            "idle-name recycling must survive this change"
        );
    }

    /// An unreadable state proves nothing, so it is not a followup target.
    #[test]
    fn unreadable_state_falls_back_to_spawn() {
        let entries = vec![entry(
            "oracle-dentistrygpt",
            "dentistrygpt",
            OracleRegistryStatus::Active,
            30,
        )];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-dentistrygpt".to_string()],
            |_| None,
            no_signal,
            false,
        );
        assert_eq!(route, DispatchRoute::Spawn { preferred: None });
    }

    /// With several live oracles on one project (the incident had four), the
    /// freshest conversation wins.
    #[test]
    fn freshest_live_oracle_wins() {
        let entries = vec![
            entry("oracle-dentistrygpt", "dentistrygpt", OracleRegistryStatus::Active, 600),
            entry("oracle-dentistrygpt-4", "dentistrygpt", OracleRegistryStatus::Active, 10),
            entry("oracle-dentistrygpt-2", "dentistrygpt", OracleRegistryStatus::Active, 300),
        ];
        let live = vec![
            "oracle-dentistrygpt".to_string(),
            "oracle-dentistrygpt-2".to_string(),
            "oracle-dentistrygpt-4".to_string(),
        ];
        let route = route_dispatch(&entries, "dentistrygpt", &live, |_| Some(false), no_signal, false);
        assert_eq!(
            route,
            DispatchRoute::Followup {
                oracle: "oracle-dentistrygpt-4".to_string()
            }
        );
    }

    /// Another project's live oracle must never capture this project's mission.
    #[test]
    fn other_projects_oracle_is_never_a_target() {
        let entries = vec![entry("oracle-Verba", "Verba", OracleRegistryStatus::Active, 30)];
        let route = route_dispatch(
            &entries,
            "dentistrygpt",
            &["oracle-Verba".to_string()],
            |_| Some(false),
            no_signal,
            false,
        );
        assert_eq!(route, DispatchRoute::Spawn { preferred: None });
    }

    /// THE SHIPPED DEFAULT. Followup routing is opt-in: an absent (or empty, or
    /// `0`) `OMEGA_FOLLOWUP_ROUTING` leaves the dispatcher on its pre-merge
    /// behavior — every mission spawns — because the delivery probe is not yet
    /// proven against a live shell.
    #[test]
    fn followup_routing_is_off_unless_explicitly_enabled() {
        assert!(!followup_routing_enabled_from(None), "absent means OFF");
        for off in ["", " ", "0", "false", "no", "off", "maybe"] {
            assert!(
                !followup_routing_enabled_from(Some(off)),
                "{off:?} must not enable followup routing"
            );
        }
        for on in ["1", "true", "yes", "on", " 1 "] {
            assert!(
                followup_routing_enabled_from(Some(on)),
                "{on:?} must enable followup routing"
            );
        }
    }

    // ── Pane readiness ──────────────────────────────────────────────────────
    //
    // The positive cases are REAL rmux captures, not hand-typed strings: the
    // hand-typed ones are what let the three false positives below ship, since
    // an author who writes both the pane and the predicate writes them to
    // agree. `tests/fixtures/` already held eight captures.

    /// Two real composers, captured from live agent sessions.
    const REAL_COMPOSER_STOPPED: &str =
        include_str!("../tests/fixtures/GOLDEN-stalled-real.txt");
    const REAL_COMPOSER_WORKING: &str =
        include_str!("../tests/fixtures/MoonBaseCapital-claude.txt");
    /// A real capture whose transcript ECHOES the question hint while the
    /// composer is drawn under it — the pane that must stay a valid target.
    const REAL_SELF_ECHO: &str =
        include_str!("../tests/fixtures/GOLDEN-self-echo-false-question.txt");
    /// Real question modals, including the three adversarial variants.
    const REAL_QUESTION: &str = include_str!("../tests/fixtures/GOLDEN-question-real.txt");
    const REAL_QUESTION_OPTION_UNDER_RULE: &str =
        include_str!("../tests/fixtures/adv-question-option-under-rule.txt");
    const REAL_QUESTION_RULE_ARROW: &str =
        include_str!("../tests/fixtures/adv-question-with-rule-arrow.txt");
    const REAL_QUESTION_RULE_BLOCKQUOTE: &str =
        include_str!("../tests/fixtures/adv-question-with-rule-blockquote.txt");
    const REAL_QUESTION_STALE_INTERRUPT: &str =
        include_str!("../tests/fixtures/adv-question-stale-interrupt.txt");

    /// The nominal path, on real captures: an empty composer with the agent's
    /// status bar under it is the ONE thing this feature waits for.
    #[test]
    fn real_agent_composers_are_ready() {
        for (name, pane) in [
            ("GOLDEN-stalled-real", REAL_COMPOSER_STOPPED),
            ("MoonBaseCapital-claude", REAL_COMPOSER_WORKING),
            ("GOLDEN-self-echo-false-question", REAL_SELF_ECHO),
        ] {
            assert!(
                pane_ready_for_followup(pane),
                "{name}: a real, empty agent composer must be a valid followup target"
            );
        }
    }

    /// Every real question capture, including the three adversarial variants
    /// that put the option list directly under a rule.
    #[test]
    fn real_question_modals_are_never_ready() {
        for (name, pane) in [
            ("GOLDEN-question-real", REAL_QUESTION),
            ("adv-question-option-under-rule", REAL_QUESTION_OPTION_UNDER_RULE),
            ("adv-question-with-rule-arrow", REAL_QUESTION_RULE_ARROW),
            ("adv-question-with-rule-blockquote", REAL_QUESTION_RULE_BLOCKQUOTE),
            ("adv-question-stale-interrupt", REAL_QUESTION_STALE_INTERRUPT),
        ] {
            assert!(
                !pane_ready_for_followup(pane),
                "{name}: a followup must never auto-answer a pending question"
            );
        }
    }

    /// FALSE POSITIVE 1, REPRODUCED IN RUNTIME (audit P3). Every oracle runs as
    /// `bash -c '<agent> …; exec bash'` (agents.rs:452), so a dead agent leaves
    /// the session ALIVE showing its last frame — composer and status bar
    /// included — with a shell prompt under it. The registry still calls it
    /// live, its state still says `analyze`, and it has no done signal: it is a
    /// perfect followup target by every OTHER condition.
    ///
    /// This capture is the one the audit replayed through the real function and
    /// got `ready=true` from. Each mission line would have run as a command,
    /// `$()` and redirections live, in the project directory.
    ///
    /// NOTE the shape: this is NOT the one-line pane the previous version of
    /// this test used. That pane had a single line, so `find_input_box`'s
    /// `for i in (1..lines.len()).rev()` never iterated and the test passed
    /// before reaching any composer logic at all.
    #[test]
    fn a_live_shell_under_a_dead_agents_last_frame_is_not_ready() {
        let pane = "● Analysis complete.\n\
                    \n\
                    ────────────────────────────────────────────────\n\
                    ❯ \n\
                    ────────────────────────────────────────────────\n\
                      ⏵⏵ bypass permissions on (shift+tab to cycle)\n\
                    vibe@Agentik-os:~/Station/SideBusiness/OmegaOS$ \n";
        assert!(
            !pane_ready_for_followup(pane),
            "a live shell under a dead agent's frame must never be typed into"
        );
    }

    /// Same cause, no dead agent needed: any command that printed a rule
    /// followed by a `>`-quoted line leaves the shell wearing the composer's
    /// shape.
    #[test]
    fn a_shell_after_a_rule_and_a_quoted_line_is_not_ready() {
        let pane = "$ omega report --tail\n\
                    ────────────────────────────────────────\n\
                    > the reviewer said the gate is green\n\
                    vibe@station:~$ \n";
        assert!(!pane_ready_for_followup(pane));
    }

    /// FALSE POSITIVE 2a, REPRODUCED (audit P2). A pane created at 200x50 is
    /// resized to whatever client attaches, and OmegaOS spawns teams into split
    /// panes: in a narrow one the modal's footer HARD-WRAPS, both hint markers
    /// stop sharing a line, and the only anti-modal guard falls silent. The
    /// selection list is still there and Enter still picks the highlighted
    /// option.
    #[test]
    fn a_hard_wrapped_question_hint_is_still_not_ready() {
        let pane = "Which approach do you want?\n\
                    ────────────────────────────────\n\
                    ❯ 1. Option A\n\
                      2. Option B\n\
                    Enter to select · ↑/↓\n\
                    to navigate · Esc to cancel\n";
        assert!(
            !pane_ready_for_followup(pane),
            "a wrapped hint must not turn a live modal into a paste target"
        );
    }

    /// FALSE POSITIVE 2b, REPRODUCED (audit P2). A tool-permission dialog draws
    /// the rule + numbered selection shape with a footer that is not the
    /// question hint at all, so the blacklist never sees it.
    #[test]
    fn a_numbered_permission_dialog_is_not_ready() {
        let pane = "Bash command\n\
                    rm -rf ./build\n\
                    ────────────────────────────────\n\
                    ❯ 1. Yes\n\
                      2. Yes, and don't ask again\n\
                      3. No, tell Claude what to do differently\n";
        assert!(
            !pane_ready_for_followup(pane),
            "Enter on a permission dialog approves it — never a followup target"
        );
    }

    /// FALSE POSITIVE 3, OBSERVED ON LIVE SESSIONS (audit P5). Two sessions on
    /// the audited machine were holding an unsubmitted draft; one read `❯ ferme
    /// la session OmegaOS`. The paste does not clear the composer (it must not,
    /// the text is the operator's), so it concatenates and submits both as one
    /// turn — the mission would have opened with "close the session".
    #[test]
    fn a_composer_holding_an_operator_draft_is_not_ready() {
        let pane = "● Ready.\n\
                    ────────────────────────────────────────────────\n\
                    ❯ ferme la session OmegaOS\n\
                    ────────────────────────────────────────────────\n\
                      ⏵⏵ bypass permissions on (shift+tab to cycle)\n";
        assert!(
            !pane_ready_for_followup(pane),
            "never append a mission to the operator's unsent draft"
        );
    }

    /// The composer must show the agent's OWN chrome. A rule-plus-marker pair
    /// with nothing under it proves nothing about what is reading the keyboard.
    #[test]
    fn a_composer_without_a_status_bar_is_not_ready() {
        let pane = "some output\n\
                    ────────────────────────────────\n\
                    ❯ \n";
        assert!(!pane_ready_for_followup(pane));
    }

    /// A composer scrolled far above the live tail is scrollback, not a place
    /// to type — the bound `question_ui_visible` already applies to its hint.
    #[test]
    fn a_composer_left_far_up_the_scrollback_is_not_ready() {
        let mut pane = String::from(
            "────────────────────────────────\n\
             ❯ \n\
             ────────────────────────────────\n\
               ⏵⏵ bypass permissions on\n",
        );
        for i in 0..(crate::session_monitor::LIVE_TAIL_LINES + 2) {
            pane.push_str(&format!("output line {i}\n"));
        }
        assert!(!pane_ready_for_followup(&pane));
    }

    /// A starship-style prompt starts with the same glyph as the agent composer
    /// but draws no rule above it.
    #[test]
    fn bare_prompt_glyph_without_a_rule_is_not_ready() {
        let pane = "some output\n❯ ";
        assert!(!pane_ready_for_followup(pane));
    }

    /// The agent composer: a horizontal rule, an EMPTY prompt marker under it,
    /// and the agent's status bar under that.
    #[test]
    fn agent_composer_is_ready() {
        let pane = "I finished the analysis.\n\
                    ────────────────────────────────\n\
                    ❯ \n\
                      ? for shortcuts";
        assert!(
            pane_ready_for_followup(pane),
            "the agent's composer is exactly what we wait for"
        );
    }

    /// A pending question draws the same rule+marker shape as a composer, but
    /// typing there ANSWERS it. Refuse, and let the caller spawn instead.
    #[test]
    fn visible_question_modal_is_not_ready() {
        let pane = "Which approach do you want?\n\
                    ────────────────────────────────\n\
                    ❯ 1. Option A\n\
                      2. Option B\n\
                    Enter to select · ↑/↓ to navigate · Esc to cancel";
        assert!(
            !pane_ready_for_followup(pane),
            "a followup must never auto-answer a pending question"
        );
    }

    /// A pane that has not finished drawing is not ready.
    ///
    /// The previous version of this test passed `""`, whose `.lines()` yields
    /// ZERO elements: no loop ran, no predicate was reached, and no regression
    /// could ever have failed it. The real shapes of "not drawn yet" are a
    /// half-painted boot frame and a session whose screen is still blank, so
    /// those are what is asserted.
    #[test]
    fn a_pane_that_is_not_drawn_yet_is_not_ready() {
        for (name, pane) in [
            ("empty capture", ""),
            ("blank screen", "\n\n   \n\n"),
            ("rule drawn, composer not yet", "● Booting\n────────────────────────────────\n"),
            (
                "composer drawn, status bar not yet",
                "● Booting\n────────────────────────────────\n❯ \n",
            ),
            ("agent starting up in its shell", "vibe@station:~/OmegaOS$ claude --model opus\n"),
        ] {
            assert!(
                !pane_ready_for_followup(pane),
                "{name}: an undrawn pane must never be typed into"
            );
        }
    }

    // ── Output contract ─────────────────────────────────────────────────────

    /// The Telegram bridge's ACTUAL regex, copied from
    /// telegram-bot/omega-tg-bot.ts:1273.
    const BRIDGE_RE: &str = r"Oracle dispatched:?\s*(oracle-[A-Za-z0-9._-]+)";

    /// REGRESSION GUARD. The bridge recovers the oracle name by matching the
    /// line above against `omega dispatch` stdout; a miss is reported to the
    /// operator as a FAILED dispatch and the progress thread is lost. So the
    /// followup case must keep matching, and must yield the SAME name.
    #[test]
    fn followup_output_still_matches_the_telegram_bridge_regex() {
        let re = regex::Regex::new(BRIDGE_RE).unwrap();
        for delivery in [
            DispatchDelivery::Spawned,
            DispatchDelivery::Followup,
            DispatchDelivery::SpawnedPaneNotReady,
        ] {
            let outcome = DispatchOutcome {
                oracle_name: "oracle-dentistrygpt-2".to_string(),
                delivery,
            };
            let stdout = outcome.report_lines().join("\n");
            let captured = re
                .captures(&stdout)
                .unwrap_or_else(|| panic!("bridge regex failed to match for {:?}:\n{}", delivery, stdout));
            assert_eq!(
                captured.get(1).unwrap().as_str(),
                "oracle-dentistrygpt-2",
                "the bridge must recover the right oracle name for {:?}",
                delivery
            );
        }
    }

    /// The followup and fallback cases must be VISIBLY different from a plain
    /// spawn, on their own extra line.
    #[test]
    fn followup_and_fallback_announce_themselves_on_an_extra_line() {
        let spawned = DispatchOutcome {
            oracle_name: "oracle-X".into(),
            delivery: DispatchDelivery::Spawned,
        };
        let lines = spawned.report_lines();
        assert_eq!(lines.len(), 2, "canonical line + the machine line");
        assert!(lines[0].contains("Oracle dispatched: oracle-X"));

        let followup = DispatchOutcome {
            oracle_name: "oracle-X".into(),
            delivery: DispatchDelivery::Followup,
        };
        let lines = followup.report_lines();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Oracle dispatched: oracle-X"));
        assert!(lines[1].contains("suivi"));
        assert!(lines[1].contains("aucun nouvel oracle"));

        let fallback = DispatchOutcome {
            oracle_name: "oracle-X".into(),
            delivery: DispatchDelivery::SpawnedPaneNotReady,
        };
        let lines = fallback.report_lines();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("pas pret"));
    }

    /// THE MACHINE CONTRACT, consumed by a parallel change in the Telegram
    /// bridge. Exactly one line, exactly this spelling, on EVERY success path —
    /// a consumer that has to parse the French prose line breaks the day
    /// somebody rewords it.
    #[test]
    fn every_success_path_prints_exactly_one_dispatch_delivery_line() {
        for (delivery, tag) in [
            (DispatchDelivery::Spawned, "spawned"),
            (DispatchDelivery::Followup, "followup"),
            (
                DispatchDelivery::SpawnedPaneNotReady,
                "spawned_pane_not_ready",
            ),
        ] {
            let outcome = DispatchOutcome {
                oracle_name: "oracle-dentistrygpt-2".into(),
                delivery,
            };
            let lines = outcome.report_lines();
            let machine: Vec<&String> = lines
                .iter()
                .filter(|l| l.starts_with("DISPATCH_DELIVERY="))
                .collect();
            assert_eq!(
                machine.len(),
                1,
                "exactly one machine line for {delivery:?}, got {lines:?}"
            );
            assert_eq!(machine[0], &format!("DISPATCH_DELIVERY={tag}"));
            // Line 0 stays the bridge's canonical line, ahead of it.
            assert!(lines[0].starts_with("◆ Oracle dispatched: "));
        }
    }

    // ── The production adapters ─────────────────────────────────────────────

    /// F12. Every routing test above injects a hand-written constant closure,
    /// so NONE of them executes the adapters that read the real files —
    /// inverting `oracle_is_closeable` to `!st.is_closeable()` left the whole
    /// suite green while re-delivering the incident the feature exists to fix.
    ///
    /// This test drives `route_now` — the single production wiring — against a
    /// real state directory on disk, so the `OracleState` read, the
    /// `is_closeable()` predicate, the registry load, and the done-signal read
    /// are all executed for real.
    #[test]
    fn the_production_adapters_decide_the_route_on_real_state_files() {
        use crate::oracle_lifecycle::OraclePhase;

        let tmp = std::env::temp_dir().join(format!(
            "omega-dispatch-adapters-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // A live oracle, mid-mission: the real state file, the real registry.
        let mut registry = OracleRegistry::load(&tmp);
        registry.oracles.push(entry(
            "oracle-adapters",
            "adapters",
            OracleRegistryStatus::Idle,
            30,
        ));
        registry.save(&tmp).unwrap();
        let mission = crate::mission::Mission::new("adapters", "do the thing", tmp.clone());
        let mut state = OracleState::new("oracle-adapters", &mission);
        state.phase = OraclePhase::Analyze;
        state.write(&tmp).unwrap();

        let live = vec!["oracle-adapters".to_string()];
        assert_eq!(
            oracle_is_closeable(&tmp, "oracle-adapters"),
            Some(false),
            "a mission in Analyze is not closeable — this is the value the route depends on"
        );
        assert_eq!(
            route_now(&tmp, "adapters", &live, false),
            DispatchRoute::Followup {
                oracle: "oracle-adapters".to_string()
            },
            "the REAL adapters must route a live mid-mission oracle to a followup"
        );

        // The same oracle once it has signalled done: patrol will reap it, so
        // the done-signal adapter must veto the followup.
        crate::done::OracleDoneSignal::new(
            "oracle-adapters",
            "adapters",
            crate::done::DoneStatus::DoneClean,
            "do the thing",
        )
        .write(&tmp)
        .unwrap();
        assert!(
            oracle_has_done_signal(&tmp, "oracle-adapters"),
            "the signal we just wrote must be readable by the production adapter"
        );
        assert!(
            matches!(
                route_now(&tmp, "adapters", &live, false),
                DispatchRoute::Spawn { .. }
            ),
            "an oracle queued for reaping must not receive a followup"
        );

        // An unreadable state proves nothing: spawn.
        let _ = std::fs::remove_dir_all(&tmp);
    }
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
            attempt_id: None,
            plan_revision: None,
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
