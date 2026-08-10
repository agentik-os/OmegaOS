//! Adversarial verification tests written by an independent reviewer (not
//! the implementer) for the `GET /v1/agents/{name}/install/stream` WS route
//! in `crates/omega-gateway/src/routes_agents.rs`.
//!
//! SAFETY: exactly like `agents_install_test.rs`, every test here sets
//! `OMEGA_BIN` to a fake, purely-local bash script BEFORE making any
//! request that could reach `install_stream`. No test in this file ever
//! touches the real `~/.local/bin/omega`, `curl`, or `npm`, and no test
//! reaches the network.
//!
//! Two things are verified here that the existing suite does not cover:
//!
//! 1. `dual_stream_large_volume_...`: a few thousand lines on BOTH stdout
//!    and stderr, concurrently, prove the two `forward_lines` tasks feeding
//!    one mpsc channel don't deadlock or drop lines under volume.
//!
//! 2. `mid_stream_client_disconnect_...`: a client that disconnects while
//!    the fake installer is still producing output. This is the important
//!    one — it does NOT merely check "the test process doesn't hang" (that
//!    alone would pass even if the bug below existed, because
//!    `forward_lines` returning on a failed `tx.send` after `drop(rx)`
//!    eventually unblocks *this* test regardless). It checks whether the
//!    REAL child process tree is actually terminated on disconnect, by
//!    giving the fake "installer" a NESTED child process (mirroring
//!    `omega-cli`'s real `cmd_install`, which runs `bash -c
//!    <install_command>` via `.status()` — a genuine child of the `omega`
//!    process, not the process the gateway itself spawned) that would write
//!    a marker file well after the point where the gateway kills on
//!    disconnect. `install_stream_loop` now spawns `omega` into its own
//!    process group (`process_group(0)`) and, on disconnect, kills the
//!    WHOLE group (`kill -- -<pid>`, see `kill_process_group` in
//!    `routes_agents.rs`) instead of the single direct PID — which reaches
//!    the nested `bash -c` too, since it inherits the group. If the marker
//!    file is written anyway, the group-kill failed to actually stop the
//!    real work — the socket closed but the install kept running
//!    unsupervised.

use futures_util::StreamExt;
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

fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// (1) Large-volume dual-stream: 3000 stdout lines + 3000 stderr lines,
/// printed with NO delay so the OS pipe buffers (typically 64 KiB) fill up
/// many times over on both fds concurrently. Proves the two `forward_lines`
/// tasks (one per fd) reading concurrently into one mpsc channel neither
/// deadlock nor drop lines.
#[tokio::test]
async fn dual_stream_large_volume_neither_deadlocks_nor_drops_lines() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    install_fake_omega(
        dir.path(),
        r#"
if [ "$1" = "install" ]; then
    for i in $(seq 1 3000); do
        echo "out $i"
        echo "err $i" >&2
    done
    exit 0
fi
exit 1
"#,
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    let mut stdout_lines: Vec<i64> = Vec::new();
    let mut stderr_lines: Vec<i64> = Vec::new();
    let mut exit_frame: Option<serde_json::Value> = None;

    // Bound the WHOLE drain: if the two forwarder tasks (or the mpsc
    // channel between them and the socket-writer loop) deadlock under
    // volume, this fires instead of the test hanging forever.
    let drain = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        loop {
            let msg = ws.next().await.unwrap().unwrap();
            let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
            if v["type"] == "exit" {
                exit_frame = Some(v);
                break;
            }
            let stream = v["stream"].as_str().unwrap();
            let text = v["text"].as_str().unwrap();
            let n: i64 = text.split_whitespace().nth(1).unwrap().parse().unwrap();
            if stream == "stdout" {
                stdout_lines.push(n);
            } else {
                stderr_lines.push(n);
            }
        }
    })
    .await;

    assert!(drain.is_ok(), "dual-stream drain did not complete within 30s — deadlock");
    let exit_frame = exit_frame.expect("exit frame must have been received");
    assert_eq!(exit_frame["success"], true);
    assert_eq!(exit_frame["code"], 0);

    assert_eq!(stdout_lines.len(), 3000, "dropped/duplicated stdout lines");
    assert_eq!(stderr_lines.len(), 3000, "dropped/duplicated stderr lines");
    stdout_lines.sort_unstable();
    stderr_lines.sort_unstable();
    let expected: Vec<i64> = (1..=3000).collect();
    assert_eq!(stdout_lines, expected, "stdout sequence has a gap or a duplicate");
    assert_eq!(stderr_lines, expected, "stderr sequence has a gap or a duplicate");

    std::env::remove_var("OMEGA_BIN");
}

/// (2) Mid-stream client disconnect: proves (or falsifies) the claim that
/// killing on client-disconnect actually stops the real work, including
/// the nested process the direct child itself spawns.
///
/// The fake `omega install` here has the SAME process shape as the real
/// `omega-cli::cmd_install`: the top-level process (the one the gateway's
/// `Command::new(omega_bin())` actually spawns, and the one whose PID the
/// gateway directly controls) runs a NESTED `bash -c '<installer>'` in the
/// foreground (mirroring `cmd_install`'s `Command::new("bash").args(["-c",
/// cmd]).status()`), which is a process-tree CHILD OF THE FAKE OMEGA
/// SCRIPT, not of the gateway. That nested process prints a few more lines
/// (so the gateway's next post-disconnect send attempt fails and triggers
/// the kill path) and then, well after the kill should have landed, would
/// write a marker file.
///
/// `install_stream_loop` spawns the fake `omega` into its own process group
/// (`process_group(0)`) and, on disconnect, kills the WHOLE group
/// (`kill_process_group`, a `kill -- -<pid>` negative-PID kill) rather than
/// only the direct PID. Because the nested `bash -c` inherits that same
/// group, the group-kill reaches it too, so the marker must NEVER appear —
/// proving a disconnect now actually aborts the whole install tree, not
/// just the outer wrapper.
#[tokio::test]
async fn mid_stream_client_disconnect_kills_the_nested_installer_too() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("orphan-still-ran.marker");
    install_fake_omega(
        dir.path(),
        &format!(
            r#"
if [ "$1" = "install" ]; then
    echo "starting"
    # Mirrors cmd_install's `Command::new("bash").args(["-c", cmd]).status()`:
    # a foreground child of THIS script, not of the gateway's own Command.
    bash -c '
        for i in 1 2 3; do
            sleep 1
            echo "installer progress $i"
        done
        touch "{marker}"
    '
fi
"#,
            marker = marker.display()
        ),
    );
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/agents/codex/install/stream", &token);
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Read exactly the first frame ("starting"), then disconnect immediately
    // — before any of the nested installer's own output arrives. This is
    // the "client closes mid-stream" scenario from the review brief.
    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    drop(ws); // client-side disconnect

    assert!(!marker.exists(), "marker must not exist yet — installer hasn't reached it");

    // Give the server up to 8s to (a) notice the disconnect on its next
    // send attempt (triggered by "installer progress 1" at ~1s) and (b)
    // group-kill the child. The nested installer's OWN loop needs ~3s to
    // finish and touch the marker if it is NOT actually killed, so poll for
    // up to that same ~8s bound rather than checking once instantly (which
    // could pass by accident before the nested process even had a chance to
    // run) — a bounded wait that only PASSES once we're confident the
    // marker will never appear, not merely that it hasn't appeared yet.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the marker WAS written: the nested installer process kept running to \
             completion after the gateway's disconnect-kill fired. Killing only the \
             direct child (the omega wrapper) does not reach a nested `bash -c \
             <install_command>` process the wrapper itself spawned, exactly as \
             omega-cli::cmd_install's real Command::new(\"bash\")...status() does — \
             the group-kill (process_group(0) + kill -- -<pid>) must reach it too."
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the marker WAS written at the deadline: the nested installer survived the \
         disconnect-triggered group-kill — the fix did not actually stop the real work."
    );

    std::env::remove_var("OMEGA_BIN");
}
