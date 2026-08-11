//! `POST /v1/sessions` — `crates/omega-gateway/src/routes_sessions.rs::create`.
//! Wraps `omega new [OPTIONS] <NAME>`.
//!
//! A dedicated file (rather than extending `sessions_test.rs`, which is
//! scoped to `GET /v1/sessions` and mutates the unrelated `OMEGA_RMUX_BIN`
//! env var): this endpoint's tests mutate `OMEGA_BIN` and `HOME` instead,
//! and keeping the two env-var concerns in separate files/binaries avoids
//! any risk of one file's `LOCK` scope leaking into the other's.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN and HOME are process-global; serialize every test in this binary
// that touches either (same pattern as dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script that also appends its full argv
/// (one per line) to `capture_file` — same idiom `dispatch_test.rs::
/// install_fake_omega` uses.
fn install_fake_omega(bin_dir: &std::path::Path, capture_file: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    let capture = capture_file.display();
    std::fs::write(
        &path,
        format!("#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\n{script_body}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(gateway_dir.to_path_buf(), GatewayConfig::default()));
    (app, token)
}

fn clear_env() {
    std::env::remove_var("OMEGA_BIN");
    std::env::remove_var("HOME");
}

/// A real agent name off the actual roster, so this test tracks the real
/// roster rather than a hardcoded guess (same posture
/// `dispatch_test.rs::known_agent_dispatches_normally` uses).
fn real_agent_name() -> &'static str {
    let agents = omega_core::agents::Agent::all();
    assert!(!agents.is_empty());
    agents[0].name()
}

#[tokio::test]
async fn happy_path_with_explicit_name_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "printf 'Created session: my-session\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "name": "my-session",
            "dir": null,
            "prompt": "do the thing"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "my-session");
    assert_eq!(body["agent"], agent);
    assert!(body["output"].as_str().unwrap().contains("Created session"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["new", "--agent", agent, "--prompt", "do the thing", "--", "my-session"]);

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_builds_exact_argv_in_order() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();
    let agent = real_agent_name();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home.path().join("Station").join("Proj").display().to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "name": "my-session",
            "dir": dir_str,
            "prompt": "hello"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec!["new", "--agent", agent, "--dir", &dir_str, "--prompt", "hello", "--", "my-session"]
    );

    clear_env();
}

#[tokio::test]
async fn happy_path_with_no_name_generates_one_and_uses_it_as_the_positional() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let name = body["name"].as_str().unwrap().to_string();
    // Server-generated pattern: "gw-" + 12 lowercase hex chars (6 random bytes).
    assert!(name.starts_with("gw-"), "expected gw- prefix, got {name}");
    let hex_part = &name["gw-".len()..];
    assert_eq!(hex_part.len(), 12, "expected 12 hex chars, got {hex_part}");
    assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["new", "--agent", agent, "--", name.as_str()]);

    clear_env();
}

#[tokio::test]
async fn unknown_agent_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": "not-a-real-agent" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("not-a-real-agent"));
    assert!(!capture_file.exists(), "omega subprocess was spawned for an unknown agent");

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_leading_dash_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "-leading-dash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for an invalid name");

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_slash_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "has/slash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for an invalid name");

    clear_env();
}

#[tokio::test]
async fn invalid_caller_name_nul_byte_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "has\u{0000}nul" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for a NUL-containing name");

    clear_env();
}

#[tokio::test]
async fn dir_outside_home_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let agent = real_agent_name();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "agent": agent,
            "dir": outside.path().display().to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("home directory"));
    assert!(!capture_file.exists(), "omega subprocess was spawned for a dir outside home");

    clear_env();
}

#[tokio::test]
async fn prompt_over_length_cap_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // One byte over routes_sessions.rs's MAX_PROMPT_LEN (8000).
    let too_long = "x".repeat(8001);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "prompt": too_long }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too long"));
    assert!(!capture_file.exists(), "omega subprocess was spawned for an over-length prompt");

    clear_env();
}

#[tokio::test]
async fn nonzero_exit_surfaces_stdout_and_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let agent = real_agent_name();

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'partial output' ; echo 'session name already in use' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "agent": agent, "name": "dup-session" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["stdout"].as_str().unwrap().contains("partial output"));
    assert!(body["stderr"].as_str().unwrap().contains("session name already in use"));
    assert!(body.get("name").is_none(), "must never fabricate a session on failure");

    clear_env();
}

#[tokio::test]
async fn post_sessions_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/sessions"))
        .json(&serde_json::json!({ "agent": "claude" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
