//! Transactional event ledger for OmegaOS Orchestration V3.
//!
//! The ledger is the only write authority. JSON files, Telegram cards, task
//! lists and timelines are projections that can be rebuilt from these events.
//! SQLite gives the single-host runtime atomic event + projection + outbox
//! commits. External effects remain truthfully at-least-once.

use crate::mission::{
    InvalidTransition, Mission, MissionId, MissionState, PlanContract, TaskAttemptState,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub enum LedgerError {
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    Chrono(chrono::ParseError),
    MissionNotFound(String),
    MissionAlreadyExists(String),
    VersionConflict {
        expected: u64,
        actual: u64,
    },
    AttemptVersionConflict {
        attempt_id: String,
        expected: u64,
        actual: u64,
    },
    InvalidTransition(InvalidTransition),
    InvalidTaskTransition(InvalidTransition),
    InvalidInput(String),
    LeaseHeld {
        resource: String,
        owner: String,
        token: u64,
    },
    StaleFence {
        resource: String,
        expected: u64,
        actual: Option<u64>,
    },
    OutboxClaimConflict(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LedgerError::*;
        match self {
            Sqlite(error) => write!(f, "sqlite error: {error}"),
            Json(error) => write!(f, "json error: {error}"),
            Chrono(error) => write!(f, "timestamp error: {error}"),
            MissionNotFound(id) => write!(f, "mission not found: {id}"),
            MissionAlreadyExists(id) => write!(f, "mission already exists: {id}"),
            VersionConflict { expected, actual } => {
                write!(
                    f,
                    "mission version conflict: expected {expected}, actual {actual}"
                )
            }
            AttemptVersionConflict {
                attempt_id,
                expected,
                actual,
            } => write!(
                f,
                "task attempt {attempt_id} version conflict: expected {expected}, actual {actual}"
            ),
            InvalidTransition(error) | InvalidTaskTransition(error) => error.fmt(f),
            InvalidInput(message) => write!(f, "invalid ledger input: {message}"),
            LeaseHeld {
                resource,
                owner,
                token,
            } => write!(
                f,
                "lease {resource} is held by {owner} with fencing token {token}"
            ),
            StaleFence {
                resource,
                expected,
                actual,
            } => write!(
                f,
                "stale fencing token for {resource}: supplied {expected}, current {:?}",
                actual
            ),
            OutboxClaimConflict(id) => {
                write!(f, "outbox record {id} is not claimed by this worker")
            }
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<rusqlite::Error> for LedgerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<serde_json::Error> for LedgerError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<chrono::ParseError> for LedgerError {
    fn from(value: chrono::ParseError) -> Self {
        Self::Chrono(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionEvent {
    pub event_id: String,
    pub mission_id: MissionId,
    pub task_id: Option<String>,
    pub attempt_id: Option<String>,
    pub sequence: u64,
    pub expected_version: u64,
    pub schema_version: u32,
    pub idempotency_key: String,
    pub actor: String,
    pub provider: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub fencing_token: Option<u64>,
    #[serde(default)]
    pub plan_revision: Option<u64>,
    pub recorded_at: DateTime<Utc>,
    pub kind: String,
    pub payload: Value,
    pub resulting_mission_state: Option<MissionState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionProjection {
    pub mission_id: MissionId,
    pub state: MissionState,
    pub version: u64,
    pub active_plan_revision: Option<u64>,
    pub projection_hash: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttemptProjection {
    pub mission_id: MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub state: TaskAttemptState,
    pub version: u64,
    pub fencing_token: Option<u64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttemptMutation {
    pub task_id: String,
    pub attempt_id: String,
    pub plan_revision: u64,
    pub expected_version: u64,
    pub next_state: TaskAttemptState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewOutboxEffect {
    pub idempotency_key: String,
    pub kind: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppendEvent {
    pub event_id: String,
    pub mission_id: MissionId,
    pub expected_version: u64,
    pub idempotency_key: String,
    pub actor: String,
    pub provider: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub kind: String,
    pub payload: Value,
    pub next_mission_state: Option<MissionState>,
    pub task_attempt: Option<TaskAttemptMutation>,
    pub plan: Option<PlanContract>,
    pub lease_resource: Option<String>,
    pub fencing_token: Option<u64>,
    pub outbox: Vec<NewOutboxEffect>,
}

impl AppendEvent {
    pub fn new(
        mission_id: MissionId,
        expected_version: u64,
        idempotency_key: impl Into<String>,
        actor: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            event_id: stable_id("event"),
            mission_id,
            expected_version,
            idempotency_key: idempotency_key.into(),
            actor: actor.into(),
            provider: None,
            causation_id: None,
            correlation_id: None,
            kind: kind.into(),
            payload: Value::Null,
            next_mission_state: None,
            task_attempt: None,
            plan: None,
            lease_resource: None,
            fencing_token: None,
            outbox: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppendOutcome {
    pub event: MissionEvent,
    pub projection: MissionProjection,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    Active,
    Released,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaseRecord {
    pub resource_key: String,
    pub mission_id: MissionId,
    pub task_id: String,
    pub attempt_id: String,
    pub owner: String,
    pub fencing_token: u64,
    pub expires_at: DateTime<Utc>,
    pub status: LeaseStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    Pending,
    Processing,
    Delivered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub outbox_id: String,
    pub mission_id: MissionId,
    pub event_id: String,
    pub idempotency_key: String,
    pub kind: String,
    pub payload: Value,
    pub status: OutboxStatus,
    pub attempts: u32,
    pub available_at: DateTime<Utc>,
    pub claim_owner: Option<String>,
    pub claim_until: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub remote_ref: Option<String>,
}

pub struct MissionLedger {
    connection: Mutex<Connection>,
}

impl MissionLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LedgerError> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    pub fn open_in_memory() -> Result<Self, LedgerError> {
        let connection = Connection::open_in_memory()?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, LedgerError> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = FULL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS missions (
                mission_id TEXT PRIMARY KEY,
                mission_json TEXT NOT NULL,
                state_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                active_plan_revision INTEGER,
                projection_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                event_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT,
                attempt_id TEXT,
                sequence INTEGER NOT NULL,
                expected_version INTEGER NOT NULL,
                schema_version INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                actor TEXT NOT NULL,
                provider TEXT,
                causation_id TEXT,
                correlation_id TEXT,
                fencing_token INTEGER,
                plan_revision INTEGER,
                recorded_at TEXT NOT NULL,
                kind TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                mission_state_json TEXT,
                UNIQUE(mission_id, sequence),
                UNIQUE(mission_id, idempotency_key),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS plans (
                mission_id TEXT NOT NULL,
                plan_id TEXT NOT NULL,
                revision INTEGER NOT NULL,
                contract_json TEXT NOT NULL,
                content_digest TEXT NOT NULL,
                PRIMARY KEY(mission_id, revision),
                UNIQUE(mission_id, content_digest),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS task_attempts (
                attempt_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                plan_revision INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                version INTEGER NOT NULL,
                fencing_token INTEGER,
                updated_at TEXT NOT NULL,
                UNIQUE(mission_id, task_id, attempt_id),
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id)
            );

            CREATE TABLE IF NOT EXISTS leases (
                resource_key TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                attempt_id TEXT NOT NULL,
                owner TEXT NOT NULL,
                fencing_token INTEGER NOT NULL,
                expires_at TEXT NOT NULL,
                status TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS outbox (
                outbox_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                event_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                kind TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                status TEXT NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                available_at TEXT NOT NULL,
                claim_owner TEXT,
                claim_until TEXT,
                last_error TEXT,
                remote_ref TEXT,
                FOREIGN KEY(mission_id) REFERENCES missions(mission_id),
                FOREIGN KEY(event_id) REFERENCES events(event_id)
            );

            CREATE INDEX IF NOT EXISTS idx_events_mission_sequence
                ON events(mission_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_outbox_delivery
                ON outbox(status, available_at);
            "#,
        )?;
        // Forward-compatible migration for ledgers created by the first V3
        // draft, before plan_revision became part of the immutable event.
        let has_plan_revision = {
            let mut statement = connection.prepare("PRAGMA table_info(events)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            let mut found = false;
            for column in columns {
                if column? == "plan_revision" {
                    found = true;
                    break;
                }
            }
            found
        };
        if !has_plan_revision {
            connection.execute("ALTER TABLE events ADD COLUMN plan_revision INTEGER", [])?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn create_mission(
        &self,
        mission: &Mission,
        idempotency_key: &str,
        actor: &str,
    ) -> Result<AppendOutcome, LedgerError> {
        validate_key(idempotency_key, "idempotency_key")?;
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(event) =
            read_event_by_idempotency(&transaction, mission.id.as_str(), idempotency_key)?
        {
            let projection = read_projection(&transaction, mission.id.as_str())?
                .ok_or_else(|| LedgerError::MissionNotFound(mission.id.0.clone()))?;
            transaction.commit()?;
            return Ok(AppendOutcome {
                event,
                projection,
                idempotent_replay: true,
            });
        }

        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE mission_id = ?1)",
            params![mission.id.as_str()],
            |row| row.get(0),
        )?;
        if exists {
            return Err(LedgerError::MissionAlreadyExists(mission.id.0.clone()));
        }

        let now = Utc::now();
        let state = MissionState::Created;
        let projection_hash = projection_hash(&mission.id, state, 1, None)?;
        transaction.execute(
            "INSERT INTO missions (
                mission_id, mission_json, state_json, version,
                active_plan_revision, projection_hash, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 1, NULL, ?4, ?5, ?5)",
            params![
                mission.id.as_str(),
                serde_json::to_string(mission)?,
                serde_json::to_string(&state)?,
                projection_hash,
                now.to_rfc3339(),
            ],
        )?;
        let event = MissionEvent {
            event_id: stable_id("event"),
            mission_id: mission.id.clone(),
            task_id: None,
            attempt_id: None,
            sequence: 1,
            expected_version: 0,
            schema_version: SCHEMA_VERSION,
            idempotency_key: idempotency_key.to_string(),
            actor: actor.to_string(),
            provider: None,
            causation_id: None,
            correlation_id: None,
            fencing_token: None,
            plan_revision: None,
            recorded_at: now,
            kind: "mission_created".to_string(),
            payload: serde_json::to_value(mission)?,
            resulting_mission_state: Some(state),
        };
        insert_event(&transaction, &event)?;
        let projection = read_projection(&transaction, mission.id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(mission.id.0.clone()))?;
        transaction.commit()?;
        Ok(AppendOutcome {
            event,
            projection,
            idempotent_replay: false,
        })
    }

    /// Atomically append an event, update materialized projections, persist an
    /// optional immutable plan revision, and enqueue external effects.
    pub fn append(&self, request: AppendEvent) -> Result<AppendOutcome, LedgerError> {
        validate_key(&request.idempotency_key, "idempotency_key")?;
        validate_key(&request.event_id, "event_id")?;
        if let Some(plan) = &request.plan {
            plan.verify_integrity()
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
            if plan.mission_id != request.mission_id {
                return Err(LedgerError::InvalidInput(
                    "plan mission_id differs from event mission_id".to_string(),
                ));
            }
            if plan.created_from_version != request.expected_version {
                return Err(LedgerError::InvalidInput(format!(
                    "plan created_from_version {} differs from expected mission version {}",
                    plan.created_from_version, request.expected_version
                )));
            }
        }

        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(event) = read_event_by_idempotency(
            &transaction,
            request.mission_id.as_str(),
            &request.idempotency_key,
        )? {
            let projection = read_projection(&transaction, request.mission_id.as_str())?
                .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
            transaction.commit()?;
            return Ok(AppendOutcome {
                event,
                projection,
                idempotent_replay: true,
            });
        }

        let current = read_projection(&transaction, request.mission_id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
        if current.version != request.expected_version {
            return Err(LedgerError::VersionConflict {
                expected: request.expected_version,
                actual: current.version,
            });
        }
        if let Some(resource) = &request.lease_resource {
            let actual_fencing_token = current_lease_token(&transaction, resource)?;
            let supplied = request
                .fencing_token
                .ok_or_else(|| LedgerError::StaleFence {
                    resource: resource.clone(),
                    expected: 0,
                    actual: actual_fencing_token,
                })?;
            assert_fence_tx(&transaction, resource, supplied)?;
        }

        let next_state = match request.next_mission_state {
            Some(next) => current
                .state
                .transition(next)
                .map_err(LedgerError::InvalidTransition)?,
            None => current.state,
        };
        let now = Utc::now();
        let next_version = current.version.saturating_add(1);
        let task_projection = if let Some(mutation) = &request.task_attempt {
            Some(apply_task_mutation(
                &transaction,
                &request.mission_id,
                mutation,
                request.fencing_token,
                now,
            )?)
        } else {
            None
        };

        let active_plan_revision = if let Some(plan) = &request.plan {
            persist_plan(&transaction, plan)?;
            Some(plan.revision)
        } else {
            current.active_plan_revision
        };
        let hash = projection_hash(
            &request.mission_id,
            next_state,
            next_version,
            active_plan_revision,
        )?;
        transaction.execute(
            "UPDATE missions
             SET state_json = ?1, version = ?2, active_plan_revision = ?3,
                 projection_hash = ?4, updated_at = ?5
             WHERE mission_id = ?6 AND version = ?7",
            params![
                serde_json::to_string(&next_state)?,
                as_i64(next_version)?,
                active_plan_revision.map(as_i64).transpose()?,
                hash,
                now.to_rfc3339(),
                request.mission_id.as_str(),
                as_i64(request.expected_version)?,
            ],
        )?;

        let event = MissionEvent {
            event_id: request.event_id,
            mission_id: request.mission_id.clone(),
            task_id: task_projection.as_ref().map(|task| task.task_id.clone()),
            attempt_id: task_projection.as_ref().map(|task| task.attempt_id.clone()),
            sequence: next_version,
            expected_version: request.expected_version,
            schema_version: SCHEMA_VERSION,
            idempotency_key: request.idempotency_key,
            actor: request.actor,
            provider: request.provider,
            causation_id: request.causation_id,
            correlation_id: request.correlation_id,
            fencing_token: request.fencing_token,
            plan_revision: request
                .plan
                .as_ref()
                .map(|plan| plan.revision)
                .or_else(|| {
                    request
                        .task_attempt
                        .as_ref()
                        .map(|attempt| attempt.plan_revision)
                }),
            recorded_at: now,
            kind: request.kind,
            payload: request.payload,
            resulting_mission_state: request.next_mission_state,
        };
        insert_event(&transaction, &event)?;
        for (index, effect) in request.outbox.iter().enumerate() {
            insert_outbox(&transaction, &event, effect, index, now)?;
        }
        let projection = read_projection(&transaction, request.mission_id.as_str())?
            .ok_or_else(|| LedgerError::MissionNotFound(request.mission_id.0.clone()))?;
        transaction.commit()?;
        Ok(AppendOutcome {
            event,
            projection,
            idempotent_replay: false,
        })
    }

    pub fn mission(
        &self,
        mission_id: &MissionId,
    ) -> Result<Option<MissionProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        read_projection(&connection, mission_id.as_str())
    }

    /// Return the immutable plan revision currently selected by the mission
    /// projection. Compatibility readers use this to verify a legacy
    /// done.json against the checks that existed before the worker ran.
    pub fn active_plan(
        &self,
        mission_id: &MissionId,
    ) -> Result<Option<PlanContract>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let revision: Option<i64> = connection
            .query_row(
                "SELECT active_plan_revision FROM missions WHERE mission_id = ?1",
                params![mission_id.as_str()],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        let Some(revision) = revision else {
            return Ok(None);
        };
        let contract: Option<String> = connection
            .query_row(
                "SELECT contract_json FROM plans WHERE mission_id = ?1 AND revision = ?2",
                params![mission_id.as_str(), revision],
                |row| row.get(0),
            )
            .optional()?;
        contract
            .map(|json| serde_json::from_str(&json).map_err(LedgerError::from))
            .transpose()
    }

    pub fn task_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<Option<TaskAttemptProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        read_task_attempt(&connection, attempt_id)
    }

    pub fn task_attempts(
        &self,
        mission_id: &MissionId,
    ) -> Result<Vec<TaskAttemptProjection>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let mut statement = connection.prepare(
            "SELECT mission_id, task_id, attempt_id, plan_revision, state_json,
                    version, fencing_token, updated_at
             FROM task_attempts WHERE mission_id = ?1
             ORDER BY task_id, attempt_id",
        )?;
        let rows = statement.query_map(params![mission_id.as_str()], |row| {
            let mission_id: String = row.get(0)?;
            let state_json: String = row.get(4)?;
            let updated_at: String = row.get(7)?;
            Ok((
                mission_id,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                state_json,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                updated_at,
            ))
        })?;
        let mut attempts = Vec::new();
        for row in rows {
            let (
                mission_id,
                task_id,
                attempt_id,
                plan_revision,
                state_json,
                version,
                fencing_token,
                updated_at,
            ) = row?;
            attempts.push(TaskAttemptProjection {
                mission_id: MissionId(mission_id),
                task_id,
                attempt_id,
                plan_revision: as_u64(plan_revision)?,
                state: serde_json::from_str(&state_json)?,
                version: as_u64(version)?,
                fencing_token: fencing_token.map(as_u64).transpose()?,
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            });
        }
        Ok(attempts)
    }

    pub fn events(&self, mission_id: &MissionId) -> Result<Vec<MissionEvent>, LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let mut statement = connection.prepare(
            "SELECT event_id, mission_id, task_id, attempt_id, sequence,
                    expected_version, schema_version, idempotency_key, actor,
                    provider, causation_id, correlation_id, fencing_token,
                    plan_revision, recorded_at, kind, payload_json, mission_state_json
             FROM events WHERE mission_id = ?1 ORDER BY sequence ASC",
        )?;
        let rows = statement.query_map(params![mission_id.as_str()], event_row)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row??);
        }
        Ok(events)
    }

    /// Replay only the append-only event sequence. Materialized mission rows are
    /// not consulted, so this is an independent corruption/drift check.
    pub fn replay(&self, mission_id: &MissionId) -> Result<MissionProjection, LedgerError> {
        let events = self.events(mission_id)?;
        if events.is_empty() {
            return Err(LedgerError::MissionNotFound(mission_id.0.clone()));
        }
        let mut state: Option<MissionState> = None;
        let mut version = 0_u64;
        let mut active_plan_revision = None;
        let mut updated_at = events[0].recorded_at;
        for event in events {
            if event.sequence != version.saturating_add(1) {
                return Err(LedgerError::InvalidInput(format!(
                    "non-contiguous event sequence at {}",
                    event.sequence
                )));
            }
            if let Some(next) = event.resulting_mission_state {
                state = Some(match state {
                    None if next == MissionState::Created => next,
                    Some(current) => current
                        .transition(next)
                        .map_err(LedgerError::InvalidTransition)?,
                    _ => {
                        return Err(LedgerError::InvalidInput(
                            "first stateful event must create the mission".to_string(),
                        ))
                    }
                });
            }
            if let Some(plan_revision) = event.plan_revision {
                active_plan_revision = Some(plan_revision);
            }
            version = event.sequence;
            updated_at = event.recorded_at;
        }
        let state = state.ok_or_else(|| {
            LedgerError::InvalidInput("event stream contains no mission state".to_string())
        })?;
        Ok(MissionProjection {
            mission_id: mission_id.clone(),
            state,
            version,
            active_plan_revision,
            projection_hash: projection_hash(mission_id, state, version, active_plan_revision)?,
            updated_at,
        })
    }

    pub fn acquire_lease(
        &self,
        resource_key: &str,
        mission_id: &MissionId,
        task_id: &str,
        attempt_id: &str,
        owner: &str,
        ttl: Duration,
    ) -> Result<LeaseRecord, LedgerError> {
        validate_key(resource_key, "resource_key")?;
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = read_lease(&transaction, resource_key)?;
        let now = Utc::now();
        if let Some(lease) = &existing {
            if lease.status == LeaseStatus::Active && lease.expires_at > now {
                if lease.owner == owner
                    && lease.mission_id == *mission_id
                    && lease.attempt_id == attempt_id
                {
                    transaction.commit()?;
                    return Ok(lease.clone());
                }
                return Err(LedgerError::LeaseHeld {
                    resource: resource_key.to_string(),
                    owner: lease.owner.clone(),
                    token: lease.fencing_token,
                });
            }
        }
        let token = existing
            .as_ref()
            .map(|lease| lease.fencing_token.saturating_add(1))
            .unwrap_or(1);
        let expires_at = now
            + ChronoDuration::from_std(ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        let record = LeaseRecord {
            resource_key: resource_key.to_string(),
            mission_id: mission_id.clone(),
            task_id: task_id.to_string(),
            attempt_id: attempt_id.to_string(),
            owner: owner.to_string(),
            fencing_token: token,
            expires_at,
            status: LeaseStatus::Active,
        };
        transaction.execute(
            "INSERT INTO leases (
                resource_key, mission_id, task_id, attempt_id, owner,
                fencing_token, expires_at, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active')
             ON CONFLICT(resource_key) DO UPDATE SET
                mission_id = excluded.mission_id,
                task_id = excluded.task_id,
                attempt_id = excluded.attempt_id,
                owner = excluded.owner,
                fencing_token = excluded.fencing_token,
                expires_at = excluded.expires_at,
                status = 'active'",
            params![
                record.resource_key,
                record.mission_id.as_str(),
                record.task_id,
                record.attempt_id,
                record.owner,
                as_i64(record.fencing_token)?,
                record.expires_at.to_rfc3339(),
            ],
        )?;
        transaction.commit()?;
        Ok(record)
    }

    pub fn renew_lease(
        &self,
        resource_key: &str,
        owner: &str,
        fencing_token: u64,
        ttl: Duration,
    ) -> Result<LeaseRecord, LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut lease =
            read_lease(&transaction, resource_key)?.ok_or_else(|| LedgerError::StaleFence {
                resource: resource_key.to_string(),
                expected: fencing_token,
                actual: None,
            })?;
        if lease.status != LeaseStatus::Active
            || lease.fencing_token != fencing_token
            || lease.owner != owner
            || lease.expires_at <= Utc::now()
        {
            return Err(LedgerError::StaleFence {
                resource: resource_key.to_string(),
                expected: fencing_token,
                actual: Some(lease.fencing_token),
            });
        }
        lease.expires_at = Utc::now()
            + ChronoDuration::from_std(ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        transaction.execute(
            "UPDATE leases SET expires_at = ?1
             WHERE resource_key = ?2 AND owner = ?3
               AND fencing_token = ?4 AND status = 'active'",
            params![
                lease.expires_at.to_rfc3339(),
                resource_key,
                owner,
                as_i64(fencing_token)?,
            ],
        )?;
        transaction.commit()?;
        Ok(lease)
    }

    pub fn release_lease(&self, resource_key: &str, fencing_token: u64) -> Result<(), LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        assert_fence_tx(&transaction, resource_key, fencing_token)?;
        transaction.execute(
            "UPDATE leases SET status = 'released', expires_at = ?1
             WHERE resource_key = ?2 AND fencing_token = ?3",
            params![
                Utc::now().to_rfc3339(),
                resource_key,
                as_i64(fencing_token)?,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn assert_fence(&self, resource_key: &str, fencing_token: u64) -> Result<(), LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        assert_fence_tx(&connection, resource_key, fencing_token)
    }

    /// Claim pending effects for at-least-once delivery. A crash after a remote
    /// send and before `mark_outbox_delivered` may cause a duplicate; handlers
    /// must reconcile or record that possibility rather than claim exactly-once.
    pub fn claim_outbox(
        &self,
        worker: &str,
        limit: usize,
        claim_ttl: Duration,
    ) -> Result<Vec<OutboxRecord>, LedgerError> {
        let mut connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = Utc::now();
        let claim_until = now
            + ChronoDuration::from_std(claim_ttl)
                .map_err(|error| LedgerError::InvalidInput(error.to_string()))?;
        let ids = {
            let mut statement = transaction.prepare(
                "SELECT outbox_id FROM outbox
                 WHERE available_at <= ?1
                   AND (
                     status = 'pending'
                     OR (status = 'processing' AND claim_until < ?1)
                   )
                 ORDER BY available_at, outbox_id
                 LIMIT ?2",
            )?;
            let rows = statement
                .query_map(params![now.to_rfc3339(), as_i64(limit as u64)?], |row| {
                    row.get::<_, String>(0)
                })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        for id in &ids {
            transaction.execute(
                "UPDATE outbox
                 SET status = 'processing', claim_owner = ?1, claim_until = ?2,
                     attempts = attempts + 1
                 WHERE outbox_id = ?3",
                params![worker, claim_until.to_rfc3339(), id],
            )?;
        }
        let mut records = Vec::new();
        for id in ids {
            records.push(read_outbox(&transaction, &id)?.ok_or_else(|| {
                LedgerError::InvalidInput(format!("claimed outbox row disappeared: {id}"))
            })?);
        }
        transaction.commit()?;
        Ok(records)
    }

    pub fn mark_outbox_delivered(
        &self,
        outbox_id: &str,
        worker: &str,
        remote_ref: Option<&str>,
    ) -> Result<(), LedgerError> {
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let changed = connection.execute(
            "UPDATE outbox
             SET status = 'delivered', remote_ref = ?1,
                 claim_owner = NULL, claim_until = NULL, last_error = NULL
             WHERE outbox_id = ?2 AND status = 'processing' AND claim_owner = ?3",
            params![remote_ref, outbox_id, worker],
        )?;
        if changed == 0 {
            return Err(LedgerError::OutboxClaimConflict(outbox_id.to_string()));
        }
        Ok(())
    }

    pub fn mark_outbox_retry(
        &self,
        outbox_id: &str,
        worker: &str,
        error: &str,
        retry_after: Duration,
    ) -> Result<(), LedgerError> {
        let available_at = Utc::now()
            + ChronoDuration::from_std(retry_after)
                .map_err(|failure| LedgerError::InvalidInput(failure.to_string()))?;
        let connection = self
            .connection
            .lock()
            .expect("mission ledger mutex poisoned");
        let changed = connection.execute(
            "UPDATE outbox
             SET status = 'pending', available_at = ?1, last_error = ?2,
                 claim_owner = NULL, claim_until = NULL
             WHERE outbox_id = ?3 AND status = 'processing' AND claim_owner = ?4",
            params![available_at.to_rfc3339(), error, outbox_id, worker],
        )?;
        if changed == 0 {
            return Err(LedgerError::OutboxClaimConflict(outbox_id.to_string()));
        }
        Ok(())
    }
}

fn stable_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos:032x}-{counter:016x}")
}

fn validate_key(value: &str, name: &str) -> Result<(), LedgerError> {
    if value.trim().is_empty() {
        Err(LedgerError::InvalidInput(format!(
            "{name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn as_i64(value: u64) -> Result<i64, LedgerError> {
    i64::try_from(value)
        .map_err(|_| LedgerError::InvalidInput(format!("value exceeds SQLite INTEGER: {value}")))
}

fn as_u64(value: i64) -> Result<u64, LedgerError> {
    u64::try_from(value)
        .map_err(|_| LedgerError::InvalidInput(format!("negative SQLite INTEGER: {value}")))
}

fn projection_hash(
    mission_id: &MissionId,
    state: MissionState,
    version: u64,
    plan_revision: Option<u64>,
) -> Result<String, LedgerError> {
    let bytes = serde_json::to_vec(&(mission_id, state, version, plan_revision))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn persist_plan(transaction: &Transaction<'_>, plan: &PlanContract) -> Result<(), LedgerError> {
    let previous: Option<i64> = transaction
        .query_row(
            "SELECT MAX(revision) FROM plans WHERE mission_id = ?1",
            params![plan.mission_id.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    if let Some(previous) = previous {
        let previous = as_u64(previous)?;
        if plan.revision != previous.saturating_add(1) {
            return Err(LedgerError::InvalidInput(format!(
                "plan revision must advance contiguously: previous {previous}, new {}",
                plan.revision
            )));
        }
    } else if plan.revision != 1 {
        return Err(LedgerError::InvalidInput(format!(
            "first plan revision must be 1, got {}",
            plan.revision
        )));
    }
    transaction.execute(
        "INSERT INTO plans (
            mission_id, plan_id, revision, contract_json, content_digest
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            plan.mission_id.as_str(),
            plan.plan_id.0,
            as_i64(plan.revision)?,
            serde_json::to_string(plan)?,
            plan.content_digest,
        ],
    )?;
    Ok(())
}

fn apply_task_mutation(
    transaction: &Transaction<'_>,
    mission_id: &MissionId,
    mutation: &TaskAttemptMutation,
    fencing_token: Option<u64>,
    now: DateTime<Utc>,
) -> Result<TaskAttemptProjection, LedgerError> {
    let existing = read_task_attempt(transaction, &mutation.attempt_id)?;
    let (next_state, next_version) = match existing {
        None => {
            if mutation.expected_version != 0 {
                return Err(LedgerError::AttemptVersionConflict {
                    attempt_id: mutation.attempt_id.clone(),
                    expected: mutation.expected_version,
                    actual: 0,
                });
            }
            if mutation.next_state != TaskAttemptState::Queued {
                return Err(LedgerError::InvalidInput(
                    "a new task attempt must start in queued".to_string(),
                ));
            }
            (TaskAttemptState::Queued, 1)
        }
        Some(ref current) => {
            if current.version != mutation.expected_version {
                return Err(LedgerError::AttemptVersionConflict {
                    attempt_id: mutation.attempt_id.clone(),
                    expected: mutation.expected_version,
                    actual: current.version,
                });
            }
            if current.mission_id != *mission_id
                || current.task_id != mutation.task_id
                || current.plan_revision != mutation.plan_revision
            {
                return Err(LedgerError::InvalidInput(
                    "task attempt identity or plan revision changed".to_string(),
                ));
            }
            (
                current
                    .state
                    .transition(mutation.next_state)
                    .map_err(LedgerError::InvalidTaskTransition)?,
                current.version.saturating_add(1),
            )
        }
    };
    let projection = TaskAttemptProjection {
        mission_id: mission_id.clone(),
        task_id: mutation.task_id.clone(),
        attempt_id: mutation.attempt_id.clone(),
        plan_revision: mutation.plan_revision,
        state: next_state,
        version: next_version,
        fencing_token,
        updated_at: now,
    };
    transaction.execute(
        "INSERT INTO task_attempts (
            attempt_id, mission_id, task_id, plan_revision, state_json,
            version, fencing_token, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(attempt_id) DO UPDATE SET
            state_json = excluded.state_json,
            version = excluded.version,
            fencing_token = excluded.fencing_token,
            updated_at = excluded.updated_at",
        params![
            projection.attempt_id,
            projection.mission_id.as_str(),
            projection.task_id,
            as_i64(projection.plan_revision)?,
            serde_json::to_string(&projection.state)?,
            as_i64(projection.version)?,
            projection.fencing_token.map(as_i64).transpose()?,
            projection.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(projection)
}

fn insert_event(transaction: &Transaction<'_>, event: &MissionEvent) -> Result<(), LedgerError> {
    transaction.execute(
        "INSERT INTO events (
            event_id, mission_id, task_id, attempt_id, sequence,
            expected_version, schema_version, idempotency_key, actor,
            provider, causation_id, correlation_id, fencing_token,
            plan_revision, recorded_at, kind, payload_json, mission_state_json
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
         )",
        params![
            event.event_id,
            event.mission_id.as_str(),
            event.task_id,
            event.attempt_id,
            as_i64(event.sequence)?,
            as_i64(event.expected_version)?,
            i64::from(event.schema_version),
            event.idempotency_key,
            event.actor,
            event.provider,
            event.causation_id,
            event.correlation_id,
            event.fencing_token.map(as_i64).transpose()?,
            event.plan_revision.map(as_i64).transpose()?,
            event.recorded_at.to_rfc3339(),
            event.kind,
            serde_json::to_string(&event.payload)?,
            event
                .resulting_mission_state
                .map(|state| serde_json::to_string(&state))
                .transpose()?,
        ],
    )?;
    Ok(())
}

fn insert_outbox(
    transaction: &Transaction<'_>,
    event: &MissionEvent,
    effect: &NewOutboxEffect,
    index: usize,
    now: DateTime<Utc>,
) -> Result<(), LedgerError> {
    validate_key(&effect.idempotency_key, "outbox idempotency_key")?;
    transaction.execute(
        "INSERT INTO outbox (
            outbox_id, mission_id, event_id, idempotency_key, kind,
            payload_json, status, attempts, available_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', 0, ?7)",
        params![
            format!("outbox-{}-{index}", event.event_id),
            event.mission_id.as_str(),
            event.event_id,
            effect.idempotency_key,
            effect.kind,
            serde_json::to_string(&effect.payload)?,
            now.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn read_projection(
    connection: &Connection,
    mission_id: &str,
) -> Result<Option<MissionProjection>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT state_json, version, active_plan_revision,
                    projection_hash, updated_at
             FROM missions WHERE mission_id = ?1",
            params![mission_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?;
    row.map(|(state, version, plan_revision, hash, updated_at)| {
        Ok(MissionProjection {
            mission_id: MissionId(mission_id.to_string()),
            state: serde_json::from_str(&state)?,
            version: as_u64(version)?,
            active_plan_revision: plan_revision.map(as_u64).transpose()?,
            projection_hash: hash,
            updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
        })
    })
    .transpose()
}

fn read_task_attempt(
    connection: &Connection,
    attempt_id: &str,
) -> Result<Option<TaskAttemptProjection>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, task_id, plan_revision, state_json,
                    version, fencing_token, updated_at
             FROM task_attempts WHERE attempt_id = ?1",
            params![attempt_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(mission_id, task_id, plan_revision, state, version, token, updated_at)| {
            Ok(TaskAttemptProjection {
                mission_id: MissionId(mission_id),
                task_id,
                attempt_id: attempt_id.to_string(),
                plan_revision: as_u64(plan_revision)?,
                state: serde_json::from_str(&state)?,
                version: as_u64(version)?,
                fencing_token: token.map(as_u64).transpose()?,
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            })
        },
    )
    .transpose()
}

fn read_event_by_idempotency(
    connection: &Connection,
    mission_id: &str,
    idempotency_key: &str,
) -> Result<Option<MissionEvent>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT event_id, mission_id, task_id, attempt_id, sequence,
                    expected_version, schema_version, idempotency_key, actor,
                    provider, causation_id, correlation_id, fencing_token,
                    plan_revision, recorded_at, kind, payload_json, mission_state_json
             FROM events WHERE mission_id = ?1 AND idempotency_key = ?2",
            params![mission_id, idempotency_key],
            event_row,
        )
        .optional()?;
    row.transpose()
}

fn event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<MissionEvent, LedgerError>> {
    let event_id = row.get::<_, String>(0)?;
    let mission_id = row.get::<_, String>(1)?;
    let task_id = row.get::<_, Option<String>>(2)?;
    let attempt_id = row.get::<_, Option<String>>(3)?;
    let sequence = row.get::<_, i64>(4)?;
    let expected_version = row.get::<_, i64>(5)?;
    let schema_version = row.get::<_, i64>(6)?;
    let idempotency_key = row.get::<_, String>(7)?;
    let actor = row.get::<_, String>(8)?;
    let provider = row.get::<_, Option<String>>(9)?;
    let causation_id = row.get::<_, Option<String>>(10)?;
    let correlation_id = row.get::<_, Option<String>>(11)?;
    let fencing_token = row.get::<_, Option<i64>>(12)?;
    let plan_revision = row.get::<_, Option<i64>>(13)?;
    let recorded_at = row.get::<_, String>(14)?;
    let kind = row.get::<_, String>(15)?;
    let payload = row.get::<_, String>(16)?;
    let state = row.get::<_, Option<String>>(17)?;
    Ok((|| {
        Ok(MissionEvent {
            event_id,
            mission_id: MissionId(mission_id),
            task_id,
            attempt_id,
            sequence: as_u64(sequence)?,
            expected_version: as_u64(expected_version)?,
            schema_version: u32::try_from(schema_version).map_err(|_| {
                LedgerError::InvalidInput(format!("invalid schema version: {schema_version}"))
            })?,
            idempotency_key,
            actor,
            provider,
            causation_id,
            correlation_id,
            fencing_token: fencing_token.map(as_u64).transpose()?,
            plan_revision: plan_revision.map(as_u64).transpose()?,
            recorded_at: DateTime::parse_from_rfc3339(&recorded_at)?.with_timezone(&Utc),
            kind,
            payload: serde_json::from_str(&payload)?,
            resulting_mission_state: state
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
        })
    })())
}

fn read_lease(
    connection: &Connection,
    resource_key: &str,
) -> Result<Option<LeaseRecord>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, task_id, attempt_id, owner,
                    fencing_token, expires_at, status
             FROM leases WHERE resource_key = ?1",
            params![resource_key],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(mission_id, task_id, attempt_id, owner, token, expires_at, status)| {
            Ok(LeaseRecord {
                resource_key: resource_key.to_string(),
                mission_id: MissionId(mission_id),
                task_id,
                attempt_id,
                owner,
                fencing_token: as_u64(token)?,
                expires_at: DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc),
                status: match status.as_str() {
                    "active" => LeaseStatus::Active,
                    "released" => LeaseStatus::Released,
                    other => {
                        return Err(LedgerError::InvalidInput(format!(
                            "unknown lease status: {other}"
                        )))
                    }
                },
            })
        },
    )
    .transpose()
}

fn current_lease_token(
    connection: &Connection,
    resource_key: &str,
) -> Result<Option<u64>, LedgerError> {
    read_lease(connection, resource_key).map(|lease| lease.map(|lease| lease.fencing_token))
}

fn assert_fence_tx(
    connection: &Connection,
    resource_key: &str,
    fencing_token: u64,
) -> Result<(), LedgerError> {
    let lease = read_lease(connection, resource_key)?;
    match lease {
        Some(lease)
            if lease.status == LeaseStatus::Active
                && lease.fencing_token == fencing_token
                && lease.expires_at > Utc::now() =>
        {
            Ok(())
        }
        other => Err(LedgerError::StaleFence {
            resource: resource_key.to_string(),
            expected: fencing_token,
            actual: other.map(|lease| lease.fencing_token),
        }),
    }
}

fn read_outbox(
    connection: &Connection,
    outbox_id: &str,
) -> Result<Option<OutboxRecord>, LedgerError> {
    let row = connection
        .query_row(
            "SELECT mission_id, event_id, idempotency_key, kind, payload_json,
                    status, attempts, available_at, claim_owner, claim_until,
                    last_error, remote_ref
             FROM outbox WHERE outbox_id = ?1",
            params![outbox_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        )
        .optional()?;
    row.map(
        |(
            mission_id,
            event_id,
            idempotency_key,
            kind,
            payload,
            status,
            attempts,
            available_at,
            claim_owner,
            claim_until,
            last_error,
            remote_ref,
        )| {
            Ok(OutboxRecord {
                outbox_id: outbox_id.to_string(),
                mission_id: MissionId(mission_id),
                event_id,
                idempotency_key,
                kind,
                payload: serde_json::from_str(&payload)?,
                status: match status.as_str() {
                    "pending" => OutboxStatus::Pending,
                    "processing" => OutboxStatus::Processing,
                    "delivered" => OutboxStatus::Delivered,
                    other => {
                        return Err(LedgerError::InvalidInput(format!(
                            "unknown outbox status: {other}"
                        )))
                    }
                },
                attempts: u32::try_from(attempts).map_err(|_| {
                    LedgerError::InvalidInput(format!("invalid outbox attempts value: {attempts}"))
                })?,
                available_at: DateTime::parse_from_rfc3339(&available_at)?.with_timezone(&Utc),
                claim_owner,
                claim_until: claim_until
                    .map(|value| {
                        DateTime::parse_from_rfc3339(&value).map(|dt| dt.with_timezone(&Utc))
                    })
                    .transpose()?,
                last_error,
                remote_ref,
            })
        },
    )
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn mission(id: &str) -> Mission {
        let mut mission = Mission::new("OmegaOS", "test mission", PathBuf::from("/tmp"));
        mission.id = MissionId(id.to_string());
        mission
    }

    fn transition(
        mission_id: &MissionId,
        expected_version: u64,
        key: &str,
        next: MissionState,
    ) -> AppendEvent {
        let mut request = AppendEvent::new(
            mission_id.clone(),
            expected_version,
            key,
            "test",
            "mission_transition",
        );
        request.next_mission_state = Some(next);
        request
    }

    #[test]
    fn cas_and_idempotency_are_fail_closed() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-cas");
        let created = ledger
            .create_mission(&mission, "create-m-cas", "test")
            .unwrap();
        assert_eq!(created.projection.version, 1);

        let first = ledger
            .append(transition(
                &mission.id,
                1,
                "classify-once",
                MissionState::Classified,
            ))
            .unwrap();
        assert_eq!(first.projection.version, 2);
        let replay = ledger
            .append(transition(
                &mission.id,
                1,
                "classify-once",
                MissionState::Classified,
            ))
            .unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.event.event_id, first.event.event_id);
        assert!(matches!(
            ledger.append(transition(
                &mission.id,
                1,
                "different-command",
                MissionState::Classified,
            )),
            Err(LedgerError::VersionConflict {
                expected: 1,
                actual: 2
            })
        ));
    }

    #[test]
    fn concurrent_expected_version_has_one_winner() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("ledger.sqlite");
        let ledger = MissionLedger::open(&db).unwrap();
        let mission = mission("m-race");
        ledger
            .create_mission(&mission, "create-m-race", "test")
            .unwrap();
        drop(ledger);

        let workers = 12;
        let barrier = Arc::new(Barrier::new(workers));
        let mut joins = Vec::new();
        for index in 0..workers {
            let db = db.clone();
            let barrier = Arc::clone(&barrier);
            let mission_id = mission.id.clone();
            joins.push(thread::spawn(move || {
                let ledger = MissionLedger::open(db).unwrap();
                barrier.wait();
                ledger.append(transition(
                    &mission_id,
                    1,
                    &format!("race-{index}"),
                    MissionState::Classified,
                ))
            }));
        }
        let results: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(
                    result,
                    Err(LedgerError::VersionConflict {
                        expected: 1,
                        actual: 2
                    })
                ))
                .count(),
            workers - 1
        );
    }

    #[test]
    fn replay_matches_materialized_projection() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-replay");
        ledger
            .create_mission(&mission, "create-m-replay", "test")
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                1,
                "classify-replay",
                MissionState::Classified,
            ))
            .unwrap();
        ledger
            .append(transition(
                &mission.id,
                2,
                "plan-replay",
                MissionState::Planned,
            ))
            .unwrap();
        let materialized = ledger.mission(&mission.id).unwrap().unwrap();
        let replayed = ledger.replay(&mission.id).unwrap();
        assert_eq!(materialized.state, replayed.state);
        assert_eq!(materialized.version, replayed.version);
        assert_eq!(materialized.projection_hash, replayed.projection_hash);
    }

    #[test]
    fn replay_reconstructs_plan_revision_from_typed_event_field() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-plan-replay");
        ledger
            .create_mission(&mission, "create-plan-replay", "test")
            .unwrap();
        let plan = PlanContract::new(mission.id.clone(), 1, 1, Vec::new(), Vec::new(), Vec::new())
            .unwrap();
        let mut event = AppendEvent::new(
            mission.id.clone(),
            1,
            "record-plan",
            "test",
            "arbitrary_event_name",
        );
        event.next_mission_state = Some(MissionState::Classified);
        event.plan = Some(plan);
        let materialized = ledger.append(event).unwrap().projection;
        let replayed = ledger.replay(&mission.id).unwrap();
        assert_eq!(materialized.active_plan_revision, Some(1));
        assert_eq!(replayed.active_plan_revision, Some(1));
        assert_eq!(materialized.projection_hash, replayed.projection_hash);
    }

    #[test]
    fn outbox_is_transactional_and_idempotent() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-outbox");
        ledger
            .create_mission(&mission, "create-m-outbox", "test")
            .unwrap();
        let mut request = transition(
            &mission.id,
            1,
            "classify-with-notify",
            MissionState::Classified,
        );
        request.outbox.push(NewOutboxEffect {
            idempotency_key: "notify-m-outbox".to_string(),
            kind: "telegram_message".to_string(),
            payload: serde_json::json!({"text": "classified"}),
        });
        ledger.append(request.clone()).unwrap();
        ledger.append(request).unwrap();

        let claimed = ledger
            .claim_outbox("notifier", 10, Duration::from_secs(30))
            .unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].attempts, 1);
        ledger
            .mark_outbox_delivered(&claimed[0].outbox_id, "notifier", Some("telegram:42"))
            .unwrap();
        assert!(ledger
            .claim_outbox("notifier", 10, Duration::from_secs(30))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn lease_fencing_rejects_aba_writer() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-lease");
        ledger
            .create_mission(&mission, "create-m-lease", "test")
            .unwrap();
        let first = ledger
            .acquire_lease(
                "src/auth.rs",
                &mission.id,
                "auth",
                "attempt-1",
                "worker-a",
                Duration::ZERO,
            )
            .unwrap();
        let second = ledger
            .acquire_lease(
                "src/auth.rs",
                &mission.id,
                "auth",
                "attempt-2",
                "worker-b",
                Duration::from_secs(30),
            )
            .unwrap();
        assert!(second.fencing_token > first.fencing_token);
        assert!(matches!(
            ledger.assert_fence("src/auth.rs", first.fencing_token),
            Err(LedgerError::StaleFence { .. })
        ));
        ledger
            .assert_fence("src/auth.rs", second.fencing_token)
            .unwrap();
    }

    #[test]
    fn task_attempt_cannot_skip_candidate_verification() {
        let ledger = MissionLedger::open_in_memory().unwrap();
        let mission = mission("m-task");
        ledger
            .create_mission(&mission, "create-m-task", "test")
            .unwrap();
        let mut queued =
            AppendEvent::new(mission.id.clone(), 1, "queue-task", "engine", "task_queued");
        queued.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a1".to_string(),
            plan_revision: 1,
            expected_version: 0,
            next_state: TaskAttemptState::Queued,
        });
        ledger.append(queued).unwrap();

        let mut invalid = AppendEvent::new(
            mission.id.clone(),
            2,
            "accept-task-directly",
            "engine",
            "task_accepted",
        );
        invalid.task_attempt = Some(TaskAttemptMutation {
            task_id: "task-a".to_string(),
            attempt_id: "attempt-a1".to_string(),
            plan_revision: 1,
            expected_version: 1,
            next_state: TaskAttemptState::Accepted,
        });
        assert!(matches!(
            ledger.append(invalid),
            Err(LedgerError::InvalidTaskTransition(_))
        ));
    }
}
