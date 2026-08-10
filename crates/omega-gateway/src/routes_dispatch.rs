//! `POST /v1/dispatch` — the one mutating endpoint in this plan: launches an
//! oracle (or delivers a followup into a live one) via the real `omega`
//! CLI's `dispatch` subcommand. SECURITY-CRITICAL INVARIANT: the
//! `omega_cli::run` subprocess is NEVER spawned for a `project` that isn't
//! in the discovered-project list — validation completes fully before any
//! process touches the filesystem beyond the read-only `discover` walk.
//!
//! ## Output contract (confirmed against `omega_core::dispatch.rs` and
//! `omega_cli::main.rs::cmd_dispatch`, not guessed):
//!
//! - `DispatchOutcome::report_lines()` (dispatch.rs:195) ALWAYS emits line 0
//!   as `"◆ Oracle dispatched: {name}"` — a leading glyph + space precede
//!   "Oracle", so the parser searches for the substring `"Oracle dispatched"`
//!   rather than assuming the line starts with it. A `Followup` /
//!   `SpawnedPaneNotReady` / `FollowupUnconfirmed` delivery inserts ONE
//!   additional French annotation line after line 0; `Spawned` does not. The
//!   LAST line `report_lines()` itself emits is always
//!   `"DISPATCH_DELIVERY={tag}"` where `tag` is one of `spawned` / `followup`
//!   / `spawned_pane_not_ready` / `followup_unconfirmed`
//!   (`DispatchDelivery::tag()`, dispatch.rs:246).
//! - BUT `cmd_dispatch` (main.rs:6096-6128), the actual CLI entry point, prints
//!   ONE MORE line AFTER `report_lines()`: `"  Mission: {mission}"`. So real
//!   CLI stdout does NOT end with the `DISPATCH_DELIVERY=` line — the parser
//!   below finds the line that STARTS WITH `"DISPATCH_DELIVERY="` by scanning
//!   all lines, never by indexing `lines.last()`.
//! - The `omega dispatch` argv shape (`Commands::Dispatch`, main.rs:402-417):
//!   positional `project`, positional `mission`, `--agent <value>` (only when
//!   given), `--new` (a bare flag, only when the caller wants to force a new
//!   oracle rather than let the followup router decide).

use crate::protocol::{DispatchRequest, DispatchResponse};
use axum::{http::StatusCode, Json};
use serde_json::json;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg.into() })))
}

/// Parses `omega dispatch`'s stdout into a `DispatchResponse`, per the
/// contract documented at the top of this file. Returns `None` on any
/// shape mismatch — the caller treats that as a 502, never a guess.
fn parse_dispatch_stdout(stdout: &str) -> Option<DispatchResponse> {
    let mut lines = stdout.lines();
    let first = lines.next()?;
    // Line 0: find "Oracle dispatched" anywhere in the line (real output is
    // "◆ Oracle dispatched: <name>"), optional colon, whitespace, then the
    // oracle-<name> token.
    let idx = first.find("Oracle dispatched")?;
    let after = &first[idx + "Oracle dispatched".len()..];
    let after = after.strip_prefix(':').unwrap_or(after);
    let oracle = after.trim();
    if oracle.is_empty() {
        return None;
    }
    let oracle = oracle.to_string();

    // The DISPATCH_DELIVERY= line can appear anywhere in the remaining
    // output (a followup annotation line, and the CLI's own trailing
    // "  Mission: ..." line, both come after it) — scan every line rather
    // than trusting position.
    let delivery = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("DISPATCH_DELIVERY="))?
        .trim()
        .to_string();
    if delivery.is_empty() {
        return None;
    }

    Some(DispatchResponse { oracle, delivery })
}

pub async fn create(
    Json(req): Json<DispatchRequest>,
) -> Result<Json<DispatchResponse>, ApiError> {
    // Step 1: reject empty/whitespace-only project/mission BEFORE any other
    // check — this is free and catches the common client bug of an empty
    // form field without even touching the filesystem.
    if req.project.trim().is_empty() {
        return Err(bad_request("project must not be empty"));
    }
    if req.mission.trim().is_empty() {
        return Err(bad_request("mission must not be empty"));
    }

    // Step 2: validate the project against the discovered-project list
    // BEFORE spawning anything. This is the security-critical gate: no
    // subprocess is spawned for an unknown project, full stop.
    let project = req.project.clone();
    let known = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home).into_iter().any(|p| p.name == project)
    })
    .await
    .unwrap_or(false);
    if !known {
        return Err(bad_request(format!("unknown project: {}", req.project)));
    }

    // Step 3: build argv (never a shell string) and run `omega dispatch`.
    let project = req.project.clone();
    let mission = req.mission.clone();
    let agent = req.agent.clone();
    let force_new = req.new == Some(true);
    let output = tokio::task::spawn_blocking(move || {
        let mut args: Vec<&str> = vec!["dispatch", &project, &mission];
        if let Some(agent) = agent.as_deref() {
            args.push("--agent");
            args.push(agent);
        }
        if force_new {
            args.push("--new");
        }
        crate::omega_cli::run(&args)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("dispatch task panicked: {e}") })),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("failed to spawn omega: {e}") })),
        )
    })?;

    // Step 4: non-zero exit or unparseable stdout → 502, never fabricate an
    // oracle name.
    if !output.success {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "omega dispatch failed", "stderr": output.stderr, "stdout": output.stdout })),
        ));
    }
    let Some(parsed) = parse_dispatch_stdout(&output.stdout) else {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "unparseable omega dispatch output", "stdout": output.stdout })),
        ));
    };

    Ok(Json(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spawned_output() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: do the thing\n";
        let parsed = parse_dispatch_stdout(stdout).unwrap();
        assert_eq!(parsed.oracle, "oracle-TestProj-1");
        assert_eq!(parsed.delivery, "spawned");
    }

    #[test]
    fn parses_followup_output_with_extra_annotation_line() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-Verba\n  (suivi: route dans l oracle vivant, aucun nouvel oracle cree)\nDISPATCH_DELIVERY=followup\n  Mission: more work\n";
        let parsed = parse_dispatch_stdout(stdout).unwrap();
        assert_eq!(parsed.oracle, "oracle-Verba");
        assert_eq!(parsed.delivery, "followup");
    }

    #[test]
    fn rejects_missing_delivery_tag() {
        let stdout = "\u{25c6} Oracle dispatched: oracle-X\n  Mission: x\n";
        assert!(parse_dispatch_stdout(stdout).is_none());
    }

    #[test]
    fn rejects_missing_oracle_line() {
        let stdout = "something unexpected\nDISPATCH_DELIVERY=spawned\n";
        assert!(parse_dispatch_stdout(stdout).is_none());
    }
}
