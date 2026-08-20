use crate::config::OmegaConfig;
use crate::done::{DoneSignal, DoneStatus, OracleDoneSignal, WorkerBlocked};
use crate::inbox::{Inbox, InboxEvent};
use crate::oracle_lifecycle::{
    OracleRegistry, OracleRegistryStatus, OracleState, SignalWatcher, StallAction, StallThresholds,
    WorkerEntry, WorkerEntryStatus, WorkerStallDetector,
};
use crate::scope::ScopeClaim;
use crate::session::{SessionManager, SessionRole};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::time::Duration;

const STALL_THRESHOLD_SECS: i64 = 900; // 15 minutes without progress = stalled (file-based)
const AUTO_DONE_IDLE_SECS: i64 = 120; // 2 minutes idle after 100% todos = patrol auto-done
const TEAM_LEASE_RENEW_TTL_SECS: u64 = 300;
const WORKER_RUNTIME_PREPARED_RECOVERY_GRACE_SECS: i64 = 30;
const WORKER_RUNTIME_PREPARED_STALE_SECS: i64 = 300;

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

// Ceiling on how old a done signal may be and still authorize an orphan reap.
//
// The grace above is a FLOOR and was the only bound, which is safe on the
// registered-parent branch (the signal is read by exact oracle name, and a
// same-name re-dispatch clears it first) but NOT on the project-match branch
// below, which accepts any closeable signal sharing the worker's project. A
// project accumulates done.json files forever, so the oldest one kept matching
// and authorized killing workers it never governed.
//
// Measured incident (2026-08-04): `oracle-OmegaOS-4.done.json`, done_clean and
// finished 2026-06-28, matched every freshly spawned `OmegaOS-worker-*` and the
// sweep closed three README workers within 60s of spawn each, before any of
// them could write a file or a done signal.
//
// A genuine orphan is swept within GRACE + one patrol tick of its mission
// finishing, so a signal older than this ceiling cannot still have live
// legitimate orphans: anything running under it is a NEW worker. The window is
// deliberately generous (24h) to tolerate patrol downtime, and erring toward
// leaving a worker alive matches this module's own doctrine that losing a
// worker's commits is far worse than leaking one - a leak is recoverable with
// `omega reap`, a wrongful kill destroys unsaved work.
const ORPHAN_SIGNAL_MAX_AGE_SECS: i64 = 86_400;

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

#[derive(Debug, Default)]
struct WorkerRuntimePatrolOutcome {
    managed_sessions: std::collections::HashSet<String>,
    inventory_compromised: bool,
}

#[derive(Debug)]
struct WorkerRuntimeAuthority {
    attempt: crate::orchestration::AuthoritativeTaskAttempt,
    mission_state: crate::mission::MissionState,
    attempt_state: crate::mission::TaskAttemptState,
    task_name: String,
    task_scope: Vec<String>,
}

enum WorkerRuntimeCandidateEvidence {
    None,
    MarkerOnly,
    Exact {
        runtime: Box<crate::worker_runtime::WorkerRuntimeManifest>,
        signal: Box<DoneSignal>,
    },
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

    /// Resolve V3 task authority for one worker. `None` means the session is a
    /// legacy/unregistered worker; any malformed V3 binding is an error, never
    /// permission to fall back to compatibility state.
    fn v3_worker_attempt_state(
        &self,
        oracle_states: &[OracleState],
        session: &str,
    ) -> Result<Option<crate::mission::TaskAttemptState>> {
        let Some((oracle, worker)) = oracle_states.iter().find_map(|oracle| {
            oracle
                .workers
                .iter()
                .find(|worker| worker.session_name == session)
                .map(|worker| (oracle, worker))
        }) else {
            return Ok(None);
        };
        let attempt_id = worker
            .attempt_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no attempt id"))?;
        let revision = worker
            .plan_revision
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no plan revision"))?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle.require_ledger_authority(&ledger)?;
        let attempt = ledger
            .task_attempt(attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("registered V3 attempt is missing"))?;
        if attempt.mission_id != oracle.mission_id
            || attempt.task_id != worker.task_id
            || attempt.plan_revision != revision
        {
            anyhow::bail!("registered worker differs from exact ledger task authority");
        }
        Ok(Some(attempt.state))
    }

    fn release_exact_accepted_worker_scopes(
        &self,
        oracle: &OracleState,
        worker: &crate::oracle_lifecycle::WorkerEntry,
    ) -> Result<()> {
        let attempt_id = worker
            .attempt_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no attempt id"))?;
        let plan_revision = worker
            .plan_revision
            .ok_or_else(|| anyhow::anyhow!("registered V3 worker has no plan revision"))?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle.require_ledger_authority(&ledger)?;
        let projection = ledger
            .task_attempt(attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("registered V3 attempt is missing"))?;
        if projection.mission_id != oracle.mission_id
            || projection.task_id != worker.task_id
            || projection.plan_revision != plan_revision
            || projection.state != crate::mission::TaskAttemptState::Accepted
        {
            anyhow::bail!("scope release requires the exact Accepted V3 attempt");
        }
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: oracle.mission_id.clone(),
            task_id: worker.task_id.clone(),
            attempt_id: attempt_id.to_string(),
            plan_revision,
            owner: Some(worker.session_name.clone()),
            leases: ledger.active_leases_for_attempt(
                &oracle.mission_id,
                &worker.task_id,
                attempt_id,
            )?,
            scope_receipt: None,
        };
        crate::orchestration::release_authoritative_scopes(
            &ledger,
            &self.config.state_dir,
            &attempt,
        )
    }

    fn worker_runtime_ledger(&self) -> Result<crate::mission_ledger::MissionLedger> {
        crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )
        .map_err(Into::into)
    }

    fn load_worker_runtime_authority(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    ) -> Result<(crate::mission_ledger::MissionLedger, WorkerRuntimeAuthority)> {
        let ledger = self.worker_runtime_ledger()?;
        let identity = runtime.attempt();
        let mission = ledger
            .mission_record(&identity.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        let mission_projection = ledger
            .mission(&identity.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission projection disappeared"))?;
        if mission.project != runtime.project()
            || std::fs::canonicalize(&mission.working_dir)? != runtime.authority_workspace()
        {
            anyhow::bail!(
                "worker runtime project/authority workspace differs from its immutable mission"
            );
        }
        let plan = ledger
            .active_plan(&identity.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission has no active plan"))?;
        plan.verify_integrity()
            .map_err(|error| anyhow::anyhow!("worker runtime plan integrity failed: {error}"))?;
        if plan.revision != identity.plan_revision || plan.content_digest != identity.plan_digest {
            anyhow::bail!("worker runtime differs from the active immutable plan");
        }
        let task = plan
            .tasks
            .iter()
            .find(|task| task.task_id.as_str() == identity.task_id)
            .ok_or_else(|| anyhow::anyhow!("worker runtime task is absent from its active plan"))?;
        let projection = ledger
            .task_attempt(&identity.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime attempt disappeared"))?;
        if projection.mission_id != identity.mission_id
            || projection.task_id != identity.task_id
            || projection.plan_revision != identity.plan_revision
            || identity.owner != runtime.session().session
        {
            anyhow::bail!("worker runtime attempt identity differs from ledger authority");
        }
        let ledger_parent = ledger.parent_link_for_child(&identity.mission_id)?;
        if ledger_parent.as_ref() != runtime.parent_link() {
            anyhow::bail!("worker runtime parent link differs from reciprocal ledger authority");
        }

        let active_leases = ledger.active_leases_for_attempt(
            &identity.mission_id,
            &identity.task_id,
            &identity.attempt_id,
        )?;
        match runtime.scope() {
            Some(scope) => {
                let receipt = scope.receipt();
                if receipt.mission_id != identity.mission_id
                    || receipt.task_id != identity.task_id
                    || receipt.attempt_id != identity.attempt_id
                    || receipt.plan_revision != identity.plan_revision
                    || receipt.owner != identity.owner
                    || receipt.claim.session != identity.owner
                    || receipt.claim.files_owned != task.scope
                {
                    anyhow::bail!("worker runtime scope receipt differs from its task authority");
                }
                let receipt_events = ledger
                    .events(&identity.mission_id)?
                    .into_iter()
                    .filter(|event| {
                        event.kind == "scope_claim_authority_prepared"
                            && event.plan_revision == Some(identity.plan_revision)
                    })
                    .filter_map(|event| {
                        serde_json::from_value::<crate::orchestration::AuthoritativeScopeReceipt>(
                            event.payload,
                        )
                        .ok()
                    })
                    .filter(|candidate| candidate.attempt_id == identity.attempt_id)
                    .collect::<Vec<_>>();
                if receipt_events.len() != 1 {
                    anyhow::bail!(
                        "worker runtime attempt has {} immutable scope receipts",
                        receipt_events.len()
                    );
                }
                let immutable = &receipt_events[0];
                if immutable.mission_id != receipt.mission_id
                    || immutable.task_id != receipt.task_id
                    || immutable.attempt_id != receipt.attempt_id
                    || immutable.plan_revision != receipt.plan_revision
                    || immutable.owner != receipt.owner
                    || immutable.claim != receipt.claim
                {
                    anyhow::bail!("worker runtime frozen scope differs from its ledger receipt");
                }
                if !active_leases.is_empty()
                    && (active_leases.len() != scope.resources().len()
                        || active_leases.iter().any(|lease| {
                            lease.owner != identity.owner
                                || !scope.resources().iter().any(|resource| {
                                    resource.resource_key == lease.resource_key
                                        && resource.fencing_token == lease.fencing_token
                                })
                        }))
                {
                    anyhow::bail!("worker runtime active leases differ from frozen fences");
                }
                match ScopeClaim::read_strict(&self.config.state_dir, &identity.owner)? {
                    Some(claim) if claim == receipt.claim => {}
                    None if active_leases.is_empty() => {}
                    Some(_) => anyhow::bail!(
                        "worker runtime compatibility claim differs from its immutable receipt"
                    ),
                    None => anyhow::bail!(
                        "worker runtime compatibility claim disappeared while leases remain"
                    ),
                }
            }
            None => {
                if !task.scope.is_empty()
                    || !active_leases.is_empty()
                    || ScopeClaim::read_strict(&self.config.state_dir, &identity.owner)?.is_some()
                {
                    anyhow::bail!("read-only worker runtime carries writable scope authority");
                }
            }
        }
        if !matches!(
            projection.state,
            crate::mission::TaskAttemptState::Queued | crate::mission::TaskAttemptState::Cancelled
        ) {
            let running_actor = ledger
                .events(&identity.mission_id)?
                .into_iter()
                .any(|event| {
                    event.actor == identity.owner
                        && event
                            .resulting_task_attempt
                            .as_ref()
                            .is_some_and(|attempt| {
                                attempt.attempt_id == identity.attempt_id
                                    && attempt.state == crate::mission::TaskAttemptState::Running
                            })
                });
            if !running_actor {
                anyhow::bail!("worker runtime owner did not author its Running transition");
            }
        }
        Ok((
            ledger,
            WorkerRuntimeAuthority {
                attempt: crate::orchestration::AuthoritativeTaskAttempt {
                    mission_id: identity.mission_id.clone(),
                    task_id: identity.task_id.clone(),
                    attempt_id: identity.attempt_id.clone(),
                    plan_revision: identity.plan_revision,
                    owner: Some(identity.owner.clone()),
                    leases: active_leases,
                    scope_receipt: None,
                },
                mission_state: mission_projection.state,
                attempt_state: projection.state,
                task_name: task.name.clone(),
                task_scope: task.scope.clone(),
            },
        ))
    }

    fn renew_worker_runtime_leases(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    ) -> Result<()> {
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        if authority.attempt_state.is_terminal() {
            return Ok(());
        }
        let Some(scope) = runtime.scope() else {
            return Ok(());
        };
        if authority.attempt.leases.len() != scope.resources().len() {
            anyhow::bail!("worker runtime lease cardinality changed before renewal");
        }
        for resource in scope.resources() {
            let renewed = ledger.renew_lease(
                &resource.resource_key,
                &runtime.attempt().owner,
                resource.fencing_token,
                Duration::from_secs(TEAM_LEASE_RENEW_TTL_SECS),
            )?;
            if renewed.mission_id != runtime.attempt().mission_id
                || renewed.task_id != runtime.attempt().task_id
                || renewed.attempt_id != runtime.attempt().attempt_id
                || renewed.owner != runtime.attempt().owner
                || renewed.fencing_token != resource.fencing_token
            {
                anyhow::bail!("worker runtime lease renewal changed immutable authority");
            }
        }
        Ok(())
    }

    fn mark_worker_runtime_blocked(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        reason: &str,
    ) -> Result<()> {
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        match authority.attempt_state {
            crate::mission::TaskAttemptState::Queued
            | crate::mission::TaskAttemptState::Running
            | crate::mission::TaskAttemptState::CorrectionRequired => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Cancelled,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::TaskAttemptState::CandidateDone => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Verifying,
                    &runtime.attempt().owner,
                )?;
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::TaskAttemptState::Verifying => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::TaskAttemptState::Blocked
            | crate::mission::TaskAttemptState::Accepted
            | crate::mission::TaskAttemptState::Failed
            | crate::mission::TaskAttemptState::Cancelled => {}
        }
        let mut mission = ledger
            .mission(&runtime.attempt().mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        if mission.state == crate::mission::MissionState::Planned
            || mission.state == crate::mission::MissionState::CorrectionRequired
        {
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &runtime.attempt().mission_id,
                crate::mission::MissionState::Running,
                &runtime.attempt().owner,
            )?;
            mission = ledger
                .mission(&runtime.attempt().mission_id)?
                .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        }
        if mission.state == crate::mission::MissionState::Running {
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &runtime.attempt().mission_id,
                crate::mission::MissionState::Verifying,
                &runtime.attempt().owner,
            )?;
            mission = ledger
                .mission(&runtime.attempt().mission_id)?
                .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        }
        if mission.state == crate::mission::MissionState::Verifying {
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &runtime.attempt().mission_id,
                crate::mission::MissionState::Blocked,
                &runtime.attempt().owner,
            )?;
        }
        let final_state = ledger
            .mission(&runtime.attempt().mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?
            .state;
        if !matches!(
            final_state,
            crate::mission::MissionState::Blocked
                | crate::mission::MissionState::Failed
                | crate::mission::MissionState::Cancelled
                | crate::mission::MissionState::Delivered
        ) {
            anyhow::bail!(
                "worker runtime containment failed but mission remained {:?}: {reason}",
                final_state
            );
        }
        Ok(())
    }

    fn reconcile_worker_runtime_authority_after_absence(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        absence: &crate::worker_runtime::ConfirmedWorkerAbsence,
        force_blocked: bool,
    ) -> Result<()> {
        absence.proves(runtime)?;
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        match authority.attempt_state {
            crate::mission::TaskAttemptState::Queued
            | crate::mission::TaskAttemptState::Running
            | crate::mission::TaskAttemptState::CorrectionRequired
            | crate::mission::TaskAttemptState::Blocked => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Cancelled,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::TaskAttemptState::CandidateDone => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Verifying,
                    &runtime.attempt().owner,
                )?;
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &runtime.attempt().owner,
                )?;
                if !force_blocked {
                    crate::orchestration::transition_authoritative_attempt(
                        &ledger,
                        &authority.attempt,
                        crate::mission::TaskAttemptState::Cancelled,
                        &runtime.attempt().owner,
                    )?;
                }
            }
            crate::mission::TaskAttemptState::Verifying => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &runtime.attempt().owner,
                )?;
                if !force_blocked {
                    crate::orchestration::transition_authoritative_attempt(
                        &ledger,
                        &authority.attempt,
                        crate::mission::TaskAttemptState::Cancelled,
                        &runtime.attempt().owner,
                    )?;
                }
            }
            crate::mission::TaskAttemptState::Accepted
            | crate::mission::TaskAttemptState::Failed
            | crate::mission::TaskAttemptState::Cancelled => {}
        }

        let mission = ledger
            .mission(&runtime.attempt().mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        if !mission.state.is_terminal() {
            if force_blocked {
                self.mark_worker_runtime_blocked(runtime, "post-absence runtime reconciliation")?;
            } else {
                match mission.state {
                    crate::mission::MissionState::Planned
                    | crate::mission::MissionState::Running
                    | crate::mission::MissionState::CorrectionRequired
                    | crate::mission::MissionState::Blocked => {
                        crate::orchestration::transition_authoritative_mission(
                            &ledger,
                            &runtime.attempt().mission_id,
                            crate::mission::MissionState::Cancelled,
                            &runtime.attempt().owner,
                        )?;
                    }
                    crate::mission::MissionState::Verifying => {
                        crate::orchestration::transition_authoritative_mission(
                            &ledger,
                            &runtime.attempt().mission_id,
                            crate::mission::MissionState::Failed,
                            &runtime.attempt().owner,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        let (_, refreshed) = self.load_worker_runtime_authority(runtime)?;
        let release = refreshed.attempt;
        crate::orchestration::release_authoritative_scopes(
            &ledger,
            &self.config.state_dir,
            &release,
        )?;
        if !ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )?
            .is_empty()
            || ScopeClaim::read_strict(&self.config.state_dir, &runtime.attempt().owner)?.is_some()
        {
            anyhow::bail!("worker runtime authority remained after confirmed process absence");
        }
        if crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &self.config.state_dir,
            runtime.runtime_id(),
        )?
        .is_none()
        {
            anyhow::bail!("worker runtime evidence disappeared during reconciliation");
        }
        Ok(())
    }

    fn ensure_worker_runtime_registered(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    ) -> Result<Option<String>> {
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        let states = OracleState::read_all_strict(&self.config.state_dir)?;
        let existing_owners = states
            .iter()
            .filter(|state| {
                state
                    .workers
                    .iter()
                    .any(|worker| worker.session_name == runtime.session().session)
            })
            .map(|state| state.oracle_name.as_str())
            .collect::<Vec<_>>();
        let Some(link) = runtime.parent_link() else {
            if !existing_owners.is_empty() {
                anyhow::bail!(
                    "standalone worker runtime is registered under Oracle state {:?}",
                    existing_owners
                );
            }
            return Ok(None);
        };
        let parent_id = crate::mission::MissionId(link.parent_mission_id.clone());
        let matches = states
            .iter()
            .filter(|state| state.mission_id == parent_id)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            anyhow::bail!(
                "worker runtime parent mission resolves to {} Oracle states",
                matches.len()
            );
        }
        let oracle_name = matches[0].oracle_name.clone();
        if existing_owners
            .iter()
            .any(|owner| *owner != oracle_name.as_str())
        {
            anyhow::bail!("worker runtime session is registered under another Oracle");
        }
        let _lock = crate::worker_runtime::lock_oracle_worker_registry(
            &self.config.state_dir,
            &oracle_name,
        )?;
        let mut state = OracleState::read(&self.config.state_dir, &oracle_name)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime parent Oracle state disappeared"))?;
        if state.mission_id != parent_id {
            anyhow::bail!("worker runtime parent Oracle generation changed under registry lock");
        }
        state.require_ledger_authority(&ledger)?;
        if let Some(existing) = state
            .workers
            .iter()
            .find(|worker| worker.session_name == runtime.session().session)
        {
            if existing.task_id != runtime.attempt().task_id
                || existing.attempt_id.as_deref() != Some(runtime.attempt().attempt_id.as_str())
                || existing.plan_revision != Some(runtime.attempt().plan_revision)
                || existing.files_owned != authority.task_scope
            {
                anyhow::bail!("existing Oracle worker entry differs from exact runtime authority");
            }
        }
        let status = match authority.attempt_state {
            crate::mission::TaskAttemptState::Queued
            | crate::mission::TaskAttemptState::Running => WorkerEntryStatus::Running,
            crate::mission::TaskAttemptState::CandidateDone
            | crate::mission::TaskAttemptState::Verifying
            | crate::mission::TaskAttemptState::CorrectionRequired => WorkerEntryStatus::Pending,
            crate::mission::TaskAttemptState::Accepted => WorkerEntryStatus::DoneClean,
            crate::mission::TaskAttemptState::Blocked => WorkerEntryStatus::Blocked,
            crate::mission::TaskAttemptState::Failed => WorkerEntryStatus::Failed,
            crate::mission::TaskAttemptState::Cancelled => WorkerEntryStatus::Failed,
        };
        state.register_worker(WorkerEntry {
            session_name: runtime.session().session.clone(),
            task_id: runtime.attempt().task_id.clone(),
            task_name: authority.task_name,
            attempt_id: Some(runtime.attempt().attempt_id.clone()),
            plan_revision: Some(runtime.attempt().plan_revision),
            files_owned: authority.task_scope,
            dispatched_at: runtime.prepared_at(),
            status,
        });
        state
            .write(&self.config.state_dir)
            .with_context(|| format!("registering recovered worker under {oracle_name}"))?;
        Ok(Some(oracle_name))
    }

    fn update_worker_runtime_registry_status(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        status: WorkerEntryStatus,
    ) -> Result<Option<String>> {
        let Some(oracle_name) = self.ensure_worker_runtime_registered(runtime)? else {
            return Ok(None);
        };
        let _lock = crate::worker_runtime::lock_oracle_worker_registry(
            &self.config.state_dir,
            &oracle_name,
        )?;
        let mut state = OracleState::read(&self.config.state_dir, &oracle_name)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime parent Oracle state disappeared"))?;
        let matches = state
            .workers
            .iter()
            .filter(|worker| worker.session_name == runtime.session().session)
            .count();
        if matches != 1 {
            anyhow::bail!("worker runtime registry upsert did not produce one exact entry");
        }
        state.update_worker_status(&runtime.session().session, status);
        state
            .write(&self.config.state_dir)
            .with_context(|| format!("updating recovered worker status under {oracle_name}"))?;
        Ok(Some(oracle_name))
    }

    fn load_worker_runtime_candidate_evidence(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    ) -> Result<WorkerRuntimeCandidateEvidence> {
        let ledger = self.worker_runtime_ledger()?;
        let events = ledger.events(&runtime.attempt().mission_id)?;
        let candidates = events
            .into_iter()
            .filter(|event| {
                event.kind == "worker_runtime_completion_candidate"
                    && (event.correlation_id.as_deref() == Some(runtime.runtime_id())
                        || event.attempt_id.as_deref()
                            == Some(runtime.attempt().attempt_id.as_str()))
            })
            .collect::<Vec<_>>();
        if candidates.len() > 1 {
            anyhow::bail!("worker runtime has multiple immutable completion candidates");
        }
        let marker = DoneSignal::read(&self.config.state_dir, &runtime.session().session)?;
        let Some(event) = candidates.into_iter().next() else {
            if runtime.candidate().is_some() {
                anyhow::bail!("worker runtime binds a candidate absent from the ledger");
            }
            return Ok(if marker.is_some() {
                WorkerRuntimeCandidateEvidence::MarkerOnly
            } else {
                WorkerRuntimeCandidateEvidence::None
            });
        };
        let started = runtime
            .started()
            .ok_or_else(|| anyhow::anyhow!("prepared runtime has a completion candidate"))?;
        let resulting = event
            .resulting_task_attempt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("worker candidate event has no attempt projection"))?;
        if event.mission_id != runtime.attempt().mission_id
            || event.task_id.as_deref() != Some(runtime.attempt().task_id.as_str())
            || event.attempt_id.as_deref() != Some(runtime.attempt().attempt_id.as_str())
            || event.plan_revision != Some(runtime.attempt().plan_revision)
            || event.actor != runtime.attempt().owner
            || event.provider.as_deref() != Some(runtime.provider())
            || event.correlation_id.as_deref() != Some(runtime.runtime_id())
            || resulting.mission_id != runtime.attempt().mission_id
            || resulting.task_id != runtime.attempt().task_id
            || resulting.attempt_id != runtime.attempt().attempt_id
            || resulting.plan_revision != runtime.attempt().plan_revision
            || resulting.state != crate::mission::TaskAttemptState::CandidateDone
            || event.recorded_at < started.activated_at
        {
            anyhow::bail!("worker candidate event differs from exact runtime authority");
        }
        let mut signal: DoneSignal = serde_json::from_value(event.payload.clone())?;
        if signal.session != runtime.session().session
            || signal.projection.is_some()
            || signal.finished_at < started.activated_at
            || signal.finished_at > event.recorded_at
        {
            anyhow::bail!("worker candidate payload differs from its activated generation");
        }
        let digest = crate::worker_runtime::candidate_payload_digest(&event.payload)?;
        let identity =
            crate::worker_runtime::WorkerCandidateIdentity::new(event.event_id.clone(), digest)?;
        let runtime = match runtime.candidate() {
            Some(bound) if bound.identity == identity && bound.bound_at >= started.activated_at => {
                runtime.clone()
            }
            Some(_) => {
                anyhow::bail!("worker runtime candidate binding differs from ledger evidence")
            }
            None => runtime.bind_candidate(&self.config.state_dir, identity)?,
        };
        let projection = ledger
            .projection_at(&runtime.attempt().mission_id, event.sequence)?
            .ok_or_else(|| anyhow::anyhow!("worker candidate projection cannot be replayed"))?;
        signal.projection = Some(crate::done::ProjectionProvenance {
            source: "mission-engine-v3.sqlite3".to_string(),
            event_id: event.event_id,
            event_sequence: event.sequence,
            mission_version: projection.version,
            projection_hash: projection.projection_hash,
        });
        match marker {
            Some(recorded)
                if serde_json::to_value(&recorded)? != serde_json::to_value(&signal)? =>
            {
                anyhow::bail!("worker completion marker differs from immutable candidate")
            }
            Some(_) => {}
            None => signal
                .write(&self.config.state_dir)
                .context("recovering exact worker completion signal from ledger")?,
        }
        Ok(WorkerRuntimeCandidateEvidence::Exact {
            runtime: Box::new(runtime),
            signal: Box::new(signal),
        })
    }

    fn release_worker_runtime_scopes_after_absence(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        absence: &crate::worker_runtime::ConfirmedWorkerAbsence,
    ) -> Result<()> {
        absence.proves(runtime)?;
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        crate::orchestration::release_authoritative_scopes(
            &ledger,
            &self.config.state_dir,
            &authority.attempt,
        )?;
        if !ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )?
            .is_empty()
            || ScopeClaim::read_strict(&self.config.state_dir, &runtime.attempt().owner)?.is_some()
        {
            anyhow::bail!("worker scope authority remained after daemon-confirmed absence");
        }
        Ok(())
    }

    fn retire_worker_runtime_if_terminal(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        absence: &crate::worker_runtime::ConfirmedWorkerAbsence,
    ) -> Result<bool> {
        absence.proves(runtime)?;
        if crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
            &self.config.state_dir,
            runtime.runtime_id(),
        )?
        .is_some()
        {
            return Ok(true);
        }
        let Some(current) = crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &self.config.state_dir,
            runtime.runtime_id(),
        )?
        else {
            anyhow::bail!("worker runtime disappeared without terminal archive evidence");
        };
        let ledger = self.worker_runtime_ledger()?;
        let mission = ledger
            .mission(&runtime.attempt().mission_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
        let attempt = ledger
            .task_attempt(&runtime.attempt().attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("worker runtime attempt disappeared"))?;
        if !mission.state.is_terminal() || !attempt.state.is_terminal() {
            return Ok(false);
        }
        let archive = current.retire_terminal(&self.config.state_dir, absence)?;
        if archive.runtime_id != runtime.runtime_id()
            || archive.terminal.mission_state != mission.state
            || archive.terminal.attempt_state != attempt.state
            || archive.terminal.absence != *absence
        {
            anyhow::bail!("worker runtime terminal archive differs from accepted evidence");
        }
        Ok(true)
    }

    fn publish_worker_runtime_outcome(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        signal: &DoneSignal,
        status: WorkerEntryStatus,
        status_label: &str,
    ) -> Result<()> {
        let Some(oracle_name) = self.update_worker_runtime_registry_status(runtime, status)? else {
            return Ok(());
        };
        let states = OracleState::read_all_strict(&self.config.state_dir)?;
        let binding = strict_worker_binding(&states, &runtime.session().session)?;
        let key = worker_done_event_key(status_label, signal, binding)?;
        if inbox_event_already_sent(
            &self.config.state_dir,
            &runtime.session().session,
            "done",
            &key,
        )? {
            return Ok(());
        }
        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle_name);
        inbox.push(&InboxEvent::worker_done(
            &runtime.session().session,
            status_label,
        ))?;
        record_inbox_event_sent(
            &self.config.state_dir,
            &runtime.session().session,
            "done",
            &key,
        )
    }

    fn settle_worker_runtime_candidate_after_absence(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        signal: &DoneSignal,
        absence: &crate::worker_runtime::ConfirmedWorkerAbsence,
    ) -> Result<bool> {
        absence.proves(runtime)?;
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        let mut accepted = authority.attempt_state == crate::mission::TaskAttemptState::Accepted;
        match signal.status {
            DoneStatus::DoneClean => {
                // Publication is deliberately retriable. A previous tick may
                // have persisted the independent verdict and mission state,
                // then crashed before updating OracleState or its inbox. Do
                // not try to run the quality gate again from those durable
                // states; settle only the missing idempotent side effects.
                if authority.mission_state == crate::mission::MissionState::Blocked {
                    self.mark_worker_runtime_blocked(
                        runtime,
                        "recovering a previously blocked clean candidate",
                    )?;
                    self.release_worker_runtime_scopes_after_absence(runtime, absence)?;
                    self.publish_worker_runtime_outcome(
                        runtime,
                        signal,
                        WorkerEntryStatus::Blocked,
                        "blocked",
                    )?;
                    return Ok(false);
                }
                if matches!(
                    authority.mission_state,
                    crate::mission::MissionState::Failed | crate::mission::MissionState::Cancelled
                ) {
                    self.reconcile_worker_runtime_authority_after_absence(runtime, absence, false)?;
                    self.publish_worker_runtime_outcome(
                        runtime,
                        signal,
                        WorkerEntryStatus::Failed,
                        "failed",
                    )?;
                    return self.retire_worker_runtime_if_terminal(runtime, absence);
                }
                if matches!(
                    authority.attempt_state,
                    crate::mission::TaskAttemptState::CandidateDone
                        | crate::mission::TaskAttemptState::Verifying
                ) {
                    let outcome = crate::orchestration::verify_and_finalize_candidate(
                        &ledger,
                        &runtime.attempt().mission_id,
                        &runtime.attempt().task_id,
                        &runtime.attempt().attempt_id,
                        runtime.attempt().plan_revision,
                        &runtime.attempt().owner,
                        signal,
                        runtime.workspace(),
                    )?;
                    accepted = outcome.accepted
                        && outcome.attempt_state == crate::mission::TaskAttemptState::Accepted;
                    if !accepted {
                        self.mark_worker_runtime_blocked(
                            runtime,
                            "independent runtime candidate verification rejected",
                        )?;
                        self.release_worker_runtime_scopes_after_absence(runtime, absence)?;
                        self.publish_worker_runtime_outcome(
                            runtime,
                            signal,
                            WorkerEntryStatus::Blocked,
                            "blocked",
                        )?;
                        return Ok(false);
                    }
                }
                if !accepted {
                    anyhow::bail!(
                        "clean worker candidate cannot settle from {:?}",
                        authority.attempt_state
                    );
                }

                // The task is independently Accepted and the exact process is
                // absent. Release its fences before the mission gate, whose
                // own acceptance contract requires zero active leases.
                self.release_worker_runtime_scopes_after_absence(runtime, absence)?;

                let mut mission = ledger
                    .mission(&runtime.attempt().mission_id)?
                    .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                if mission.state == crate::mission::MissionState::Running {
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        crate::mission::MissionState::Verifying,
                        "omega-worker-runtime-patrol",
                    )?;
                    mission = ledger
                        .mission(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                }
                if mission.state == crate::mission::MissionState::Verifying {
                    let mission_record = ledger
                        .mission_record(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker mission contract disappeared"))?;
                    let plan = ledger
                        .active_plan(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker active plan disappeared"))?;
                    let rubric =
                        crate::orchestration::build_authoritative_rubric(&mission_record, &plan);
                    let result = crate::mission::WorkerResult {
                        task_id: runtime.attempt().task_id.clone(),
                        session_name: runtime.attempt().owner.clone(),
                        status: DoneStatus::DoneClean,
                        summary: signal.summary.clone(),
                        commit: signal.commit.clone(),
                        duration_secs: (signal.finished_at - runtime.prepared_at())
                            .num_seconds()
                            .max(0) as u64,
                    };
                    let gate = crate::orchestration::Orchestrator::run_quality_gate(
                        &self.config.state_dir,
                        &ledger,
                        &mission_record,
                        &plan,
                        &rubric,
                        &[result],
                    )?;
                    if !gate.overall_pass {
                        crate::orchestration::transition_authoritative_mission(
                            &ledger,
                            &runtime.attempt().mission_id,
                            crate::mission::MissionState::CorrectionRequired,
                            "omega-worker-runtime-patrol",
                        )?;
                        self.mark_worker_runtime_blocked(
                            runtime,
                            "authoritative mission quality gate rejected",
                        )?;
                        self.publish_worker_runtime_outcome(
                            runtime,
                            signal,
                            WorkerEntryStatus::Blocked,
                            "blocked",
                        )?;
                        return Ok(false);
                    }
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        crate::mission::MissionState::Accepted,
                        "omega-worker-runtime-patrol",
                    )?;
                    mission = ledger
                        .mission(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                }
                if mission.state == crate::mission::MissionState::Accepted {
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        crate::mission::MissionState::Reporting,
                        "omega-worker-runtime-patrol",
                    )?;
                    mission = ledger
                        .mission(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                }
                if mission.state == crate::mission::MissionState::Reporting {
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        crate::mission::MissionState::Delivered,
                        "omega-worker-runtime-patrol",
                    )?;
                    mission = ledger
                        .mission(&runtime.attempt().mission_id)?
                        .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                }
                if mission.state != crate::mission::MissionState::Delivered {
                    anyhow::bail!(
                        "accepted worker candidate left mission in {:?}",
                        mission.state
                    );
                }
                ledger.validate_mission_acceptance(&runtime.attempt().mission_id)?;
                self.publish_worker_runtime_outcome(
                    runtime,
                    signal,
                    WorkerEntryStatus::DoneClean,
                    "done_clean",
                )?;
            }
            DoneStatus::Failed | DoneStatus::Blocked | DoneStatus::Pending => {
                let target = if signal.status == DoneStatus::Failed {
                    crate::mission::TaskAttemptState::Failed
                } else {
                    crate::mission::TaskAttemptState::Blocked
                };
                crate::orchestration::finalize_nonclean_candidate(
                    &ledger,
                    &runtime.attempt().mission_id,
                    &runtime.attempt().task_id,
                    &runtime.attempt().attempt_id,
                    runtime.attempt().plan_revision,
                    &runtime.attempt().owner,
                    signal,
                    target,
                )?;
                let mission = ledger
                    .mission(&runtime.attempt().mission_id)?
                    .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                if mission.state == crate::mission::MissionState::Running {
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        crate::mission::MissionState::Verifying,
                        "omega-worker-runtime-patrol",
                    )?;
                }
                let mission = ledger
                    .mission(&runtime.attempt().mission_id)?
                    .ok_or_else(|| anyhow::anyhow!("worker runtime mission disappeared"))?;
                if mission.state == crate::mission::MissionState::Verifying {
                    crate::orchestration::transition_authoritative_mission(
                        &ledger,
                        &runtime.attempt().mission_id,
                        if target == crate::mission::TaskAttemptState::Failed {
                            crate::mission::MissionState::Failed
                        } else {
                            crate::mission::MissionState::Blocked
                        },
                        "omega-worker-runtime-patrol",
                    )?;
                }
                self.release_worker_runtime_scopes_after_absence(runtime, absence)?;
                let (entry_status, label) = if target == crate::mission::TaskAttemptState::Failed {
                    (WorkerEntryStatus::Failed, "failed")
                } else {
                    (WorkerEntryStatus::Blocked, "blocked")
                };
                self.publish_worker_runtime_outcome(runtime, signal, entry_status, label)?;
            }
        }
        self.retire_worker_runtime_if_terminal(runtime, absence)
    }

    fn recover_prepared_worker_authority(
        &self,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    ) -> Result<()> {
        let (ledger, authority) = self.load_worker_runtime_authority(runtime)?;
        match authority.mission_state {
            crate::mission::MissionState::Planned
            | crate::mission::MissionState::CorrectionRequired => {
                crate::orchestration::transition_authoritative_mission(
                    &ledger,
                    &runtime.attempt().mission_id,
                    crate::mission::MissionState::Running,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::MissionState::Running => {}
            state => anyhow::bail!(
                "prepared worker cannot activate while mission is {:?}",
                state
            ),
        }
        match authority.attempt_state {
            crate::mission::TaskAttemptState::Queued
            | crate::mission::TaskAttemptState::CorrectionRequired
            | crate::mission::TaskAttemptState::Blocked => {
                crate::orchestration::transition_authoritative_attempt(
                    &ledger,
                    &authority.attempt,
                    crate::mission::TaskAttemptState::Running,
                    &runtime.attempt().owner,
                )?;
            }
            crate::mission::TaskAttemptState::Running => {}
            state => anyhow::bail!(
                "prepared worker cannot activate while attempt is {:?}",
                state
            ),
        }
        Ok(())
    }

    fn validate_prepared_worker_observation(
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        observed: &crate::worker_runtime::ObservedWorkerProcess,
    ) -> Result<()> {
        if observed.session != runtime.session().session
            || observed.command_digest != runtime.expected_command_digest()
            || observed.working_dir != runtime.workspace()
        {
            anyhow::bail!("prepared effect differs from the expected command/workspace generation");
        }
        Ok(())
    }

    async fn contain_worker_runtime_failure(
        &self,
        mgr: &SessionManager,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        reason: &str,
        report: &mut PatrolReport,
    ) {
        match mgr.contain_worker_runtime_session(runtime).await {
            Ok(absence) => {
                match self.reconcile_worker_runtime_authority_after_absence(
                    runtime,
                    &absence,
                    true,
                ) {
                    Ok(()) => match self.ensure_worker_runtime_registered(runtime) {
                        Ok(_) => match self.retire_worker_runtime_if_terminal(runtime, &absence) {
                            Ok(retired) => report.actions_taken.push(format!(
                                "Worker runtime {} contained after failure; exact absence proved, terminal/Blocked authority reconciled and Oracle registry synchronized{} ({reason})",
                                runtime.session().session,
                                if retired { "; terminal evidence archived" } else { "" }
                            )),
                            Err(error) => report.actions_taken.push(format!(
                                "Worker runtime {} contained and Oracle registry synchronized, but terminal archival failed closed ({error:#}); runtime evidence retained ({reason})",
                                runtime.session().session
                            )),
                        },
                        Err(error) => report.actions_taken.push(format!(
                            "Worker runtime {} contained and terminal/Blocked authority reconciled, but Oracle registry synchronization failed closed ({error:#}); runtime evidence retained ({reason})",
                            runtime.session().session
                        )),
                    },
                    Err(error) => report.actions_taken.push(format!(
                        "Worker runtime {} contained but ledger/scope reconciliation failed closed ({error:#}); runtime evidence retained ({reason})",
                        runtime.session().session
                    )),
                }
            }
            Err(containment_error) => {
                let blocked = self.mark_worker_runtime_blocked(runtime, reason);
                report.actions_taken.push(match blocked {
                    Ok(()) => format!(
                        "Worker runtime {} containment failed ({containment_error:#}); mission Blocked and scope authority retained ({reason})",
                        runtime.session().session
                    ),
                    Err(block_error) => format!(
                        "Worker runtime {} containment and Blocked persistence both failed closed (containment: {containment_error:#}; ledger: {block_error:#}); all authority retained ({reason})",
                        runtime.session().session
                    ),
                });
            }
        }
    }

    async fn process_one_worker_runtime(
        &self,
        mgr: &SessionManager,
        runtime: &crate::worker_runtime::WorkerRuntimeManifest,
        state: crate::worker_runtime::WorkerRuntimeReconcileState,
        observed: Option<&crate::worker_runtime::ObservedWorkerProcess>,
        observation_error: Option<&str>,
        report: &mut PatrolReport,
    ) -> Result<()> {
        if let Some(error) = observation_error {
            anyhow::bail!("live worker process could not be observed exactly: {error}");
        }
        match state {
            crate::worker_runtime::WorkerRuntimeReconcileState::PreparedNoEffect => {
                if observed.is_some() {
                    anyhow::bail!("prepared-no-effect runtime unexpectedly has an observation");
                }
                if !prepared_worker_runtime_is_stale(runtime, Utc::now()) {
                    self.load_worker_runtime_authority(runtime)?;
                    self.renew_worker_runtime_leases(runtime)?;
                    report.actions_taken.push(format!(
                        "Worker runtime {} is freshly Prepared with no observed effect; retained for the in-flight spawn window",
                        runtime.session().session
                    ));
                    return Ok(());
                }
                if runtime.is_start_gate_released(&self.config.state_dir)? {
                    anyhow::bail!(
                        "stale Prepared runtime has a released gate and cannot prove no external effect"
                    );
                }
                let absence = mgr.confirm_worker_runtime_absence(runtime).await?;
                self.reconcile_worker_runtime_authority_after_absence(runtime, &absence, false)?;
                self.update_worker_runtime_registry_status(runtime, WorkerEntryStatus::Failed)?;
                if !self.retire_worker_runtime_if_terminal(runtime, &absence)? {
                    anyhow::bail!("prepared-no-effect runtime did not reach terminal authority");
                }
                report.actions_taken.push(format!(
                    "Worker runtime {} had no external effect; exact absence confirmed, authority Cancelled, evidence archived",
                    runtime.session().session
                ));
            }
            crate::worker_runtime::WorkerRuntimeReconcileState::PreparedEffectObserved => {
                let observed = observed.ok_or_else(|| {
                    anyhow::anyhow!("prepared-effect runtime lost its exact observation")
                })?;
                if runtime.is_start_gate_released(&self.config.state_dir)? {
                    anyhow::bail!(
                        "prepared runtime start gate was released before durable activation"
                    );
                }
                Self::validate_prepared_worker_observation(runtime, observed)?;
                if !prepared_worker_runtime_recovery_ready(runtime, Utc::now()) {
                    self.load_worker_runtime_authority(runtime)?;
                    self.renew_worker_runtime_leases(runtime)?;
                    report.actions_taken.push(format!(
                        "Worker runtime {} has a fresh gated Prepared effect; retained for the spawning CLI to publish activation",
                        runtime.session().session
                    ));
                    return Ok(());
                }
                self.recover_prepared_worker_authority(runtime)?;
                let activated =
                    runtime.activate_started(&self.config.state_dir, observed.clone())?;
                self.ensure_worker_runtime_registered(&activated)?;
                self.renew_worker_runtime_leases(&activated)?;
                activated.release_start_gate(&self.config.state_dir)?;
                mgr.validate_live_worker_process(observed).await?;
                report.actions_taken.push(format!(
                    "Worker runtime {} recovered across Prepared/Started crash window; exact process activated and gate released",
                    runtime.session().session
                ));
            }
            crate::worker_runtime::WorkerRuntimeReconcileState::StartedRunning => {
                let started = runtime
                    .started()
                    .ok_or_else(|| anyhow::anyhow!("started runtime lost activation evidence"))?;
                let observed = observed.ok_or_else(|| {
                    anyhow::anyhow!("started runtime lost its exact live observation")
                })?;
                if observed != &started.observed {
                    anyhow::bail!("started runtime observation changed generation");
                }
                mgr.validate_live_worker_process(&started.observed).await?;
                let (_, authority) = self.load_worker_runtime_authority(runtime)?;
                if !matches!(
                    authority.attempt_state,
                    crate::mission::TaskAttemptState::Running
                        | crate::mission::TaskAttemptState::CandidateDone
                        | crate::mission::TaskAttemptState::Verifying
                        | crate::mission::TaskAttemptState::Accepted
                        | crate::mission::TaskAttemptState::Failed
                        | crate::mission::TaskAttemptState::Blocked
                ) {
                    anyhow::bail!(
                        "started runtime carries incompatible {:?} attempt authority",
                        authority.attempt_state
                    );
                }
                self.ensure_worker_runtime_registered(runtime)?;
                self.renew_worker_runtime_leases(runtime)?;
                if !runtime.is_start_gate_released(&self.config.state_dir)? {
                    if authority.attempt_state != crate::mission::TaskAttemptState::Running {
                        anyhow::bail!(
                            "unreleased start gate cannot be recovered after candidate/terminal transition"
                        );
                    }
                    runtime.release_start_gate(&self.config.state_dir)?;
                    mgr.validate_live_worker_process(&started.observed).await?;
                }
                match self.load_worker_runtime_candidate_evidence(runtime)? {
                    WorkerRuntimeCandidateEvidence::None => {
                        if authority.attempt_state != crate::mission::TaskAttemptState::Running {
                            anyhow::bail!(
                                "started runtime has {:?} authority without one exact candidate",
                                authority.attempt_state
                            );
                        }
                        report.actions_taken.push(format!(
                            "Worker runtime {} exact process validated and leases renewed",
                            runtime.session().session
                        ));
                    }
                    WorkerRuntimeCandidateEvidence::MarkerOnly => {
                        if authority.attempt_state != crate::mission::TaskAttemptState::Running {
                            anyhow::bail!(
                                "marker-only runtime has non-Running {:?} authority",
                                authority.attempt_state
                            );
                        }
                        report.actions_taken.push(format!(
                            "Worker runtime {} has marker-only completion evidence; no acceptance performed, process remains supervised",
                            runtime.session().session
                        ));
                    }
                    WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => {
                        // Freeze the checkout before verifier execution. The
                        // stable pane is closed first; the generation-scoped
                        // aggregate is then re-probed to mint durable absence.
                        mgr.contain_worker_process(&started.observed).await?;
                        let absence = mgr.contain_worker_runtime_session(&runtime).await?;
                        let retired = self.settle_worker_runtime_candidate_after_absence(
                            &runtime, &signal, &absence,
                        )?;
                        report.actions_taken.push(format!(
                            "Worker runtime {} candidate settled after exact process absence{}",
                            runtime.session().session,
                            if retired {
                                "; terminal evidence archived"
                            } else {
                                "; Blocked evidence retained"
                            }
                        ));
                    }
                }
            }
            crate::worker_runtime::WorkerRuntimeReconcileState::StartedSessionMissing => {
                let absence = mgr.confirm_worker_runtime_absence(runtime).await?;
                match self.load_worker_runtime_candidate_evidence(runtime)? {
                    WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => {
                        let retired = self.settle_worker_runtime_candidate_after_absence(
                            &runtime, &signal, &absence,
                        )?;
                        report.actions_taken.push(format!(
                            "Worker runtime {} missing session reconciled from exact immutable candidate{}",
                            runtime.session().session,
                            if retired { "; terminal evidence archived" } else { "; Blocked evidence retained" }
                        ));
                    }
                    WorkerRuntimeCandidateEvidence::None
                    | WorkerRuntimeCandidateEvidence::MarkerOnly => {
                        self.reconcile_worker_runtime_authority_after_absence(
                            runtime, &absence, false,
                        )?;
                        self.update_worker_runtime_registry_status(
                            runtime,
                            WorkerEntryStatus::Failed,
                        )?;
                        if !self.retire_worker_runtime_if_terminal(runtime, &absence)? {
                            anyhow::bail!(
                                "missing worker without exact candidate did not terminalize"
                            );
                        }
                        report.actions_taken.push(format!(
                            "Worker runtime {} disappeared without an exact candidate; marker ignored, authority Cancelled, evidence archived",
                            runtime.session().session
                        ));
                    }
                }
            }
            crate::worker_runtime::WorkerRuntimeReconcileState::ProcessGenerationMismatch
            | crate::worker_runtime::WorkerRuntimeReconcileState::SessionCollision => {
                let absence = mgr.contain_worker_runtime_session(runtime).await?;
                self.reconcile_worker_runtime_authority_after_absence(runtime, &absence, true)?;
                self.update_worker_runtime_registry_status(runtime, WorkerEntryStatus::Blocked)?;
                report.actions_taken.push(format!(
                    "Worker runtime {} {:?} contained; marker/candidate not accepted, Blocked authority retained",
                    runtime.session().session,
                    state
                ));
            }
        }
        Ok(())
    }

    async fn process_worker_runtimes(
        &self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<WorkerRuntimePatrolOutcome> {
        let inventory =
            crate::worker_runtime::WorkerRuntimeManifest::list_strict(&self.config.state_dir)?;
        let mut outcome = WorkerRuntimePatrolOutcome {
            managed_sessions: inventory
                .manifests
                .iter()
                .map(|runtime| runtime.session().session.clone())
                .collect(),
            inventory_compromised: worker_runtime_inventory_is_compromised(&inventory),
        };
        for corrupt in &inventory.corrupt_entries {
            report.actions_taken.push(format!(
                "Worker runtime inventory failed closed: corrupt {} ({})",
                corrupt.filename, corrupt.error
            ));
        }
        for session in &inventory.duplicate_sessions {
            outcome.managed_sessions.insert(session.clone());
            report.actions_taken.push(format!(
                "Worker runtime inventory failed closed: duplicate session {session}"
            ));
        }
        if outcome.inventory_compromised {
            // Do not even build an observational adoption set while the
            // namespace is ambiguous. Containment is the only legal mutation;
            // every valid sibling is handled independently and all generic
            // Worker paths remain disabled for the tick.
            for runtime in &inventory.manifests {
                self.contain_worker_runtime_failure(
                    mgr,
                    runtime,
                    "global worker runtime inventory is corrupt or ambiguous",
                    report,
                )
                .await;
            }
            return Ok(outcome);
        }

        let live_names = sessions
            .iter()
            .map(|session| session.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        let mut observed = Vec::new();
        let mut observation_errors = std::collections::HashMap::new();
        for runtime in &inventory.manifests {
            if !live_names.contains(runtime.session().session.as_str()) {
                continue;
            }
            let observation = match mgr.get_session(&runtime.session().session).await {
                Ok(session) => {
                    mgr.observe_worker_process(
                        &session,
                        &runtime.session().session,
                        runtime.workspace(),
                    )
                    .await
                }
                Err(error) => Err(error),
            };
            match observation {
                Ok(process) => observed.push(process),
                Err(error) => {
                    observation_errors
                        .insert(runtime.runtime_id().to_string(), format!("{error:#}"));
                }
            }
        }
        let reconciliation =
            crate::worker_runtime::reconcile_worker_runtimes(&self.config.state_dir, &observed)?;
        if !reconciliation.corrupt_entries.is_empty()
            || !reconciliation.duplicate_sessions.is_empty()
        {
            outcome.inventory_compromised = true;
        }
        for unbound in &reconciliation.unbound_observations {
            report.actions_taken.push(format!(
                "Unbound worker observation {} was not adopted; no runtime authority exists",
                unbound.session
            ));
        }

        for entry in reconciliation.entries {
            outcome.managed_sessions.insert(entry.session.clone());
            let Some(runtime) = crate::worker_runtime::WorkerRuntimeManifest::load_strict(
                &self.config.state_dir,
                &entry.runtime_id,
            )?
            else {
                if crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
                    &self.config.state_dir,
                    &entry.runtime_id,
                )?
                .is_none()
                {
                    report.actions_taken.push(format!(
                        "Worker runtime {} vanished without archive evidence",
                        entry.runtime_id
                    ));
                }
                continue;
            };
            if outcome.inventory_compromised {
                self.contain_worker_runtime_failure(
                    mgr,
                    &runtime,
                    "global worker runtime inventory is corrupt or ambiguous",
                    report,
                )
                .await;
                continue;
            }
            let exact_observation = observed
                .iter()
                .find(|process| process.session == entry.session);
            if let Err(error) = self
                .process_one_worker_runtime(
                    mgr,
                    &runtime,
                    entry.state,
                    exact_observation,
                    observation_errors
                        .get(&entry.runtime_id)
                        .map(String::as_str),
                    report,
                )
                .await
            {
                self.contain_worker_runtime_failure(mgr, &runtime, &format!("{error:#}"), report)
                    .await;
            }
        }
        Ok(outcome)
    }

    fn require_delivered_oracle_authority(
        &self,
        oracle_states: &[OracleState],
        oracle_name: &str,
        signal: &OracleDoneSignal,
    ) -> Result<OracleState> {
        let matches: Vec<_> = oracle_states
            .iter()
            .filter(|state| state.oracle_name == oracle_name)
            .collect();
        if matches.len() != 1 {
            anyhow::bail!(
                "expected one strict V3 OracleState for {oracle_name}, found {}",
                matches.len()
            );
        }
        let state = matches[0];
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        state.require_ledger_authority(&ledger)?;
        ledger
            .mission_record(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("immutable V3 mission is missing"))?;
        let projection = ledger
            .mission(&state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("V3 mission projection is missing"))?;
        if projection.state != crate::mission::MissionState::Delivered {
            anyhow::bail!("V3 mission is {:?}, not Delivered", projection.state);
        }
        ledger.validate_mission_acceptance(&state.mission_id)?;
        crate::orchestration::validate_oracle_done_signal_authority(
            &ledger,
            &state.mission_id,
            oracle_name,
            signal,
        )?;
        Ok(state.clone())
    }

    async fn process_team_runtimes(
        &self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<()> {
        let scan = crate::team::scan_team_runtime_manifests(&self.config.state_dir)?;
        for corrupt in scan.corrupt {
            report.actions_taken.push(format!(
                "Team runtime authority held: corrupt manifest {} ({})",
                corrupt.path.display(),
                corrupt.error
            ));
        }
        for manifest in scan.manifests {
            if let Err(error) = self
                .process_one_team_runtime(mgr, sessions, report, &manifest)
                .await
            {
                report.actions_taken.push(format!(
                    "Team {} held after isolated runtime failure ({error:#})",
                    manifest.aggregate_session
                ));
            }
        }
        Ok(())
    }

    async fn confirm_team_session_absent(
        &self,
        mgr: &SessionManager,
        aggregate_session: &str,
    ) -> Result<Vec<String>> {
        let before = mgr.list_sessions().await?;
        if before
            .iter()
            .any(|session| session.name == aggregate_session)
        {
            if let Err(kill_error) = mgr.kill_session(aggregate_session).await {
                let after_error = mgr.list_sessions().await?;
                if after_error
                    .iter()
                    .any(|session| session.name == aggregate_session)
                {
                    return Err(kill_error).with_context(|| {
                        format!("killing aggregate team session {aggregate_session}")
                    });
                }
                return Ok(after_error
                    .into_iter()
                    .map(|session| session.name)
                    .collect());
            }
        }
        let after = mgr.list_sessions().await?;
        if after
            .iter()
            .any(|session| session.name == aggregate_session)
        {
            anyhow::bail!(
                "aggregate team session {aggregate_session} remained live after containment"
            );
        }
        Ok(after.into_iter().map(|session| session.name).collect())
    }

    fn renew_team_runtime_leases(
        &self,
        ledger: &crate::mission_ledger::MissionLedger,
        manifest: &crate::team::TeamRuntimeManifest,
        status: &crate::team::TeamRuntimeStatus,
    ) -> Result<()> {
        for member in &manifest.members {
            let member_state = status
                .members
                .iter()
                .find(|candidate| candidate.owner == member.owner)
                .ok_or_else(|| anyhow::anyhow!("team member status disappeared"))?
                .state;
            if member_state.is_terminal() {
                continue;
            }
            let leases = ledger.active_leases_for_attempt(
                &manifest.mission_id,
                &member.task_id,
                &member.attempt_id,
            )?;
            if leases.len() != member.files_owned.len() {
                anyhow::bail!(
                    "team member {} lease cardinality changed before renewal",
                    member.owner
                );
            }
            for lease in leases {
                let renewed = ledger.renew_lease(
                    &lease.resource_key,
                    &member.owner,
                    lease.fencing_token,
                    Duration::from_secs(TEAM_LEASE_RENEW_TTL_SECS),
                )?;
                if renewed.mission_id != manifest.mission_id
                    || renewed.task_id != member.task_id
                    || renewed.attempt_id != member.attempt_id
                    || renewed.owner != member.owner
                    || renewed.fencing_token != lease.fencing_token
                {
                    anyhow::bail!("team member lease renewal changed immutable authority");
                }
            }
        }
        Ok(())
    }

    fn team_attempt(
        ledger: &crate::mission_ledger::MissionLedger,
        manifest: &crate::team::TeamRuntimeManifest,
        member: &crate::team::TeamRuntimeMember,
    ) -> Result<crate::orchestration::AuthoritativeTaskAttempt> {
        Ok(crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: manifest.mission_id.clone(),
            task_id: member.task_id.clone(),
            attempt_id: member.attempt_id.clone(),
            plan_revision: member.plan_revision,
            owner: Some(member.owner.clone()),
            leases: ledger.active_leases_for_attempt(
                &manifest.mission_id,
                &member.task_id,
                &member.attempt_id,
            )?,
            scope_receipt: None,
        })
    }

    fn block_team_mission(
        ledger: &crate::mission_ledger::MissionLedger,
        mission_id: &crate::mission::MissionId,
    ) -> Result<()> {
        let mission = ledger
            .mission(mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team mission disappeared"))?;
        if mission.state == crate::mission::MissionState::Running {
            crate::orchestration::transition_authoritative_mission(
                ledger,
                mission_id,
                crate::mission::MissionState::Verifying,
                "omega-team-patrol",
            )?;
        }
        let mission = ledger
            .mission(mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team mission disappeared"))?;
        if mission.state == crate::mission::MissionState::Verifying {
            crate::orchestration::transition_authoritative_mission(
                ledger,
                mission_id,
                crate::mission::MissionState::Blocked,
                "omega-team-patrol",
            )?;
        }
        Ok(())
    }

    fn block_team_member_after_pane_close_failure(
        ledger: &crate::mission_ledger::MissionLedger,
        manifest: &crate::team::TeamRuntimeManifest,
        member: &crate::team::TeamRuntimeMember,
    ) -> Result<()> {
        let attempt = Self::team_attempt(ledger, manifest, member)?;
        let current = ledger
            .task_attempt(&member.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
        match current.state {
            crate::mission::TaskAttemptState::CandidateDone => {
                crate::orchestration::transition_authoritative_attempt(
                    ledger,
                    &attempt,
                    crate::mission::TaskAttemptState::Verifying,
                    &member.owner,
                )?;
                crate::orchestration::transition_authoritative_attempt(
                    ledger,
                    &attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &member.owner,
                )?;
            }
            crate::mission::TaskAttemptState::Verifying => {
                crate::orchestration::transition_authoritative_attempt(
                    ledger,
                    &attempt,
                    crate::mission::TaskAttemptState::Blocked,
                    &member.owner,
                )?;
            }
            crate::mission::TaskAttemptState::Blocked => {}
            state => anyhow::bail!(
                "pane-close failure cannot block team member {} from {:?}",
                member.owner,
                state
            ),
        }
        Self::block_team_mission(ledger, &manifest.mission_id)
    }

    /// Settle a non-clean aggregate only after the caller has killed the rmux
    /// session and obtained a fresh absence snapshot. The immutable runtime
    /// manifest is deliberately retained as durable evidence; only cleanly
    /// Delivered teams may clear it from Patrol.
    fn reconcile_contained_nonclean_team(
        &self,
        manifest: &crate::team::TeamRuntimeManifest,
        live_sessions_after_kill: &[String],
    ) -> Result<crate::team::TeamRuntimeStatus> {
        if live_sessions_after_kill
            .iter()
            .any(|session| session == &manifest.aggregate_session)
        {
            anyhow::bail!(
                "non-clean team reconciliation refused while aggregate {} is live",
                manifest.aggregate_session
            );
        }
        let status = crate::team::reconcile_stopped_team(
            &self.config.state_dir,
            &manifest.aggregate_session,
            live_sessions_after_kill,
        )?;
        if !status.mission_state.is_terminal() || !status.all_terminal {
            anyhow::bail!(
                "non-clean team {} did not reach terminal ledger authority",
                manifest.aggregate_session
            );
        }

        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(&self.config.state_dir),
        )?;
        for member in &manifest.members {
            let attempt = ledger
                .task_attempt(&member.attempt_id)?
                .ok_or_else(|| anyhow::anyhow!("contained team member attempt disappeared"))?;
            if !attempt.state.is_terminal() {
                anyhow::bail!(
                    "contained team member {} retained non-terminal {:?} authority",
                    member.owner,
                    attempt.state
                );
            }
            if !ledger
                .active_leases_for_attempt(
                    &manifest.mission_id,
                    &member.task_id,
                    &member.attempt_id,
                )?
                .is_empty()
            {
                anyhow::bail!(
                    "contained team member {} retained active scope leases",
                    member.owner
                );
            }
            if crate::scope::ScopeClaim::read_strict(&self.config.state_dir, &member.owner)?
                .is_some()
            {
                anyhow::bail!(
                    "contained team member {} retained compatibility scope authority",
                    member.owner
                );
            }
        }
        let retained = crate::team::load_team_runtime_manifest(
            &self.config.state_dir,
            &manifest.aggregate_session,
            &manifest.mission_id,
        )?;
        if retained.as_ref() != Some(manifest) {
            anyhow::bail!(
                "non-clean team {} lost its immutable runtime evidence",
                manifest.aggregate_session
            );
        }
        Ok(status)
    }

    async fn process_one_team_runtime(
        &self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
        manifest: &crate::team::TeamRuntimeManifest,
    ) -> Result<()> {
        let aggregate_live = sessions
            .iter()
            .any(|session| session.name == manifest.aggregate_session);
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(&self.config.state_dir),
        )?;
        let mut status = manifest.validate_against_ledger(&ledger, &self.config.state_dir)?;

        if status.mission_state == crate::mission::MissionState::Delivered {
            ledger.validate_mission_acceptance(&manifest.mission_id)?;
            let _ = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            crate::team::clear_team_runtime_manifest(&self.config.state_dir, manifest, false)?;
            report.actions_taken.push(format!(
                "Team {} Delivered: exact aggregate absence confirmed and manifest cleared",
                manifest.aggregate_session
            ));
            return Ok(());
        }

        if !status.started || !status.start_barrier_released {
            let live_after = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            status = crate::team::reconcile_stopped_team(
                &self.config.state_dir,
                &manifest.aggregate_session,
                &live_after,
            )?;
            if status.mission_state.is_terminal() && status.all_terminal {
                crate::team::clear_team_runtime_manifest(&self.config.state_dir, manifest, false)?;
            }
            report.actions_taken.push(format!(
                "Team {} incomplete spawn reconciled after exact aggregate absence",
                manifest.aggregate_session
            ));
            return Ok(());
        }

        if status.mission_state == crate::mission::MissionState::Blocked {
            let live_after = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            status = self.reconcile_contained_nonclean_team(manifest, &live_after)?;
            report.actions_taken.push(format!(
                "Team {} resumed durable Blocked containment; aggregate absence confirmed, all authority reconciled in {:?}, runtime evidence retained",
                manifest.aggregate_session, status.mission_state
            ));
            return Ok(());
        }

        if !matches!(
            status.mission_state,
            crate::mission::MissionState::Running | crate::mission::MissionState::Verifying
        ) {
            report.actions_taken.push(format!(
                "Team {} held in {:?}; runtime authority retained",
                manifest.aggregate_session, status.mission_state
            ));
            return Ok(());
        }

        if !aggregate_live {
            let live_after = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            status = crate::team::reconcile_stopped_team(
                &self.config.state_dir,
                &manifest.aggregate_session,
                &live_after,
            )?;
            if status.mission_state.is_terminal() && status.all_terminal {
                crate::team::clear_team_runtime_manifest(&self.config.state_dir, manifest, false)?;
            }
            report.actions_taken.push(format!(
                "Team {} crashed with unfinished authority and was exactly reconciled",
                manifest.aggregate_session
            ));
            return Ok(());
        }

        let acknowledgement = status
            .started_ack
            .clone()
            .ok_or_else(|| anyhow::anyhow!("started team has no immutable activation ack"))?;
        if let Err(error) =
            crate::team::validate_live_team_active_members(mgr, manifest, &status).await
        {
            Self::block_team_mission(&ledger, &manifest.mission_id)?;
            let live_after = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            let reconciled = self.reconcile_contained_nonclean_team(manifest, &live_after)?;
            report.actions_taken.push(format!(
                "Team {} activation validation failed; aggregate contained, sibling authority reconciled in {:?}, runtime evidence retained ({error})",
                manifest.aggregate_session, reconciled.mission_state
            ));
            return Ok(());
        }
        self.renew_team_runtime_leases(&ledger, manifest, &status)?;

        let mut results = Vec::new();
        let mut failed_member = false;
        for member in &manifest.members {
            let member_state = status
                .members
                .iter()
                .find(|candidate| candidate.owner == member.owner)
                .ok_or_else(|| anyhow::anyhow!("team member status disappeared"))?
                .state;
            let mut candidate = crate::team::load_team_member_candidate_for_manifest(
                &self.config.state_dir,
                manifest,
                &member.owner,
            )?;
            if member_state == crate::mission::TaskAttemptState::Running {
                if let Some(signal) = DoneSignal::read(&self.config.state_dir, &member.owner)? {
                    if signal.finished_at < manifest.created_at {
                        report.actions_taken.push(format!(
                            "Team member {} stale completion predates runtime generation; signal ignored and authority retained",
                            member.owner
                        ));
                        continue;
                    }
                    if signal.projection.is_some() {
                        anyhow::bail!(
                            "team member {} has projected signal while ledger attempt is Running",
                            member.owner
                        );
                    }
                    candidate = crate::team::record_team_member_candidate(
                        &self.config.state_dir,
                        &signal,
                        &manifest.provider,
                    )?;
                }
            }
            if member_state == crate::mission::TaskAttemptState::CandidateDone
                && candidate.is_none()
            {
                anyhow::bail!(
                    "team member {} is CandidateDone without one immutable candidate",
                    member.owner
                );
            }
            let Some(candidate) = candidate else {
                continue;
            };
            if candidate.signal.finished_at < manifest.created_at {
                anyhow::bail!(
                    "team member {} immutable candidate predates its runtime generation",
                    member.owner
                );
            }
            match DoneSignal::read(&self.config.state_dir, &member.owner)? {
                Some(recorded)
                    if serde_json::to_value(&recorded)?
                        != serde_json::to_value(&candidate.signal)? =>
                {
                    anyhow::bail!(
                        "team member {} completion file differs from immutable candidate",
                        member.owner
                    );
                }
                Some(_) => {}
                None => candidate
                    .signal
                    .write(&self.config.state_dir)
                    .with_context(|| {
                        format!(
                            "recovering exact CandidateDone signal for team member {}",
                            member.owner
                        )
                    })?,
            }

            let current = ledger
                .task_attempt(&member.attempt_id)?
                .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
            if current.state == crate::mission::TaskAttemptState::CandidateDone {
                if let Err(error) = crate::team::close_activated_team_member_pane(
                    mgr,
                    manifest,
                    &acknowledgement,
                    &member.owner,
                )
                .await
                {
                    Self::block_team_member_after_pane_close_failure(&ledger, manifest, member)?;
                    let live_after = self
                        .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                        .await?;
                    let reconciled =
                        self.reconcile_contained_nonclean_team(manifest, &live_after)?;
                    report.actions_taken.push(format!(
                        "Team member {} pane containment validation failed; aggregate contained, sibling authority reconciled in {:?}, runtime evidence retained ({error})",
                        member.owner, reconciled.mission_state
                    ));
                    return Ok(());
                }
                match candidate.signal.status {
                    DoneStatus::DoneClean => {
                        let outcome = crate::orchestration::verify_and_finalize_candidate(
                            &ledger,
                            &manifest.mission_id,
                            &member.task_id,
                            &member.attempt_id,
                            member.plan_revision,
                            &member.owner,
                            &candidate.signal,
                            &manifest.working_dir,
                        )?;
                        if outcome.attempt_state == crate::mission::TaskAttemptState::Accepted {
                            let attempt = Self::team_attempt(&ledger, manifest, member)?;
                            crate::orchestration::release_authoritative_scopes(
                                &ledger,
                                &self.config.state_dir,
                                &attempt,
                            )?;
                        }
                    }
                    DoneStatus::Failed => {
                        crate::orchestration::finalize_nonclean_candidate(
                            &ledger,
                            &manifest.mission_id,
                            &member.task_id,
                            &member.attempt_id,
                            member.plan_revision,
                            &member.owner,
                            &candidate.signal,
                            crate::mission::TaskAttemptState::Failed,
                        )?;
                        failed_member = true;
                    }
                    DoneStatus::Blocked | DoneStatus::Pending => {
                        crate::orchestration::finalize_nonclean_candidate(
                            &ledger,
                            &manifest.mission_id,
                            &member.task_id,
                            &member.attempt_id,
                            member.plan_revision,
                            &member.owner,
                            &candidate.signal,
                            crate::mission::TaskAttemptState::Blocked,
                        )?;
                        Self::block_team_mission(&ledger, &manifest.mission_id)?;
                        let live_after = self
                            .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                            .await?;
                        let status =
                            self.reconcile_contained_nonclean_team(manifest, &live_after)?;
                        report.actions_taken.push(format!(
                            "Team member {} is non-clean; aggregate contained, every member authority terminalized, scopes released, mission {:?}, runtime evidence retained",
                            member.owner, status.mission_state
                        ));
                        return Ok(());
                    }
                }
            }
            let current = ledger
                .task_attempt(&member.attempt_id)?
                .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
            if current.state == crate::mission::TaskAttemptState::Accepted {
                results.push(crate::mission::WorkerResult {
                    task_id: member.task_id.clone(),
                    session_name: member.owner.clone(),
                    status: DoneStatus::DoneClean,
                    summary: candidate.signal.summary.clone(),
                    commit: candidate.signal.commit.clone(),
                    duration_secs: (candidate.signal.finished_at - manifest.created_at)
                        .num_seconds()
                        .max(0) as u64,
                });
            } else if current.state == crate::mission::TaskAttemptState::Failed {
                failed_member = true;
            }
        }

        if failed_member {
            let live_after = self
                .confirm_team_session_absent(mgr, &manifest.aggregate_session)
                .await?;
            status = self.reconcile_contained_nonclean_team(manifest, &live_after)?;
            report.actions_taken.push(format!(
                "Team {} failed member propagated; aggregate contained, authority reconciled in {:?}, runtime evidence retained",
                manifest.aggregate_session, status.mission_state
            ));
            return Ok(());
        }

        status = manifest.validate_against_ledger(&ledger, &self.config.state_dir)?;
        if !status.all_accepted {
            return Ok(());
        }
        if results.len() != manifest.members.len() {
            anyhow::bail!(
                "team {} has all Accepted projections but incomplete immutable worker results",
                manifest.aggregate_session
            );
        }
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &manifest.mission_id,
            crate::mission::MissionState::Verifying,
            "omega-team-patrol",
        )?;
        let mission = ledger
            .mission_record(&manifest.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team mission contract disappeared"))?;
        let plan = ledger
            .active_plan(&manifest.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team active plan disappeared"))?;
        let rubric = crate::orchestration::build_authoritative_rubric(&mission, &plan);
        let gate = crate::orchestration::Orchestrator::run_quality_gate(
            &self.config.state_dir,
            &ledger,
            &mission,
            &plan,
            &rubric,
            &results,
        )?;
        if !gate.overall_pass {
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &manifest.mission_id,
                crate::mission::MissionState::CorrectionRequired,
                "omega-team-patrol",
            )?;
            report.actions_taken.push(format!(
                "Team {} failed the authoritative quality gate; manifest retained",
                manifest.aggregate_session
            ));
            return Ok(());
        }
        for target in [
            crate::mission::MissionState::Accepted,
            crate::mission::MissionState::Reporting,
            crate::mission::MissionState::Delivered,
        ] {
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &manifest.mission_id,
                target,
                "omega-team-patrol",
            )?;
        }
        ledger.validate_mission_acceptance(&manifest.mission_id)?;
        let _ = self
            .confirm_team_session_absent(mgr, &manifest.aggregate_session)
            .await?;
        crate::team::clear_team_runtime_manifest(&self.config.state_dir, manifest, false)?;
        report.actions_taken.push(format!(
            "Team {} independently verified, gated, Delivered and exactly reaped",
            manifest.aggregate_session
        ));
        Ok(())
    }

    #[allow(dead_code)]
    async fn process_team_runtimes_legacy(
        &self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        report: &mut PatrolReport,
    ) -> Result<()> {
        for manifest in crate::team::list_team_runtime_manifests(&self.config.state_dir)? {
            let aggregate_live = sessions
                .iter()
                .any(|session| session.name == manifest.aggregate_session);
            let ledger = crate::mission_ledger::MissionLedger::open(
                crate::oracle_lifecycle::mission_ledger_path(&self.config.state_dir),
            )?;
            let mut status = manifest.validate_against_ledger(&ledger, &self.config.state_dir)?;

            if status.mission_state == crate::mission::MissionState::Delivered {
                ledger.validate_mission_acceptance(&manifest.mission_id)?;
                if aggregate_live {
                    if let Err(error) = mgr.kill_session(&manifest.aggregate_session).await {
                        report.actions_taken.push(format!(
                            "Team {} Delivered but aggregate reap failed ({error}); manifest retained",
                            manifest.aggregate_session
                        ));
                        continue;
                    }
                }
                crate::team::clear_team_runtime_manifest(&self.config.state_dir, &manifest, false)?;
                report.actions_taken.push(format!(
                    "Team {} Delivered: aggregate closed and exact runtime manifest cleared",
                    manifest.aggregate_session
                ));
                continue;
            }
            if !matches!(
                status.mission_state,
                crate::mission::MissionState::Running | crate::mission::MissionState::Verifying
            ) {
                report.actions_taken.push(format!(
                    "Team {} held in {:?}; no completion or reap mutation performed",
                    manifest.aggregate_session, status.mission_state
                ));
                continue;
            }

            let mut results = Vec::new();
            for member in &manifest.members {
                let member_state = status
                    .members
                    .iter()
                    .find(|candidate| candidate.owner == member.owner)
                    .ok_or_else(|| {
                        anyhow::anyhow!("team member {} vanished from strict status", member.owner)
                    })?
                    .state;
                let mut candidate = crate::team::load_team_member_candidate_for_manifest(
                    &self.config.state_dir,
                    &manifest,
                    &member.owner,
                )?;
                if member_state == crate::mission::TaskAttemptState::Running {
                    if let Some(signal) = DoneSignal::read(&self.config.state_dir, &member.owner)? {
                        if signal.projection.is_some() {
                            anyhow::bail!(
                                "team member {} has projected signal while ledger attempt is Running",
                                member.owner
                            );
                        }
                        candidate = crate::team::record_team_member_candidate(
                            &self.config.state_dir,
                            &signal,
                            &manifest.provider,
                        )?;
                    }
                }
                if member_state == crate::mission::TaskAttemptState::CandidateDone
                    && candidate.is_none()
                {
                    anyhow::bail!(
                        "team member {} is CandidateDone without one immutable candidate",
                        member.owner
                    );
                }
                let Some(candidate) = candidate else {
                    continue;
                };
                match DoneSignal::read(&self.config.state_dir, &member.owner)? {
                    Some(recorded)
                        if serde_json::to_value(&recorded)?
                            != serde_json::to_value(&candidate.signal)? =>
                    {
                        anyhow::bail!(
                            "team member {} completion file differs from immutable candidate",
                            member.owner
                        );
                    }
                    Some(_) => {}
                    None => candidate
                        .signal
                        .write(&self.config.state_dir)
                        .with_context(|| {
                            format!(
                                "recovering exact CandidateDone signal for team member {}",
                                member.owner
                            )
                        })?,
                }

                let current = ledger
                    .task_attempt(&member.attempt_id)?
                    .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
                if current.state == crate::mission::TaskAttemptState::CandidateDone {
                    match candidate.signal.status {
                        DoneStatus::DoneClean => {
                            let outcome = crate::orchestration::verify_and_finalize_candidate(
                                &ledger,
                                &manifest.mission_id,
                                &member.task_id,
                                &member.attempt_id,
                                member.plan_revision,
                                &member.owner,
                                &candidate.signal,
                                &manifest.working_dir,
                            )?;
                            if outcome.attempt_state == crate::mission::TaskAttemptState::Accepted {
                                let attempt = crate::orchestration::AuthoritativeTaskAttempt {
                                    mission_id: manifest.mission_id.clone(),
                                    task_id: member.task_id.clone(),
                                    attempt_id: member.attempt_id.clone(),
                                    plan_revision: member.plan_revision,
                                    owner: Some(member.owner.clone()),
                                    leases: ledger.active_leases_for_attempt(
                                        &manifest.mission_id,
                                        &member.task_id,
                                        &member.attempt_id,
                                    )?,
                                    scope_receipt: None,
                                };
                                crate::orchestration::release_authoritative_scopes(
                                    &ledger,
                                    &self.config.state_dir,
                                    &attempt,
                                )?;
                            }
                        }
                        DoneStatus::Failed => {
                            crate::orchestration::finalize_nonclean_candidate(
                                &ledger,
                                &manifest.mission_id,
                                &member.task_id,
                                &member.attempt_id,
                                member.plan_revision,
                                &member.owner,
                                &candidate.signal,
                                crate::mission::TaskAttemptState::Failed,
                            )?;
                        }
                        DoneStatus::Blocked | DoneStatus::Pending => {
                            crate::orchestration::finalize_nonclean_candidate(
                                &ledger,
                                &manifest.mission_id,
                                &member.task_id,
                                &member.attempt_id,
                                member.plan_revision,
                                &member.owner,
                                &candidate.signal,
                                crate::mission::TaskAttemptState::Blocked,
                            )?;
                        }
                    }
                }
                let current = ledger
                    .task_attempt(&member.attempt_id)?
                    .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
                if current.state == crate::mission::TaskAttemptState::Accepted {
                    results.push(crate::mission::WorkerResult {
                        task_id: member.task_id.clone(),
                        session_name: member.owner.clone(),
                        status: DoneStatus::DoneClean,
                        summary: candidate.signal.summary.clone(),
                        commit: candidate.signal.commit.clone(),
                        duration_secs: (candidate.signal.finished_at - manifest.created_at)
                            .num_seconds()
                            .max(0) as u64,
                    });
                }
            }

            status = manifest.validate_against_ledger(&ledger, &self.config.state_dir)?;
            if !status.all_accepted {
                if !aggregate_live {
                    report.actions_taken.push(format!(
                        "Team {} aggregate is absent with unfinished members; virtual scope authority retained",
                        manifest.aggregate_session
                    ));
                }
                continue;
            }
            if results.len() != manifest.members.len() {
                anyhow::bail!(
                    "team {} has all Accepted projections but incomplete immutable worker results",
                    manifest.aggregate_session
                );
            }
            crate::orchestration::transition_authoritative_mission(
                &ledger,
                &manifest.mission_id,
                crate::mission::MissionState::Verifying,
                "omega-team-patrol",
            )?;
            let mission = ledger
                .mission_record(&manifest.mission_id)?
                .ok_or_else(|| anyhow::anyhow!("team mission contract disappeared"))?;
            let plan = ledger
                .active_plan(&manifest.mission_id)?
                .ok_or_else(|| anyhow::anyhow!("team active plan disappeared"))?;
            let rubric = crate::orchestration::build_authoritative_rubric(&mission, &plan);
            let gate = crate::orchestration::Orchestrator::run_quality_gate(
                &self.config.state_dir,
                &ledger,
                &mission,
                &plan,
                &rubric,
                &results,
            )?;
            if !gate.overall_pass {
                crate::orchestration::transition_authoritative_mission(
                    &ledger,
                    &manifest.mission_id,
                    crate::mission::MissionState::CorrectionRequired,
                    "omega-team-patrol",
                )?;
                report.actions_taken.push(format!(
                    "Team {} failed the authoritative quality gate; aggregate and manifest retained",
                    manifest.aggregate_session
                ));
                continue;
            }
            for target in [
                crate::mission::MissionState::Accepted,
                crate::mission::MissionState::Reporting,
                crate::mission::MissionState::Delivered,
            ] {
                crate::orchestration::transition_authoritative_mission(
                    &ledger,
                    &manifest.mission_id,
                    target,
                    "omega-team-patrol",
                )?;
            }
            ledger.validate_mission_acceptance(&manifest.mission_id)?;
            if aggregate_live {
                if let Err(error) = mgr.kill_session(&manifest.aggregate_session).await {
                    report.actions_taken.push(format!(
                        "Team {} Delivered but aggregate reap failed ({error}); manifest retained",
                        manifest.aggregate_session
                    ));
                    continue;
                }
            }
            crate::team::clear_team_runtime_manifest(&self.config.state_dir, &manifest, false)?;
            report.actions_taken.push(format!(
                "Team {} independently verified, gated, Delivered and reaped",
                manifest.aggregate_session
            ));
        }
        Ok(())
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
            oracles: sessions
                .iter()
                .filter(|s| s.role == SessionRole::Oracle)
                .count(),
            workers: sessions
                .iter()
                .filter(|s| s.role == SessionRole::Worker)
                .count(),
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

        // Typed runtimes own their rmux generations. Reconcile them before any
        // generic pane repair or legacy worker path can respawn, nudge, reap,
        // auto-complete, or independently finalize the same process.
        let team_scan = crate::team::scan_team_runtime_manifests(&self.config.state_dir)?;
        let team_runtime_inventory_compromised = !team_scan.corrupt.is_empty();
        let mut typed_runtime_sessions = team_scan
            .manifests
            .iter()
            .map(|manifest| manifest.aggregate_session.clone())
            .collect::<std::collections::HashSet<_>>();
        let mut worker_runtime = self
            .process_worker_runtimes(&mgr, &sessions, &mut report)
            .await?;
        typed_runtime_sessions.extend(worker_runtime.managed_sessions.iter().cloned());
        if team_runtime_inventory_compromised {
            // A corrupt aggregate may hide its exact session identity. Refuse
            // every generic Worker mutation for this tick instead of guessing.
            worker_runtime.inventory_compromised = true;
        }
        self.process_team_runtimes(&mgr, &sessions, &mut report)
            .await?;

        // Read every oracle's persisted state ONCE per tick. find_parent_oracle
        // needs it to resolve a worker -> its governing oracle, and it's called
        // once per signaling worker; reading it per call was an O(W×O) disk scan +
        // JSON parse every tick. Compute it here and pass the slice down.
        let oracle_states =
            crate::oracle_lifecycle::OracleState::read_all_strict(&self.config.state_dir)?;

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
            if typed_runtime_protects_session(
                session,
                &typed_runtime_sessions,
                worker_runtime.inventory_compromised,
                team_runtime_inventory_compromised,
            ) {
                report.actions_taken.push(format!(
                    "Typed runtime protected {} from generic broken-pane repair",
                    session.name
                ));
                continue;
            }
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
                let configured = session
                    .provider
                    .as_deref()
                    .unwrap_or(self.config.agent_command.as_str());
                let Some(agent) = crate::agents::Agent::from_name(configured) else {
                    report.actions_taken.push(format!(
                        "Session {}: pane respawned but agent relaunch REFUSED; unknown configured provider {:?}",
                        session.name, configured
                    ));
                    continue;
                };
                let working_dir = session
                    .working_dir
                    .as_deref()
                    .and_then(std::path::Path::to_str);
                if let Err(error) = mgr
                    .relaunch_agent_session_with_opts(
                        &session.name,
                        working_dir,
                        agent,
                        None,
                        crate::agents::LaunchOptions {
                            resume_conversation: true,
                            ..Default::default()
                        },
                    )
                    .await
                {
                    report.actions_taken.push(format!(
                        "Session {}: pane respawned but typed agent relaunch FAILED ({error})",
                        session.name
                    ));
                    continue;
                }
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
            if session.role == SessionRole::Worker
                && !worker_runtime.inventory_compromised
                && !typed_runtime_sessions.contains(&session.name)
            {
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

                    // A worker's narration is only a CandidateDone. Patrol uses
                    // the same exact-plan independent finalizer as native
                    // orchestration; no compatibility-only artifact path may
                    // write Accepted.
                    let repo_root = crate::session::OmegaSession::classify(&session.name)
                        .project
                        .as_deref()
                        .and_then(|p| resolve_repo_root(&self.config, p));
                    let mut effective_status = done.status;
                    let mut contest_reason: Option<String> = None;
                    let mut accepted_persisted = false;
                    let v3_worker = strict_worker_binding(&oracle_states, &session.name)?;
                    if done.status == DoneStatus::DoneClean {
                        match (v3_worker, repo_root.as_deref()) {
                            (Some((oracle, worker)), Some(root)) => {
                                let attempt_id = worker.attempt_id.as_deref();
                                let plan_revision = worker.plan_revision;
                                let result = match (attempt_id, plan_revision) {
                                    (Some(attempt_id), Some(plan_revision)) => {
                                        let ledger = crate::mission_ledger::MissionLedger::open(
                                            self.config.state_dir.join("mission-engine-v3.sqlite3"),
                                        );
                                        ledger.map_err(anyhow::Error::from).and_then(|ledger| {
                                            crate::orchestration::verify_and_finalize_candidate(
                                                &ledger,
                                                &oracle.mission_id,
                                                &worker.task_id,
                                                attempt_id,
                                                plan_revision,
                                                &session.name,
                                                &done,
                                                root,
                                            )
                                        })
                                    }
                                    _ => Err(anyhow::anyhow!(
                                        "worker registry has no exact V3 attempt binding"
                                    )),
                                };
                                match result {
                                    Ok(outcome)
                                        if outcome.accepted
                                            && outcome.attempt_state
                                                == crate::mission::TaskAttemptState::Accepted =>
                                    {
                                        match self
                                            .release_exact_accepted_worker_scopes(oracle, worker)
                                        {
                                            Ok(()) => {
                                                accepted_persisted = true;
                                                effective_status = DoneStatus::DoneClean;
                                            }
                                            Err(error) => {
                                                effective_status = DoneStatus::Pending;
                                                report.actions_taken.push(format!(
                                                    "{}: Accepted persisted but exact scope release failed closed ({error})",
                                                    session.name
                                                ));
                                            }
                                        }
                                    }
                                    Ok(outcome) => {
                                        effective_status = DoneStatus::Pending;
                                        let reason = if outcome.verification.failures.is_empty() {
                                            format!(
                                                "authoritative attempt settled as {:?}",
                                                outcome.attempt_state
                                            )
                                        } else {
                                            outcome.verification.failures.join("; ")
                                        };
                                        let contradicted = outcome
                                            .verification
                                            .observations
                                            .iter()
                                            .filter(|observation| !observation.passed)
                                            .map(|observation| observation.detail.as_str())
                                            .any(|detail| {
                                                detail.contains(" exited ")
                                                    || detail.contains("does NOT exist")
                                                    || detail.contains("returned ")
                                            });
                                        if contradicted {
                                            contest_reason = Some(reason.clone());
                                        }
                                        report.actions_taken.push(format!(
                                            "{}: candidate retained as pending; exact independent verification rejected ({reason})",
                                            session.name
                                        ));
                                    }
                                    Err(error) => {
                                        effective_status = DoneStatus::Pending;
                                        report.actions_taken.push(format!(
                                            "{}: candidate authority/verification failed closed; scope held ({error})",
                                            session.name
                                        ));
                                    }
                                }
                            }
                            (None, _) => {
                                effective_status = DoneStatus::Pending;
                                report.actions_taken.push(format!(
                                    "{}: no V3 worker binding; candidate remains pending and scope held",
                                    session.name
                                ));
                            }
                            (_, None) => {
                                effective_status = DoneStatus::Pending;
                                report.actions_taken.push(format!(
                                    "{}: project root is unresolved; candidate remains pending and scope held",
                                    session.name
                                ));
                            }
                        }
                    } else if let Some((oracle, worker)) = v3_worker {
                        let target = match done.status {
                            DoneStatus::Failed => crate::mission::TaskAttemptState::Failed,
                            DoneStatus::Blocked | DoneStatus::Pending => {
                                crate::mission::TaskAttemptState::Blocked
                            }
                            DoneStatus::DoneClean => unreachable!("handled above"),
                        };
                        let result = match (worker.attempt_id.as_deref(), worker.plan_revision) {
                            (Some(attempt_id), Some(plan_revision)) => {
                                let ledger = crate::mission_ledger::MissionLedger::open(
                                    self.config.state_dir.join("mission-engine-v3.sqlite3"),
                                );
                                ledger.map_err(anyhow::Error::from).and_then(|ledger| {
                                    oracle.require_ledger_authority(&ledger)?;
                                    crate::orchestration::finalize_nonclean_candidate(
                                        &ledger,
                                        &oracle.mission_id,
                                        &worker.task_id,
                                        attempt_id,
                                        plan_revision,
                                        &session.name,
                                        &done,
                                        target,
                                    )
                                })
                            }
                            _ => Err(anyhow::anyhow!(
                                "worker registry has no exact V3 attempt binding"
                            )),
                        };
                        if let Err(error) = result {
                            effective_status = DoneStatus::Pending;
                            report.actions_taken.push(format!(
                                "{}: non-clean authoritative transition failed; scope held ({error})",
                                session.name
                            ));
                        }
                    }

                    if let Some(oracle) =
                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                    {
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
                        // repeatedly. The marker key is a digest of the full
                        // signal plus its immutable attempt generation. This
                        // distinguishes signals written in the same second and
                        // prevents a recycled worker name from inheriting an
                        // acknowledgement from a prior attempt.
                        let event_key = worker_done_event_key(status_str, &done, v3_worker)?;
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "done",
                            &event_key,
                        )? {
                            let pushed = match inbox
                                .push(&InboxEvent::worker_done(&session.name, status_str))
                            {
                                Ok(()) => true,
                                Err(error) => {
                                    report.actions_taken.push(format!(
                                        "{}: worker_done inbox delivery failed; marker not advanced, retry retained ({error})",
                                        session.name
                                    ));
                                    false
                                }
                            };
                            // Surface the fabrication detail so the oracle can
                            // re-dispatch with eyes open.
                            if let Some(reason) = &contest_reason {
                                if let Err(error) = inbox.push(&InboxEvent::worker_blocked(
                                    &session.name,
                                    &format!("GROUND-TRUTH CONTEST: {}", reason),
                                )) {
                                    report.actions_taken.push(format!(
                                        "{}: contested worker_blocked inbox delivery failed; retry retained ({error})",
                                        session.name
                                    ));
                                }
                            }
                            // Record only on a successful push — a failed one
                            // must retry next tick, not be marked delivered.
                            if pushed {
                                record_inbox_event_sent(
                                    &self.config.state_dir,
                                    &session.name,
                                    "done",
                                    &event_key,
                                )?;
                            }
                        }

                        // Update oracle state with worker completion
                        let mut oracle_state =
                            OracleState::read(&self.config.state_dir, &oracle.name)?.ok_or_else(
                                || {
                                    anyhow::anyhow!(
                                        "parent oracle state {} disappeared during worker update",
                                        oracle.name
                                    )
                                },
                            )?;
                        let ws = match effective_status {
                            DoneStatus::DoneClean => WorkerEntryStatus::DoneClean,
                            DoneStatus::Pending => WorkerEntryStatus::Pending,
                            DoneStatus::Failed => WorkerEntryStatus::Failed,
                            DoneStatus::Blocked => WorkerEntryStatus::Blocked,
                        };
                        oracle_state.update_worker_status(&session.name, ws);
                        oracle_state
                            .write(&self.config.state_dir)
                            .with_context(|| {
                                format!(
                                    "persisting worker {} status in oracle {}",
                                    session.name, oracle.name
                                )
                            })?;
                    }

                    if let Some(reason) = contest_reason {
                        // Fabrication: keep the scope claim HELD (work is not
                        // actually done) and flag it loudly.
                        report.actions_taken.push(format!(
                            "CONTESTED {}: done_clean failed ground-truth — {}",
                            session.name, reason
                        ));
                        // Loop Engineering — bounded retries then escalate. A
                        // worker that re-writes a contested done_clean is
                        // thrashing; count consecutive contested attempts and at
                        // THRASH_CAP hand the loop to a human instead of letting
                        // it re-fabricate forever (L1's "3rd change → runtime
                        // evidence", enforced at the orchestration layer).
                        let thrash =
                            crate::loop_guard::bump_thrash(&self.config.state_dir, &session.name);
                        if let Some(oracle) =
                            self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                        {
                            crate::loop_guard::MissionLog::event(
                                &self.config.state_dir,
                                &oracle.name,
                                "contest",
                                &format!(
                                    "{} contested ({}/{}): {}",
                                    session.name,
                                    thrash,
                                    crate::loop_guard::THRASH_CAP,
                                    reason
                                ),
                            );
                            if thrash >= crate::loop_guard::THRASH_CAP
                                && crate::loop_guard::escalate_to_human(
                                    &self.config.state_dir,
                                    &oracle.name,
                                    crate::loop_guard::EscalationReason::ContestedFabrication,
                                    &format!(
                                        "worker {} contested {}× — {}",
                                        session.name, thrash, reason
                                    ),
                                )
                            {
                                report.actions_taken.push(format!(
                                    "ESCALATED TO HUMAN: {} thrashed {}× (contested fabrication)",
                                    session.name, thrash
                                ));
                            }
                        }
                    } else if accepted_persisted && effective_status == DoneStatus::DoneClean {
                        self.stall_detector.forget(&session.name);
                        // Clean, uncontested close → the loop converged; reset
                        // the worker's thrash counter so a future reuse of the
                        // name starts fresh.
                        crate::loop_guard::clear_thrash(&self.config.state_dir, &session.name);
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
                        )
                        .with_context(|| {
                            format!("persisting close authority for {}", session.name)
                        })?;
                        report.actions_taken.push(format!(
                            "Released scope for {} (ground-truth [+]); marked Closeable",
                            session.name
                        ));
                    }
                }

                // Check for blocked workers
                if let Ok(Some(blocked)) =
                    WorkerBlocked::read(&self.config.state_dir, &session.name)
                {
                    report.blocked_workers.push(session.name.clone());
                    if let Some(oracle) =
                        self.find_parent_oracle(&session.name, &oracle_sessions, &oracle_states)
                    {
                        let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                        // Same push-once contract as worker_done above: the
                        // blocked file persists across ticks, so an unguarded
                        // push re-delivered the question every minute. Keyed
                        // on blocked_at — a NEW block re-arms.
                        let bkey = worker_blocked_event_key(
                            &blocked,
                            strict_worker_binding(&oracle_states, &session.name)?,
                        )?;
                        if !inbox_event_already_sent(
                            &self.config.state_dir,
                            &session.name,
                            "blocked",
                            &bkey,
                        )? {
                            match inbox.push(&InboxEvent::worker_blocked(
                                &session.name,
                                &blocked.question,
                            )) {
                                Ok(()) => record_inbox_event_sent(
                                    &self.config.state_dir,
                                    &session.name,
                                    "blocked",
                                    &bkey,
                                )?,
                                Err(error) => report.actions_taken.push(format!(
                                    "{}: worker_blocked inbox delivery failed; marker not advanced, retry retained ({error})",
                                    session.name
                                )),
                            }
                        }
                    }
                }
            }
        }

        // ── Worker patrol: pane-based stall detection (30s nudge / 5min escalate) ──
        for session in &sessions {
            if session.role != SessionRole::Worker
                || worker_runtime.inventory_compromised
                || typed_runtime_sessions.contains(&session.name)
            {
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
                            let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                            let _ = inbox.push(&InboxEvent::worker_blocked(&session.name, reason));
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
                        StallAction::Nudge {
                            ref session,
                            idle_secs,
                        } => {
                            tracing::info!(worker = %session, idle_secs, "Worker idle — nudge");
                            // Send a nudge via the session pane
                            match mgr
                                .send_text(
                                    session,
                                    "You appear idle. Continue your mission or report done.",
                                )
                                .await
                            {
                                Ok(()) => report
                                    .actions_taken
                                    .push(format!("Nudged {} (idle {}s)", session, idle_secs)),
                                Err(error) => report.actions_taken.push(format!(
                                    "Nudge failed for {} (idle {}s): {}",
                                    session, idle_secs, error
                                )),
                            }
                        }
                        StallAction::Escalate {
                            ref session,
                            idle_secs,
                        } => {
                            report.stalled_workers.push(session.clone());
                            if let Some(oracle) =
                                self.find_parent_oracle(session, &oracle_sessions, &oracle_states)
                            {
                                let inbox = Inbox::for_oracle(&self.config.state_dir, &oracle.name);
                                let _ = inbox.push(&InboxEvent::worker_stalled(session, idle_secs));
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
            if session.role == SessionRole::Worker
                && !worker_runtime.inventory_compromised
                && !typed_runtime_sessions.contains(&session.name)
            {
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
                            if idle_secs > STALL_THRESHOLD_SECS
                                && !report.stalled_workers.contains(&session.name)
                            {
                                report.stalled_workers.push(session.name.clone());
                                if let Some(oracle) = self.find_parent_oracle(
                                    &session.name,
                                    &oracle_sessions,
                                    &oracle_states,
                                ) {
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
                            let session_gone = mgr.capture_pane(&session.name).await.is_err();
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
                                if !session_gone {
                                    if let Err(error) = mgr.kill_session(&session.name).await {
                                        report.actions_taken.push(format!(
                                            "Auto-done held {}: live session kill failed ({error}); no signal/state mutation performed",
                                            session.name
                                        ));
                                        continue;
                                    }
                                }
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
                                let mut signal =
                                    DoneSignal::new(&session.name, DoneStatus::Pending, reason);
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
                                        if let Some(oracle) = self.find_parent_oracle(
                                            &session.name,
                                            &oracle_sessions,
                                            &oracle_states,
                                        ) {
                                            let inbox = Inbox::for_oracle(
                                                &self.config.state_dir,
                                                &oracle.name,
                                            );
                                            // Mark the event sent under the SAME
                                            // key the main done pass will compute
                                            // for this signal next tick, so it
                                            // doesn't re-deliver it (only on a
                                            // successful push — a failure retries).
                                            match inbox.push(&InboxEvent::worker_done(
                                                &session.name,
                                                "pending",
                                            )) {
                                            Ok(()) => {
                                            let binding = strict_worker_binding(
                                                &oracle_states,
                                                &session.name,
                                            )?;
                                            let event_key = worker_done_event_key(
                                                "pending",
                                                &signal,
                                                binding,
                                            )?;
                                            record_inbox_event_sent(
                                                &self.config.state_dir,
                                                &session.name,
                                                "done",
                                                &event_key,
                                            )?;
                                            }
                                            Err(error) => report.actions_taken.push(format!(
                                                "{}: auto-done inbox delivery failed; marker not advanced, retry retained ({error})",
                                                session.name
                                            )),
                                        }
                                            let mut oracle_state = OracleState::read(
                                            &self.config.state_dir,
                                            &oracle.name,
                                        )?
                                        .ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "parent oracle state {} disappeared during auto-done update",
                                                oracle.name
                                            )
                                        })?;
                                            oracle_state.update_worker_status(
                                                &session.name,
                                                WorkerEntryStatus::Pending,
                                            );
                                            oracle_state.write(&self.config.state_dir).with_context(
                                            || {
                                                format!(
                                                    "persisting auto-done status for {} in oracle {}",
                                                    session.name, oracle.name
                                                )
                                            },
                                        )?;
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
        for marker in WorkerCloseMarker::read_all(&self.config.state_dir)? {
            if worker_runtime.inventory_compromised
                || typed_runtime_sessions.contains(&marker.session)
            {
                continue;
            }
            match self.v3_worker_attempt_state(&oracle_states, &marker.session) {
                Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                Ok(None) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: no exact V3 worker authority",
                        marker.session
                    ));
                    continue;
                }
                Ok(Some(state)) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: V3 attempt is {:?}, not Accepted",
                        marker.session, state
                    ));
                    continue;
                }
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Held close marker and scope for {}: V3 authority failed closed ({error})",
                        marker.session
                    ));
                    continue;
                }
            }
            // Oracle ack = the worker_done event was drained from its inbox.
            // peek() lists what's still queued; absence after we pushed it ⇒
            // the oracle consumed it (its drain deletes the file).
            let oracle_acked = match &marker.oracle {
                Some(oracle_name) => {
                    let inbox = Inbox::for_oracle(&self.config.state_dir, oracle_name);
                    let still_queued = match inbox.peek() {
                        Ok(events) => events.iter().any(|event| {
                            event.event_type == crate::inbox::EventType::WorkerDone
                                && event
                                    .payload
                                    .get("session")
                                    .and_then(|value| value.as_str())
                                    == Some(marker.session.as_str())
                        }),
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held close marker and scope for {}: oracle inbox {} could not be read ({error})",
                                marker.session, oracle_name
                            ));
                            continue;
                        }
                    };
                    !still_queued
                }
                // No known parent ⇒ rely solely on the grace window.
                None => false,
            };
            let closeable_secs = (Utc::now() - marker.since).num_seconds();
            if should_reap_closeable(oracle_acked, closeable_secs) {
                let scope_receipt =
                    match ScopeClaim::read_strict(&self.config.state_dir, &marker.session) {
                        Ok(receipt) => receipt,
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held {}: exact scope receipt could not be read ({error})",
                                marker.session
                            ));
                            continue;
                        }
                    };
                // Kill the rmux session (no-op/Err if already gone) and release
                // any remaining scope lock, atomically from patrol's view.
                if live_session_names.contains(marker.session.as_str()) {
                    if let Err(error) = mgr.kill_session(&marker.session).await {
                        report.actions_taken.push(format!(
                            "Held scope/marker for {}: session reap failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                if let Some(receipt) = &scope_receipt {
                    if let Err(error) = ScopeClaim::release_exact(&self.config.state_dir, receipt) {
                        report.actions_taken.push(format!(
                            "Reaped {} but retained close marker: exact scope release failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                self.stall_detector.forget(&marker.session);
                WorkerCloseMarker::remove(&self.config.state_dir, &marker.session)?;
                remove_inbox_event_markers(&self.config.state_dir, &marker.session)?;
                let trigger = if oracle_acked {
                    "oracle ack'd"
                } else {
                    "grace elapsed"
                };
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
                match ScopeClaim::read_strict(&self.config.state_dir, &marker.session) {
                    Ok(Some(receipt)) => {
                        if let Err(error) =
                            ScopeClaim::release_exact(&self.config.state_dir, &receipt)
                        {
                            report.actions_taken.push(format!(
                                "Held close marker for {}: exact scope release failed ({error})",
                                marker.session
                            ));
                            continue;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Held close marker for {}: scope receipt read failed ({error})",
                            marker.session
                        ));
                        continue;
                    }
                }
                WorkerCloseMarker::remove(&self.config.state_dir, &marker.session)?;
                remove_inbox_event_markers(&self.config.state_dir, &marker.session)?;
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
        for claim in ScopeClaim::read_all_strict(&self.config.state_dir)? {
            if worker_runtime.inventory_compromised
                || typed_runtime_sessions.contains(&claim.session)
            {
                continue;
            }
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
                match self.v3_worker_attempt_state(&oracle_states, &claim.session) {
                    Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                    Ok(None) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: no exact V3 worker authority",
                            claim.session
                        ));
                        continue;
                    }
                    Ok(Some(state)) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: V3 attempt is {:?}, not Accepted",
                            claim.session, state
                        ));
                        continue;
                    }
                    Err(error) => {
                        report.actions_taken.push(format!(
                            "Held scope of dead session {}: V3 authority failed closed ({error})",
                            claim.session
                        ));
                        continue;
                    }
                }
                match ScopeClaim::release_exact(&self.config.state_dir, &claim) {
                    Ok(()) => report.actions_taken.push(format!(
                        "Released scope of dead Accepted V3 session {}",
                        claim.session
                    )),
                    Err(error) => report.actions_taken.push(format!(
                        "Held scope of dead session {}: release failed ({error})",
                        claim.session
                    )),
                }
            }
        }

        // ── Orphan detection: sessions with no done/progress and empty pane ──
        for session in &sessions {
            if session.role == SessionRole::Worker
                && !worker_runtime.inventory_compromised
                && !typed_runtime_sessions.contains(&session.name)
            {
                let has_done = DoneSignal::read(&self.config.state_dir, &session.name)?.is_some();
                let has_progress =
                    crate::progress::ProgressInfo::read(&self.config.state_dir, &session.name)
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
        self.patrol_oracles(&mgr, &sessions, &oracle_states, &mut report)
            .await?;

        // ── Orphan-worker sweep: workers whose done_clean oracle is gone ──
        // The cascade close above only fires while the oracle SESSION is still
        // alive to be reaped. When the oracle already closed (inline auto-close,
        // manual kill, crash-after-done) its leftover workers had NO reaper at
        // all — the 7-zombie dentistrygpt incident. Sweep them here.
        self.sweep_orphan_workers(
            &mgr,
            &sessions,
            &oracle_states,
            &typed_runtime_sessions,
            worker_runtime.inventory_compromised,
            &mut report,
        )
        .await?;

        // ── Oracle recovery: resurrect crashed-mid-mission oracles (guarded) ──
        if let Err(error) = self.resurrect_dead_oracles(&mut report).await {
            report.actions_taken.push(format!(
                "Oracle resurrection authority failed closed ({error})"
            ));
        }

        // ── Signal file watcher: detect new oracle result files ──
        match self.signal_watcher.poll() {
            Ok(new_signals) => {
                for (oracle_name, signal) in &new_signals {
                    report.actions_taken.push(format!(
                        "Signal file detected: {} (status: {:?})",
                        oracle_name, signal.status
                    ));
                }
            }
            Err(error) => report
                .actions_taken
                .push(format!("Oracle signal watcher failed closed ({error})")),
        }

        // ── State-dir GC (bounded, age-gated) ──
        self.gc_state_dir(&live_session_names, &mut report);

        self.log_patrol(&report)?;

        Ok(report)
    }

    /// Mission wall-clock breaker (Loop Engineering bounded-runtime). A running
    /// oracle with no closeable done signal gets a timeline note at the SOFT
    /// ceiling and an operator escalation at the HARD ceiling. Never a kill —
    /// the goal is to force a human to LOOK at a loop that has run unusually
    /// long, not to murder legitimately long work (L5). Soft is marker-gated so
    /// the timeline gets one note, not one per patrol tick.
    fn check_mission_wallclock(
        &self,
        session_name: &str,
        spawned_at: Option<DateTime<Utc>>,
        has_closeable_done: bool,
        report: &mut PatrolReport,
    ) {
        use crate::loop_guard::{self, EscalationReason};
        if has_closeable_done {
            return;
        }
        let Some(start) = spawned_at else {
            return;
        };
        let elapsed = (Utc::now() - start).num_seconds();
        let hrs = elapsed / 3_600;
        if elapsed >= loop_guard::HARD_WALLCLOCK_SECS {
            if loop_guard::escalate_to_human(
                &self.config.state_dir,
                session_name,
                EscalationReason::WallClock,
                &format!(
                    "running {}h with no closeable done signal (hard ceiling {}h)",
                    hrs,
                    loop_guard::HARD_WALLCLOCK_SECS / 3_600
                ),
            ) {
                report.actions_taken.push(format!(
                    "ESCALATED TO HUMAN: {} past hard wall-clock ceiling ({}h)",
                    session_name, hrs
                ));
            }
        } else if elapsed >= loop_guard::SOFT_WALLCLOCK_SECS {
            // One-shot timeline note (no operator ping at soft).
            let key = session_name.strip_prefix("oracle-").unwrap_or(session_name);
            let marker = self
                .config
                .state_dir
                .join(format!("{}.wallclock-soft", key));
            if !marker.exists() {
                let _ = std::fs::write(&marker, Utc::now().to_rfc3339());
                loop_guard::MissionLog::event(
                    &self.config.state_dir,
                    session_name,
                    "wallclock",
                    &format!(
                        "mission running {}h (soft ceiling {}h) — still no closeable done signal",
                        hrs,
                        loop_guard::SOFT_WALLCLOCK_SECS / 3_600
                    ),
                );
            }
        }
    }

    /// Patrol oracle sessions: check for done oracles, update registry, handle close.
    async fn patrol_oracles(
        &mut self,
        mgr: &SessionManager,
        sessions: &[crate::session::OmegaSession],
        oracle_states: &[OracleState],
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
        let registry = OracleRegistry::load_strict(&self.config.state_dir)?;
        let mut status_changes: Vec<(String, OracleRegistryStatus)> = Vec::new();

        for session in sessions {
            if session.role != SessionRole::Oracle {
                continue;
            }

            // ── Loop Engineering: mission wall-clock breaker ──
            // A loop with no ceiling is the article's "cognitive surrender".
            // We do NOT kill long missions (L5 / ultracode: a 37h mission is
            // legitimate) — at the SOFT ceiling we drop a timeline note, and
            // only at the HARD ceiling do we ping the operator to come look.
            let spawned_at = registry
                .oracles
                .iter()
                .find(|e| e.session_name == session.name)
                .map(|e| e.spawned_at);
            let has_closeable_done = OracleDoneSignal::read(&self.config.state_dir, &session.name)
                .ok()
                .flatten()
                .map(|d| d.is_closeable())
                .unwrap_or(false);
            self.check_mission_wallclock(&session.name, spawned_at, has_closeable_done, report);

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
                // `omega done` downgrades done_clean → Pending for EITHER of two
                // reasons: the plan is <100%, or the independent quality gate is
                // absent / not accepted. `omega progress` upgrades it back only
                // when BOTH have since been satisfied. If that tick was missed,
                // resolve it here — against the same two conditions, so the two
                // copies of the rule can never disagree.
                //
                // This used to read the raw `done` / `total` counters off the
                // progress JSON and ignore the gate entirely, so a mission
                // refused ONLY by the missing gate was auto-accepted within one
                // patrol cycle and the gate's refusal text was cleared with it.
                // The ledger is now the source of truth (the counters can be
                // stale or hand-edited, and `is_complete` also refuses a plan
                // holding a failed task), and an unreadable ledger or an absent
                // gate leaves the signal HONESTLY pending.
                if !stale && done.status == DoneStatus::Pending && done.gate_pending {
                    let plan_complete =
                        crate::oracle_todo::OracleTodo::load(&self.config.state_dir, &session.name)
                            .map(|t| t.is_complete())
                            .unwrap_or(false);
                    // `None` = no gate result on disk at all, which is a
                    // REFUSAL here, never a pass (L5: an absent verdict is not
                    // an accepted one).
                    let gate_overall_pass =
                        crate::gate::GateResult::read(&self.config.state_dir, &session.name)
                            .ok()
                            .flatten()
                            .map(|g| g.overall_pass);
                    if may_upgrade_gate_pending(plan_complete, gate_overall_pass) {
                        done.status = DoneStatus::DoneClean;
                        done.pending_actions.clear();
                        done.gate_pending = false;
                        done.finished_at = Utc::now();
                        done.duration_secs =
                            (done.finished_at - done.started_at).num_seconds().max(0) as u64;
                        if let Err(error) = done.write(&self.config.state_dir) {
                            report.actions_taken.push(format!(
                                "Held gate-pending oracle {}: secure signal rewrite failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                        // The notifier may have already reported the
                        // transient Pending state and written its per-path
                        // marker — invalidate it so the corrected
                        // done_clean is notified exactly once.
                        if let Err(error) = OracleDoneSignal::invalidate_notified_strict(
                            &self.config.state_dir,
                            &session.name,
                        ) {
                            report.actions_taken.push(format!(
                                "Oracle {} upgraded but notification re-arm failed ({error})",
                                session.name
                            ));
                            continue;
                        }
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

                if !stale && done.is_closeable() {
                    let delivered_state = match self.require_delivered_oracle_authority(
                        oracle_states,
                        &session.name,
                        &done,
                    ) {
                        Ok(state) => state,
                        Err(error) => {
                            report.actions_taken.push(format!(
                                "Held closeable oracle {}: exact Delivered V3 authority failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                    };
                    report.done_oracles.push(session.name.clone());
                    status_changes.push((session.name.clone(), OracleRegistryStatus::Done));

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
                        let lw = match crate::oracle_lifecycle::live_workers_of_oracle_strict(
                            &self.config.state_dir,
                            &session.name,
                            sessions,
                        ) {
                            Ok(workers) => workers,
                            Err(error) => {
                                report.actions_taken.push(format!(
                                    "Held oracle {}: strict V3 worker authority failed ({error})",
                                    session.name
                                ));
                                continue;
                            }
                        };
                        let state = &delivered_state;
                        let worker_names = lw.all();
                        let mut worker_receipts = Vec::with_capacity(worker_names.len());
                        let mut receipt_error = None;
                        for worker in &worker_names {
                            match ScopeClaim::read_strict(&self.config.state_dir, worker) {
                                Ok(receipt) => worker_receipts.push((worker.clone(), receipt)),
                                Err(error) => {
                                    receipt_error = Some(format!(
                                        "worker {worker} scope receipt failed: {error}"
                                    ));
                                    break;
                                }
                            }
                        }
                        let oracle_receipt =
                            match ScopeClaim::read_strict(&self.config.state_dir, &session.name) {
                                Ok(receipt) => receipt,
                                Err(error) => {
                                    receipt_error =
                                        Some(format!("oracle scope receipt failed: {error}"));
                                    None
                                }
                            };
                        if let Some(error) = receipt_error {
                            report.actions_taken.push(format!(
                                "Held oracle {} before cascade kill: {error}",
                                session.name
                            ));
                            continue;
                        }
                        let mut close_failed = false;
                        for (w, receipt) in worker_receipts {
                            if let Err(error) = mgr.kill_session(&w).await {
                                close_failed = true;
                                report.actions_taken.push(format!(
                                    "Held scope for {} and oracle {}: worker kill failed ({error})",
                                    w, session.name
                                ));
                                continue;
                            }
                            if let Some(receipt) = &receipt {
                                if let Err(error) =
                                    ScopeClaim::release_exact(&self.config.state_dir, receipt)
                                {
                                    close_failed = true;
                                    report.actions_taken.push(format!(
                                        "Worker {} closed but exact scope release failed ({error})",
                                        w
                                    ));
                                    continue;
                                }
                            }
                            self.stall_detector.forget(&w);
                            WorkerCloseMarker::remove(&self.config.state_dir, &w)?;
                            remove_inbox_event_markers(&self.config.state_dir, &w)?;
                            let was_running = lw.running.contains(&w);
                            tracing::info!(
                                oracle = %session.name, worker = %w, was_running,
                                "Cascade close: worker closed with its done_clean oracle"
                            );
                            report.actions_taken.push(format!(
                                "Cascade-closed worker {} with done_clean oracle {}{}",
                                w,
                                session.name,
                                if was_running {
                                    " (was still running!)"
                                } else {
                                    ""
                                }
                            ));
                        }
                        if close_failed {
                            continue;
                        }
                        if let Err(error) = mgr.kill_session(&session.name).await {
                            report.actions_taken.push(format!(
                                "Held oracle scope {}: oracle kill failed ({error})",
                                session.name
                            ));
                            continue;
                        }
                        // Release any scope claim the oracle still held —
                        // parity with the worker reap above (a gate-pending
                        // oracle skips the cmd_done-time release because its
                        // signal was not closeable yet, so the claim would
                        // otherwise leak until a manual cleanup).
                        if let Some(receipt) = &oracle_receipt {
                            if let Err(error) =
                                ScopeClaim::release_exact(&self.config.state_dir, receipt)
                            {
                                report.actions_taken.push(format!(
                                    "Oracle {} closed but exact scope release failed ({error})",
                                    session.name
                                ));
                                continue;
                            }
                        }
                        tracing::info!(
                            oracle = %session.name,
                            closeable_secs,
                            "Deterministic reap: done_clean oracle closed"
                        );
                        report.actions_taken.push(format!(
                            "Reaped done_clean oracle {} ({}s past finished_at)",
                            session.name, closeable_secs
                        ));
                        if let Err(error) =
                            self.maybe_trigger_curator(mgr, state, &session.name).await
                        {
                            report.actions_taken.push(format!(
                                "Oracle {} closed, curator trigger failed closed ({error})",
                                session.name
                            ));
                        }
                    }
                }
            }

            // Check oracle state for all-workers-terminal
            if let Some(oracle_state) = oracle_states
                .iter()
                .find(|state| state.oracle_name == session.name)
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
        OracleRegistry::update_locked(&self.config.state_dir, |reg| {
            reg.cleanup(&live_names);
            for (name, status) in &status_changes {
                reg.mark_status(name, *status);
            }
        })?;
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
        typed_runtime_sessions: &std::collections::HashSet<String>,
        worker_runtime_inventory_compromised: bool,
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
            if worker_runtime_inventory_compromised || typed_runtime_sessions.contains(&w.name) {
                continue;
            }
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
                    let Some(project) = w.project.as_deref() else {
                        continue;
                    };
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
            match self.v3_worker_attempt_state(oracle_states, &w.name) {
                Ok(Some(crate::mission::TaskAttemptState::Accepted)) => {}
                Ok(None) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: no exact V3 worker authority",
                        w.name
                    ));
                    continue;
                }
                Ok(Some(state)) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: V3 attempt is {:?}, not Accepted",
                        w.name, state
                    ));
                    continue;
                }
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: V3 authority failed closed ({error})",
                        w.name
                    ));
                    continue;
                }
            }
            let scope_receipt = match ScopeClaim::read_strict(&self.config.state_dir, &w.name) {
                Ok(receipt) => receipt,
                Err(error) => {
                    report.actions_taken.push(format!(
                        "Orphan sweep held {}: exact scope receipt failed ({error})",
                        w.name
                    ));
                    continue;
                }
            };
            if let Err(error) = mgr.kill_session(&w.name).await {
                report.actions_taken.push(format!(
                    "Orphan sweep held scope for {}: kill failed ({error})",
                    w.name
                ));
                continue;
            }
            if let Some(receipt) = &scope_receipt {
                if let Err(error) = ScopeClaim::release_exact(&self.config.state_dir, receipt) {
                    report.actions_taken.push(format!(
                        "Orphan {} closed but exact scope release failed ({error})",
                        w.name
                    ));
                    continue;
                }
            }
            self.stall_detector.forget(&w.name);
            WorkerCloseMarker::remove(&self.config.state_dir, &w.name)?;
            remove_inbox_event_markers(&self.config.state_dir, &w.name)?;
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
                report.actions_taken.push(format!(
                    "Resurrected crashed oracle {} (mission unfinished)",
                    name
                ));
            }
        }
        Ok(())
    }

    /// Self-improvement hook: when an oracle's done.json flips to a
    /// closeable status, spawn a curator worker that reads the trajectory
    /// + done.json and proposes NEW_SKILL / EDIT_SKILL / NEW_RULE /
    ///   NEW_MEMORY items. Output lands in
    ///   `~/.omega/state/curator/<oracle>-<timestamp>.md`.
    ///
    /// Idempotent: marker file `~/.omega/state/curator-triggered/<oracle>.flag`
    /// prevents re-trigger on subsequent patrol ticks.
    async fn maybe_trigger_curator(
        &self,
        mgr: &SessionManager,
        oracle_state: &OracleState,
        oracle_name: &str,
    ) -> Result<()> {
        crate::scope::validate_session_identity(oracle_name)?;
        let ledger = crate::mission_ledger::MissionLedger::open(
            self.config.state_dir.join("mission-engine-v3.sqlite3"),
        )?;
        oracle_state.require_ledger_authority(&ledger)?;
        let mission = ledger
            .mission_record(&oracle_state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("immutable mission is missing"))?;
        let projection = ledger
            .mission(&oracle_state.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("mission projection is missing"))?;
        if projection.state != crate::mission::MissionState::Delivered {
            anyhow::bail!("curator dispatch requires exact Delivered V3 authority");
        }
        let signal = OracleDoneSignal::read(&self.config.state_dir, oracle_name)?
            .filter(|signal| signal.is_closeable())
            .ok_or_else(|| {
                anyhow::anyhow!("curator dispatch requires a strict closeable signal")
            })?;
        crate::orchestration::validate_oracle_done_signal_authority(
            &ledger,
            &oracle_state.mission_id,
            oracle_name,
            &signal,
        )?;
        let flag_dir = self.config.state_dir.join("curator-triggered");
        let flag = flag_dir.join(format!("{}.flag", oracle_name));
        {
            let _lock = crate::scope::lock_private_state_file(
                &self.config.state_dir,
                ".curator-trigger.lock",
            )?;
            if crate::config::read_private_optional(&flag)?.is_some() {
                return Ok(());
            }
        }
        std::fs::create_dir_all(&flag_dir)?;
        std::fs::create_dir_all(self.config.state_dir.join("curator"))?;

        let curator_session =
            crate::session::sanitize_session_name(&format!("curator-{}", oracle_name));
        let done_key = oracle_name.strip_prefix("oracle-").unwrap_or(oracle_name);
        let done_path = self
            .config
            .state_dir
            .join(format!("oracle-{}.done.json", done_key));
        let prompt = format!("/omega-curate {}", done_path.to_string_lossy());
        let agent =
            crate::agents::Agent::from_name(&self.config.agent_command).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown configured curator provider {:?}",
                    self.config.agent_command
                )
            })?;
        let working_dir = mission
            .working_dir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("mission working directory is not UTF-8"))?;
        mgr.create_agent_session_with_opts(
            &curator_session,
            working_dir,
            agent,
            Some(&prompt),
            crate::agents::LaunchOptions::default(),
        )
        .await?;
        {
            let _lock = crate::scope::lock_private_state_file(
                &self.config.state_dir,
                ".curator-trigger.lock",
            )?;
            if crate::config::read_private_optional(&flag)?.is_none() {
                crate::config::atomic_write_private(&flag, Utc::now().to_rfc3339().as_bytes())?;
            }
        }
        tracing::info!(
            oracle = %oracle_name,
            curator = %curator_session,
            "curator dispatched through typed provider launch"
        );
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
    fn gc_state_dir(&self, live: &std::collections::HashSet<&str>, report: &mut PatrolReport) {
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
    fn validate_identity(session: &str, oracle: Option<&str>) -> Result<()> {
        if session.is_empty() || crate::session::sanitize_session_name(session) != session {
            anyhow::bail!("worker close marker has unsafe session identity `{session}`");
        }
        if let Some(oracle) = oracle {
            if oracle.is_empty() || crate::session::sanitize_session_name(oracle) != oracle {
                anyhow::bail!("worker close marker has unsafe oracle identity `{oracle}`");
            }
        }
        Ok(())
    }

    fn path(state_dir: &std::path::Path, session: &str) -> Result<std::path::PathBuf> {
        Self::validate_identity(session, None)?;
        Ok(state_dir.join(format!("worker-close-{session}.json")))
    }

    fn validate_at_path(&self, path: &std::path::Path) -> Result<()> {
        Self::validate_identity(&self.session, self.oracle.as_deref())?;
        let state_dir = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("worker close marker path has no parent"))?;
        if Self::path(state_dir, &self.session)? != path {
            anyhow::bail!(
                "worker close marker filename differs from embedded session {}",
                self.session
            );
        }
        Ok(())
    }

    fn read_path(path: &std::path::Path) -> Result<Option<WorkerCloseMarker>> {
        let Some(bytes) = crate::config::read_private_optional(path)? else {
            return Ok(None);
        };
        let marker: WorkerCloseMarker = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing worker close marker {}", path.display()))?;
        marker.validate_at_path(path)?;
        Ok(Some(marker))
    }

    /// Write the marker once. Idempotent: if it already exists, keep the
    /// original `since` so the grace clock isn't reset every tick.
    fn ensure(state_dir: &std::path::Path, session: &str, oracle: Option<&str>) -> Result<()> {
        Self::validate_identity(session, oracle)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, ".worker-close.lock")?;
        let path = Self::path(state_dir, session)?;
        if let Some(existing) = Self::read_path(&path)? {
            if existing.oracle.as_deref() != oracle {
                anyhow::bail!(
                    "worker close marker oracle conflict for {session}: recorded {:?}, requested {:?}",
                    existing.oracle,
                    oracle
                );
            }
            return Ok(());
        }
        let marker = WorkerCloseMarker {
            session: session.to_string(),
            oracle: oracle.map(|s| s.to_string()),
            since: Utc::now(),
        };
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(&marker)?)?;
        let recorded = Self::read_path(&path)?
            .ok_or_else(|| anyhow::anyhow!("worker close marker vanished after publication"))?;
        if recorded.session != marker.session
            || recorded.oracle != marker.oracle
            || recorded.since != marker.since
        {
            anyhow::bail!("worker close marker changed during publication");
        }
        Ok(())
    }

    fn read_all(state_dir: &std::path::Path) -> Result<Vec<WorkerCloseMarker>> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(state_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if name.starts_with("worker-close-") && name.ends_with(".json") {
                    out.push(Self::read_path(&path)?.ok_or_else(|| {
                        anyhow::anyhow!("worker close marker vanished during strict enumeration")
                    })?);
                }
            }
        }
        out.sort_by(|left, right| left.session.cmp(&right.session));
        Ok(out)
    }

    fn remove(state_dir: &std::path::Path, session: &str) -> Result<()> {
        let _lock = crate::scope::lock_private_state_file(state_dir, ".worker-close.lock")?;
        let path = Self::path(state_dir, session)?;
        if let Some(marker) = Self::read_path(&path)? {
            if marker.session != session {
                anyhow::bail!("worker close marker identity changed before removal");
            }
        }
        crate::scope::remove_private_file(&path)
    }
}

/// L4 gate-pending upgrade predicate (pure + testable). Patrol may rewrite a
/// `gate_pending` signal back to `done_clean` only when BOTH refusals `omega
/// done` can record have since been lifted: the plan is genuinely complete
/// (`OracleTodo::is_complete`, not the raw counters) AND the independent
/// quality gate reports `overall_pass`.
///
/// The gate argument is an `Option` on purpose, because ABSENT and FAILED are
/// two different states that must not be collapsed at the call site: `None` is
/// "no `GateResult` on disk", `Some(false)` is "the gate ran and refused", and
/// only `Some(true)` is an accepted verdict. Both refusals leave the signal
/// pending, which is the whole point — a `done_clean` nobody verified is worse
/// than an honest pending that waits (R-VERIFY, L5).
fn may_upgrade_gate_pending(plan_complete: bool, gate_overall_pass: Option<bool>) -> bool {
    plan_complete && gate_overall_pass == Some(true)
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
/// (closeable done signal) AND `finished_at` sits inside the reap WINDOW:
/// at least the generous grace has elapsed (so a same-name re-dispatch, which
/// clears the stale signal first, can never race the sweep) and no more than
/// [`ORPHAN_SIGNAL_MAX_AGE_SECS`] has elapsed.
///
/// The ceiling is what makes this safe on the project-match branch. Matching by
/// project alone means the signal was NOT necessarily written by this worker's
/// parent, so "grace elapsed" cannot imply "this signal governs this worker" —
/// an ancient done_clean stayed eligible forever and reaped brand-new workers.
/// A real orphan is swept within grace plus one tick; past the ceiling, a live
/// worker is by definition a newer one that this mission never dispatched.
fn should_reap_orphan(closeable: bool, finished_secs: i64) -> bool {
    closeable && (ORPHAN_WORKER_GRACE_SECS..=ORPHAN_SIGNAL_MAX_AGE_SECS).contains(&finished_secs)
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

fn typed_runtime_protects_session(
    session: &crate::session::OmegaSession,
    typed_runtime_sessions: &std::collections::HashSet<String>,
    worker_runtime_inventory_compromised: bool,
    team_runtime_inventory_compromised: bool,
) -> bool {
    team_runtime_inventory_compromised
        || typed_runtime_sessions.contains(&session.name)
        || (worker_runtime_inventory_compromised && session.role == SessionRole::Worker)
}

fn prepared_worker_runtime_is_stale(
    runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    now: DateTime<Utc>,
) -> bool {
    (now - runtime.prepared_at()).num_seconds() >= WORKER_RUNTIME_PREPARED_STALE_SECS
}

fn prepared_worker_runtime_recovery_ready(
    runtime: &crate::worker_runtime::WorkerRuntimeManifest,
    now: DateTime<Utc>,
) -> bool {
    (now - runtime.prepared_at()).num_seconds() >= WORKER_RUNTIME_PREPARED_RECOVERY_GRACE_SECS
}

fn worker_runtime_inventory_is_compromised(
    inventory: &crate::worker_runtime::WorkerRuntimeInventory,
) -> bool {
    !inventory.corrupt_entries.is_empty() || !inventory.duplicate_sessions.is_empty()
}

fn strict_worker_binding<'a>(
    states: &'a [OracleState],
    session: &str,
) -> Result<Option<(&'a OracleState, &'a WorkerEntry)>> {
    let matches = states
        .iter()
        .flat_map(|state| {
            state
                .workers
                .iter()
                .filter(move |worker| worker.session_name == session)
                .map(move |worker| (state, worker))
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [binding] => Ok(Some(*binding)),
        _ => anyhow::bail!(
            "worker session {session} is bound by {} OracleState generations",
            matches.len()
        ),
    }
}

fn worker_done_event_key(
    effective_status: &str,
    signal: &DoneSignal,
    binding: Option<(&OracleState, &WorkerEntry)>,
) -> Result<String> {
    let binding = binding.map(|(oracle, worker)| {
        serde_json::json!({
            "mission_id": oracle.mission_id,
            "oracle_name": oracle.oracle_name,
            "task_id": worker.task_id,
            "attempt_id": worker.attempt_id,
            "plan_revision": worker.plan_revision,
            "dispatched_at": worker.dispatched_at,
        })
    });
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "kind": "done",
        "effective_status": effective_status,
        "signal": signal,
        "attempt_generation": binding,
    }))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn worker_blocked_event_key(
    signal: &WorkerBlocked,
    binding: Option<(&OracleState, &WorkerEntry)>,
) -> Result<String> {
    let binding = binding.map(|(oracle, worker)| {
        serde_json::json!({
            "mission_id": oracle.mission_id,
            "oracle_name": oracle.oracle_name,
            "task_id": worker.task_id,
            "attempt_id": worker.attempt_id,
            "plan_revision": worker.plan_revision,
            "dispatched_at": worker.dispatched_at,
        })
    });
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "kind": "blocked",
        "signal": signal,
        "attempt_generation": binding,
    }))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

/// Crash-safe, owner-only push-once authority for per-tick inbox delivery.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct InboxEventMarker {
    schema_version: u32,
    session: String,
    kind: String,
    key: String,
}

fn validate_inbox_event_marker_identity(session: &str, kind: &str) -> Result<()> {
    if session.is_empty() || crate::session::sanitize_session_name(session) != session {
        anyhow::bail!("inbox event marker has unsafe session identity `{session}`");
    }
    if !matches!(kind, "done" | "blocked") {
        anyhow::bail!("inbox event marker has unsupported kind `{kind}`");
    }
    Ok(())
}

fn event_sent_path(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
) -> Result<std::path::PathBuf> {
    validate_inbox_event_marker_identity(session, kind)?;
    Ok(state_dir.join(format!("worker-{session}.{kind}.sent")))
}

fn read_inbox_event_marker(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
) -> Result<Option<InboxEventMarker>> {
    let path = event_sent_path(state_dir, session, kind)?;
    let Some(bytes) = crate::config::read_private_optional(&path)? else {
        return Ok(None);
    };
    let marker: InboxEventMarker = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing inbox event marker {}", path.display()))?;
    if marker.schema_version != 1 || marker.session != session || marker.kind != kind {
        anyhow::bail!(
            "inbox event marker {} does not match its immutable path identity",
            path.display()
        );
    }
    if marker.key.is_empty() {
        anyhow::bail!("inbox event marker {} has an empty digest", path.display());
    }
    Ok(Some(marker))
}

fn inbox_event_already_sent(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
    key: &str,
) -> Result<bool> {
    Ok(read_inbox_event_marker(state_dir, session, kind)?.is_some_and(|marker| marker.key == key))
}

fn record_inbox_event_sent(
    state_dir: &std::path::Path,
    session: &str,
    kind: &str,
    key: &str,
) -> Result<()> {
    if key.is_empty() {
        anyhow::bail!("refusing an empty inbox event digest");
    }
    let _lock = crate::scope::lock_private_state_file(state_dir, ".inbox-event-marker.lock")?;
    let path = event_sent_path(state_dir, session, kind)?;
    let expected = InboxEventMarker {
        schema_version: 1,
        session: session.to_string(),
        kind: kind.to_string(),
        key: key.to_string(),
    };
    if read_inbox_event_marker(state_dir, session, kind)?.as_ref() == Some(&expected) {
        return Ok(());
    }
    crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(&expected)?)?;
    if read_inbox_event_marker(state_dir, session, kind)?.as_ref() != Some(&expected) {
        anyhow::bail!("inbox event marker changed during publication");
    }
    Ok(())
}

fn remove_inbox_event_markers(state_dir: &std::path::Path, session: &str) -> Result<()> {
    let _lock = crate::scope::lock_private_state_file(state_dir, ".inbox-event-marker.lock")?;
    for kind in ["done", "blocked"] {
        let path = event_sent_path(state_dir, session, kind)?;
        if let Some(marker) = read_inbox_event_marker(state_dir, session, kind)? {
            if marker.session != session || marker.kind != kind {
                anyhow::bail!("inbox event marker identity changed before removal");
            }
        }
        crate::scope::remove_private_file(&path)?;
    }
    Ok(())
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

/// The ONLY predicate that may contest a worker's `done_clean`.
///
/// A check that RAN and refuted the claim is a concrete fabrication. A check
/// that could not run — no repo root, `git` absent, a command that was never
/// predeclared — proves nothing and must not contest anything: it still fails
/// the verdict, so the work falls through to `Pending` with its scope held.
/// This used to be `any(|c| !c.passed)`, which could not tell those apart and
/// escalated four unverifiable checks to a human as fabrications on mission
/// OmegaOS-m-8fe7d35df5bf.
#[cfg(test)]
fn verdict_contests_worker(verdict: &crate::done::GroundTruthVerdict) -> bool {
    verdict.checks.iter().any(|c| c.outcome.is_contradicted())
}

/// Resolve the repo root the ground-truth gate must check a worker's artifacts
/// against.
///
/// There were two project registries and the gate read the wrong one.
/// `~/.omega/projects.json` (`ProjectRegistry`) is the live one — 25 projects on
/// this machine — while `config.projects` from `~/.omega/config.toml` is
/// literally `projects = []`. Reading only the config handed the gate a `None`
/// root, every git/file check went unverified, and an unverifiable check was
/// then branded a fabrication. The registry is consulted FIRST; the config
/// remains the fallback so nothing regresses where it IS populated.
fn resolve_repo_root(config: &OmegaConfig, project: &str) -> Option<std::path::PathBuf> {
    resolve_repo_root_in(
        &crate::project_manager::ProjectRegistry::registry_path(),
        config,
        project,
    )
}

/// Testable core of [`resolve_repo_root`]: the registry path is explicit so a
/// test never reads (or writes) the machine's real `~/.omega/projects.json`.
fn resolve_repo_root_in(
    registry_path: &std::path::Path,
    config: &OmegaConfig,
    project: &str,
) -> Option<std::path::PathBuf> {
    crate::project_manager::ProjectRegistry::load_from(registry_path)
        .find(project)
        .map(|managed| managed.path.clone())
        .or_else(|| config.find_project(project).map(|pc| pc.path.clone()))
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

    struct WorkerRuntimeFixture {
        _tmp: tempfile::TempDir,
        state_dir: std::path::PathBuf,
        workspace: std::path::PathBuf,
        ledger: crate::mission_ledger::MissionLedger,
        runtime: crate::worker_runtime::WorkerRuntimeManifest,
        patrol: Patrol,
    }

    fn worker_runtime_fixture(suffix: &str, started: bool) -> WorkerRuntimeFixture {
        let tmp = tempfile::TempDir::new().unwrap();
        let state_dir = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        let workspace = std::fs::canonicalize(workspace).unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&workspace)
            .status()
            .unwrap()
            .success());
        let team = crate::team::TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: format!("Team-OmegaOS-runtime-{suffix}"),
            working_dir: workspace.to_string_lossy().into_owned(),
            agent_command: "codex".to_string(),
            members: vec![crate::team::TeamMember {
                name: "writer".to_string(),
                role: "worker".to_string(),
                prompt: "write src/output.rs".to_string(),
                files_owned: vec!["src/output.rs".to_string()],
            }],
        };
        let mut prepared = crate::team::prepare_team_authority(&state_dir, &team).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(&state_dir),
        )
        .unwrap();
        let base = format!("OmegaOS-worker-{suffix}");
        let attempt = prepared.authority.attempts.first().unwrap().clone();
        let preliminary = crate::worker_runtime::WorkerRuntimeIntent {
            attempt: crate::worker_runtime::WorkerAttemptIdentity {
                mission_id: attempt.mission_id.clone(),
                plan_revision: attempt.plan_revision,
                plan_digest: prepared.authority.plan.content_digest.clone(),
                task_id: attempt.task_id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                owner: base.clone(),
            },
            launch: crate::worker_runtime::WorkerLaunchIdentity {
                session: base.clone(),
                expected_command_digest: "0".repeat(64),
                authority_workspace: workspace.clone(),
                execution_workspace: workspace.clone(),
                worktree: None,
                project: "OmegaOS".to_string(),
                provider: "codex".to_string(),
            },
            scope: None,
        };
        let session = preliminary.generation_scoped_session(&base).unwrap();
        let attempt = prepared.authority.attempts.first_mut().unwrap();
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &attempt.mission_id,
            crate::mission::MissionState::Running,
            &session,
        )
        .unwrap();
        crate::orchestration::claim_authoritative_scopes(
            &ledger,
            &state_dir,
            &workspace,
            attempt,
            &session,
            &["src/output.rs".to_string()],
            Duration::from_secs(3600),
        )
        .unwrap();
        let receipt = attempt.scope_receipt.as_ref().unwrap();
        let scope = crate::worker_runtime::WorkerRuntimeScope::from_authority(
            crate::worker_runtime::WorkerScopeReceipt {
                schema_version: 1,
                mission_id: receipt.mission_id.clone(),
                task_id: receipt.task_id.clone(),
                attempt_id: receipt.attempt_id.clone(),
                plan_revision: receipt.plan_revision,
                owner: receipt.owner.clone(),
                claim: receipt.claim.clone(),
            },
            &attempt.leases,
        )
        .unwrap();
        let expected_command_digest = "ab".repeat(32);
        let intent = crate::worker_runtime::WorkerRuntimeIntent {
            attempt: crate::worker_runtime::WorkerAttemptIdentity {
                mission_id: attempt.mission_id.clone(),
                plan_revision: attempt.plan_revision,
                plan_digest: prepared.authority.plan.content_digest.clone(),
                task_id: attempt.task_id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                owner: session.clone(),
            },
            launch: crate::worker_runtime::WorkerLaunchIdentity {
                session: session.clone(),
                expected_command_digest: expected_command_digest.clone(),
                authority_workspace: workspace.clone(),
                execution_workspace: workspace.clone(),
                worktree: None,
                project: "OmegaOS".to_string(),
                provider: "codex".to_string(),
            },
            scope: Some(scope),
        };
        let mut runtime =
            crate::worker_runtime::WorkerRuntimeManifest::prepare(&state_dir, intent).unwrap();
        if started {
            crate::orchestration::transition_authoritative_attempt(
                &ledger,
                attempt,
                crate::mission::TaskAttemptState::Running,
                &session,
            )
            .unwrap();
            let observed = crate::worker_runtime::ObservedWorkerProcess::new(
                &session,
                rmux_sdk::PaneId::new(1),
                rmux_sdk::SessionId::new(1),
                rmux_sdk::WindowId::new(1),
                1,
                1000,
                expected_command_digest,
                &workspace,
            )
            .unwrap();
            runtime = runtime.activate_started(&state_dir, observed).unwrap();
        }
        let config = OmegaConfig {
            state_dir: state_dir.clone(),
            ..OmegaConfig::default()
        };
        WorkerRuntimeFixture {
            _tmp: tmp,
            state_dir,
            workspace,
            ledger,
            runtime,
            patrol: Patrol::new(config),
        }
    }

    fn append_worker_runtime_candidate(
        fixture: &WorkerRuntimeFixture,
        status: DoneStatus,
    ) -> DoneSignal {
        let runtime = &fixture.runtime;
        runtime.release_start_gate(&fixture.state_dir).unwrap();
        let mut signal = DoneSignal::new(&runtime.session().session, status, "runtime candidate");
        if status == DoneStatus::DoneClean {
            signal.todos_total = 1;
            signal.todos_completed = 1;
            signal.corroboration = vec![
                crate::done::CorroborationSource::WorkerSelfReport,
                crate::done::CorroborationSource::CiExitCode,
            ];
            signal.artifacts = vec![crate::done::DoneArtifact::Command {
                cmd: "git diff --check".to_string(),
                exit_code: 0,
            }];
        }
        signal.finished_at = runtime.started().unwrap().activated_at;
        let mission = fixture
            .ledger
            .mission(&runtime.attempt().mission_id)
            .unwrap()
            .unwrap();
        let attempt = fixture
            .ledger
            .task_attempt(&runtime.attempt().attempt_id)
            .unwrap()
            .unwrap();
        let leases = fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap();
        let mut event = crate::mission_ledger::AppendEvent::new(
            runtime.attempt().mission_id.clone(),
            mission.version,
            format!("test:{}:candidate", runtime.runtime_id()),
            runtime.attempt().owner.clone(),
            "worker_runtime_completion_candidate",
        );
        event.provider = Some(runtime.provider().to_string());
        event.correlation_id = Some(runtime.runtime_id().to_string());
        event.payload = serde_json::to_value(&signal).unwrap();
        event.task_attempt = Some(crate::mission_ledger::TaskAttemptMutation {
            task_id: runtime.attempt().task_id.clone(),
            attempt_id: runtime.attempt().attempt_id.clone(),
            plan_revision: runtime.attempt().plan_revision,
            expected_version: attempt.version,
            next_state: crate::mission::TaskAttemptState::CandidateDone,
        });
        event.lease_assertions = leases
            .iter()
            .map(crate::mission_ledger::LeaseAssertion::from)
            .collect();
        fixture.ledger.append(event).unwrap();
        signal
    }

    #[test]
    fn prepared_effect_crash_window_validates_command_before_running_transition() {
        let fixture = worker_runtime_fixture("prepared-crash", false);
        let runtime = &fixture.runtime;
        assert!(!prepared_worker_runtime_is_stale(runtime, Utc::now()));
        assert!(!prepared_worker_runtime_recovery_ready(runtime, Utc::now()));
        assert!(prepared_worker_runtime_recovery_ready(
            runtime,
            runtime.prepared_at()
                + chrono::Duration::seconds(WORKER_RUNTIME_PREPARED_RECOVERY_GRACE_SECS)
        ));
        assert!(prepared_worker_runtime_is_stale(
            runtime,
            runtime.prepared_at() + chrono::Duration::seconds(WORKER_RUNTIME_PREPARED_STALE_SECS)
        ));
        let wrong = crate::worker_runtime::ObservedWorkerProcess::new(
            &runtime.session().session,
            rmux_sdk::PaneId::new(1),
            rmux_sdk::SessionId::new(1),
            rmux_sdk::WindowId::new(1),
            1,
            1000,
            "cd".repeat(32),
            &fixture.workspace,
        )
        .unwrap();
        assert!(Patrol::validate_prepared_worker_observation(runtime, &wrong).is_err());
        assert_eq!(
            fixture
                .ledger
                .mission(&runtime.attempt().mission_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::MissionState::Running
        );
        assert_eq!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Queued
        );

        let exact = crate::worker_runtime::ObservedWorkerProcess::new(
            &runtime.session().session,
            rmux_sdk::PaneId::new(1),
            rmux_sdk::SessionId::new(1),
            rmux_sdk::WindowId::new(1),
            1,
            1000,
            runtime.expected_command_digest(),
            &fixture.workspace,
        )
        .unwrap();
        Patrol::validate_prepared_worker_observation(runtime, &exact).unwrap();
        fixture
            .patrol
            .recover_prepared_worker_authority(runtime)
            .unwrap();
        let activated = runtime.activate_started(&fixture.state_dir, exact).unwrap();
        assert!(!activated
            .is_start_gate_released(&fixture.state_dir)
            .unwrap());
        activated.release_start_gate(&fixture.state_dir).unwrap();
        assert!(activated
            .is_start_gate_released(&fixture.state_dir)
            .unwrap());
    }

    #[test]
    fn generation_mismatch_and_marker_alone_never_accept_or_release_early() {
        let fixture = worker_runtime_fixture("generation-mismatch", true);
        let runtime = &fixture.runtime;
        let wrong = crate::worker_runtime::ObservedWorkerProcess::new(
            &runtime.session().session,
            rmux_sdk::PaneId::new(9),
            rmux_sdk::SessionId::new(1),
            rmux_sdk::WindowId::new(1),
            9,
            9000,
            runtime.expected_command_digest(),
            &fixture.workspace,
        )
        .unwrap();
        let report = crate::worker_runtime::reconcile_worker_runtimes(
            &fixture.state_dir,
            std::slice::from_ref(&wrong),
        )
        .unwrap();
        assert_eq!(
            report.entries[0].state,
            crate::worker_runtime::WorkerRuntimeReconcileState::ProcessGenerationMismatch
        );
        DoneSignal::new(
            &runtime.session().session,
            DoneStatus::DoneClean,
            "marker only",
        )
        .write(&fixture.state_dir)
        .unwrap();
        assert!(matches!(
            fixture
                .patrol
                .load_worker_runtime_candidate_evidence(runtime)
                .unwrap(),
            WorkerRuntimeCandidateEvidence::MarkerOnly
        ));
        assert!(!fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
        assert_eq!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Running
        );

        let absence = crate::worker_runtime::ConfirmedWorkerAbsence::new(runtime).unwrap();
        fixture
            .patrol
            .reconcile_worker_runtime_authority_after_absence(runtime, &absence, true)
            .unwrap();
        assert_eq!(
            fixture
                .ledger
                .mission(&runtime.attempt().mission_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::MissionState::Blocked
        );
        assert_ne!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Accepted
        );
        assert!(fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn candidate_crash_window_rebinds_exact_event_and_is_idempotent() {
        let fixture = worker_runtime_fixture("candidate-rebind", true);
        let unprojected = append_worker_runtime_candidate(&fixture, DoneStatus::Blocked);
        assert!(fixture.runtime.candidate().is_none());
        assert!(
            DoneSignal::read(&fixture.state_dir, &fixture.runtime.session().session,)
                .unwrap()
                .is_none()
        );

        let first = fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&fixture.runtime)
            .unwrap();
        let (runtime, signal) = match first {
            WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => (runtime, signal),
            _ => panic!("exact ledger candidate was not recovered"),
        };
        assert!(runtime.candidate().is_some());
        assert_eq!(signal.status, unprojected.status);
        assert!(signal.projection.is_some());
        let generation = runtime.storage_generation();
        let second = fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&runtime)
            .unwrap();
        match second {
            WorkerRuntimeCandidateEvidence::Exact {
                runtime: replay,
                signal: replay_signal,
            } => {
                assert_eq!(replay.storage_generation(), generation);
                assert_eq!(
                    serde_json::to_value(replay_signal).unwrap(),
                    serde_json::to_value(signal).unwrap()
                );
            }
            _ => panic!("idempotent candidate replay lost exact evidence"),
        }
    }

    #[test]
    fn exact_clean_candidate_is_frozen_verified_delivered_released_and_archived() {
        let fixture = worker_runtime_fixture("clean-settlement", true);
        append_worker_runtime_candidate(&fixture, DoneStatus::DoneClean);
        let (runtime, signal) = match fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&fixture.runtime)
            .unwrap()
        {
            WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => (runtime, signal),
            _ => panic!("exact clean candidate was not recovered"),
        };
        let absence = crate::worker_runtime::ConfirmedWorkerAbsence::new(&runtime).unwrap();
        assert!(fixture
            .patrol
            .settle_worker_runtime_candidate_after_absence(&runtime, &signal, &absence)
            .unwrap());
        assert_eq!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Accepted
        );
        assert_eq!(
            fixture
                .ledger
                .mission(&runtime.attempt().mission_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::MissionState::Delivered
        );
        fixture
            .ledger
            .validate_mission_acceptance(&runtime.attempt().mission_id)
            .unwrap();
        assert!(fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
        assert!(crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &fixture.state_dir,
            runtime.runtime_id(),
        )
        .unwrap()
        .is_none());
        assert!(
            crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
                &fixture.state_dir,
                runtime.runtime_id(),
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn blocked_clean_candidate_retries_publication_without_reopening_verification() {
        let fixture = worker_runtime_fixture("clean-blocked-retry", true);
        append_worker_runtime_candidate(&fixture, DoneStatus::DoneClean);
        let (runtime, signal) = match fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&fixture.runtime)
            .unwrap()
        {
            WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => (runtime, signal),
            _ => panic!("exact clean candidate was not recovered"),
        };
        let outcome = crate::orchestration::verify_and_finalize_candidate(
            &fixture.ledger,
            &runtime.attempt().mission_id,
            &runtime.attempt().task_id,
            &runtime.attempt().attempt_id,
            runtime.attempt().plan_revision,
            &runtime.attempt().owner,
            &signal,
            &fixture.workspace,
        )
        .unwrap();
        assert!(outcome.accepted);
        fixture
            .patrol
            .mark_worker_runtime_blocked(&runtime, "simulated quality-gate rejection")
            .unwrap();
        assert_eq!(
            fixture
                .ledger
                .mission(&runtime.attempt().mission_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::MissionState::Blocked
        );
        assert!(!fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());

        let absence = crate::worker_runtime::ConfirmedWorkerAbsence::new(&runtime).unwrap();
        assert!(!fixture
            .patrol
            .settle_worker_runtime_candidate_after_absence(&runtime, &signal, &absence)
            .unwrap());
        assert!(fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
        let event_count = fixture
            .ledger
            .events(&runtime.attempt().mission_id)
            .unwrap()
            .len();
        assert!(!fixture
            .patrol
            .settle_worker_runtime_candidate_after_absence(&runtime, &signal, &absence)
            .unwrap());
        assert_eq!(
            fixture
                .ledger
                .events(&runtime.attempt().mission_id)
                .unwrap()
                .len(),
            event_count
        );
        assert_eq!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Accepted
        );
        assert!(crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &fixture.state_dir,
            runtime.runtime_id(),
        )
        .unwrap()
        .is_some());
    }

    #[test]
    fn terminal_failed_clean_candidate_retries_publication_and_archives() {
        let fixture = worker_runtime_fixture("clean-failed-retry", true);
        append_worker_runtime_candidate(&fixture, DoneStatus::DoneClean);
        let (runtime, signal) = match fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&fixture.runtime)
            .unwrap()
        {
            WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => (runtime, signal),
            _ => panic!("exact clean candidate was not recovered"),
        };
        let outcome = crate::orchestration::verify_and_finalize_candidate(
            &fixture.ledger,
            &runtime.attempt().mission_id,
            &runtime.attempt().task_id,
            &runtime.attempt().attempt_id,
            runtime.attempt().plan_revision,
            &runtime.attempt().owner,
            &signal,
            &fixture.workspace,
        )
        .unwrap();
        assert!(outcome.accepted);
        for target in [
            crate::mission::MissionState::Verifying,
            crate::mission::MissionState::Failed,
        ] {
            crate::orchestration::transition_authoritative_mission(
                &fixture.ledger,
                &runtime.attempt().mission_id,
                target,
                "patrol-retry-test",
            )
            .unwrap();
        }

        let absence = crate::worker_runtime::ConfirmedWorkerAbsence::new(&runtime).unwrap();
        assert!(fixture
            .patrol
            .settle_worker_runtime_candidate_after_absence(&runtime, &signal, &absence)
            .unwrap());
        let archive = crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
            &fixture.state_dir,
            runtime.runtime_id(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            archive.terminal.mission_state,
            crate::mission::MissionState::Failed
        );
        assert_eq!(
            archive.terminal.attempt_state,
            crate::mission::TaskAttemptState::Accepted
        );
    }

    #[test]
    fn exact_blocked_candidate_releases_only_after_absence_and_retains_runtime() {
        let fixture = worker_runtime_fixture("blocked-settlement", true);
        append_worker_runtime_candidate(&fixture, DoneStatus::Blocked);
        let (runtime, signal) = match fixture
            .patrol
            .load_worker_runtime_candidate_evidence(&fixture.runtime)
            .unwrap()
        {
            WorkerRuntimeCandidateEvidence::Exact { runtime, signal } => (runtime, signal),
            _ => panic!("exact blocked candidate was not recovered"),
        };
        let absence = crate::worker_runtime::ConfirmedWorkerAbsence::new(&runtime).unwrap();
        assert!(!fixture
            .patrol
            .settle_worker_runtime_candidate_after_absence(&runtime, &signal, &absence)
            .unwrap());
        assert_eq!(
            fixture
                .ledger
                .task_attempt(&runtime.attempt().attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Blocked
        );
        assert_eq!(
            fixture
                .ledger
                .mission(&runtime.attempt().mission_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::MissionState::Blocked
        );
        assert!(fixture
            .ledger
            .active_leases_for_attempt(
                &runtime.attempt().mission_id,
                &runtime.attempt().task_id,
                &runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
        assert!(crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &fixture.state_dir,
            runtime.runtime_id(),
        )
        .unwrap()
        .is_some());
        assert!(
            crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
                &fixture.state_dir,
                runtime.runtime_id(),
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn absence_proof_orders_scope_release_and_terminal_retirement() {
        let fixture = worker_runtime_fixture("release-order", false);
        let foreign = worker_runtime_fixture("foreign-proof", false);
        let wrong = crate::worker_runtime::ConfirmedWorkerAbsence::new(&foreign.runtime).unwrap();
        assert!(fixture
            .patrol
            .reconcile_worker_runtime_authority_after_absence(&fixture.runtime, &wrong, false,)
            .is_err());
        assert!(!fixture
            .ledger
            .active_leases_for_attempt(
                &fixture.runtime.attempt().mission_id,
                &fixture.runtime.attempt().task_id,
                &fixture.runtime.attempt().attempt_id,
            )
            .unwrap()
            .is_empty());
        assert!(
            crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
                &fixture.state_dir,
                fixture.runtime.runtime_id(),
            )
            .unwrap()
            .is_none()
        );

        let exact = crate::worker_runtime::ConfirmedWorkerAbsence::new(&fixture.runtime).unwrap();
        fixture
            .patrol
            .reconcile_worker_runtime_authority_after_absence(&fixture.runtime, &exact, false)
            .unwrap();
        let event_count = fixture
            .ledger
            .events(&fixture.runtime.attempt().mission_id)
            .unwrap()
            .len();
        fixture
            .patrol
            .reconcile_worker_runtime_authority_after_absence(&fixture.runtime, &exact, false)
            .unwrap();
        assert_eq!(
            fixture
                .ledger
                .events(&fixture.runtime.attempt().mission_id)
                .unwrap()
                .len(),
            event_count
        );
        assert!(fixture
            .patrol
            .retire_worker_runtime_if_terminal(&fixture.runtime, &exact)
            .unwrap());
        assert!(crate::worker_runtime::WorkerRuntimeManifest::load_strict(
            &fixture.state_dir,
            fixture.runtime.runtime_id(),
        )
        .unwrap()
        .is_none());
        let archive = crate::worker_runtime::WorkerRuntimeManifest::load_history_strict(
            &fixture.state_dir,
            fixture.runtime.runtime_id(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(archive.terminal.absence, exact);
        assert!(fixture
            .runtime
            .retire_terminal(&fixture.state_dir, &exact)
            .is_ok());
    }

    #[test]
    fn corrupt_or_duplicate_inventory_protects_typed_and_legacy_workers() {
        let fixture = worker_runtime_fixture("corrupt-sibling", false);
        crate::config::atomic_write_private(
            &fixture
                .state_dir
                .join(format!("worker-runtime-{}.json", "f".repeat(64))),
            b"{not-json",
        )
        .unwrap();
        let inventory =
            crate::worker_runtime::WorkerRuntimeManifest::list_strict(&fixture.state_dir).unwrap();
        assert_eq!(inventory.manifests.len(), 1);
        assert_eq!(inventory.corrupt_entries.len(), 1);
        assert!(worker_runtime_inventory_is_compromised(&inventory));
        let typed = std::iter::once(fixture.runtime.session().session.clone())
            .collect::<std::collections::HashSet<_>>();
        let typed_session = crate::session::OmegaSession {
            name: fixture.runtime.session().session.clone(),
            role: SessionRole::Worker,
            project: Some("OmegaOS".to_string()),
            oracle_index: None,
            working_dir: Some(fixture.workspace.clone()),
            provider: Some("codex".to_string()),
        };
        let unrelated_legacy = crate::session::OmegaSession {
            name: "OmegaOS-worker-legacy".to_string(),
            role: SessionRole::Worker,
            project: Some("OmegaOS".to_string()),
            oracle_index: None,
            working_dir: Some(fixture.workspace.clone()),
            provider: Some("codex".to_string()),
        };
        assert!(typed_runtime_protects_session(
            &typed_session,
            &typed,
            false,
            false,
        ));
        assert!(typed_runtime_protects_session(
            &unrelated_legacy,
            &typed,
            true,
            false,
        ));
        assert!(typed_runtime_protects_session(
            &crate::session::OmegaSession {
                name: "OmegaOS-team-unknown-role".to_string(),
                role: SessionRole::System,
                project: Some("OmegaOS".to_string()),
                oracle_index: None,
                working_dir: Some(fixture.workspace.clone()),
                provider: None,
            },
            &typed,
            false,
            true,
        ));

        let duplicate = crate::worker_runtime::WorkerRuntimeInventory {
            manifests: Vec::new(),
            corrupt_entries: Vec::new(),
            duplicate_sessions: vec!["OmegaOS-worker-collision".to_string()],
        };
        assert!(worker_runtime_inventory_is_compromised(&duplicate));
    }

    fn two_member_team_fixture(
        suffix: &str,
    ) -> (
        tempfile::TempDir,
        std::path::PathBuf,
        crate::mission_ledger::MissionLedger,
        crate::team::PreparedTeamAuthority,
        crate::team::TeamRuntimeManifest,
        Patrol,
    ) {
        let tmp = tempfile::TempDir::new().unwrap();
        let state_dir = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        let workspace = std::fs::canonicalize(workspace).unwrap();
        let team = crate::team::TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: format!("Team-OmegaOS-patrol-{suffix}"),
            working_dir: workspace.to_string_lossy().into_owned(),
            agent_command: "codex".to_string(),
            members: vec![
                crate::team::TeamMember {
                    name: "writer".to_string(),
                    role: "worker".to_string(),
                    prompt: "write".to_string(),
                    files_owned: vec!["src/writer.rs".to_string()],
                },
                crate::team::TeamMember {
                    name: "reviewer".to_string(),
                    role: "worker".to_string(),
                    prompt: "review".to_string(),
                    files_owned: vec!["src/reviewer.rs".to_string()],
                },
            ],
        };
        let mut prepared = crate::team::prepare_team_authority(&state_dir, &team).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(&state_dir),
        )
        .unwrap();
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &prepared.mission.id,
            crate::mission::MissionState::Running,
            "test-team-patrol",
        )
        .unwrap();
        for (member, task) in team.members.iter().zip(&prepared.legacy_plan.tasks) {
            let owner = crate::team::team_member_owner_for_mission(
                &team.session_name,
                &member.name,
                &prepared.mission.id,
            )
            .unwrap();
            let attempt = prepared.authority.attempt_mut(&task.id).unwrap();
            crate::orchestration::claim_authoritative_scopes(
                &ledger,
                &state_dir,
                &workspace,
                attempt,
                &owner,
                &member.files_owned,
                Duration::from_secs(3600),
            )
            .unwrap();
            crate::orchestration::transition_authoritative_attempt(
                &ledger,
                attempt,
                crate::mission::TaskAttemptState::Running,
                &owner,
            )
            .unwrap();
        }

        let members = team
            .members
            .iter()
            .zip(&prepared.legacy_plan.tasks)
            .enumerate()
            .map(|(pane_index, (member, task))| {
                let attempt = prepared.authority.attempt(&task.id).unwrap();
                crate::team::TeamRuntimeMember {
                    member_name: member.name.clone(),
                    pane_index: pane_index as u32,
                    owner: attempt.owner.clone().unwrap(),
                    task_id: task.id.clone(),
                    attempt_id: attempt.attempt_id.clone(),
                    plan_revision: attempt.plan_revision,
                    files_owned: member.files_owned.clone(),
                    scope_claim_id: attempt
                        .scope_receipt
                        .as_ref()
                        .and_then(|receipt| receipt.claim.claim_id.clone()),
                }
            })
            .collect::<Vec<_>>();
        let mut manifest = crate::team::TeamRuntimeManifest {
            schema_version: 1,
            aggregate_session: team.session_name.clone(),
            mission_id: prepared.mission.id.clone(),
            plan_revision: prepared.authority.plan.revision,
            plan_digest: prepared.authority.plan.content_digest.clone(),
            working_dir: workspace,
            provider: team.agent_command.clone(),
            created_at: Utc::now(),
            members,
            manifest_digest: String::new(),
        };
        manifest.manifest_digest = blake3::hash(&serde_json::to_vec(&manifest).unwrap())
            .to_hex()
            .to_string();
        manifest.verify_integrity().unwrap();
        let manifest_path = state_dir.join(format!(
            "team-runtime-{}-{}.json",
            manifest.aggregate_session,
            manifest.mission_id.as_str()
        ));
        crate::config::atomic_write_private(
            &manifest_path,
            &serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let panes = manifest
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let mut pane = crate::team::TeamPaneActivation {
                    owner: member.owner.clone(),
                    pane_id: rmux_sdk::PaneId::new(index as u32 + 1),
                    session_id: rmux_sdk::SessionId::new(1),
                    window_id: rmux_sdk::WindowId::new(1),
                    process_generation: index as u64 + 1,
                    process_pid: index as u32 + 1000,
                    command_digest: "ab".repeat(32),
                    working_dir: manifest.working_dir.clone(),
                    activation_digest: String::new(),
                };
                pane.activation_digest = blake3::hash(&serde_json::to_vec(&pane).unwrap())
                    .to_hex()
                    .to_string();
                pane
            })
            .collect();
        let acknowledgement = manifest.started_ack(panes).unwrap();
        crate::team::record_team_runtime_started(&state_dir, &manifest, &acknowledgement).unwrap();
        crate::team::release_team_runtime_start_barrier(&state_dir, &manifest).unwrap();

        let config = OmegaConfig {
            state_dir: state_dir.clone(),
            ..OmegaConfig::default()
        };
        let patrol = Patrol::new(config);
        (tmp, state_dir, ledger, prepared, manifest, patrol)
    }

    fn assert_two_member_team_fully_contained(
        state_dir: &std::path::Path,
        ledger: &crate::mission_ledger::MissionLedger,
        manifest: &crate::team::TeamRuntimeManifest,
        status: &crate::team::TeamRuntimeStatus,
    ) {
        assert_eq!(status.mission_state, crate::mission::MissionState::Failed);
        assert!(status.all_terminal);
        for member in &manifest.members {
            assert_eq!(
                ledger
                    .task_attempt(&member.attempt_id)
                    .unwrap()
                    .unwrap()
                    .state,
                crate::mission::TaskAttemptState::Cancelled
            );
            assert!(ledger
                .active_leases_for_attempt(
                    &manifest.mission_id,
                    &member.task_id,
                    &member.attempt_id,
                )
                .unwrap()
                .is_empty());
            assert!(ScopeClaim::read_strict(state_dir, &member.owner)
                .unwrap()
                .is_none());
        }
        assert_eq!(
            crate::team::load_team_runtime_manifest(
                state_dir,
                &manifest.aggregate_session,
                &manifest.mission_id,
            )
            .unwrap(),
            Some(manifest.clone())
        );
    }

    #[test]
    fn nonclean_team_contains_two_members_releases_scopes_and_retains_evidence() {
        let (_tmp, state_dir, ledger, prepared, manifest, patrol) =
            two_member_team_fixture("nonclean");

        let blocked = prepared.authority.attempts[0].clone();
        for next in [
            crate::mission::TaskAttemptState::CandidateDone,
            crate::mission::TaskAttemptState::Verifying,
            crate::mission::TaskAttemptState::Blocked,
        ] {
            crate::orchestration::transition_authoritative_attempt(
                &ledger,
                &blocked,
                next,
                blocked.owner.as_deref().unwrap(),
            )
            .unwrap();
        }
        Patrol::block_team_mission(&ledger, &manifest.mission_id).unwrap();
        let status = patrol
            .reconcile_contained_nonclean_team(&manifest, &[])
            .unwrap();
        assert_two_member_team_fully_contained(&state_dir, &ledger, &manifest, &status);
    }

    #[test]
    fn activation_validation_failure_contains_two_members_after_exact_absence() {
        let (_tmp, state_dir, ledger, _prepared, manifest, patrol) =
            two_member_team_fixture("activation-failure");
        Patrol::block_team_mission(&ledger, &manifest.mission_id).unwrap();

        assert!(patrol
            .reconcile_contained_nonclean_team(
                &manifest,
                std::slice::from_ref(&manifest.aggregate_session),
            )
            .is_err());
        for member in &manifest.members {
            assert_eq!(
                ledger
                    .task_attempt(&member.attempt_id)
                    .unwrap()
                    .unwrap()
                    .state,
                crate::mission::TaskAttemptState::Running
            );
            assert!(!ledger
                .active_leases_for_attempt(
                    &manifest.mission_id,
                    &member.task_id,
                    &member.attempt_id,
                )
                .unwrap()
                .is_empty());
        }

        let status = patrol
            .reconcile_contained_nonclean_team(&manifest, &[])
            .unwrap();
        assert_two_member_team_fully_contained(&state_dir, &ledger, &manifest, &status);
    }

    #[test]
    fn pane_close_validation_failure_contains_sibling_and_retains_evidence() {
        let (_tmp, state_dir, ledger, prepared, manifest, patrol) =
            two_member_team_fixture("pane-close-failure");
        let closing = &manifest.members[0];
        let attempt = prepared.authority.attempts[0].clone();
        crate::orchestration::transition_authoritative_attempt(
            &ledger,
            &attempt,
            crate::mission::TaskAttemptState::CandidateDone,
            &closing.owner,
        )
        .unwrap();
        Patrol::block_team_member_after_pane_close_failure(&ledger, &manifest, closing).unwrap();
        assert_eq!(
            ledger
                .task_attempt(&closing.attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Blocked
        );
        assert_eq!(
            ledger
                .task_attempt(&manifest.members[1].attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Running
        );

        let status = patrol
            .reconcile_contained_nonclean_team(&manifest, &[])
            .unwrap();
        assert_two_member_team_fully_contained(&state_dir, &ledger, &manifest, &status);
        assert!(ledger
            .events(&manifest.mission_id)
            .unwrap()
            .iter()
            .any(|event| {
                event
                    .resulting_task_attempt
                    .as_ref()
                    .is_some_and(|projection| {
                        projection.attempt_id == closing.attempt_id
                            && projection.state == crate::mission::TaskAttemptState::Blocked
                    })
            }));
    }

    #[test]
    fn detects_content_filter_block() {
        let pane = "working\nAPI Error: Output blocked by content filtering policy\n❯";
        assert_eq!(detect_fatal_agent_error(pane), Some("content-filter block"));
    }

    #[test]
    fn detects_hard_api_error() {
        assert_eq!(
            detect_fatal_agent_error("boom\nAPI Error: 500 internal\n❯"),
            Some("API error")
        );
    }

    #[test]
    fn ignores_retrying_and_normal_output() {
        assert_eq!(
            detect_fatal_agent_error("API Error: 529 overloaded, Retrying in 5s"),
            None
        );
        assert_eq!(detect_fatal_agent_error("just working on it\n❯"), None);
    }

    #[test]
    fn gate_upgrade_needs_complete_plan_and_passing_gate() {
        // The ONLY accepting shape: the plan is 100% and the independent gate
        // ran and passed.
        assert!(may_upgrade_gate_pending(true, Some(true)));
    }

    #[test]
    fn gate_upgrade_refused_when_gate_absent() {
        // The defect this predicate exists to close: patrol used to read only
        // the plan counters, so a mission refused ONLY because no GateResult
        // was ever written was auto-accepted on the next cycle.
        assert!(!may_upgrade_gate_pending(true, None));
    }

    #[test]
    fn gate_upgrade_refused_when_gate_failed() {
        // The gate ran and said no. An explicit refusal is at least as binding
        // as a missing one.
        assert!(!may_upgrade_gate_pending(true, Some(false)));
    }

    #[test]
    fn gate_upgrade_refused_when_plan_incomplete_whatever_the_gate() {
        // The plan condition is independent: no gate verdict rescues an
        // unfinished (or failed, per `is_complete`) plan.
        assert!(!may_upgrade_gate_pending(false, Some(true)));
        assert!(!may_upgrade_gate_pending(false, Some(false)));
        assert!(!may_upgrade_gate_pending(false, None));
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
    fn orphan_sweep_ignores_an_ancient_signal_that_never_governed_the_worker() {
        // Regression, measured 2026-08-04. `oracle-OmegaOS-4.done.json` was
        // done_clean and 3_185_365s (36.9 days) old. On the project-match branch
        // it kept matching every newly spawned `OmegaOS-worker-*`, and the sweep
        // closed three README workers within 60s of spawn — before any of them
        // could write a file or a done signal. Grace-elapsed alone said "reap".
        const INCIDENT_AGE_SECS: i64 = 3_185_365;
        const { assert!(INCIDENT_AGE_SECS > ORPHAN_SIGNAL_MAX_AGE_SECS) };
        assert!(
            !should_reap_orphan(true, INCIDENT_AGE_SECS),
            "a 36-day-old done signal must never authorize reaping a live worker"
        );

        // The window is closed at the top and stays open right up to it, so a
        // genuine orphan (swept within grace + one tick) is still reaped.
        assert!(should_reap_orphan(true, ORPHAN_SIGNAL_MAX_AGE_SECS));
        assert!(!should_reap_orphan(true, ORPHAN_SIGNAL_MAX_AGE_SECS + 1));
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
        assert!(signal_predates_session(
            spawn - chrono::Duration::hours(3),
            Some(spawn)
        ));
        assert!(signal_predates_session(
            spawn - chrono::Duration::seconds(1),
            Some(spawn)
        ));
        // Signal written BY this session (at or after spawn) → fresh.
        assert!(!signal_predates_session(spawn, Some(spawn)));
        assert!(!signal_predates_session(
            spawn + chrono::Duration::seconds(30),
            Some(spawn)
        ));
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
        use chrono::TimeZone;

        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mut first_signal = DoneSignal::new("w1", DoneStatus::DoneClean, "first");
        first_signal.finished_at = Utc.timestamp_opt(100, 100_000_000).single().unwrap();
        let first_key = worker_done_event_key("done_clean", &first_signal, None).unwrap();
        // Nothing sent yet.
        assert!(!inbox_event_already_sent(dir, "w1", "done", &first_key).unwrap());
        record_inbox_event_sent(dir, "w1", "done", &first_key).unwrap();
        // Same signal → already sent (no per-tick re-push).
        assert!(inbox_event_already_sent(dir, "w1", "done", &first_key).unwrap());

        // Two distinct signals in the same wall-clock second must never share
        // delivery authority. Nanoseconds and the full payload are digested.
        let mut second_signal = first_signal.clone();
        second_signal.finished_at = first_signal.finished_at + chrono::Duration::milliseconds(1);
        second_signal.summary = "second".to_string();
        assert_eq!(
            first_signal.finished_at.timestamp(),
            second_signal.finished_at.timestamp()
        );
        let second_key = worker_done_event_key("done_clean", &second_signal, None).unwrap();
        assert_ne!(first_key, second_key);
        assert!(!inbox_event_already_sent(dir, "w1", "done", &second_key).unwrap());

        let pending_key = worker_done_event_key("pending", &first_signal, None).unwrap();
        assert_ne!(first_key, pending_key);
        assert!(!inbox_event_already_sent(dir, "w1", "done", &pending_key).unwrap());
        // Kinds are independent.
        assert!(!inbox_event_already_sent(dir, "w1", "blocked", "digest").unwrap());
        remove_inbox_event_markers(dir, "w1").unwrap();
        assert!(!inbox_event_already_sent(dir, "w1", "done", &first_key).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn inbox_event_marker_refuses_symlink_and_preserves_target() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let victim = dir.join("victim");
        std::fs::write(&victim, b"operator-owned").unwrap();
        let marker = event_sent_path(dir, "w1", "done").unwrap();
        symlink(&victim, &marker).unwrap();

        assert!(inbox_event_already_sent(dir, "w1", "done", "digest").is_err());
        assert!(record_inbox_event_sent(dir, "w1", "done", "digest").is_err());
        assert_eq!(std::fs::read(&victim).unwrap(), b"operator-owned");
        assert!(std::fs::symlink_metadata(marker)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn retired_done_name_matcher() {
        assert!(is_retired_done_name(
            "oracle-OmegaOS-prev1765432100.done.json"
        ));
        // A live signal is never "retired", even for a project containing -prev.
        assert!(!is_retired_done_name("oracle-OmegaOS.done.json"));
        assert!(!is_retired_done_name("oracle-x-prevention.done.json"));
        assert!(!is_retired_done_name("oracle-x-prev.done.json"));
    }

    #[test]
    fn close_marker_is_idempotent_keeps_since() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).unwrap();
        let first = WorkerCloseMarker::read_all(dir).unwrap();
        assert_eq!(first.len(), 1);
        let since0 = first[0].since;
        // Re-ensure must NOT reset `since` (grace clock stability).
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).unwrap();
        let second = WorkerCloseMarker::read_all(dir).unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].since, since0);
        WorkerCloseMarker::remove(dir, "worker-x").unwrap();
        assert!(WorkerCloseMarker::read_all(dir).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn close_marker_refuses_symlink_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let victim = dir.join("victim.json");
        std::fs::write(&victim, b"operator-owned").unwrap();
        let marker_path = WorkerCloseMarker::path(dir, "worker-x").unwrap();
        symlink(&victim, &marker_path).unwrap();

        assert!(WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).is_err());
        assert_eq!(std::fs::read(&victim).unwrap(), b"operator-owned");
        assert!(std::fs::symlink_metadata(&marker_path)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn close_marker_refuses_hard_linked_authority() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        WorkerCloseMarker::ensure(dir, "worker-x", Some("oracle-X")).unwrap();
        let marker_path = WorkerCloseMarker::path(dir, "worker-x").unwrap();
        std::fs::hard_link(&marker_path, dir.join("alias.json")).unwrap();

        assert!(WorkerCloseMarker::read_all(dir).is_err());
        assert!(WorkerCloseMarker::remove(dir, "worker-x").is_err());
    }

    #[test]
    fn ignores_stale_error_in_scrollback() {
        let mut lines = vec!["API Error: Output blocked by content filtering policy"];
        lines.extend(std::iter::repeat_n("normal output line", 20));
        assert_eq!(detect_fatal_agent_error(&lines.join("\n")), None);
    }

    fn done_citing_sha(sha: &str) -> crate::done::DoneSignal {
        let mut done = crate::done::DoneSignal::stub("Proj-worker-x", DoneStatus::DoneClean);
        done.todos_total = 1;
        done.todos_completed = 1;
        done.artifacts.push(crate::done::DoneArtifact::GitSha {
            sha: sha.to_string(),
            branch: None,
        });
        done
    }

    /// MANDATORY TEST 1 (patrol half). A real SHA that could not be looked up
    /// (no repo root) must NOT contest the worker: patrol falls through to the
    /// `!verdict.passes` branch, which is `Pending` with the scope held — never
    /// `Failed`, never a contested-fabrication escalation.
    #[test]
    fn unverifiable_check_never_contests_a_worker() {
        let done = done_citing_sha("0123456789abcdef0123456789abcdef01234567");
        let verdict = crate::done::verify_done_against_repo(&done, None);
        assert!(
            !verdict_contests_worker(&verdict),
            "an unrun check was branded a fabrication: {:?}",
            verdict.failures
        );
        // The classification patrol then applies, verbatim from the call site.
        let effective = if verdict_contests_worker(&verdict) {
            DoneStatus::Failed
        } else if !verdict.passes {
            DoneStatus::Pending
        } else {
            done.status
        };
        assert_eq!(effective, DoneStatus::Pending);
    }

    /// MANDATORY TEST 2 (patrol half). The detector is NOT weakened: a bogus
    /// SHA looked up in a REAL repo still contests the worker and still lands
    /// on `Failed`.
    #[test]
    fn contradicted_check_still_contests_a_worker() {
        let temp = tempfile::tempdir().unwrap();
        let status = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .status()
            .expect("git init");
        assert!(status.success());
        let done = done_citing_sha("0123456789abcdef0123456789abcdef01234567");
        let verdict = crate::done::verify_done_against_repo(&done, Some(temp.path()));
        assert!(
            verdict_contests_worker(&verdict),
            "a proven fabrication must still be caught: {:?}",
            verdict.failures
        );
        let effective = if verdict_contests_worker(&verdict) {
            DoneStatus::Failed
        } else if !verdict.passes {
            DoneStatus::Pending
        } else {
            done.status
        };
        assert_eq!(effective, DoneStatus::Failed);
    }

    /// MANDATORY TEST 3. The registry (`projects.json`) is the real one, and
    /// `config.projects` is empty on a real machine — that split-brain is why
    /// `repo_root` was `None` in the first place. Uses a tempdir registry: the
    /// machine's own `~/.omega/projects.json` is never read or written
    /// (project_manager.rs:429 records the day a test clobbered it).
    #[test]
    fn repo_root_resolves_from_the_projects_registry_when_config_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let registry_path = tmp.path().join("projects.json");
        let project_path = tmp.path().join("Station/SideBusiness/OmegaOS");
        std::fs::create_dir_all(&project_path).unwrap();
        std::fs::write(
            &registry_path,
            serde_json::json!({
                "projects": [{
                    "name": "OmegaOS",
                    "path": project_path,
                    "telegram_topic_id": null,
                    "oracle_session": null,
                    "git_email": null,
                    "created_at": "2026-08-05T00:00:00Z"
                }]
            })
            .to_string(),
        )
        .unwrap();

        let config = OmegaConfig::default();
        assert!(
            config.find_project("OmegaOS").is_none(),
            "premise: config.projects does not carry the project"
        );
        assert_eq!(
            resolve_repo_root_in(&registry_path, &config, "OmegaOS"),
            Some(project_path),
            "the real registry must be consulted first"
        );
        // Unknown project, empty config → still None (no invented root).
        assert_eq!(
            resolve_repo_root_in(&registry_path, &config, "NotAProject"),
            None
        );
    }
}
