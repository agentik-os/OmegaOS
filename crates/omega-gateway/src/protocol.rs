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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
    pub ts: String,
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
    pub chat_stream_server_msg: ChatStreamServerMsg,
    pub chat_stream_client_msg: ChatStreamClientMsg,
    pub mission: Mission,
    pub mission_task: MissionTask,
}

pub fn schema_json() -> String {
    let schema = schema_for!(Protocol);
    serde_json::to_string_pretty(&schema).expect("serialize schema")
}
