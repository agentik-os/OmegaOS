//! `GET /v1/new-project/stream?name=&category=&group=` — runs `omega
//! new-project` and streams its stdout/stderr.
//!
//! DEVIATION FROM THE LITERAL TASK BRIEF, already recorded in
//! `.superpowers/sdd/progress.md`'s Task B ground-truth section:
//! `cmd_new_project` (`crates/omega-cli/src/main.rs:3191`) is FAST, NOT
//! long-running — a non-dry-run invocation does exactly ONE thing beyond
//! printing: `mgr.create_session_with_agent(...).await?`, which spawns a
//! Codex rmux session running the `/omega-new-project ...` prompt and
//! returns in well under a second, identical in shape to
//! `cmd_new`/`cmd_dispatch`. The WHOLE vision -> PRD -> brand -> planner ->
//! build pipeline then runs ASYNCHRONOUSLY *inside* that spawned session —
//! invisible to this CLI process, and out of scope for this wave to watch
//! (that would mean streaming an arbitrary rmux pane, R-STREAM's job, not
//! this endpoint's). Built anyway, exactly as the brief asked: it's still
//! the correct, useful shape (mirrors `routes_agents::install_stream`'s
//! pre-upgrade-validation + Line/Exit-frame pattern 1:1, surfaces a real
//! spawn failure over the socket, and an `Exit` frame confirms the session
//! was created) even though in practice it will emit ~3 short lines then
//! `Exit` almost immediately — the same "browsers can't upgrade a POST"
//! reasoning wave7 already used to collapse `POST /v1/orchestrate` into a
//! GET WS stream, so no separate synchronous `POST /v1/new-project` ships
//! alongside it (one endpoint, not two, avoiding a redundant second code
//! path for the same subprocess).
//!
//! `name` is validated against the CLI's own documented charset
//! (`^[a-z0-9-]+$`, non-empty, capped at 64 bytes — confirmed live via
//! `omega new-project --help`: "Project name (lowercase [a-z0-9-])").
//! `category` is validated against the literal closed set the same `--help`
//! output documents (`Category: works | client | 1-life | AgentikOS`); when
//! omitted, it resolves to the CLI's own documented default (`works`,
//! `#[arg(default_value = "works")]` on `Commands::NewProject::category`).
//! `group` is optional and forwarded as `--group <value>` only when given
//! (`--group <GROUP>` is a real flag, confirmed live: "Provisioning
//! credential group (client projects) [default: default]") — it has no
//! separately documented charset, so this endpoint reuses `name`'s slug
//! check as the safe conservative default, since it flows straight into
//! `--group` and downstream provisioning credential lookup.
//!
//! `stack` (the 2nd positional) is NEVER exposed as a caller-controlled
//! parameter — always hardcoded to the literal `"nextstack"` ("the only
//! stack today" per the CLI's own help text). `--build`, `--resume`,
//! `--from`, `--skip`, `--budget`, and `--dry-run` are NEVER passed — this
//! endpoint only does the plain default bootstrap, none of the CLI's other
//! opt-in modes.
//!
//! CONCURRENCY: capped via `AppState::new_project_permits` — see
//! `server.rs`'s `MAX_CONCURRENT_NEW_PROJECT_SPAWNS` doc comment for why
//! this is sized the same as `orchestrate_permits` despite the subprocess
//! itself being fast: what it spawns is not.
//!
//! NEVER RUN FOR REAL IN TESTS (R-SEC / this task's explicit brief): every
//! test in `tests/new_project_test.rs` points `OMEGA_BIN` at a fake script.
//! Nothing in this crate's test suite or live-verify ever invokes a real
//! `omega new-project`, which spawns an actual Codex session running a real
//! pipeline prompt.

use crate::protocol::NewProjectStreamMsg;
use crate::server::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, OwnedSemaphorePermit};

/// Closed set of categories `omega new-project`'s own `--help` documents
/// (`Category: works | client | 1-life | AgentikOS`, confirmed live).
/// `Commands::NewProject::category` is a plain `String` with
/// `default_value = "works"` in the real CLI — clap enforces nothing itself
/// beyond that default, so this endpoint enforces the closed set the help
/// text documents rather than forwarding an arbitrary caller string.
const ALLOWED_CATEGORIES: &[&str] = &["works", "client", "1-life", "AgentikOS"];

/// The CLI's own documented default (`#[arg(default_value = "works")]`,
/// confirmed live via `omega new-project --help`), applied here when the
/// caller omits `category` entirely — so omitting it and passing it
/// explicitly produce the identical resolved value.
const DEFAULT_CATEGORY: &str = "works";

/// `STACK`, the 2nd positional — ALWAYS this literal, never a
/// caller-controlled parameter (see this module's doc comment).
const STACK: &str = "nextstack";

/// Byte-length cap on `name`/`group`, mirroring `routes_orchestrate.rs::
/// MAX_MISSION_LEN` (duplicated rather than shared — see this module's doc
/// comment on why routes in this crate don't share stream-loop machinery).
const MAX_SLUG_LEN: usize = 64;

/// `true` iff `s` matches `^[a-z0-9-]+$` (non-empty, only lowercase ASCII
/// letters, digits, and hyphens) — the CLI's own documented charset for
/// `NAME` (`omega new-project --help`: "Project name (lowercase
/// [a-z0-9-])"), confirmed live. Also reused for `group`: it has no
/// separately documented charset, but the same conservative check is the
/// safe default for a value that flows straight into `--group` and
/// downstream provisioning credential lookup.
fn is_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Validates `name`/`category`/`group` BEFORE any upgrade or spawn, and
/// resolves `category` to its documented default when the caller omits it.
/// Returns `(name, category, group)` ready to build argv from.
fn resolve_new_project_request(
    name: &str,
    category: &str,
    group: &str,
) -> Result<(String, String, Option<String>), (StatusCode, String)> {
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty".to_string()));
    }
    if name.len() > MAX_SLUG_LEN {
        return Err((StatusCode::BAD_REQUEST, format!("name too long (max {MAX_SLUG_LEN} bytes)")));
    }
    if !is_slug(name) {
        return Err((
            StatusCode::BAD_REQUEST,
            "name must match ^[a-z0-9-]+$ (lowercase letters, digits, hyphens only)".to_string(),
        ));
    }

    let category = if category.is_empty() { DEFAULT_CATEGORY.to_string() } else { category.to_string() };
    if !ALLOWED_CATEGORIES.contains(&category.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("unknown category: {category} (must be one of: {})", ALLOWED_CATEGORIES.join(", ")),
        ));
    }

    let group = if group.is_empty() {
        None
    } else {
        if group.len() > MAX_SLUG_LEN {
            return Err((StatusCode::BAD_REQUEST, format!("group too long (max {MAX_SLUG_LEN} bytes)")));
        }
        if !is_slug(group) {
            return Err((
                StatusCode::BAD_REQUEST,
                "group must match ^[a-z0-9-]+$ (lowercase letters, digits, hyphens only)".to_string(),
            ));
        }
        Some(group.to_string())
    };

    Ok((name.to_string(), category, group))
}

/// `GET /v1/new-project/stream?name=&category=&group=` — validates BEFORE
/// ever upgrading (a bad param returns a plain
/// `StatusCode::BAD_REQUEST.into_response()`, mirroring
/// `routes_orchestrate::stream`'s exact shape), then acquires a
/// `new_project_permits` permit BEFORE upgrading (a bare 429 on exhaustion,
/// never an upgrade followed by an in-loop rejection).
pub async fn stream(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let name = query.get("name").cloned().unwrap_or_default();
    let category = query.get("category").cloned().unwrap_or_default();
    let group = query.get("group").cloned().unwrap_or_default();

    match resolve_new_project_request(&name, &category, &group) {
        Ok((name, category, group)) => {
            let Ok(permit) = state.new_project_permits.clone().try_acquire_owned() else {
                return StatusCode::TOO_MANY_REQUESTS.into_response();
            };
            ws.on_upgrade(move |socket| new_project_stream_loop(socket, name, category, group, permit))
        }
        Err((code, _msg)) => code.into_response(),
    }
}

/// Serializes and sends one new-project-stream frame. `Err` means the
/// socket is dead.
async fn send_new_project_frame(
    socket: &mut WebSocket,
    frame: &NewProjectStreamMsg,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(frame).expect("serialize NewProjectStreamMsg");
    socket.send(Message::Text(text.into())).await
}

/// Reads `reader` line-by-line and forwards each line as a `Line{stream:
/// stream_name, ..}` frame on `tx`. Returns on EOF, a read error, or the
/// receiver having been dropped. Identical shape to
/// `routes_orchestrate::forward_lines` — `stdout` and `stderr` are each read
/// by their OWN spawned instance of this function (see
/// [`new_project_stream_loop`]) so neither pipe can starve the other.
async fn forward_lines<R: AsyncRead + Unpin>(
    reader: R,
    stream_name: &'static str,
    tx: mpsc::Sender<NewProjectStreamMsg>,
) {
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(text)) => {
                let frame = NewProjectStreamMsg::Line { stream: stream_name.to_string(), text };
                if tx.send(frame).await.is_err() {
                    return;
                }
            }
            Ok(None) | Err(_) => return,
        }
    }
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — identical to
/// `routes_orchestrate::kill_process_group` (same negative-PID `kill --
/// -<pid>` idiom, same `spawn_blocking`, same rationale: `pid` must be the
/// PID of a process spawned with `process_group(0)`, so this reaches both it
/// and every nested child it forked, in particular `omega`'s own nested
/// subprocess work).
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill").arg("--").arg(format!("-{pid}")).status()
    })
    .await;
}

/// Shared disconnect-cleanup, called from BOTH places
/// [`new_project_stream_loop`] can learn the client is gone — identical
/// structure to `routes_orchestrate::kill_and_drain`.
async fn kill_and_drain(
    rx: mpsc::Receiver<NewProjectStreamMsg>,
    mut child: tokio::process::Child,
    child_pid: Option<u32>,
    stdout_task: tokio::task::JoinHandle<()>,
    stderr_task: tokio::task::JoinHandle<()>,
) {
    drop(rx);
    match child_pid {
        Some(pid) => {
            kill_process_group(pid).await;
            let _ = child.wait().await;
        }
        None => {
            let _ = child.kill().await;
        }
    }
    let _ = stdout_task.await;
    let _ = stderr_task.await;
}

/// Runs `omega new-project [--group <group>] -- <name> nextstack <category>`
/// and streams its output to `socket`. Deliberately NOT shared code with
/// `routes_orchestrate::orchestrate_stream_loop` — see this module's doc
/// comment.
///
/// The spawned `omega` process is placed into its OWN process group
/// (`process_group(0)`) for the same reason `orchestrate_stream_loop`
/// documents at length: `omega new-project`'s real work spawns a nested
/// rmux session, and a plain single-PID `kill()` would not reach it.
/// `kill_on_drop(true)` guarantees the DIRECT child is reaped even if this
/// future is ever dropped without reaching the explicit disconnect-kill
/// path.
///
/// The `tokio::select!` loop watches BOTH the internal mpsc frame channel
/// AND the socket's own read side, so a disconnect is noticed even when the
/// child has gone quiet — the same read-driven fix `orchestrate_stream_loop`
/// carries.
///
/// `_permit` is held for the WHOLE lifetime of this function and released
/// automatically when it completes, whatever the reason.
async fn new_project_stream_loop(
    mut socket: WebSocket,
    name: String,
    category: String,
    group: Option<String>,
    _permit: OwnedSemaphorePermit,
) {
    let mut cmd = Command::new(crate::omega_cli::omega_bin());
    cmd.arg("new-project");
    if let Some(group) = &group {
        cmd.arg("--group").arg(group);
    }
    // Flags first, then `--`, then the THREE positionals LAST (NAME, STACK,
    // CATEGORY — the CLI's own declared argument order) — `name`/`category`
    // could themselves look flag-like even after this endpoint's charset
    // validation (a leading hyphen is legal under `^[a-z0-9-]+$`), and
    // everything after `--` is treated as a positional value by clap
    // regardless (same clap-safe argv construction
    // `routes_orchestrate.rs::orchestrate_stream_loop` documents).
    cmd.arg("--").arg(&name).arg(STACK).arg(&category);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = send_new_project_frame(
                &mut socket,
                &NewProjectStreamMsg::Error { message: format!("failed to spawn omega: {e}") },
            )
            .await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };
    // Captured now, before any consuming/`.await`ing method (`kill`/`wait`)
    // is ever called on `child` — `Child::id()` returns `None` once the
    // child has been polled to completion.
    let child_pid = child.id();

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (tx, mut rx) = mpsc::channel::<NewProjectStreamMsg>(64);
    let stdout_task = tokio::spawn(forward_lines(stdout, "stdout", tx.clone()));
    let stderr_task = tokio::spawn(forward_lines(stderr, "stderr", tx.clone()));
    drop(tx);

    loop {
        tokio::select! {
            frame = rx.recv() => {
                match frame {
                    Some(frame) => {
                        if send_new_project_frame(&mut socket, &frame).await.is_err() {
                            kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                            return;
                        }
                    }
                    None => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => {
                        kill_and_drain(rx, child, child_pid, stdout_task, stderr_task).await;
                        return;
                    }
                    Some(Ok(_)) => {}
                }
            }
        }
    }

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let exit_frame = match child.wait().await {
        Ok(status) => NewProjectStreamMsg::Exit { success: status.success(), code: status.code() },
        Err(e) => NewProjectStreamMsg::Error { message: format!("failed to wait on child: {e}") },
    };
    let _ = send_new_project_frame(&mut socket, &exit_frame).await;
    let _ = socket.send(Message::Close(None)).await;
}

#[cfg(test)]
mod resolve_new_project_request_tests {
    use super::resolve_new_project_request;
    use axum::http::StatusCode;

    #[test]
    fn rejects_empty_name() {
        let err = resolve_new_project_request("", "", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("name"));
    }

    #[test]
    fn rejects_name_over_length_cap() {
        let too_long = "a".repeat(65);
        let err = resolve_new_project_request(&too_long, "", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("too long"));
    }

    #[test]
    fn rejects_name_with_uppercase() {
        let err = resolve_new_project_request("MyProject", "", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_name_with_underscore() {
        let err = resolve_new_project_request("my_project", "", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn accepts_valid_slug_name() {
        let (name, category, group) = resolve_new_project_request("my-cool-project-1", "", "").unwrap();
        assert_eq!(name, "my-cool-project-1");
        assert_eq!(category, "works");
        assert_eq!(group, None);
    }

    #[test]
    fn defaults_category_to_works_when_omitted() {
        let (_, category, _) = resolve_new_project_request("proj", "", "").unwrap();
        assert_eq!(category, "works");
    }

    #[test]
    fn accepts_every_documented_category() {
        for cat in ["works", "client", "1-life", "AgentikOS"] {
            let (_, category, _) = resolve_new_project_request("proj", cat, "").unwrap();
            assert_eq!(category, cat);
        }
    }

    #[test]
    fn rejects_unknown_category() {
        let err = resolve_new_project_request("proj", "not-a-category", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("category"));
    }

    #[test]
    fn rejects_category_with_wrong_case() {
        // The closed set is case-sensitive: "Works" is not "works".
        let err = resolve_new_project_request("proj", "Works", "").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn group_is_none_when_omitted() {
        let (_, _, group) = resolve_new_project_request("proj", "", "").unwrap();
        assert_eq!(group, None);
    }

    #[test]
    fn group_is_forwarded_when_given() {
        let (_, _, group) = resolve_new_project_request("proj", "", "acme-client").unwrap();
        assert_eq!(group, Some("acme-client".to_string()));
    }

    #[test]
    fn rejects_bad_group_charset() {
        let err = resolve_new_project_request("proj", "", "Acme_Client").unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("group"));
    }
}
