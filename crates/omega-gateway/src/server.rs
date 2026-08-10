use axum::{routing::get, Json, Router};
use serde_json::json;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .with_state(state)
}
