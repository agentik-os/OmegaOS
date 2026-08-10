use crate::auth::{Device, DeviceStore};
use crate::config::GatewayConfig;
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Json, Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
    pub cfg: GatewayConfig,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

pub async fn require_device(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let header_token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    let token = header_token.or_else(|| query.get("token").cloned());
    let Some(token) = token else { return Err(StatusCode::UNAUTHORIZED) };
    let Some(device) = DeviceStore::open(&state.dir).verify(&token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    req.extensions_mut().insert(device);
    Ok(next.run(req).await)
}

async fn whoami(Extension(device): Extension<Device>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "device_id": device.id, "name": device.name }))
}

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/whoami", get(whoami))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_device));
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/pair", axum::routing::post(crate::routes_pair::pair))
        .merge(protected)
        .with_state(state)
}
