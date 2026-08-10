//! Centralized wire-protocol types for the omega-gateway HTTP/WS API.
//!
//! This module is the source of truth for the app's TypeScript types: every
//! request/response/frame shape crossing the gateway boundary is defined here
//! once, derives `JsonSchema`, and is exported via [`schema_json`] for the
//! `omega-gatewayd schema` CLI command. Route handlers in `routes_pair.rs` and
//! `routes_sessions.rs` import these types rather than redefining them.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct PairRequest {
    pub code: String,
    pub device_name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct PairResponse {
    pub device_id: String,
    pub token: String,
}

#[derive(Serialize, JsonSchema)]
pub struct SessionEntry {
    pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Frame { text: String },
    Error { message: String },
}

#[derive(Serialize, JsonSchema)]
pub struct WhoamiResponse {
    pub device_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatAgent {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMeta {
    pub id: String,
    pub title: Option<String>,
    pub agent: ChatAgent,
    pub cwd: String,
    pub created_at: String,
    pub updated_at: String,
    pub provider_session_id: Option<String>,
    /// The account slot this chat's turns run under, if one was chosen at
    /// creation (else the kind's default account is resolved per turn).
    /// `#[serde(default)]` so a pre-Task-3 `meta.json` without this field
    /// still deserializes.
    #[serde(default)]
    pub account_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
    pub ts: String,
}

/// `GET /v1/chats/{id}/messages` response body — a bounded, newest-first
/// page of a chat's transcript (see `ChatStore::tail_page`), plus the
/// cursor for the next (older) page.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChatMessagesPage {
    /// Newest first.
    pub messages: Vec<ChatMessage>,
    pub next_cursor: Option<u64>,
}

/// `GET /v1/chats/{id}` response body — chat metadata plus the most recent
/// window of messages, chronological (oldest first, matching the pre-Task-B
/// contract). `next_cursor` lets a client page further back into the
/// transcript via `GET /v1/chats/{id}/messages`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChatDetailResponse {
    pub meta: ChatMeta,
    /// Chronological, oldest first.
    pub messages: Vec<ChatMessage>,
    pub next_cursor: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Claude,
    Codex,
}

/// Metadata for one isolated credential slot — METADATA ONLY, never
/// credentials themselves (those live under the slot's hardened directory,
/// out of band from this struct and never serialized here).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    pub slug: String,
    pub label: String,
    pub kind: AccountKind,
    pub created_at: String,
    pub is_default: bool,
}

/// One [`Account`]'s stored metadata plus its LIVE auth status
/// (`"logged_in"` / `"logged_out"` / `"unknown"`), as returned by
/// `GET /v1/accounts`. The status is read fresh from the provider CLI on
/// every list call — never persisted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AccountWithStatus {
    #[serde(flatten)]
    pub account: Account,
    pub status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AccountCreateRequest {
    pub slug: String,
    pub label: String,
    pub kind: AccountKind,
}

/// Body of `POST /v1/accounts/{slug}/apikey` — the headless Codex API-key
/// login. `api_key` is piped straight to `codex login --with-api-key` and is
/// never stored, logged, or echoed back by the gateway.
#[derive(Deserialize, JsonSchema)]
pub struct ApiKeyRequest {
    pub api_key: String,
}

/// Server frames on the `GET /v1/accounts/{slug}/login` WebSocket.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountLoginServerMsg {
    LoginUrl { url: String },
    LoginDone,
    LoginNeedsBox,
    Error { message: String },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamServerMsg {
    Delta { text: String },
    AssistantMessage { text: String },
    ToolEvent {
        name: String,
        detail: Option<String>,
    },
    TurnDone,
    Error { message: String },
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamClientMsg {
    UserMessage { text: String },
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MissionTask {
    pub title: String,
    pub status: String,
}

/// A mirror of one oracle progress ledger
/// (`~/.omega/state/oracle-<key>.progress.json`) — read-only, never written
/// by the gateway. `title` is the first line of the ledger's free-text
/// `mission` field, truncated to 120 chars.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Mission {
    pub key: String,
    pub project: Option<String>,
    pub title: Option<String>,
    pub done: u32,
    pub total: u32,
    pub tasks: Vec<MissionTask>,
    pub updated_at: String,
}

/// Server-pushed events on the `/v1/events` WebSocket (mission updates,
/// alerts, heartbeat). Emitted by [`crate::events::EventHub`].
///
/// KNOWN LIMIT (V2): `Alert` is only ever emitted by an IN-PROCESS caller
/// (e.g. a test, or a future in-process alert source) — there is no
/// external alert ingestion yet (the real alert path is
/// `~/.omega/bin/omega-alert-send.sh`, which has no local success log to
/// tail). Wiring a real external alert source into the hub is a later plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayEvent {
    MissionUpdated { key: String, updated_at: String },
    Alert { message: String, ts: String },
    Heartbeat { ts: String },
}

/// One entry of `GET /v1/rules`'s `laws` array — mirrors
/// `omega_core::rules::Rule` for a Law (`RuleKind::Law`): id, title,
/// category. Laws carry no scope/domain variance worth exposing beyond
/// category (they are universal by invariant).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LawEntry {
    pub id: String,
    pub title: String,
    pub category: String,
}

/// One entry of `GET /v1/rules`'s `rules` array — mirrors
/// `omega_core::rules::Rule` for an operational Rule (`RuleKind::Rule`).
/// `category` is the `RuleCategory` enum's Debug form (`Universal` /
/// `QualityGate` / `Orchestration` / `Reporting` / `Safety`).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RuleEntry {
    pub id: String,
    pub title: String,
    pub category: String,
    pub added_at: String,
}

/// `GET /v1/rules` response body — the OmegaOS doctrine split into the two
/// SSOT tiers (`omega_core::rules::laws()` / `operational_rules()`), never
/// re-derived by filtering `all_rules()` here.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RulesResponse {
    pub laws: Vec<LawEntry>,
    pub rules: Vec<RuleEntry>,
}

/// One entry of `GET /v1/agents`'s `agents` array — mirrors
/// `omega_core::agents::Agent` 1:1: `name`/`display_name`/`available` map
/// directly to `Agent::name()`/`Agent::display_name()`/`Agent::is_available()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentEntry {
    pub name: String,
    pub display_name: String,
    pub available: bool,
}

/// `GET /v1/agents` response body — the fixed dispatch-target roster from
/// `omega_core::agents::Agent::all()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentsResponse {
    pub agents: Vec<AgentEntry>,
}

/// One entry of `GET /v1/skills`'s `skills` array — mirrors
/// `omega_core::skill_registry::Skill` for display: `category` is
/// `skill.category.label()` (e.g. `"Audit"`/`"Design"`), not a raw Debug
/// form, since the label method exists precisely for this.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub category: String,
}

/// `GET /v1/skills` response body — the OmegaOS skill catalog from
/// `omega_core::skill_registry::SkillRegistry::discover_default()`, optionally
/// narrowed by `?q=`/`?limit=`. `total` is always the UNFILTERED catalog
/// size, so the caller can show "showing N of total" even when `skills` is
/// narrowed.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillsResponse {
    pub skills: Vec<SkillEntry>,
    pub total: usize,
}

/// One entry of `GET /v1/projects`'s `projects` array — a field-for-field
/// mirror of `omega_core::projects::DiscoveredProject`, minus `path` (a full
/// filesystem path is server-internal, not something the wire protocol
/// should leak to a mobile client — the same posture `Account` already
/// takes by never serializing credentials) and minus `score` (an internal
/// ranking heuristic, not product-facing; the response is already
/// best-first sorted, which is all the app needs).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectEntry {
    pub name: String,
    pub container: String,
    pub stack: Vec<String>,
    pub last_active_days: Option<u64>,
}

/// `GET /v1/projects` response body — the auto-discovered project list from
/// `omega_core::projects::discover`, best-first sorted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectsResponse {
    pub projects: Vec<ProjectEntry>,
}

/// One entry of `GET /v1/oracles`'s `oracles` array — a top-level mission
/// ledger ([`Mission`]) composed with its live rmux session status.
///
/// NAMING NOTE: `key` and `session` carry the SAME string. `Mission.key` is
/// populated verbatim from the ledger JSON's `"oracle"` field (see
/// `missions.rs`), and that field already IS the full session name (e.g.
/// `"oracle-dentistrygpt"`) — there is no separate bare identifier anywhere
/// in the ledger-parsing code to derive a shorter `key` from. `key` names
/// the entry's identity, `session` names its role as the exact string
/// checked against `rmux::list_sessions()` for `live`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OracleEntry {
    pub key: String,
    pub session: String,
    pub live: bool,
    pub mission: Option<Mission>,
}

/// `GET /v1/oracles` response body — the live oracle roster: every
/// top-level mission ledger from `missions::list()`, each annotated with
/// whether its session currently appears in `rmux::list_sessions()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OraclesResponse {
    pub oracles: Vec<OracleEntry>,
}

/// Body of `POST /v1/dispatch` — mirrors `omega dispatch <PROJECT> <MISSION>
/// [--agent ...] [--new]` 1:1. `agent` and `new` are the exact `Option`s the
/// CLI's own clap flags model (`--agent` is `Option<String>`, `--new` is a
/// bare flag whose ABSENCE means "let the followup-or-spawn router decide",
/// not "force a spawn" — so this is `Option<bool>`, not `bool`, and
/// `routes_dispatch::create` only appends `--new` when it is exactly
/// `Some(true)`).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DispatchRequest {
    pub project: String,
    pub mission: String,
    pub agent: Option<String>,
    pub new: Option<bool>,
}

/// `POST /v1/dispatch` response body — `oracle` is the session name parsed
/// off `omega dispatch`'s stdout line 0 (`"◆ Oracle dispatched: <name>"`,
/// see `omega_core::dispatch::DispatchOutcome::report_lines`), `delivery` is
/// the `DispatchDelivery::tag()` value from the `DISPATCH_DELIVERY=<tag>`
/// line — passed through verbatim as a `String` rather than re-modeled as a
/// gateway-side enum, since the CLI already owns that vocabulary.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DispatchResponse {
    pub oracle: String,
    pub delivery: String,
}

/// Body of `POST /v1/sessions/{name}/keys` — sends keystrokes to a live
/// rmux session. `data` is sent literally (`rmux send-keys -l`); when
/// `enter` is true a separate `rmux send-keys Enter` call follows.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendKeysRequest {
    pub data: String,
    #[serde(default)]
    pub enter: bool,
}

/// `POST /v1/sessions/{name}/keys` response body.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SendKeysResponse {
    pub ok: bool,
}

/// `POST /v1/sessions/{name}/close` response body — the classification +
/// outcome of running `omega kill <name>` (never `--force`: a REFUSED
/// live-workers case must surface to the app so it can render a strong
/// confirm, never be silently forced by the gateway). `killed` is `true`
/// only on `omega kill`'s own 2xx-equivalent (zero) exit — a REFUSED kill
/// (non-zero exit) is a NORMAL, expected outcome the app needs to render,
/// so the endpoint still answers HTTP 200 with `killed: false` rather than
/// a 502. `message` carries the raw operator-facing text `omega kill`
/// produced (preferring stdout, falling back to stderr — see
/// `routes_sessions::close`'s doc comment for why: the real CLI's REFUSED
/// bail lands on stderr, not stdout, contrary to a naive first read of its
/// call site).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CloseSessionResponse {
    pub killed: bool,
    pub already_closed: bool,
    pub is_oracle: bool,
    pub cascaded_count: u32,
    pub message: String,
}

/// Body of `POST /v1/sessions/{name}/rename`. `new_name` is validated as a
/// safe slug (see `routes_sessions::valid_new_session_name`) BEFORE it ever
/// reaches rmux — rmux silently REWRITES `:`/`.` to `_` rather than
/// rejecting them, and the caller expects `new_name` back verbatim.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RenameSessionRequest {
    pub new_name: String,
}

/// `POST /v1/sessions/{name}/rename` response body — `name` is the
/// already-validated `new_name`, i.e. the session's new effective name.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RenameSessionResponse {
    pub name: String,
}

/// One session's client-organizational overlay: a folder, a custom label,
/// and a pinned flag. Purely metadata the gateway persists so it survives
/// app restarts and syncs across the operator's devices — it is NEVER
/// validated against a currently-live rmux session, the overlay can freely
/// reference a session that has since closed (see `session_org.rs`'s doc
/// comment). `pinned` is the only field that isn't optional: an entry with
/// no label and no folder is still meaningful once pinned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionOrgEntry {
    pub label: Option<String>,
    pub folder: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

/// `GET /v1/session-org` response body — the whole overlay map, session
/// name -> [`SessionOrgEntry`]. Empty when no session has ever been tagged.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SessionOrgResponse {
    pub entries: std::collections::HashMap<String, SessionOrgEntry>,
}

/// Body of `PUT /v1/session-org/{name}`. Standard HTTP PUT semantics: a
/// FULL REPLACE of that session's overlay entry, never a partial merge with
/// whatever was persisted before — an omitted `label`/`folder` becomes
/// `None`, an omitted `pinned` becomes `false`, exactly as if the entry were
/// newly created. `label`/`folder` are length-capped server-side (see
/// `routes_session_org::MAX_LABEL_LEN` / `MAX_FOLDER_LEN`) before ever
/// reaching the store.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SessionOrgUpdateRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub pinned: Option<bool>,
}

/// `POST /v1/deposit` response body — the HTTP twin of the Telegram DEPOSIT
/// bot's reply. `file` is the final timestamped filename written to the
/// inbox, `boxes` lists the named boxes the deposit actually reached (empty
/// when `held` is true), `held` is true when the upload looked like a
/// credential (see `deposit::looks_secret`) and was kept in the inbox only.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DepositResponse {
    pub file: String,
    pub boxes: Vec<String>,
    pub held: bool,
}

/// Umbrella type so one schema document carries every wire type.
/// Only JsonSchema is needed: this type is never serialized itself.
#[derive(JsonSchema)]
pub struct Protocol {
    pub pair_request: PairRequest,
    pub pair_response: PairResponse,
    pub sessions_response: SessionsResponse,
    pub stream_frame: StreamFrame,
    pub whoami_response: WhoamiResponse,
    pub chat_meta: ChatMeta,
    pub chat_message: ChatMessage,
    pub chat_messages_page: ChatMessagesPage,
    pub chat_detail_response: ChatDetailResponse,
    pub chat_stream_server_msg: ChatStreamServerMsg,
    pub chat_stream_client_msg: ChatStreamClientMsg,
    pub mission: Mission,
    pub mission_task: MissionTask,
    pub gateway_event: GatewayEvent,
    pub account: Account,
    pub account_kind: AccountKind,
    pub account_with_status: AccountWithStatus,
    pub account_create_request: AccountCreateRequest,
    pub api_key_request: ApiKeyRequest,
    pub account_login_server_msg: AccountLoginServerMsg,
    pub law_entry: LawEntry,
    pub rule_entry: RuleEntry,
    pub rules_response: RulesResponse,
    pub agent_entry: AgentEntry,
    pub agents_response: AgentsResponse,
    pub skill_entry: SkillEntry,
    pub skills_response: SkillsResponse,
    pub project_entry: ProjectEntry,
    pub projects_response: ProjectsResponse,
    pub oracle_entry: OracleEntry,
    pub oracles_response: OraclesResponse,
    pub dispatch_request: DispatchRequest,
    pub dispatch_response: DispatchResponse,
    pub send_keys_request: SendKeysRequest,
    pub send_keys_response: SendKeysResponse,
    pub close_session_response: CloseSessionResponse,
    pub rename_session_request: RenameSessionRequest,
    pub rename_session_response: RenameSessionResponse,
    pub deposit_response: DepositResponse,
    pub session_org_entry: SessionOrgEntry,
    pub session_org_response: SessionOrgResponse,
    pub session_org_update_request: SessionOrgUpdateRequest,
}

pub fn schema_json() -> String {
    let schema = schema_for!(Protocol);
    serde_json::to_string_pretty(&schema).expect("serialize schema")
}
