//! Chat REST + WebSocket routes: create/list/inspect chats, and stream real
//! agent turns over a persistent WebSocket (R-STREAM discipline — a parse or
//! agent error becomes an `Error` frame, never a closed socket; only a dead
//! socket or an explicit client close ends the loop).

use crate::chat_driver::run_turn;
use crate::protocol::{ChatAgent, ChatMessage, ChatMeta, ChatStreamClientMsg, ChatStreamServerMsg};
use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path as FsPath, PathBuf};

#[derive(Deserialize)]
pub struct ChatCreateRequest {
    pub agent: ChatAgent,
    pub cwd: String,
    #[serde(default)]
    pub title: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "chats": state.chats.list() }))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<ChatCreateRequest>,
) -> (StatusCode, Json<ChatMeta>) {
    let meta = state.chats.create(req.agent, req.cwd, req.title);
    (StatusCode::CREATED, Json(meta))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let meta = state.chats.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    let messages = state.chats.transcript(&id);
    Ok(Json(json!({ "meta": meta, "messages": messages })))
}

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| stream_loop(socket, id, state))
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// `<gateway_dir>/accounts/default` when present, else `None` (the box's
/// ambient claude config is used). Kept intentionally simple for V2.
fn resolve_account_dir(gateway_dir: &FsPath) -> Option<PathBuf> {
    let dir = gateway_dir.join("accounts").join("default");
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// Serializes and sends one server frame. `Err` means the socket is dead.
async fn send_frame(socket: &mut WebSocket, frame: &ChatStreamServerMsg) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize ChatStreamServerMsg");
    socket.send(Message::Text(text.into())).await
}

/// Sends `Error{message}` then `TurnDone`, the "can't start a turn" pair used
/// by both the unknown-chat and busy-semaphore short-circuits. Returns
/// `Err(())` if the socket died mid-send, so the caller can stop the loop.
async fn send_error_turn_done(socket: &mut WebSocket, message: impl Into<String>) -> Result<(), ()> {
    send_frame(socket, &ChatStreamServerMsg::Error { message: message.into() })
        .await
        .map_err(|_| ())?;
    send_frame(socket, &ChatStreamServerMsg::TurnDone).await.map_err(|_| ())
}

async fn stream_loop(mut socket: WebSocket, id: String, state: AppState) {
    // R-STREAM: this loop never exits on error; only a dead socket or an
    // explicit client close ends it. The socket supports multiple turns.
    loop {
        let text = match socket.recv().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            Some(Ok(Message::Close(_))) | None => return, // client closed or gone
            Some(Ok(_)) => continue,                       // ping/pong/binary: not a turn
            Some(Err(_)) => return,                        // socket error: dead
        };

        let client_msg: ChatStreamClientMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                if send_error_turn_done(&mut socket, format!("bad client message: {e}")).await.is_err() {
                    return;
                }
                continue;
            }
        };
        let ChatStreamClientMsg::UserMessage { text: user_text } = client_msg;

        let Some(meta) = state.chats.get(&id) else {
            if send_error_turn_done(&mut socket, "chat not found").await.is_err() {
                return;
            }
            continue;
        };

        state.chats.append_message(
            &id,
            &ChatMessage { role: "user".to_string(), text: user_text.clone(), ts: now() },
        );

        let Ok(permit) = state.chat_permits.clone().try_acquire_owned() else {
            if send_error_turn_done(&mut socket, "busy, too many active chats").await.is_err() {
                return;
            }
            continue;
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<ChatStreamServerMsg>(64);
        let timeout = std::time::Duration::from_millis(state.cfg.chat_turn_timeout_ms);
        let account_dir = resolve_account_dir(&state.dir);
        let turn_meta = meta.clone();
        let turn_handle = tokio::spawn(async move {
            let _permit = permit; // held for the whole turn, released on drop
            run_turn(&turn_meta, &user_text, None, account_dir.as_deref(), timeout, tx).await
        });

        let mut assistant_text = String::new();
        let mut turn_done_seen = false;
        let mut socket_dead = false;
        while let Some(frame) = rx.recv().await {
            // Defensive dedupe (hardening from the Task 3 re-review): run_turn
            // may emit a trailing TurnDone after the stream's own. The client
            // must see exactly one TurnDone per turn, and nothing after the
            // first is persisted or forwarded.
            if turn_done_seen {
                continue;
            }
            match &frame {
                ChatStreamServerMsg::Delta { text } | ChatStreamServerMsg::AssistantMessage { text } => {
                    assistant_text.push_str(text);
                }
                ChatStreamServerMsg::TurnDone => turn_done_seen = true,
                ChatStreamServerMsg::ToolEvent { .. } | ChatStreamServerMsg::Error { .. } => {}
            }
            if !socket_dead && send_frame(&mut socket, &frame).await.is_err() {
                socket_dead = true;
            }
        }

        let provider_session_id = turn_handle.await.ok().flatten();

        if !assistant_text.is_empty() {
            state.chats.append_message(
                &id,
                &ChatMessage { role: "assistant".to_string(), text: assistant_text, ts: now() },
            );
        }
        if let Some(sid) = provider_session_id {
            state.chats.set_provider_session(&id, &sid);
        }

        if socket_dead {
            return;
        }
    }
}
