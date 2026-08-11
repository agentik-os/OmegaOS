//! `POST /v1/duo` — `crates/omega-gateway/src/routes_duo.rs` (wave8 Task D).
//!
//! Fake-`OMEGA_DUO_BIN`-tested ONLY: no test in this file ever invokes a
//! real Codex/Claude/GLM turn. `OMEGA_DUO_DIR` scopes every scratch file to
//! a tempdir, never the operator's real `~/.omega/state/gateway-duo`.
//! `project`-based tests point `OMEGA_HOME` at a fake discoverable-project
//! tree (mirrors `dispatch_test.rs::install_fake_home`); `dir`-based tests
//! override the REAL `HOME` env var instead, since `dir_under_home`
//! deliberately reads `dirs::home_dir()` rather than the `OMEGA_HOME`
//! override (mirrors `sessions_create_test.rs`'s own dir-based tests).

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_DUO_BIN / OMEGA_DUO_DIR / OMEGA_HOME / HOME / DUO_TIMEOUT_SECS are
// process-global; serialize every test in this binary that touches any of
// them (same pattern as pdf_test.rs's / dispatch_test.rs's own LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(gateway_dir.to_path_buf(), GatewayConfig::default()));
    (app, token)
}

/// Creates a fake `$OMEGA_HOME` containing exactly one discoverable project
/// (a `.git`-marked directory named `name`), and points `OMEGA_HOME` at it
/// — the override `config::home_dir()` / `routes_duo::resolve_project_path`
/// respects (identical to `dispatch_test.rs::install_fake_home`).
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) {
    std::fs::create_dir_all(home_dir.join(project_name).join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
}

/// Writes an executable fake `omega-duo` that: (1) appends its full argv
/// (one per line) to `capture_file`; (2) optionally sleeps `sleep_secs`
/// (fractional seconds, bash `sleep` accepts e.g. `"0.15"`) BEFORE printing,
/// so concurrency/timeout tests can hold a run "in flight"; (3) prints
/// `stdout_body` VERBATIM (no trailing newline added by this helper -- the
/// caller decides); (4) exits `exit_code`. `stdout_body` must not contain a
/// single-quote character (this helper single-quotes it into the script).
fn install_fake_duo(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    sleep_secs: &str,
    stdout_body: &str,
    exit_code: i32,
) {
    use std::os::unix::fs::PermissionsExt;
    assert!(!stdout_body.contains('\''), "test stdout body must not contain a single quote");
    let path = bin_dir.join("omega-duo");
    let capture = capture_file.display();
    let body = format!(
        "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\nsleep {sleep_secs}\nprintf '%s' '{stdout_body}'\nexit {exit_code}\n"
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_DUO_BIN", &path);
}

fn clear_env() {
    std::env::remove_var("OMEGA_DUO_BIN");
    std::env::remove_var("OMEGA_DUO_DIR");
    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("HOME");
    std::env::remove_var("DUO_TIMEOUT_SECS");
}

fn argv_lines(capture_file: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(capture_file).unwrap_or_default().lines().map(str::to_string).collect()
}

/// The full real `BridgeResult` shape, filled with distinguishable
/// non-default values in every field so a field-mapping bug (a swapped or
/// dropped field) cannot hide behind a coincidental default.
fn fake_bridge_result_json() -> serde_json::Value {
    serde_json::json!({
        "agent": "codex",
        "ok": true,
        "output": "implemented the thing",
        "fell_back": false,
        "reason": null,
        "exit_code": 0,
        "log": "/home/x/.omega/logs/duo/20260101-run.log",
        "sandbox_degraded": false,
        "capabilities": { "shell_exec": true, "worktree_read": true },
        "guard_error": null,
        "verify": null,
        "checkpoint": { "head": "abc123", "stash": null, "ref": "refs/heads/main" },
        "diffstat": "1 file changed, 3 insertions(+)",
        "agent_ok": true
    })
}

// ── happy paths ───────────────────────────────────────────────────────────

#[tokio::test]
async fn happy_path_with_project_builds_exact_argv_and_maps_every_field() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    let body = fake_bridge_result_json();
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "fix the bug", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let resp: serde_json::Value = res.json().await.unwrap();
    // Field-for-field: the response IS the fake's JSON, mapped through
    // DuoResponse without loss or renaming (`ref` -> `git_ref` in Rust, but
    // still `ref` on the wire).
    assert_eq!(resp, body);

    let argv = argv_lines(&capture_file);
    let project_path = home_dir.path().join("TestProj");
    assert_eq!(argv[0], "run");
    let task_idx = argv.iter().position(|l| l == "--task").unwrap();
    let task_path = &argv[task_idx + 1];
    assert!(task_path.starts_with(duo_scratch.path().join("tasks").to_str().unwrap()), "argv: {argv:?}");
    let cwd_idx = argv.iter().position(|l| l == "--cwd").unwrap();
    assert_eq!(argv[cwd_idx + 1], project_path.to_str().unwrap());
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "code");
    // --agent / --verify are NEVER passed by this endpoint.
    assert!(!argv.iter().any(|l| l == "--agent" || l == "--verify"), "argv: {argv:?}");
    // No `--` separator (omega-duo's own parser has no positionals to
    // protect) and no `=`-joined flags (its parser does not understand
    // them -- see routes_duo.rs's doc comment).
    assert!(!argv.iter().any(|l| l == "--"), "argv: {argv:?}");
    assert!(!argv.iter().any(|l| l.contains('=')), "argv: {argv:?}");

    // The scratch task file genuinely contains the caller's prompt.
    let on_disk = std::fs::read_to_string(task_path).unwrap();
    assert_eq!(on_disk, "fix the bug");

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_uses_the_dir_under_home_resolved_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    let body = fake_bridge_result_json();
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home.path().join("Station").join("Proj").display().to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": dir_str, "prompt": "think about the architecture", "profile": "reflect" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let argv = argv_lines(&capture_file);
    let cwd_idx = argv.iter().position(|l| l == "--cwd").unwrap();
    assert_eq!(argv[cwd_idx + 1], dir_str);
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "plan");

    clear_env();
}

#[tokio::test]
async fn profile_review_maps_to_mode_review() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "review this diff", "profile": "review" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let argv = argv_lines(&capture_file);
    let mode_idx = argv.iter().position(|l| l == "--mode").unwrap();
    assert_eq!(argv[mode_idx + 1], "review");

    clear_env();
}

// ── validation, before any spawn ─────────────────────────────────────────

#[tokio::test]
async fn both_project_and_dir_given_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "X", "dir": "/tmp/y", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("ambiguous"));
    assert!(!capture_file.exists(), "no subprocess was ever spawned");

    clear_env();
}

#[tokio::test]
async fn neither_project_nor_dir_given_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("no target"));
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn unknown_project_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "RealProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "GhostProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "no subprocess was ever spawned for an unknown project");

    clear_env();
}

#[tokio::test]
async fn dir_outside_home_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // /etc definitely exists and is definitely not under the fake $HOME.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": "/etc", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn dir_with_parent_dir_component_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let fake_home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    std::env::set_var("HOME", fake_home.path());
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let escaping = fake_home.path().join("does-not-exist-yet").join("..").join("..").join("etc");
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "dir": escaping.display().to_string(), "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn empty_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "   ", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn oversized_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let too_long = "x".repeat(8001);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": too_long, "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn nul_byte_prompt_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "bad\u{0}prompt", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn unknown_profile_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "yolo" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("build"));
    assert!(!capture_file.exists());

    clear_env();
}

// ── malformed output ─────────────────────────────────────────────────────

#[tokio::test]
async fn malformed_stdout_is_502_with_stdout_and_stderr_surfaced() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0", "this is not json at all", 1);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["stdout"].as_str().unwrap().contains("not json"));

    clear_env();
}

/// A non-zero `omega-duo` exit code with a VALID `BridgeResult` JSON body
/// (`ok:false`, a legitimate outcome like "verify-failed") is still a 200 —
/// the JSON body's own `ok`/`agent_ok` fields are the real signal, never
/// this endpoint's own HTTP status. See `routes_duo.rs`'s doc comment.
#[tokio::test]
async fn nonzero_exit_with_valid_json_is_still_200() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    let mut body = fake_bridge_result_json();
    body["ok"] = serde_json::json!(false);
    body["agent_ok"] = serde_json::json!(true);
    body["reason"] = serde_json::json!("verify-failed");
    body["exit_code"] = serde_json::json!(1);
    install_fake_duo(bin_dir.path(), &capture_file, "0", &body.to_string(), 1);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let resp: serde_json::Value = res.json().await.unwrap();
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["agent_ok"], true);
    assert_eq!(resp["reason"], "verify-failed");

    clear_env();
}

// ── timeout ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn subprocess_past_a_short_overridden_timeout_returns_bounded_with_a_clear_error() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    // Sleeps 3s; the endpoint's own timeout is overridden to 1s, so this
    // must return well under the full sleep, never the full 1800s default.
    install_fake_duo(bin_dir.path(), &capture_file, "3", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());
    std::env::set_var("DUO_TIMEOUT_SECS", "1");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let started = std::time::Instant::now();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    let elapsed = started.elapsed();
    assert_eq!(res.status(), 504);
    assert!(elapsed < std::time::Duration::from_secs(3), "took {elapsed:?}, expected well under the 3s sleep");
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("timed out"));

    clear_env();
}

// ── concurrency ───────────────────────────────────────────────────────────

#[tokio::test]
async fn concurrency_cap_returns_429_when_duo_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    // Two DIFFERENT discoverable projects so the per-cwd lock (a SEPARATE
    // mechanism) never interferes with this test's concurrency-cap concern.
    std::fs::create_dir_all(home_dir.path().join("ProjA").join(".git")).unwrap();
    std::fs::create_dir_all(home_dir.path().join("ProjB").join(".git")).unwrap();
    std::fs::create_dir_all(home_dir.path().join("ProjC").join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir.path());
    install_fake_duo(bin_dir.path(), &capture_file, "0.15", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Must match server.rs's MAX_CONCURRENT_DUO_RUNS.
    const MAX_CONCURRENT_DUO_RUNS: usize = 2;
    let projects = ["ProjA", "ProjB"];

    let mut in_flight = Vec::new();
    for project in projects.iter().take(MAX_CONCURRENT_DUO_RUNS) {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        let project = project.to_string();
        in_flight.push(tokio::spawn(async move {
            client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "project": project, "prompt": "p", "profile": "build" }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let busy_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "ProjC", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(busy_res.status(), 429);
    let body: serde_json::Value = busy_res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too many concurrent"));

    for task in in_flight {
        assert_eq!(task.await.unwrap(), 200);
    }

    clear_env();
}

#[tokio::test]
async fn per_cwd_lock_rejects_a_second_concurrent_run_then_releases_after_the_first_completes() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_duo(bin_dir.path(), &capture_file, "0.25", &fake_bridge_result_json().to_string(), 0);
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let first = {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        tokio::spawn(async move {
            client
                .post(format!("{base}/v1/duo"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "project": "TestProj", "prompt": "p1", "profile": "build" }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        })
    };

    // Give the first request time to pass validation and acquire the
    // per-cwd lock before the second one (targeting the SAME project/cwd)
    // fires.
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let second_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p2", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(second_res.status(), 409);
    let body: serde_json::Value = second_res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("already in flight"));

    assert_eq!(first.await.unwrap(), 200);

    // The lock was actually released, not leaked: a THIRD request against
    // the same project/cwd, fired only after the first completed, is
    // accepted.
    let third_res = client
        .post(format!("{base}/v1/duo"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "TestProj", "prompt": "p3", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(third_res.status(), 200);

    clear_env();
}

// ── auth ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let duo_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DUO_DIR", duo_scratch.path());
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/duo"))
        .json(&serde_json::json!({ "project": "X", "prompt": "p", "profile": "build" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}
