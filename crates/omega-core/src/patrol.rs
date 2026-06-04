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

        // ── Worker patrol: done signals ──
        for session in &sessions {
            if session.role == SessionRole::Worker {
                if let Some(done) = DoneSignal::read(&self.config.state_dir, &session.name)? {
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
                        let _ = inbox.push(&InboxEvent::worker_done(&session.name, status_str));
                        // Surface the fabrication detail so the oracle can
                        // re-dispatch with eyes open.
                        if let Some(reason) = &contest_reason {
                            let _ = inbox.push(&InboxEvent::worker_blocked(
                                &session.name,
                                &format!("GROUND-TRUTH CONTEST: {}", reason),
                            ));
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
                        let _ = inbox.push(&InboxEvent::worker_blocked(
                            &session.name,
                            &blocked.question,
                        ));
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
                            if idle_secs > AUTO_DONE_IDLE_SECS {
                                // ── Conservative ground-truth gate ──
                                // Ticking all todos is NOT proof the worker
                                // finished cleanly — it may have crashed
                                // mid-edit right after the last tick. The
                                // strongest available "finished cleanly"
                                // signal is the rmux session being GONE (the
                                // process actually exited), not merely idle at
                                // a prompt. Re-probe liveness now via the
                                // SessionManager: `capture_pane` returns Err
                                // when the session/pane no longer resolves —
                                // the same dead-session idiom used by the pane
                                // stall + orphan passes above/below. We do NOT
                                // suppress legitimate auto-done: an alive-but-
                                // idle worker keeps the prior behaviour, just
                                // logged distinctly so the heuristic is visible.
                                let session_gone =
                                    mgr.capture_pane(&session.name).await.is_err();
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
                                        let _ = inbox.push(&InboxEvent::worker_done(
                                            &session.name,
                                            "pending",
                                        ));
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

        self.log_patrol(&report)?;

        Ok(report)
    }

    /// Patrol oracle sessions: check for done oracles, update registry, handle close.
    async fn patrol_oracles(
        &self,
        _mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let live_names: Vec<String> = sessions.iter().map(|s| s.name.clone()).collect();
        let mut registry = OracleRegistry::load(&self.config.state_dir);

        // Cleanup dead entries from registry
        registry.cleanup(&live_names);

        for session in sessions {
            if session.role != SessionRole::Oracle {
                continue;
            }

            // Check oracle done signal
            if let Ok(Some(done)) =
                OracleDoneSignal::read(&self.config.state_dir, &session.name)
            {
                if done.is_closeable() {
                    report.done_oracles.push(session.name.clone());
                    registry.mark_status(&session.name, OracleRegistryStatus::Done);
                    // Self-improvement: auto-dispatch the curator worker
                    // ONCE per done oracle. The marker file prevents
                    // re-triggering after the curator already ran.
                    let _ = self.maybe_trigger_curator(&session.name);
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
                    registry.mark_status(&session.name, OracleRegistryStatus::Idle);
                }
            }
        }

        let _ = registry.save(&self.config.state_dir);
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
        let done_path = self
            .config
            .state_dir
            .join(format!("oracle-{}.done.json", oracle_name));
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
