//! `POST /v1/duo` — wraps `omega-duo run`, the deterministic bridge behind
//! the `/duo` skill (Claude ⇄ Codex/GLM binome pairing). wave8 Task D.
//!
//! GROUND TRUTH (read off the real installed binary, `~/.local/bin/
//! omega-duo`, a Bun/TypeScript script — not compiled, not guessed):
//! `"duo"` is NOT a member of `omega_core::agents::Agent::all()`, so there
//! is no persistent rmux session to spawn for it the way `POST /v1/sessions
//! {agent:"duo"}` would imply — the real mechanism is a bounded, one-shot
//! CLI: `omega-duo run --task <file.md> --cwd <dir> --mode <plan|code|
//! review> [--agent codex|claude|glm] [--verify "<cmd>"]`. It reads the
//! task's content itself off disk (`readFileSync(taskFile, ...)`, binary
//! line 1221) and prints EXACTLY ONE JSON line to stdout on completion
//! (`console.log(JSON.stringify(result))`, binary line 287) — no
//! progressive output, so this ships as a plain request/response endpoint,
//! never a WS stream.
//!
//! `profile` is this crate's own name for the `/duo` skill's three real
//! profiles, mapped 1:1 to the bridge's `--mode` by [`profile_to_mode`]:
//! `build` → `code`, `review` → `review`, `reflect` → `plan`.
//!
//! `--agent` is NEVER forwarded (no explicit agent override exposed this
//! wave — GLM is opt-in-only doctrine, and this is a programmatic caller,
//! not an operator explicitly asking for GLM; the bridge defaults to
//! codex-first/claude-on-quota-fallback on its own). `--verify` is NEVER
//! passed (no safe, generic success command exists for an arbitrary
//! caller-supplied prompt).
//!
//! ARGV SHAPE — VERIFIED, not assumed: `omega-duo run`'s own arg parser
//! (`parseArgs`, binary line 1462) recognizes ONLY `--flag value` pairs
//! (two separate argv elements) — there is NO support for a clap-style
//! `--flag=value` single token. Passing `--cwd=/some/path` would NOT be
//! read as `cwd`; `parseArgs` would set the literal key `"cwd=/some/path"`
//! to `"true"`, and `cmdRun`'s `args.cwd` would then be `undefined`,
//! silently falling back to `process.cwd()` (the gatewayd process's OWN
//! cwd) instead of the caller's resolved target. So this endpoint
//! deliberately does NOT use the `--flag=value` idiom `routes_sessions.rs`
//! uses for `omega new` (a clap-based CLI, where that idiom is exactly
//! right) — using it here would be a real correctness bug, not a defense.
//!
//! There is also no bare positional in `omega-duo run`'s own argv (`case
//! "run": parseArgs(rest)`, binary line 2630-2631 — `parseArgs`, unlike its
//! sibling `splitArgs`, ignores every non-`--`-prefixed token entirely), so
//! no `--` separator is needed or meaningful here.
//!
//! HYPHEN-LEADING VALUE RISK — VERIFIED: `parseArgs`'s value-consumption
//! check is `!argv[i + 1].startsWith("--")` — DOUBLE dash, not single. A
//! resolved `--cwd` value starting with a single `-` (e.g. a directory
//! literally named `-x`) is consumed correctly as the value; only a value
//! starting with a literal `--` would be misread as a boolean flag and the
//! following token reparsed as a new flag. [`dir_under_home`] only ever
//! returns an ORIGINAL (uncanonicalized) requested path, so in a
//! pathological case (a directory that happens to be named e.g. `--evil`
//! AND already exists under an ancestor resolvable relative to this
//! process's own cwd) this endpoint's own argv could theoretically hand
//! the bridge a `--`-prefixed cwd value — but even then the failure mode
//! is a harmless bridge-side `cwd is not a directory` bad-input rejection
//! (`cmdRun` resolves the swallowed `"true"` string, which is never a real
//! directory), NOT an argv-injection into a different flag's semantics.
//! `--task`/`--mode` never carry this risk at all: the task path is always
//! this endpoint's OWN server-generated absolute scratch path, and `mode`
//! is always one of the three literal strings [`profile_to_mode`] returns.
//!
//! SUCCESS SIGNAL — VERIFIED, not assumed: `emit()` (binary line 286) sets
//! `process.exitCode = result.ok ? 0 : (result.exit_code || 1)` — the OS
//! exit code is FULLY DERIVED from the JSON body's own `ok` field, not an
//! independent signal. So this endpoint's HTTP status is gated ONLY on
//! whether stdout parses into the expected [`crate::protocol::DuoResponse`]
//! shape — a non-zero `omega-duo` exit with VALID JSON is still a 200 (the
//! JSON's `ok`/`agent_ok`/`reason` fields are the real outcome, e.g.
//! `ok:false, agent_ok:true, reason:"verify-failed"` is a legitimate "the
//! run happened, its success criterion was red" result, never a gateway
//! error). Only stdout that fails to parse at all (a genuinely malformed or
//! empty response — a spawn crash, a bridge bug) is a 502, mirroring
//! `routes_dispatch.rs::create`'s unparseable-stdout failure path.
//!
//! `prompt` is written to a SERVER-CHOSEN scratch file under
//! [`duo_tasks_dir`] (`~/.omega/state/gateway-duo/tasks/<random hex>.md`)
//! — NEVER a client-supplied path reaching `--task`, mirroring
//! `routes_pdf.rs`'s `data`-scratch-file pattern. The scratch file is NOT
//! deleted after the run — this crate's established, previously-accepted
//! gap (see `routes_box::backup_dir`/`routes_pdf.rs`'s own output dir: no
//! reaper for scratch directories anywhere in this crate yet).
//!
//! CONCURRENCY, two layers: (1) [`crate::server::AppState::duo_permits`], a
//! flat cap on concurrently-running `omega-duo` subprocesses (429 on
//! exhaustion), and (2) [`crate::server::AppState::duo_active_dirs`], an
//! IN-PROCESS per-resolved-cwd lock (409 on a second concurrent request
//! against the SAME cwd) — the `/duo` skill's own doc is explicit that two
//! `omega-duo run`s on the same worktree corrupt each other's
//! git-checkpoint guard. See [`CwdLockGuard`].
//!
//! NEVER RUN A REAL Codex/Claude/GLM TURN IN TESTS: every test in
//! `tests/duo_test.rs` points `OMEGA_DUO_BIN` at a fake script.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::protocol::{DuoRequest, DuoResponse};
use crate::server::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg.into() })))
}

fn conflict(msg: impl Into<String>) -> ApiError {
    (StatusCode::CONFLICT, Json(serde_json::json!({ "error": msg.into() })))
}

fn too_many_requests(msg: impl Into<String>) -> ApiError {
    (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({ "error": msg.into() })))
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": msg.into() })))
}

fn gateway_timeout(msg: impl Into<String>) -> ApiError {
    (StatusCode::GATEWAY_TIMEOUT, Json(serde_json::json!({ "error": msg.into() })))
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": msg.to_string() })))
}

/// Byte-length cap on `prompt`, mirroring `routes_dispatch.rs::
/// MAX_MISSION_LEN`'s cap (duplicated rather than shared — that constant is
/// private to its own module, the same "deliberate mirror" every other
/// route file in this crate already carries for this exact bound).
const MAX_DUO_PROMPT_LEN: usize = 8000;

/// Default hard ceiling on how long `omega-duo run`'s subprocess may run
/// before this endpoint kills it and reports a 504 — `DUO_TIMEOUT_SECS`
/// env-overridable, seconds-granular (`tests/duo_test.rs`'s timeout test
/// overrides it to `"1"` and points the fake bin's sleep past that).
/// Deliberately far more generous than `routes_pdf.rs`'s 300s bound: a real
/// Codex "code"-mode turn against a non-trivial task can genuinely take
/// many minutes (`.superpowers/sdd/progress.md`'s Task D ground-truth
/// section records this explicitly), and killing a genuinely-in-progress
/// agent turn early is worse than holding a permit a while longer — the
/// [`crate::server::AppState::duo_permits`] cap (not this timeout) is what
/// bounds resource exhaustion under load.
const DEFAULT_DUO_TIMEOUT_SECS: u64 = 1800;

fn duo_timeout() -> Duration {
    let secs = std::env::var("DUO_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_DUO_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Root scratch directory for this endpoint's own files — `OMEGA_DUO_DIR`
/// env override, else `~/.omega/state/gateway-duo`. Mirrors
/// `routes_pdf.rs::pdf_root_dir`'s reasoning exactly (never
/// `std::env::temp_dir()`'s shared, world-writable `/tmp` — every other
/// per-purpose OmegaOS scratch directory this crate reads back from lives
/// under the operator's own `~/.omega`, the right trust boundary).
fn duo_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_DUO_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir().expect("no home dir").join(".omega").join("state").join("gateway-duo")
}

/// Where this endpoint writes the caller's `prompt` before invoking
/// `omega-duo run --task <this>`. Never deleted after the run — see this
/// module's doc comment.
fn duo_tasks_dir() -> PathBuf {
    duo_root_dir().join("tasks")
}

/// Resolves the `omega-duo` binary path: `OMEGA_DUO_BIN` env override, else
/// `~/.local/bin/omega-duo` (confirmed the real installed path on this box
/// via `which omega-duo`). Mirrors `omega_cli::omega_bin()`'s / `rmux::
/// rmux_bin()`'s exact two-tier shape — the `/duo` skill doc's own fuller
/// three-tier fallback (PATH, then a skills-relative path, then a repo
/// fallback) is DELIBERATELY not replicated here: this crate's established
/// convention for every other CLI binary it wraps is exactly two tiers, and
/// the real box only ever needs the second one.
pub(crate) fn duo_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_DUO_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir().expect("no home dir").join(".local/bin/omega-duo")
}

/// Maps this endpoint's `profile` (the `/duo` skill's own three real
/// profiles) to the bridge's `--mode` value. `None` for anything outside
/// the closed set — the caller turns that into a 400 before any spawn.
pub(crate) fn profile_to_mode(profile: &str) -> Option<&'static str> {
    match profile {
        "build" => Some("code"),
        "review" => Some("review"),
        "reflect" => Some("plan"),
        _ => None,
    }
}

/// Resolves `project` against the discovered-project allowlist — the exact
/// pattern `routes_orchestrate.rs::resolve_project_path`/`routes_dispatch.rs`
/// use: no filesystem access beyond this read-only `discover` walk happens
/// for an unknown project, and the real on-disk root is resolved
/// SERVER-SIDE, never trusted from the client.
async fn resolve_project_path(project: &str) -> Result<PathBuf, ApiError> {
    let name = project.to_string();
    let found = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home).into_iter().find(|p| p.name == name).map(|p| p.path)
    })
    .await
    .unwrap_or(None);
    found.ok_or_else(|| bad_request(format!("unknown project: {project}")))
}

/// RAII guard for a [`crate::server::AppState::duo_active_dirs`] entry. The
/// resolved cwd is removed on EVERY exit path this handler can take
/// (success, a downstream validation failure, a subprocess timeout, an
/// early `?` return) because this is a plain `Drop` impl rather than a
/// hand-maintained "remove it before every `return`" discipline a future
/// edit could silently break — the exact class of leak `routes_orchestrate.
/// rs`'s disconnect-cleanup functions exist to prevent for a WS stream,
/// applied here to a plain request/response handler instead.
struct CwdLockGuard {
    set: Arc<Mutex<HashSet<PathBuf>>>,
    path: PathBuf,
}

impl Drop for CwdLockGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.set.lock() {
            set.remove(&self.path);
        }
    }
}

/// Inserts `path` into `set`, returning a guard that removes it again on
/// drop. `Err(409)` — without inserting anything — when `path` is already
/// present, i.e. another `POST /v1/duo` request against the SAME resolved
/// cwd is still in flight.
fn acquire_cwd_lock(set: &Arc<Mutex<HashSet<PathBuf>>>, path: PathBuf) -> Result<CwdLockGuard, ApiError> {
    let mut guard = set.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.insert(path.clone()) {
        return Err(conflict("a duo run is already in flight against this directory"));
    }
    drop(guard);
    Ok(CwdLockGuard { set: Arc::clone(set), path })
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — the same
/// negative-PID `kill -- -<pid>` idiom as `routes_orchestrate::
/// kill_process_group`. UNLIKE `routes_new_project.rs`'s documented
/// no-op-in-practice case, this one is a REAL safety net here: `omega-duo`
/// spawns Codex/Claude as direct child processes via Node's `child_process.
/// spawn` with no `detached: true` (verified at the binary's `runProcess`,
/// line ~177) — so a nested agent turn stays in the SAME process group
/// [`run_omega_duo`] places the direct child into, and a negative-PID kill
/// reaches it too, not just the `omega-duo` process itself.
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill").arg("--").arg(format!("-{pid}")).status()
    })
    .await;
}

struct DuoOutput {
    stdout: String,
    stderr: String,
}

/// Runs `omega-duo <args>` as a killable, time-bounded async child —
/// `tokio::process::Command`, NOT this crate's usual blocking `omega_cli::
/// run` (which cannot be cancelled once spawned), mirroring `routes_pdf.
/// rs::run_omega_pdf`'s exact idiom. `process_group(0)` here is a REAL
/// cancellation mechanism, not decorative — see [`kill_process_group`]'s
/// doc comment for why this differs from `routes_new_project.rs`'s
/// documented no-op case; this endpoint is also a plain request/response
/// handler with no WebSocket to disconnect from, so the ONLY thing that
/// ever triggers a kill here is [`duo_timeout`] firing, never a client
/// disconnect.
async fn run_omega_duo(args: &[&str]) -> Result<DuoOutput, ApiError> {
    let bin = duo_bin();
    let mut cmd = tokio::process::Command::new(&bin);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.process_group(0);

    let child = cmd.spawn().map_err(|e| bad_gateway(format!("failed to spawn omega-duo: {e}")))?;
    // Captured now, before the child is ever consumed by `wait_with_output`
    // (which takes `self` by value) — `Child::id()` returns `None` once the
    // child has been polled to completion.
    let child_pid = child.id();

    match tokio::time::timeout(duo_timeout(), child.wait_with_output()).await {
        Ok(Ok(out)) => Ok(DuoOutput {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        }),
        Ok(Err(e)) => Err(bad_gateway(format!("omega-duo process error: {e}"))),
        Err(_) => {
            // The timed-out future owned `child`; dropping it here (it was
            // already dropped when the future was) triggers `kill_on_drop`
            // for the DIRECT child. That alone does not reach a nested
            // Codex/Claude turn, so also send an explicit whole-group kill
            // using the PID captured above — see [`kill_process_group`].
            if let Some(pid) = child_pid {
                kill_process_group(pid).await;
            }
            Err(gateway_timeout(format!(
                "omega-duo timed out after {}s and was killed",
                duo_timeout().as_secs()
            )))
        }
    }
}

/// `POST /v1/duo` — see this module's doc comment for the full ground
/// truth. Validates everything BEFORE any subprocess spawn, in order:
/// concurrency permit, `profile`, `prompt`, exactly-one-of `project`/`dir`,
/// the resolved target path, the per-cwd lock — then writes the scratch
/// task file and runs `omega-duo run`.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<DuoRequest>,
) -> Result<Json<DuoResponse>, ApiError> {
    // Step 0: acquire a concurrency permit as the FIRST thing this handler
    // does — the cheapest possible short-circuit, mirroring
    // `routes_pdf::create`'s / `routes_dispatch::create`'s own Step 0.
    let Ok(_permit) = state.duo_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests("too many concurrent duo runs, try again shortly"));
    };

    // Step 1: `profile` -> `--mode`, validated against the literal closed
    // set before anything else touches the filesystem or a subprocess.
    let Some(mode) = profile_to_mode(&req.profile) else {
        return Err(bad_request(format!(
            "unknown profile '{}' (expected one of: build, review, reflect)",
            req.profile
        )));
    };

    // Step 2: `prompt` — non-empty, no NUL byte, length-capped.
    if req.prompt.trim().is_empty() {
        return Err(bad_request("prompt must not be empty"));
    }
    if req.prompt.contains('\0') {
        return Err(bad_request("prompt must not contain a NUL byte"));
    }
    if req.prompt.len() > MAX_DUO_PROMPT_LEN {
        return Err(bad_request(format!("prompt too long (max {MAX_DUO_PROMPT_LEN} bytes)")));
    }

    // Step 3: exactly one of `project`/`dir`, then resolve it to a real,
    // server-validated on-disk path — no subprocess is ever spawned for an
    // ambiguous, missing, unknown, or out-of-bounds target.
    let target = match (&req.project, &req.dir) {
        (Some(_), Some(_)) => {
            return Err(bad_request("ambiguous target: give exactly one of project or dir"))
        }
        (None, None) => return Err(bad_request("no target: give project or dir")),
        (Some(p), None) => resolve_project_path(p).await?,
        (None, Some(d)) => crate::routes_sessions::dir_under_home(d)?,
    };

    // Step 4: per-cwd in-process lock, BEFORE writing the scratch file or
    // spawning anything — see [`CwdLockGuard`].
    let _cwd_guard = acquire_cwd_lock(&state.duo_active_dirs, target.clone())?;

    // Step 5: write `prompt` to a SERVER-CHOSEN scratch file — never a
    // client-supplied path reaching `--task`.
    let tasks_dir = duo_tasks_dir();
    let task_path = tasks_dir.join(format!("{}.md", crate::util::random_hex(8)));
    let task_path_str = task_path.to_string_lossy().to_string();
    let prompt = req.prompt.clone();
    tokio::task::spawn_blocking({
        let tasks_dir = tasks_dir.clone();
        let task_path = task_path.clone();
        move || -> Result<(), ApiError> {
            std::fs::create_dir_all(&tasks_dir)
                .map_err(|e| internal(format!("mkdir {}: {e}", tasks_dir.display())))?;
            std::fs::write(&task_path, prompt).map_err(|e| internal(format!("write task file: {e}")))?;
            Ok(())
        }
    })
    .await
    .map_err(|e| internal(format!("duo setup task panicked: {e}")))??;

    // Step 6: run `omega-duo run --task <scratch> --cwd <resolved> --mode
    // <mode>` — plain space-separated argv, no `--` separator and no
    // `=`-joined flags (see this module's doc comment for why both would be
    // wrong for THIS binary's own arg parser).
    let cwd_str = target.to_string_lossy().to_string();
    let output =
        run_omega_duo(&["run", "--task", &task_path_str, "--cwd", &cwd_str, "--mode", mode]).await?;

    // Step 7: parse EXACTLY one JSON line from stdout. A non-zero
    // `omega-duo` exit is NOT gated on here — see this module's doc comment
    // ("SUCCESS SIGNAL") for why the JSON body's own `ok`/`agent_ok` fields
    // are the real outcome, never this endpoint's own HTTP status.
    match serde_json::from_str::<DuoResponse>(output.stdout.trim()) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("omega-duo produced no parseable result: {e}"),
                "stdout": output.stdout,
                "stderr": output.stderr,
            })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both tests below mutate process-global env vars; without a lock they
    // race across cargo's parallel test threads (same pattern as
    // `omega_cli.rs`'s own LOCK / `routes_pdf.rs`'s own LOCK).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn duo_bin_honors_env_override() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("OMEGA_DUO_BIN", "/tmp/fake-omega-duo");
        assert_eq!(duo_bin(), PathBuf::from("/tmp/fake-omega-duo"));
        std::env::remove_var("OMEGA_DUO_BIN");
    }

    #[test]
    fn duo_bin_default_is_local_bin() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("OMEGA_DUO_BIN");
        let bin = duo_bin();
        assert!(bin.ends_with(".local/bin/omega-duo"), "expected ~/.local/bin/omega-duo, got {}", bin.display());
    }

    #[test]
    fn profile_to_mode_maps_the_skills_three_real_profiles() {
        assert_eq!(profile_to_mode("build"), Some("code"));
        assert_eq!(profile_to_mode("review"), Some("review"));
        assert_eq!(profile_to_mode("reflect"), Some("plan"));
    }

    #[test]
    fn profile_to_mode_rejects_anything_else() {
        assert_eq!(profile_to_mode(""), None);
        assert_eq!(profile_to_mode("code"), None);
        assert_eq!(profile_to_mode("plan"), None);
        assert_eq!(profile_to_mode("Build"), None);
    }

    #[test]
    fn cwd_lock_guard_removes_its_entry_on_drop() {
        let set: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let path = PathBuf::from("/tmp/some-project");
        {
            let _guard = acquire_cwd_lock(&set, path.clone()).unwrap();
            assert!(set.lock().unwrap().contains(&path));
            // A second acquire against the SAME path while the guard is
            // still alive is refused.
            assert!(acquire_cwd_lock(&set, path.clone()).is_err());
        }
        // Dropped — the entry is gone, and the path can be locked again.
        assert!(!set.lock().unwrap().contains(&path));
        assert!(acquire_cwd_lock(&set, path).is_ok());
    }
}
