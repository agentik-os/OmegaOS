//! `GET /v1/agents` — the dispatch-target agent roster, from
//! `omega_core::agents::Agent::all()`. Pure, synchronous, side-effect-free
//! data apart from `is_available()`'s per-agent PATH lookup (cheap, ~8
//! calls, no network) — same `spawn_blocking` idiom `routes_rules::list`
//! uses, kept off the async runtime thread.

use crate::protocol::{AgentEntry, AgentsResponse};
use axum::Json;

pub async fn list() -> Json<AgentsResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let agents = omega_core::agents::Agent::all()
            .iter()
            .map(|a| AgentEntry {
                name: a.name().to_string(),
                display_name: a.display_name().to_string(),
                available: a.is_available(),
            })
            .collect();
        AgentsResponse { agents }
    })
    .await
    .unwrap_or(AgentsResponse { agents: vec![] });
    Json(response)
}
