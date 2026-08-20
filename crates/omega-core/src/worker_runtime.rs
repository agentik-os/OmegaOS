//! Durable authority for one externally spawned worker runtime.
//!
//! A runtime is prepared and persisted before rmux/process creation. The
//! immutable intent binds the exact ledger attempt, launch identity and scope
//! credentials. Starting the observed process and accepting one candidate are
//! monotonic compare-and-swap transitions over that intent. Runtime files are
//! never diagnostic truth by themselves, but they are the crash-safe bridge
//! that lets ledger-driven callers identify and contain the exact external
//! effect they created.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use rmux_sdk::{PaneId, SessionId, WindowId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const WORKER_RUNTIME_SCHEMA_VERSION: u32 = 2;
const WORKER_RUNTIME_PREFIX: &str = "worker-runtime-";
const WORKER_RUNTIME_SUFFIX: &str = ".json";
const WORKER_RUNTIME_LOCK: &str = ".worker-runtime.lock";
const WORKER_RUNTIME_HISTORY_DIR: &str = "worker-runtime-history";
const WORKER_RUNTIME_HISTORY_SUFFIX: &str = ".terminal.json";
const WORKER_START_GATE_PREFIX: &str = ".worker-runtime-";
const WORKER_START_GATE_SUFFIX: &str = ".start-gate";
const MAX_WORKER_RUNTIME_BYTES: u64 = 1024 * 1024;
static WORKER_RUNTIME_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Exact ledger identity whose external worker effect this runtime represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerAttemptIdentity {
    pub mission_id: crate::mission::MissionId,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub task_id: String,
    pub attempt_id: String,
    pub owner: String,
}

/// Canonical launch coordinates supplied to [`WorkerRuntimeManifest::prepare`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerLaunchIdentity {
    pub session: String,
    /// Digest of the exact one-element daemon command vector containing the
    /// generation-gated shell command.
    pub expected_command_digest: String,
    /// Source/authority checkout shared with the parent mission.
    pub authority_workspace: PathBuf,
    /// Directory in which the provider process actually executes.
    pub execution_workspace: PathBuf,
    /// Required when execution occurs in an isolated git worktree.
    pub worktree: Option<WorkerWorktreeProvenance>,
    pub project: String,
    pub provider: String,
}

/// Immutable proof that an execution checkout is one registered worktree of
/// the source authority repository, captured before the worker can run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerWorktreeProvenance {
    pub git_common_dir: PathBuf,
    pub base_head: String,
    pub branch: String,
}

impl WorkerWorktreeProvenance {
    pub fn capture(authority_workspace: &Path, execution_workspace: &Path) -> Result<Self> {
        let authority_workspace =
            std::fs::canonicalize(authority_workspace).with_context(|| {
                format!(
                    "canonicalizing authority workspace {}",
                    authority_workspace.display()
                )
            })?;
        let execution_workspace =
            std::fs::canonicalize(execution_workspace).with_context(|| {
                format!(
                    "canonicalizing execution workspace {}",
                    execution_workspace.display()
                )
            })?;
        if authority_workspace == execution_workspace {
            bail!("worktree provenance requires distinct authority and execution workspaces");
        }
        let authority_common = git_canonical_path(&authority_workspace, "--git-common-dir")?;
        let execution_common = git_canonical_path(&execution_workspace, "--git-common-dir")?;
        if authority_common != execution_common {
            bail!("execution worktree does not share the authority repository git common-dir");
        }
        let base_head = git_text(&execution_workspace, &["rev-parse", "HEAD"])?;
        validate_git_object(&base_head, "worktree base HEAD")?;
        let branch = git_text(
            &execution_workspace,
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )?;
        validate_opaque_identity(&branch, "worktree branch", 512)?;
        let provenance = Self {
            git_common_dir: authority_common,
            base_head,
            branch,
        };
        provenance.verify(&authority_workspace, &execution_workspace)?;
        Ok(provenance)
    }

    fn verify(&self, authority_workspace: &Path, execution_workspace: &Path) -> Result<()> {
        validate_git_object(&self.base_head, "worktree base HEAD")?;
        validate_opaque_identity(&self.branch, "worktree branch", 512)?;
        let common = std::fs::canonicalize(&self.git_common_dir).with_context(|| {
            format!(
                "canonicalizing persisted git common-dir {}",
                self.git_common_dir.display()
            )
        })?;
        if common != self.git_common_dir
            || git_canonical_path(authority_workspace, "--git-common-dir")? != common
            || git_canonical_path(execution_workspace, "--git-common-dir")? != common
        {
            bail!("worker worktree no longer shares its persisted git common-dir");
        }
        if git_text(
            execution_workspace,
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )? != self.branch
        {
            bail!("worker execution worktree changed its bound branch");
        }
        Ok(())
    }
}

/// Prepared input. A dynamically spawned child binds its reciprocal ledger
/// edge afterward with [`WorkerRuntimeManifest::bind_parent_link`], because
/// that edge cites the stable digest produced by preparation itself.
#[derive(Debug, Clone)]
pub struct WorkerRuntimeIntent {
    pub attempt: WorkerAttemptIdentity,
    pub launch: WorkerLaunchIdentity,
    pub scope: Option<WorkerRuntimeScope>,
}

impl WorkerRuntimeIntent {
    pub fn runtime_generation(&self) -> Result<String> {
        validate_attempt_identity(&self.attempt)?;
        runtime_generation(&self.attempt)
    }

    /// Recommended rmux name. The persisted session also carries the complete
    /// generation separately, and global preparation rejects every duplicate
    /// exact name even when a caller does not use this helper.
    pub fn generation_scoped_session(&self, base: &str) -> Result<String> {
        generation_scoped_session(base, &self.runtime_generation()?)
    }
}

/// Exact prepared compatibility claim, independent of the mutable claim file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerScopeReceipt {
    pub schema_version: u32,
    pub mission_id: crate::mission::MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub owner: String,
    pub claim: crate::scope::ScopeClaim,
}

/// One normalized selector's exact ledger resource and fencing generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerResourceFence {
    pub selector: String,
    pub resource_key: String,
    pub fencing_token: u64,
}

/// Scope authority frozen into the runtime intent. The digest covers the
/// complete receipt, including the compatibility claim generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRuntimeScope {
    receipt: WorkerScopeReceipt,
    receipt_digest: String,
    resources: Vec<WorkerResourceFence>,
}

impl WorkerRuntimeScope {
    pub fn from_authority(
        receipt: WorkerScopeReceipt,
        leases: &[crate::mission_ledger::LeaseRecord],
    ) -> Result<Self> {
        validate_scope_receipt(&receipt)?;
        let workspace_id = receipt
            .claim
            .workspace_id
            .as_deref()
            .context("worker scope receipt has no canonical workspace identity")?;
        let selectors = receipt.claim.files_owned.clone();
        if leases.len() != selectors.len() {
            bail!(
                "worker scope receipt has {} selectors but {} leases",
                selectors.len(),
                leases.len()
            );
        }
        let mut by_resource = BTreeMap::new();
        for lease in leases {
            if lease.status != crate::mission_ledger::LeaseStatus::Active
                || lease.mission_id != receipt.mission_id
                || lease.task_id != receipt.task_id
                || lease.attempt_id != receipt.attempt_id
                || lease.owner != receipt.owner
                || lease.fencing_token == 0
            {
                bail!("worker scope lease differs from its exact receipt identity");
            }
            if by_resource
                .insert(lease.resource_key.clone(), lease.fencing_token)
                .is_some()
            {
                bail!("worker scope contains a duplicate lease resource");
            }
        }
        let mut resources = Vec::with_capacity(selectors.len());
        for selector in selectors {
            let resource_key = crate::scope::lease_resource_key(workspace_id, &selector);
            let fencing_token = by_resource.remove(&resource_key).ok_or_else(|| {
                anyhow::anyhow!(
                    "worker scope selector {selector} has no exact ledger resource/fence"
                )
            })?;
            resources.push(WorkerResourceFence {
                selector,
                resource_key,
                fencing_token,
            });
        }
        if !by_resource.is_empty() {
            bail!("worker scope contains lease resources outside its exact selector set");
        }
        let receipt_digest = scope_receipt_digest(&receipt)?;
        let scope = Self {
            receipt,
            receipt_digest,
            resources,
        };
        scope.verify_integrity()?;
        Ok(scope)
    }

    pub fn receipt(&self) -> &WorkerScopeReceipt {
        &self.receipt
    }

    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }

    pub fn resources(&self) -> &[WorkerResourceFence] {
        &self.resources
    }

    fn verify_integrity(&self) -> Result<()> {
        validate_scope_receipt(&self.receipt)?;
        validate_digest(&self.receipt_digest, "scope receipt digest")?;
        if scope_receipt_digest(&self.receipt)? != self.receipt_digest {
            bail!("worker scope receipt digest mismatch");
        }
        let workspace_id = self
            .receipt
            .claim
            .workspace_id
            .as_deref()
            .context("worker scope receipt has no workspace identity")?;
        if self.resources.len() != self.receipt.claim.files_owned.len() {
            bail!("worker scope resource set differs from its selectors");
        }
        let mut resource_keys = BTreeSet::new();
        for (selector, resource) in self.receipt.claim.files_owned.iter().zip(&self.resources) {
            if selector != &resource.selector
                || resource.resource_key != crate::scope::lease_resource_key(workspace_id, selector)
                || resource.fencing_token == 0
                || !resource_keys.insert(resource.resource_key.as_str())
            {
                bail!("worker scope resource/fence set is not exact and canonical");
            }
        }
        Ok(())
    }
}

/// Runtime session identity. `generation` is the immutable attempt-derived
/// generation, never an rmux name alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerSessionIdentity {
    pub session: String,
    pub generation: String,
}

/// Exact rmux process generation that activates a prepared intent.
///
/// A session name is an operator-facing label and can be reused. Authority is
/// therefore bound to the daemon's stable pane/session/window identities, the
/// pane generation and pid, the exact launch command, and the canonical
/// workspace. `observation_digest` makes that complete tuple deterministic
/// and tamper-evident when it crosses a process boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWorkerProcess {
    pub session: String,
    pub pane_id: PaneId,
    pub session_id: SessionId,
    pub window_id: WindowId,
    pub process_generation: u64,
    pub process_pid: u32,
    pub command_digest: String,
    pub working_dir: PathBuf,
    pub observation_digest: String,
}

impl ObservedWorkerProcess {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session: impl Into<String>,
        pane_id: PaneId,
        session_id: SessionId,
        window_id: WindowId,
        process_generation: u64,
        process_pid: u32,
        command_digest: impl Into<String>,
        working_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let working_dir = std::fs::canonicalize(working_dir.as_ref()).with_context(|| {
            format!(
                "canonicalizing observed worker workspace {}",
                working_dir.as_ref().display()
            )
        })?;
        let mut observed = Self {
            session: session.into(),
            pane_id,
            session_id,
            window_id,
            process_generation,
            process_pid,
            command_digest: command_digest.into(),
            working_dir,
            observation_digest: String::new(),
        };
        observed.observation_digest = observed.computed_digest()?;
        observed.validate()?;
        Ok(observed)
    }

    fn validate(&self) -> Result<()> {
        crate::scope::validate_session_identity(&self.session)?;
        if self.process_generation == 0 || self.process_pid == 0 {
            bail!("observed worker process has no live generation or pid");
        }
        validate_digest(&self.command_digest, "worker launch command digest")?;
        let canonical = std::fs::canonicalize(&self.working_dir).with_context(|| {
            format!(
                "canonicalizing persisted worker workspace {}",
                self.working_dir.display()
            )
        })?;
        if canonical != self.working_dir {
            bail!("observed worker workspace is not canonical");
        }
        validate_digest(
            &self.observation_digest,
            "worker process observation digest",
        )?;
        if self.computed_digest()? != self.observation_digest {
            bail!("worker process observation digest mismatch");
        }
        Ok(())
    }

    fn computed_digest(&self) -> Result<String> {
        Ok(blake3::hash(&serde_json::to_vec(&(
            "omega.worker-process-observation.v2",
            &self.session,
            self.pane_id,
            self.session_id,
            self.window_id,
            self.process_generation,
            self.process_pid,
            &self.command_digest,
            &self.working_dir,
        ))?)
        .to_hex()
        .to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRuntimeStarted {
    pub observed: ObservedWorkerProcess,
    pub activated_at: DateTime<Utc>,
}

/// Caller-supplied immutable candidate identity. `payload_digest` commits to
/// the exact completion/event payload stored elsewhere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerCandidateIdentity {
    pub candidate_id: String,
    pub payload_digest: String,
}

impl WorkerCandidateIdentity {
    pub fn new(candidate_id: impl Into<String>, payload_digest: impl Into<String>) -> Result<Self> {
        let candidate = Self {
            candidate_id: candidate_id.into(),
            payload_digest: payload_digest.into(),
        };
        candidate.validate()?;
        Ok(candidate)
    }

    fn validate(&self) -> Result<()> {
        validate_opaque_identity(&self.candidate_id, "candidate identity", 512)?;
        validate_digest(&self.payload_digest, "candidate payload digest")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerCandidateBinding {
    pub identity: WorkerCandidateIdentity,
    pub bound_at: DateTime<Utc>,
}

/// Daemon-backed proof that the generation-scoped rmux aggregate is absent.
/// Only [`crate::session::SessionManager`] can mint this after a fresh
/// inventory probe. The digest prevents a caller from rebinding a proof to a
/// different runtime or session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmedWorkerAbsence {
    schema_version: u32,
    runtime_id: String,
    session: String,
    confirmed_at: DateTime<Utc>,
    evidence_digest: String,
}

impl ConfirmedWorkerAbsence {
    pub(crate) fn new(runtime: &WorkerRuntimeManifest) -> Result<Self> {
        let mut evidence = Self {
            schema_version: 1,
            runtime_id: runtime.runtime_id.clone(),
            session: runtime.session.session.clone(),
            confirmed_at: Utc::now(),
            evidence_digest: String::new(),
        };
        evidence.evidence_digest = evidence.computed_digest()?;
        evidence.verify(runtime)?;
        Ok(evidence)
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn session(&self) -> &str {
        &self.session
    }

    pub fn confirmed_at(&self) -> DateTime<Utc> {
        self.confirmed_at
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }

    pub fn proves(&self, runtime: &WorkerRuntimeManifest) -> Result<()> {
        self.verify(runtime)
    }

    fn computed_digest(&self) -> Result<String> {
        Ok(blake3::hash(&serde_json::to_vec(&(
            "omega.worker-absence.v1",
            self.schema_version,
            &self.runtime_id,
            &self.session,
            self.confirmed_at,
        ))?)
        .to_hex()
        .to_string())
    }

    fn verify(&self, runtime: &WorkerRuntimeManifest) -> Result<()> {
        if self.schema_version != 1
            || self.runtime_id != runtime.runtime_id
            || self.session != runtime.session.session
        {
            bail!("worker absence proof differs from its runtime generation");
        }
        validate_digest(&self.evidence_digest, "worker absence evidence digest")?;
        if self.computed_digest()? != self.evidence_digest {
            bail!("worker absence evidence digest mismatch");
        }
        Ok(())
    }
}

/// Terminal ledger and process evidence sealed into the historical runtime
/// record before the live manifest can leave the active registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRuntimeTerminalReceipt {
    pub schema_version: u32,
    pub runtime_id: String,
    pub mission_state: crate::mission::MissionState,
    pub attempt_state: crate::mission::TaskAttemptState,
    pub absence: ConfirmedWorkerAbsence,
    pub retired_at: DateTime<Utc>,
    pub receipt_digest: String,
}

/// Immutable historical evidence. The original manifest is retained as the
/// exact JSON document that was strictly verified while its workspaces still
/// existed, so history remains verifiable after a disposable worktree is
/// safely removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRuntimeArchive {
    pub schema_version: u32,
    pub runtime_id: String,
    pub manifest_document: String,
    pub manifest_source_digest: String,
    pub terminal: WorkerRuntimeTerminalReceipt,
    pub archive_digest: String,
}

#[derive(Debug, Clone, Default)]
pub struct WorkerRuntimeHistoryInventory {
    pub archives: Vec<WorkerRuntimeArchive>,
    pub corrupt_entries: Vec<CorruptWorkerRuntimeEntry>,
}

impl WorkerRuntimeHistoryInventory {
    pub fn is_clean(&self) -> bool {
        self.corrupt_entries.is_empty()
    }

    pub fn require_clean(&self) -> Result<()> {
        if self.corrupt_entries.is_empty() {
            return Ok(());
        }
        bail!(
            "worker runtime history contains corrupt entries: {}",
            self.corrupt_entries
                .iter()
                .map(|entry| format!("{} ({})", entry.filename, entry.error))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl WorkerRuntimeTerminalReceipt {
    fn computed_digest(&self) -> Result<String> {
        Ok(blake3::hash(&serde_json::to_vec(&(
            "omega.worker-runtime-terminal.v1",
            self.schema_version,
            &self.runtime_id,
            self.mission_state,
            self.attempt_state,
            &self.absence,
            self.retired_at,
        ))?)
        .to_hex()
        .to_string())
    }

    fn seal(&mut self) -> Result<()> {
        self.receipt_digest = self.computed_digest()?;
        Ok(())
    }
}

impl WorkerRuntimeArchive {
    fn computed_digest(&self) -> Result<String> {
        Ok(blake3::hash(&serde_json::to_vec(&(
            "omega.worker-runtime-archive.v1",
            self.schema_version,
            &self.runtime_id,
            &self.manifest_document,
            &self.manifest_source_digest,
            &self.terminal,
        ))?)
        .to_hex()
        .to_string())
    }

    fn seal(&mut self) -> Result<()> {
        self.archive_digest = self.computed_digest()?;
        Ok(())
    }

    fn verify_integrity(&self) -> Result<()> {
        if self.schema_version != 1 {
            bail!("unsupported worker runtime archive schema");
        }
        validate_runtime_id(&self.runtime_id)?;
        validate_digest(
            &self.manifest_source_digest,
            "archived worker manifest source digest",
        )?;
        if blake3::hash(self.manifest_document.as_bytes())
            .to_hex()
            .as_str()
            != self.manifest_source_digest
        {
            bail!("archived worker manifest source digest mismatch");
        }
        let manifest: WorkerRuntimeManifest = serde_json::from_str(&self.manifest_document)
            .context("parsing archived worker manifest document")?;
        if manifest.schema_version != WORKER_RUNTIME_SCHEMA_VERSION
            || manifest.runtime_id != self.runtime_id
            || manifest.session.generation != self.runtime_id
        {
            bail!("archived worker manifest identity mismatch");
        }
        validate_digest(&manifest.intent_digest, "archived worker intent digest")?;
        validate_digest(&manifest.manifest_digest, "archived worker manifest digest")?;
        validate_digest(&manifest.document_digest, "archived worker document digest")?;
        if manifest.computed_intent_digest()? != manifest.intent_digest
            || manifest.computed_manifest_digest()? != manifest.manifest_digest
            || manifest.computed_document_digest()? != manifest.document_digest
        {
            bail!("archived worker manifest cryptographic integrity mismatch");
        }
        if self.terminal.schema_version != 1
            || self.terminal.runtime_id != self.runtime_id
            || !self.terminal.mission_state.is_terminal()
            || !self.terminal.attempt_state.is_terminal()
        {
            bail!("worker runtime archive has no terminal ledger receipt");
        }
        self.terminal.absence.verify(&manifest)?;
        validate_digest(
            &self.terminal.receipt_digest,
            "worker runtime terminal receipt digest",
        )?;
        if self.terminal.computed_digest()? != self.terminal.receipt_digest {
            bail!("worker runtime terminal receipt digest mismatch");
        }
        validate_digest(&self.archive_digest, "worker runtime archive digest")?;
        if self.computed_digest()? != self.archive_digest {
            bail!("worker runtime archive digest mismatch");
        }
        Ok(())
    }

    /// Remove only launch-time private artifacts after the archive itself is
    /// durably readable and the runtime is absent from the active registry.
    pub fn cleanup_launch_artifacts(&self, state_dir: &Path) -> Result<()> {
        self.verify_integrity()?;
        let persisted = WorkerRuntimeManifest::load_history_strict(state_dir, &self.runtime_id)?
            .context("worker runtime archive disappeared before launch-artifact cleanup")?;
        if persisted.archive_digest != self.archive_digest {
            bail!("worker runtime archive changed before launch-artifact cleanup");
        }
        if WorkerRuntimeManifest::load_strict(state_dir, &self.runtime_id)?.is_some() {
            bail!("active worker runtime still exists before launch-artifact cleanup");
        }
        crate::scope::remove_private_file(&worker_start_gate_path(state_dir, &self.runtime_id)?)?;
        crate::scope::remove_private_file(&state_dir.join(format!("{}.mcp.json", self.runtime_id)))
    }
}

/// Immutable intent plus monotonic activation/candidate bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRuntimeManifest {
    schema_version: u32,
    runtime_id: String,
    storage_generation: u64,
    intent_digest: String,
    attempt: WorkerAttemptIdentity,
    session: WorkerSessionIdentity,
    authority_workspace: PathBuf,
    authority_workspace_id: String,
    execution_workspace: PathBuf,
    execution_workspace_id: String,
    worktree: Option<WorkerWorktreeProvenance>,
    expected_command_digest: String,
    project: String,
    provider: String,
    scope: Option<WorkerRuntimeScope>,
    parent_link: Option<crate::mission_ledger::ChildMissionLink>,
    prepared_at: DateTime<Utc>,
    started: Option<WorkerRuntimeStarted>,
    candidate: Option<WorkerCandidateBinding>,
    manifest_digest: String,
    document_digest: String,
    #[serde(skip)]
    source_generation: Option<u64>,
    #[serde(skip)]
    source_digest: Option<String>,
}

impl WorkerRuntimeManifest {
    /// Persist an immutable intent before any external session/process effect.
    /// Exact retries are idempotent. Any changed binding for the same attempt,
    /// duplicate session or unreadable sibling fails closed under the global
    /// runtime lock.
    pub fn prepare(state_dir: &Path, intent: WorkerRuntimeIntent) -> Result<Self> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        let prepared = Self::from_intent(intent)?;
        let inventory = list_worker_runtimes_locked(state_dir)?;
        inventory.require_clean()?;
        if let Some(current) = inventory
            .manifests
            .iter()
            .find(|runtime| runtime.runtime_id == prepared.runtime_id)
        {
            if current.intent_digest == prepared.intent_digest {
                return Ok(current.clone());
            }
            bail!(
                "worker runtime {} already exists with a different immutable intent",
                prepared.runtime_id
            );
        }
        if inventory
            .manifests
            .iter()
            .any(|runtime| runtime.session.session == prepared.session.session)
        {
            bail!(
                "worker session {} is already bound to another runtime generation",
                prepared.session.session
            );
        }
        publish_new_locked(state_dir, &prepared)
    }

    pub fn load_strict(state_dir: &Path, runtime_id: &str) -> Result<Option<Self>> {
        validate_runtime_id(runtime_id)?;
        crate::scope::ensure_private_state_dir(state_dir)?;
        load_worker_runtime_path(&runtime_path(state_dir, runtime_id)?, Some(runtime_id))
    }

    /// Strict isolated inventory. Every matching file is either a verified
    /// manifest or an explicit corrupt entry; nothing is silently omitted.
    pub fn list_strict(state_dir: &Path) -> Result<WorkerRuntimeInventory> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        list_worker_runtimes_locked(state_dir)
    }

    /// Seal one terminal runtime into the private historical namespace and
    /// remove it from the active collision/inventory set. The caller must
    /// first prove daemon absence, terminalize both ledger machines and
    /// release every lease and compatibility claim. Exact retries are
    /// idempotent, including recovery from a crash after archive publication
    /// but before active-file removal.
    pub fn retire_terminal(
        &self,
        state_dir: &Path,
        absence: &ConfirmedWorkerAbsence,
    ) -> Result<WorkerRuntimeArchive> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        if let Some(archived) = load_worker_runtime_archive_path(
            &worker_runtime_archive_path(state_dir, &self.runtime_id)?,
            Some(&self.runtime_id),
        )? {
            if archived.manifest_source_digest
                != self
                    .source_digest
                    .as_deref()
                    .context("terminal retirement requires a strictly loaded manifest")?
            {
                bail!("worker runtime already has different terminal archive evidence");
            }
            absence.proves(self)?;
            crate::scope::remove_private_file(&runtime_path(state_dir, &self.runtime_id)?)?;
            return Ok(archived);
        }

        let current = Self::load_strict(state_dir, &self.runtime_id)?
            .context("worker runtime disappeared before terminal retirement")?;
        let expected_generation = self
            .source_generation
            .context("terminal retirement requires a strictly loaded source generation")?;
        let expected_digest = self
            .source_digest
            .as_deref()
            .context("terminal retirement requires a strictly loaded source digest")?;
        if current.storage_generation != expected_generation
            || current.source_digest.as_deref() != Some(expected_digest)
        {
            bail!("stale worker runtime terminal retirement refused");
        }
        absence.proves(&current)?;

        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(state_dir),
        )?;
        let mission = ledger
            .mission(&current.attempt.mission_id)?
            .context("worker mission disappeared before terminal retirement")?;
        let attempt = ledger
            .task_attempt(&current.attempt.attempt_id)?
            .context("worker attempt disappeared before terminal retirement")?;
        if attempt.mission_id != current.attempt.mission_id
            || attempt.task_id != current.attempt.task_id
            || attempt.plan_revision != current.attempt.plan_revision
        {
            bail!("terminal worker ledger identity differs from its runtime");
        }
        if !mission.state.is_terminal() || !attempt.state.is_terminal() {
            bail!("worker runtime cannot retire before mission and attempt are terminal");
        }
        if !ledger
            .active_leases_for_attempt(
                &current.attempt.mission_id,
                &current.attempt.task_id,
                &current.attempt.attempt_id,
            )?
            .is_empty()
        {
            bail!("worker runtime cannot retire while ledger leases remain active");
        }
        if crate::scope::ScopeClaim::read_strict(state_dir, &current.attempt.owner)?.is_some() {
            bail!("worker runtime cannot retire while its compatibility claim remains active");
        }

        let active_path = runtime_path(state_dir, &current.runtime_id)?;
        let source = read_private_bounded(&active_path)?
            .context("worker runtime disappeared while building terminal archive")?;
        if blake3::hash(&source).to_hex().as_str() != expected_digest {
            bail!("worker runtime changed while building terminal archive");
        }
        let manifest_document = String::from_utf8(source)
            .context("strict worker runtime document is not UTF-8 JSON")?;
        let mut terminal = WorkerRuntimeTerminalReceipt {
            schema_version: 1,
            runtime_id: current.runtime_id.clone(),
            mission_state: mission.state,
            attempt_state: attempt.state,
            absence: absence.clone(),
            retired_at: Utc::now(),
            receipt_digest: String::new(),
        };
        terminal.seal()?;
        let mut archive = WorkerRuntimeArchive {
            schema_version: 1,
            runtime_id: current.runtime_id.clone(),
            manifest_document,
            manifest_source_digest: expected_digest.to_string(),
            terminal,
            archive_digest: String::new(),
        };
        archive.seal()?;
        archive.verify_integrity()?;
        let archive_path = worker_runtime_archive_path(state_dir, &current.runtime_id)?;
        atomic_write_private(&archive_path, &serde_json::to_vec_pretty(&archive)?)?;
        let archived = load_worker_runtime_archive_path(&archive_path, Some(&current.runtime_id))?
            .context("worker runtime archive disappeared after publication")?;
        if archived.archive_digest != archive.archive_digest {
            bail!("worker runtime archive changed during publication");
        }
        crate::scope::remove_private_file(&active_path)?;
        if Self::load_strict(state_dir, &current.runtime_id)?.is_some() {
            bail!("terminal worker runtime remained in the active registry");
        }
        Ok(archived)
    }

    pub fn load_history_strict(
        state_dir: &Path,
        runtime_id: &str,
    ) -> Result<Option<WorkerRuntimeArchive>> {
        load_worker_runtime_archive_path(
            &worker_runtime_archive_path(state_dir, runtime_id)?,
            Some(runtime_id),
        )
    }

    pub fn list_history_strict(state_dir: &Path) -> Result<WorkerRuntimeHistoryInventory> {
        list_worker_runtime_history(state_dir)
    }

    /// Resolve only after the complete inventory proves global uniqueness.
    pub fn resolve_session_strict(state_dir: &Path, session: &str) -> Result<Option<Self>> {
        crate::scope::validate_session_identity(session)?;
        let inventory = Self::list_strict(state_dir)?;
        inventory.require_clean()?;
        Ok(inventory
            .manifests
            .into_iter()
            .find(|runtime| runtime.session.session == session))
    }

    pub fn resolve_attempt_strict(
        state_dir: &Path,
        mission_id: &crate::mission::MissionId,
        attempt_id: &str,
    ) -> Result<Option<Self>> {
        validate_opaque_identity(attempt_id, "attempt identity", 512)?;
        let inventory = Self::list_strict(state_dir)?;
        inventory.require_clean()?;
        let matches = inventory
            .manifests
            .into_iter()
            .filter(|runtime| {
                runtime.attempt.mission_id == *mission_id
                    && runtime.attempt.attempt_id == attempt_id
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [runtime] => Ok(Some(runtime.clone())),
            _ => bail!("multiple worker runtimes bind the same mission attempt"),
        }
    }

    /// Activate only the exact process observed for this prepared session.
    /// Replaying the same observation is idempotent; a different generation is
    /// an immutable conflict and never replaces the first effect.
    pub fn activate_started(
        &self,
        state_dir: &Path,
        observed: ObservedWorkerProcess,
    ) -> Result<Self> {
        observed.validate()?;
        crate::scope::ensure_private_state_dir(state_dir)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        let current = Self::load_strict(state_dir, &self.runtime_id)?
            .context("prepared worker runtime disappeared before activation")?;
        if observed.session != current.session.session {
            bail!(
                "observed session {} differs from prepared session {}",
                observed.session,
                current.session.session
            );
        }
        if observed.command_digest != current.expected_command_digest
            || observed.working_dir != current.execution_workspace
        {
            bail!("observed process differs from the immutable prepared launch command/workspace");
        }
        if let Some(started) = &current.started {
            if started.observed == observed {
                return Ok(current);
            }
            bail!("worker runtime already started with a different process generation");
        }
        let inventory = list_worker_runtimes_locked(state_dir)?;
        inventory.require_clean()?;
        let mut next = current.clone();
        next.started = Some(WorkerRuntimeStarted {
            observed,
            activated_at: Utc::now(),
        });
        publish_transition_locked(state_dir, self, next)
    }

    /// Bind the reciprocal child edge after preparation has produced the
    /// stable digest that the ledger edge cites, but before any external
    /// effect is activated. Exact replay is idempotent; a second edge cannot
    /// replace the first.
    pub fn bind_parent_link(
        &self,
        state_dir: &Path,
        link: crate::mission_ledger::ChildMissionLink,
    ) -> Result<Self> {
        crate::scope::ensure_private_state_dir(state_dir)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        let current = Self::load_strict(state_dir, &self.runtime_id)?
            .context("worker runtime disappeared before parent-link binding")?;
        validate_parent_link(
            &link,
            &current.attempt,
            &current.session.session,
            &current.project,
            &current.authority_workspace_id,
            &current.manifest_digest,
        )?;
        if let Some(bound) = &current.parent_link {
            if bound == &link {
                return Ok(current);
            }
            bail!("worker runtime already binds a different immutable parent link");
        }
        if current.started.is_some() || current.candidate.is_some() {
            bail!("worker runtime cannot bind a parent link after external activation");
        }
        let mut next = current.clone();
        next.parent_link = Some(link);
        publish_transition_locked(state_dir, self, next)
    }

    /// Bind the first exact candidate only after activation. Exact replay is
    /// idempotent; every different candidate is rejected permanently.
    pub fn bind_candidate(
        &self,
        state_dir: &Path,
        candidate: WorkerCandidateIdentity,
    ) -> Result<Self> {
        candidate.validate()?;
        crate::scope::ensure_private_state_dir(state_dir)?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        let current = Self::load_strict(state_dir, &self.runtime_id)?
            .context("worker runtime disappeared before candidate binding")?;
        if let Some(bound) = &current.candidate {
            if bound.identity == candidate {
                return Ok(current);
            }
            bail!("worker runtime already binds a different immutable candidate");
        }
        if current.started.is_none() {
            bail!("prepared worker runtime cannot bind a candidate before activation");
        }
        let mut next = current.clone();
        next.candidate = Some(WorkerCandidateBinding {
            identity: candidate,
            bound_at: Utc::now(),
        });
        publish_transition_locked(state_dir, self, next)
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn storage_generation(&self) -> u64 {
        self.storage_generation
    }

    pub fn intent_digest(&self) -> &str {
        &self.intent_digest
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn document_digest(&self) -> &str {
        &self.document_digest
    }

    pub fn attempt(&self) -> &WorkerAttemptIdentity {
        &self.attempt
    }

    pub fn session(&self) -> &WorkerSessionIdentity {
        &self.session
    }

    pub fn workspace(&self) -> &Path {
        &self.execution_workspace
    }

    pub fn workspace_id(&self) -> &str {
        &self.execution_workspace_id
    }

    pub fn authority_workspace(&self) -> &Path {
        &self.authority_workspace
    }

    pub fn authority_workspace_id(&self) -> &str {
        &self.authority_workspace_id
    }

    pub fn worktree_provenance(&self) -> Option<&WorkerWorktreeProvenance> {
        self.worktree.as_ref()
    }

    pub fn expected_command_digest(&self) -> &str {
        &self.expected_command_digest
    }

    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn scope(&self) -> Option<&WorkerRuntimeScope> {
        self.scope.as_ref()
    }

    pub fn parent_link(&self) -> Option<&crate::mission_ledger::ChildMissionLink> {
        self.parent_link.as_ref()
    }

    pub fn prepared_at(&self) -> DateTime<Utc> {
        self.prepared_at
    }

    pub fn started(&self) -> Option<&WorkerRuntimeStarted> {
        self.started.as_ref()
    }

    pub fn candidate(&self) -> Option<&WorkerCandidateBinding> {
        self.candidate.as_ref()
    }

    /// Deterministic private barrier used by both launch and crash recovery.
    /// The path is derived solely from the immutable runtime generation.
    pub fn start_gate_path(&self, state_dir: &Path) -> Result<PathBuf> {
        worker_start_gate_path(state_dir, &self.runtime_id)
    }

    pub fn start_gate_value(&self) -> &str {
        &self.runtime_id
    }

    /// Publish the exact immutable runtime generation. Exact replay is
    /// idempotent; a different pre-existing value fails closed.
    pub fn release_start_gate(&self, state_dir: &Path) -> Result<()> {
        self.verify_integrity()?;
        let _lock = crate::scope::lock_private_state_file(state_dir, WORKER_RUNTIME_LOCK)?;
        let path = self.start_gate_path(state_dir)?;
        if let Some(bytes) = read_private_bounded(&path)? {
            if bytes == self.runtime_id.as_bytes() {
                return Ok(());
            }
            bail!("worker start gate already contains a different generation value");
        }
        atomic_write_private(&path, self.runtime_id.as_bytes())?;
        let recorded = read_private_bounded(&path)?
            .context("worker start gate disappeared after publication")?;
        if recorded != self.runtime_id.as_bytes() {
            bail!("worker start gate changed while being published");
        }
        Ok(())
    }

    pub fn is_start_gate_released(&self, state_dir: &Path) -> Result<bool> {
        self.verify_integrity()?;
        let path = self.start_gate_path(state_dir)?;
        match read_private_bounded(&path)? {
            None => Ok(false),
            Some(bytes) if bytes == self.runtime_id.as_bytes() => Ok(true),
            Some(_) => bail!("worker start gate contains a different generation value"),
        }
    }

    fn from_intent(intent: WorkerRuntimeIntent) -> Result<Self> {
        validate_attempt_identity(&intent.attempt)?;
        crate::scope::validate_session_identity(&intent.launch.session)?;
        validate_digest(
            &intent.launch.expected_command_digest,
            "expected worker launch command digest",
        )?;
        crate::scope::validate_session_identity(&intent.launch.project)
            .context("invalid worker runtime project identity")?;
        let provider = crate::agents::Agent::from_name(&intent.launch.provider)
            .context("unknown worker runtime provider")?
            .name()
            .to_string();
        if provider != intent.launch.provider {
            bail!(
                "worker runtime provider {} is not canonical; use {}",
                intent.launch.provider,
                provider
            );
        }
        let authority_workspace = std::fs::canonicalize(&intent.launch.authority_workspace)
            .with_context(|| {
                format!(
                    "canonicalizing worker authority workspace {}",
                    intent.launch.authority_workspace.display()
                )
            })?;
        let execution_workspace = std::fs::canonicalize(&intent.launch.execution_workspace)
            .with_context(|| {
                format!(
                    "canonicalizing worker execution workspace {}",
                    intent.launch.execution_workspace.display()
                )
            })?;
        let authority_metadata = std::fs::metadata(&authority_workspace).with_context(|| {
            format!(
                "inspecting worker authority workspace {}",
                authority_workspace.display()
            )
        })?;
        let execution_metadata = std::fs::metadata(&execution_workspace).with_context(|| {
            format!(
                "inspecting worker execution workspace {}",
                execution_workspace.display()
            )
        })?;
        if !authority_metadata.is_dir() || !execution_metadata.is_dir() {
            bail!("worker authority and execution workspaces must both be directories");
        }
        match (&intent.launch.worktree, authority_workspace == execution_workspace) {
            (None, true) => {}
            (Some(provenance), false) => {
                provenance.verify(&authority_workspace, &execution_workspace)?;
            }
            (None, false) => bail!(
                "distinct authority/execution workspaces require immutable worktree provenance"
            ),
            (Some(_), true) => bail!(
                "worktree provenance is invalid when authority and execution workspaces are identical"
            ),
        }
        let authority_workspace_id =
            crate::scope::canonical_workspace_identity(&authority_workspace)?;
        let execution_workspace_id =
            crate::scope::canonical_workspace_identity(&execution_workspace)?;
        let runtime_id = runtime_generation(&intent.attempt)?;
        let session = WorkerSessionIdentity {
            session: intent.launch.session,
            generation: runtime_id.clone(),
        };
        if let Some(scope) = &intent.scope {
            scope.verify_integrity()?;
            validate_scope_against_attempt(
                scope,
                &intent.attempt,
                &session,
                &execution_workspace_id,
            )?;
        }
        let mut manifest = Self {
            schema_version: WORKER_RUNTIME_SCHEMA_VERSION,
            runtime_id,
            storage_generation: 1,
            intent_digest: String::new(),
            attempt: intent.attempt,
            session,
            authority_workspace,
            authority_workspace_id,
            execution_workspace,
            execution_workspace_id,
            worktree: intent.launch.worktree,
            expected_command_digest: intent.launch.expected_command_digest,
            project: intent.launch.project,
            provider,
            scope: intent.scope,
            parent_link: None,
            prepared_at: Utc::now(),
            started: None,
            candidate: None,
            manifest_digest: String::new(),
            document_digest: String::new(),
            source_generation: None,
            source_digest: None,
        };
        manifest.intent_digest = manifest.computed_intent_digest()?;
        manifest.seal()?;
        manifest.verify_integrity()?;
        Ok(manifest)
    }

    fn computed_intent_digest(&self) -> Result<String> {
        let bytes = serde_json::to_vec(&(
            "omega.worker-runtime-intent.v2",
            self.schema_version,
            &self.runtime_id,
            &self.attempt,
            &self.session,
            &self.authority_workspace,
            &self.authority_workspace_id,
            &self.execution_workspace,
            &self.execution_workspace_id,
            &self.worktree,
            &self.expected_command_digest,
            &self.project,
            &self.provider,
            &self.scope,
        ))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    fn computed_manifest_digest(&self) -> Result<String> {
        let bytes = serde_json::to_vec(&(
            "omega.worker-runtime-manifest.v2",
            self.schema_version,
            &self.runtime_id,
            &self.intent_digest,
            &self.attempt,
            &self.session,
            &self.authority_workspace,
            &self.authority_workspace_id,
            &self.execution_workspace,
            &self.execution_workspace_id,
            &self.worktree,
            &self.expected_command_digest,
            &self.project,
            &self.provider,
            &self.scope,
            self.prepared_at,
        ))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    fn computed_document_digest(&self) -> Result<String> {
        let mut unsigned = self.clone();
        unsigned.document_digest.clear();
        unsigned.source_generation = None;
        unsigned.source_digest = None;
        Ok(blake3::hash(&serde_json::to_vec(&unsigned)?)
            .to_hex()
            .to_string())
    }

    fn seal(&mut self) -> Result<()> {
        self.manifest_digest = self.computed_manifest_digest()?;
        self.document_digest = self.computed_document_digest()?;
        Ok(())
    }

    fn verify_integrity(&self) -> Result<()> {
        if self.schema_version != WORKER_RUNTIME_SCHEMA_VERSION {
            bail!(
                "unsupported worker runtime schema {}; expected {}",
                self.schema_version,
                WORKER_RUNTIME_SCHEMA_VERSION
            );
        }
        validate_runtime_id(&self.runtime_id)?;
        if self.storage_generation == 0 {
            bail!("worker runtime storage generation must be positive");
        }
        validate_attempt_identity(&self.attempt)?;
        if runtime_generation(&self.attempt)? != self.runtime_id {
            bail!("worker runtime generation differs from its attempt identity");
        }
        crate::scope::validate_session_identity(&self.session.session)?;
        if self.session.generation != self.runtime_id {
            bail!("worker session generation differs from its runtime");
        }
        validate_canonical_workspace(&self.authority_workspace, &self.authority_workspace_id)?;
        validate_canonical_workspace(&self.execution_workspace, &self.execution_workspace_id)?;
        match (
            &self.worktree,
            self.authority_workspace == self.execution_workspace,
        ) {
            (None, true) => {}
            (Some(provenance), false) => {
                provenance.verify(&self.authority_workspace, &self.execution_workspace)?;
            }
            _ => bail!("worker worktree provenance does not match its workspace topology"),
        }
        validate_digest(
            &self.expected_command_digest,
            "expected worker launch command digest",
        )?;
        crate::scope::validate_session_identity(&self.project)
            .context("invalid worker runtime project identity")?;
        let provider = crate::agents::Agent::from_name(&self.provider)
            .context("unknown worker runtime provider")?;
        if provider.name() != self.provider {
            bail!("worker runtime provider is not canonical");
        }
        if let Some(scope) = &self.scope {
            scope.verify_integrity()?;
            validate_scope_against_attempt(
                scope,
                &self.attempt,
                &self.session,
                &self.execution_workspace_id,
            )?;
        }
        if let Some(link) = &self.parent_link {
            validate_parent_link(
                link,
                &self.attempt,
                &self.session.session,
                &self.project,
                &self.authority_workspace_id,
                &self.manifest_digest,
            )?;
        }
        validate_digest(&self.intent_digest, "worker intent digest")?;
        if self.computed_intent_digest()? != self.intent_digest {
            bail!("worker runtime intent digest mismatch");
        }
        if let Some(started) = &self.started {
            started.observed.validate()?;
            if started.observed.session != self.session.session {
                bail!("started observation differs from the exact prepared session");
            }
            if started.observed.working_dir != self.execution_workspace {
                bail!("started observation differs from the prepared canonical workspace");
            }
            if started.observed.command_digest != self.expected_command_digest {
                bail!("started observation differs from the prepared launch command");
            }
        }
        if let Some(candidate) = &self.candidate {
            if self.started.is_none() {
                bail!("worker candidate exists without a started activation");
            }
            candidate.identity.validate()?;
        }
        validate_digest(&self.manifest_digest, "worker manifest digest")?;
        if self.computed_manifest_digest()? != self.manifest_digest {
            bail!("worker runtime manifest digest mismatch");
        }
        validate_digest(&self.document_digest, "worker runtime document digest")?;
        if self.computed_document_digest()? != self.document_digest {
            bail!("worker runtime document digest mismatch");
        }
        Ok(())
    }
}

/// Strict inter-process registry lock for the CLI's Oracle worker upsert. This
/// intentionally exposes the existing private lock primitive without exposing
/// arbitrary state filenames.
pub fn lock_oracle_worker_registry(state_dir: &Path, oracle_name: &str) -> Result<File> {
    crate::scope::validate_session_identity(oracle_name)?;
    crate::scope::lock_private_state_file(
        state_dir,
        &format!(".oracle-{oracle_name}.worker-registry.lock"),
    )
}

pub fn worker_start_gate_path(state_dir: &Path, runtime_id: &str) -> Result<PathBuf> {
    crate::scope::ensure_private_state_dir(state_dir)?;
    validate_runtime_id(runtime_id)?;
    Ok(state_dir.join(format!(
        "{WORKER_START_GATE_PREFIX}{runtime_id}{WORKER_START_GATE_SUFFIX}"
    )))
}

pub fn candidate_payload_digest<T: Serialize>(payload: &T) -> Result<String> {
    Ok(blake3::hash(&serde_json::to_vec(payload)?)
        .to_hex()
        .to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorruptWorkerRuntimeEntry {
    pub filename: String,
    pub error: String,
}

/// Complete isolated inventory. Corrupt entries remain explicit so callers
/// can contain/reconcile healthy generations without treating corruption as
/// absence. Authority lookups call `require_clean` before deciding uniqueness.
#[derive(Debug, Clone, Default)]
pub struct WorkerRuntimeInventory {
    pub manifests: Vec<WorkerRuntimeManifest>,
    pub corrupt_entries: Vec<CorruptWorkerRuntimeEntry>,
    pub duplicate_sessions: Vec<String>,
}

impl WorkerRuntimeInventory {
    pub fn is_clean(&self) -> bool {
        self.corrupt_entries.is_empty() && self.duplicate_sessions.is_empty()
    }

    pub fn require_clean(&self) -> Result<()> {
        if !self.corrupt_entries.is_empty() {
            bail!(
                "worker runtime inventory contains {} corrupt entr{}: {}",
                self.corrupt_entries.len(),
                if self.corrupt_entries.len() == 1 {
                    "y"
                } else {
                    "ies"
                },
                self.corrupt_entries
                    .iter()
                    .map(|entry| format!("{} ({})", entry.filename, entry.error))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if !self.duplicate_sessions.is_empty() {
            bail!(
                "worker runtime inventory contains duplicate sessions: {}",
                self.duplicate_sessions.join(", ")
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerRuntimeReconcileState {
    PreparedNoEffect,
    PreparedEffectObserved,
    StartedRunning,
    StartedSessionMissing,
    ProcessGenerationMismatch,
    SessionCollision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerRuntimeReconcileEntry {
    pub runtime_id: String,
    pub session: String,
    pub state: WorkerRuntimeReconcileState,
    pub candidate_bound: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WorkerRuntimeReconcileReport {
    pub entries: Vec<WorkerRuntimeReconcileEntry>,
    pub corrupt_entries: Vec<CorruptWorkerRuntimeEntry>,
    pub duplicate_sessions: Vec<String>,
    pub unbound_observations: Vec<ObservedWorkerProcess>,
}

/// Reconcile every valid entry while carrying corrupt siblings explicitly.
/// This function is observational: it never invents an activation, deletes a
/// manifest or treats an unreadable entry as absent.
pub fn reconcile_worker_runtimes(
    state_dir: &Path,
    observed: &[ObservedWorkerProcess],
) -> Result<WorkerRuntimeReconcileReport> {
    let inventory = WorkerRuntimeManifest::list_strict(state_dir)?;
    let mut observed_by_session = BTreeMap::new();
    for process in observed {
        process.validate()?;
        if observed_by_session
            .insert(process.session.clone(), process.clone())
            .is_some()
        {
            bail!("duplicate observed process for session {}", process.session);
        }
    }
    let collisions = inventory
        .duplicate_sessions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let bound_sessions = inventory
        .manifests
        .iter()
        .map(|runtime| runtime.session.session.as_str())
        .collect::<BTreeSet<_>>();
    let mut entries = Vec::with_capacity(inventory.manifests.len());
    for runtime in &inventory.manifests {
        let observation = observed_by_session.get(&runtime.session.session);
        let state = if collisions.contains(runtime.session.session.as_str()) {
            WorkerRuntimeReconcileState::SessionCollision
        } else {
            match (&runtime.started, observation) {
                (None, None) => WorkerRuntimeReconcileState::PreparedNoEffect,
                (None, Some(_)) => WorkerRuntimeReconcileState::PreparedEffectObserved,
                (Some(_), None) => WorkerRuntimeReconcileState::StartedSessionMissing,
                (Some(started), Some(observed)) if started.observed == *observed => {
                    WorkerRuntimeReconcileState::StartedRunning
                }
                (Some(_), Some(_)) => WorkerRuntimeReconcileState::ProcessGenerationMismatch,
            }
        };
        entries.push(WorkerRuntimeReconcileEntry {
            runtime_id: runtime.runtime_id.clone(),
            session: runtime.session.session.clone(),
            state,
            candidate_bound: runtime.candidate.is_some(),
        });
    }
    let unbound_observations = observed
        .iter()
        .filter(|process| !bound_sessions.contains(process.session.as_str()))
        .cloned()
        .collect();
    Ok(WorkerRuntimeReconcileReport {
        entries,
        corrupt_entries: inventory.corrupt_entries,
        duplicate_sessions: inventory.duplicate_sessions,
        unbound_observations,
    })
}

/// Build a recommended session name that embeds a runtime generation prefix.
pub fn generation_scoped_session(base: &str, generation: &str) -> Result<String> {
    crate::scope::validate_session_identity(base)?;
    validate_runtime_id(generation)?;
    let safe_base = crate::session::sanitize_session_name(base);
    let suffix = format!("-{}", &generation[..20]);
    let keep = crate::session::MAX_SESSION_NAME_LEN
        .checked_sub(suffix.len())
        .context("worker generation suffix exceeds rmux session-name bound")?;
    let safe_base = safe_base.chars().take(keep).collect::<String>();
    let safe_base = safe_base.trim_end_matches(['-', '.']).to_string();
    if safe_base.is_empty() {
        bail!("worker session base becomes empty after generation scoping");
    }
    let session = format!("{safe_base}{suffix}");
    crate::scope::validate_session_identity(&session).with_context(|| {
        format!("generation-scoped worker session derived from `{base}` is invalid")
    })?;
    Ok(session)
}

fn runtime_generation(attempt: &WorkerAttemptIdentity) -> Result<String> {
    validate_attempt_identity(attempt)?;
    let bytes = serde_json::to_vec(&(
        "omega.worker-runtime-generation.v2",
        &attempt.mission_id,
        &attempt.task_id,
        &attempt.attempt_id,
    ))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_attempt_identity(attempt: &WorkerAttemptIdentity) -> Result<()> {
    validate_opaque_identity(attempt.mission_id.as_str(), "mission identity", 512)?;
    if attempt.plan_revision == 0 {
        bail!("worker plan revision must be positive");
    }
    validate_digest(&attempt.plan_digest, "worker plan digest")?;
    validate_opaque_identity(&attempt.task_id, "task identity", 512)?;
    validate_opaque_identity(&attempt.attempt_id, "attempt identity", 512)?;
    crate::scope::validate_session_identity(&attempt.owner)
        .context("invalid worker owner identity")?;
    Ok(())
}

fn validate_scope_receipt(receipt: &WorkerScopeReceipt) -> Result<()> {
    if receipt.schema_version != 1 {
        bail!("unsupported worker scope receipt schema");
    }
    validate_opaque_identity(receipt.mission_id.as_str(), "scope mission identity", 512)?;
    validate_opaque_identity(&receipt.task_id, "scope task identity", 512)?;
    validate_opaque_identity(&receipt.attempt_id, "scope attempt identity", 512)?;
    if receipt.plan_revision == 0 {
        bail!("worker scope plan revision must be positive");
    }
    crate::scope::validate_session_identity(&receipt.owner)?;
    receipt.claim.validate_authority()?;
    if receipt.claim.session != receipt.owner {
        bail!("worker scope claim session differs from its owner");
    }
    let normalized = crate::scope::validate_scope_selectors(receipt.claim.files_owned.clone())?;
    if normalized.is_empty() || normalized != receipt.claim.files_owned {
        bail!("worker scope receipt selectors are empty or non-canonical");
    }
    Ok(())
}

fn validate_scope_against_attempt(
    scope: &WorkerRuntimeScope,
    attempt: &WorkerAttemptIdentity,
    session: &WorkerSessionIdentity,
    workspace_id: &str,
) -> Result<()> {
    let receipt = scope.receipt();
    if receipt.mission_id != attempt.mission_id
        || receipt.task_id != attempt.task_id
        || receipt.attempt_id != attempt.attempt_id
        || receipt.plan_revision != attempt.plan_revision
        || receipt.owner != attempt.owner
        || receipt.claim.session != attempt.owner
        || receipt.claim.workspace_id.as_deref() != Some(workspace_id)
        || session.generation.is_empty()
    {
        bail!("worker scope authority differs from its runtime attempt/workspace binding");
    }
    Ok(())
}

fn validate_parent_link(
    link: &crate::mission_ledger::ChildMissionLink,
    attempt: &WorkerAttemptIdentity,
    session: &str,
    project: &str,
    workspace_id: &str,
    manifest_digest: &str,
) -> Result<()> {
    if link.schema_version != 1
        || link.child_mission_id != attempt.mission_id.as_str()
        || link.child_plan_revision != attempt.plan_revision
        || link.child_plan_digest != attempt.plan_digest
        || link.runtime_session != session
        || link.project != project
        || link.canonical_workspace_id != workspace_id
        || link.runtime_owner != attempt.owner
        || link.runtime_task_id != attempt.task_id
        || link.runtime_manifest_digest != manifest_digest
    {
        bail!("worker parent link differs from the exact child runtime identity");
    }
    validate_opaque_identity(&link.parent_mission_id, "parent mission identity", 512)?;
    if link.parent_mission_id == link.child_mission_id || link.parent_plan_revision == 0 {
        bail!("worker parent link has an invalid parent identity/revision");
    }
    validate_digest(&link.parent_plan_digest, "parent plan digest")?;
    validate_digest(&link.child_plan_digest, "child plan digest")?;
    validate_digest(
        &link.runtime_manifest_digest,
        "linked runtime manifest digest",
    )?;
    validate_opaque_identity(
        &link.child_binding_event_id,
        "child binding event identity",
        512,
    )?;
    if link.child_binding_event_sequence == 0 {
        bail!("worker parent link has no child binding event sequence");
    }
    validate_digest(
        &link.child_binding_command_digest,
        "child binding command digest",
    )?;
    validate_digest(
        &link.child_binding_projection_hash,
        "child binding projection hash",
    )?;
    crate::scope::validate_session_identity(&link.runtime_session)?;
    validate_opaque_identity(&link.linked_by, "parent link actor", 512)
}

fn scope_receipt_digest(receipt: &WorkerScopeReceipt) -> Result<String> {
    let bytes = serde_json::to_vec(&("omega.worker-scope-receipt.v1", receipt))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_digest(value: &str, field: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        bail!("invalid {field}; expected 64 lowercase hexadecimal characters");
    }
    Ok(())
}

fn validate_runtime_id(runtime_id: &str) -> Result<()> {
    validate_digest(runtime_id, "worker runtime identity")
}

fn validate_opaque_identity(value: &str, field: &str, max_len: usize) -> Result<()> {
    if value.is_empty()
        || value.len() > max_len
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        bail!("invalid {field}");
    }
    Ok(())
}

fn validate_git_object(value: &str, field: &str) -> Result<()> {
    if !(40..=64).contains(&value.len()) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{field} must be one full hexadecimal git object identity");
    }
    Ok(())
}

fn git_text(workspace: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()
        .with_context(|| format!("running git {} in {}", args.join(" "), workspace.display()))?;
    if !output.status.success() {
        bail!(
            "git {} failed in {}: {}",
            args.join(" "),
            workspace.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let value = String::from_utf8(output.stdout)
        .context("git returned non-UTF-8 worktree provenance")?
        .trim()
        .to_string();
    if value.is_empty() {
        bail!("git {} returned an empty value", args.join(" "));
    }
    Ok(value)
}

fn git_canonical_path(workspace: &Path, arg: &str) -> Result<PathBuf> {
    let raw = PathBuf::from(git_text(workspace, &["rev-parse", arg])?);
    let path = if raw.is_absolute() {
        raw
    } else {
        workspace.join(raw)
    };
    std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalizing git provenance path {}", path.display()))
}

fn validate_canonical_workspace(workspace: &Path, expected_id: &str) -> Result<()> {
    if !workspace.is_absolute()
        || workspace
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        bail!("worker runtime workspace is not an absolute normalized path");
    }
    let canonical = std::fs::canonicalize(workspace)
        .with_context(|| format!("canonicalizing worker workspace {}", workspace.display()))?;
    if canonical != workspace {
        bail!("worker runtime workspace is not canonical");
    }
    if crate::scope::canonical_workspace_identity(&canonical)? != expected_id {
        bail!("worker runtime workspace identity changed");
    }
    Ok(())
}

fn runtime_path(state_dir: &Path, runtime_id: &str) -> Result<PathBuf> {
    validate_runtime_id(runtime_id)?;
    Ok(state_dir.join(format!(
        "{WORKER_RUNTIME_PREFIX}{runtime_id}{WORKER_RUNTIME_SUFFIX}"
    )))
}

fn worker_runtime_history_dir(state_dir: &Path) -> Result<PathBuf> {
    crate::scope::ensure_private_state_dir(state_dir)?;
    let history = state_dir.join(WORKER_RUNTIME_HISTORY_DIR);
    crate::scope::ensure_private_state_dir(&history)?;
    Ok(history)
}

fn worker_runtime_archive_path(state_dir: &Path, runtime_id: &str) -> Result<PathBuf> {
    validate_runtime_id(runtime_id)?;
    Ok(worker_runtime_history_dir(state_dir)?.join(format!(
        "{WORKER_RUNTIME_PREFIX}{runtime_id}{WORKER_RUNTIME_HISTORY_SUFFIX}"
    )))
}

fn load_worker_runtime_archive_path(
    path: &Path,
    expected_runtime_id: Option<&str>,
) -> Result<Option<WorkerRuntimeArchive>> {
    let Some(bytes) = read_private_bounded(path)? else {
        return Ok(None);
    };
    let archive: WorkerRuntimeArchive = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing worker runtime archive {}", path.display()))?;
    archive.verify_integrity()?;
    if expected_runtime_id.is_some_and(|expected| expected != archive.runtime_id) {
        bail!(
            "worker runtime archive filename/document mismatch at {}",
            path.display()
        );
    }
    Ok(Some(archive))
}

fn list_worker_runtime_history(state_dir: &Path) -> Result<WorkerRuntimeHistoryInventory> {
    let history = worker_runtime_history_dir(state_dir)?;
    let mut archives = Vec::new();
    let mut corrupt_entries = Vec::new();
    for entry in std::fs::read_dir(&history)
        .with_context(|| format!("reading worker runtime history {}", history.display()))?
    {
        let entry = entry.context("reading worker runtime history entry")?;
        let Ok(filename) = entry.file_name().into_string() else {
            continue;
        };
        let Some(runtime_id) = filename
            .strip_prefix(WORKER_RUNTIME_PREFIX)
            .and_then(|name| name.strip_suffix(WORKER_RUNTIME_HISTORY_SUFFIX))
        else {
            continue;
        };
        if let Err(error) = validate_runtime_id(runtime_id) {
            corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: error.to_string(),
            });
            continue;
        }
        match load_worker_runtime_archive_path(&entry.path(), Some(runtime_id)) {
            Ok(Some(archive)) => archives.push(archive),
            Ok(None) => corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: "entry disappeared during strict read".to_string(),
            }),
            Err(error) => corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: format!("{error:#}"),
            }),
        }
    }
    archives.sort_by(|left, right| left.runtime_id.cmp(&right.runtime_id));
    corrupt_entries.sort_by(|left, right| left.filename.cmp(&right.filename));
    Ok(WorkerRuntimeHistoryInventory {
        archives,
        corrupt_entries,
    })
}

fn list_worker_runtimes_locked(state_dir: &Path) -> Result<WorkerRuntimeInventory> {
    crate::scope::ensure_private_state_dir(state_dir)?;
    let entries = std::fs::read_dir(state_dir)
        .with_context(|| format!("reading worker runtime state {}", state_dir.display()))?;
    let mut manifests = Vec::new();
    let mut corrupt_entries = Vec::new();
    for entry in entries {
        let entry = entry.context("reading worker runtime directory entry")?;
        let Ok(filename) = entry.file_name().into_string() else {
            continue;
        };
        let Some(runtime_id) = filename
            .strip_prefix(WORKER_RUNTIME_PREFIX)
            .and_then(|name| name.strip_suffix(WORKER_RUNTIME_SUFFIX))
        else {
            continue;
        };
        if let Err(error) = validate_runtime_id(runtime_id) {
            corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: error.to_string(),
            });
            continue;
        }
        match load_worker_runtime_path(&entry.path(), Some(runtime_id)) {
            Ok(Some(runtime)) => manifests.push(runtime),
            Ok(None) => corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: "entry disappeared during strict read".to_string(),
            }),
            Err(error) => corrupt_entries.push(CorruptWorkerRuntimeEntry {
                filename,
                error: format!("{error:#}"),
            }),
        }
    }
    manifests.sort_by(|left, right| left.runtime_id.cmp(&right.runtime_id));
    corrupt_entries.sort_by(|left, right| left.filename.cmp(&right.filename));
    let mut sessions: BTreeMap<&str, usize> = BTreeMap::new();
    for runtime in &manifests {
        *sessions
            .entry(runtime.session.session.as_str())
            .or_default() += 1;
    }
    let duplicate_sessions = sessions
        .into_iter()
        .filter_map(|(session, count)| (count > 1).then_some(session.to_string()))
        .collect();
    Ok(WorkerRuntimeInventory {
        manifests,
        corrupt_entries,
        duplicate_sessions,
    })
}

fn load_worker_runtime_path(
    path: &Path,
    expected_runtime_id: Option<&str>,
) -> Result<Option<WorkerRuntimeManifest>> {
    let Some(bytes) = read_private_bounded(path)? else {
        return Ok(None);
    };
    let source_digest = blake3::hash(&bytes).to_hex().to_string();
    let mut runtime: WorkerRuntimeManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing worker runtime manifest {}", path.display()))?;
    runtime.verify_integrity()?;
    if expected_runtime_id.is_some_and(|expected| expected != runtime.runtime_id) {
        bail!(
            "worker runtime filename/document mismatch at {}",
            path.display()
        );
    }
    runtime.source_generation = Some(runtime.storage_generation);
    runtime.source_digest = Some(source_digest);
    Ok(Some(runtime))
}

fn publish_new_locked(
    state_dir: &Path,
    manifest: &WorkerRuntimeManifest,
) -> Result<WorkerRuntimeManifest> {
    manifest.verify_integrity()?;
    let path = runtime_path(state_dir, &manifest.runtime_id)?;
    if read_private_bounded(&path)?.is_some() {
        bail!("worker runtime appeared before initial compare-and-swap");
    }
    write_worker_runtime(&path, manifest)?;
    let observed = load_worker_runtime_path(&path, Some(&manifest.runtime_id))?
        .context("worker runtime vanished after prepared publication")?;
    if observed.storage_generation != 1
        || observed.intent_digest != manifest.intent_digest
        || observed.manifest_digest != manifest.manifest_digest
        || observed.document_digest != manifest.document_digest
    {
        bail!("worker runtime changed while publishing prepared intent");
    }
    Ok(observed)
}

fn publish_transition_locked(
    state_dir: &Path,
    expected: &WorkerRuntimeManifest,
    mut next: WorkerRuntimeManifest,
) -> Result<WorkerRuntimeManifest> {
    let path = runtime_path(state_dir, &expected.runtime_id)?;
    let current = load_worker_runtime_path(&path, Some(&expected.runtime_id))?
        .context("worker runtime disappeared before compare-and-swap")?;
    let expected_generation = expected
        .source_generation
        .context("worker runtime transition requires a strictly loaded source generation")?;
    let expected_digest = expected
        .source_digest
        .as_deref()
        .context("worker runtime transition requires a strictly loaded source digest")?;
    let current_digest = current
        .source_digest
        .as_deref()
        .context("strict worker runtime load lost its source digest")?;
    if current.storage_generation != expected_generation || current_digest != expected_digest {
        bail!("stale worker runtime compare-and-swap refused");
    }
    if next.runtime_id != current.runtime_id
        || next.intent_digest != current.intent_digest
        || next.attempt != current.attempt
        || next.session != current.session
        || next.authority_workspace != current.authority_workspace
        || next.authority_workspace_id != current.authority_workspace_id
        || next.execution_workspace != current.execution_workspace
        || next.execution_workspace_id != current.execution_workspace_id
        || next.worktree != current.worktree
        || next.expected_command_digest != current.expected_command_digest
        || next.project != current.project
        || next.provider != current.provider
        || next.scope != current.scope
        || next.prepared_at != current.prepared_at
        || next.manifest_digest != current.manifest_digest
    {
        bail!("worker runtime transition attempted to mutate immutable intent");
    }
    next.storage_generation = current
        .storage_generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("worker runtime storage generation overflow"))?;
    next.source_generation = None;
    next.source_digest = None;
    next.seal()?;
    next.verify_integrity()?;
    write_worker_runtime(&path, &next)?;
    let observed = load_worker_runtime_path(&path, Some(&next.runtime_id))?
        .context("worker runtime vanished after transition")?;
    if observed.storage_generation != next.storage_generation
        || observed.manifest_digest != next.manifest_digest
        || observed.document_digest != next.document_digest
    {
        bail!("worker runtime changed while publishing transition");
    }
    Ok(observed)
}

fn write_worker_runtime(path: &Path, manifest: &WorkerRuntimeManifest) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(manifest)?;
    if bytes.len() as u64 > MAX_WORKER_RUNTIME_BYTES {
        bail!(
            "worker runtime manifest is {} bytes; maximum is {} bytes",
            bytes.len(),
            MAX_WORKER_RUNTIME_BYTES
        );
    }
    atomic_write_private(path, &bytes)
}

fn read_private_bounded(path: &Path) -> Result<Option<Vec<u8>>> {
    let before = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("inspecting {}", path.display())),
    };
    if !before.file_type().is_file() {
        bail!(
            "refusing non-regular worker runtime file {}",
            path.display()
        );
    }
    if before.len() > MAX_WORKER_RUNTIME_BYTES {
        bail!(
            "worker runtime file {} is {} bytes; maximum is {} bytes",
            path.display(),
            before.len(),
            MAX_WORKER_RUNTIME_BYTES
        );
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(no_follow_flag());
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "opening worker runtime {} without symlink following",
            path.display()
        )
    })?;
    let opened = file
        .metadata()
        .with_context(|| format!("inspecting opened worker runtime {}", path.display()))?;
    validate_file_identity(path, &before, &opened)?;
    if opened.len() > MAX_WORKER_RUNTIME_BYTES {
        bail!("worker runtime file exceeds the bounded read limit");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if opened.mode() & 0o777 != 0o600 {
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("setting owner-only mode on {}", path.display()))?;
        }
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take(MAX_WORKER_RUNTIME_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("reading worker runtime {}", path.display()))?;
    if bytes.len() as u64 > MAX_WORKER_RUNTIME_BYTES {
        bail!("worker runtime file exceeded the bounded read limit");
    }
    let after = std::fs::symlink_metadata(path)
        .with_context(|| format!("re-checking worker runtime {}", path.display()))?;
    let final_opened = file
        .metadata()
        .with_context(|| format!("re-checking opened worker runtime {}", path.display()))?;
    validate_file_identity(path, &after, &final_opened)?;
    if opened.len() != final_opened.len() || bytes.len() as u64 != final_opened.len() {
        bail!("worker runtime {} changed while being read", path.display());
    }
    Ok(Some(bytes))
}

fn validate_file_identity(path: &Path, path_metadata: &Metadata, opened: &Metadata) -> Result<()> {
    if !path_metadata.file_type().is_file() || !opened.file_type().is_file() {
        bail!("worker runtime {} is not a regular file", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened.dev() || path_metadata.ino() != opened.ino() {
            bail!(
                "worker runtime {} changed identity while opening",
                path.display()
            );
        }
        if opened.nlink() != 1 {
            bail!(
                "worker runtime {} has {} hard links; expected exactly one",
                path.display(),
                opened.nlink()
            );
        }
        let uid = effective_uid();
        if opened.uid() != uid {
            bail!(
                "worker runtime {} is owned by uid {}, current uid is {}",
                path.display(),
                opened.uid(),
                uid
            );
        }
    }
    Ok(())
}

fn atomic_write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    crate::scope::ensure_private_state_dir(parent)?;
    let directory = File::open(parent)
        .with_context(|| format!("opening worker runtime parent {}", parent.display()))?;
    validate_parent_identity(parent, &directory)?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("worker-runtime.json");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let (staged, mut file) = (0..128)
        .find_map(|_| {
            let serial = WORKER_RUNTIME_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let candidate = parent.join(format!(
                ".{filename}.omega-tmp-{}-{timestamp}-{serial}",
                std::process::id()
            ));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&candidate) {
                Ok(file) => Some(Ok((candidate, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error).with_context(|| {
                    format!("creating staged worker runtime {}", candidate.display())
                })),
            }
        })
        .transpose()?
        .context("could not allocate a unique staged worker runtime file")?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)
            .with_context(|| format!("writing staged worker runtime {}", staged.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.sync_all()
            .with_context(|| format!("syncing staged worker runtime {}", staged.display()))?;
        drop(file);
        std::fs::rename(&staged, path).with_context(|| {
            format!(
                "atomically replacing worker runtime {} with {}",
                path.display(),
                staged.display()
            )
        })?;
        validate_parent_identity(parent, &directory)?;
        directory
            .sync_all()
            .with_context(|| format!("syncing worker runtime parent {}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

fn validate_parent_identity(path: &Path, opened: &File) -> Result<()> {
    let path_metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting worker runtime parent {}", path.display()))?;
    let opened_metadata = opened
        .metadata()
        .with_context(|| format!("inspecting opened runtime parent {}", path.display()))?;
    if !path_metadata.file_type().is_dir()
        || path_metadata.file_type().is_symlink()
        || !opened_metadata.file_type().is_dir()
    {
        bail!(
            "worker runtime parent {} is not a real directory",
            path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
            || opened_metadata.uid() != effective_uid()
        {
            bail!(
                "worker runtime parent {} changed or is foreign-owned",
                path.display()
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn effective_uid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: geteuid takes no arguments and has no preconditions.
    unsafe { geteuid() }
}

#[cfg(all(unix, any(target_os = "linux", target_os = "android")))]
fn no_follow_flag() -> i32 {
    0o400000
}

#[cfg(all(
    unix,
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )
))]
fn no_follow_flag() -> i32 {
    0x100
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))
))]
fn no_follow_flag() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn digest(label: &str) -> String {
        blake3::hash(label.as_bytes()).to_hex().to_string()
    }

    fn mission_id(label: &str) -> crate::mission::MissionId {
        crate::mission::MissionId(format!("m-{}", &digest(label)[..32]))
    }

    fn intent(workspace: &Path, identity: &str, session: &str) -> WorkerRuntimeIntent {
        WorkerRuntimeIntent {
            attempt: WorkerAttemptIdentity {
                mission_id: mission_id(identity),
                plan_revision: 1,
                plan_digest: digest(&format!("plan-{identity}")),
                task_id: format!("task-{identity}"),
                attempt_id: format!("attempt-{identity}-1"),
                owner: format!("owner-{identity}"),
            },
            launch: WorkerLaunchIdentity {
                session: session.to_string(),
                expected_command_digest: digest("command-1"),
                authority_workspace: workspace.to_path_buf(),
                execution_workspace: workspace.to_path_buf(),
                worktree: None,
                project: "OmegaOS".to_string(),
                provider: "codex".to_string(),
            },
            scope: None,
        }
    }

    fn runtime_file(state_dir: &Path, runtime_id: &str) -> PathBuf {
        runtime_path(state_dir, runtime_id).unwrap()
    }

    fn observed_process(workspace: &Path, session: &str, generation: u64) -> ObservedWorkerProcess {
        ObservedWorkerProcess::new(
            session,
            PaneId::new(generation as u32),
            SessionId::new(generation as u32 + 100),
            WindowId::new(generation as u32 + 200),
            generation,
            generation as u32 + 1_000,
            digest(&format!("command-{generation}")),
            workspace,
        )
        .unwrap()
    }

    fn prepared_terminal_runtime(
        root: &Path,
        label: &str,
    ) -> (
        PathBuf,
        PathBuf,
        WorkerRuntimeManifest,
        ConfirmedWorkerAbsence,
    ) {
        let workspace = root.join(format!("workspace-{label}"));
        let state = root.join(format!("state-{label}"));
        std::fs::create_dir(&workspace).unwrap();
        std::fs::create_dir(&state).unwrap();
        let mission = crate::mission::Mission::new(
            "OmegaOS",
            format!("terminal runtime {label}"),
            workspace.clone(),
        );
        let task_id = format!("task-{label}");
        let plan = crate::mission::Plan {
            mission_id: mission.id.clone(),
            complexity: crate::routing::Complexity::Simple,
            strategy: crate::mission::PlanStrategy::Direct,
            tasks: vec![crate::mission::Task::new(
                &task_id,
                &task_id,
                "produce terminal evidence",
            )],
            created_at: Utc::now(),
        };
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(&state),
        )
        .unwrap();
        let execution = crate::orchestration::prepare_authoritative_execution(
            &ledger,
            &mission,
            &plan,
            "terminal-runtime-test",
            Vec::new(),
        )
        .unwrap();
        let authority = execution.attempt(&task_id).unwrap().clone();
        let mut runtime_intent = intent(&workspace, label, "placeholder-session");
        runtime_intent.attempt.mission_id = mission.id.clone();
        runtime_intent.attempt.plan_revision = execution.plan.revision;
        runtime_intent.attempt.plan_digest = execution.plan.content_digest.clone();
        runtime_intent.attempt.task_id = authority.task_id.clone();
        runtime_intent.attempt.attempt_id = authority.attempt_id.clone();
        let session = runtime_intent
            .generation_scoped_session(&format!("OmegaOS-worker-{label}"))
            .unwrap();
        runtime_intent.attempt.owner = session.clone();
        runtime_intent.launch.session = session;
        let runtime = WorkerRuntimeManifest::prepare(&state, runtime_intent).unwrap();

        crate::orchestration::transition_authoritative_attempt(
            &ledger,
            &authority,
            crate::mission::TaskAttemptState::Cancelled,
            "terminal-runtime-test",
        )
        .unwrap();
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &mission.id,
            crate::mission::MissionState::Cancelled,
            "terminal-runtime-test",
        )
        .unwrap();
        let absence = ConfirmedWorkerAbsence::new(&runtime).unwrap();
        (workspace, state, runtime, absence)
    }

    #[test]
    fn traversal_and_noncanonical_names_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        assert!(WorkerRuntimeManifest::load_strict(&state, "../escape").is_err());

        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let invalid = intent(&workspace, "unsafe", "../escape");
        assert!(WorkerRuntimeManifest::prepare(&state, invalid).is_err());
        assert!(!root.path().join("escape").exists());

        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("worker-runtime-not-hex.json"), b"{}").unwrap();
        let inventory = WorkerRuntimeManifest::list_strict(&state).unwrap();
        assert!(inventory.manifests.is_empty());
        assert_eq!(inventory.corrupt_entries.len(), 1);
        assert!(!inventory.is_clean());
    }

    #[test]
    fn generation_helper_binds_attempt_and_produces_a_scoped_session() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let mut prepared = intent(&workspace, "generation", "placeholder");
        let generation = prepared.runtime_generation().unwrap();
        let session = prepared.generation_scoped_session("omega-worker").unwrap();
        assert!(session.ends_with(&generation[..20]));
        prepared.launch.session = session.clone();
        let manifest =
            WorkerRuntimeManifest::prepare(&root.path().join("state"), prepared).unwrap();
        assert_eq!(manifest.session().session, session);
        assert_eq!(manifest.session().generation, generation);
    }

    #[test]
    fn duplicate_session_is_rejected_globally() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        WorkerRuntimeManifest::prepare(&state, intent(&workspace, "one", "shared-session"))
            .unwrap();
        let error =
            WorkerRuntimeManifest::prepare(&state, intent(&workspace, "two", "shared-session"))
                .unwrap_err();
        assert!(error.to_string().contains("already bound"));
        let inventory = WorkerRuntimeManifest::list_strict(&state).unwrap();
        assert_eq!(inventory.manifests.len(), 1);
        assert!(inventory.is_clean());
    }

    #[test]
    fn concurrent_duplicate_session_preparation_has_one_winner() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let first_barrier = barrier.clone();
        let first_state = state.clone();
        let first_workspace = workspace.clone();
        let first = std::thread::spawn(move || {
            first_barrier.wait();
            WorkerRuntimeManifest::prepare(
                &first_state,
                intent(&first_workspace, "concurrent-one", "concurrent-session"),
            )
        });
        let second_barrier = barrier.clone();
        let second_state = state.clone();
        let second_workspace = workspace.clone();
        let second = std::thread::spawn(move || {
            second_barrier.wait();
            WorkerRuntimeManifest::prepare(
                &second_state,
                intent(&second_workspace, "concurrent-two", "concurrent-session"),
            )
        });
        barrier.wait();
        let first = first.join().unwrap();
        let second = second.join().unwrap();
        assert_ne!(first.is_ok(), second.is_ok());
        assert_eq!(
            WorkerRuntimeManifest::list_strict(&state)
                .unwrap()
                .manifests
                .len(),
            1
        );
    }

    #[test]
    fn exact_prepare_replay_is_idempotent() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = intent(&workspace, "idempotent-prepare", "prepare-session");
        let first = WorkerRuntimeManifest::prepare(&state, prepared.clone()).unwrap();
        let replay = WorkerRuntimeManifest::prepare(&state, prepared).unwrap();
        assert_eq!(first.runtime_id(), replay.runtime_id());
        assert_eq!(first.prepared_at(), replay.prepared_at());
        assert_eq!(first.storage_generation(), 1);
        assert_eq!(replay.storage_generation(), 1);
    }

    #[test]
    fn prepared_and_started_reconcile_crash_boundaries_exactly() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "lifecycle", "lifecycle-session"),
        )
        .unwrap();
        let absent = reconcile_worker_runtimes(&state, &[]).unwrap();
        assert_eq!(
            absent.entries[0].state,
            WorkerRuntimeReconcileState::PreparedNoEffect
        );

        let observed = observed_process(&workspace, "lifecycle-session", 1);
        let crash_gap = reconcile_worker_runtimes(&state, std::slice::from_ref(&observed)).unwrap();
        assert_eq!(
            crash_gap.entries[0].state,
            WorkerRuntimeReconcileState::PreparedEffectObserved
        );

        let started = prepared.activate_started(&state, observed.clone()).unwrap();
        assert_eq!(started.storage_generation(), 2);
        let running = reconcile_worker_runtimes(&state, std::slice::from_ref(&observed)).unwrap();
        assert_eq!(
            running.entries[0].state,
            WorkerRuntimeReconcileState::StartedRunning
        );
        let wrong = observed_process(&workspace, "lifecycle-session", 2);
        assert_eq!(
            reconcile_worker_runtimes(&state, &[wrong]).unwrap().entries[0].state,
            WorkerRuntimeReconcileState::ProcessGenerationMismatch
        );
        assert_eq!(
            reconcile_worker_runtimes(&state, &[]).unwrap().entries[0].state,
            WorkerRuntimeReconcileState::StartedSessionMissing
        );
    }

    #[test]
    fn stale_activation_cas_cannot_replace_the_first_process_generation() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared =
            WorkerRuntimeManifest::prepare(&state, intent(&workspace, "stale", "stale-session"))
                .unwrap();
        let stale = prepared.clone();
        let first = observed_process(&workspace, "stale-session", 1);
        prepared.activate_started(&state, first.clone()).unwrap();
        let mut conflicting = observed_process(&workspace, "stale-session", 2);
        conflicting.command_digest = digest("command-1");
        conflicting.observation_digest = conflicting.computed_digest().unwrap();
        stale.activate_started(&state, conflicting).unwrap_err();
        let current = WorkerRuntimeManifest::load_strict(&state, stale.runtime_id())
            .unwrap()
            .unwrap();
        assert_eq!(current.started().unwrap().observed, first);
        assert_eq!(current.storage_generation(), 2);
    }

    #[test]
    fn activation_requires_the_exact_prepared_command_without_mutating_on_mismatch() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "command-binding", "command-binding-session"),
        )
        .unwrap();

        let wrong = observed_process(&workspace, "command-binding-session", 2);
        prepared.activate_started(&state, wrong).unwrap_err();
        let unchanged = WorkerRuntimeManifest::load_strict(&state, prepared.runtime_id())
            .unwrap()
            .unwrap();
        assert!(unchanged.started().is_none());
        assert_eq!(unchanged.storage_generation(), 1);
        assert_eq!(unchanged.document_digest(), prepared.document_digest());

        let exact = observed_process(&workspace, "command-binding-session", 1);
        let started = prepared.activate_started(&state, exact.clone()).unwrap();
        assert_eq!(started.started().unwrap().observed, exact);
        assert_eq!(
            prepared
                .activate_started(
                    &state,
                    observed_process(&workspace, "command-binding-session", 1),
                )
                .unwrap()
                .storage_generation(),
            2
        );
    }

    #[test]
    fn prepared_crash_gap_never_adopts_a_same_name_wrong_command() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "crash-gap-command", "crash-gap-session"),
        )
        .unwrap();
        let wrong = observed_process(&workspace, "crash-gap-session", 7);

        let report = reconcile_worker_runtimes(&state, std::slice::from_ref(&wrong)).unwrap();
        assert_eq!(
            report.entries[0].state,
            WorkerRuntimeReconcileState::PreparedEffectObserved
        );
        prepared.activate_started(&state, wrong).unwrap_err();
        let current = WorkerRuntimeManifest::load_strict(&state, prepared.runtime_id())
            .unwrap()
            .unwrap();
        assert!(current.started().is_none());
        assert_eq!(current.storage_generation(), 1);
    }

    #[test]
    fn private_start_gate_is_generation_exact_and_idempotent() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "start-gate", "start-gate-session"),
        )
        .unwrap();
        let gate = prepared.start_gate_path(&state).unwrap();
        assert_eq!(
            gate,
            worker_start_gate_path(&state, prepared.runtime_id()).unwrap()
        );
        assert_eq!(prepared.start_gate_value(), prepared.runtime_id());
        assert!(!prepared.is_start_gate_released(&state).unwrap());

        prepared.release_start_gate(&state).unwrap();
        prepared.release_start_gate(&state).unwrap();
        assert!(prepared.is_start_gate_released(&state).unwrap());
        assert_eq!(
            std::fs::read(&gate).unwrap(),
            prepared.runtime_id().as_bytes()
        );

        std::fs::write(&gate, digest("different-generation")).unwrap();
        assert!(prepared.is_start_gate_released(&state).is_err());
        assert!(prepared.release_start_gate(&state).is_err());
    }

    #[test]
    fn stale_source_generation_is_rejected_by_storage_cas() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "direct-cas", "direct-cas-session"),
        )
        .unwrap();
        let stale = prepared.clone();

        let mut first = prepared.clone();
        first.started = Some(WorkerRuntimeStarted {
            observed: observed_process(&workspace, "direct-cas-session", 1),
            activated_at: Utc::now(),
        });
        {
            let _lock = crate::scope::lock_private_state_file(&state, WORKER_RUNTIME_LOCK).unwrap();
            publish_transition_locked(&state, &prepared, first).unwrap();
        }

        let mut second = stale.clone();
        second.started = Some(WorkerRuntimeStarted {
            observed: observed_process(&workspace, "direct-cas-session", 2),
            activated_at: Utc::now(),
        });
        let error = {
            let _lock = crate::scope::lock_private_state_file(&state, WORKER_RUNTIME_LOCK).unwrap();
            publish_transition_locked(&state, &stale, second).unwrap_err()
        };
        assert!(error
            .to_string()
            .contains("stale worker runtime compare-and-swap refused"));
        let current = WorkerRuntimeManifest::load_strict(&state, stale.runtime_id())
            .unwrap()
            .unwrap();
        assert_eq!(current.started().unwrap().observed.process_generation, 1);
    }

    #[test]
    fn candidate_binding_is_started_only_idempotent_and_immutable() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "candidate", "candidate-session"),
        )
        .unwrap();
        let candidate =
            WorkerCandidateIdentity::new("candidate-event-1", digest("candidate-a")).unwrap();
        assert!(prepared.bind_candidate(&state, candidate.clone()).is_err());
        let started = prepared
            .activate_started(&state, observed_process(&workspace, "candidate-session", 1))
            .unwrap();
        let bound = started.bind_candidate(&state, candidate.clone()).unwrap();
        assert_eq!(bound.storage_generation(), 3);
        let replay = started.bind_candidate(&state, candidate).unwrap();
        assert_eq!(replay.storage_generation(), 3);
        assert_eq!(replay.candidate(), bound.candidate());

        let mismatch =
            WorkerCandidateIdentity::new("candidate-event-2", digest("candidate-b")).unwrap();
        let error = started.bind_candidate(&state, mismatch).unwrap_err();
        assert!(error.to_string().contains("different immutable candidate"));
        let current = WorkerRuntimeManifest::load_strict(&state, started.runtime_id())
            .unwrap()
            .unwrap();
        assert_eq!(current.candidate(), bound.candidate());
    }

    #[test]
    fn corrupt_entry_is_isolated_but_blocks_authority_resolution() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let healthy = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "healthy", "healthy-session"),
        )
        .unwrap();
        let corrupt = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "corrupt", "corrupt-session"),
        )
        .unwrap();
        std::fs::write(runtime_file(&state, corrupt.runtime_id()), b"{partial").unwrap();

        let inventory = WorkerRuntimeManifest::list_strict(&state).unwrap();
        assert_eq!(inventory.manifests.len(), 1);
        assert_eq!(inventory.manifests[0].runtime_id(), healthy.runtime_id());
        assert_eq!(inventory.corrupt_entries.len(), 1);
        assert!(WorkerRuntimeManifest::resolve_session_strict(&state, "healthy-session").is_err());
        let report = reconcile_worker_runtimes(&state, &[]).unwrap();
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.corrupt_entries.len(), 1);
        assert_eq!(
            report.entries[0].state,
            WorkerRuntimeReconcileState::PreparedNoEffect
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_hardlink_and_symlinked_state_are_rejected() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();

        let symlink_state = root.path().join("symlink-state");
        std::fs::create_dir(&symlink_state).unwrap();
        let symlink_intent = intent(&workspace, "symlink", "symlink-session");
        let runtime_id = symlink_intent.runtime_generation().unwrap();
        let sentinel = root.path().join("sentinel");
        std::fs::write(&sentinel, b"sentinel").unwrap();
        symlink(&sentinel, runtime_file(&symlink_state, &runtime_id)).unwrap();
        assert!(WorkerRuntimeManifest::load_strict(&symlink_state, &runtime_id).is_err());
        assert!(WorkerRuntimeManifest::prepare(&symlink_state, symlink_intent).is_err());
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"sentinel");

        let source_state = root.path().join("source-state");
        let source = WorkerRuntimeManifest::prepare(
            &source_state,
            intent(&workspace, "hardlink", "hardlink-session"),
        )
        .unwrap();
        let hard_state = root.path().join("hard-state");
        std::fs::create_dir(&hard_state).unwrap();
        let target = runtime_file(&source_state, source.runtime_id());
        std::fs::hard_link(&target, runtime_file(&hard_state, source.runtime_id())).unwrap();
        assert!(WorkerRuntimeManifest::load_strict(&hard_state, source.runtime_id()).is_err());

        let real_state = root.path().join("real-state");
        std::fs::create_dir(&real_state).unwrap();
        let aliased_state = root.path().join("aliased-state");
        symlink(&real_state, &aliased_state).unwrap();
        assert!(WorkerRuntimeManifest::list_strict(&aliased_state).is_err());
    }

    #[test]
    fn bounded_read_and_interrupted_staging_fail_safely() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        std::fs::create_dir(&state).unwrap();
        let oversized_id = digest("oversized-runtime");
        let oversized = File::create(runtime_file(&state, &oversized_id)).unwrap();
        oversized.set_len(MAX_WORKER_RUNTIME_BYTES + 1).unwrap();
        let error = WorkerRuntimeManifest::load_strict(&state, &oversized_id).unwrap_err();
        assert!(error.to_string().contains("maximum is"));
        std::fs::remove_file(runtime_file(&state, &oversized_id)).unwrap();

        let staged = state.join(".worker-runtime-crashed.omega-tmp-partial");
        std::fs::write(&staged, b"{partial").unwrap();
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "after-crash", "after-crash-session"),
        )
        .unwrap();
        assert!(staged.exists());
        assert!(
            WorkerRuntimeManifest::load_strict(&state, prepared.runtime_id())
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn scope_receipt_binds_exact_normalized_resources_and_fences() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let mut runtime_intent = intent(&workspace, "scope", "scope-session");
        let attempt = runtime_intent.attempt.clone();
        let claim = crate::scope::prepare_claim_for_workspace(
            &workspace,
            &attempt.owner,
            vec!["src//a.rs".to_string(), "src/b.rs".to_string()],
        )
        .unwrap();
        assert_eq!(claim.files_owned, vec!["src/a.rs", "src/b.rs"]);
        let receipt = WorkerScopeReceipt {
            schema_version: 1,
            mission_id: attempt.mission_id.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            plan_revision: attempt.plan_revision,
            owner: attempt.owner.clone(),
            claim: claim.clone(),
        };
        let workspace_id = claim.workspace_id.as_deref().unwrap();
        let leases = claim
            .files_owned
            .iter()
            .enumerate()
            .map(|(index, selector)| crate::mission_ledger::LeaseRecord {
                resource_key: crate::scope::lease_resource_key(workspace_id, selector),
                mission_id: attempt.mission_id.clone(),
                task_id: attempt.task_id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                owner: attempt.owner.clone(),
                fencing_token: index as u64 + 1,
                expires_at: Utc::now() + Duration::hours(1),
                status: crate::mission_ledger::LeaseStatus::Active,
            })
            .collect::<Vec<_>>();
        let scope = WorkerRuntimeScope::from_authority(receipt.clone(), &leases).unwrap();
        assert_eq!(scope.resources().len(), 2);
        assert_eq!(scope.resources()[0].selector, "src/a.rs");
        assert!(WorkerRuntimeScope::from_authority(receipt.clone(), &leases[..1]).is_err());
        let mut wrong = leases.clone();
        wrong[0].resource_key = "scope:wrong".to_string();
        assert!(WorkerRuntimeScope::from_authority(receipt, &wrong).is_err());

        runtime_intent.scope = Some(scope);
        let manifest = WorkerRuntimeManifest::prepare(&state, runtime_intent).unwrap();
        assert_eq!(manifest.scope().unwrap().resources().len(), 2);
    }

    #[test]
    fn exact_parent_link_is_bound_to_child_plan_and_runtime_session() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let child = intent(&workspace, "child", "child-session");
        let prepared = WorkerRuntimeManifest::prepare(&state, child.clone()).unwrap();
        let canonical_workspace = std::fs::canonicalize(&workspace).unwrap();
        let workspace_id =
            crate::scope::canonical_workspace_identity(&canonical_workspace).unwrap();
        let link = crate::mission_ledger::ChildMissionLink {
            schema_version: 1,
            parent_mission_id: mission_id("parent").as_str().to_string(),
            parent_plan_revision: 1,
            parent_plan_digest: digest("parent-plan"),
            child_mission_id: child.attempt.mission_id.as_str().to_string(),
            child_plan_revision: child.attempt.plan_revision,
            child_plan_digest: child.attempt.plan_digest.clone(),
            runtime_session: child.launch.session.clone(),
            runtime_manifest_digest: prepared.manifest_digest().to_string(),
            project: child.launch.project.clone(),
            canonical_workspace_id: workspace_id,
            runtime_owner: child.attempt.owner.clone(),
            runtime_task_id: child.attempt.task_id.clone(),
            child_binding_event_id: "event-child-binding".to_string(),
            child_binding_event_sequence: 1,
            child_binding_command_digest: digest("child-binding-command"),
            child_binding_projection_hash: digest("child-binding-projection"),
            linked_by: "omega-child-mission-linker".to_string(),
        };
        let manifest = prepared.bind_parent_link(&state, link.clone()).unwrap();
        assert_eq!(
            manifest.parent_link().unwrap().runtime_session,
            "child-session"
        );
        assert_eq!(manifest.storage_generation(), 2);
        let replay = prepared.bind_parent_link(&state, link.clone()).unwrap();
        assert_eq!(replay.storage_generation(), 2);
        assert_eq!(replay.parent_link(), Some(&link));

        let mut wrong = link;
        wrong.runtime_session = "other-session".to_string();
        assert!(prepared.bind_parent_link(&state, wrong).is_err());
    }

    #[test]
    fn child_runtime_separates_parent_authority_from_registered_worktree_execution() {
        fn git(dir: &Path, args: &[&str]) {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let root = tempfile::tempdir().unwrap();
        let authority = root.path().join("authority");
        let execution = root.path().join("execution");
        let state = root.path().join("state");
        std::fs::create_dir(&authority).unwrap();
        git(&authority, &["init"]);
        git(
            &authority,
            &["config", "user.email", "omega-test@example.invalid"],
        );
        git(&authority, &["config", "user.name", "Omega Test"]);
        std::fs::write(authority.join("README.md"), "fixture\n").unwrap();
        git(&authority, &["add", "README.md"]);
        git(&authority, &["commit", "-m", "fixture"]);
        let execution_text = execution.to_string_lossy().to_string();
        git(
            &authority,
            &[
                "worktree",
                "add",
                "-b",
                "omega/runtime-fixture",
                &execution_text,
                "HEAD",
            ],
        );

        let provenance = WorkerWorktreeProvenance::capture(&authority, &execution).unwrap();
        let mut child = intent(&authority, "worktree-child", "worktree-child-session");
        child.launch.execution_workspace = execution.clone();
        child.launch.worktree = Some(provenance.clone());
        let prepared = WorkerRuntimeManifest::prepare(&state, child.clone()).unwrap();
        assert_ne!(prepared.authority_workspace_id(), prepared.workspace_id());
        assert_eq!(prepared.worktree_provenance(), Some(&provenance));

        let authority_path = std::fs::canonicalize(&authority).unwrap();
        let authority_id = crate::scope::canonical_workspace_identity(&authority_path).unwrap();
        let link = crate::mission_ledger::ChildMissionLink {
            schema_version: 1,
            parent_mission_id: mission_id("worktree-parent").as_str().to_string(),
            parent_plan_revision: 3,
            parent_plan_digest: digest("worktree-parent-plan"),
            child_mission_id: child.attempt.mission_id.as_str().to_string(),
            child_plan_revision: child.attempt.plan_revision,
            child_plan_digest: child.attempt.plan_digest.clone(),
            runtime_session: child.launch.session.clone(),
            runtime_manifest_digest: prepared.manifest_digest().to_string(),
            project: child.launch.project.clone(),
            canonical_workspace_id: authority_id,
            runtime_owner: child.attempt.owner.clone(),
            runtime_task_id: child.attempt.task_id.clone(),
            child_binding_event_id: "event-worktree-child-binding".to_string(),
            child_binding_event_sequence: 4,
            child_binding_command_digest: digest("worktree-child-command"),
            child_binding_projection_hash: digest("worktree-child-projection"),
            linked_by: "omega-worktree-child-linker".to_string(),
        };
        assert!(prepared.bind_parent_link(&state, link.clone()).is_ok());

        let other_root = tempfile::tempdir().unwrap();
        let unrelated = other_root.path().join("unrelated");
        std::fs::create_dir(&unrelated).unwrap();
        let mut invalid = intent(&authority, "unrelated-child", "unrelated-child-session");
        invalid.launch.execution_workspace = unrelated;
        assert!(WorkerRuntimeManifest::prepare(&state, invalid).is_err());
    }

    #[test]
    fn legacy_runtime_schema_fails_closed_before_activation() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "legacy-schema", "legacy-schema-session"),
        )
        .unwrap();
        let path = runtime_file(&state, prepared.runtime_id());
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["schema_version"] = serde_json::json!(1);
        std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        let error = WorkerRuntimeManifest::load_strict(&state, prepared.runtime_id()).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported worker runtime schema"));
        assert!(prepared
            .activate_started(
                &state,
                observed_process(&workspace, "legacy-schema-session", 1),
            )
            .is_err());
    }

    #[test]
    fn terminal_retirement_keeps_removed_workspace_out_of_active_inventory() {
        let root = tempfile::tempdir().unwrap();
        let (workspace, state, runtime, absence) =
            prepared_terminal_runtime(root.path(), "retirement");
        let archive = runtime.retire_terminal(&state, &absence).unwrap();
        assert_eq!(archive.runtime_id, runtime.runtime_id());
        assert!(
            WorkerRuntimeManifest::load_strict(&state, runtime.runtime_id())
                .unwrap()
                .is_none()
        );
        assert!(
            WorkerRuntimeManifest::load_history_strict(&state, runtime.runtime_id())
                .unwrap()
                .is_some()
        );

        std::fs::remove_dir_all(&workspace).unwrap();
        let active = WorkerRuntimeManifest::list_strict(&state).unwrap();
        assert!(active.is_clean());
        assert!(active.manifests.is_empty());
        let history = WorkerRuntimeManifest::list_history_strict(&state).unwrap();
        assert!(history.is_clean());
        assert_eq!(history.archives.len(), 1);
    }

    #[test]
    fn missing_workspace_fails_closed_until_terminal_runtime_is_retired() {
        let root = tempfile::tempdir().unwrap();
        let (workspace, state, runtime, _absence) =
            prepared_terminal_runtime(root.path(), "missing-unretired");
        std::fs::remove_dir_all(&workspace).unwrap();
        let inventory = WorkerRuntimeManifest::list_strict(&state).unwrap();
        assert!(inventory.manifests.is_empty());
        assert_eq!(inventory.corrupt_entries.len(), 1);
        assert!(
            WorkerRuntimeManifest::resolve_session_strict(&state, &runtime.session().session,)
                .is_err()
        );
    }

    #[test]
    fn terminal_history_tampering_is_detected_without_live_workspace() {
        let root = tempfile::tempdir().unwrap();
        let (workspace, state, runtime, absence) =
            prepared_terminal_runtime(root.path(), "history-tamper");
        runtime.retire_terminal(&state, &absence).unwrap();
        std::fs::remove_dir_all(&workspace).unwrap();
        let path = worker_runtime_archive_path(&state, runtime.runtime_id()).unwrap();
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["terminal"]["attempt_state"] = serde_json::json!("failed");
        std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        assert!(WorkerRuntimeManifest::load_history_strict(&state, runtime.runtime_id()).is_err());
        let history = WorkerRuntimeManifest::list_history_strict(&state).unwrap();
        assert_eq!(history.corrupt_entries.len(), 1);
    }

    #[test]
    fn manifest_digest_detects_same_schema_content_tampering() {
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let prepared =
            WorkerRuntimeManifest::prepare(&state, intent(&workspace, "tamper", "tamper-session"))
                .unwrap();
        let path = runtime_file(&state, prepared.runtime_id());
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["project"] = serde_json::Value::String("OtherProject".to_string());
        std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
        let error = WorkerRuntimeManifest::load_strict(&state, prepared.runtime_id()).unwrap_err();
        assert!(format!("{error:#}").contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn state_parent_and_manifest_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let workspace = root.path().join("workspace");
        let state = root.path().join("state");
        std::fs::create_dir(&workspace).unwrap();
        let manifest = WorkerRuntimeManifest::prepare(
            &state,
            intent(&workspace, "permissions", "permissions-session"),
        )
        .unwrap();
        assert_eq!(
            std::fs::metadata(&state).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(runtime_file(&state, manifest.runtime_id()))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
