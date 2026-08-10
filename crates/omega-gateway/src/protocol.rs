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

/// Umbrella type so one schema document carries every wire type.
/// Only JsonSchema is needed: this type is never serialized itself.
#[derive(JsonSchema)]
pub struct Protocol {
    pub pair_request: PairRequest,
    pub pair_response: PairResponse,
    pub sessions_response: SessionsResponse,
    pub stream_frame: StreamFrame,
    pub whoami_response: WhoamiResponse,
}

pub fn schema_json() -> String {
    let schema = schema_for!(Protocol);
    serde_json::to_string_pretty(&schema).expect("serialize schema")
}
