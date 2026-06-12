use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus, OracleDoneSignal, WorkerBlocked};
use crate::inbox::{Inbox, InboxEvent};
use crate::oracle_lifecycle::{
    OracleRegistry, OracleRegistryStatus, OracleState, SignalWatcher,
    WorkerEntryStatus, WorkerStallDetector, StallAction, StallThresholds,
};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;

const STALL_THRESHOLD_SECS: i64 = 900; // 15 minutes without progress = stalled (file-based)
const AUTO_DONE_IDLE_SECS: i64 = 120; // 2 minutes idle after 100% todos = patrol auto-done

// Deterministic worker close (Task#6). After a worker's done_clean clears the
// ground-truth gate, patrol marks the rmux session Closeable and reaps it (kill
// + lock release) once the parent oracle has CONSUMED/ack'd the worker_done
// event (its inbox no longer carries it) OR this bounded grace window elapses —
// whichever comes first. This removes reliance on the idle/CPU heuristic for the
// honest-done case (an honest worker used to linger as a zombie because the
// primary path never killed it). Const, not a config.rs knob, by design.
const WORKER_CLOSE_GRACE_SECS: i64 = 45;
// Grace window before a closeable (done_clean, no pending actions) oracle is
// deterministically reaped. Longer than the worker grace: the inline auto-close
// in `omega done` / `omega progress` normally fires within seconds, so patrol's
// reap is the backstop for a missed close, not the primary path.
const ORACLE_CLOSE_GRACE_SECS: i64 = 120;

// Orphan-worker sweep: a worker whose governing oracle is GONE (session dead)
// while that oracle's mission is declared done_clean is a zombie — nothing
// will ever consume its output. Generous grace after the oracle's finished_at
// so a same-name re-dispatch (which clears the stale signal first) can never
// race the sweep.
const ORPHAN_WORKER_GRACE_SECS: i64 = 300;

#[derive(Debug)]
pub struct PatrolReport {
    pub total_sessions: usize,
    pub oracles: usize,
    pub workers: usize,
    pub done_workers: Vec<String>,
    pub stalled_workers: Vec<String>,
    pub blocked_workers: Vec<String>,
    pub orphaned_sessions: Vec<String>,
    pub done_oracles: Vec<String>,
    pub actions_taken: Vec<String>,
}

pub struct Patrol {
    config: OmegaConfig,
    stall_detector: WorkerStallDetector,
    signal_watcher: SignalWatcher,
}

impl Patrol {
    pub fn new(config: OmegaConfig) -> Self {
        let signal_watcher = SignalWatcher::new(config.state_dir.clone());
        Self {
            stall_detector: WorkerStallDetector::new(StallThresholds::default()),
            signal_watcher,
            config,
        }
    }

    pub async fn run_once(&mut self) -> Result<PatrolReport> {
        // Heartbeat — proves the patrol actually fired. Lets the user (and
        // `omega doctor`) verify the self-improvement loop is alive rather
        // than silently dead (the failure mode of the old Smith agent).
        let hb = self.config.state_dir.join("patrol-heartbeat.txt");
        let _ = std::fs::create_dir_all(&self.config.state_dir);
        let _ = std::fs::write(&hb, Utc::now().to_rfc3339());

        // connect_cached: the patrol daemon calls run_once every tick — reuse one
        // process-wide rmux connection instead of opening a fresh socket per tick.
        let mgr = SessionManager::connect_cached().await?;
        let sessions = mgr.list_sessions().await?;

        let mut report = PatrolReport {
            total_sessions: sessions.len(),
            oracles: sessions.iter().filter(|s| s.role == SessionRole::Oracle).count(),
            workers: sessions.iter().filter(|s| s.role == SessionRole::Worker).count(),
            done_workers: Vec::new(),
            stalled_workers: Vec::new(),
            blocked_workers: Vec::new(),
            orphaned_sessions: Vec::new(),
            done_oracles: Vec::new(),
            actions_taken: Vec::new(),
        };

        let oracle_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();

        // Read every oracle's persisted state ONCE per tick. find_parent_oracle
        // needs it to resolve a worker -> its governing oracle, and it's called
        // once per signaling worker; reading it per call was an O(W×O) disk scan +
        // JSON parse every tick. Compute it here and pass the slice down.
        let oracle_states = crate::oracle_lifecycle::OracleState::read_all(&self.config.state_dir);

        // ── Broken-pane sweep: panes whose terminal object the daemon lost ──
        // rmux (≤0.3.1) can lose a pane's in-memory terminal while the pane
        // process keeps running (2026-06-12: recreated same-name sessions
        // listed fine but every capture/attach/status failed with "missing
        // pane terminal" — invisible in the TUI, unreachable by send-keys).
        // The pane is unusable either way, so repair beats preserving it:
        // respawn the pane (rebuilds the terminal, keeps the session and its
        // start dir), then for agent-bearing sessions relaunch the configured
        // agent with --continue so the conversation resumes where it stopped.
        // System/plain-shell sessions just get their shell back.
        for session in &sessions {
            match mgr.capture_pane(&session.name).await {
                Err(e) if format!("{e:#}").contains("missing pane terminal") => {}
                _ => continue,
            }
            tracing::warn!(session = %session.name, "Broken pane (terminal lost) — respawning");
            let respawned = std::process::Command::new("rmux")
                .args(["respawn-pane", "-k", "-t", &session.name])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !respawned {
                report.actions_taken.push(format!(
                    "Session {}: pane terminal lost, respawn FAILED — repair manually \
                     (rmux respawn-pane -k -t {})",
                    session.name, session.name
                ));
                continue;
            }
            mgr.invalidate_pane(&session.name).await;
            let relaunch_agent = session.project.is_some()
                || matches!(session.role, SessionRole::Oracle | SessionRole::Worker);
            if relaunch_agent {
                // Give the respawned shell a beat before typing into it.
                tokio::time::sleep(Duration::from_millis(500)).await;
                let agent = crate::agents::Agent::from_name(&self.config.agent_command)
                    .unwrap_or(crate::agents::Agent::Claude);
                let launch = agent.launch_command_with(
                    None,
                    crate::agents::LaunchOptions {
                        resume_conversation: true,
                        ..Default::default()
                    },
                );
                let _ = mgr.send_text(&session.name, &launch).await;
            }
            report.actions_taken.push(format!(
                "Session {}: pane terminal lost (rmux bug) — respawned pane{}",
                session.name,
                if relaunch_agent {
                    " + relaunched agent (--continue)"
                } else {
                    ""
                }
            ));
        }

        // ── Worker patrol: done signals ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                // ── Freshness guard (worker twin of the oracle guard below) ──
                // Worker names are deterministic (`<project>-worker-<task>`)
                // and the done.json survives its session, so a re-dispatch
                // under the same name would otherwise be insta-finished — and
                // then reaped — on its PREDECESSOR's stale signal. Date the
                // signal against the worker's `dispatched_at` from its
                // oracle's persisted state (register_worker refreshes it on a
                // re-dispatch). No WorkerEntry → treat as fresh: hand-spawned
                // workers have no registry entry, and dropping their signal
                // would break done delivery for them entirely.
                let fresh_done = match DoneSignal::read(&self.config.state_dir, &session.name)? {
                    Some(done) => {
                        let dispatched_at = oracle_states
                            .iter()
                            .flat_map(|s| s.workers.iter())
                            .filter(|w| w.session_name == session.name)
                            .map(|w| w.dispatched_at)
                            .max();
                        if worker_signal_is_stale(done.finished_at, dispatched_at) {
                            tracing::warn!(
                                worker = %session.name,
                                finished_at = %done.finished_at,
                                dispatched_at = ?dispatched_at,
                                "stale worker done signal predates dispatch — ignored"
                            );
                            report.actions_taken.push(format!(
                                "Ignored stale done signal for {} (predates dispatch)",
                                session.name
                            ));
                            None
                        } else {
                            Some(done)
                        }
                    }
                    None => None,
                };
                if let Some(done) = fresh_done {
                    report.done_workers.push(session.name.clone());

                    // ── Opus 4.8 ground-truth gate ──
                    // A worker's `done_clean` narration is inadmissible as
                    // proof. Verify every artifact it cited against the real
                    // repo. We CONTEST (downgrade to failed) only on a
                    // CONCRETE fabrication — a cited SHA/branch/file that does
                    // not exist. Absence of artifacts is a soft warning, never
                    // a block (avoids false-positiving honest non-code work).
                    let repo_root = crate::session::OmegaSession::classify(&session.name)
                        .project
                        .as_deref()
                        .and_then(|p| self.config.find_project(p).map(|pc| pc.path.clone()))
                        .map(std::path::PathBuf::from);
                    let mut effective_status = done.status;
                    let mut contest_reason: Option<String> = None;
                    if done.status == DoneStatus::DoneClean {
                        let verdict = crate::done::verify_done_against_repo(
                            &done,
                            repo_root.as_deref(),
                        );
                        let fabrication = verdict.checks.iter().any(|c| !c.passed);
                        if fabrication {
                            let reasons: Vec<String> = verdict
                                .checks
                                .iter()
                                .filter(|c| !c.passed)
                                .map(|c| c.detail.clone())
                                .collect();
                            contest_reason = Some(reasons.join("; "));
                            effective_status = DoneStatus::Failed;
                        } else if !verdict.passes {
                            // Weak proof (no artifacts / single-source) — accept
                            // but log it so the trend is visible.
                            report.actions_taken.push(format!(
                                "{}: done_clean accepted with weak proof ({})",
                                session.name,
                                verdict.failures.join("; ")
                            ));
                        }
                    }

                    if let Some(oracle) = self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states) {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        let status_str = if contest_reason.is_some() {
                            "contested"
                        } else {
                            match effective_status {
                                DoneStatus::DoneClean => "done_clean",
                                DoneStatus::Pending => "pending",
                                DoneStatus::Failed => "failed",
                                DoneStatus::Blocked => "blocked",
                            }
                        };
                        // Push the worker_done event ONCE per signal, not once
                        // per tick. The reap pass treats "event absent from
                        // the oracle inbox" as the ack — a re-push every tick
                        // made the ack unobservable (only the grace timer ever
                        // fired) and delivered the same event to the oracle
                        // repeatedly. The marker is keyed on status+finished_at
                        // so a NEW or upgraded signal re-arms automatically.
                        let event_key =
                            format!("{}:{}", status_str, done.finished_at.timestamp());
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "done",
                            &event_key,
                        ) {
                            let pushed = inbox
                                .push(&InboxEvent::worker_done(&session.name, status_str))
                                .is_ok();
                            // Surface the fabrication detail so the oracle can
                            // re-dispatch with eyes open.
                            if let Some(reason) = &contest_reason {
                                let _ = inbox.push(&InboxEvent::worker_blocked(
                                    &session.name,
                                    &format!("GROUND-TRUTH CONTEST: {}", reason),
                                ));
                            }
                            // Record only on a successful push — a failed one
                            // must retry next tick, not be marked delivered.
                            if pushed {
                                record_inbox_event_sent(
                                    &self.config.state_dir,
                                    &session.name,
                                    "done",
                                    &event_key,
                                );
                            }
                        }

                        // Update oracle state with worker completion
                        if let Ok(Some(mut oracle_state)) =
                            OracleState::read(&self.config.state_dir, &oracle.name)
                        {
                            let ws = match effective_status {
                                DoneStatus::DoneClean => WorkerEntryStatus::DoneClean,
                                DoneStatus::Pending => WorkerEntryStatus::Pending,
                                DoneStatus::Failed => WorkerEntryStatus::Failed,
                                DoneStatus::Blocked => WorkerEntryStatus::Blocked,
                            };
                            oracle_state.update_worker_status(&session.name, ws);
                            let _ = oracle_state.write(&self.config.state_dir);
                        }
                    }

                    if let Some(reason) = contest_reason {
                        // Fabrication: keep the scope claim HELD (work is not
                        // actually done) and flag it loudly.
                        report.actions_taken.push(format!(
                            "CONTESTED {}: done_clean failed ground-truth — {}",
                            session.name, reason
                        ));
                    } else if effective_status == DoneStatus::DoneClean {
                        let _ = ScopeClaim::release(&self.config.state_dir, &session.name);
                        self.stall_detector.forget(&session.name);
                        // Task#6 — deterministic close: an honest worker that
                        // wrote a verified done_clean used to keep its rmux
                        // session ALIVE (the only kill_session was the idle
                        // heuristic), leaving a zombie. Mark it Closeable now; the
                        // reap pass below kills it once the parent oracle ack's
                        // the worker_done event OR the grace window elapses.
                        let parent = self
                            .find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                            .map(|o| o.name.clone());
                        WorkerCloseMarker::ensure(
                            &self.config.state_dir,
                            &session.name,
                            parent.as_deref(),
                        );
                        report
                            .actions_taken
                            .push(format!("Released scope for {} (ground-truth [+]); marked Closeable", session.name));
                    }
                }

                // Check for blocked workers
                if let Ok(Some(blocked)) =
                    WorkerBlocked::read(&self.config.state_dir, &session.name)
                {
                    report.blocked_workers.push(session.name.clone());
                    if let Some(oracle) = self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states) {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        // Same push-once contract as worker_done above: the
                        // blocked file persists across ticks, so an unguarded
                        // push re-delivered the question every minute. Keyed
                        // on blocked_at — a NEW block re-arms.
                        let bkey = blocked.blocked_at.timestamp().to_string();
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "blocked",
                            &bkey,
                        ) && inbox
                            .push(&InboxEvent::worker_blocked(
                                &session.name,
                                &blocked.question,
                            ))
                            .is_ok()
                        {
                            record_inbox_event_sent(
                                &self.config.state_dir,
                                &session.name,
                                "blocked",
                                &bkey,
                            );
                        }
                    }
                }
            }
        }

        // ── Worker patrol: pane-based stall detection (30s nudge / 5min escalate) ──
        for session in &sessions {
            if session.role != SessionRole::Worker {
                continue;
            }
            let has_done = DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
            if has_done {
                continue;
            }

            match mgr.capture_pane(&session.name).await {
                Ok(content) => {
                    // A content-filter block or hard API error means the agent is
                    // stuck on an error, not merely idle. Escalate to the oracle now
                    // (with the reason) instead of waiting out the stall thresholds,
                    // and stop tracking it as a potential stall.
                    if let Some(reason) = detect_fatal_agent_error(&content) {
                        report.blocked_workers.push(session.name.clone());
                        if let Some(oracle) =
                            self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                        {
                            let inbox =
                                Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                            let _ = inbox
                                .push(&InboxEvent::worker_blocked(&session.name, reason));
                        }
                        report.actions_taken.push(format!(
                            "Worker {} blocked by {} — escalated to oracle",
                            session.name, reason
                        ));
                        self.stall_detector.forget(&session.name);
                        continue;
                    }
                    let action = self.stall_detector.check(&session.name, &content);
                    match action {
                        StallAction::Nudge { ref session, idle_secs } => {
                            tracing::info!(worker = %session, idle_secs, "Worker idle — nudge");
                            // Send a nudge via the session pane
                            let _ = mgr
                                .send_text(
                                    session,
                                    "You appear idle. Continue your mission or report done.",
                                )
                                .await;
                            report.actions_taken.push(format!(
                                "Nudged {} (idle {}s)",
                                session, idle_secs
                            ));
                        }
                        StallAction::Escalate { ref session, idle_secs } => {
                            report.stalled_workers.push(session.clone());
                            if let Some(oracle) =
                                self.find_parent_oracle(session, &oracle_sessions, &oracle_states)
                            {
                                let inbox =
                                    Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                let _ = inbox.push(&InboxEvent::worker_stalled(
                                    session,
                                    idle_secs,
                                ));
                            }
                            report.actions_taken.push(format!(
                                "Escalated stall: {} (idle {}s)",
                                session, idle_secs
                            ));
                        }
                        StallAction::Active => {}
                    }
                }
                Err(_) => {
                    // Can't capture pane — session might be dead
                    report.orphaned_sessions.push(session.name.clone());
                }
            }
        }

        // ── Worker patrol: file-based stall detection (progress files) ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(progress) =
                    crate::progress::ProgressInfo::read(&self.config.state_dir, &session.name)
                {
                    let has_done =
                        DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                    if !has_done
                        && progress.todos_completed < progress.todos_total
                        && !progress.blocked
                    {
                        if let Some(last_update) = progress.last_updated {
                            let idle_secs = (Utc::now() - last_update).num_seconds();
                            if idle_secs > STALL_THRESHOLD_SECS {
                                if !report.stalled_workers.contains(&session.name) {
                                    report.stalled_workers.push(session.name.clone());
                                    if let Some(oracle) =
                                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                                    {
                                        let inbox =
                                            Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                        let _ = inbox.push(&InboxEvent::worker_stalled(
                                            &session.name,
                                            idle_secs as u64,
                                        ));
                                    }
                                    report.actions_taken.push(format!(
                                        "Stall detected (progress): {} (idle {}s)",
                                        session.name, idle_secs
                                    ));
                                }
                            }
                        }
                    }

                    // Auto-done: worker completed all todos but exited without calling
                    // worker-mark-done.sh. After AUTO_DONE_IDLE_SECS of inactivity, patrol
                    // writes DoneSignal::done_clean on the worker's behalf.
                    if !has_done
                        && progress.todos_total > 0
                        && progress.todos_completed >= progress.todos_total
                        && !report.done_workers.contains(&session.name)
                    {
                        if let Some(last_update) = progress.last_updated {
                            let idle_secs = (Utc::now() - last_update).num_seconds();
                            // ── Conservative ground-truth gate ──
                            // Ticking all todos is NOT proof the worker finished
                            // cleanly — it may have crashed mid-edit right after
                            // the last tick. The strongest available "finished
                            // cleanly" signal is the rmux session being GONE (the
                            // process actually exited), not merely idle at a
                            // prompt. Re-probe liveness via the SessionManager:
                            // `capture_pane` returns Err when the session/pane no
                            // longer resolves — the same dead-session idiom used
                            // by the pane stall + orphan passes above/below.
                            //
                            // Thresholds split by liveness: a GONE session is safe
                            // to record after AUTO_DONE_IDLE_SECS, but an ALIVE
                            // worker that just ticked its last todo is routinely
                            // deep in its verify step (build/test > 2 min, writes
                            // no progress ticks) — killing it that early aborts
                            // real work mid-verification. Alive sessions get the
                            // full file-stall bar (STALL_THRESHOLD_SECS), the same
                            // patience as the stall pass. (This branch was dead
                            // code until the progress-schema fix made these files
                            // parse — the 120s tuning never ran against a live
                            // worker.)
                            let session_gone =
                                mgr.capture_pane(&session.name).await.is_err();
                            let idle_threshold = if session_gone {
                                AUTO_DONE_IDLE_SECS
                            } else {
                                STALL_THRESHOLD_SECS
                            };
                            if idle_secs > idle_threshold {
                                if session_gone {
                                    tracing::info!(
                                        worker = %session.name,
                                        idle_secs,
                                        "Auto-done: rmux session GONE — clean-exit confirmed"
                                    );
                                } else {
                                    tracing::warn!(
                                        worker = %session.name,
                                        idle_secs,
                                        "Auto-done HEURISTIC: session still alive but idle past \
                                         threshold with all todos ticked — proceeding (may have \
                                         stalled mid-edit; kill+auto-done as before)"
                                    );
                                    report.actions_taken.push(format!(
                                        "Auto-done HEURISTIC (session alive, idle): {} ({}/{} todos, idle {}s)",
                                        session.name,
                                        progress.todos_completed,
                                        progress.todos_total,
                                        idle_secs,
                                    ));
                                }
                                // Kill the still-live idle worker so it cannot keep
                                // editing files while we record its (un-trusted)
                                // state. N8: scope is NOT released here — it stays
                                // HELD until a real done_clean clears the gate, so
                                // no other worker can claim these files yet.
                                // (No-op / Err when the session is already gone.)
                                let _ = mgr.kill_session(&session.name).await;
                                // N8: the idle-heuristic NEVER claims done_clean
                                // on a silently-exited worker's behalf. Ticking
                                // todos + going idle is not ground truth — only a
                                // worker-written done-signal that survives the
                                // ground-truth gate is. We write Pending instead:
                                // it re-confirms next tick, preserves the contest
                                // mechanism, and — critically — does NOT release
                                // the scope claim as if the work were clean.
                                let reason = if session_gone {
                                    "auto-done HEURISTIC: todos completed + session gone — recorded PENDING (not clean; re-confirm next tick), scope HELD"
                                } else {
                                    "auto-done HEURISTIC: todos completed + idle past threshold (session still alive) — patrol killed the worker, recorded PENDING (not clean), scope HELD"
                                };
                                let mut signal = DoneSignal::new(
                                    &session.name,
                                    DoneStatus::Pending,
                                    reason,
                                );
                                signal.todos_total = progress.todos_total;
                                signal.todos_completed = progress.todos_completed;
                                match signal.write(&self.config.state_dir) {
                                    Ok(()) => {
                                    report.done_workers.push(session.name.clone());
                                    // Do NOT release scope here — the heuristic is
                                    // not proof of clean completion. Scope stays
                                    // held until a real done_clean clears the
                                    // ground-truth gate in the primary path.
                                    self.stall_detector.forget(&session.name);
                                    if let Some(oracle) =
                                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                                    {
                                        let inbox = Inbox::for_oracle(
                                            &self.config.state_dir,
                                            &oracle.name,
                                        );
                                        // Mark the event sent under the SAME
                                        // key the main done pass will compute
                                        // for this signal next tick, so it
                                        // doesn't re-deliver it (only on a
                                        // successful push — a failure retries).
                                        if inbox
                                            .push(&InboxEvent::worker_done(
                                                &session.name,
                                                "pending",
                                            ))
                                            .is_ok()
                                        {
                                            record_inbox_event_sent(
                                                &self.config.state_dir,
                                                &session.name,
                                                "done",
                                                &format!(
                                                    "pending:{}",
                                                    signal.finished_at.timestamp()
                                                ),
                                            );
                                        }
                                        if let Ok(Some(mut oracle_state)) = OracleState::read(
                                            &self.config.state_dir,
                                            &oracle.name,
                                        ) {
                                            oracle_state.update_worker_status(
                                                &session.name,
                                                WorkerEntryStatus::Pending,
                                            );
                                            let _ = oracle_state.write(&self.config.state_dir);
                                        }
                                    }
                                    tracing::info!(
                                        worker = %session.name,
                                        todos = progress.todos_completed,
                                        idle_secs,
                                        "Patrol auto-done HEURISTIC: worker recorded PENDING (scope held)"
                                    );
                                    report.actions_taken.push(format!(
                                        "Auto-done HEURISTIC -> PENDING {} ({}/{} todos, idle {}s, scope held)",
                                        session.name,
                                        progress.todos_completed,
                                        progress.todos_total,
                                        idle_secs,
                                    ));
                                    }
                                    Err(write_error) => {
                                        // The worker was already killed above but the
                                        // PENDING signal could not be persisted. Without
                                        // a signal the worker is invisible: not in
                                        // done_workers, no inbox event, no oracle-state
                                        // update — yet its scope claim stays HELD,
                                        // blocking re-dispatch. Surface it loudly (error
                                        // log + report action) so the orphan is observable
                                        // instead of failing silently.
                                        tracing::error!(
                                            worker = %session.name,
                                            error = %write_error,
                                            "Patrol auto-done FAILED to write PENDING signal — \
                                             worker killed but scope HELD with no recorded signal"
                                        );
                                        report.actions_taken.push(format!(
                                            "Auto-done FAILED to write signal for {}: {} (worker killed, scope HELD, no signal recorded)",
                                            session.name, write_error
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Deterministic worker reap (Task#6) ──
        // For each Closeable worker, reap (kill rmux session + release any
        // remaining locks) once the parent oracle has CONSUMED its worker_done
        // event (its inbox no longer carries one for this worker) OR the grace
        // window elapsed. Honest-done workers no longer linger as zombies, and
        // the reap is deterministic rather than gated on the idle/CPU heuristic.
        let live_session_names: std::collections::HashSet<&str> =
            sessions.iter().map(|s| s.name.as_str()).collect();
        for marker in WorkerCloseMarker::read_all(&self.config.state_dir) {
            // Oracle ack = the worker_done event was drained from its inbox.
            // peek() lists what's still queued; absence after we pushed it ⇒
            // the oracle consumed it (its drain deletes the file).
            let oracle_acked = match &marker.oracle {
                Some(oracle_name) => {
                    let inbox = Inbox::for_oracle(&self.config.state_dir, oracle_name);
                    let still_queued = inbox
                        .peek()
                        .map(|evts| {
                            evts.iter().any(|e| {
                                e.event_type == crate::inbox::EventType::WorkerDone
                                    && e.payload.get("session").and_then(|v| v.as_str())
                                        == Some(marker.session.as_str())
                            })
                        })
                        .unwrap_or(false);
                    !still_queued
                }
                // No known parent ⇒ rely solely on the grace window.
                None => false,
            };
            let closeable_secs = (Utc::now() - marker.since).num_seconds();
            if should_reap_closeable(oracle_acked, closeable_secs) {
                // Kill the rmux session (no-op/Err if already gone) and release
                // any remaining scope lock, atomically from patrol's view.
                let _ = mgr.kill_session(&marker.session).await;
                let _ = ScopeClaim::release(&self.config.state_dir, &marker.session);
                self.stall_detector.forget(&marker.session);
                WorkerCloseMarker::remove(&self.config.state_dir, &marker.session);
                remove_inbox_event_markers(&self.config.state_dir, &marker.session);
                let trigger = if oracle_acked { "oracle ack'd" } else { "grace elapsed" };
                tracing::info!(
                    worker = %marker.session,
                    trigger,
                    closeable_secs,
                    "Deterministic reap: honest-done worker closed"
                );
                report.actions_taken.push(format!(
                    "Reaped done_clean worker {} ({}, {}s closeable)",
                    marker.session, trigger, closeable_secs
                ));
            } else if !live_session_names.contains(marker.session.as_str()) {
                // Session already gone (e.g. the worker exited on its own before
                // the reap fired). Nothing to kill — just clear the marker + lock.
                let _ = ScopeClaim::release(&self.config.state_dir, &marker.session);
                WorkerCloseMarker::remove(&self.config.state_dir, &marker.session);
                remove_inbox_event_markers(&self.config.state_dir, &marker.session);
            }
        }

        // ── Scope-claim janitor ──
        // Only the done_clean path releases a worker's scope; failed /
        // blocked / contested workers and patrol's auto-done (PENDING, scope
        // deliberately HELD pending a real done_clean) leave the claim
        // behind. Once the owning session is DEAD that hold can never be
        // cleared by the worker itself — the files stay locked and every
        // re-dispatch over them bails on "Scope conflict" until a manual
        // `omega cleanup`. A dead owner cannot write, so releasing is safe.
        // Two guards against racing a spawn-in-progress (spawn-worker claims
        // scope an instant BEFORE its rmux session appears): require a
        // recorded done/blocked signal on disk AND a minimum claim age.
        const SCOPE_RELEASE_MIN_AGE_SECS: i64 = 300;
        for claim in ScopeClaim::read_all(&self.config.state_dir) {
            if live_session_names.contains(claim.session.as_str()) {
                continue;
            }
            if (Utc::now() - claim.claimed_at).num_seconds() < SCOPE_RELEASE_MIN_AGE_SECS {
                continue;
            }
            let has_signal = DoneSignal::read(&self.config.state_dir, &claim.session)
                .ok()
                .flatten()
                .is_some()
                || WorkerBlocked::read(&self.config.state_dir, &claim.session)
                    .ok()
                    .flatten()
                    .is_some();
            if has_signal {
                let _ = ScopeClaim::release(&self.config.state_dir, &claim.session);
                report.actions_taken.push(format!(
                    "Released scope of dead session {} (terminal signal on disk)",
                    claim.session
                ));
            }
        }

        // ── Orphan detection: sessions with no done/progress and empty pane ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                let has_done =
                    DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                let has_progress = crate::progress::ProgressInfo::read(
                    &self.config.state_dir,
                    &session.name,
                )
                .is_some();

                if !has_done && !has_progress && !report.orphaned_sessions.contains(&session.name) {
                    match mgr.capture_pane(&session.name).await {
                        Ok(content) => {
                            let trimmed = content.trim();
                            if trimmed.is_empty() || trimmed.lines().count() <= 1 {
                                report.orphaned_sessions.push(session.name.clone());
                            }
                        }
                        Err(_) => {
                            report.orphaned_sessions.push(session.name.clone());
                        }
                    }
                }
            }
        }

        // ── Oracle patrol: check done signals + registry cleanup ──
        self.patrol_oracles(&mgr, &sessions, &mut report).await?;

        // ── Orphan-worker sweep: workers whose done_clean oracle is gone ──
        // The cascade close above only fires while the oracle SESSION is still
        // alive to be reaped. When the oracle already closed (inline auto-close,
        // manual kill, crash-after-done) its leftover workers had NO reaper at
        // all — the 7-zombie dentistrygpt incident. Sweep them here.
        self.sweep_orphan_workers(&mgr, &sessions, &oracle_states, &mut report)
            .await?;

        // ── Oracle recovery: resurrect crashed-mid-mission oracles (guarded) ──
        let _ = self.resurrect_dead_oracles(&mut report).await;

        // ── Signal file watcher: detect new oracle result files ──
        if let Ok(new_signals) = self.signal_watcher.poll() {
            for (oracle_name, signal) in &new_signals {
                report.actions_taken.push(format!(
                    "Signal file detected: {} (status: {:?})",
                    oracle_name, signal.status
                ));
            }
        }

        // ── State-dir GC (bounded, age-gated) ──
        self.gc_state_dir(&live_session_names, &mut report);

        self.log_patrol(&report)?;

        Ok(report)
    }

    /// Patrol oracle sessions: check for done oracles, update registry, handle close.
    async fn patrol_oracles(
        &mut self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_names: Vec<String> = sessions.iter().map(|s| s.name.clone()).collect();
        // Read-only SNAPSHOT for the spawned_at lookups below. All mutations
        // (cleanup + status changes) are collected during the loop and applied
        // at the END under the `.oracle-registry.lock` — the old pattern
        // (load here, mutate across the kill_session awaits, save at the end,
        // no lock) clobbered any oracle a concurrent locked dispatch
        // registered mid-tick, erasing the spawned_at its freshness guard
        // depends on.
        let registry = OracleRegistry::load(&self.config.state_dir);
        let mut status_changes: Vec<(String, OracleRegistryStatus)> = Vec::new();

        for session in sessions {
            if session.role != SessionRole::Oracle {
                continue;
            }

            // Check oracle done signal
            if let Ok(Some(mut done)) =
                OracleDoneSignal::read(&self.config.state_dir, &session.name)
            {
                // ── Freshness guard (layered defense, stale-reap audit) ──
                // Oracle names recycle (Dead-purged registry entries make
                // next_oracle_name re-issue the base name) and the done.json
                // survives its session, so the signal on disk can belong to a
                // PREVIOUS mission. Acting on a stale signal killed brand-new
                // oracles (reap) and forged completions (upgrade). Date the
                // signal against the live session's registry spawned_at:
                // signal older than the session → ignore + warn. Unknown spawn
                // time (no registry entry) → ignore too: never act on a signal
                // you cannot date — the inline auto-close in `omega done` /
                // `omega progress` remains the primary close path.
                let spawned_at = registry
                    .oracles
                    .iter()
                    .find(|e| e.session_name == session.name)
                    .map(|e| e.spawned_at);
                let stale = signal_predates_session(done.finished_at, spawned_at);
                if stale {
                    tracing::warn!(
                        oracle = %session.name,
                        finished_at = %done.finished_at,
                        spawned_at = ?spawned_at,
                        "stale done signal predates session spawn — ignored (no upgrade, no reap)"
                    );
                    report.actions_taken.push(format!(
                        "Ignored stale done signal for {} (predates session spawn)",
                        session.name
                    ));
                }

                // ── L4 gate-pending upgrade (backstop for a missed progress tick) ──
                // `omega done` downgrades done_clean → Pending while the plan is
                // <100% (gate_pending=true); `omega progress` upgrades it back when
                // the final task lands. If that tick was missed, resolve it here:
                // a 100%-done, no-failure plan satisfies the L4 gate.
                if !stale && done.status == DoneStatus::Pending && done.gate_pending {
                    let key = session
                        .name
                        .strip_prefix("oracle-")
                        .unwrap_or(&session.name);
                    let pp = self
                        .config
                        .state_dir
                        .join(format!("oracle-{}.progress.json", key));
                    if let Some(pj) = std::fs::read_to_string(&pp)
                        .ok()
                        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                    {
                        let total = pj.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                        let done_n = pj.get("done").and_then(|v| v.as_u64()).unwrap_or(0);
                        let any_fail = pj
                            .get("tasks")
                            .and_then(|v| v.as_array())
                            .map(|ts| {
                                ts.iter().any(|t| {
                                    t.get("s").and_then(|v| v.as_str()) == Some("fail")
                                })
                            })
                            .unwrap_or(false);
                        if total > 0 && done_n == total && !any_fail {
                            done.status = DoneStatus::DoneClean;
                            done.pending_actions.clear();
                            done.gate_pending = false;
                            done.finished_at = Utc::now();
                            done.duration_secs =
                                (done.finished_at - done.started_at).num_seconds().max(0) as u64;
                            let _ = done.write(&self.config.state_dir);
                            // The notifier may have already reported the
                            // transient Pending state and written its per-path
                            // marker — invalidate it so the corrected
                            // done_clean is notified exactly once.
                            OracleDoneSignal::invalidate_notified(
                                &self.config.state_dir,
                                &session.name,
                            );
                            tracing::info!(
                                oracle = %session.name,
                                "L4 gate satisfied — pending upgraded to done_clean"
                            );
                            report.actions_taken.push(format!(
                                "L4 gate satisfied for {} — upgraded pending to done_clean",
                                session.name
                            ));
                        }
                    }
                }

                if !stale && done.is_closeable() {
                    report.done_oracles.push(session.name.clone());
                    status_changes.push((session.name.clone(), OracleRegistryStatus::Done));
                    // Self-improvement: auto-dispatch the curator worker
                    // ONCE per done oracle. The marker file prevents
                    // re-triggering after the curator already ran. Must run
                    // BEFORE the reap below — its flag file keeps it idempotent.
                    let _ = self.maybe_trigger_curator(&session.name);

                    // ── Deterministic oracle reap (mirror of the worker reap) ──
                    // The inline auto-close in `omega done` / `omega progress`
                    // normally closes the session within seconds; if that was
                    // missed, reap here once the grace window elapsed so an
                    // honest-done oracle never lingers as a zombie.
                    let closeable_secs = (Utc::now() - done.finished_at).num_seconds();
                    if should_reap_oracle(done.is_closeable(), closeable_secs) {
                        // ── Cascade close — an oracle NEVER leaves orphan
                        // workers behind. The mission is declared done_clean
                        // and the grace elapsed: any worker session still
                        // alive is a zombie by definition (nothing will ever
                        // consume its output), so close them all WITH the
                        // oracle. The `omega done` close-gate refuses
                        // done_clean while a worker still runs, so a running
                        // worker here means an old-binary or hand-written
                        // signal — reaped too, loudly.
                        let lw = crate::oracle_lifecycle::live_workers_of_oracle(
                            &self.config.state_dir,
                            &session.name,
                            sessions,
                        );
                        for w in lw.all() {
                            let _ = mgr.kill_session(&w).await;
                            let _ = ScopeClaim::release(&self.config.state_dir, &w);
                            self.stall_detector.forget(&w);
                            WorkerCloseMarker::remove(&self.config.state_dir, &w);
                            remove_inbox_event_markers(&self.config.state_dir, &w);
                            let was_running = lw.running.contains(&w);
                            tracing::info!(
                                oracle = %session.name, worker = %w, was_running,
                                "Cascade close: worker closed with its done_clean oracle"
                            );
                            report.actions_taken.push(format!(
                                "Cascade-closed worker {} with done_clean oracle {}{}",
                                w,
                                session.name,
                                if was_running { " (was still running!)" } else { "" }
                            ));
                        }
                        let _ = mgr.kill_session(&session.name).await;
                        // Release any scope claim the oracle still held —
                        // parity with the worker reap above (a gate-pending
                        // oracle skips the cmd_done-time release because its
                        // signal was not closeable yet, so the claim would
                        // otherwise leak until a manual cleanup).
                        let _ = ScopeClaim::release(&self.config.state_dir, &session.name);
                        tracing::info!(
                            oracle = %session.name,
                            closeable_secs,
                            "Deterministic reap: done_clean oracle closed"
                        );
                        report.actions_taken.push(format!(
                            "Reaped done_clean oracle {} ({}s past finished_at)",
                            session.name, closeable_secs
                        ));
                    }
                }
            }

            // Check oracle state for all-workers-terminal
            if let Ok(Some(oracle_state)) =
                OracleState::read(&self.config.state_dir, &session.name)
            {
                if oracle_state.all_workers_terminal()
                    && !report.done_oracles.contains(&session.name)
                {
                    // All workers are done but oracle hasn't written done signal yet — mark idle
                    status_changes.push((session.name.clone(), OracleRegistryStatus::Idle));
                }
            }
        }

        // Apply cleanup + the collected status changes atomically on a FRESH
        // reload under the registry lock, so a registration made by a
        // concurrent dispatch during this tick is merged, never lost.
        let _ = OracleRegistry::update_locked(&self.config.state_dir, |reg| {
            reg.cleanup(&live_names);
            for (name, status) in &status_changes {
                reg.mark_status(name, *status);
            }
        });
        Ok(())
    }

    /// Reap live WORKER sessions whose governing oracle is dead and whose
    /// mission is over (a closeable oracle done-signal past the grace).
    ///
    /// Parent resolution mirrors `live_workers_of_oracle`: the OracleState
    /// registry is authoritative; unregistered workers fall back to their
    /// project name. The fallback is vetoed while ANY oracle session of that
    /// project is live — a running mission may legitimately own them — and a
    /// worker with no signal to date is left alone (resurrect handles a
    /// crashed-mid-mission oracle; a signal-less orphan is its evidence).
    async fn sweep_orphan_workers(
        &mut self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        oracle_states: &[crate::oracle_lifecycle::OracleState],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_oracles: std::collections::HashSet<&str> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .map(|s| s.name.as_str())
            .collect();
        let live_oracle_projects: std::collections::HashSet<&str> = sessions
            .iter()
            .filter(|s| s.role == SessionRole::Oracle)
            .filter_map(|s| s.project.as_deref())
            .collect();
        let done_signals = OracleDoneSignal::read_all(&self.config.state_dir);

        for w in sessions.iter().filter(|s| s.role == SessionRole::Worker) {
            let registered_parent = oracle_states
                .iter()
                .find(|st| st.workers.iter().any(|e| e.session_name == w.name))
                .map(|st| st.oracle_name.clone());
            let (signal, governed_by) = match &registered_parent {
                Some(oracle_name) => {
                    if live_oracles.contains(oracle_name.as_str()) {
                        continue; // parent alive — cascade/ack paths own this
                    }
                    (
                        OracleDoneSignal::read(&self.config.state_dir, oracle_name)
                            .ok()
                            .flatten(),
                        oracle_name.clone(),
                    )
                }
                None => {
                    let Some(project) = w.project.as_deref() else { continue };
                    if live_oracle_projects.contains(project) {
                        continue; // a live oracle of this project may own it
                    }
                    (
                        done_signals
                            .iter()
                            .find(|d| d.project == project && d.is_closeable())
                            .cloned(),
                        format!("project {project}"),
                    )
                }
            };
            let Some(sig) = signal else { continue };
            let finished_secs = (Utc::now() - sig.finished_at).num_seconds();
            if !should_reap_orphan(sig.is_closeable(), finished_secs) {
                continue;
            }
            let _ = mgr.kill_session(&w.name).await;
            let _ = ScopeClaim::release(&self.config.state_dir, &w.name);
            self.stall_detector.forget(&w.name);
            WorkerCloseMarker::remove(&self.config.state_dir, &w.name);
            remove_inbox_event_markers(&self.config.state_dir, &w.name);
            tracing::info!(
                worker = %w.name, governed_by = %governed_by, finished_secs,
                "Orphan sweep: worker closed (oracle gone, mission done_clean)"
            );
            report.actions_taken.push(format!(
                "Orphan sweep: closed worker {} ({} done_clean {}s ago, oracle session gone)",
                w.name, governed_by, finished_secs
            ));
        }
        Ok(())
    }

    /// Auto-resurrect oracles that crashed mid-mission — the install-time
    /// equivalent of an oracle-watchdog. Guarded against thrash: an oracle is
    /// only brought back if it has unfinished work (workers not all terminal AND
    /// no closeable done signal), its mission is still recent (phase changed
    /// within 24h), and we have not already tried within the last 5 minutes. A
    /// finished, abandoned, or stale-stopped oracle stays dead.
    async fn resurrect_dead_oracles(&self, report: &mut PatrolReport) -> Result<()> {
        let dispatcher = crate::dispatch::Dispatcher::new(
            SessionManager::connect_cached().await?,
            self.config.clone(),
        );
        for name in dispatcher.dead_oracles().await {
            let state = match OracleState::read(&self.config.state_dir, &name) {
                Ok(Some(s)) => s,
                _ => continue,
            };
            // Finished → leave it dead.
            if state.all_workers_terminal() {
                continue;
            }
            // Never started → leave it dead. An oracle that registered ZERO
            // workers never decomposed a mission, so there is nothing to resume:
            // resurrecting it only replays the original (often malformed) dispatch
            // and spawns an empty oracle shell, which patrol then resurrects again
            // every 5 min — an infinite "empty session keeps reopening" loop.
            // (all_workers_terminal() is false for an empty worker list, so this
            // case is NOT caught above.)
            if state.workers.is_empty() {
                continue;
            }
            if let Ok(Some(done)) = OracleDoneSignal::read(&self.config.state_dir, &name) {
                if done.is_closeable() {
                    continue;
                }
            }
            // Abandoned (no activity in 24h) → leave it dead.
            if (Utc::now() - state.phase_entered_at).num_hours() > 24 {
                continue;
            }
            // Anti-thrash: don't retry within 5 minutes.
            let marker = self
                .config
                .state_dir
                .join(format!("oracle-{}.resurrect-attempt", name));
            let recently_tried = std::fs::metadata(&marker)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .map(|e| e.as_secs() < 300)
                .unwrap_or(false);
            if recently_tried {
                continue;
            }
            let _ = std::fs::write(&marker, Utc::now().to_rfc3339());
            if let Ok(crate::dispatch::ResurrectOutcome::Resurrected) =
                dispatcher.resurrect_oracle(&name).await
            {
                report
                    .actions_taken
                    .push(format!("Resurrected crashed oracle {} (mission unfinished)", name));
            }
        }
        Ok(())
    }

    /// Self-improvement hook: when an oracle's done.json flips to a
    /// closeable status, spawn a curator worker that reads the trajectory
    /// + done.json and proposes NEW_SKILL / EDIT_SKILL / NEW_RULE /
    /// NEW_MEMORY items. Output lands in
    /// `~/.omega/state/curator/<oracle>-<timestamp>.md`.
    ///
    /// Idempotent: marker file `~/.omega/state/curator-triggered/<oracle>.flag`
    /// prevents re-trigger on subsequent patrol ticks.
    fn maybe_trigger_curator(&self, oracle_name: &str) -> Result<()> {
        let flag_dir = self.config.state_dir.join("curator-triggered");
        let flag = flag_dir.join(format!("{}.flag", oracle_name));
        if flag.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(&flag_dir)?;
        std::fs::create_dir_all(self.config.state_dir.join("curator"))?;
        std::fs::write(&flag, Utc::now().to_rfc3339())?;

        // Spawn the curator as a detached rmux session so its output is
        // visible in `omega menu`. The session name is prefixed with
        // "curator-" so it's grouped distinctly in the TUI session list.
        // This is a direct `rmux new-session` (not via SessionManager), so
        // slugify here too — keep the curator name killable even if oracle_name
        // carries legacy garbage. See session::sanitize_session_name.
        let curator_session =
            crate::session::sanitize_session_name(&format!("curator-{}", oracle_name));
        // `oracle_name` is the FULL session name (`oracle-X`), but the signal
        // on disk is single-prefixed via OracleDoneSignal's oracle_key rule —
        // formatting the full name in produced `oracle-oracle-X.done.json`, a
        // path that never exists, so every curator since install read nothing
        // (7 trigger flags, zero outputs). Strip the one prefix first.
        let done_key = oracle_name.strip_prefix("oracle-").unwrap_or(oracle_name);
        let done_path = self
            .config
            .state_dir
            .join(format!("oracle-{}.done.json", done_key));
        let prompt = format!(
            "/omega-curate {}",
            done_path.to_string_lossy()
        );
        // Use claude --print --dangerously-skip-permissions for a
        // non-interactive one-shot. The session's output goes to its
        // pane (capturable) AND the curator skill writes its report
        // markdown to ~/.omega/state/curator/.
        // Manual shell-escape: wrap in single quotes and escape any
        // internal single quotes. Keeps us dependency-free.
        let escaped = prompt.replace('\'', r"'\''");
        // Build the curator command from the configured agent, not a hardcoded
        // "claude". Resolve config.agent_command -> Agent -> binary name; fall
        // back to the literal string if it's an unknown agent name.
        let agent_bin = crate::agents::Agent::from_name(&self.config.agent_command)
            .map(|a| a.name().to_string())
            .unwrap_or_else(|| self.config.agent_command.clone());
        let cmd = format!(
            "{} --print --dangerously-skip-permissions '{}' ; exec bash",
            agent_bin, escaped
        );
        let mgr_dispatch = std::process::Command::new("rmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &curator_session,
                "bash",
                "-c",
                &cmd,
            ])
            .status();
        match mgr_dispatch {
            Ok(s) if s.success() => {
                tracing::info!(
                    oracle = %oracle_name,
                    curator = %curator_session,
                    "curator dispatched"
                );
            }
            _ => {
                tracing::warn!(oracle = %oracle_name, "curator dispatch failed");
            }
        }
        Ok(())
    }

    /// Bounded state-dir garbage collection. The spawn paths write per-session
    /// side files (`{name}.mcp.json`, `{name}.debug.log`), and the protocol
    /// leaves per-signal files behind (worker done.json, push-once `.sent`
    /// markers, resurrect/stuck markers, retired `-prev` done.json) — nothing
    /// deleted any of them automatically, so a live host accumulated 460+
    /// files. Every rule below is conservative: keyed on the owning session
    /// being DEAD and an age floor, so an in-flight spawn (side files written
    /// an instant before the rmux session appears) is never swept. Also
    /// migrates legacy double-prefixed oracle state files in passing (see
    /// `OracleState::state_key`).
    fn gc_state_dir(
        &self,
        live: &std::collections::HashSet<&str>,
        report: &mut PatrolReport,
    ) {
        const HOUR: u64 = 3_600;
        const DAY: u64 = 86_400;
        let dir = &self.config.state_dir;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut removed = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let age = file_age_secs(&path).unwrap_or(0);
            let dead_after =
                |session: &str, min_age: u64| !live.contains(session) && age >= min_age;

            // Legacy double-prefixed oracle state → migrate once (rename into
            // the canonical single-prefix name; drop it if the canonical file
            // already exists — the live binary has been writing there).
            if name.starts_with("oracle-oracle-") && name.ends_with(".state.json") {
                let target = dir.join(&name["oracle-".len()..]);
                if target.exists() {
                    let _ = std::fs::remove_file(&path);
                } else {
                    let _ = std::fs::rename(&path, &target);
                }
                continue;
            }

            let stale = if let Some(session) = name.strip_suffix(".mcp.json") {
                // One written per spawn (oracle AND worker), never read after
                // launch — the single largest garbage class.
                dead_after(session, HOUR)
            } else if let Some(session) = name.strip_suffix(".debug.log") {
                // Post-mortem value decays fast; keep a week.
                dead_after(session, 7 * DAY)
            } else if name.starts_with("worker-") && name.ends_with(".done.json") {
                // Long-consumed worker signals. The spawn-time clear protects a
                // re-dispatch immediately; this only bounds the pile.
                let session = &name["worker-".len()..name.len() - ".done.json".len()];
                dead_after(session, 7 * DAY)
            } else if name.ends_with(".resurrect-attempt") {
                // Pure anti-thrash stamp with a 5-minute window — a day-old
                // marker is garbage regardless of session state.
                age >= DAY
            } else if name.starts_with("oracle-")
                && (name.ends_with(".state.json") || name.ends_with(".progress.json"))
            {
                // Lifecycle state of a DEAD oracle. Resurrect abandons a
                // mission after 24h (phase_entered_at), so two-days-dead
                // state is a pure phantom: its only remaining effect was
                // feeding the stuck-alert cron endless "oracle bloqué"
                // pings for sessions the operator can't even see (216 such
                // files had accumulated by 2026-06-11). The .state.json
                // basename IS the session name (state_key strips the
                // oracle- prefix before formatting).
                let stem = name
                    .trim_end_matches(".state.json")
                    .trim_end_matches(".progress.json");
                dead_after(stem, 2 * DAY)
            } else if name.starts_with("oracle-")
                && (name.ends_with(".report.json")
                    || name.ends_with(".report.pdf")
                    || name.ends_with(".findings.md"))
            {
                // Delivered mission artifacts — same 14-day record window
                // as retired done.json signals.
                let stem = name
                    .trim_end_matches(".report.json")
                    .trim_end_matches(".report.pdf")
                    .trim_end_matches(".findings.md");
                dead_after(stem, 14 * DAY)
            } else if name.ends_with(".inbox.lock") || name.ends_with(".inbox.jsonl") {
                // Inbox side files are keyed on the FULL session name with an
                // extra oracle- prefix ("oracle-<session>.inbox.lock", giving
                // oracle-oracle-X for oracles) — strip one prefix to recover
                // the owning session.
                let stem = name
                    .trim_end_matches(".inbox.lock")
                    .trim_end_matches(".inbox.jsonl");
                let session = stem.strip_prefix("oracle-").unwrap_or(stem);
                dead_after(session, 7 * DAY)
            } else if let Some(stem) = name.strip_suffix(".stuck-alerted") {
                // The cron keys this on the state-file basename: `oracle-X`,
                // or legacy `oracle-oracle-X`. Live when either form maps to
                // a live session; dead → removing re-arms the alert for a
                // recycled name (mirrors OracleDoneSignal::clear).
                let owner_live = live.contains(stem)
                    || stem
                        .strip_prefix("oracle-")
                        .map(|s| live.contains(s))
                        .unwrap_or(false);
                !owner_live && age >= HOUR
            } else if name.starts_with("worker-") && name.ends_with(".sent") {
                // Push-once markers — the reap removes them; this catches leaks.
                let stem = &name["worker-".len()..];
                let session = stem
                    .strip_suffix(".done.sent")
                    .or_else(|| stem.strip_suffix(".blocked.sent"))
                    .unwrap_or(stem);
                dead_after(session, 7 * DAY)
            } else if name.starts_with("oracle-")
                && name.ends_with(".done.json")
                && is_retired_done_name(&name)
            {
                // Retired signals: only once DELIVERED (.notified sibling) and
                // past a 14-day record window — never destroy an unsent report.
                let notified = dir.join(format!("{}.notified", name));
                if notified.exists() && age >= 14 * DAY {
                    let _ = std::fs::remove_file(&notified);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if stale && std::fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
        if removed > 0 {
            report
                .actions_taken
                .push(format!("GC: removed {} stale state file(s)", removed));
        }
    }

    fn find_parent_oracle<'a>(
        &self,
        worker_name: &str,
        oracles: &'a [&crate::session::OmegaSession],
        states: &[crate::oracle_lifecycle::OracleState],
    ) -> Option<&'a crate::session::OmegaSession> {
        // Authoritative: the oracle whose OracleState registry actually lists
        // this worker. This is correct even with multiple oracles per project.
        // `states` is read ONCE per tick by the caller (was an O(W×O) per-call
        // disk scan before).
        if let Some(state) = states
            .iter()
            .find(|s| s.workers.iter().any(|w| w.session_name == worker_name))
        {
            if let Some(o) = oracles.iter().find(|o| o.name == state.oracle_name) {
                return Some(o);
            }
        }
        // Fallback: first oracle of the same project (best-effort if no registry hit).
        let worker_session = crate::session::OmegaSession::classify(worker_name);
        let worker_project = worker_session.project.as_deref()?;
        oracles
            .iter()
            .find(|o| o.project.as_deref() == Some(worker_project))
            .copied()
    }

    pub async fn run_loop(&mut self, interval: Duration) -> Result<()> {
        tracing::info!(interval_secs = interval.as_secs(), "Patrol daemon started");
        loop {
            match self.run_once().await {
                Ok(report) => {
                    tracing::info!(
                        sessions = report.total_sessions,
                        done_workers = report.done_workers.len(),
                        stalled = report.stalled_workers.len(),
                        done_oracles = report.done_oracles.len(),
                        orphaned = report.orphaned_sessions.len(),
                        actions = report.actions_taken.len(),
                        "Patrol tick"
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Patrol tick failed");
                }
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn log_patrol(&self, report: &PatrolReport) -> Result<()> {
        let log_line = format!(
            "[{}] sessions={} oracles={} workers={} done_w={} stalled={} blocked={} orphaned={} done_o={} actions={}\n",
            Utc::now().format("%Y-%m-%dT%H:%M:%S"),
            report.total_sessions,
            report.oracles,
            report.workers,
            report.done_workers.len(),
            report.stalled_workers.len(),
            report.blocked_workers.len(),
            report.orphaned_sessions.len(),
            report.done_oracles.len(),
            report.actions_taken.len(),
        );

        let log_path = self.config.logs_dir.join("patrol.log");
        std::fs::create_dir_all(&self.config.logs_dir)?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(log_line.as_bytes())?;
        Ok(())
    }
}

/// Deterministic worker-close marker (Task#6). Written when a worker's
/// done_clean clears the ground-truth gate; consumed by the reap pass. Persisted
/// so a patrol restart still reaps a pending close instead of leaking a zombie.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkerCloseMarker {
    session: String,
    /// Parent oracle to watch for the worker_done ack (inbox consumption).
    oracle: Option<String>,
    /// When the worker became Closeable — start of the grace window.
    since: chrono::DateTime<Utc>,
}

impl WorkerCloseMarker {
    fn path(state_dir: &std::path::Path, session: &str) -> std::path::PathBuf {
        state_dir.join(format!("worker-close-{}.json", session))
    }

    /// Write the marker once. Idempotent: if it already exists, keep the
    /// original `since` so the grace clock isn't reset every tick.
    fn ensure(state_dir: &std::path::Path, session: &str, oracle: Option<&str>) {
        let path = Self::path(state_dir, session);
        if path.exists() {
            return;
        }
        let marker = WorkerCloseMarker {
            session: session.to_string(),
            oracle: oracle.map(|s| s.to_string()),
            since: Utc::now(),
        };
        if let Ok(content) = serde_json::to_string(&marker) {
            let _ = std::fs::write(&path, content);
        }
    }

    fn read_all(state_dir: &std::path::Path) -> Vec<WorkerCloseMarker> {
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("worker-close-") && name.ends_with(".json") {
                        if let Ok(c) = std::fs::read_to_string(&p) {
                            if let Ok(m) = serde_json::from_str::<WorkerCloseMarker>(&c) {
                                out.push(m);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    fn remove(state_dir: &std::path::Path, session: &str) {
        let _ = std::fs::remove_file(Self::path(state_dir, session));
    }
}

/// Reap predicate (Task#6, pure + testable). Given whether the parent oracle
/// has consumed the worker_done event (`oracle_acked`) and how long the worker
/// has been Closeable, decide whether to reap NOW. Reap when the oracle ack'd OR
/// the bounded grace window elapsed — whichever first.
fn should_reap_closeable(oracle_acked: bool, closeable_secs: i64) -> bool {
    oracle_acked || closeable_secs >= WORKER_CLOSE_GRACE_SECS
}

/// Oracle reap predicate (pure + testable). An oracle is reaped only when its
/// done signal is closeable (done_clean, no pending actions) AND the grace
/// window since `finished_at` has elapsed — the grace gives the inline
/// auto-close (and the done-notifier cron) time to act first.
fn should_reap_oracle(closeable: bool, secs: i64) -> bool {
    closeable && secs >= ORACLE_CLOSE_GRACE_SECS
}

/// Orphan-worker predicate (pure + testable). A live worker whose governing
/// oracle session is GONE is reaped only when that oracle's mission is over
/// (closeable done signal) AND the generous orphan grace has elapsed since
/// `finished_at` — a same-name re-dispatch clears the stale signal before
/// spawning, so the sweep can never act on a superseded mission's signal.
fn should_reap_orphan(closeable: bool, finished_secs: i64) -> bool {
    closeable && finished_secs >= ORPHAN_WORKER_GRACE_SECS
}

/// Freshness guard predicate (pure + testable). A done signal whose
/// `finished_at` predates the live session's spawn belongs to a PREVIOUS
/// mission that recycled the name — patrol must never upgrade or reap on it.
/// Unknown spawn time (no registry entry for the session) is treated as stale
/// too: never act on a signal you cannot date. Dispatch registers spawned_at
/// via reserve_oracle and resurrect via register_resurrected, so a live
/// OmegaOS-launched oracle always has one; the conservative default only
/// affects hand-made sessions, where killing would be worse than lingering.
fn signal_predates_session(
    finished_at: chrono::DateTime<Utc>,
    session_spawned_at: Option<chrono::DateTime<Utc>>,
) -> bool {
    match session_spawned_at {
        Some(spawned_at) => finished_at < spawned_at,
        None => true,
    }
}

/// Worker-signal freshness predicate (pure + testable, the worker twin of
/// `signal_predates_session`). A done.json whose `finished_at` predates the
/// worker's `dispatched_at` belongs to a PREVIOUS mission that recycled the
/// deterministic worker name — acting on it insta-finishes (and reaps) the
/// new worker. Unlike the oracle guard, an UNKNOWN dispatch time is treated
/// as FRESH: workers without a registry entry (hand-spawned) have no other
/// done-delivery path, so dropping their signal would silence them entirely.
fn worker_signal_is_stale(
    finished_at: chrono::DateTime<Utc>,
    dispatched_at: Option<chrono::DateTime<Utc>>,
) -> bool {
    matches!(dispatched_at, Some(d) if finished_at < d)
}

/// Push-once markers for the per-tick inbox pushes. Patrol re-detects the
/// same done/blocked file every tick while it exists; the marker records the
/// content key (status + signal timestamp) of the event last pushed for a
/// session, so the event reaches the oracle exactly once per signal. A new
/// or upgraded signal carries a different key and re-arms automatically —
/// no coordination with the spawn-time stale-signal clear is needed.
fn event_sent_path(state_dir: &std::path::Path, session: &str, kind: &str) -> std::path::PathBuf {
    state_dir.join(format!("worker-{}.{}.sent", session, kind))
}

fn inbox_event_already_sent(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
    key: &str,
) -> bool {
    std::fs::read_to_string(event_sent_path(state_dir, session, kind))
        .map(|c| c.trim() == key)
        .unwrap_or(false)
}

fn record_inbox_event_sent(state_dir: &std::path::Path, session: &str, kind: &str, key: &str) {
    let _ = std::fs::write(event_sent_path(state_dir, session, kind), key);
}

fn remove_inbox_event_markers(state_dir: &std::path::Path, session: &str) {
    let _ = std::fs::remove_file(event_sent_path(state_dir, session, "done"));
    let _ = std::fs::remove_file(event_sent_path(state_dir, session, "blocked"));
}

/// Age of a file in seconds via mtime — `None` when unreadable (the GC then
/// treats it as age 0, i.e. never deletes on an unknown clock).
fn file_age_secs(path: &std::path::Path) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
}

/// True for a RETIRED oracle signal — `oracle-<key>-prev<ts>.done.json`, the
/// rename `OracleDoneSignal::clear` performs on an un-notified signal. The
/// `<ts>` digits requirement keeps a project whose name merely contains
/// "-prev" out of the GC's reach.
fn is_retired_done_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".done.json") else {
        return false;
    };
    match stem.rfind("-prev") {
        Some(i) => {
            let ts = &stem[i + "-prev".len()..];
            !ts.is_empty() && ts.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Detect a fatal, non-recoverable agent error in a session's pane output — the
/// agent is stuck on an error rather than working or idle. Only the tail (the
/// live error, not old scrollback) is inspected. A content-filter block and a
/// hard API error qualify; a line that says it is retrying does not. The
/// returned string is a short reason for the oracle's inbox.
fn detect_fatal_agent_error(content: &str) -> Option<&'static str> {
    let tail: String = content.lines().rev().take(8).collect::<Vec<_>>().join("\n");
    if tail.contains("content filtering policy") || tail.contains("Output blocked by content") {
        Some("content-filter block")
    } else if tail.contains("API Error") && !tail.contains("retry") && !tail.contains("Retrying") {
        Some("API error")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_content_filter_block() {
        let pane = "working\nAPI Error: Output blocked by content filtering policy\n❯";
        assert_eq!(detect_fatal_agent_error(pane), Some("content-filter block"));
    }

    #[test]
    fn detects_hard_api_error() {
        assert_eq!(detect_fatal_agent_error("boom\nAPI Error: 500 internal\n❯"), Some("API error"));
    }

    #[test]
    fn ignores_retrying_and_normal_output() {
        assert_eq!(detect_fatal_agent_error("API Error: 529 overloaded, Retrying in 5s"), None);
        assert_eq!(detect_fatal_agent_error("just working on it\n❯"), None);
    }

    #[test]
    fn orphan_sweep_needs_closeable_signal_and_grace() {
        // No closeable signal → never reap, however old (resurrect's domain).
        assert!(!should_reap_orphan(false, 0));
        assert!(!should_reap_orphan(false, ORPHAN_WORKER_GRACE_SECS * 10));
        // Closeable but inside the grace → wait (re-dispatch race window).
        assert!(!should_reap_orphan(true, 0));
        assert!(!should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS - 1));
        // Closeable + grace elapsed → reap.
        assert!(should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS));
        assert!(should_reap_orphan(true, ORPHAN_WORKER_GRACE_SECS + 600));
    }

    #[test]
    fn reap_fires_on_oracle_ack_or_grace() {
        // Task#6 reap predicate: reap as soon as the oracle ack's, regardless of
        // how little time has elapsed.
        assert!(should_reap_closeable(true, 0));
        assert!(should_reap_closeable(true, WORKER_CLOSE_GRACE_SECS - 1));
        // Without an ack, reap only after the bounded grace window elapses.
        assert!(!should_reap_closeable(false, 0));
        assert!(!should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS - 1));
        assert!(should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS));
        assert!(should_reap_closeable(false, WORKER_CLOSE_GRACE_SECS + 10));
    }

    #[test]
    fn oracle_reap_fires_only_when_closeable_and_grace_elapsed() {
        // Mirrors should_reap_closeable: a non-closeable oracle is NEVER reaped,
        // no matter how long it has been finished.
        assert!(!should_reap_oracle(false, 0));
        assert!(!should_reap_oracle(false, ORACLE_CLOSE_GRACE_SECS + 600));
        // Closeable but inside the grace window — give the inline auto-close
        // a chance first.
        assert!(!should_reap_oracle(true, 0));
        assert!(!should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS - 1));
        // Closeable + grace elapsed → reap.
        assert!(should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS));
        assert!(should_reap_oracle(true, ORACLE_CLOSE_GRACE_SECS + 10));
    }

    #[test]
    fn stale_signal_predates_session_guard() {
        let spawn = Utc::now();
        // Signal from a PRIOR mission (finished before this session spawned)
        // → stale: no reap, no gate-pending upgrade.
        assert!(signal_predates_session(spawn - chrono::Duration::hours(3), Some(spawn)));
        assert!(signal_predates_session(spawn - chrono::Duration::seconds(1), Some(spawn)));
        // Signal written BY this session (at or after spawn) → fresh.
        assert!(!signal_predates_session(spawn, Some(spawn)));
        assert!(!signal_predates_session(spawn + chrono::Duration::seconds(30), Some(spawn)));
        // Unknown spawn time (no registry entry) → conservatively stale:
        // never kill a session you cannot date.
        assert!(signal_predates_session(Utc::now(), None));
    }

    #[test]
    fn worker_stale_signal_guard() {
        let dispatch = Utc::now();
        // Predecessor's signal (finished before this dispatch) → stale.
        assert!(worker_signal_is_stale(
            dispatch - chrono::Duration::hours(2),
            Some(dispatch)
        ));
        // Signal written by THIS dispatch → fresh.
        assert!(!worker_signal_is_stale(
            dispatch + chrono::Duration::seconds(30),
            Some(dispatch)
        ));
        // Unknown dispatch time (hand-spawned worker, no registry entry) →
        // FRESH: the opposite default to the oracle guard, because dropping
        // the signal would break done delivery for unregistered workers.
        assert!(!worker_signal_is_stale(Utc::now(), None));
    }

    #[test]
    fn inbox_event_markers_are_content_keyed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        // Nothing sent yet.
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:100"));
        record_inbox_event_sent(dir, "w1", "done", "done_clean:100");
        // Same signal → already sent (no per-tick re-push).
        assert!(inbox_event_already_sent(dir, "w1", "done", "done_clean:100"));
        // Upgraded / new signal (different key) → re-armed.
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:200"));
        assert!(!inbox_event_already_sent(dir, "w1", "done", "pending:100"));
        // Kinds are independent.
        assert!(!inbox_event_already_sent(dir, "w1", "blocked", "100"));
        remove_inbox_event_markers(dir, "w1");
        assert!(!inbox_event_already_sent(dir, "w1", "done", "done_clean:100"));
    }

    #[test]
    fn retired_done_name_matcher() {
        assert!(is_retired_done_name("oracle-OmegaOS-prev1765432100.done.json"));
        // A live signal is never "retired", even for a project containing -prev.
        assert!(!is_retired_done_name("oracle-OmegaOS.done.json"));
        assert!(!is_retired_done_name("oracle-x-prevention.done.json"));
        assert!(!is_retired_done_name("oracle-x-prev.done.json"));
    }

    #[test]
    fn close_marker_is_idempotent_keeps_since() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X"));
        let first = WorkerCloseMarker::read_all(dir);
        assert_eq!(first.len(), 1);
        let since0 = first[0].since;
        // Re-ensure must NOT reset `since` (grace clock stability).
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X"));
        let second = WorkerCloseMarker::read_all(dir);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].since, since0);
        WorkerCloseMarker::remove(dir, "worker-x");
        assert!(WorkerCloseMarker::read_all(dir).is_empty());
    }

    #[test]
    fn ignores_stale_error_in_scrollback() {
        let mut lines = vec!["API Error: Output blocked by content filtering policy"];
        for _ in 0..20 {
            lines.push("normal output line");
        }
        assert_eq!(detect_fatal_agent_error(&lines.join("\n")), None);
    }
}
