//! Executor — the autonomous Driver. Selects work ONLY via
//! PlanTracker::ready_steps (the LLM is never asked "what next?"), spawns one
//! worker per step through a WorkerRuntime, and gates every completion through
//! the Guardian before marking a step Done.

use crate::agents::Agent;
use crate::done::{DoneSignal, DoneStatus};
use crate::guardian::{Guardian, Verdict};
use crate::planner::{PlanStep, PlanTracker, StepStatus};
use crate::scope;
use crate::session::SessionManager;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Pluggable worker backend — real rmux sessions in prod, scripted in tests.
#[allow(async_fn_in_trait)]
pub trait WorkerRuntime {
    /// Spawn a worker for `step`; return its session name.
    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String>;
    /// Block until the worker's done.json appears (or timeout).
    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal>;
    /// Reconcile a worker whose wait_done failed (timeout / bad done.json):
    /// kill its session and release its file-scope claim so the next run starts
    /// clean. Default no-op (scripted test runtimes have nothing to clean).
    async fn cleanup_failed(&self, _session: &str) -> Result<()> {
        Ok(())
    }
    /// Release a worker's file-scope claim after a TERMINAL verdict without
    /// killing its session. A Pass-verdict worker has finished its job but its
    /// claim must still be freed, or the next run that touches the same files is
    /// rejected (claim_or_reject) and that step can never start. cleanup_failed
    /// kills+releases (right for Retry/Fail); this releases only (right for a
    /// successful worker we leave for inspection). Default no-op.
    async fn release_scope(&self, _session: &str) -> Result<()> {
        Ok(())
    }
    /// Crash-resume adoption probe: if this step's worker from a PREVIOUS run
    /// is still observable (its detached session is alive, or its done.json
    /// already landed), return the session name so run() WAITS on it instead
    /// of resetting the step — re-dispatch would kill live work mid-edit or
    /// discard a finished-but-unprocessed result (spawn clears both). Default
    /// None: scripted test runtimes own no detached sessions.
    async fn adoptable_session(&self, _step: &PlanStep) -> Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub parallelism: usize,
    pub max_attempts: u8,
    pub worker_timeout: Duration,
    /// Cap on one Guardian re-run of a verify_command. Without it a verify
    /// that never exits (serve-without-exit, watch mode) froze the whole run.
    pub verify_timeout: Duration,
    /// Explicitly authorize plan steps that mutate no files. Empty
    /// `files_to_touch` is otherwise rejected because it would silently skip
    /// R-SCOPE. This flag is for genuinely read-only/deploy-only steps whose
    /// worker brief and verification command have no writable project surface.
    pub allow_read_only_steps: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            parallelism: 3,
            max_attempts: 2,
            worker_timeout: Duration::from_secs(60 * 30),
            verify_timeout: Duration::from_secs(60 * 10),
            allow_read_only_steps: false,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunReport {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
    pub blocked: Vec<String>,
    pub success: bool,
}

/// Render the worker prompt for one step from its typed fields.
pub fn render_brief(step: &PlanStep) -> String {
    let mut brief = format!(
        "{title}\n\n{description}\n\nFiles you own (touch ONLY these): {files}\n\nDone criteria: {criteria}\n\nVerify before done.json: {verify}",
        title = step.title,
        description = step.description,
        files = step.files_to_touch.join(", "),
        criteria = step.done_criteria,
        verify = step.verify_command,
    );
    // A retry must not be blind: without the prior attempt's verification
    // evidence in the brief, max_attempts just repeats the same mistake.
    if let Some(feedback) = &step.last_feedback {
        brief.push_str(&format!(
            "\n\nPREVIOUS ATTEMPT FAILED VERIFICATION — diagnose and fix this first:\n{feedback}"
        ));
    }
    brief
}

/// Worker timeout: explicit per-step override > terminal-wave default >
/// RunOptions. Audit/deploy-wave steps run forensic audits / the acceptance
/// browser sweep, which routinely outlive the 30-min default — killing them
/// mid-audit falsely fails the heaviest, most valuable steps, so terminal
/// waves get 4x.
fn step_timeout(step: &PlanStep, opts: &RunOptions) -> Duration {
    step.timeout_mins
        .map(|m| Duration::from_secs(m * 60))
        .unwrap_or_else(|| {
            if step.wave.is_some_and(|w| w.is_terminal()) {
                opts.worker_timeout * 4
            } else {
                opts.worker_timeout
            }
        })
}

/// Drive a plan to completion. Selects work via ready_steps only; gates every
/// completion through the Guardian. Resumable: persists after each transition.
pub async fn run<R: WorkerRuntime>(
    project_dir: &Path,
    runtime: &R,
    opts: RunOptions,
) -> Result<RunReport> {
    // load_strict so a malformed tracker surfaces its parse error instead of
    // masquerading as "no tracker" (the operator must never be told the plan
    // doesn't exist while it sits there corrupt).
    let mut tracker = PlanTracker::load_strict(project_dir)
        .context("loading .planner/tracker.json")?
        .context("no .planner/tracker.json — run `omega plan` first")?;
    // Structural gate BEFORE any worker spawns: cycles, dangling deps,
    // duplicate ids and trivial verify_commands on REMAINING steps are refused
    // here — so a malformed plan can never silently mis-sequence, deadlock, or
    // fake-complete the build. Run-time validation deliberately skips terminal
    // steps and demotes files_to_touch issues to warnings: a mid-flight
    // tracker from an older schema must stay resumable (validate_for_run doc).
    tracker
        .validate_for_run()
        .context("plan failed validation — fix the tracker.json")?;
    if !opts.allow_read_only_steps {
        let unscoped: Vec<&str> = tracker
            .steps
            .iter()
            .filter(|step| {
                matches!(step.status, StepStatus::Pending | StepStatus::InProgress)
                    && step.files_to_touch.is_empty()
            })
            .map(|step| step.step_id.as_str())
            .collect();
        if !unscoped.is_empty() {
            bail!(
                "plan has writable-scope ambiguity on step(s) {}: list exact files, or explicitly set allow_read_only_steps for a genuinely non-writing run",
                unscoped.join(", ")
            );
        }
    }
    // Crash-resume reconciliation: a previous plan-run that died mid-flight
    // persisted its dispatched steps as InProgress (the tracker saves at spawn
    // time, completion only later). ready_steps selects Pending ONLY, so
    // without this pass those steps — and every dependent — stay stuck
    // forever. BUT step workers are DETACHED rmux sessions that survive the
    // engine dying: an InProgress step whose worker is still alive (or whose
    // done.json already landed) is NOT an orphan — re-dispatching it would
    // kill live work mid-edit / discard the unconsumed result (spawn clears
    // both). Those steps are ADOPTED: kept InProgress and fed into the normal
    // wait→verify path on the first loop iteration. Only steps with no
    // observable worker are reset to Pending for re-dispatch (spawn() clears
    // the stale done.json + session, and a same-name scope claim never
    // self-conflicts, so no further cleanup is needed there).
    let mut adopted: Vec<(String, String, Duration)> = Vec::new();
    let orphaned: Vec<String> = tracker
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::InProgress)
        .map(|s| s.step_id.clone())
        .collect();
    if !orphaned.is_empty() {
        tracing::warn!(steps = ?orphaned, "reconciling in_progress steps left by a previous run");
        for sid in &orphaned {
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            if let Some(session) = runtime
                .adoptable_session(&step)
                .await
                .with_context(|| format!("probing worker authority for step {sid}"))?
            {
                tracing::info!(step = %sid, session = %session,
                    "adopting in_progress step (live worker session or existing done signal)");
                let timeout = step_timeout(&step, &opts);
                adopted.push((sid.clone(), session, timeout));
            } else {
                tracker.reset_to_pending(sid)?;
            }
        }
        tracker.save(project_dir)?;
    }
    let guardian = Guardian::new(opts.max_attempts, opts.verify_timeout);
    let mut report = RunReport::default();

    loop {
        let ready: Vec<String> = tracker
            .ready_steps(opts.parallelism)
            .iter()
            .map(|s| s.step_id.clone())
            .collect();

        if ready.is_empty() && adopted.is_empty() {
            let st = tracker.status();
            report.success = st.is_complete();
            if !st.is_complete() {
                // Terminal sweep: classify whatever never reached Done.
                // A step the Guardian failed mid-loop was already recorded in
                // `report.failed`, so guard against double-counting it here.
                for s in &tracker.steps {
                    match s.status {
                        StepStatus::Failed if !report.failed.contains(&s.step_id) => {
                            report.failed.push(s.step_id.clone());
                        }
                        StepStatus::Pending => report.blocked.push(s.step_id.clone()),
                        // Defensive: the start-of-run reconciliation makes a
                        // leftover InProgress unreachable in this loop, but a
                        // stuck step must still be VISIBLE in the report, never
                        // silently dropped from the failure accounting.
                        StepStatus::InProgress => report.blocked.push(s.step_id.clone()),
                        _ => {}
                    }
                }
            }
            return Ok(report);
        }

        // Adopted steps (crash-resume) drain into the first batch: they are
        // already InProgress with a live/finished worker — no spawn needed.
        let mut inflight: Vec<(String, String, Duration)> = std::mem::take(&mut adopted);
        for sid in &ready {
            tracker.start_step(sid)?;
            // Persist the dispatch intent BEFORE the external spawn. If the
            // process dies after rmux creates the worker, the next run must see
            // InProgress and adopt that exact deterministic session instead of
            // treating the step as Pending and retiring live work as "stale".
            tracker
                .save(project_dir)
                .with_context(|| format!("persisting dispatch intent for step {sid}"))?;
            let step = tracker.get_step(sid).context("step vanished")?.clone();
            let timeout = step_timeout(&step, &opts);
            let brief = render_brief(&step);
            let session = match runtime.spawn(&step, &brief, project_dir).await {
                Ok(session) => session,
                Err(spawn_error) => {
                    // RmuxRuntime rolls back any scope generation it acquired
                    // before returning an error. Publish the terminal attempt
                    // state as a separate durable transition so previously
                    // spawned siblings remain InProgress and can be adopted on
                    // resume; never roll the whole batch back to Pending.
                    tracker.mark_failed(sid)?;
                    tracker
                        .save(project_dir)
                        .with_context(|| format!("persisting failed spawn state for step {sid}"))?;
                    return Err(spawn_error).with_context(|| format!("spawning step {sid}"));
                }
            };
            inflight.push((sid.clone(), session, timeout));
        }
        tracker.save(project_dir)?;

        for (sid, session, timeout) in inflight {
            // A wait error may leave the runtime generation alive. Contain it
            // before publishing any terminal tracker state.
            let done = match runtime.wait_done(&session, timeout).await {
                Ok(done) => done,
                Err(e) => {
                    tracing::error!(step = %sid, session = %session, error = %e, "worker wait failed; containing its runtime generation before terminalizing");
                    // If kill/absence confirmation or exact scope release fails,
                    // the worker may still be live and authoritative. Keep the
                    // step InProgress on disk so restart reconciliation retries
                    // this exact generation instead of orphaning it behind Failed.
                    if let Err(cleanup_error) = runtime.cleanup_failed(&session).await {
                        tracker.save(project_dir)?;
                        return Err(cleanup_error).with_context(|| {
                            format!(
                                "containing worker {session} after wait failure ({e:#}); step remains in_progress"
                            )
                        });
                    }
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    tracker.save(project_dir)?;
                    continue;
                }
            };
            let step = tracker.get_step(&sid).context("step vanished")?.clone();
            let attempt = step.attempt + 1;
            // A worker's done.json is an input, never the verdict (R-VERIFY) —
            // but a worker that DECLARES blocked/failed must never reach the
            // Guardian: a verify weaker than the step's actual work (e.g. a
            // build that already passed BEFORE the step) would convert that
            // honest negative into a Pass and mark the step Done. Treat any
            // non-done_clean signal like a failed verification: retry with the
            // worker's stated blocker as feedback while attempts remain.
            if done.status != DoneStatus::DoneClean {
                let label = match done.status {
                    DoneStatus::DoneClean => unreachable!(),
                    DoneStatus::Pending => "pending",
                    DoneStatus::Failed => "failed",
                    DoneStatus::Blocked => "blocked",
                };
                tracing::warn!(step = %sid, status = %label, summary = %done.summary, "worker did not signal done_clean");
                if attempt < opts.max_attempts {
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                    if let Some(s) = tracker.get_step_mut(&sid) {
                        s.last_feedback = Some(format!(
                            "previous worker signalled `{label}`: {}",
                            done.summary
                        ));
                    }
                } else {
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                }
                // Either way the worker is finished with this attempt: kill the
                // session + free the scope claim so a retry can re-claim.
                runtime
                    .cleanup_failed(&session)
                    .await
                    .with_context(|| format!("fail-closed cleanup for worker {session}"))?;
                tracker.save(project_dir)?;
                continue;
            }
            match guardian.verify(&step, project_dir, attempt).await {
                Verdict::Pass => {
                    // Worker announced done + Guardian re-verified → the engine has
                    // advanced past this step, so CLOSE the worker: kill its session
                    // and free the file-scope claim (otherwise finished workers pile
                    // up as idle panes and the lock outlives them, blocking re-runs).
                    // Release BEFORE persisting Done: a failed release leaves the
                    // step resumable rather than publishing completion while its
                    // writable scope remains ambiguously held.
                    runtime
                        .release_scope(&session)
                        .await
                        .with_context(|| format!("releasing accepted worker {session}"))?;
                    tracker.mark_done(&sid)?;
                    report.completed.push(sid.clone());
                }
                Verdict::Retry { feedback } => {
                    tracing::warn!(step = %sid, %feedback, "guardian: retry");
                    tracker.bump_attempt(&sid)?;
                    tracker.reset_to_pending(&sid)?;
                    // Persist the Guardian's evidence ON the step: render_brief
                    // appends it to the retry worker's prompt. Logging alone
                    // delivered nothing to the worker — retries were blind.
                    if let Some(s) = tracker.get_step_mut(&sid) {
                        s.last_feedback = Some(feedback);
                    }
                    // Release the file-scope claim from this attempt so the retry
                    // re-spawn can re-claim it. Without this, claim_or_reject in the
                    // next spawn() rejects (files still locked by the prior attempt)
                    // and the step can never actually retry. spawn() also clears the
                    // stale done.json + kills the old session.
                    runtime
                        .cleanup_failed(&session)
                        .await
                        .with_context(|| format!("cleaning retry worker {session}"))?;
                }
                Verdict::Fail { reason } => {
                    tracing::error!(step = %sid, %reason, "guardian: fail");
                    tracker.mark_failed(&sid)?;
                    report.failed.push(sid.clone());
                    // Terminal failure: kill the worker + free its scope claim
                    // (same cleanup as Retry), else the lock leaks forever.
                    runtime
                        .cleanup_failed(&session)
                        .await
                        .with_context(|| format!("cleaning failed worker {session}"))?;
                }
            }
            tracker.save(project_dir)?;
        }
    }
}

/// Real backend: spawns rmux worker sessions and waits on their done.json —
/// the exact contract the Orchestrator already uses
/// (`create_session_with_agent` + polling `state_dir/worker-{session}.done.json`).
pub struct RmuxRuntime<'a> {
    pub mgr: &'a SessionManager,
    pub state_dir: PathBuf,
    pub project: String,
    pub agent: Agent,
    pub poll: Duration,
}

fn read_done_signal_strict(state_dir: &Path, session: &str) -> Result<Option<DoneSignal>> {
    let signal = DoneSignal::read(state_dir, session)?;
    if let Some(signal) = &signal {
        if signal.session != session {
            bail!(
                "done signal path/session mismatch: expected {}, document names {}",
                session,
                signal.session
            );
        }
        if let Some(projection) = &signal.projection {
            if projection.source != "mission-engine-v3.sqlite3"
                || projection.event_id.trim().is_empty()
                || projection.event_sequence == 0
                || projection.mission_version < projection.event_sequence
                || projection.projection_hash.len() != 64
                || !projection
                    .projection_hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                bail!("done signal {session} carries malformed ledger provenance");
            }
        }
    }
    Ok(signal)
}

impl RmuxRuntime<'_> {
    /// Retire exactly one previous rmux/scope generation. A daemon, kill,
    /// confirmation or exact-release failure aborts before the name is reused.
    async fn retire_session_generation(&self, session: &str) -> Result<()> {
        crate::scope::validate_session_identity(session)?;
        let observed_claim = scope::ScopeClaim::read_strict(&self.state_dir, session)?;
        let live = self
            .mgr
            .list_sessions()
            .await
            .with_context(|| format!("listing sessions before retiring {session}"))?;
        if live.iter().any(|candidate| candidate.name == session) {
            self.mgr
                .kill_session(session)
                .await
                .with_context(|| format!("killing previous session {session}"))?;
            let after = self
                .mgr
                .list_sessions()
                .await
                .with_context(|| format!("confirming retirement of {session}"))?;
            if after.iter().any(|candidate| candidate.name == session) {
                bail!("session {session} remained live after kill acknowledgement");
            }
        }
        if let Some(claim) = observed_claim {
            scope::ScopeClaim::release_exact(&self.state_dir, &claim)
                .with_context(|| format!("releasing exact scope generation for {session}"))?;
        }
        Ok(())
    }
}

impl WorkerRuntime for RmuxRuntime<'_> {
    async fn adoptable_session(&self, step: &PlanStep) -> Result<Option<String>> {
        // Same deterministic name spawn() uses, the only coupling adoption needs.
        let session = format!("{}-step-{}", self.project, step.step_id.to_lowercase());
        if read_done_signal_strict(&self.state_dir, &session)?.is_some() {
            return Ok(Some(session));
        }
        // Do not collapse daemon/capture failure into "session absent". That
        // would reset persisted InProgress state and spawn() could retire a
        // still-live worker. The authoritative list is fallible, so propagate
        // errors and leave this generation untouched for restart recovery.
        let sessions = self
            .mgr
            .list_sessions()
            .await
            .with_context(|| format!("listing sessions while probing adoption of {session}"))?;
        Ok(sessions
            .iter()
            .any(|candidate| candidate.name == session)
            .then_some(session))
    }

    async fn spawn(&self, step: &PlanStep, brief: &str, cwd: &Path) -> Result<String> {
        let session = format!("{}-step-{}", self.project, step.step_id.to_lowercase());

        // Retry-safety: the session name is deterministic per step, so on a
        // guardian-Retry re-dispatch the OLD attempt's worker-<session>.done.json
        // still exists (wait_done would return it instantly and re-verify
        // unchanged state) and the rmux session may still be alive (create-or-reuse
        // would attach to the stale pane and never deliver the new brief). Clear
        // both so a retry actually re-runs the step with the fresh brief:
        let done_path = self.state_dir.join(format!("worker-{session}.done.json"));
        // Inspect before killing so unsafe/corrupt authority cannot be erased
        // as a retry side effect.
        let stale_done = read_done_signal_strict(&self.state_dir, &session)?;
        self.retire_session_generation(&session).await?;
        if stale_done.is_some() {
            crate::scope::remove_private_file(&done_path)
                .with_context(|| format!("removing stale done signal for {session}"))?;
        }

        // Claim file scope upfront — fail fast on conflict (same gate as dispatch_task).
        let scope_claim = if !step.files_to_touch.is_empty() {
            Some(
                scope::claim_or_reject_for_workspace(
                    &self.state_dir,
                    cwd,
                    &session,
                    step.files_to_touch.clone(),
                )
                .with_context(|| format!("scope claim for {session}"))?,
            )
        } else {
            None
        };
        // THE FUNNEL — inject the Worker-scoped Laws + operational rules. This
        // plan-run/executor dispatch path previously spawned workers with NO
        // doctrine; mirror cmd_spawn_worker so every dispatched agent gets it.
        let mut full_brief = brief.to_string();
        // CRITICAL: tell the worker HOW to signal completion. Without this the
        // worker does the work, verifies, writes a "Resume:" line, then idles at the
        // prompt — never emitting worker-<session>.done.json, so the executor's
        // wait_done() blocks forever and the ENTIRE plan stalls at this step. The
        // `omega done <session> …` invocation (with the exact session baked in) is the
        // signal the engine polls for. This is the #1 reason a build "never finishes".
        full_brief.push_str(&format!(
            "\n\n## SESSION IDENTITY\nYou are worker `{session}` — this exact string is your rmux \
             session name, your Claude conversation name (resumable via `claude --resume {session}`), \
             and the key for your state files in ~/.omega/state/. Use it verbatim in every omega call.\n\
             \n## SIGNAL COMPLETION — REQUIRED (the build engine BLOCKS until you do this)\n\
             This is an AUTOMATED build step. As your VERY LAST action, after your Verify \
             command passes, you MUST run exactly:\n  \
             omega done {session} done_clean \"<one-line summary of what you changed>\"\n\
             That writes the done signal the engine waits for. If you genuinely cannot finish, run\n  \
             omega done {session} blocked \"<what blocks you>\"   (or: failed)\n\
             Do NOT stop at the prompt, ask a question, or write only a 'Resume:' line and idle — \
             the whole plan halts on this step until `omega done {session} …` runs. No exceptions.",
            session = session
        ));
        let ctx = crate::rules::agent_context_block_for_mission(
            crate::rules::RuleScope::Worker,
            &full_brief,
        );
        if !ctx.is_empty() {
            full_brief.push_str("\n\n");
            full_brief.push_str(&ctx);
        }
        let cwd = cwd.to_string_lossy();
        // Claude-side session label (`--name`): mirror the rmux session name so the
        // step-worker's conversation is addressable/resumable by the same
        // deterministic identity (non-Claude providers ignore the field).
        let opts = crate::agents::LaunchOptions {
            session_name: Some(session.clone()),
            ..Default::default()
        };
        if let Err(e) = self
            .mgr
            .create_agent_session_with_opts(&session, &cwd, self.agent, Some(&full_brief), opts)
            .await
        {
            if let Some(claim) = &scope_claim {
                if let Err(release_error) = scope::ScopeClaim::release_exact(&self.state_dir, claim)
                {
                    bail!(
                        "spawning {session} failed: {e:#}; exact scope rollback also failed: {release_error:#}"
                    );
                }
            }
            return Err(e).with_context(|| format!("spawning {session}"));
        }
        Ok(session)
    }

    async fn wait_done(&self, session: &str, timeout: Duration) -> Result<DoneSignal> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(signal) = read_done_signal_strict(&self.state_dir, session)? {
                return Ok(signal);
            }
            if Instant::now() >= deadline {
                bail!("worker {session} timed out after {}s", timeout.as_secs());
            }
            tokio::time::sleep(self.poll).await;
        }
    }

    async fn cleanup_failed(&self, session: &str) -> Result<()> {
        // Kill the hung/failed worker session (best-effort) and free its scope
        // claim so a resumed plan run does not see a locked file or a zombie pane.
        self.retire_session_generation(session).await
    }

    async fn release_scope(&self, session: &str) -> Result<()> {
        // A worker whose step is DONE has announced completion (its done.json was
        // accepted and the Guardian re-verified) — so CLOSE it: kill the session
        // and free its scope claim. Leaving Pass workers alive piled up dozens of
        // idle `*-step-*` panes per build; the oracle/engine has already advanced
        // past this step, so the worker has no reason to stay open.
        self.retire_session_generation(session).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::done::DoneStatus;
    use std::collections::HashMap;

    /// Scripts a DoneStatus per step_id; session names are "fake-<step_id>".
    pub struct FakeRuntime {
        pub script: HashMap<String, DoneStatus>,
    }

    impl WorkerRuntime for FakeRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self
                .script
                .get(step_id)
                .cloned()
                .unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
    }

    struct ReleaseFailureRuntime;

    impl WorkerRuntime for ReleaseFailureRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn release_scope(&self, _session: &str) -> Result<()> {
            bail!("synthetic exact-release failure")
        }
    }

    #[derive(Default)]
    struct AdoptionProbeFailureRuntime {
        spawn_calls: Mutex<usize>,
        cleanup_calls: Mutex<usize>,
    }

    impl WorkerRuntime for AdoptionProbeFailureRuntime {
        async fn spawn(&self, _step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            *self.spawn_calls.lock().unwrap() += 1;
            Ok("must-not-spawn".to_string())
        }

        async fn adoptable_session(&self, _step: &PlanStep) -> Result<Option<String>> {
            bail!("synthetic authoritative session-list failure")
        }

        async fn wait_done(&self, _session: &str, _timeout: Duration) -> Result<DoneSignal> {
            bail!("must not wait after an adoption probe failure")
        }

        async fn cleanup_failed(&self, _session: &str) -> Result<()> {
            *self.cleanup_calls.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[derive(Default)]
    struct WaitAndCleanupFailureRuntime {
        cleanup_calls: Mutex<usize>,
    }

    impl WorkerRuntime for WaitAndCleanupFailureRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("live-{}", step.step_id))
        }

        async fn wait_done(&self, _session: &str, _timeout: Duration) -> Result<DoneSignal> {
            bail!("synthetic wait failure with possibly live worker")
        }

        async fn cleanup_failed(&self, _session: &str) -> Result<()> {
            *self.cleanup_calls.lock().unwrap() += 1;
            bail!("synthetic kill/absence confirmation failure")
        }
    }

    #[derive(Default)]
    struct SpawnResumeState {
        spawn_calls: Vec<String>,
        adoption_calls: Vec<String>,
        live_sessions: std::collections::HashSet<String>,
    }

    #[derive(Clone, Default)]
    struct FailSecondSpawnRuntime {
        state: std::sync::Arc<Mutex<SpawnResumeState>>,
    }

    impl WorkerRuntime for FailSecondSpawnRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            let mut state = self.state.lock().unwrap();
            state.spawn_calls.push(step.step_id.clone());
            if step.step_id == "STEP-002" {
                bail!("synthetic second spawn failure");
            }
            let session = format!("resume-{}", step.step_id);
            state.live_sessions.insert(session.clone());
            Ok(session)
        }

        async fn adoptable_session(&self, step: &PlanStep) -> Result<Option<String>> {
            let session = format!("resume-{}", step.step_id);
            let mut state = self.state.lock().unwrap();
            if state.live_sessions.contains(&session) {
                state.adoption_calls.push(step.step_id.clone());
                Ok(Some(session))
            } else {
                Ok(None)
            }
        }

        async fn wait_done(&self, session: &str, _timeout: Duration) -> Result<DoneSignal> {
            if !self.state.lock().unwrap().live_sessions.contains(session) {
                bail!("session {session} was not live for adoption");
            }
            Ok(DoneSignal::stub(session, DoneStatus::DoneClean))
        }

        async fn release_scope(&self, session: &str) -> Result<()> {
            self.state.lock().unwrap().live_sessions.remove(session);
            Ok(())
        }
    }

    #[tokio::test]
    async fn spawn_failure_persists_started_sibling_for_resume_without_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("parallel spawn crash consistency");
        let phase = tracker.add_phase("parallel", "dispatch independent steps");
        for (step_id, file) in [("STEP-001", "a.rs"), ("STEP-002", "b.rs")] {
            tracker
                .add_step(
                    phase,
                    step(step_id, phase)
                        .title(step_id)
                        .files(&[file])
                        .criteria("verified")
                        .verify("test -d .")
                        .build(),
                )
                .unwrap();
        }
        tracker.save(dir.path()).unwrap();

        let runtime = FailSecondSpawnRuntime::default();
        let error = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("spawning step STEP-002"));

        let persisted = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            persisted.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "the already spawned sibling must remain durably adoptable"
        );
        assert_eq!(
            persisted.get_step("STEP-002").unwrap().status,
            StepStatus::Failed,
            "the failed external spawn must have an explicit durable terminal state"
        );

        let report = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap();
        assert!(!report.success, "the failed sibling remains visible");
        assert_eq!(report.completed, vec!["STEP-001"]);
        assert_eq!(report.failed, vec!["STEP-002"]);

        let state = runtime.state.lock().unwrap();
        assert_eq!(state.spawn_calls, vec!["STEP-001", "STEP-002"]);
        assert_eq!(state.adoption_calls, vec!["STEP-001"]);
        assert!(state.live_sessions.is_empty());
    }

    #[test]
    fn strict_done_signal_rejects_path_alias_and_forged_provenance() {
        let tmp = tempfile::tempdir().unwrap();
        let mut signal = DoneSignal::stub("worker-safe", DoneStatus::DoneClean);
        signal.write(tmp.path()).unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-safe")
            .unwrap()
            .is_some());

        signal.session = "worker-other".to_string();
        std::fs::write(
            tmp.path().join("worker-worker-safe.done.json"),
            serde_json::to_vec(&signal).unwrap(),
        )
        .unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-safe").is_err());

        let mut forged = DoneSignal::stub("worker-forged", DoneStatus::DoneClean);
        forged.projection = Some(crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: "event".to_string(),
            event_sequence: 1,
            mission_version: 1,
            projection_hash: "not-a-hash".to_string(),
        });
        forged.write(tmp.path()).unwrap();
        assert!(read_done_signal_strict(tmp.path(), "worker-forged").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn strict_done_signal_rejects_symlink_and_hardlink() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        std::fs::write(&target, b"{}").unwrap();
        symlink(&target, root.path().join("worker-worker-link.done.json")).unwrap();
        assert!(read_done_signal_strict(root.path(), "worker-link").is_err());

        let hard_target = root.path().join("hard-target");
        std::fs::write(&hard_target, b"{}").unwrap();
        std::fs::hard_link(
            &hard_target,
            root.path().join("worker-worker-hard.done.json"),
        )
        .unwrap();
        assert!(read_done_signal_strict(root.path(), "worker-hard").is_err());
    }

    #[tokio::test]
    async fn empty_scope_requires_explicit_read_only_authorization() {
        let dir = tempfile::tempdir().unwrap();
        let mut tracker = PlanTracker::new("P");
        let phase = tracker.add_phase("read", "diagnostic");
        tracker
            .add_step(
                phase,
                step("STEP-READ", phase)
                    .title("read only")
                    .files(&[])
                    .criteria("observed")
                    .verify("test -d .")
                    .build(),
            )
            .unwrap();
        tracker.save(dir.path()).unwrap();
        let runtime = FakeRuntime {
            script: HashMap::new(),
        };
        assert!(run(dir.path(), &runtime, RunOptions::default())
            .await
            .is_err());
        let report = run(
            dir.path(),
            &runtime,
            RunOptions {
                allow_read_only_steps: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(report.success);
    }

    #[tokio::test]
    async fn release_failure_prevents_persisted_completion() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let error = run(dir.path(), &ReleaseFailureRuntime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("releasing accepted worker"));
        let tracker = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            tracker.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "completion must not publish while exact scope release failed"
        );
    }

    #[tokio::test]
    async fn adoption_probe_error_preserves_in_progress_authority_without_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let mut tracker = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        tracker.start_step("STEP-001").unwrap();
        tracker.save(dir.path()).unwrap();

        let runtime = AdoptionProbeFailureRuntime::default();
        let error = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("probing worker authority for step STEP-001"));

        let persisted = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            persisted.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "an unknown session state must retain the exact in-progress generation"
        );
        assert_eq!(*runtime.spawn_calls.lock().unwrap(), 0);
        assert_eq!(*runtime.cleanup_calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn wait_and_cleanup_error_never_persists_failed_over_live_authority() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let runtime = WaitAndCleanupFailureRuntime::default();

        let error = run(dir.path(), &runtime, RunOptions::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("step remains in_progress"));
        assert_eq!(*runtime.cleanup_calls.lock().unwrap(), 1);

        let persisted = PlanTracker::load_strict(dir.path()).unwrap().unwrap();
        assert_eq!(
            persisted.get_step("STEP-001").unwrap().status,
            StepStatus::InProgress,
            "Failed would orphan a runtime generation whose containment was not confirmed"
        );
        assert_eq!(
            persisted.get_step("STEP-002").unwrap().status,
            StepStatus::Pending
        );
    }

    #[test]
    fn render_brief_includes_files_and_verify() {
        use crate::planner::step;
        let s = step("STEP-001", 1)
            .title("T")
            .description("D")
            .files(&["a.rs"])
            .criteria("ok")
            .verify("true")
            .build();
        let b = render_brief(&s);
        assert!(b.contains("a.rs"));
        assert!(b.contains("true"));
    }

    use crate::planner::step;

    fn save_linear_plan(dir: &Path, verify: &str) {
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("STEP-001", p)
                .title("a")
                .files(&["a.rs"])
                .criteria("ok")
                .verify(verify)
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("STEP-002", p)
                .title("b")
                .files(&["b.rs"])
                .criteria("ok")
                .verify(verify)
                .depends(&["STEP-001"])
                .build(),
        )
        .unwrap();
        t.add_step(
            p,
            step("STEP-003", p)
                .title("c")
                .files(&["c.rs"])
                .criteria("ok")
                .verify(verify)
                .depends(&["STEP-002"])
                .build(),
        )
        .unwrap();
        t.save(dir).unwrap();
    }

    #[tokio::test]
    async fn run_completes_all_steps_in_order() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
        let t = PlanTracker::load(dir.path()).unwrap();
        assert!(t.status().is_complete());
    }

    #[tokio::test]
    async fn resume_reconciles_orphaned_in_progress_steps() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        // Simulate a previous plan-run that died after dispatch: the step was
        // persisted as in_progress with no live worker behind it.
        let mut t = PlanTracker::load(dir.path()).unwrap();
        t.start_step("STEP-001").unwrap();
        t.save(dir.path()).unwrap();
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(
            report.success,
            "orphaned in_progress step must be re-dispatched, not stuck"
        );
        assert_eq!(report.completed, vec!["STEP-001", "STEP-002", "STEP-003"]);
    }

    #[tokio::test]
    async fn blocked_worker_is_never_marked_done() {
        let dir = tempfile::tempdir().unwrap();
        // The trap: a verify that PASSES regardless of the step's work. The
        // worker's honest `blocked` must not be laundered into Done by it.
        save_linear_plan(dir.path(), "test -d .");
        let mut script = HashMap::new();
        script.insert("STEP-001".to_string(), DoneStatus::Blocked);
        let rt = FakeRuntime { script };
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        assert!(report.completed.is_empty());
    }

    /// Records every brief it is handed, so feedback delivery is provable.
    #[derive(Default)]
    struct BriefRecorder {
        briefs: Mutex<Vec<String>>,
    }

    impl WorkerRuntime for BriefRecorder {
        async fn spawn(&self, step: &PlanStep, brief: &str, _cwd: &Path) -> Result<String> {
            self.briefs.lock().unwrap().push(brief.to_string());
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            Ok(DoneSignal::stub(step_id, DoneStatus::DoneClean))
        }
    }

    #[tokio::test]
    async fn guardian_retry_feedback_reaches_the_retry_brief() {
        let dir = tempfile::tempdir().unwrap();
        // Fail-once verify: the first Guardian run plants a marker and exits 1;
        // the re-run sees the marker and passes.
        let verify = "test -f marker || { touch marker; exit 1; }";
        let mut t = PlanTracker::new("P");
        let p = t.add_phase("F", "g");
        t.add_step(
            p,
            step("STEP-001", p)
                .title("a")
                .files(&["a.rs"])
                .criteria("ok")
                .verify(verify)
                .build(),
        )
        .unwrap();
        t.save(dir.path()).unwrap();
        let rt = BriefRecorder::default();
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        let briefs = rt.briefs.lock().unwrap();
        assert_eq!(briefs.len(), 2, "expected exactly one retry dispatch");
        assert!(
            !briefs[0].contains("PREVIOUS ATTEMPT"),
            "first brief must be clean"
        );
        assert!(
            briefs[1].contains("PREVIOUS ATTEMPT FAILED VERIFICATION"),
            "retry brief must carry the Guardian feedback, got:\n{}",
            briefs[1]
        );
    }

    #[tokio::test]
    async fn guardian_overrides_worker_done_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // worker claims DoneClean but verify fails
        let rt = FakeRuntime {
            script: HashMap::new(),
        };
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        assert_eq!(report.failed, vec!["STEP-001"]);
        assert!(report.completed.is_empty());
    }

    use std::sync::Mutex;

    /// Records the scope-lifecycle calls so the file-scope-leak regression is
    /// guarded: a Pass must release its claim, a Fail must clean it up.
    #[derive(Default)]
    struct TrackingRuntime {
        script: HashMap<String, DoneStatus>,
        released: Mutex<Vec<String>>,
        cleaned: Mutex<Vec<String>>,
    }

    impl WorkerRuntime for TrackingRuntime {
        async fn spawn(&self, step: &PlanStep, _brief: &str, _cwd: &Path) -> Result<String> {
            Ok(format!("fake-{}", step.step_id))
        }
        async fn wait_done(&self, session: &str, _t: Duration) -> Result<DoneSignal> {
            let step_id = session.strip_prefix("fake-").unwrap_or(session);
            let status = self
                .script
                .get(step_id)
                .cloned()
                .unwrap_or(DoneStatus::DoneClean);
            Ok(DoneSignal::stub(step_id, status))
        }
        async fn cleanup_failed(&self, session: &str) -> Result<()> {
            self.cleaned.lock().unwrap().push(session.to_string());
            Ok(())
        }
        async fn release_scope(&self, session: &str) -> Result<()> {
            self.released.lock().unwrap().push(session.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn pass_releases_scope_claim() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "test -d .");
        let rt = TrackingRuntime::default();
        let report = run(dir.path(), &rt, RunOptions::default()).await.unwrap();
        assert!(report.success);
        // Every completed step freed its file-scope claim (release-only)...
        assert_eq!(
            rt.released.lock().unwrap().clone(),
            vec!["fake-STEP-001", "fake-STEP-002", "fake-STEP-003"]
        );
        // ...and no successful step needed the kill+release cleanup path.
        assert!(rt.cleaned.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn fail_releases_scope_via_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        save_linear_plan(dir.path(), "false"); // verify fails -> Fail at max_attempts=1
        let rt = TrackingRuntime::default();
        let report = run(
            dir.path(),
            &rt,
            RunOptions {
                max_attempts: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!report.success);
        // The failed step's claim was freed via cleanup_failed (kill+release);
        // dependents stay blocked so only STEP-001 was cleaned.
        assert_eq!(rt.cleaned.lock().unwrap().clone(), vec!["fake-STEP-001"]);
        // A Fail is not a Pass: the release-only path was NOT taken.
        assert!(rt.released.lock().unwrap().is_empty());
    }
}
