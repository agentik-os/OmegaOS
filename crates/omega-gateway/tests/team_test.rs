//! `POST /v1/team` — `crates/omega-gateway/src/routes_team.rs::create`.
//! Wraps `omega team [OPTIONS] <PROJECT> [MEMBERS]...`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN and HOME are process-global; serialize every test in this binary
// that touches either (same pattern as sessions_create_test.rs's LOCK).
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

#[tokio::test]
async fn happy_path_with_members_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "printf '\\xe2\\x97\\x86 Team spawned: Team-Acme\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "project": "Acme",
            "count": 2,
            "members": ["alice:build the API", "bob:build the UI"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["session"], "Team-Acme");
    assert!(body["output"].as_str().unwrap().contains("Team spawned"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec!["team", "--count", "2", "--", "Acme", "alice:build the API", "bob:build the UI"]
    );

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_and_no_members_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home.path().display().to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "dir": dir_str }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["session"], "Team-Acme");

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["team", "--dir", &dir_str, "--", "Acme"]);

    clear_env();
}

#[tokio::test]
async fn only_count_no_members_still_forwards_count_with_no_member_args() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 5 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["team", "--count", "5", "--", "Acme"]);

    clear_env();
}

#[tokio::test]
async fn project_failing_slug_check_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "-leading-dash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for an invalid project name");

    clear_env();
}

#[tokio::test]
async fn count_zero_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for count=0");

    clear_env();
}

#[tokio::test]
async fn count_nine_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 9 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for count=9");

    clear_env();
}

#[tokio::test]
async fn count_one_and_eight_are_both_accepted() {
    let _g = LOCK.lock().await;
    for count in [1, 8] {
        let gateway_dir = tempfile::tempdir().unwrap();
        let bin_dir = tempfile::tempdir().unwrap();
        let capture_dir = tempfile::tempdir().unwrap();
        let capture_file = capture_dir.path().join("argv.txt");

        install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

        let (app, token) = app_and_token(gateway_dir.path()).await;
        let base = spawn(app).await;

        let res = reqwest::Client::new()
            .post(format!("{base}/v1/team"))
            .bearer_auth(&token)
            .json(&serde_json::json!({ "project": "Acme", "count": count }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "count={count} should be accepted");
        assert!(capture_file.exists());
    }

    clear_env();
}

#[tokio::test]
async fn oversized_member_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let too_long = format!("alice:{}", "x".repeat(500));
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": [too_long] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too long"));
    assert!(!capture_file.exists(), "omega subprocess was spawned for an oversized member");

    clear_env();
}

#[tokio::test]
async fn member_with_nul_byte_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": ["alice:has\u{0000}nul"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "omega subprocess was spawned for a NUL-containing member");

    clear_env();
}

#[tokio::test]
async fn nonzero_exit_surfaces_stdout_and_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'partial output'; echo 'rmux daemon unreachable' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["stdout"].as_str().unwrap().contains("partial output"));
    assert!(body["stderr"].as_str().unwrap().contains("rmux daemon unreachable"));
    assert!(body.get("session").is_none(), "must never fabricate a session on failure");

    clear_env();
}

#[tokio::test]
async fn post_team_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .json(&serde_json::json!({ "project": "Acme" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
