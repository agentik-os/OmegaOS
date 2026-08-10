use crate::protocol::{
    SendKeysRequest, SendKeysResponse, SessionEntry, SessionsResponse, StreamFrame,
};
use axum::Json;

pub async fn list() -> Json<SessionsResponse> {
    match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => Json(SessionsResponse {
            sessions: names.into_iter().map(|name| SessionEntry { name }).collect(),
            error: None,
        }),
        Ok(Err(e)) => Json(SessionsResponse { sessions: vec![], error: Some(e.to_string()) }),
        Err(e) => Json(SessionsResponse { sessions: vec![], error: Some(e.to_string()) }),
    }
}

use crate::server::AppState;
use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    Path, Query, State,
};
use axum::http::StatusCode;
use axum::response::Response;
use std::collections::HashMap;

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let color = query.get("color").map(|v| v == "1").unwrap_or(false);
    ws.on_upgrade(move |socket| stream_loop(socket, name, state, color))
}

async fn stream_loop(mut socket: WebSocket, name: String, state: AppState, color: bool) {
    let interval = std::time::Duration::from_millis(state.cfg.stream_interval_ms);
    let lines = state.cfg.stream_lines;
    let mut last: Option<String> = None;
    // R-STREAM: this loop never exits on error; errors are rendered as frames.
    // KNOWN LIMIT (V1): the only exit is a failed send, which fires instantly
    // on a clean client close but only after the kernel TCP timeout on a
    // silent network death. Plan 2 hardening adds ping/pong liveness.
    // KNOWN LIMIT (V1): revoking a device does not terminate an already-open
    // stream — the socket keeps running on the token that was valid at
    // upgrade time. Re-verification on a live connection lands with the
    // Plan 2 ping/pong liveness pass above.
    // KNOWN LIMIT (V1): no pairing or stream rate limiting yet; that is
    // Plan 2 scope too.
    loop {
        let session = name.clone();
        let captured = tokio::task::spawn_blocking(move || {
            if color {
                crate::rmux::capture_pane_ansi(&session, lines)
            } else {
                crate::rmux::capture_pane(&session, lines)
            }
        })
        .await;
        let frame = match captured {
            Ok(Ok(text)) => {
                if last.as_deref() == Some(text.as_str()) {
                    None
                } else {
                    last = Some(text.clone());
                    Some(StreamFrame::Frame { text })
                }
            }
            Ok(Err(e)) => Some(StreamFrame::Error { message: e.to_string() }),
            Err(e) => Some(StreamFrame::Error { message: e.to_string() }),
        };
        if let Some(frame) = frame {
            let text = serde_json::to_string(&frame).expect("serialize frame");
            if socket.send(Message::Text(text.into())).await.is_err() {
                return; // client went away: the ONLY exit
            }
        }
        tokio::time::sleep(interval).await;
    }
}

/// Maximum accepted `data` payload for `POST /v1/sessions/{name}/keys`, in
/// bytes. A safety valve, not a hard product requirement: rmux `send-keys`
/// has no documented limit, but an unbounded body lets one request tie up
/// the subprocess for an unreasonable time and is a trivial DoS/typo vector
/// (a client accidentally posting a whole file). 8 KiB comfortably covers
/// any real interactive keystroke burst (a pasted command, a multi-line
/// snippet) while staying far below anything that would matter for argv
/// size or subprocess latency.
const MAX_SEND_KEYS_BYTES: usize = 8192;

/// Session names this endpoint will act on: mirrors `routes_chat.rs`'s
/// `valid_chat_id` shape — reject anything that could path-traverse or
/// shell-inject BEFORE it ever reaches a subprocess argv. rmux session names
/// in practice are `oracle-<Project>-<n>`-shaped or similar identifiers, so a
/// conservative charset (letters/digits/`_`/`-`/`.`) with a generous length
/// cap covers every real name without accepting `/`, `..`, or NUL.
fn valid_session_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 200 {
        return false;
    }
    if name.contains('/') || name.contains("..") || name.contains('\0') {
        return false;
    }
    name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

pub async fn send_keys(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SendKeysRequest>,
) -> Result<Json<SendKeysResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validate BEFORE touching any subprocess, same discipline
    // routes_dispatch.rs uses for `project`.
    if !valid_session_name(&name) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid session name" })),
        ));
    }
    if req.data.len() > MAX_SEND_KEYS_BYTES {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "data too large" })),
        ));
    }

    let session = name.clone();
    let data = req.data.clone();
    tokio::task::spawn_blocking(move || crate::rmux::send_keys_literal(&session, &data))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("send_keys task panicked: {e}") })),
            )
        })?
        .map_err(|e| {
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))
        })?;

    if req.enter {
        let session = name.clone();
        tokio::task::spawn_blocking(move || crate::rmux::send_enter(&session))
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": format!("send_enter task panicked: {e}") })),
                )
            })?
            .map_err(|e| {
                (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))
            })?;
    }

    Ok(Json(SendKeysResponse { ok: true }))
}

#[cfg(test)]
mod valid_session_name_tests {
    use super::valid_session_name;

    #[test]
    fn accepts_a_real_session_name() {
        assert!(valid_session_name("oracle-Foo-1"));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(!valid_session_name("../etc"));
    }

    #[test]
    fn rejects_slash() {
        assert!(!valid_session_name("foo/bar"));
    }

    #[test]
    fn rejects_empty() {
        assert!(!valid_session_name(""));
    }

    #[test]
    fn rejects_very_long_name() {
        let long = "a".repeat(201);
        assert!(!valid_session_name(&long));
    }
}
