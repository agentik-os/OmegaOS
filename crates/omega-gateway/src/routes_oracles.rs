//! `GET /v1/oracles` — the live oracle roster: every top-level mission
//! ledger (`missions::list()`) composed with whether its session currently
//! appears in `rmux::list_sessions()`. Read-only, no gateway state written.
//!
//! NAMING NOTE (see `protocol::OracleEntry`): `Mission.key` is populated
//! verbatim from the ledger JSON's `"oracle"` field, and that field already
//! IS the full rmux session name (e.g. `"oracle-dentistrygpt"`) —
//! `missions.rs` never strips or re-derives a bare identifier from the
//! filename. So `key` and `session` below are the same string; there is no
//! `format!("oracle-{}", ...)` prefixing to apply (that would double the
//! prefix into `"oracle-oracle-dentistrygpt"`).

use crate::protocol::{OracleEntry, OraclesResponse};
use axum::Json;

pub async fn list() -> Json<OraclesResponse> {
    let missions = tokio::task::spawn_blocking(crate::missions::list).await.unwrap_or_default();
    // Never 500 on a degraded rmux read (same posture as routes_sessions::list):
    // a failed liveness probe just means every entry reports live: false.
    let live_sessions = match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => names,
        _ => vec![],
    };

    let oracles = missions
        .into_iter()
        .map(|m| {
            let session = m.key.clone();
            let live = live_sessions.contains(&session);
            OracleEntry { key: m.key.clone(), session, live, mission: Some(m) }
        })
        .collect();
    Json(OraclesResponse { oracles })
}
