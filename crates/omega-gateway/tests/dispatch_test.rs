//! `POST /v1/dispatch` — the one mutating endpoint in this plan.
//!
//! The most important test in this file is
//! `unknown_project_rejects_before_any_subprocess_spawn`: it proves that an
//! unrecognized project name is rejected with a 400 WITHOUT ever invoking
//! the `omega` binary — the capture file a spawned fake `omega` would have
//! written is asserted absent.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_HOME and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as oracles_test.rs's
// OMEGA_STATE_DIR/OMEGA_RMUX_BIN lock).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script and points `OMEGA_BIN` at it.
/// The script also appends its full argv (one per line, `--`-separated) to
/// a capture file under `capture_dir`, so a test can prove exactly what was
/// passed to the subprocess.
fn install_fake_omega(bin_dir: &std::path::Path, capture_file: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    let capture = capture_file.display();
    std::fs::write(
        &path,
        format!(
            "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\n{script_body}\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// Creates a fake `$HOME` containing exactly one discoverable project (a
/// `.git`-marked directory named `name`), and points `OMEGA_HOME` at it —
/// the override `config::home_dir()` / `routes_projects.rs` /
/// `routes_dispatch.rs` all respect (Task 8's HOME-override fix).
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) {
    std::fs::create_dir_all(home_dir.join(project_name).join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(gateway_dir.to_path_buf(), GatewayConfig::default()));
    (app, token)
}

#[tokio::test]
async fn happy_path_dispatches_and_returns_oracle_and_delivery() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        r#"printf '\xe2\x97\x86 Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: do the thing\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["oracle"], "oracle-TestProj-1");
    assert_eq!(body["delivery"], "spawned");

    // Prove no shell-string interpolation happened and no extra/missing args:
    // the recorded argv is exactly ["dispatch", "TestProj", "do the thing"].
    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["dispatch", "TestProj", "do the thing"]);

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn unknown_project_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    // A real project exists ("TestProj"), but the request names a different,
    // random-UUID project that provably does not exist anywhere.
    install_fake_home(home_dir.path(), "TestProj");
    // Install a fake omega that would fail loudly if invoked at all, proving
    // a spawn never happens rather than merely happening to succeed.
    install_fake_omega(bin_dir.path(), &capture_file, "echo 'SHOULD NEVER RUN' >&2; exit 1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let unknown = format!("definitely-not-a-real-project-{}", uuid_like());
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": unknown, "mission": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains(&unknown));

    // The single most important assertion in this plan: the subprocess was
    // NEVER spawned, so the capture file the fake omega script writes on
    // every invocation must not exist.
    assert!(!capture_file.exists(), "omega subprocess was spawned for an unknown project");

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn subprocess_failure_surfaces_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'oracle registry lock held by another dispatch' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["stderr"].as_str().unwrap().contains("oracle registry lock held"));
    assert!(body.get("oracle").is_none(), "must never fabricate an oracle name on failure");

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn empty_project_and_mission_reject_before_discovery() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "  ", "mission": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": ""}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn post_dispatch_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .json(&serde_json::json!({"project": "x", "mission": "y"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

/// A small dependency-free stand-in for a UUID: unique enough within one
/// test run that it provably won't collide with any real project name.
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{nanos:x}")
}
