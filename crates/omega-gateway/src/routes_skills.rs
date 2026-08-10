//! `GET /v1/skills` — the OmegaOS skill catalog, from
//! `omega_core::skill_registry::SkillRegistry::discover_default()`. Optional
//! `?q=` filters by case-insensitive substring match against name OR
//! description; `?limit=` caps the returned page (default 50, hard ceiling
//! 200 — a bad/non-numeric value falls back to the default rather than
//! panicking). `total` always reflects the UNFILTERED catalog size.
//!
//! `discover_default()` walks `~/.omega/skills/` on disk, so a discovery
//! failure (e.g. an unreadable dir) degrades to an empty `{skills:[],
//! total:0}` response with a `tracing::warn!` — same "never 500 on a
//! degraded read" posture `routes_sessions::list` uses for a failed `rmux`
//! call — kept off the async runtime thread via `spawn_blocking`, same idiom
//! `routes_rules::list` / `routes_agents::list` use.

use crate::protocol::{SkillEntry, SkillsResponse};
use axum::extract::Query;
use axum::Json;
use std::collections::HashMap;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;

pub async fn list(Query(params): Query<HashMap<String, String>>) -> Json<SkillsResponse> {
    let response = tokio::task::spawn_blocking(move || {
        let registry = match omega_core::skill_registry::SkillRegistry::discover_default() {
            Ok(registry) => registry,
            Err(e) => {
                tracing::warn!("skill discovery failed: {e}");
                return SkillsResponse { skills: vec![], total: 0 };
            }
        };

        let all = registry.list();
        let total = all.len();

        let q = params.get("q").map(|s| s.to_lowercase()).filter(|s| !s.is_empty());
        let limit = params
            .get("limit")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let skills = all
            .into_iter()
            .filter(|skill| match &q {
                None => true,
                Some(q) => {
                    skill.name.to_lowercase().contains(q.as_str())
                        || skill.description.to_lowercase().contains(q.as_str())
                }
            })
            .take(limit)
            .map(|skill| SkillEntry {
                name: skill.name.clone(),
                description: skill.description.clone(),
                category: skill.category.label().to_string(),
            })
            .collect();

        SkillsResponse { skills, total }
    })
    .await
    .unwrap_or(SkillsResponse { skills: vec![], total: 0 });
    Json(response)
}
