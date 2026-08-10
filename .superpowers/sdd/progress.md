# omega-gateway wave 5 — progress ledger

Branch: `omega-gateway-wave5` (worktree `~/.omega/worktrees/omega-gateway-wave5`)
Base: `origin/main` @ bbde842

## Plan (enumerated, operator order)

- [x] Task A — Session lifecycle: `POST /v1/sessions/{name}/close` (via `omega kill`,
      classify is_oracle + cascaded_count) + `POST /v1/sessions/{name}/rename` (rmux
      rename-session, slug-validated) — protocol.rs, routes_sessions.rs, server.rs, lib.rs —
      commit 6526261, adversarial review NOT CLEAN first pass (3 real bugs: message
      dropping REFUSED text on alias-resolution, is_oracle off unresolved name,
      valid_new_session_name admitting a rmux-trimmed leading -/.), all fixed with
      regression tests (10 added), re-verified clean (build/test/clippy green, 18+10=28
      new tests total for Task A)
- [x] Task B — Session organization overlay: `<gateway_dir>/session_org.json`
      (0600, atomic write, chat_store idiom) + `GET /v1/session-org` +
      `PUT /v1/session-org/{name}` — new session_org.rs, new routes_session_org.rs,
      protocol.rs, server.rs, lib.rs — commit a5a6012, adversarial review CLEAN
      (reviewer itself added the concurrent-write + corrupted-file regression tests,
      19 tests total for Task B)
- [x] Task C — Agent install: `POST /v1/agents/{name}/install` (validates against
      installable roster) + `GET /v1/agents/{name}/install/stream` WS (streams
      `omega install <agent>` stdout lines, fake-bin only, never a real install) —
      protocol.rs, routes_agents.rs, server.rs, lib.rs — commit ff44b97, adversarial
      review NOT CLEAN first pass (P0: disconnect only killed the direct omega child,
      not the nested bash -c installer it spawns — POSIX kill doesn't propagate to
      grandchildren), fixed with process_group(0) + group kill, regression test
      flipped from characterization to proof, re-verified clean (build/test/clippy
      green, 291 tests total across the whole crate after Task C)
- [x] Wiring — server.rs route table (ABOVE route_layer), protocol.rs schema_test,
      lib.rs module decl — done incrementally per task, each reviewed as part of it
- [x] Final opus whole-branch review — RUN THE BINARY against live requests — real
      pairing flow, real network WS, real rmux. Found P1 (commit 76a9d2b):
      install_stream_loop's disconnect detection was send-driven only (only noticed
      a dead client as a side effect of a failed send), so a nested installer that
      went quiet after the client left (the realistic case) was never killed on
      either a hard TCP RST or a clean WS close — live-reproduced via `ss -tnp`
      showing CLOSE-WAIT with the close frame unread while the orphaned installer
      ran to completion. Fixed with tokio::select! watching both the frame channel
      and the socket read side; A/B and C-happy-path were PASS on first live pass.
- [x] Runtime verify (L1): release build + live checks for all 3 features, done
      TWICE — once by the final reviewer (found the P1 above), once by the
      controller after the fix, against a fresh scratch daemon (real pairing,
      real rmux throwaway session, fake OMEGA_BIN for install): rename + close on a
      real rmux session (is_oracle:false, cascaded_count:0, session genuinely gone
      per `rmux ls`), session-org PUT+GET round-trip + on-disk 0600 file, install
      happy path (3 stdout+stderr lines + exit frame over real WS), and the P1 fix
      re-confirmed live — a silent nested installer (sleep 6 + touch marker) was
      killed within the wait window after a hard client disconnect, marker never
      created, no orphaned processes, unauth requests 401. Scratch cleaned up,
      `rmux ls` matches pre-existing baseline, prod gateway (pid 408819) untouched.
- [x] Rebase on origin/main, leave clean, report

Tasks are SERIALIZED (not parallel fan-out) because A/B/C all touch shared files
(protocol.rs, server.rs, lib.rs) — R-SCOPE (one writer per file) forbids concurrent
delegates on those. Each task still gets a fresh implementer + fresh reviewer per
the SDD contract; they just run one task at a time.

## Ground truth gathered before implementing (controller read, not guessed)

- `server.rs::build_router`: protected routes registered on the `protected` Router
  BEFORE `.route_layer(middleware::from_fn_with_state(state.clone(), require_device))`.
  A comment at that line is explicit: new protected routes go ABOVE it or they ship
  unauthenticated. `AppState.dir` IS the gateway dir (ctor arg), no separate accessor.
- `rmux.rs`: `run(args) -> Result<String>`, non-zero exit -> Err(stderr). No
  `rename-session` wrapper exists yet — must be added, mirroring `send_keys_literal`'s
  argv-only + `rmux_bin()` env-override shape. rmux SILENTLY REWRITES `:`/`.` in a
  session name to `_` (R-STREAM/rename-debug-assert-zombie doctrine) — the new_name
  validator must reject `:`/`.` outright (400) rather than let rmux silently mangle it,
  since a caller expecting the literal name back would get a different actual name.
- `routes_sessions.rs::valid_session_name`: charset alnum/_/-/. , no `/`, `..`, NUL,
  len<=200. Reused verbatim for the `{name}` path param on close/rename. For the NEW
  `new_name` (Task A rename), a stricter slug (no `.` since rmux mangles it) is needed —
  written as a new `valid_new_session_name` rather than relaxing the existing guard.
- `omega_cli.rs::run(args) -> Result<CommandOutput>`: non-zero exit is `success:false`,
  NEVER an `Err` — only a spawn failure errors. `omega_bin()` env override `OMEGA_BIN`.
- `omega-cli/src/main.rs::cmd_kill` (real behavior, confirmed by reading, not guessed):
  resolves an oracle alias first (prints `"[i] {name} resolved to the oracle session
  {resolved}"` only when it differs), classifies `is_oracle` via
  `omega_core::session::OmegaSession::classify(name).role == SessionRole::Oracle`
  (PURE, no I/O — the gateway calls this directly rather than re-deriving from CLI
  text), refuses non-force close of an oracle with live workers (bails, non-zero exit,
  `success:false`), and on proceed prints one `"  cascaded worker closed: {w}"` or
  `"  cascaded worker {w} could not be killed ({e})"` line per cascaded worker BEFORE
  the final `"Killed session: {name}"` / `"...(pane cleanup reported: {e})"` line, or
  `"Session {name} is already closed — nothing live to kill."` when nothing was live.
  Gateway parses `cascaded_count` by counting lines starting with `"  cascaded worker"`
  in stdout (best-effort — 0 is a legitimate count, not a parse failure).
- `omega_core::agents::Agent`: `from_name(&str) -> Option<Agent>` (case-insensitive),
  `install_command(&self) -> Option<&'static str>` is `None` for exactly `Kimi` and
  `Shell` — matches the brief's exclusion list precisely, so the endpoint's
  "installable" check is `agent.install_command().is_some()`, never a hand-maintained
  allowlist that could drift from the CLI's own roster.
- `crates/omega-cli/src/main.rs::cmd_install`: runs `bash -c <install_command>`,
  streaming its own stdout/stderr to the CLI's stdout/stderr (inherited, not captured) —
  so `omega install <agent>` run as a subprocess by the gateway has ITS OWN stdout
  carrying whatever the underlying installer (npm/curl|sh) printed, unstructured text,
  never a stable line format. The gateway forwards raw lines verbatim, never parses them.
- `chat_driver.rs::run_turn`: the async streaming idiom to mirror for the install WS —
  `tokio::process::Command`, `Stdio::piped()` on stdout, `kill_on_drop(true)`,
  `BufReader::new(stdout).lines()` read loop forwarding each line as a frame over an
  mpsc channel to the WS send loop, ending with a terminal frame. Never `spawn_blocking`
  + std `Command` for this one (unlike rmux.rs/omega_cli.rs's short-lived calls) because
  the process can run arbitrarily long and the WS needs to forward lines as they arrive.
- `chat_store.rs::write_meta`: atomic write idiom to mirror for `session_org.rs` —
  write to a `.json.tmp` sibling, `harden_file`, `std::fs::rename` (atomic same-fs),
  `harden_file` again on the renamed target; dir `harden_dir`'d to 0700 on open.
- Test idiom confirmed: `tokio::sync::Mutex<()>` LOCK for tests mutating an env var
  inside an async test (`session_keys_test.rs`, `dispatch_test.rs`); fake-bin scripts
  under a tempdir with `printf '%s\n' "$@"` argv capture + `--CALL--` separator for
  multi-invocation proof; `reqwest` for HTTP route tests, `tokio-tungstenite` for WS.
- `protocol.rs::Protocol` umbrella struct + `tests/schema_test.rs`'s `for ty in [...]`
  list are BOTH updated together for every new wire type, or `schema_contains_all_wire_types`
  fails.

## Log

(updated as each task closes)
