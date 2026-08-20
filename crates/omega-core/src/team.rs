use crate::session::SessionManager;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rmux_sdk::{Pane, PaneId, PaneProcessState, SessionId, SplitDirection, WindowId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

const TEAM_RUNTIME_SCHEMA_VERSION: u32 = 1;
const TEAM_RUNTIME_STARTED_SCHEMA_VERSION: u32 = 2;
const TEAM_RUNTIME_PREFIX: &str = "team-runtime-";
const TEAM_RUNTIME_SUFFIX: &str = ".json";
const TEAM_RUNTIME_LOCK: &str = ".team-runtime.lock";
const TEAM_START_BARRIER_PREFIX: &str = "team-start-barrier-";
const TEAM_LEDGER_CAS_RETRIES: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub prompt: String,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub project: String,
    pub session_name: String,
    pub working_dir: String,
    pub agent_command: String,
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Clone)]
pub struct PreparedTeamAuthority {
    pub mission: crate::mission::Mission,
    pub legacy_plan: crate::mission::Plan,
    pub authority: crate::orchestration::AuthoritativeExecution,
}

/// Immutable write-ahead identity for one member pane inside an aggregate
/// rmux team session. `owner` is the identity used by scope leases, the
/// Running transition and `omega done`; it is deliberately distinct from the
/// aggregate rmux session that owns the pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRuntimeMember {
    pub member_name: String,
    pub pane_index: u32,
    pub owner: String,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub files_owned: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_claim_id: Option<String>,
}

/// Crash-safe bridge between one real rmux session and the V3 identities of
/// every pane it contains. The digest detects partial/corrupt edits; ledger
/// replay and the active plan remain the actual authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRuntimeManifest {
    pub schema_version: u32,
    pub aggregate_session: String,
    pub mission_id: crate::mission::MissionId,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub working_dir: PathBuf,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub members: Vec<TeamRuntimeMember>,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRuntimeStartedAck {
    pub schema_version: u32,
    pub aggregate_session: String,
    pub mission_id: crate::mission::MissionId,
    pub plan_revision: u64,
    pub plan_digest: String,
    pub manifest_digest: String,
    pub panes: Vec<TeamPaneActivation>,
    pub activated_at: DateTime<Utc>,
    pub acknowledgement_digest: String,
}

/// Stable daemon-observed identity of one member process. Pane indexes are
/// deliberately absent: indexes are mutable layout positions, while PaneId,
/// SessionId and the rmux process generation survive splits and reordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamPaneActivation {
    pub owner: String,
    pub pane_id: PaneId,
    pub session_id: SessionId,
    pub window_id: WindowId,
    pub process_generation: u64,
    pub process_pid: u32,
    pub command_digest: String,
    pub working_dir: PathBuf,
    pub activation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMemberCandidateBinding {
    pub aggregate_session: String,
    pub manifest_digest: String,
    pub working_dir: PathBuf,
    pub mission_id: crate::mission::MissionId,
    pub plan_revision: u64,
    pub task_id: String,
    pub attempt_id: String,
    pub owner: String,
    pub pane_index: u32,
    pub pane_activation: TeamPaneActivation,
    pub files_owned: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TeamMemberCandidate {
    pub binding: TeamMemberCandidateBinding,
    /// Exact payload recorded in the immutable CandidateDone event, with that
    /// event's replay provenance attached. Recovery must write this value, not
    /// reconstruct a fresh timestamp/summary.
    pub signal: crate::done::DoneSignal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMemberRuntimeStatus {
    pub owner: String,
    pub task_id: String,
    pub attempt_id: String,
    pub state: crate::mission::TaskAttemptState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRuntimeStatus {
    pub aggregate_session: String,
    pub mission_id: crate::mission::MissionId,
    pub mission_state: crate::mission::MissionState,
    pub started: bool,
    pub started_ack: Option<TeamRuntimeStartedAck>,
    pub start_barrier_released: bool,
    pub members: Vec<TeamMemberRuntimeStatus>,
    pub all_terminal: bool,
    pub all_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorruptTeamRuntimeManifest {
    pub path: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct TeamRuntimeManifestScan {
    pub manifests: Vec<TeamRuntimeManifest>,
    pub corrupt: Vec<CorruptTeamRuntimeManifest>,
}

impl TeamPaneActivation {
    fn computed_digest(&self) -> Result<String> {
        let mut unsigned = self.clone();
        unsigned.activation_digest.clear();
        Ok(blake3::hash(&serde_json::to_vec(&unsigned)?)
            .to_hex()
            .to_string())
    }

    fn seal(mut self) -> Result<Self> {
        self.activation_digest = self.computed_digest()?;
        self.verify_integrity()?;
        Ok(self)
    }

    fn verify_integrity(&self) -> Result<()> {
        crate::scope::validate_session_identity(&self.owner)?;
        if self.process_generation == 0 || self.process_pid == 0 {
            anyhow::bail!("team pane activation has no observed live process generation");
        }
        for (label, digest) in [
            ("command", self.command_digest.as_str()),
            ("activation", self.activation_digest.as_str()),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                anyhow::bail!("team pane {label} digest is not a lowercase BLAKE3 digest");
            }
        }
        if self.computed_digest()? != self.activation_digest {
            anyhow::bail!("team pane activation digest mismatch");
        }
        let canonical = std::fs::canonicalize(&self.working_dir).with_context(|| {
            format!(
                "canonicalizing activated team pane workspace {}",
                self.working_dir.display()
            )
        })?;
        if canonical != self.working_dir {
            anyhow::bail!("team pane activation workspace is not canonical");
        }
        Ok(())
    }
}

impl TeamRuntimeStartedAck {
    fn computed_digest(&self) -> Result<String> {
        let mut unsigned = self.clone();
        unsigned.acknowledgement_digest.clear();
        Ok(blake3::hash(&serde_json::to_vec(&unsigned)?)
            .to_hex()
            .to_string())
    }

    fn verify_for_manifest(&self, manifest: &TeamRuntimeManifest) -> Result<()> {
        if self.schema_version != TEAM_RUNTIME_STARTED_SCHEMA_VERSION
            || self.aggregate_session != manifest.aggregate_session
            || self.mission_id != manifest.mission_id
            || self.plan_revision != manifest.plan_revision
            || self.plan_digest != manifest.plan_digest
            || self.manifest_digest != manifest.manifest_digest
            || self.activated_at < manifest.created_at
        {
            anyhow::bail!("team runtime started acknowledgement identity mismatch");
        }
        if self.acknowledgement_digest.len() != 64
            || !self
                .acknowledgement_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || self.computed_digest()? != self.acknowledgement_digest
        {
            anyhow::bail!("team runtime started acknowledgement digest mismatch");
        }
        if self.panes.len() != manifest.members.len() {
            anyhow::bail!("team runtime started acknowledgement pane count mismatch");
        }
        let expected_owners = manifest
            .members
            .iter()
            .map(|member| member.owner.as_str())
            .collect::<BTreeSet<_>>();
        let mut observed_owners = BTreeSet::new();
        let mut pane_ids = BTreeSet::new();
        let mut session_ids = BTreeSet::new();
        for pane in &self.panes {
            pane.verify_integrity()?;
            if pane.working_dir != manifest.working_dir
                || !expected_owners.contains(pane.owner.as_str())
                || !observed_owners.insert(pane.owner.as_str())
                || !pane_ids.insert(pane.pane_id)
            {
                anyhow::bail!(
                    "team runtime started acknowledgement has contradictory pane identity"
                );
            }
            session_ids.insert(pane.session_id);
        }
        if observed_owners != expected_owners || session_ids.len() != 1 {
            anyhow::bail!(
                "team runtime started acknowledgement does not cover one exact rmux session"
            );
        }
        Ok(())
    }
}

pub fn team_member_owner(aggregate_session: &str, member_name: &str) -> Result<String> {
    let sanitized_session = crate::session::sanitize_session_name(aggregate_session);
    if aggregate_session.is_empty() || sanitized_session != aggregate_session {
        anyhow::bail!(
            "team aggregate session `{aggregate_session}` is not a canonical rmux identity; use `{sanitized_session}`"
        );
    }
    let member = sanitize_identity(member_name);
    if member_name.trim().is_empty() {
        anyhow::bail!("team member name is empty");
    }
    Ok(format!("{aggregate_session}-{member}"))
}

/// Generation-scoped member identity. Reusing `Team-X` and the same display
/// name for a later mission can never consume the prior mission's done file or
/// scope claim because the owner includes a collision-resistant mission tag.
pub fn team_member_owner_for_mission(
    aggregate_session: &str,
    member_name: &str,
    mission_id: &crate::mission::MissionId,
) -> Result<String> {
    let base = team_member_owner(aggregate_session, member_name)?;
    let generation = blake3::hash(mission_id.as_str().as_bytes())
        .to_hex()
        .to_string();
    let owner = format!("{base}-{}", &generation[..24]);
    crate::scope::validate_session_identity(&owner)?;
    Ok(owner)
}

fn validate_team_identities(config: &TeamConfig) -> Result<()> {
    let mut owners = BTreeSet::new();
    for member in &config.members {
        let owner = team_member_owner(&config.session_name, &member.name)?;
        if !owners.insert(owner.clone()) {
            anyhow::bail!(
                "team members collapse to duplicate runtime owner `{owner}`; use distinct names"
            );
        }
    }
    Ok(())
}

fn normalized_team_config(config: &TeamConfig) -> Result<TeamConfig> {
    if config.project.trim().is_empty() {
        anyhow::bail!("team project identity is empty");
    }
    let canonical_session = crate::session::sanitize_session_name(&config.session_name);
    if config.session_name.is_empty() || canonical_session != config.session_name {
        anyhow::bail!(
            "team aggregate session `{}` is not canonical; use `{canonical_session}`",
            config.session_name
        );
    }
    let canonical_working_dir = std::fs::canonicalize(&config.working_dir)
        .with_context(|| format!("canonicalizing team workspace {}", config.working_dir))?;
    let mut normalized = config.clone();
    normalized.working_dir = canonical_working_dir.to_string_lossy().to_string();
    for member in &mut normalized.members {
        member.files_owned = crate::scope::validate_scope_selectors(member.files_owned.clone())?;
    }
    validate_team_identities(&normalized)?;
    Ok(normalized)
}

impl TeamRuntimeManifest {
    fn path(&self, state_dir: &Path) -> Result<PathBuf> {
        manifest_path(state_dir, &self.aggregate_session, &self.mission_id)
    }

    fn computed_digest(&self) -> Result<String> {
        let mut unsigned = self.clone();
        unsigned.manifest_digest.clear();
        Ok(blake3::hash(&serde_json::to_vec(&unsigned)?)
            .to_hex()
            .to_string())
    }

    fn start_barrier_path(&self, state_dir: &Path) -> Result<PathBuf> {
        let canonical_state = std::fs::canonicalize(state_dir).with_context(|| {
            format!(
                "canonicalizing team runtime state directory {}",
                state_dir.display()
            )
        })?;
        Ok(canonical_state.join(format!(
            "{TEAM_START_BARRIER_PREFIX}{}-{}-{}.gate",
            self.aggregate_session,
            self.mission_id.as_str(),
            &self.manifest_digest[..16]
        )))
    }

    fn start_barrier_released(&self, state_dir: &Path) -> Result<bool> {
        let path = self.start_barrier_path(state_dir)?;
        match crate::config::read_private_optional(&path)? {
            None => Ok(false),
            Some(bytes) if bytes == self.manifest_digest.as_bytes() => Ok(true),
            Some(_) => anyhow::bail!(
                "team runtime start barrier differs from manifest generation {}",
                self.manifest_digest
            ),
        }
    }

    fn seal(mut self) -> Result<Self> {
        self.manifest_digest = self.computed_digest()?;
        Ok(self)
    }

    pub fn started_ack(&self, panes: Vec<TeamPaneActivation>) -> Result<TeamRuntimeStartedAck> {
        let mut acknowledgement = TeamRuntimeStartedAck {
            schema_version: TEAM_RUNTIME_STARTED_SCHEMA_VERSION,
            aggregate_session: self.aggregate_session.clone(),
            mission_id: self.mission_id.clone(),
            plan_revision: self.plan_revision,
            plan_digest: self.plan_digest.clone(),
            manifest_digest: self.manifest_digest.clone(),
            panes,
            activated_at: Utc::now(),
            acknowledgement_digest: String::new(),
        };
        acknowledgement.acknowledgement_digest = acknowledgement.computed_digest()?;
        acknowledgement.verify_for_manifest(self)?;
        Ok(acknowledgement)
    }

    pub fn verify_integrity(&self) -> Result<()> {
        if self.schema_version != TEAM_RUNTIME_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported team runtime schema {}; expected {}",
                self.schema_version,
                TEAM_RUNTIME_SCHEMA_VERSION
            );
        }
        if self.manifest_digest.len() != 64
            || !self
                .manifest_digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || self.computed_digest()? != self.manifest_digest
        {
            anyhow::bail!("team runtime manifest digest mismatch");
        }
        if self.members.is_empty() {
            anyhow::bail!("team runtime manifest has no members");
        }
        let canonical = std::fs::canonicalize(&self.working_dir).with_context(|| {
            format!(
                "canonicalizing team runtime workspace {}",
                self.working_dir.display()
            )
        })?;
        if canonical != self.working_dir {
            anyhow::bail!("team runtime workspace is not canonical");
        }
        let mut owners = BTreeSet::new();
        let mut tasks = BTreeSet::new();
        let mut attempts = BTreeSet::new();
        let mut panes = BTreeSet::new();
        for member in &self.members {
            if member.plan_revision != self.plan_revision {
                anyhow::bail!("team member plan revision differs from its manifest");
            }
            let expected_owner = team_member_owner_for_mission(
                &self.aggregate_session,
                &member.member_name,
                &self.mission_id,
            )?;
            if member.owner != expected_owner {
                anyhow::bail!(
                    "team member `{}` owner `{}` differs from canonical `{expected_owner}`",
                    member.member_name,
                    member.owner
                );
            }
            if !owners.insert(member.owner.clone())
                || !tasks.insert(member.task_id.clone())
                || !attempts.insert(member.attempt_id.clone())
                || !panes.insert(member.pane_index)
            {
                anyhow::bail!("team runtime manifest contains a duplicate identity");
            }
        }
        if panes != (0..self.members.len() as u32).collect::<BTreeSet<_>>() {
            anyhow::bail!("team runtime pane indexes are not contiguous from zero");
        }
        Ok(())
    }

    /// Validate the mutable projection against immutable replay, the exact
    /// active plan and every member attempt/scope binding.
    pub fn validate_against_ledger(
        &self,
        ledger: &crate::mission_ledger::MissionLedger,
        state_dir: &Path,
    ) -> Result<TeamRuntimeStatus> {
        self.verify_integrity()?;
        let mission = ledger
            .mission(&self.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team runtime mission disappeared"))?;
        if ledger.replay(&self.mission_id)? != mission {
            anyhow::bail!("team runtime mission materialization diverges from replay");
        }
        let plan = ledger
            .active_plan(&self.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team runtime mission has no active plan"))?;
        plan.verify_integrity()
            .map_err(|error| anyhow::anyhow!("team runtime plan integrity failed: {error}"))?;
        if plan.revision != self.plan_revision || plan.content_digest != self.plan_digest {
            anyhow::bail!("team runtime manifest differs from the exact active plan revision");
        }
        if plan.tasks.len() != self.members.len() {
            anyhow::bail!("team runtime member count differs from the active plan");
        }
        let replayed_attempts = ledger.replay_task_attempts(&self.mission_id)?;
        let events = ledger.events(&self.mission_id)?;
        let known_attempts = self
            .members
            .iter()
            .map(|member| member.attempt_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut scope_receipts: BTreeMap<
            String,
            Vec<crate::orchestration::AuthoritativeScopeReceipt>,
        > = BTreeMap::new();
        for event in events
            .iter()
            .filter(|event| event.kind == "scope_claim_authority_prepared")
        {
            let receipt: crate::orchestration::AuthoritativeScopeReceipt =
                serde_json::from_value(event.payload.clone()).with_context(|| {
                    format!("decoding immutable scope receipt event {}", event.event_id)
                })?;
            if !known_attempts.contains(receipt.attempt_id.as_str())
                || event.actor != "omega-scope-authority"
                || event.plan_revision != Some(receipt.plan_revision)
                || event.idempotency_key
                    != format!("orchestration:{}:scope-authority", receipt.attempt_id)
                || receipt.schema_version != 1
                || receipt.mission_id != self.mission_id
                || receipt.plan_revision != self.plan_revision
            {
                anyhow::bail!("scope receipt event differs from exact team authority");
            }
            scope_receipts
                .entry(receipt.attempt_id.clone())
                .or_default()
                .push(receipt);
        }
        let started_events = events
            .iter()
            .filter(|event| event.kind == "team_runtime_started")
            .collect::<Vec<_>>();
        if started_events.len() > 1 {
            anyhow::bail!("team runtime has multiple started acknowledgements");
        }
        let started_ack = if let Some(event) = started_events.first() {
            let recorded: TeamRuntimeStartedAck = serde_json::from_value(event.payload.clone())
                .context("decoding team runtime started acknowledgement")?;
            if event.actor != "omega-team-runtime" || event.plan_revision.is_some() {
                anyhow::bail!("team runtime started acknowledgement differs from its manifest");
            }
            recorded.verify_for_manifest(self)?;
            Some(recorded)
        } else {
            None
        };
        let started = started_ack.is_some();
        let start_barrier_released = self.start_barrier_released(state_dir)?;
        if start_barrier_released && !started {
            anyhow::bail!("team runtime start barrier exists without immutable started authority");
        }
        let mut statuses = Vec::with_capacity(self.members.len());
        for member in &self.members {
            let contract = plan
                .tasks
                .iter()
                .find(|task| task.task_id.as_str() == member.task_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "team member task {} is absent from the plan",
                        member.task_id
                    )
                })?;
            if contract.name != member.member_name || contract.scope != member.files_owned {
                anyhow::bail!(
                    "team member {} differs from its immutable task contract",
                    member.owner
                );
            }
            let attempt = ledger
                .task_attempt(&member.attempt_id)?
                .ok_or_else(|| anyhow::anyhow!("team member attempt disappeared"))?;
            if attempt.mission_id != self.mission_id
                || attempt.task_id != member.task_id
                || attempt.plan_revision != self.plan_revision
                || replayed_attempts
                    .iter()
                    .find(|candidate| candidate.attempt_id == member.attempt_id)
                    != Some(&attempt)
            {
                anyhow::bail!("team member attempt differs from immutable replay authority");
            }
            let running_actor = events.iter().find_map(|event| {
                event.resulting_task_attempt.as_ref().and_then(|result| {
                    (result.attempt_id == member.attempt_id
                        && result.state == crate::mission::TaskAttemptState::Running)
                        .then_some(event.actor.as_str())
                })
            });
            if !matches!(
                attempt.state,
                crate::mission::TaskAttemptState::Queued
                    | crate::mission::TaskAttemptState::Cancelled
            ) && running_actor != Some(member.owner.as_str())
            {
                anyhow::bail!("team member Running actor differs from its manifest owner");
            }

            let active_leases = ledger.active_leases_for_attempt(
                &self.mission_id,
                &member.task_id,
                &member.attempt_id,
            )?;
            if member.files_owned.is_empty() {
                if member.scope_claim_id.is_some()
                    || !active_leases.is_empty()
                    || scope_receipts.contains_key(&member.attempt_id)
                {
                    anyhow::bail!("read-only team member carries writable scope authority");
                }
            } else {
                let Some(receipts) = scope_receipts.get(&member.attempt_id) else {
                    if matches!(
                        attempt.state,
                        crate::mission::TaskAttemptState::Queued
                            | crate::mission::TaskAttemptState::Cancelled
                    ) && active_leases.is_empty()
                        && member.scope_claim_id.is_none()
                        && crate::scope::ScopeClaim::read_strict(state_dir, &member.owner)?
                            .is_none()
                    {
                        statuses.push(TeamMemberRuntimeStatus {
                            owner: member.owner.clone(),
                            task_id: member.task_id.clone(),
                            attempt_id: member.attempt_id.clone(),
                            state: attempt.state,
                        });
                        continue;
                    }
                    anyhow::bail!("writable team member has no immutable scope receipt");
                };
                if receipts.len() != 1 {
                    anyhow::bail!("writable team member has duplicate immutable scope receipts");
                }
                let receipt = &receipts[0];
                if receipt.mission_id != self.mission_id
                    || receipt.task_id != member.task_id
                    || receipt.plan_revision != self.plan_revision
                    || receipt.owner != member.owner
                    || receipt.claim.session != member.owner
                    || receipt.claim.files_owned != member.files_owned
                    || member
                        .scope_claim_id
                        .as_ref()
                        .is_some_and(|expected| receipt.claim.claim_id.as_ref() != Some(expected))
                {
                    anyhow::bail!("team member scope receipt differs from its manifest binding");
                }
                if active_leases
                    .iter()
                    .any(|lease| lease.owner != member.owner)
                {
                    anyhow::bail!("team member active lease belongs to another owner");
                }
                if !attempt.state.is_terminal()
                    && (active_leases.len() != member.files_owned.len()
                        || active_leases
                            .iter()
                            .map(|lease| lease.resource_key.as_str())
                            .collect::<BTreeSet<_>>()
                            .len()
                            != member.files_owned.len())
                {
                    anyhow::bail!(
                        "writable team member lacks the exact live lease cardinality from its immutable scope receipt"
                    );
                }
                match crate::scope::ScopeClaim::read_strict(state_dir, &member.owner)? {
                    Some(claim) if claim == receipt.claim => {}
                    Some(_) => anyhow::bail!(
                        "team member compatibility scope differs from immutable authority"
                    ),
                    None if active_leases.is_empty() => {}
                    None => anyhow::bail!(
                        "team member compatibility scope disappeared while leases remain active"
                    ),
                }
            }
            statuses.push(TeamMemberRuntimeStatus {
                owner: member.owner.clone(),
                task_id: member.task_id.clone(),
                attempt_id: member.attempt_id.clone(),
                state: attempt.state,
            });
        }
        Ok(TeamRuntimeStatus {
            aggregate_session: self.aggregate_session.clone(),
            mission_id: self.mission_id.clone(),
            mission_state: mission.state,
            started,
            started_ack,
            start_barrier_released,
            all_terminal: statuses.iter().all(|member| member.state.is_terminal()),
            all_accepted: statuses
                .iter()
                .all(|member| member.state == crate::mission::TaskAttemptState::Accepted),
            members: statuses,
        })
    }

    fn write_new(&self, state_dir: &Path) -> Result<()> {
        self.verify_integrity()?;
        let _lock = crate::scope::lock_private_state_file(state_dir, TEAM_RUNTIME_LOCK)?;
        let path = self.path(state_dir)?;
        if let Some(existing) = crate::config::read_private_optional(&path)? {
            let recorded: Self = serde_json::from_slice(&existing)
                .with_context(|| format!("parsing existing team runtime {}", path.display()))?;
            recorded.verify_integrity()?;
            if recorded == *self {
                return Ok(());
            }
            anyhow::bail!(
                "team runtime CAS conflict for mission {}",
                self.mission_id.as_str()
            );
        }
        crate::config::atomic_write_private(&path, &serde_json::to_vec_pretty(self)?)?;
        let recorded = load_team_runtime_path(&path)?
            .ok_or_else(|| anyhow::anyhow!("team runtime disappeared after publication"))?;
        if recorded != *self {
            anyhow::bail!("team runtime changed while being published");
        }
        Ok(())
    }
}

fn manifest_path(
    state_dir: &Path,
    aggregate_session: &str,
    mission_id: &crate::mission::MissionId,
) -> Result<PathBuf> {
    let _ = team_member_owner(aggregate_session, "manifest")?;
    if mission_id.as_str().is_empty()
        || !mission_id
            .as_str()
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        anyhow::bail!("team runtime mission id is not safe for a state filename");
    }
    Ok(state_dir.join(format!(
        "{TEAM_RUNTIME_PREFIX}{aggregate_session}-{}{TEAM_RUNTIME_SUFFIX}",
        mission_id.as_str()
    )))
}

fn load_team_runtime_path(path: &Path) -> Result<Option<TeamRuntimeManifest>> {
    let Some(bytes) = crate::config::read_private_optional(path)? else {
        return Ok(None);
    };
    let manifest: TeamRuntimeManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing team runtime manifest {}", path.display()))?;
    manifest.verify_integrity()?;
    let expected = manifest.path(
        path.parent()
            .ok_or_else(|| anyhow::anyhow!("team runtime manifest path has no parent"))?,
    )?;
    if expected != path {
        anyhow::bail!("team runtime filename differs from its immutable identity");
    }
    Ok(Some(manifest))
}

/// Enumerate persisted team runtimes without allowing one corrupt entry to
/// hide healthy authorities from Patrol. Callers must surface every entry in
/// `corrupt`; silently treating it as absent would abandon authority.
pub fn scan_team_runtime_manifests(state_dir: &Path) -> Result<TeamRuntimeManifestScan> {
    let mut manifests = Vec::new();
    let mut corrupt = Vec::new();
    let entries = match std::fs::read_dir(state_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TeamRuntimeManifestScan { manifests, corrupt });
        }
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(TEAM_RUNTIME_PREFIX) || !name.ends_with(TEAM_RUNTIME_SUFFIX) {
            continue;
        }
        let path = entry.path();
        match load_team_runtime_path(&path) {
            Ok(Some(manifest)) => manifests.push(manifest),
            Ok(None) => corrupt.push(CorruptTeamRuntimeManifest {
                path,
                error: "team runtime vanished during enumeration".to_string(),
            }),
            Err(error) => corrupt.push(CorruptTeamRuntimeManifest {
                path,
                error: format!("{error:#}"),
            }),
        }
    }
    manifests.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.mission_id.as_str().cmp(right.mission_id.as_str()))
    });
    corrupt.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(TeamRuntimeManifestScan { manifests, corrupt })
}

/// Strict caller-compatible enumeration. A corrupt authority file remains a
/// hard error, while recovery loops should use `scan_team_runtime_manifests`
/// so they can continue reconciling unrelated healthy generations.
pub fn list_team_runtime_manifests(state_dir: &Path) -> Result<Vec<TeamRuntimeManifest>> {
    let scan = scan_team_runtime_manifests(state_dir)?;
    if let Some(corrupt) = scan.corrupt.first() {
        anyhow::bail!(
            "corrupt team runtime manifest {}: {}",
            corrupt.path.display(),
            corrupt.error
        );
    }
    Ok(scan.manifests)
}

pub fn load_team_runtime_manifest(
    state_dir: &Path,
    aggregate_session: &str,
    mission_id: &crate::mission::MissionId,
) -> Result<Option<TeamRuntimeManifest>> {
    let path = manifest_path(state_dir, aggregate_session, mission_id)?;
    load_team_runtime_path(&path)
}

pub fn team_runtime_status(
    state_dir: &Path,
    manifest: &TeamRuntimeManifest,
) -> Result<TeamRuntimeStatus> {
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    manifest.validate_against_ledger(&ledger, state_dir)
}

/// Commit the activation barrier only after every pane was created. Before
/// this event, done signals are inadmissible and Patrol must reconcile the
/// runtime as an incomplete spawn.
pub fn record_team_runtime_started(
    state_dir: &Path,
    manifest: &TeamRuntimeManifest,
    acknowledgement: &TeamRuntimeStartedAck,
) -> Result<TeamRuntimeStatus> {
    acknowledgement.verify_for_manifest(manifest)?;
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    for _ in 0..TEAM_LEDGER_CAS_RETRIES {
        let status = manifest.validate_against_ledger(&ledger, state_dir)?;
        if status.started {
            if status.started_ack.as_ref() != Some(acknowledgement) {
                anyhow::bail!("team runtime already started with another pane activation");
            }
            return Ok(status);
        }
        if status.mission_state != crate::mission::MissionState::Running
            || status
                .members
                .iter()
                .any(|member| member.state != crate::mission::TaskAttemptState::Running)
        {
            anyhow::bail!(
                "team runtime cannot start until mission and every exact member attempt are Running"
            );
        }
        if ledger
            .events(&manifest.mission_id)?
            .iter()
            .any(|event| event.kind == "legacy_worker_completion_candidate")
        {
            anyhow::bail!("team runtime cannot start after a completion candidate already exists");
        }
        let mission = ledger
            .mission(&manifest.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team runtime mission disappeared"))?;
        let mut event = crate::mission_ledger::AppendEvent::new(
            manifest.mission_id.clone(),
            mission.version,
            format!(
                "team-runtime:{}:started:{}",
                manifest.mission_id.as_str(),
                manifest.manifest_digest
            ),
            "omega-team-runtime",
            "team_runtime_started",
        );
        event.correlation_id = Some(manifest.aggregate_session.clone());
        event.payload = serde_json::to_value(acknowledgement)?;
        match ledger.append(event) {
            Ok(_) => {
                let status = manifest.validate_against_ledger(&ledger, state_dir)?;
                if !status.started {
                    anyhow::bail!("team runtime started event was not replayable");
                }
                if status.started_ack.as_ref() != Some(acknowledgement) {
                    anyhow::bail!("team runtime replayed a different pane activation");
                }
                return Ok(status);
            }
            Err(crate::mission_ledger::LedgerError::VersionConflict { .. }) => continue,
            Err(crate::mission_ledger::LedgerError::IdempotencyConflict { .. }) => {
                let status = manifest.validate_against_ledger(&ledger, state_dir)?;
                if status.started && status.started_ack.as_ref() == Some(acknowledgement) {
                    return Ok(status);
                }
                anyhow::bail!(
                    "team runtime started idempotency key conflicts with another payload"
                );
            }
            Err(error) => return Err(error.into()),
        }
    }
    anyhow::bail!(
        "team runtime start acknowledgement did not converge after {} retries",
        TEAM_LEDGER_CAS_RETRIES
    )
}

/// Release the process barrier only after the immutable started event is
/// replayable. The private file is a one-generation CAS token: mismatched
/// content is corruption, never something to overwrite or auto-heal.
pub fn release_team_runtime_start_barrier(
    state_dir: &Path,
    manifest: &TeamRuntimeManifest,
) -> Result<TeamRuntimeStatus> {
    manifest.verify_integrity()?;
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let status = manifest.validate_against_ledger(&ledger, state_dir)?;
    if !status.started {
        anyhow::bail!("team runtime start barrier cannot release before team_runtime_started");
    }
    if status.start_barrier_released {
        return Ok(status);
    }

    let _lock = crate::scope::lock_private_state_file(state_dir, TEAM_RUNTIME_LOCK)?;
    let path = manifest.start_barrier_path(state_dir)?;
    if let Some(existing) = crate::config::read_private_optional(&path)? {
        if existing != manifest.manifest_digest.as_bytes() {
            anyhow::bail!("team runtime start barrier CAS conflict");
        }
    } else {
        crate::config::atomic_write_private(&path, manifest.manifest_digest.as_bytes())?;
    }
    let recorded = crate::config::read_private_optional(&path)?
        .ok_or_else(|| anyhow::anyhow!("team runtime start barrier disappeared after release"))?;
    if recorded != manifest.manifest_digest.as_bytes() {
        anyhow::bail!("team runtime start barrier changed during release");
    }
    let status = manifest.validate_against_ledger(&ledger, state_dir)?;
    if !status.start_barrier_released {
        anyhow::bail!("team runtime start barrier release is not replayable");
    }
    Ok(status)
}

/// Remove one exact manifest only after the aggregate session is confirmed
/// dead and both the mission and every member attempt are terminal. Historical
/// ambiguity is preferable to deleting live authority.
pub fn clear_team_runtime_manifest(
    state_dir: &Path,
    manifest: &TeamRuntimeManifest,
    aggregate_session_live: bool,
) -> Result<()> {
    if aggregate_session_live {
        anyhow::bail!(
            "cannot clear team runtime while aggregate session {} is live",
            manifest.aggregate_session
        );
    }
    let status = team_runtime_status(state_dir, manifest)?;
    if !status.mission_state.is_terminal() || !status.all_terminal {
        anyhow::bail!("cannot clear non-terminal team runtime authority");
    }
    let _lock = crate::scope::lock_private_state_file(state_dir, TEAM_RUNTIME_LOCK)?;
    let path = manifest.path(state_dir)?;
    let recorded = load_team_runtime_path(&path)?
        .ok_or_else(|| anyhow::anyhow!("team runtime is already absent"))?;
    if recorded != *manifest {
        anyhow::bail!("team runtime clear refused: manifest generation changed");
    }
    crate::scope::remove_private_file(&manifest.start_barrier_path(state_dir)?)?;
    crate::scope::remove_private_file(&path)
}

/// Reconcile a team only after the caller proves the aggregate rmux session is
/// absent from a fresh live-session snapshot. This is the authoritative half
/// of `omega kill Team-*`: every virtual attempt is cancelled and every exact
/// member scope generation is released. Any residual cleanup is returned and
/// the mission is left Blocked rather than falsely Failed.
pub fn reconcile_stopped_team(
    state_dir: &Path,
    aggregate_session: &str,
    live_sessions_after_kill: &[String],
) -> Result<TeamRuntimeStatus> {
    let sanitized = crate::session::sanitize_session_name(aggregate_session);
    if aggregate_session.is_empty() || sanitized != aggregate_session {
        anyhow::bail!("team stop target is not a canonical aggregate session identity");
    }
    if live_sessions_after_kill
        .iter()
        .any(|session| session == aggregate_session)
    {
        anyhow::bail!("team stop refused: aggregate session {aggregate_session} is still live");
    }
    let manifests = list_team_runtime_manifests(state_dir)?;
    let matching = manifests
        .into_iter()
        .filter(|manifest| manifest.aggregate_session == aggregate_session)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        anyhow::bail!("team stop target has no immutable runtime manifest");
    }
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let mut live = Vec::new();
    let mut terminal = Vec::new();
    for manifest in matching {
        let mission = ledger
            .mission(&manifest.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team stop mission disappeared"))?;
        if mission.state.is_terminal() {
            terminal.push(manifest);
        } else {
            live.push(manifest);
        }
    }
    if live.len() > 1 {
        anyhow::bail!("multiple non-terminal team runtimes share one aggregate session");
    }
    let manifest = if let Some(manifest) = live.pop() {
        manifest
    } else {
        let manifest = terminal
            .pop()
            .ok_or_else(|| anyhow::anyhow!("team stop runtime disappeared"))?;
        return manifest.validate_against_ledger(&ledger, state_dir);
    };
    manifest.validate_against_ledger(&ledger, state_dir)?;
    let mut attempts = Vec::with_capacity(manifest.members.len());
    for member in &manifest.members {
        attempts.push(crate::orchestration::AuthoritativeTaskAttempt {
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
        });
    }
    let authority = crate::orchestration::AuthoritativeExecution {
        mission_id: manifest.mission_id.clone(),
        plan: ledger
            .active_plan(&manifest.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team stop mission has no active plan"))?,
        attempts,
    };
    abort_team_authority(&ledger, state_dir, &authority)
        .context("reconciling stopped aggregate team authority")?;
    manifest.validate_against_ledger(&ledger, state_dir)
}

/// Resolve one virtual member only through an exact active manifest and
/// immutable Running authority. Historical manifests prevent a same-looking
/// name from silently falling through to the legacy worker path.
pub fn resolve_team_member_binding(
    state_dir: &Path,
    owner: &str,
) -> Result<Option<TeamMemberCandidateBinding>> {
    let manifests = list_team_runtime_manifests(state_dir)?;
    let ledger_path = crate::oracle_lifecycle::mission_ledger_path(state_dir);
    if manifests.is_empty() || !ledger_path.exists() {
        return Ok(None);
    }
    let ledger = crate::mission_ledger::MissionLedger::open(ledger_path)?;
    let mut historical_match = false;
    let mut active = Vec::new();
    for manifest in manifests {
        let Some(member) = manifest.members.iter().find(|member| member.owner == owner) else {
            continue;
        };
        historical_match = true;
        let status = manifest.validate_against_ledger(&ledger, state_dir)?;
        if matches!(
            status.mission_state,
            crate::mission::MissionState::Running | crate::mission::MissionState::Verifying
        ) {
            if !status.started || !status.start_barrier_released {
                anyhow::bail!(
                    "team member `{owner}` belongs to an incomplete spawn without a released runtime start barrier"
                );
            }
            let pane_activation = status
                .started_ack
                .as_ref()
                .and_then(|ack| ack.panes.iter().find(|pane| pane.owner == member.owner))
                .cloned()
                .ok_or_else(|| {
                    anyhow::anyhow!("team member has no exact stable pane activation")
                })?;
            active.push(TeamMemberCandidateBinding {
                aggregate_session: manifest.aggregate_session.clone(),
                manifest_digest: manifest.manifest_digest.clone(),
                working_dir: manifest.working_dir.clone(),
                mission_id: manifest.mission_id.clone(),
                plan_revision: manifest.plan_revision,
                task_id: member.task_id.clone(),
                attempt_id: member.attempt_id.clone(),
                owner: member.owner.clone(),
                pane_index: member.pane_index,
                pane_activation,
                files_owned: member.files_owned.clone(),
            });
        }
    }
    match active.len() {
        0 if historical_match => anyhow::bail!(
            "team member `{owner}` has no active Running/Verifying team mission; refusing legacy fallback"
        ),
        0 => Ok(None),
        1 => Ok(active.pop()),
        _ => anyhow::bail!(
            "team member `{owner}` maps to multiple active team missions; refusing ambiguous authority"
        ),
    }
}

fn member_binding_from_manifest(
    manifest: &TeamRuntimeManifest,
    owner: &str,
    acknowledgement: &TeamRuntimeStartedAck,
) -> Result<TeamMemberCandidateBinding> {
    acknowledgement.verify_for_manifest(manifest)?;
    let member = manifest
        .members
        .iter()
        .find(|member| member.owner == owner)
        .ok_or_else(|| anyhow::anyhow!("team manifest has no member `{owner}`"))?;
    Ok(TeamMemberCandidateBinding {
        aggregate_session: manifest.aggregate_session.clone(),
        manifest_digest: manifest.manifest_digest.clone(),
        working_dir: manifest.working_dir.clone(),
        mission_id: manifest.mission_id.clone(),
        plan_revision: manifest.plan_revision,
        task_id: member.task_id.clone(),
        attempt_id: member.attempt_id.clone(),
        owner: member.owner.clone(),
        pane_index: member.pane_index,
        pane_activation: acknowledgement
            .panes
            .iter()
            .find(|pane| pane.owner == member.owner)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("team manifest member has no stable pane activation"))?,
        files_owned: member.files_owned.clone(),
    })
}

fn provenance_for_team_event(
    ledger: &crate::mission_ledger::MissionLedger,
    event: &crate::mission_ledger::MissionEvent,
) -> Result<crate::done::ProjectionProvenance> {
    let projection = ledger
        .projection_at(&event.mission_id, event.sequence)?
        .ok_or_else(|| anyhow::anyhow!("team candidate event is not replayable"))?;
    Ok(crate::done::ProjectionProvenance {
        source: "mission-engine-v3.sqlite3".to_string(),
        event_id: event.event_id.clone(),
        event_sequence: event.sequence,
        mission_version: projection.version,
        projection_hash: projection.projection_hash,
    })
}

fn candidate_from_team_event(
    ledger: &crate::mission_ledger::MissionLedger,
    binding: &TeamMemberCandidateBinding,
    event: &crate::mission_ledger::MissionEvent,
) -> Result<TeamMemberCandidate> {
    if event.kind != "legacy_worker_completion_candidate"
        || event.actor != binding.owner
        || event.mission_id != binding.mission_id
        || event.task_id.as_deref() != Some(binding.task_id.as_str())
        || event.attempt_id.as_deref() != Some(binding.attempt_id.as_str())
        || event.plan_revision != Some(binding.plan_revision)
        || event.resulting_task_attempt.as_ref().is_none_or(|result| {
            result.mission_id != binding.mission_id
                || result.task_id != binding.task_id
                || result.attempt_id != binding.attempt_id
                || result.plan_revision != binding.plan_revision
                || result.state != crate::mission::TaskAttemptState::CandidateDone
        })
    {
        anyhow::bail!("team completion event differs from its exact member binding");
    }
    let mut signal: crate::done::DoneSignal = serde_json::from_value(event.payload.clone())
        .context("decoding immutable team completion payload")?;
    if signal.session != binding.owner || signal.projection.is_some() {
        anyhow::bail!("immutable team completion payload has invalid owner or embedded provenance");
    }
    signal.projection = Some(provenance_for_team_event(ledger, event)?);
    Ok(TeamMemberCandidate {
        binding: binding.clone(),
        signal,
    })
}

/// Recover the one exact member candidate from ledger replay. Patrol uses
/// this after a crash between CandidateDone append and compatibility-file
/// publication.
pub fn load_team_member_candidate(
    state_dir: &Path,
    owner: &str,
) -> Result<Option<TeamMemberCandidate>> {
    let Some(binding) = resolve_team_member_binding(state_dir, owner)? else {
        return Ok(None);
    };
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let mut matches = ledger
        .events(&binding.mission_id)?
        .into_iter()
        .filter(|event| {
            event.kind == "legacy_worker_completion_candidate"
                && event.actor == binding.owner
                && event.task_id.as_deref() == Some(binding.task_id.as_str())
                && event.attempt_id.as_deref() == Some(binding.attempt_id.as_str())
                && event.plan_revision == Some(binding.plan_revision)
        });
    let Some(event) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        anyhow::bail!("team member has multiple immutable completion candidates");
    }
    candidate_from_team_event(&ledger, &binding, &event).map(Some)
}

/// Historical recovery variant for Patrol. Unlike the live worker lookup it
/// remains valid while the mission is Verifying (or later), but the exact
/// manifest, replayed plan and member owner must still agree.
pub fn load_team_member_candidate_for_manifest(
    state_dir: &Path,
    manifest: &TeamRuntimeManifest,
    owner: &str,
) -> Result<Option<TeamMemberCandidate>> {
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let status = manifest.validate_against_ledger(&ledger, state_dir)?;
    let acknowledgement = status
        .started_ack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("team manifest has no started pane activation"))?;
    let binding = member_binding_from_manifest(manifest, owner, acknowledgement)?;
    let mut matches = ledger
        .events(&manifest.mission_id)?
        .into_iter()
        .filter(|event| {
            event.kind == "legacy_worker_completion_candidate"
                && event.actor == binding.owner
                && event.task_id.as_deref() == Some(binding.task_id.as_str())
                && event.attempt_id.as_deref() == Some(binding.attempt_id.as_str())
                && event.plan_revision == Some(binding.plan_revision)
        });
    let Some(event) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        anyhow::bail!("team member has multiple immutable completion candidates");
    }
    candidate_from_team_event(&ledger, &binding, &event).map(Some)
}

/// Append the exact CandidateDone event for a virtual member. This function
/// does not verify, accept, release scope, close the aggregate rmux session or
/// deliver the mission; those are independent Patrol/orchestrator duties.
pub fn record_team_member_candidate(
    state_dir: &Path,
    signal: &crate::done::DoneSignal,
    provider: &str,
) -> Result<Option<TeamMemberCandidate>> {
    let Some(binding) = resolve_team_member_binding(state_dir, &signal.session)? else {
        return Ok(None);
    };
    if signal.projection.is_some() {
        anyhow::bail!("team candidate input already carries projection provenance");
    }
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let payload = serde_json::to_value(signal)?;
    for _ in 0..TEAM_LEDGER_CAS_RETRIES {
        let mission = ledger
            .mission(&binding.mission_id)?
            .ok_or_else(|| anyhow::anyhow!("team candidate mission disappeared"))?;
        if mission.state != crate::mission::MissionState::Running {
            anyhow::bail!("team candidate mission is no longer Running");
        }
        let attempt = ledger
            .task_attempt(&binding.attempt_id)?
            .ok_or_else(|| anyhow::anyhow!("team candidate attempt disappeared"))?;
        if attempt.mission_id != binding.mission_id
            || attempt.task_id != binding.task_id
            || attempt.plan_revision != binding.plan_revision
        {
            anyhow::bail!("team candidate attempt identity changed");
        }
        if attempt.state == crate::mission::TaskAttemptState::CandidateDone {
            return load_team_member_candidate(state_dir, &binding.owner)?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "CandidateDone team member has no exact immutable completion event"
                    )
                })
                .map(Some);
        }
        if attempt.state != crate::mission::TaskAttemptState::Running {
            anyhow::bail!("team candidate attempt is {:?}, not Running", attempt.state);
        }
        let leases = ledger.active_leases_for_attempt(
            &binding.mission_id,
            &binding.task_id,
            &binding.attempt_id,
        )?;
        if leases.iter().any(|lease| lease.owner != binding.owner) {
            anyhow::bail!("team candidate lease owner differs from member identity");
        }
        let mut event = crate::mission_ledger::AppendEvent::new(
            binding.mission_id.clone(),
            mission.version,
            format!(
                "legacy_worker_completion_candidate:{}:{}",
                binding.owner,
                signal.finished_at.to_rfc3339()
            ),
            &binding.owner,
            "legacy_worker_completion_candidate",
        );
        event.provider = Some(provider.to_string());
        event.correlation_id = Some(binding.aggregate_session.clone());
        event.payload = payload.clone();
        event.task_attempt = Some(crate::mission_ledger::TaskAttemptMutation {
            task_id: binding.task_id.clone(),
            attempt_id: binding.attempt_id.clone(),
            plan_revision: binding.plan_revision,
            expected_version: attempt.version,
            next_state: crate::mission::TaskAttemptState::CandidateDone,
        });
        event.lease_assertions = leases
            .iter()
            .map(crate::mission_ledger::LeaseAssertion::from)
            .collect();
        match ledger.append(event) {
            Ok(outcome) => {
                return candidate_from_team_event(&ledger, &binding, &outcome.event).map(Some)
            }
            Err(crate::mission_ledger::LedgerError::VersionConflict { .. })
            | Err(crate::mission_ledger::LedgerError::AttemptVersionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    anyhow::bail!(
        "team completion candidate did not converge after {} retries",
        TEAM_LEDGER_CAS_RETRIES
    )
}

fn build_team_runtime_manifest(
    config: &TeamConfig,
    authority: &crate::orchestration::AuthoritativeExecution,
) -> Result<TeamRuntimeManifest> {
    let working_dir = std::fs::canonicalize(&config.working_dir)
        .with_context(|| format!("canonicalizing team workspace {}", config.working_dir))?;
    let mut attempts = BTreeMap::new();
    for attempt in &authority.attempts {
        attempts.insert(attempt.task_id.as_str(), attempt);
    }
    let members = config
        .members
        .iter()
        .enumerate()
        .map(|(pane_index, member)| {
            let task_id = format!(
                "team-{}-{}",
                pane_index + 1,
                sanitize_identity(&member.name)
            );
            let attempt = attempts.get(task_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!("team member {} has no prepared attempt", member.name)
            })?;
            Ok(TeamRuntimeMember {
                member_name: member.name.clone(),
                pane_index: pane_index as u32,
                owner: team_member_owner_for_mission(
                    &config.session_name,
                    &member.name,
                    &authority.mission_id,
                )?,
                task_id,
                attempt_id: attempt.attempt_id.clone(),
                plan_revision: attempt.plan_revision,
                files_owned: member.files_owned.clone(),
                // Claim ids are generated only after this pre-effect intent is
                // durable. The immutable ledger receipt later binds its exact
                // claim generation to these owner/workspace/selectors.
                scope_claim_id: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    TeamRuntimeManifest {
        schema_version: TEAM_RUNTIME_SCHEMA_VERSION,
        aggregate_session: config.session_name.clone(),
        mission_id: authority.mission_id.clone(),
        plan_revision: authority.plan.revision,
        plan_digest: authority.plan.content_digest.clone(),
        working_dir,
        provider: config.agent_command.clone(),
        created_at: Utc::now(),
        members,
        manifest_digest: String::new(),
    }
    .seal()
}

fn transition_team_attempts_running(
    ledger: &crate::mission_ledger::MissionLedger,
    config: &TeamConfig,
    legacy_plan: &crate::mission::Plan,
    authority: &crate::orchestration::AuthoritativeExecution,
) -> Result<()> {
    for (member, task) in config.members.iter().zip(&legacy_plan.tasks) {
        let attempt = authority.attempt(&task.id)?;
        let owner = team_member_owner_for_mission(
            &config.session_name,
            &member.name,
            &authority.mission_id,
        )?;
        crate::orchestration::transition_authoritative_attempt(
            ledger,
            attempt,
            crate::mission::TaskAttemptState::Running,
            &owner,
        )
        .with_context(|| {
            format!(
                "recording pre-spawn Running authority for team member {}",
                member.name
            )
        })?;
    }
    Ok(())
}

/// Freeze a team into the same V3 mission/plan/attempt contracts used by the
/// orchestrator. This function performs no rmux or provider effect, making it
/// the fail-closed preparation surface for CLI and tests.
pub fn prepare_team_authority(
    state_dir: &Path,
    config: &TeamConfig,
) -> Result<PreparedTeamAuthority> {
    let config = normalized_team_config(config)?;
    if config.members.is_empty() {
        anyhow::bail!("team has no members");
    }
    validate_team_identities(&config)?;
    let agent = resolve_team_agent(&config.agent_command)?;
    for member in &config.members {
        if member.files_owned.is_empty() && !is_explicit_read_only_role(&member.role) {
            anyhow::bail!(
                "team member `{}` has writable role `{}` but no files_owned scope; declare scope or use an explicit read-only role",
                member.name,
                member.role
            );
        }
    }
    let working_dir = PathBuf::from(&config.working_dir);
    let mission = crate::mission::Mission::new(
        &config.project,
        format!(
            "Team {}: {}",
            config.session_name,
            config
                .members
                .iter()
                .map(|member| format!("{} ({})", member.name, member.role))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        working_dir,
    );
    let tasks = config
        .members
        .iter()
        .enumerate()
        .map(|(index, member)| crate::mission::Task {
            id: format!("team-{}-{}", index + 1, sanitize_identity(&member.name)),
            name: member.name.clone(),
            prompt: member.prompt.clone(),
            files_owned: member.files_owned.clone(),
            depends_on: Vec::new(),
            agent: agent.name().to_string(),
            estimated_minutes: 60,
        })
        .collect::<Vec<_>>();
    let legacy_plan = crate::mission::Plan {
        mission_id: mission.id.clone(),
        complexity: crate::routing::Complexity::Complex,
        strategy: crate::mission::PlanStrategy::Team,
        tasks,
        created_at: chrono::Utc::now(),
    };
    let ledger = crate::mission_ledger::MissionLedger::open(
        crate::oracle_lifecycle::mission_ledger_path(state_dir),
    )?;
    let authority = crate::orchestration::prepare_authoritative_execution(
        &ledger,
        &mission,
        &legacy_plan,
        "omega-team",
        vec!["independent_verification".to_string()],
    )?;
    Ok(PreparedTeamAuthority {
        mission,
        legacy_plan,
        authority,
    })
}

pub struct TeamSpawner<'a> {
    session_mgr: &'a SessionManager,
    state_dir: Result<PathBuf, String>,
}

async fn kill_and_confirm_team(
    session_mgr: &SessionManager,
    aggregate_session: &str,
) -> Result<()> {
    let before = session_mgr
        .list_sessions()
        .await
        .with_context(|| format!("checking aggregate team session {aggregate_session}"))?;
    if before
        .iter()
        .any(|session| session.name == aggregate_session)
    {
        session_mgr
            .kill_session(aggregate_session)
            .await
            .with_context(|| format!("killing aggregate team session {aggregate_session}"))?;
    }
    let after = session_mgr
        .list_sessions()
        .await
        .with_context(|| format!("confirming aggregate team session {aggregate_session} died"))?;
    if after
        .iter()
        .any(|session| session.name == aggregate_session)
    {
        anyhow::bail!("aggregate team session {aggregate_session} remained live after kill");
    }
    Ok(())
}

async fn observe_team_pane_activation(
    owner: &str,
    pane: &Pane,
    expected_working_dir: &Path,
) -> Result<TeamPaneActivation> {
    let pane_id = pane
        .id()
        .await
        .context("reading stable rmux pane id")?
        .ok_or_else(|| anyhow::anyhow!("team pane disappeared before activation"))?;
    let snapshot = pane.info().await.context("reading rmux pane activation")?;
    let info = snapshot
        .pane(pane_id)
        .ok_or_else(|| anyhow::anyhow!("rmux info omitted the exact activated pane"))?;
    let process_pid = match &info.process {
        PaneProcessState::Running { pid: Some(pid) } if *pid > 0 => *pid,
        state => anyhow::bail!("team pane has no observed live pid at activation: {state:?}"),
    };
    if info.generation == 0 {
        anyhow::bail!("team pane has no observed process generation at activation");
    }
    let command = info
        .command
        .as_ref()
        .filter(|command| !command.is_empty())
        .ok_or_else(|| anyhow::anyhow!("team pane has no daemon-recorded start command"))?;
    let working_dir = info
        .working_directory
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("team pane has no daemon-recorded working directory"))?;
    let working_dir = std::fs::canonicalize(working_dir)
        .with_context(|| format!("canonicalizing activated pane workspace {working_dir}"))?;
    if working_dir != expected_working_dir {
        anyhow::bail!("team pane activated in a different workspace");
    }
    TeamPaneActivation {
        owner: owner.to_string(),
        pane_id,
        session_id: info.session_id,
        window_id: info.window_id,
        process_generation: info.generation,
        process_pid,
        command_digest: blake3::hash(&serde_json::to_vec(command)?)
            .to_hex()
            .to_string(),
        working_dir,
        activation_digest: String::new(),
    }
    .seal()
}

/// Re-observe every exact daemon identity before a recovery path treats a
/// Team pane as the process recorded by the started event. Renumbering panes,
/// daemon restarts and PID reuse therefore fail closed.
pub async fn validate_live_team_activation(
    session_manager: &SessionManager,
    manifest: &TeamRuntimeManifest,
    acknowledgement: &TeamRuntimeStartedAck,
) -> Result<()> {
    acknowledgement.verify_for_manifest(manifest)?;
    for expected in &acknowledgement.panes {
        validate_live_team_pane(session_manager, manifest, expected).await?;
    }
    Ok(())
}

/// Recovery validation for a partially accepted Team. Previously closed
/// terminal panes are deliberately excluded; every non-terminal member must
/// still resolve to the exact process generation in the immutable start ack.
pub async fn validate_live_team_active_members(
    session_manager: &SessionManager,
    manifest: &TeamRuntimeManifest,
    status: &TeamRuntimeStatus,
) -> Result<()> {
    if status.aggregate_session != manifest.aggregate_session
        || status.mission_id != manifest.mission_id
    {
        anyhow::bail!("team runtime status differs from its manifest identity");
    }
    let acknowledgement = status
        .started_ack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("team runtime has no immutable started acknowledgement"))?;
    acknowledgement.verify_for_manifest(manifest)?;
    for member in status
        .members
        .iter()
        .filter(|member| !member.state.is_terminal())
    {
        let expected = acknowledgement
            .panes
            .iter()
            .find(|pane| pane.owner == member.owner)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "team started acknowledgement has no active owner {}",
                    member.owner
                )
            })?;
        validate_live_team_pane(session_manager, manifest, expected).await?;
    }
    Ok(())
}

async fn validate_live_team_pane(
    session_manager: &SessionManager,
    manifest: &TeamRuntimeManifest,
    expected: &TeamPaneActivation,
) -> Result<Pane> {
    let session = session_manager
        .get_session(&manifest.aggregate_session)
        .await
        .context("resolving aggregate team session for activation validation")?;
    let pane = session
        .pane_by_id(expected.pane_id)
        .await
        .with_context(|| format!("resolving stable pane {}", expected.pane_id))?;
    validate_resolved_team_pane(&pane, expected).await?;
    Ok(pane)
}

async fn validate_resolved_team_pane(pane: &Pane, expected: &TeamPaneActivation) -> Result<()> {
    let snapshot = pane
        .info()
        .await
        .with_context(|| format!("re-observing stable pane {}", expected.pane_id))?;
    let current = snapshot
        .pane(expected.pane_id)
        .ok_or_else(|| anyhow::anyhow!("stable team pane disappeared during validation"))?;
    let current_pid = match &current.process {
        PaneProcessState::Running { pid: Some(pid) } if *pid > 0 => *pid,
        state => anyhow::bail!("stable team pane is no longer a live process: {state:?}"),
    };
    let command = current
        .command
        .as_ref()
        .filter(|command| !command.is_empty())
        .ok_or_else(|| anyhow::anyhow!("stable team pane lost its start command"))?;
    let command_digest = blake3::hash(&serde_json::to_vec(command)?)
        .to_hex()
        .to_string();
    if current.session_id != expected.session_id
        || current.window_id != expected.window_id
        || current.generation != expected.process_generation
        || current_pid != expected.process_pid
        || command_digest != expected.command_digest
    {
        anyhow::bail!(
            "stable team pane {} differs from its activated process generation",
            expected.pane_id
        );
    }
    Ok(())
}

/// Close exactly the activated member process and prove the stable pane is
/// absent before its attempt can be accepted or its scope authority released.
pub async fn close_activated_team_member_pane(
    session_manager: &SessionManager,
    manifest: &TeamRuntimeManifest,
    acknowledgement: &TeamRuntimeStartedAck,
    owner: &str,
) -> Result<()> {
    acknowledgement.verify_for_manifest(manifest)?;
    let expected = acknowledgement
        .panes
        .iter()
        .find(|pane| pane.owner == owner)
        .ok_or_else(|| anyhow::anyhow!("team started acknowledgement has no owner {owner}"))?;
    let aggregate_is_live = session_manager
        .list_sessions()
        .await
        .context("checking aggregate team session before exact pane close")?
        .iter()
        .any(|session| session.name == manifest.aggregate_session);
    if !aggregate_is_live {
        return Ok(());
    }
    let session = match session_manager
        .get_session(&manifest.aggregate_session)
        .await
    {
        Ok(session) => session,
        Err(error) => {
            let still_live = session_manager
                .list_sessions()
                .await
                .context("rechecking aggregate team session after resolution race")?
                .iter()
                .any(|session| session.name == manifest.aggregate_session);
            if !still_live {
                return Ok(());
            }
            return Err(error.context("resolving live aggregate team session for pane close"));
        }
    };
    let pane = match session.pane_by_id(expected.pane_id).await {
        Ok(pane) => pane,
        Err(rmux_sdk::RmuxError::PaneNotFound {
            session_name,
            pane_id,
            ..
        }) if session_name.as_str() == manifest.aggregate_session
            && pane_id == expected.pane_id =>
        {
            return Ok(());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("resolving stable pane {} for close", expected.pane_id));
        }
    };
    validate_resolved_team_pane(&pane, expected).await?;
    let probe = pane.clone();
    pane.close()
        .await
        .with_context(|| format!("closing exact activated pane {}", expected.pane_id))?;
    if probe
        .exists()
        .await
        .with_context(|| format!("confirming activated pane {} is absent", expected.pane_id))?
    {
        anyhow::bail!(
            "activated team pane {} remained live after close",
            expected.pane_id
        );
    }
    Ok(())
}

impl<'a> TeamSpawner<'a> {
    pub fn new(session_mgr: &'a SessionManager) -> Self {
        Self {
            session_mgr,
            state_dir: crate::config::OmegaConfig::load()
                .map(|config| config.state_dir)
                .map_err(|error| error.to_string()),
        }
    }

    /// Pin the exact state directory resolved by the caller. CLI surfaces
    /// should use this so tests/relocated installs cannot drift from the
    /// ledger used by the rest of the command.
    pub fn with_state_dir(mut self, state_dir: impl Into<PathBuf>) -> Self {
        self.state_dir = Ok(state_dir.into());
        self
    }

    pub async fn spawn_team(&self, config: &TeamConfig) -> Result<Vec<String>> {
        let config = normalized_team_config(config)?;
        let state_dir = self.state_dir.as_ref().map_err(|error| {
            anyhow::anyhow!("cannot resolve authoritative OmegaOS state directory: {error}")
        })?;
        let working_dir = PathBuf::from(&config.working_dir);
        let agent = resolve_team_agent(&config.agent_command)?;
        let live_before = self
            .session_mgr
            .list_sessions()
            .await
            .context("checking aggregate team session before creating authority")?;
        if live_before
            .iter()
            .any(|session| session.name == config.session_name)
        {
            anyhow::bail!(
                "aggregate team session {} is already live; refusing parallel authority",
                config.session_name
            );
        }
        for previous in list_team_runtime_manifests(state_dir)?
            .into_iter()
            .filter(|manifest| manifest.aggregate_session == config.session_name)
        {
            let status = team_runtime_status(state_dir, &previous)?;
            if !status.mission_state.is_terminal() {
                anyhow::bail!(
                    "aggregate team session {} has non-terminal manifest {}; reconcile it before reuse",
                    config.session_name,
                    previous.mission_id.as_str()
                );
            }
        }
        let providers = crate::providers::ProvidersConfig::try_load()
            .context("loading one immutable provider snapshot for the team")?;
        let prepared = prepare_team_authority(state_dir, &config)?;
        let mission = prepared.mission;
        let legacy_plan = prepared.legacy_plan;
        let mut authority = prepared.authority;
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(state_dir),
        )?;

        // Publish the immutable recovery intent immediately after the ledger
        // prepared the mission/plan/attempts. A crash after any later Running
        // transition or individual scope acquisition is now discoverable even
        // when no rmux process was ever created.
        let runtime = match build_team_runtime_manifest(&config, &authority) {
            Ok(runtime) => runtime,
            Err(error) => {
                let rollback = abort_team_authority(&ledger, state_dir, &authority);
                return Err(combine_team_spawn_failure(
                    error.context("building immutable team runtime intent"),
                    None,
                    rollback,
                ));
            }
        };
        if let Err(error) = runtime.write_new(state_dir) {
            let rollback = abort_team_authority(&ledger, state_dir, &authority);
            return Err(combine_team_spawn_failure(
                error.context("publishing immutable team runtime intent"),
                None,
                rollback,
            ));
        }

        if let Err(error) = crate::orchestration::transition_authoritative_mission(
            &ledger,
            &mission.id,
            crate::mission::MissionState::Running,
            "omega-team",
        ) {
            let rollback = abort_team_authority(&ledger, state_dir, &authority);
            return Err(combine_team_spawn_failure(
                error.context("starting authoritative team mission"),
                None,
                rollback,
            ));
        }

        for (member, task) in config.members.iter().zip(&legacy_plan.tasks) {
            let owner =
                team_member_owner_for_mission(&config.session_name, &member.name, &mission.id)?;
            let attempt = authority.attempt_mut(&task.id)?;
            if let Err(error) = crate::orchestration::claim_authoritative_scopes(
                &ledger,
                state_dir,
                &working_dir,
                attempt,
                &owner,
                &member.files_owned,
                Duration::from_secs(24 * 60 * 60),
            ) {
                let rollback = abort_team_authority(&ledger, state_dir, &authority);
                return Err(combine_team_spawn_failure(
                    error.context(format!("claiming team scope for {}", member.name)),
                    None,
                    rollback,
                ));
            }
        }

        // Freeze every provider launch before the first rmux side effect. The
        // command is non-secret; provider credentials remain in the structured
        // environment carried by AgentLaunch.
        let mut launches = Vec::with_capacity(config.members.len());
        for (member, task) in config.members.iter().zip(&legacy_plan.tasks) {
            let attempt = authority.attempt(&task.id)?;
            let fences = attempt
                .leases
                .iter()
                .map(|lease| format!("{}#{}", lease.resource_key, lease.fencing_token))
                .collect::<Vec<_>>()
                .join(", ");
            let agent_prompt =
                build_team_member_prompt(&config, &mission, member, task, attempt, &fences, agent)?;
            let launch = match agent.launch_with_providers(
                Some(&agent_prompt),
                crate::agents::LaunchOptions::default(),
                &providers,
            ) {
                Ok(launch) => launch,
                Err(error) => {
                    let rollback = abort_team_authority(&ledger, state_dir, &authority);
                    return Err(combine_team_spawn_failure(
                        error.context(format!(
                            "building typed provider launch for team member {}",
                            member.name
                        )),
                        None,
                        rollback,
                    ));
                }
            };
            launches.push(launch);
        }

        // Write every Running transition before the first rmux effect. The
        // session/panes may start executing immediately, so a post-spawn
        // transition leaves a real race where `omega done` observes Queued.
        // A later spawn failure cancels this exact, fully-attributed set.
        if let Err(error) =
            transition_team_attempts_running(&ledger, &config, &legacy_plan, &authority)
        {
            let rollback = abort_team_authority(&ledger, state_dir, &authority);
            return Err(combine_team_spawn_failure(error, None, rollback));
        }

        let start_barrier_path = match runtime.start_barrier_path(state_dir) {
            Ok(path) => path,
            Err(error) => {
                let rollback = abort_team_authority(&ledger, state_dir, &authority);
                return Err(combine_team_spawn_failure(
                    error.context("resolving generation-scoped team start barrier"),
                    None,
                    rollback,
                ));
            }
        };
        let gated_commands = match launches
            .iter()
            .map(|launch| {
                crate::session::gated_shell_command(
                    launch.command(),
                    &start_barrier_path,
                    &runtime.manifest_digest,
                )
            })
            .collect::<Result<Vec<_>>>()
        {
            Ok(commands) => commands,
            Err(error) => {
                let rollback = abort_team_authority(&ledger, state_dir, &authority);
                return Err(combine_team_spawn_failure(
                    error.context("building generation-gated team provider commands"),
                    None,
                    rollback,
                ));
            }
        };

        let session = match self
            .session_mgr
            .create_recorded_agent_session_create_only_gated(
                &config.session_name,
                Some(&config.working_dir),
                agent,
                launches[0].clone(),
                &start_barrier_path,
                &runtime.manifest_digest,
            )
            .await
        {
            Ok(session) => session,
            Err(error) => {
                let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
                let rollback = if kill.is_ok() {
                    abort_team_authority(&ledger, state_dir, &authority)
                } else {
                    Ok(())
                };
                return Err(combine_team_spawn_failure(
                    error.context("failed to create aggregate team session"),
                    Some(kill),
                    rollback,
                ));
            }
        };

        let first_pane = session.pane(0, 0);
        let mut pane_names = Vec::new();
        let mut pane_activations = Vec::with_capacity(config.members.len());

        for (i, (member, launch)) in config.members.iter().zip(&launches).enumerate() {
            let pane_result: Result<Pane> = async {
                let pane = if i == 0 {
                    first_pane.clone()
                } else {
                    let direction = if i % 2 == 1 {
                        SplitDirection::Right
                    } else {
                        SplitDirection::Down
                    };
                    let mut split = first_pane.split_with(direction).shell(&gated_commands[i]);
                    for (key, value) in launch.environment() {
                        split = split.env(key, value);
                    }
                    split.await?
                };
                pane.set_title(&member.name).await?;
                Ok(pane)
            }
            .await;
            let pane = match pane_result {
                Ok(pane) => pane,
                Err(error) => {
                    let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
                    let rollback = if kill.is_ok() {
                        abort_team_authority(&ledger, state_dir, &authority)
                    } else {
                        Ok(())
                    };
                    return Err(combine_team_spawn_failure(
                        error.context(format!("starting team member {}", member.name)),
                        Some(kill),
                        rollback,
                    ));
                }
            };

            let owner =
                team_member_owner_for_mission(&config.session_name, &member.name, &mission.id)?;
            let activation = match observe_team_pane_activation(&owner, &pane, &runtime.working_dir)
                .await
            {
                Ok(activation) => activation,
                Err(error) => {
                    let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
                    let rollback = if kill.is_ok() {
                        abort_team_authority(&ledger, state_dir, &authority)
                    } else {
                        Ok(())
                    };
                    return Err(combine_team_spawn_failure(
                        error.context(format!("observing team member {} activation", member.name)),
                        Some(kill),
                        rollback,
                    ));
                }
            };

            pane_names.push(owner);
            pane_activations.push(activation);
        }

        let started_ack = match runtime.started_ack(pane_activations) {
            Ok(acknowledgement) => acknowledgement,
            Err(error) => {
                let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
                let rollback = if kill.is_ok() {
                    abort_team_authority(&ledger, state_dir, &authority)
                } else {
                    Ok(())
                };
                return Err(combine_team_spawn_failure(
                    error.context("sealing stable team pane activation"),
                    Some(kill),
                    rollback,
                ));
            }
        };

        if let Err(error) = record_team_runtime_started(state_dir, &runtime, &started_ack) {
            let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
            let rollback = if kill.is_ok() {
                abort_team_authority(&ledger, state_dir, &authority)
            } else {
                Ok(())
            };
            return Err(combine_team_spawn_failure(
                error.context("committing team_runtime_started after all panes were created"),
                Some(kill),
                rollback,
            ));
        }
        if let Err(error) = release_team_runtime_start_barrier(state_dir, &runtime) {
            let kill = kill_and_confirm_team(self.session_mgr, &config.session_name).await;
            let rollback = if kill.is_ok() {
                abort_team_authority(&ledger, state_dir, &authority)
            } else {
                Ok(())
            };
            return Err(combine_team_spawn_failure(
                error.context("releasing team provider start barrier"),
                Some(kill),
                rollback,
            ));
        }

        // Even out the grid. The alternating Right/Down splits above produce a
        // lopsided binary-tree layout (pane 0 stays huge, later panes get
        // cramped) — on a client smaller than the spawn size that reads as
        // "empty space + agents you have to scroll to find". `tiled` arranges
        // every member in an even grid that reflows proportionally when the
        // window resizes to the attaching client, so the grid stays balanced
        // at attach. Best-effort: a layout hiccup must never fail the spawn.
        if config.members.len() > 1 {
            match tokio::process::Command::new("rmux")
                .args(["select-layout", "-t", &config.session_name, "tiled"])
                .output()
                .await
            {
                Ok(o) if !o.status.success() => tracing::warn!(
                    team = %config.session_name,
                    stderr = %String::from_utf8_lossy(&o.stderr),
                    "select-layout tiled failed (panes left in zigzag layout)"
                ),
                Err(e) => tracing::warn!(
                    team = %config.session_name,
                    error = %e,
                    "could not run rmux select-layout (panes left in zigzag layout)"
                ),
                _ => {}
            }
        }

        tracing::info!(
            team = %config.session_name,
            members = config.members.len(),
            "Team spawned"
        );

        Ok(pane_names)
    }
}

fn abort_team_authority(
    ledger: &crate::mission_ledger::MissionLedger,
    state_dir: &Path,
    authority: &crate::orchestration::AuthoritativeExecution,
) -> Result<()> {
    let mut failures = Vec::new();
    for attempt in &authority.attempts {
        match ledger.task_attempt(&attempt.attempt_id) {
            Ok(Some(projection)) => {
                if matches!(
                    projection.state,
                    crate::mission::TaskAttemptState::Queued
                        | crate::mission::TaskAttemptState::Running
                        | crate::mission::TaskAttemptState::CorrectionRequired
                        | crate::mission::TaskAttemptState::Blocked
                ) {
                    if let Err(error) = crate::orchestration::transition_authoritative_attempt(
                        ledger,
                        attempt,
                        crate::mission::TaskAttemptState::Cancelled,
                        attempt.owner.as_deref().unwrap_or("omega-team"),
                    ) {
                        failures.push(format!("cancel attempt {}: {error:#}", attempt.attempt_id));
                    }
                }
            }
            Ok(None) => failures.push(format!(
                "team attempt {} disappeared during rollback",
                attempt.attempt_id
            )),
            Err(error) => failures.push(format!(
                "read team attempt {} during rollback: {error}",
                attempt.attempt_id
            )),
        }
        if attempt.owner.is_some() || !attempt.leases.is_empty() || attempt.scope_receipt.is_some()
        {
            if let Err(error) =
                crate::orchestration::release_authoritative_scopes(ledger, state_dir, attempt)
            {
                failures.push(format!(
                    "release attempt {} authority: {error:#}",
                    attempt.attempt_id
                ));
            }
        }
    }

    let cleanup_incomplete = !failures.is_empty();
    match ledger.mission(&authority.mission_id) {
        Ok(Some(mission)) => {
            let terminal_result = match (mission.state, cleanup_incomplete) {
                (crate::mission::MissionState::Running, incomplete) => {
                    crate::orchestration::transition_authoritative_mission(
                        ledger,
                        &authority.mission_id,
                        crate::mission::MissionState::Verifying,
                        "omega-team-rollback",
                    )
                    .and_then(|()| {
                        crate::orchestration::transition_authoritative_mission(
                            ledger,
                            &authority.mission_id,
                            if incomplete {
                                crate::mission::MissionState::Blocked
                            } else {
                                crate::mission::MissionState::Failed
                            },
                            "omega-team-rollback",
                        )
                    })
                }
                (crate::mission::MissionState::Verifying, true) => {
                    crate::orchestration::transition_authoritative_mission(
                        ledger,
                        &authority.mission_id,
                        crate::mission::MissionState::Blocked,
                        "omega-team-rollback",
                    )
                }
                (crate::mission::MissionState::Verifying, false)
                | (crate::mission::MissionState::Blocked, false) => {
                    crate::orchestration::transition_authoritative_mission(
                        ledger,
                        &authority.mission_id,
                        crate::mission::MissionState::Failed,
                        "omega-team-rollback",
                    )
                }
                (crate::mission::MissionState::Blocked, true)
                | (crate::mission::MissionState::Failed, _)
                | (crate::mission::MissionState::Cancelled, _) => Ok(()),
                (
                    crate::mission::MissionState::Created
                    | crate::mission::MissionState::Classified
                    | crate::mission::MissionState::Planned
                    | crate::mission::MissionState::CorrectionRequired,
                    false,
                ) => crate::orchestration::transition_authoritative_mission(
                    ledger,
                    &authority.mission_id,
                    crate::mission::MissionState::Cancelled,
                    "omega-team-rollback",
                ),
                (state, true) => Err(anyhow::anyhow!(
                    "cleanup incomplete while mission is {state:?}; no honest Blocked transition exists"
                )),
                (state, false) => Err(anyhow::anyhow!(
                    "clean rollback cannot close mission from {state:?}"
                )),
            };
            if let Err(error) = terminal_result {
                failures.push(format!("record team rollback mission state: {error:#}"));
            }
        }
        Ok(None) => failures.push("team mission disappeared during rollback".to_string()),
        Err(error) => failures.push(format!("read team mission during rollback: {error}")),
    }
    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "team authority rollback incomplete: {}",
            failures.join("; ")
        )
    }
}

fn combine_team_spawn_failure(
    primary: anyhow::Error,
    kill_result: Option<Result<()>>,
    rollback_result: Result<()>,
) -> anyhow::Error {
    let mut residual = Vec::new();
    if let Some(Err(error)) = kill_result {
        residual.push(format!(
            "aggregate rmux session death was not confirmed, authority retained: {error:#}"
        ));
    }
    if let Err(error) = rollback_result {
        residual.push(format!("authority rollback failed: {error:#}"));
    }
    if residual.is_empty() {
        primary
    } else {
        anyhow::anyhow!(
            "{primary:#}; team rollback incomplete: {}",
            residual.join("; ")
        )
    }
}

fn sanitize_identity(raw: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "member".to_string()
    } else {
        sanitized
    }
}

fn resolve_team_agent(raw: &str) -> Result<crate::agents::Agent> {
    let agent = crate::agents::Agent::from_name(raw).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown team agent `{raw}`; refusing an untyped or injection-shaped launch command"
        )
    })?;
    if agent == crate::agents::Agent::Shell {
        anyhow::bail!("shell is not an agent provider and cannot own a team task");
    }
    Ok(agent)
}

fn is_explicit_read_only_role(role: &str) -> bool {
    matches!(
        role.trim().to_ascii_lowercase().as_str(),
        "read-only" | "readonly" | "reviewer" | "verifier" | "researcher" | "oracle"
    )
}

fn build_team_member_prompt(
    config: &TeamConfig,
    mission: &crate::mission::Mission,
    member: &TeamMember,
    task: &crate::mission::Task,
    attempt: &crate::orchestration::AuthoritativeTaskAttempt,
    fences: &str,
    agent: crate::agents::Agent,
) -> Result<String> {
    let owner = team_member_owner_for_mission(&config.session_name, &member.name, &mission.id)?;
    let mut prompt = format!(
        "[DISPATCHED] Team member: {} ({})\n\
         Third Law: decide and proceed, never wait.\n\n\
         {}\n\n\
         Mission ID: {}\n\
         Task ID: {}\n\
         Attempt ID: {}\n\
         Plan revision: {}\n\
         Fenced scopes: {}\n\
         Files owned: {}\n\
         When done: omega done {} done_clean \"<summary>\"",
        member.name,
        member.role,
        member.prompt,
        mission.id.as_str(),
        task.id,
        attempt.attempt_id,
        attempt.plan_revision,
        if fences.is_empty() { "none" } else { fences },
        if member.files_owned.is_empty() {
            "none (read-only)".to_string()
        } else {
            member.files_owned.join(", ")
        },
        owner,
    );
    let compiled = crate::rules::compile_rule_context_for_provider(
        crate::rules::RuleScope::Worker,
        Some(&prompt),
        crate::orchestration::provider_family_for_agent(agent),
    )
    .map_err(|error| {
        anyhow::anyhow!(
            "cannot compile policy context for team member {} using {}: {}",
            member.name,
            agent.name(),
            error
        )
    })?;
    if !compiled.markdown.is_empty() {
        prompt.push_str("\n\n");
        prompt.push_str(&compiled.markdown);
    }
    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_preparation_cannot_exist_without_plan_and_attempt_contracts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: workspace.to_string_lossy().into_owned(),
            agent_command: "codex".to_string(),
            members: vec![
                TeamMember {
                    name: "core".to_string(),
                    role: "worker".to_string(),
                    prompt: "Implement core".to_string(),
                    files_owned: vec!["src/core.rs".to_string()],
                },
                TeamMember {
                    name: "tests".to_string(),
                    role: "verifier".to_string(),
                    prompt: "Verify core".to_string(),
                    files_owned: vec!["tests/core.rs".to_string()],
                },
            ],
        };

        let prepared = prepare_team_authority(tmp.path(), &config).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(tmp.path()),
        )
        .unwrap();
        let mission = ledger.mission(&prepared.mission.id).unwrap().unwrap();
        assert_eq!(mission.state, crate::mission::MissionState::Planned);
        let plan = ledger.active_plan(&prepared.mission.id).unwrap().unwrap();
        assert_eq!(plan.tasks.len(), config.members.len());
        assert_eq!(prepared.authority.attempts.len(), config.members.len());
        for attempt in &prepared.authority.attempts {
            let projection = ledger.task_attempt(&attempt.attempt_id).unwrap().unwrap();
            assert_eq!(projection.state, crate::mission::TaskAttemptState::Queued);
            assert_eq!(projection.plan_revision, plan.revision);
        }
    }

    #[test]
    fn empty_team_is_rejected_without_creating_a_mission() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: "/tmp/OmegaOS".to_string(),
            agent_command: "codex".to_string(),
            members: Vec::new(),
        };
        assert!(prepare_team_authority(tmp.path(), &config).is_err());
        assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());
    }

    #[test]
    fn injection_shaped_or_unknown_agent_is_rejected_before_authority_creation() {
        for raw in ["codex; touch /tmp/pwned", "unknown-provider", "shell"] {
            let tmp = tempfile::TempDir::new().unwrap();
            let config = TeamConfig {
                project: "OmegaOS".to_string(),
                session_name: "Team-OmegaOS".to_string(),
                working_dir: "/tmp/OmegaOS".to_string(),
                agent_command: raw.to_string(),
                members: vec![TeamMember {
                    name: "core".to_string(),
                    role: "worker".to_string(),
                    prompt: "Implement core".to_string(),
                    files_owned: vec!["src/core.rs".to_string()],
                }],
            };
            assert!(prepare_team_authority(tmp.path(), &config).is_err());
            assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());
        }
    }

    #[test]
    fn writable_team_member_without_scope_is_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: workspace.to_string_lossy().into_owned(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: Vec::new(),
            }],
        };
        assert!(prepare_team_authority(tmp.path(), &config).is_err());
        assert!(!crate::oracle_lifecycle::mission_ledger_path(tmp.path()).exists());

        let mut read_only = config;
        read_only.members[0].role = "verifier".to_string();
        assert!(prepare_team_authority(tmp.path(), &read_only).is_ok());
    }

    #[test]
    fn team_prompt_uses_typed_provider_launch_and_canonical_rule_funnel() {
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: "/tmp/OmegaOS".to_string(),
            agent_command: "codex".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: vec!["src/core.rs".to_string()],
            }],
        };
        let mission =
            crate::mission::Mission::new("OmegaOS", "team", PathBuf::from("/tmp/OmegaOS"));
        let task = crate::mission::Task {
            id: "team-1-core".to_string(),
            name: "core".to_string(),
            prompt: "Implement core".to_string(),
            files_owned: vec!["src/core.rs".to_string()],
            depends_on: Vec::new(),
            agent: "codex".to_string(),
            estimated_minutes: 10,
        };
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: mission.id.clone(),
            task_id: task.id.clone(),
            attempt_id: "attempt-team-core-1".to_string(),
            plan_revision: 1,
            owner: Some("Team-OmegaOS-core".to_string()),
            leases: Vec::new(),
            scope_receipt: None,
        };
        let prompt = build_team_member_prompt(
            &config,
            &mission,
            &config.members[0],
            &task,
            &attempt,
            "",
            crate::agents::Agent::Codex,
        )
        .unwrap();
        assert!(prompt.contains("[L0]"), "worker rules were not injected");
        assert!(prompt.contains("Attempt ID: attempt-team-core-1"));
        assert!(!prompt.contains("[R-GOAL]"));
        assert!(!prompt.contains("[R-MODEL]"));
        let command = resolve_team_agent("codex")
            .unwrap()
            .launch_command(Some(&prompt));
        assert!(command.contains("codex"));
        assert!(command.contains("--no-alt-screen"));
        assert!(!command.starts_with("codex -p "));
    }

    #[test]
    fn team_prompt_compiles_provider_specific_rules() {
        let config = TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: "/tmp/OmegaOS".to_string(),
            agent_command: "claude".to_string(),
            members: vec![TeamMember {
                name: "core".to_string(),
                role: "worker".to_string(),
                prompt: "Implement core".to_string(),
                files_owned: vec!["src/core.rs".to_string()],
            }],
        };
        let mission =
            crate::mission::Mission::new("OmegaOS", "team", PathBuf::from("/tmp/OmegaOS"));
        let task = crate::mission::Task {
            id: "team-1-core".to_string(),
            name: "core".to_string(),
            prompt: "Implement core".to_string(),
            files_owned: vec!["src/core.rs".to_string()],
            depends_on: Vec::new(),
            agent: "claude".to_string(),
            estimated_minutes: 10,
        };
        let attempt = crate::orchestration::AuthoritativeTaskAttempt {
            mission_id: mission.id.clone(),
            task_id: task.id.clone(),
            attempt_id: "attempt-team-core-1".to_string(),
            plan_revision: 1,
            owner: Some("Team-OmegaOS-core".to_string()),
            leases: Vec::new(),
            scope_receipt: None,
        };

        for (agent, expects_claude_rules) in [
            (crate::agents::Agent::Claude, true),
            (crate::agents::Agent::Codex, false),
            (crate::agents::Agent::Gemini, false),
        ] {
            let prompt = build_team_member_prompt(
                &config,
                &mission,
                &config.members[0],
                &task,
                &attempt,
                "",
                agent,
            )
            .unwrap();
            assert_eq!(prompt.contains("[R-GOAL]"), expects_claude_rules);
            assert_eq!(prompt.contains("[R-MODEL]"), expects_claude_rules);
        }
    }

    fn runtime_config(workspace: &Path, members: Vec<TeamMember>) -> TeamConfig {
        TeamConfig {
            project: "OmegaOS".to_string(),
            session_name: "Team-OmegaOS".to_string(),
            working_dir: workspace.to_string_lossy().into_owned(),
            agent_command: "codex".to_string(),
            members,
        }
    }

    fn synthetic_started_ack(manifest: &TeamRuntimeManifest) -> TeamRuntimeStartedAck {
        let command_digest = blake3::hash(b"synthetic-test-command").to_hex().to_string();
        let panes = manifest
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                TeamPaneActivation {
                    owner: member.owner.clone(),
                    pane_id: PaneId::new(index as u32 + 1),
                    session_id: SessionId::new(1),
                    window_id: WindowId::new(1),
                    process_generation: 1,
                    process_pid: index as u32 + 1000,
                    command_digest: command_digest.clone(),
                    working_dir: manifest.working_dir.clone(),
                    activation_digest: String::new(),
                }
                .seal()
                .unwrap()
            })
            .collect();
        manifest.started_ack(panes).unwrap()
    }

    fn prepare_claimed_runtime(
        state_dir: &Path,
        workspace: &Path,
        config: &TeamConfig,
    ) -> (
        PreparedTeamAuthority,
        crate::mission_ledger::MissionLedger,
        TeamRuntimeManifest,
    ) {
        std::fs::create_dir_all(state_dir).unwrap();
        std::fs::create_dir_all(workspace).unwrap();
        let mut prepared = prepare_team_authority(state_dir, config).unwrap();
        let ledger = crate::mission_ledger::MissionLedger::open(
            crate::oracle_lifecycle::mission_ledger_path(state_dir),
        )
        .unwrap();
        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &prepared.mission.id,
            crate::mission::MissionState::Running,
            "test-team",
        )
        .unwrap();
        for (member, task) in config.members.iter().zip(&prepared.legacy_plan.tasks) {
            let owner = team_member_owner_for_mission(
                &config.session_name,
                &member.name,
                &prepared.mission.id,
            )
            .unwrap();
            crate::orchestration::claim_authoritative_scopes(
                &ledger,
                state_dir,
                workspace,
                prepared.authority.attempt_mut(&task.id).unwrap(),
                &owner,
                &member.files_owned,
                Duration::from_secs(3600),
            )
            .unwrap();
        }
        let manifest = build_team_runtime_manifest(config, &prepared.authority).unwrap();
        (prepared, ledger, manifest)
    }

    fn publish_running_runtime(
        state_dir: &Path,
        workspace: &Path,
        config: &TeamConfig,
    ) -> (
        PreparedTeamAuthority,
        crate::mission_ledger::MissionLedger,
        TeamRuntimeManifest,
    ) {
        let (prepared, ledger, manifest) = prepare_claimed_runtime(state_dir, workspace, config);
        manifest.write_new(state_dir).unwrap();
        transition_team_attempts_running(
            &ledger,
            config,
            &prepared.legacy_plan,
            &prepared.authority,
        )
        .unwrap();
        let acknowledgement = synthetic_started_ack(&manifest);
        record_team_runtime_started(state_dir, &manifest, &acknowledgement).unwrap();
        release_team_runtime_start_barrier(state_dir, &manifest).unwrap();
        (prepared, ledger, manifest)
    }

    #[test]
    fn runtime_start_barrier_is_exact_generation_bound_and_idempotent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "review".to_string(),
                files_owned: Vec::new(),
            }],
        );
        let (prepared, ledger, manifest) = prepare_claimed_runtime(&state, &workspace, &config);
        manifest.write_new(&state).unwrap();
        transition_team_attempts_running(
            &ledger,
            &config,
            &prepared.legacy_plan,
            &prepared.authority,
        )
        .unwrap();
        let owner = manifest.members[0].owner.clone();
        let signal =
            crate::done::DoneSignal::new(&owner, crate::done::DoneStatus::DoneClean, "too early");

        let before = team_runtime_status(&state, &manifest).unwrap();
        assert!(!before.started);
        assert!(!before.start_barrier_released);
        assert!(release_team_runtime_start_barrier(&state, &manifest).is_err());
        assert!(record_team_member_candidate(&state, &signal, "codex").is_err());

        let acknowledgement = synthetic_started_ack(&manifest);
        let started = record_team_runtime_started(&state, &manifest, &acknowledgement).unwrap();
        assert!(started.started);
        assert!(!started.start_barrier_released);
        assert!(record_team_member_candidate(&state, &signal, "codex").is_err());

        let barrier = manifest.start_barrier_path(&state).unwrap();
        crate::config::atomic_write_private(&barrier, b"wrong-generation").unwrap();
        assert!(release_team_runtime_start_barrier(&state, &manifest).is_err());
        crate::scope::remove_private_file(&barrier).unwrap();

        let released = release_team_runtime_start_barrier(&state, &manifest).unwrap();
        assert!(released.started);
        assert!(released.start_barrier_released);
        assert!(
            release_team_runtime_start_barrier(&state, &manifest)
                .unwrap()
                .start_barrier_released
        );
    }

    #[test]
    fn started_ack_is_bound_to_unique_stable_process_generations() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![
                TeamMember {
                    name: "one".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "one".to_string(),
                    files_owned: Vec::new(),
                },
                TeamMember {
                    name: "two".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "two".to_string(),
                    files_owned: Vec::new(),
                },
            ],
        );
        let (prepared, ledger, manifest) = prepare_claimed_runtime(&state, &workspace, &config);
        manifest.write_new(&state).unwrap();
        transition_team_attempts_running(
            &ledger,
            &config,
            &prepared.legacy_plan,
            &prepared.authority,
        )
        .unwrap();

        let acknowledgement = synthetic_started_ack(&manifest);
        acknowledgement.verify_for_manifest(&manifest).unwrap();

        let mut tampered = acknowledgement.clone();
        tampered.panes[0].process_generation += 1;
        assert!(tampered.verify_for_manifest(&manifest).is_err());

        let mut duplicate = acknowledgement.clone();
        duplicate.panes[1].pane_id = duplicate.panes[0].pane_id;
        duplicate.panes[1].activation_digest.clear();
        duplicate.panes[1] = duplicate.panes[1].clone().seal().unwrap();
        duplicate.acknowledgement_digest = duplicate.computed_digest().unwrap();
        assert!(duplicate.verify_for_manifest(&manifest).is_err());

        record_team_runtime_started(&state, &manifest, &acknowledgement).unwrap();
        let mut conflicting_panes = acknowledgement.panes.clone();
        conflicting_panes[0].process_pid += 1;
        conflicting_panes[0].activation_digest.clear();
        conflicting_panes[0] = conflicting_panes[0].clone().seal().unwrap();
        let conflicting = manifest.started_ack(conflicting_panes).unwrap();
        assert!(record_team_runtime_started(&state, &manifest, &conflicting).is_err());
    }

    #[test]
    fn manifest_intent_is_replayable_before_mission_or_claim_activation() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "writer".to_string(),
                role: "worker".to_string(),
                prompt: "write".to_string(),
                files_owned: vec!["src/**".to_string()],
            }],
        );
        let prepared = prepare_team_authority(&state, &config).unwrap();
        let manifest = build_team_runtime_manifest(&config, &prepared.authority).unwrap();
        assert!(manifest.members[0].scope_claim_id.is_none());
        manifest.write_new(&state).unwrap();

        let status = team_runtime_status(&state, &manifest).unwrap();
        assert_eq!(status.mission_state, crate::mission::MissionState::Planned);
        assert_eq!(
            status.members[0].state,
            crate::mission::TaskAttemptState::Queued
        );
        assert!(!status.started);
        assert_eq!(
            load_team_runtime_manifest(&state, &manifest.aggregate_session, &manifest.mission_id)
                .unwrap(),
            Some(manifest)
        );
    }

    #[test]
    fn absent_unstarted_aggregate_is_cancelled_and_releases_every_scope() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "writer".to_string(),
                role: "worker".to_string(),
                prompt: "write".to_string(),
                files_owned: vec!["src/**".to_string()],
            }],
        );
        let (prepared, ledger, manifest) = prepare_claimed_runtime(&state, &workspace, &config);
        manifest.write_new(&state).unwrap();
        transition_team_attempts_running(
            &ledger,
            &config,
            &prepared.legacy_plan,
            &prepared.authority,
        )
        .unwrap();
        assert!(!team_runtime_status(&state, &manifest).unwrap().started);

        let stopped = reconcile_stopped_team(&state, &config.session_name, &[]).unwrap();
        assert_eq!(stopped.mission_state, crate::mission::MissionState::Failed);
        assert!(stopped.all_terminal);
        assert!(
            crate::scope::ScopeClaim::read_strict(&state, &manifest.members[0].owner)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn canonical_member_owner_drives_prompt_and_duplicate_names_fail_closed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "Core Reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "Review core".to_string(),
                files_owned: Vec::new(),
            }],
        );
        let prepared = prepare_team_authority(tmp.path(), &config).unwrap();
        let prompt = build_team_member_prompt(
            &config,
            &prepared.mission,
            &config.members[0],
            &prepared.legacy_plan.tasks[0],
            &prepared.authority.attempts[0],
            "",
            crate::agents::Agent::Codex,
        )
        .unwrap();
        let exact_owner = team_member_owner_for_mission(
            &config.session_name,
            &config.members[0].name,
            &prepared.mission.id,
        )
        .unwrap();
        assert!(prompt.contains(&format!("omega done {exact_owner} done_clean")));
        assert!(!prompt.contains("Team-OmegaOS-Core Reviewer"));

        let duplicate = runtime_config(
            &workspace,
            vec![
                TeamMember {
                    name: "Core".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "one".to_string(),
                    files_owned: Vec::new(),
                },
                TeamMember {
                    name: "core".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "two".to_string(),
                    files_owned: Vec::new(),
                },
            ],
        );
        let state = tmp.path().join("duplicate-state");
        let error = prepare_team_authority(&state, &duplicate).unwrap_err();
        assert!(error.to_string().contains("duplicate runtime owner"));
        assert!(!crate::oracle_lifecycle::mission_ledger_path(&state).exists());
    }

    #[test]
    fn all_running_authority_is_durable_before_the_spawn_boundary() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![
                TeamMember {
                    name: "writer".to_string(),
                    role: "worker".to_string(),
                    prompt: "write".to_string(),
                    files_owned: vec!["src/**".to_string()],
                },
                TeamMember {
                    name: "reviewer".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "review".to_string(),
                    files_owned: Vec::new(),
                },
            ],
        );
        let (prepared, ledger, manifest) = prepare_claimed_runtime(&state, &workspace, &config);
        manifest.write_new(&state).unwrap();
        assert!(prepared.authority.attempts.iter().all(|attempt| {
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state
                == crate::mission::TaskAttemptState::Queued
        }));
        transition_team_attempts_running(
            &ledger,
            &config,
            &prepared.legacy_plan,
            &prepared.authority,
        )
        .unwrap();
        assert!(prepared.authority.attempts.iter().all(|attempt| {
            ledger
                .task_attempt(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .state
                == crate::mission::TaskAttemptState::Running
        }));
        assert_eq!(list_team_runtime_manifests(&state).unwrap(), vec![manifest]);
    }

    #[test]
    fn manifest_rejects_digest_corruption_stale_revision_fake_member_and_false_aggregate() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "review".to_string(),
                files_owned: Vec::new(),
            }],
        );
        let (_prepared, ledger, manifest) = publish_running_runtime(&state, &workspace, &config);
        manifest.validate_against_ledger(&ledger, &state).unwrap();

        let mut stale = manifest.clone();
        stale.plan_revision += 1;
        stale.members[0].plan_revision += 1;
        stale.manifest_digest.clear();
        let stale = stale.seal().unwrap();
        stale.verify_integrity().unwrap();
        assert!(stale.validate_against_ledger(&ledger, &state).is_err());

        let mut fake_member = manifest.clone();
        fake_member.members[0].member_name = "intruder".to_string();
        fake_member.members[0].owner = team_member_owner_for_mission(
            &fake_member.aggregate_session,
            "intruder",
            &fake_member.mission_id,
        )
        .unwrap();
        fake_member.manifest_digest.clear();
        let fake_member = fake_member.seal().unwrap();
        fake_member.verify_integrity().unwrap();
        assert!(fake_member
            .validate_against_ledger(&ledger, &state)
            .is_err());

        let mut false_aggregate = manifest.clone();
        false_aggregate.aggregate_session = "Team-Forged".to_string();
        false_aggregate.members[0].owner = team_member_owner_for_mission(
            "Team-Forged",
            &false_aggregate.members[0].member_name,
            &false_aggregate.mission_id,
        )
        .unwrap();
        false_aggregate.manifest_digest.clear();
        let false_aggregate = false_aggregate.seal().unwrap();
        false_aggregate.verify_integrity().unwrap();
        assert!(false_aggregate
            .validate_against_ledger(&ledger, &state)
            .is_err());

        let mut corrupt = manifest.clone();
        corrupt.manifest_digest = "0".repeat(64);
        crate::config::atomic_write_private(
            &manifest.path(&state).unwrap(),
            &serde_json::to_vec_pretty(&corrupt).unwrap(),
        )
        .unwrap();
        assert!(list_team_runtime_manifests(&state).is_err());
        let scan = scan_team_runtime_manifests(&state).unwrap();
        assert!(scan.manifests.is_empty());
        assert_eq!(scan.corrupt.len(), 1);
        assert_eq!(scan.corrupt[0].path, manifest.path(&state).unwrap());
    }

    #[test]
    fn candidate_append_recovers_exact_payload_after_file_write_crash() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "review".to_string(),
                files_owned: Vec::new(),
            }],
        );
        let (_prepared, ledger, manifest) = publish_running_runtime(&state, &workspace, &config);
        let owner = manifest.members[0].owner.clone();
        let mut first = crate::done::DoneSignal::new(
            &owner,
            crate::done::DoneStatus::DoneClean,
            "first immutable result",
        );
        first.todos_total = 1;
        first.todos_completed = 1;
        let recorded = record_team_member_candidate(&state, &first, "codex")
            .unwrap()
            .unwrap();
        assert!(recorded.signal.projection.is_some());
        assert!(!state.join(format!("worker-{owner}.done.json")).exists());

        let retry = crate::done::DoneSignal::new(
            &owner,
            crate::done::DoneStatus::Failed,
            "different retry payload",
        );
        let recovered = record_team_member_candidate(&state, &retry, "codex")
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&recovered.signal).unwrap(),
            serde_json::to_value(&recorded.signal).unwrap()
        );
        let recovered_again = load_team_member_candidate(&state, &owner).unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(&recovered_again.signal).unwrap(),
            serde_json::to_value(&recorded.signal).unwrap()
        );
        let candidates = ledger
            .events(&manifest.mission_id)
            .unwrap()
            .into_iter()
            .filter(|event| event.kind == "legacy_worker_completion_candidate")
            .count();
        assert_eq!(candidates, 1);
        recovered.signal.write(&state).unwrap();
        assert_eq!(
            serde_json::to_value(
                crate::done::DoneSignal::read(&state, &owner)
                    .unwrap()
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_value(&recorded.signal).unwrap()
        );

        crate::orchestration::transition_authoritative_mission(
            &ledger,
            &manifest.mission_id,
            crate::mission::MissionState::Verifying,
            "test-patrol",
        )
        .unwrap();
        assert!(load_team_member_candidate(&state, &owner)
            .unwrap()
            .is_some());
        assert!(record_team_member_candidate(&state, &retry, "codex").is_err());
    }

    #[test]
    fn rollback_surfaces_cleanup_failures_and_never_reports_false_failed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "writer".to_string(),
                role: "worker".to_string(),
                prompt: "write".to_string(),
                files_owned: vec!["src/**".to_string()],
            }],
        );
        let (prepared, ledger, _manifest) = publish_running_runtime(&state, &workspace, &config);
        let owner =
            team_member_owner_for_mission(&config.session_name, "writer", &prepared.mission.id)
                .unwrap();
        let mut changed = crate::scope::ScopeClaim::read_strict(&state, &owner)
            .unwrap()
            .unwrap();
        changed.files_owned = vec!["other/**".to_string()];
        crate::config::atomic_write_private(
            &state.join(format!("scope-{owner}.json")),
            &serde_json::to_vec_pretty(&changed).unwrap(),
        )
        .unwrap();

        let error = abort_team_authority(&ledger, &state, &prepared.authority).unwrap_err();
        assert!(error.to_string().contains("release attempt"));
        assert_eq!(
            ledger.mission(&prepared.mission.id).unwrap().unwrap().state,
            crate::mission::MissionState::Blocked
        );
        assert_eq!(
            ledger
                .task_attempt(&prepared.authority.attempts[0].attempt_id)
                .unwrap()
                .unwrap()
                .state,
            crate::mission::TaskAttemptState::Cancelled
        );
        assert!(crate::scope::ScopeClaim::read_strict(&state, &owner)
            .unwrap()
            .is_some());

        let combined = combine_team_spawn_failure(
            anyhow::anyhow!("primary spawn failure"),
            Some(Err(anyhow::anyhow!("kill failure"))),
            Err(anyhow::anyhow!("rollback failure")),
        );
        let text = combined.to_string();
        assert!(text.contains("primary spawn failure"));
        assert!(text.contains("kill failure"));
        assert!(text.contains("rollback failure"));
    }

    #[test]
    fn confirmed_team_stop_cancels_members_releases_scopes_and_is_idempotent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![
                TeamMember {
                    name: "writer".to_string(),
                    role: "worker".to_string(),
                    prompt: "write".to_string(),
                    files_owned: vec!["src/**".to_string()],
                },
                TeamMember {
                    name: "reviewer".to_string(),
                    role: "reviewer".to_string(),
                    prompt: "review".to_string(),
                    files_owned: Vec::new(),
                },
            ],
        );
        let (_prepared, _ledger, manifest) = publish_running_runtime(&state, &workspace, &config);
        assert!(reconcile_stopped_team(
            &state,
            &config.session_name,
            std::slice::from_ref(&config.session_name)
        )
        .is_err());
        for member in &manifest.members {
            assert_eq!(
                team_runtime_status(&state, &manifest)
                    .unwrap()
                    .members
                    .iter()
                    .find(|status| status.owner == member.owner)
                    .unwrap()
                    .state,
                crate::mission::TaskAttemptState::Running
            );
        }

        let stopped = reconcile_stopped_team(&state, &config.session_name, &[]).unwrap();
        assert_eq!(stopped.mission_state, crate::mission::MissionState::Failed);
        assert!(stopped.all_terminal);
        for member in &manifest.members {
            assert!(crate::scope::ScopeClaim::read_strict(&state, &member.owner)
                .unwrap()
                .is_none());
        }
        assert_eq!(
            reconcile_stopped_team(&state, &config.session_name, &[])
                .unwrap()
                .mission_state,
            crate::mission::MissionState::Failed
        );
        assert!(clear_team_runtime_manifest(&state, &manifest, true).is_err());
        clear_team_runtime_manifest(&state, &manifest, false).unwrap();
        assert!(load_team_runtime_manifest(
            &state,
            &manifest.aggregate_session,
            &manifest.mission_id
        )
        .unwrap()
        .is_none());
    }

    #[cfg(unix)]
    #[test]
    fn manifest_publication_rejects_unsafe_lock_hardlink() {
        let tmp = tempfile::TempDir::new().unwrap();
        let state = tmp.path().join("state");
        let workspace = tmp.path().join("workspace");
        let config = runtime_config(
            &workspace,
            vec![TeamMember {
                name: "reviewer".to_string(),
                role: "reviewer".to_string(),
                prompt: "review".to_string(),
                files_owned: Vec::new(),
            }],
        );
        let (_prepared, _ledger, manifest) = prepare_claimed_runtime(&state, &workspace, &config);
        let target = state.join("lock-target");
        std::fs::write(&target, b"lock").unwrap();
        std::fs::hard_link(&target, state.join(TEAM_RUNTIME_LOCK)).unwrap();
        assert!(manifest.write_new(&state).is_err());
    }
}
