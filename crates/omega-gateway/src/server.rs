use crate::accounts::AccountStore;
use crate::auth::{Device, DeviceStore};
use crate::chat_store::ChatStore;
use crate::config::GatewayConfig;
use crate::events::EventHub;
use crate::protocol::WhoamiResponse;
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
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Global cap on concurrently-running chat turns (across all devices/chats).
const MAX_CONCURRENT_CHAT_TURNS: usize = 8;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
    pub cfg: GatewayConfig,
    pub chats: Arc<ChatStore>,
    /// Isolated per-account Claude/Codex credential slots. Stateless
    /// (file-backed), so `AccountStore` is cheaply `Clone` itself rather
    /// than `Arc`-wrapped.
    pub accounts: AccountStore,
    pub chat_permits: Arc<Semaphore>,
    /// Event bus for `/v1/events` (mission updates, alerts, heartbeat).
    /// Cloning `AppState` shares this hub, so a test (or a future
    /// in-process alert source) can hold its own clone and call
    /// `emit(...)` while the router forwards from the same bus.
    pub events: EventHub,
}

impl AppState {
    /// Builds the full app state, opening the chat + account stores rooted
    /// at `dir` and sizing the global chat-turn semaphore to
    /// [`MAX_CONCURRENT_CHAT_TURNS`].
    pub fn new(dir: PathBuf, cfg: GatewayConfig) -> Self {
        let chats = Arc::new(ChatStore::open(&dir));
        let accounts = AccountStore::open(&dir);
        let chat_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_CHAT_TURNS));
        let events = EventHub::new();
        Self { dir, cfg, chats, accounts, chat_permits, events }
    }
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
    // Query tokens exist for WebSocket clients that cannot set headers.
    // Request logging must therefore never log full request URIs.
    let token = header_token.or_else(|| query.get("token").cloned());
    let Some(token) = token else { return Err(StatusCode::UNAUTHORIZED) };
    let Some(device) = DeviceStore::open(&state.dir).verify(&token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    req.extensions_mut().insert(device);
    Ok(next.run(req).await)
}

async fn whoami(Extension(device): Extension<Device>) -> Json<WhoamiResponse> {
    Json(WhoamiResponse { device_id: device.id, name: device.name })
}

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/whoami", get(whoami))
        .route("/v1/sessions", get(crate::routes_sessions::list))
        .route("/v1/sessions/{name}/stream", get(crate::routes_sessions::stream))
        .route(
            "/v1/chats",
            get(crate::routes_chat::list).post(crate::routes_chat::create),
        )
        .route("/v1/chats/{id}", get(crate::routes_chat::get))
        .route("/v1/chats/{id}/stream", get(crate::routes_chat::stream))
        .route("/v1/missions", get(crate::routes_missions::list))
        .route("/v1/rules", get(crate::routes_rules::list))
        .route("/v1/events", get(crate::routes_events::events))
        .route(
            "/v1/accounts",
            get(crate::routes_accounts::list).post(crate::routes_accounts::create),
        )
        .route(
            "/v1/accounts/{slug}",
            axum::routing::delete(crate::routes_accounts::delete),
        )
        .route(
            "/v1/accounts/{slug}/default",
            axum::routing::post(crate::routes_accounts::set_default),
        )
        .route("/v1/accounts/{slug}/login", get(crate::routes_accounts::login))
        .route(
            "/v1/accounts/{slug}/apikey",
            axum::routing::post(crate::routes_accounts::apikey),
        )
        // IMPORTANT: route_layer only wraps routes registered BEFORE it is
        // called. Add every new protected .route(...) ABOVE this line, or it
        // ships unauthenticated.
        .route_layer(middleware::from_fn_with_state(state.clone(), require_device));
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/pair", axum::routing::post(crate::routes_pair::pair))
        .merge(protected)
        .with_state(state)
}
