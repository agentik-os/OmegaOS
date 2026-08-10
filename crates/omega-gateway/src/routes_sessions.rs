use crate::protocol::{
    CloseSessionResponse, RenameSessionRequest, RenameSessionResponse, SendKeysRequest,
    SendKeysResponse, SessionEntry, SessionsResponse, StreamFrame,
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

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg.into() })))
}

/// Parses `cmd_kill`'s alias-resolution line (`"[i] {name} resolved to the
/// oracle session {resolved}"`, printed to **stdout** before anything else
/// when the caller-supplied name needed `resolve_oracle_alias` — see
/// `cmd_kill` in `crates/omega-cli/src/main.rs`) and returns the resolved
/// name it names, or `None` when no such line is present (the common case:
/// no resolution happened). Pure/testable, mirroring `routes_dispatch.rs`'s
/// `parse_dispatch_stdout` style.
fn resolved_oracle_name(stdout: &str) -> Option<&str> {
    let line = stdout.lines().find(|l| l.starts_with("[i] "))?;
    let resolved = line.split(" resolved to the oracle session ").nth(1)?;
    let resolved = resolved.trim();
    if resolved.is_empty() {
        None
    } else {
        Some(resolved)
    }
}

/// `POST /v1/sessions/{name}/close` — runs `omega kill <name>` (never
/// `--force`, so the REFUSED-because-live-workers case surfaces to the app
/// explicitly rather than the endpoint silently forcing it away).
///
/// ## Output-contract deviation from a first read of `cmd_kill` (R-VERIFY /
/// L1 — verified against real runtime behavior, not assumed): `cmd_kill`'s
/// REFUSED path is an `anyhow::bail!`, and `omega-cli`'s `main()` is
/// `#[tokio::main] async fn main() -> Result<()>` — Rust's default
/// `Termination` impl for a failing `Result<(), E: Debug>` prints
/// `"Error: {err:?}"` to **stderr** (confirmed with a throwaway
/// `anyhow::bail!` + `#[tokio::main] async fn main() -> anyhow::Result<()>`
/// reproduction: stdout was empty, stderr was exactly `"Error: kill
/// REFUSED — ...\n"`), not stdout. BUT `cmd_kill` may ALSO print an
/// alias-resolution line to stdout BEFORE hitting the REFUSED bail (when the
/// caller-supplied name needed `resolve_oracle_alias`), so stdout emptiness
/// is not a reliable signal of success — `message` switches on
/// `output.success` instead: on success it is `stdout` alone (where the
/// success/already-closed/cascaded-worker lines all live), on failure it is
/// `stdout` + `stderr` concatenated so a resolution line is never dropped
/// alongside the REFUSED text.
///
/// `cascaded_count` / `already_closed` still parse `stdout` only, per the
/// brief: both of those lines are real `println!`s in the non-REFUSED
/// paths, so they are unaffected by this success/failure split.
///
/// `is_oracle` is classified off the name `cmd_kill` actually resolved and
/// acted on — parsed via [`resolved_oracle_name`] — not the raw path param,
/// since an alias-shaped input (e.g. `"Verba-3"`) falls through
/// `OmegaSession::classify` to `SessionRole::Home` while the real resolved
/// session (`"oracle-Verba-3"`) is an oracle.
pub async fn close(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CloseSessionResponse>, ApiError> {
    // Validate BEFORE touching any subprocess, same discipline as send_keys.
    if !valid_session_name(&name) {
        return Err(bad_request("invalid session name"));
    }

    let session = name.clone();
    let output = tokio::task::spawn_blocking(move || crate::omega_cli::run(&["kill", &session]))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("kill task panicked: {e}") })),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("failed to spawn omega: {e}") })),
            )
        })?;

    // Classify off the resolved name when `cmd_kill` reported one, else the
    // raw caller-supplied name — never parsed off the CLI otherwise.
    let classify_name = resolved_oracle_name(&output.stdout).unwrap_or(&name);
    let is_oracle = omega_core::session::OmegaSession::classify(classify_name).role
        == omega_core::session::SessionRole::Oracle;

    let cascaded_count =
        output.stdout.lines().filter(|l| l.starts_with("  cascaded worker")).count() as u32;
    let already_closed = output.stdout.contains("is already closed — nothing live to kill.");
    let message = if output.success {
        output.stdout.clone()
    } else {
        format!("{}{}", output.stdout, output.stderr)
    };

    Ok(Json(CloseSessionResponse {
        killed: output.success,
        already_closed,
        is_oracle,
        cascaded_count,
        message,
    }))
}

/// Safe-slug charset for a `rename`'s *new* name — a deliberately TIGHTER,
/// FRESH check than [`valid_session_name`] (never weakened to admit `.`,
/// which rmux is documented to silently REWRITE to `_` rather than reject —
/// see `rmux.rs`'s own quoting-discipline doc comments). ASCII alnum + `_` +
/// `-` only: no `.`, `/`, `:`, whitespace, or NUL. A chosen new name is
/// short, so the length cap is intentionally tighter (100) than
/// `valid_session_name`'s 200.
///
/// Also rejects a name whose FIRST byte is `-` or `.`: live-verified against
/// the real `rmux` binary (a `--` argv separator does not help — this is
/// daemon-side trimming, not client argv parsing), `rmux rename-session -t
/// <session> -q` actually renames the session to `q`, silently dropping the
/// leading `-`. A leading `.` is rejected for the same reason this function
/// already bans `.` everywhere else in the name. Left unchecked, the
/// response's echoed `name` would diverge from the real session name.
fn valid_new_session_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 100 {
        return false;
    }
    if name.starts_with('-') || name.starts_with('.') {
        return false;
    }
    name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// `POST /v1/sessions/{name}/rename` — runs
/// `rmux rename-session -t <name> <new_name>`. Both the path `{name}` and
/// the body's `new_name` are validated BEFORE any subprocess spawn:
/// `new_name` especially, since the caller expects it back verbatim and
/// rmux would otherwise silently rewrite an unsafe character rather than
/// reject it.
pub async fn rename(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<RenameSessionRequest>,
) -> Result<Json<RenameSessionResponse>, ApiError> {
    if !valid_session_name(&name) {
        return Err(bad_request("invalid session name"));
    }
    if !valid_new_session_name(&req.new_name) {
        return Err(bad_request("invalid new session name"));
    }

    let old = name.clone();
    let new = req.new_name.clone();
    tokio::task::spawn_blocking(move || crate::rmux::rename_session(&old, &new))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("rename task panicked: {e}") })),
            )
        })?
        .map_err(|e| {
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))
        })?;

    Ok(Json(RenameSessionResponse { name: req.new_name }))
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

#[cfg(test)]
mod resolved_oracle_name_tests {
    use super::resolved_oracle_name;

    #[test]
    fn extracts_the_resolved_name_from_the_exact_cmd_kill_line() {
        let stdout = "[i] Verba-3 resolved to the oracle session oracle-Verba-3\nKilled session: oracle-Verba-3\n";
        assert_eq!(resolved_oracle_name(stdout), Some("oracle-Verba-3"));
    }

    #[test]
    fn returns_none_when_no_resolution_line_present() {
        let stdout = "Killed session: worker-Foo-1\n";
        assert_eq!(resolved_oracle_name(stdout), None);
    }

    #[test]
    fn returns_none_on_empty_stdout() {
        assert_eq!(resolved_oracle_name(""), None);
    }
}

#[cfg(test)]
mod valid_new_session_name_tests {
    use super::valid_new_session_name;

    #[test]
    fn accepts_a_plain_slug() {
        assert!(valid_new_session_name("my-renamed-session_1"));
    }

    #[test]
    fn accepts_a_name_with_an_internal_dash() {
        assert!(valid_new_session_name("my-session"));
    }

    #[test]
    fn rejects_leading_dash() {
        assert!(!valid_new_session_name("-q"));
    }

    #[test]
    fn rejects_leading_dot() {
        assert!(!valid_new_session_name(".hidden"));
    }

    #[test]
    fn rejects_dot() {
        assert!(!valid_new_session_name("foo.bar"));
    }

    #[test]
    fn rejects_colon() {
        assert!(!valid_new_session_name("foo:bar"));
    }

    #[test]
    fn rejects_slash() {
        assert!(!valid_new_session_name("foo/bar"));
    }

    #[test]
    fn rejects_whitespace() {
        assert!(!valid_new_session_name("foo bar"));
    }

    #[test]
    fn rejects_empty() {
        assert!(!valid_new_session_name(""));
    }

    #[test]
    fn rejects_over_100_chars() {
        let long = "a".repeat(101);
        assert!(!valid_new_session_name(&long));
    }

    #[test]
    fn accepts_exactly_100_chars() {
        let ok = "a".repeat(100);
        assert!(valid_new_session_name(&ok));
    }
}
