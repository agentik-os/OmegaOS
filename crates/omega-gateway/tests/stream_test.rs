use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// tokio::sync::Mutex (not std): the guard is held across .await points below,
// and clippy::await_holding_lock correctly flags a std guard doing that.
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

#[tokio::test]
async fn stream_sends_frame_then_only_on_change() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    // fake rmux: capture-pane output changes based on a counter file
    install_fake_rmux(dir.path(), &format!(r#"
counter="{}/count"
n=$(cat "$counter" 2>/dev/null || echo 0)
echo $((n+1)) > "$counter"
if [ $n -lt 2 ]; then echo "SCREEN-A"; else echo "SCREEN-B"; fi"#,
        dir.path().display()));
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig { stream_interval_ms: 50, ..GatewayConfig::default() };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/demo/stream?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(f1["type"], "frame");
    assert!(f1["text"].as_str().unwrap().contains("SCREEN-A"));

    // next frame arrives only when content changes to SCREEN-B
    let second = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f2: serde_json::Value = serde_json::from_str(&second).unwrap();
    assert!(f2["text"].as_str().unwrap().contains("SCREEN-B"));
}

#[tokio::test]
async fn capture_failure_becomes_error_frame_and_loop_survives() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), "echo 'session not found' >&2; exit 1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let cfg = GatewayConfig { stream_interval_ms: 50, ..GatewayConfig::default() };
    let app = build_router(AppState::new(dir.path().to_path_buf(), cfg));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/ghost/stream?token={token}");
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "error");
    assert!(frame["message"].as_str().unwrap().contains("session not found"));
    // the connection is still alive: another error frame arrives instead of a close
    let msg2 = ws.next().await.unwrap().unwrap();
    assert!(msg2.is_text());
}
