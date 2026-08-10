use axum::Json;
use serde_json::json;

pub async fn list() -> Json<serde_json::Value> {
    match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => Json(json!({
            "sessions": names.iter().map(|n| json!({ "name": n })).collect::<Vec<_>>()
        })),
        Ok(Err(e)) => Json(json!({ "sessions": [], "error": e.to_string() })),
        Err(e) => Json(json!({ "sessions": [], "error": e.to_string() })),
    }
}
