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

/// Global cap on concurrently-running `POST /v1/dispatch` requests. A
/// dispatch spawns a whole oracle session (much heavier than a single chat
/// turn), so this stays well below [`MAX_CONCURRENT_CHAT_TURNS`].
const MAX_CONCURRENT_DISPATCHES: usize = 4;

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
    /// Caps concurrently-running `POST /v1/dispatch` requests, mirroring
    /// `chat_permits` — see [`MAX_CONCURRENT_DISPATCHES`].
    pub dispatch_permits: Arc<Semaphore>,
    /// Event bus for `/v1/events` (mission updates, alerts, heartbeat).
    /// Cloning `AppState` shares this hub, so a test (or a future
    /// in-process alert source) can hold its own clone and call
    /// `emit(...)` while the router forwards from the same bus.
    pub events: EventHub,
}

impl AppState {
    /// Builds the full app state, opening the chat + account stores rooted
    /// at `dir` and sizing the global chat-turn semaphore to
    /// [`MAX_CONCURRENT_CHAT_TURNS`] and the dispatch semaphore to
    /// [`MAX_CONCURRENT_DISPATCHES`].
    pub fn new(dir: PathBuf, cfg: GatewayConfig) -> Self {
        let chats = Arc::new(ChatStore::open(&dir));
        let accounts = AccountStore::open(&dir);
        let chat_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_CHAT_TURNS));
        let dispatch_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_DISPATCHES));
        let events = EventHub::new();
        Self { dir, cfg, chats, accounts, chat_permits, dispatch_permits, events }
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
            "/v1/sessions/{name}/keys",
            axum::routing::post(crate::routes_sessions::send_keys),
        )
        .route(
            "/v1/sessions/{name}/close",
            axum::routing::post(crate::routes_sessions::close),
        )
        .route(
            "/v1/sessions/{name}/rename",
            axum::routing::post(crate::routes_sessions::rename),
        )
        .route(
            "/v1/chats",
            get(crate::routes_chat::list).post(crate::routes_chat::create),
        )
        .route("/v1/chats/{id}", get(crate::routes_chat::get))
        .route("/v1/chats/{id}/messages", get(crate::routes_chat::messages))
        .route("/v1/chats/{id}/stream", get(crate::routes_chat::stream))
        .route("/v1/missions", get(crate::routes_missions::list))
        .route("/v1/oracles", get(crate::routes_oracles::list))
        .route("/v1/rules", get(crate::routes_rules::list))
        .route("/v1/agents", get(crate::routes_agents::list))
        .route("/v1/skills", get(crate::routes_skills::list))
        .route("/v1/projects", get(crate::routes_projects::list))
        .route("/v1/dispatch", axum::routing::post(crate::routes_dispatch::create))
        // B1 fix: axum 0.8's `Multipart` extractor falls back to its own
        // internal 2 MiB default body limit when no `DefaultBodyLimit` layer
        // is set, silently overriding `MAX_DEPOSIT_BYTES` (the crate's own
        // 50 MiB cap in routes_deposit.rs) for any upload between 2 MiB and
        // 50 MiB. Scoped to THIS route only (via `.layer()` on the
        // `MethodRouter`, not the outer `Router`) so no other route's body
        // limit changes. The +8192 margin covers multipart boundary/header
        // overhead so a file at exactly MAX_DEPOSIT_BYTES still clears this
        // layer and reaches the crate's own size check, which is the one
        // that must make the final call.
        .route(
            "/v1/deposit",
            axum::routing::post(crate::routes_deposit::create).layer(
                axum::extract::DefaultBodyLimit::max(crate::routes_deposit::MAX_DEPOSIT_BYTES + 8192),
            ),
        )
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
