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
async fn get_skills_returns_the_full_catalog_capped_at_the_default_limit() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/skills"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let total = body["total"].as_u64().unwrap();
    assert!(total >= 300, "expected at least 300 skills in the catalog, got {total}");

    let skills = body["skills"].as_array().unwrap();
    assert!(skills.len() <= 50, "default cap is 50, got {}", skills.len());
}

#[tokio::test]
async fn get_skills_filters_by_q_and_caps_by_limit() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/skills?q=audit&limit=5"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let skills = body["skills"].as_array().unwrap();
    assert!(skills.len() <= 5, "limit=5 must cap the returned skills, got {}", skills.len());
    assert!(!skills.is_empty(), "expected at least one skill matching 'audit'");

    for skill in skills {
        let name = skill["name"].as_str().unwrap().to_lowercase();
        let description = skill["description"].as_str().unwrap().to_lowercase();
        assert!(
            name.contains("audit") || description.contains("audit"),
            "skill {name:?} / {description:?} does not match q=audit"
        );
    }
}

#[tokio::test]
async fn get_skills_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;

    let res = reqwest::Client::new().get(format!("{base}/v1/skills")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn get_skills_filters_by_server_category() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/skills?category=Audit&limit=200"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let skills = body["skills"].as_array().unwrap();
    assert!(!skills.is_empty());
    assert!(skills.iter().all(|skill| skill["category"] == "Audit"));
}

#[tokio::test]
async fn skill_detail_returns_content_but_never_a_host_path() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default()));
    let base = spawn(app).await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/skills/monitor"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["skill"]["name"], "monitor");
    assert!(body["skill"]["content"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(body["skill"].get("path").is_none());
}
