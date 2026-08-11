//! PDF generation: `POST /v1/pdf` and `GET /v1/pdf/download?path=` — wave7
//! task E.
//!
//! `omega pdf --template=<t> --data=<path> --out=<path>` has no in-process
//! library entry point in this workspace (pdfgen is its own tool under
//! `tools/pdfgen/`, invoked as a subprocess by the CLI itself — see
//! `crates/omega-cli/src/main.rs::cmd_pdf`), so `POST /v1/pdf` shells to
//! `omega_cli::run`, argv-only, exactly like `routes_box::backup` does.
//!
//! `template` is validated against the literal known set
//! (`whitepaper`/`audit`/`marketing`/`doc`) BEFORE any spawn. `data` is
//! client-supplied JSON: written to a SERVER-CHOSEN scratch path under
//! [`pdf_data_dir`] (never a client-supplied path passed to `--data`),
//! mirroring `routes_box::backup`'s server-chosen-`--out` posture — an
//! authenticated caller can never direct a write to an arbitrary path the
//! gateway process can reach. `--out` is likewise always server-chosen
//! (under [`pdf_output_dir`]), never client-supplied.
//!
//! `GET /v1/pdf/download?path=` is scoped to [`pdf_output_dir`] ONLY: the
//! query value is reduced to its bare file NAME (`Path::file_name()`)
//! before ever touching the filesystem, then resolved through
//! `routes_files::resolve_scoped_path` — the SAME canonicalize-and-prefix-
//! check idiom `routes_files.rs` already carries, including its
//! ancestor-walk fix for the outside-root-403-vs-404 existence-oracle leak
//! wave6 found. Reducing to `file_name()` first means even a client that
//! echoes back an absolute path (or attempts `../../etc/passwd`) only ever
//! contributes a single path COMPONENT — traversal is structurally
//! impossible here, not merely rejected after the fact. `data` scratch
//! files live under the SEPARATE [`pdf_data_dir`] (never inside
//! `pdf_output_dir`), so the download endpoint's scope can never reach a
//! generation request's raw input JSON.
//!
//! `--send`/`--caption` are NEVER passed by this endpoint (documented in
//! `.superpowers/sdd/progress.md`'s Task E ground-truth section): pushing
//! to the operator's real Telegram from an API call with no operator-side
//! confirmation would be exactly the kind of surprising, hard-to-reverse
//! side effect this crate's other mutating endpoints (e.g.
//! `routes_oracles::reap`/`resurrect`) go out of their way to avoid.

use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::protocol::PdfRequest;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg.into() })))
}

fn not_found(msg: impl Into<String>) -> ApiError {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": msg.into() })))
}

fn forbidden(msg: impl Into<String>) -> ApiError {
    (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": msg.into() })))
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": msg.into() })))
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": msg.to_string() })))
}

/// The literal `--template` values `omega pdf` accepts (`crates/omega-cli/
/// src/main.rs::Commands::Pdf` doc comment: "whitepaper, audit, marketing,
/// doc"). Validated BEFORE any subprocess spawn.
const KNOWN_TEMPLATES: &[&str] = &["whitepaper", "audit", "marketing", "doc"];

/// Root scratch directory for this endpoint's own files — `OMEGA_PDF_DIR`
/// env override, else `std::env::temp_dir()`, mirroring
/// `routes_box::backup_dir`'s `OMEGA_BACKUP_DIR` convention (this is
/// disposable scratch output, not `~/.omega` state).
fn pdf_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_PDF_DIR") {
        return PathBuf::from(dir);
    }
    std::env::temp_dir().join("omega-gateway-pdf")
}

/// Where generated PDFs land — the ONLY directory `GET /v1/pdf/download`
/// can ever serve from. Kept SEPARATE from [`pdf_data_dir`] so a download
/// request can never reach a request's raw input JSON.
fn pdf_output_dir() -> PathBuf {
    pdf_root_dir().join("output")
}

/// Where this endpoint writes the client's `data` JSON before invoking
/// `omega pdf --data=<this>`. Never exposed to `GET /v1/pdf/download`.
fn pdf_data_dir() -> PathBuf {
    pdf_root_dir().join("data")
}

/// `POST /v1/pdf` — validates `template`, writes `data` to a server-chosen
/// scratch file, runs `omega pdf --template=<t> --data=<scratch> --out=<scratch>`,
/// and returns the generated path + size. A non-zero exit is a real error
/// (502) — there is no "expected non-zero" outcome for PDF generation the
/// way there is for `omega doctor`.
pub async fn create(Json(req): Json<PdfRequest>) -> Result<Json<crate::protocol::PdfResponse>, ApiError> {
    if !KNOWN_TEMPLATES.contains(&req.template.as_str()) {
        return Err(bad_request(format!(
            "unknown template '{}' (expected one of: {})",
            req.template,
            KNOWN_TEMPLATES.join(", ")
        )));
    }

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.f").to_string();
    let data_dir = pdf_data_dir();
    let out_dir = pdf_output_dir();
    let data_path = data_dir.join(format!("data-{ts}.json"));
    let out_path = out_dir.join(format!("omega-report-{ts}.pdf"));
    let data_path_str = data_path.to_string_lossy().to_string();
    let out_path_str = out_path.to_string_lossy().to_string();
    let template = req.template.clone();
    let data = req.data.clone();
    let argv_out_path_str = out_path_str.clone();

    let output = tokio::task::spawn_blocking(move || -> Result<crate::omega_cli::CommandOutput, ApiError> {
        std::fs::create_dir_all(&data_dir).map_err(|e| internal(format!("mkdir {}: {e}", data_dir.display())))?;
        std::fs::create_dir_all(&out_dir).map_err(|e| internal(format!("mkdir {}: {e}", out_dir.display())))?;
        let json = serde_json::to_string_pretty(&data).map_err(|e| internal(format!("serialize data: {e}")))?;
        std::fs::write(&data_path, json).map_err(|e| internal(format!("write data file: {e}")))?;
        crate::omega_cli::run(&[
            "pdf",
            "--template",
            &template,
            "--data",
            &data_path_str,
            "--out",
            &argv_out_path_str,
        ])
        .map_err(|e| bad_gateway(format!("failed to run omega pdf: {e}")))
    })
    .await
    .map_err(|e| internal(format!("pdf task panicked: {e}")))??;

    if !output.success {
        return Err(bad_gateway(format!(
            "omega pdf exited non-zero: {}",
            if output.stderr.trim().is_empty() { &output.stdout } else { &output.stderr }
        )));
    }

    let size_bytes = tokio::task::spawn_blocking(move || std::fs::metadata(&out_path).map(|m| m.len()))
        .await
        .map_err(|e| internal(format!("stat task panicked: {e}")))?
        .map_err(|e| bad_gateway(format!("omega pdf reported success but no file at {out_path_str}: {e}")))?;

    Ok(Json(crate::protocol::PdfResponse { path: out_path_str, size_bytes }))
}

/// `GET /v1/pdf/download?path=` — see this module's doc comment. Streams
/// the raw PDF bytes with `Content-Type: application/pdf`.
pub async fn download(Query(query): Query<HashMap<String, String>>) -> Result<Response, ApiError> {
    let raw = query.get("path").cloned().unwrap_or_default();
    if raw.trim().is_empty() {
        return Err(bad_request("path is required"));
    }
    // Reduce to the bare file name FIRST -- see this module's doc comment
    // for why this makes traversal structurally impossible rather than
    // merely rejected.
    let Some(name) = std::path::Path::new(&raw).file_name().and_then(|n| n.to_str()) else {
        return Err(bad_request("path must contain a valid file name"));
    };
    let name = name.to_string();

    let root = pdf_output_dir();
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ApiError> {
        let resolved = crate::routes_files::resolve_scoped_path(&root, &name).map_err(|e| match e {
            crate::routes_files::PathError::Invalid(msg) => bad_request(msg),
            crate::routes_files::PathError::Escaped => forbidden("path escapes the pdf output directory"),
            crate::routes_files::PathError::NotFound => not_found("no such generated pdf"),
        })?;
        let meta = std::fs::metadata(&resolved).map_err(|e| internal(format!("stat failed: {e}")))?;
        if !meta.is_file() {
            return Err(bad_request("path is not a file"));
        }
        std::fs::read(&resolved).map_err(|e| internal(format!("read failed: {e}")))
    })
    .await
    .map_err(|e| internal(format!("download task panicked: {e}")))??;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf")],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_root_dir_honors_env_override() {
        std::env::set_var("OMEGA_PDF_DIR", "/tmp/omega-pdf-test-root");
        assert_eq!(pdf_root_dir(), PathBuf::from("/tmp/omega-pdf-test-root"));
        assert_eq!(pdf_output_dir(), PathBuf::from("/tmp/omega-pdf-test-root/output"));
        assert_eq!(pdf_data_dir(), PathBuf::from("/tmp/omega-pdf-test-root/data"));
        std::env::remove_var("OMEGA_PDF_DIR");
    }

    #[test]
    fn known_templates_match_the_cli() {
        for t in ["whitepaper", "audit", "marketing", "doc"] {
            assert!(KNOWN_TEMPLATES.contains(&t));
        }
        assert!(!KNOWN_TEMPLATES.contains(&"evil"));
    }
}
