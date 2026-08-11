//! Integration tests for `POST /v1/pdf` and `GET /v1/pdf/download` —
//! `crates/omega-gateway/src/routes_pdf.rs` (wave7 task E).
//!
//! Fake-`OMEGA_BIN`-tested only (never a real `npx tsx pdfgen.ts` run in
//! this crate's tests). `OMEGA_PDF_DIR` scopes every scratch file to a
//! tempdir, never the operator's real `/tmp/omega-gateway-pdf`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN / OMEGA_PDF_DIR are process-global; serialize every test in
// this binary that touches either (same pattern as box_test.rs's LOCK).
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

fn clear_env() {
    std::env::remove_var("OMEGA_BIN");
    std::env::remove_var("OMEGA_PDF_DIR");
}

/// Fake `omega` that captures its full argv AND actually writes fake PDF
/// bytes to whatever `--out <path>` it was given -- this crate's `create()`
/// handler stats the out path for real after a successful exit, so a fake
/// bin that only prints text (like `box_test.rs`'s backup fake) would make
/// every happy-path test 502.
fn install_fake_omega_pdf(bin_dir: &std::path::Path, capture_file: &std::path::Path, exit_code: i32) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    let capture = capture_file.display();
    let body = format!(
        r#"#!/usr/bin/env bash
printf '%s\n' "$@" > '{capture}'
out=""
while [[ $# -gt 0 ]]; do
  if [[ "$1" == "--out" ]]; then out="$2"; fi
  shift
done
if [[ {exit_code} -eq 0 && -n "$out" ]]; then
  printf '%%PDF-FAKE-CONTENT' > "$out"
fi
exit {exit_code}
"#
    );
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

fn argv_lines(capture_file: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(capture_file).unwrap_or_default().lines().map(str::to_string).collect()
}

#[tokio::test]
async fn create_happy_path_returns_path_and_size_and_writes_the_data_file() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_omega_pdf(bin_dir.path(), &capture_file, 0);
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "template": "whitepaper", "data": { "title": "Q3 report" } }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let path = body["path"].as_str().unwrap().to_string();
    assert!(path.starts_with(pdf_scratch.path().join("output").to_str().unwrap()));
    assert_eq!(body["size_bytes"], "%PDF-FAKE-CONTENT".len() as u64);

    // Real argv: --template whitepaper --data <scratch>/data/... --out <scratch>/output/...
    let argv = argv_lines(&capture_file);
    assert_eq!(argv[0], "pdf");
    assert!(argv.contains(&"--template".to_string()));
    assert!(argv.contains(&"whitepaper".to_string()));
    assert!(argv.iter().any(|l| l.contains("/data/")), "argv: {argv:?}");
    assert!(argv.iter().any(|l| l.contains("/output/")), "argv: {argv:?}");
    // --send / --caption are NEVER passed.
    assert!(!argv.iter().any(|l| l.contains("send") || l.contains("caption")), "argv: {argv:?}");

    // The data file genuinely holds the client's JSON.
    let data_idx = argv.iter().position(|l| l == "--data").unwrap();
    let data_path = &argv[data_idx + 1];
    let data_raw = std::fs::read_to_string(data_path).unwrap();
    let data_json: serde_json::Value = serde_json::from_str(&data_raw).unwrap();
    assert_eq!(data_json["title"], "Q3 report");

    clear_env();
}

#[tokio::test]
async fn create_rejects_unknown_template_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_omega_pdf(bin_dir.path(), &capture_file, 0);
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "template": "evil-template", "data": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(!capture_file.exists(), "no subprocess was ever spawned for an unknown template");

    clear_env();
}

#[tokio::test]
async fn create_nonzero_exit_is_a_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_omega_pdf(bin_dir.path(), &capture_file, 1);
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "template": "audit", "data": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);

    clear_env();
}

#[tokio::test]
async fn create_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .json(&serde_json::json!({ "template": "doc", "data": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}

// ── GET /v1/pdf/download ─────────────────────────────────────────────────

#[tokio::test]
async fn download_round_trips_a_generated_pdf() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_omega_pdf(bin_dir.path(), &capture_file, 0);
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let create_res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "template": "doc", "data": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let create_body: serde_json::Value = create_res.json().await.unwrap();
    let path = create_body["path"].as_str().unwrap().to_string();

    let dl_res = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", path.as_str())])
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res.status(), 200);
    assert_eq!(dl_res.headers().get("content-type").unwrap(), "application/pdf");
    let bytes = dl_res.bytes().await.unwrap();
    assert_eq!(&bytes[..], b"%PDF-FAKE-CONTENT");

    clear_env();
}

#[tokio::test]
async fn download_rejects_traversal_even_when_the_client_echoes_an_absolute_outside_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // /etc/passwd definitely exists on this box -- proves the traversal
    // guard, not merely "the file happened not to exist".
    let dl_res = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", "/etc/passwd")])
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    // file_name("/etc/passwd") == "passwd", scoped under the EMPTY pdf
    // output dir -- it is not there, so this must be a clean 404, and in
    // particular must NEVER be 200 with /etc/passwd's real content.
    assert_eq!(dl_res.status(), 404);
    let body_text = dl_res.text().await.unwrap();
    assert!(!body_text.contains("root:"), "must never leak real /etc/passwd content");

    // A relative traversal attempt is likewise reduced to a bare file name.
    let dl_res2 = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", "../../../etc/passwd")])
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res2.status(), 404);

    clear_env();
}

#[tokio::test]
async fn download_missing_file_is_404() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dl_res = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", "no-such-report.pdf")])
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res.status(), 404);

    clear_env();
}

#[tokio::test]
async fn download_cannot_reach_the_separate_data_dir() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    let capture_file = bin_dir.path().join("capture.txt");
    install_fake_omega_pdf(bin_dir.path(), &capture_file, 0);
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let create_res = reqwest::Client::new()
        .post(format!("{base}/v1/pdf"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "template": "doc", "data": { "secret": "input-only" } }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let argv = argv_lines(&capture_file);
    let data_idx = argv.iter().position(|l| l == "--data").unwrap();
    let data_path = std::path::Path::new(&argv[data_idx + 1]);
    let data_filename = data_path.file_name().unwrap().to_str().unwrap();

    // The data JSON file's bare name is never reachable through the
    // download endpoint -- it is scoped to the output dir only.
    let dl_res = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", data_filename)])
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res.status(), 404);

    clear_env();
}

#[tokio::test]
async fn download_requires_auth() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let pdf_scratch = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_PDF_DIR", pdf_scratch.path());
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/pdf/download"))
        .query(&[("path", "x.pdf")])
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    clear_env();
}
