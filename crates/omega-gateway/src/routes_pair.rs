use crate::auth::{DeviceStore, PairingCode};
use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct PairRequest {
    pub code: String,
    pub device_name: String,
}

pub async fn pair(
    State(state): State<AppState>,
    Json(req): Json<PairRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !PairingCode::consume(&state.dir, &req.code) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "invalid or expired code" })));
    }
    let mut store = DeviceStore::open(&state.dir);
    let (device, token) = store.issue(&req.device_name);
    (StatusCode::OK, Json(json!({ "device_id": device.id, "token": token })))
}
