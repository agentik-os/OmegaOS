//! Dynamic operative-system catalog for the Omega app.

use crate::protocol::{OsProductEntry, OsProductsResponse};
use axum::Json;
use omega_core::os_products::OsGroup;

/// The label lives on `OsGroup` (generated from the suite registry), so adding
/// a group never silently breaks this endpoint.
fn group_label(group: OsGroup) -> &'static str {
    group.label()
}

pub async fn list() -> Json<OsProductsResponse> {
    let os = tokio::task::spawn_blocking(|| {
        omega_core::os_products::list_os_entries()
            .into_iter()
            .map(|entry| OsProductEntry {
                slug: entry.product.slug.to_string(),
                name: entry.product.name.to_string(),
                category: group_label(entry.product.group).to_string(),
                status: entry.status_label().to_string(),
                path: format!("OS/{}", entry.product.slug),
                bot: entry.bot_linked,
            })
            .collect()
    })
    .await
    .unwrap_or_default();
    Json(OsProductsResponse { os })
}
