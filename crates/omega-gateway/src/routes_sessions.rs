use crate::protocol::{SessionEntry, SessionsResponse, StreamFrame};
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
    Path, State,
};
use axum::response::Response;

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| stream_loop(socket, name, state))
}

async fn stream_loop(mut socket: WebSocket, name: String, state: AppState) {
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
        let captured =
            tokio::task::spawn_blocking(move || crate::rmux::capture_pane(&session, lines)).await;
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
