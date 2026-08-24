use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn get_projects_returns_the_discovered_project_list() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/projects"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let projects = body["projects"].as_array().unwrap();
    assert!(
        projects.len() >= 10,
        "expected at least 10 discovered projects on this box, got {}",
        projects.len()
    );

    let first = &projects[0];
    assert!(
        first["name"].as_str().is_some(),
        "first project missing name"
    );
    assert!(
        first["container"].as_str().is_some(),
        "first project missing container"
    );
    assert!(
        first["stack"].as_array().is_some(),
        "first project missing stack"
    );
}

#[tokio::test]
async fn get_projects_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
