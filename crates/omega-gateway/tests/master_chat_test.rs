//! Integration tests for `GET /v1/master/chat` —
//! `crates/omega-gateway/src/routes_master.rs`. Mirrors `omega aisb-chat`'s
//! exact file protocol (inbox append + conversation-log poll), gated on
//! `aisb-master` rmux-session liveness.
//!
//! Every test here mutates `$HOME` (the CLI's own home resolution, NOT
//! `$OMEGA_HOME`), `OMEGA_RMUX_BIN` (fake rmux), and/or
//! `OMEGA_AISB_POLL_ATTEMPTS`/`OMEGA_AISB_POLL_INTERVAL_MS` — all
//! process-global env vars, so every test acquires `LOCK` first (same
//! pattern as `audit_test.rs` / `box_test.rs` / `stream_test.rs`).

use futures_util::{SinkExt, StreamExt};
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

/// Writes an executable fake `rmux` script and points `OMEGA_RMUX_BIN` at
/// it — same pattern as `stream_test.rs::install_fake_rmux`.
fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

/// Fake rmux whose `ls -F #S` reports `aisb-master` as a live session.
fn install_fake_rmux_master_live(dir: &std::path::Path) {
    install_fake_rmux(dir, "echo 'aisb-master'");
}

/// Fake rmux whose `ls -F #S` reports some OTHER session, never
/// `aisb-master`.
fn install_fake_rmux_master_absent(dir: &std::path::Path) {
    install_fake_rmux(dir, "echo 'oracle-Foo-1'");
}

fn clear_env() {
    std::env::remove_var("HOME");
    std::env::remove_var("OMEGA_RMUX_BIN");
    std::env::remove_var("OMEGA_AISB_POLL_ATTEMPTS");
    std::env::remove_var("OMEGA_AISB_POLL_INTERVAL_MS");
}

fn inbox_path(fake_home: &std::path::Path) -> std::path::PathBuf {
    fake_home.join(".omega/state/aisb-local-inbox.jsonl")
}

fn log_path(fake_home: &std::path::Path) -> std::path::PathBuf {
    fake_home.join(".omega/state/aisb-conversation.log")
}

#[tokio::test]
async fn not_running_sends_frame_and_never_touches_inbox() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let rmux_bin_dir = tempfile::tempdir().unwrap();
    install_fake_rmux_master_absent(rmux_bin_dir.path());
    std::env::set_var("HOME", fake_home.path());

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let url = ws_url(&base, "/v1/master/chat", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(tokio_tungstenite::tungstenite::Message::Text("hello master".to_string())).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "not_running");

    assert!(
        !inbox_path(fake_home.path()).exists(),
        "inbox must never be created/touched when aisb-master is not live"
    );

    clear_env();
}

#[tokio::test]
async fn running_reply_round_trip_writes_inbox_and_returns_reply() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let rmux_bin_dir = tempfile::tempdir().unwrap();
    install_fake_rmux_master_live(rmux_bin_dir.path());
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("OMEGA_AISB_POLL_INTERVAL_MS", "20");
    std::env::set_var("OMEGA_AISB_POLL_ATTEMPTS", "50"); // 1s budget

    // Pre-seed the conversation log with known initial content so the
    // server's "before" length is fixed before any background growth.
    let log = log_path(fake_home.path());
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    std::fs::write(&log, "existing conversation\n").unwrap();

    // Background growth: after a short delay (shorter than the 1s poll
    // budget, longer than a couple of poll ticks), append the "response" the
    // bridge would have written.
    let log_for_growth = log.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(&log_for_growth).unwrap();
        writeln!(f, "AISB: hello back").unwrap();
    });

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let url = ws_url(&base, "/v1/master/chat", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(tokio_tungstenite::tungstenite::Message::Text("hello master".to_string())).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "reply");
    assert!(
        frame["text"].as_str().unwrap().contains("AISB: hello back"),
        "unexpected reply text: {frame}"
    );

    // The inbox DOES contain the appended JSON line for this round trip.
    let inbox_content = std::fs::read_to_string(inbox_path(fake_home.path())).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(inbox_content.trim()).unwrap();
    assert_eq!(parsed["text"], "hello master");
    assert!(parsed["ts"].as_str().is_some());

    clear_env();
}

#[tokio::test]
async fn running_timeout_when_log_never_grows() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let rmux_bin_dir = tempfile::tempdir().unwrap();
    install_fake_rmux_master_live(rmux_bin_dir.path());
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("OMEGA_AISB_POLL_INTERVAL_MS", "5");
    std::env::set_var("OMEGA_AISB_POLL_ATTEMPTS", "3"); // 15ms budget, never grows

    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let url = ws_url(&base, "/v1/master/chat", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(tokio_tungstenite::tungstenite::Message::Text("anyone there?".to_string())).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "timeout");

    // The inbox was still written (a real round trip that just never got a
    // reply in time) — same distinction the CLI itself makes.
    assert!(inbox_path(fake_home.path()).exists());

    clear_env();
}

#[tokio::test]
async fn requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let url = format!("{}/v1/master/chat", base.replacen("http", "ws", 1));
    let err = connect_async(url).await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("401") || msg.contains("Unauthorized"), "unexpected error: {msg}");
}
