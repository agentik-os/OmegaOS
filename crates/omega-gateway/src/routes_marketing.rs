//! `GET /v1/marketing` — the marketing-enabled project list, from
//! `omega_core::marketing::list_marketing_projects()`. Pure, synchronous,
//! side-effect-free filesystem scan (a project is "marketing-enabled" when
//! it has a `<path>/marketing/` directory) — no daemon, no network — kept
//! off the async runtime thread via `spawn_blocking`, same idiom
//! `routes_rules::list` / `routes_projects::list` use for their own
//! blocking calls.
//!
//! Read-only, list-only: `accounts` (connected social-account count) is
//! NEVER populated by this endpoint. Populating it needs
//! `omega_core::marketing::project_accounts`, which shells out to
//! `omega-zernio` per project and is bounded by its own 4s timeout — fine
//! for an on-demand detail lookup, wrong for a list endpoint that would
//! then shell out once per discovered project on every call. So `accounts`
//! (and `accounts_tried`) always mirror the list-only values the source
//! function itself never populates (`None` / `false`).

use crate::protocol::{MarketingProjectEntry, MarketingResponse};
use axum::Json;

pub async fn list() -> Json<MarketingResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let projects = omega_core::marketing::list_marketing_projects()
            .into_iter()
            .map(|p| MarketingProjectEntry {
                name: p.name,
                slug: p.slug,
                has_content: p.has_content,
                calendar_posts: p.calendar_posts,
                engine_on: p.engine_on,
                accounts: p.accounts,
                accounts_tried: p.accounts_tried,
                has_context: p.has_context,
                has_strategy: p.has_strategy,
                has_copy: p.has_copy,
                has_visual: p.has_visual,
                has_branding: p.has_branding,
            })
            .collect();
        MarketingResponse { projects }
    })
    .await
    .unwrap_or(MarketingResponse { projects: vec![] });
    Json(response)
}
