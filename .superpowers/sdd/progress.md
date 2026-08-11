# omega-gateway wave 6 — progress ledger

Branch: `omega-gateway-wave6` (worktree `~/.omega/worktrees/omega-gateway-wave6`)
Base: `origin/main` @ e40a904

## Plan (enumerated, operator order)

- [x] Task A — Files browse/read: `GET /v1/files` + `GET /v1/files/read`,
      scoped to one discovered project's path via `project=` query param
      (never `$HOME`), traversal guard = component-join + canonicalize +
      `starts_with(root)`. Implementer commit `80df155` (22 tests, build/
      test/clippy green). Adversarial review NOT CLEAN first pass: real
      info-leak bug — `resolve_scoped_path` canonicalized the leaf itself
      before classifying Escaped(403)/NotFound(404), so a traversal to an
      EXISTING outside file returned 403 while the SAME traversal to a
      nonexistent outside file returned 404 — an authenticated caller could
      enumerate arbitrary-file existence on the box from the status code
      alone, without reading any content. Fixed in `6a42db8` (falls back to
      canonicalizing the leaf's PARENT when the leaf itself doesn't resolve,
      so escape-via-existing-parent is still 403 regardless of leaf
      existence): regression test proven fail-before/pass-after. Re-verified
      clean: build/test/clippy green, 315 tests total in the crate.
- [x] Task B — Audit (Quality Arsenal): `GET /v1/audits` (catalog, in-process
      from `omega_core::audit::all_audits()`, never CLI-parsed) + `POST
      /v1/audit` (pre-flight validate project+kind, mirrors
      `routes_agents::install_check`) + `GET /v1/audit/stream` WS (runs
      `omega audit run <kind> --dir <project_path>`, mirrors
      `install_stream_loop`'s disconnect-safe process-group-kill contract).
      Implementer commit `26749d0` (13 tests). Adversarial review CLEAN — no
      fix needed. Reviewer independently verified: project path passed to
      `--dir` comes only from server-side `discover()`, never client input;
      both unknown-kind/unknown-project are rejected BEFORE the WS upgrade
      (plain 400, never 101-then-error); both new routes sit above
      `route_layer`; the silent-disconnect process-group kill test was
      proven non-vacuous (broke `kill_process_group` -> test failed with the
      marker written; restored -> passed, working tree byte-identical after);
      `AuditDomain::label()` used correctly (SEO/DX, not raw Debug Seo/Dx).
      Also resolved a test-count discrepancy from the implementer's own
      report (233) — a fresh full `cargo test -p omega-gateway` reproducibly
      returns 328 (315 carried from Task A + 13 new), confirmed by two
      independent counting methods; 233 came from a scoped/partial run, not
      the real total. Crate total after Task B: 328 tests.
- [x] Task C — Box health + usage + backup: `GET /v1/doctor` (shells to bare
      `omega doctor`, no `--fix`/`--deep`; parses the two-space-indented
      check-line format WITHOUT a fixed-width name/detail split — proven
      against a real >16-char check name), `GET /v1/usage` (in-process
      `omega_core::monitor::UsageSnapshot::read()`, zero subprocess, `$HOME`
      env-override tested), `GET /v1/box-info` (hostname + `omega --version`
      + gateway `CARGO_PKG_VERSION` + process uptime via `AppState.started_at:
      Instant`), `POST /v1/backup` (the one mutating endpoint — takes ZERO
      client input, server picks `<OMEGA_BACKUP_DIR|tmpdir>/omega-gateway-
      backup-<ns-precision-ts>.tgz` itself). Implementer commit `40db4fb`
      (16 tests). Adversarial review NOT CLEAN first pass: `parse_doctor_
      output` panicked on a check line whose glyph slot held a multi-byte
      UTF-8 char (raw `&s[0..3]` byte-slice on a non-char-boundary — reachable
      from adversarial/malformed `omega doctor` stdout, would have dropped
      the client connection rather than degrading to 502). Fixed in `c707a6b`
      (`.get(0..3)` instead of a raw index range, skips the line instead of
      panicking): regression proven fail-before/pass-after at both the unit
      and HTTP-integration layer (second request on the same server still
      succeeds post-fix, proving the process itself survives either way).
      Reviewer also independently judged the backup-path symlink-race
      question theoretical-not-realistic for this single-tenant local
      gateway (would need an attacker to predict a nanosecond-precision,
      request-time filename before an authenticated POST fires) and confirmed
      cross-test-binary env isolation is a non-issue (`cargo test` runs each
      `tests/*.rs` file as its own OS process). Crate total after Task C:
      347 tests.
- [ ] Wiring — server.rs route table (every new route ABOVE route_layer),
      protocol.rs schema_test, lib.rs module decls — done incrementally per
      task, each reviewed as part of it.
- [ ] Final opus whole-branch review — RUN THE BINARY against live requests.
- [ ] Runtime verify (L1): rebuild release, live-check every endpoint against
      a real paired token; capture real evidence.
- [ ] Rebase on origin/main, leave clean (no merge/push), report.

Tasks are SERIALIZED (not parallel fan-out) because A/B/C all touch shared
files (protocol.rs, server.rs, lib.rs, schema_test.rs) — R-SCOPE (one writer
per file) forbids concurrent delegates on those, same reasoning wave5 recorded.
Each task still gets a fresh implementer + fresh reviewer per the SDD
contract; they just run one task at a time, each its own commit.

## Ground truth gathered before implementing (controller read, not guessed)

- `server.rs::build_router`: every protected `.route(...)` MUST be registered
  on `protected` BEFORE `.route_layer(middleware::from_fn_with_state(...,
  require_device))` or it ships unauthenticated — comment at that line is
  explicit.
- `omega audit list` has NO `--json` flag (confirmed: `--json` errors
  "unexpected argument"). `omega_core::audit::all_audits() -> Vec<AuditSkill>`
  (id, name, domain: AuditDomain, phases, max_score, read_only) is the real
  in-process registry (parsed once from `skills/audits/registry.toml`,
  cached in a `OnceLock`) — `GET /v1/audits` calls this DIRECTLY, exactly the
  idiom `routes_rules::list` already uses for `omega_core::rules`, never a
  CLI subprocess.
- `omega audit run <id> --dir <dir>` does NOT run the audit — it prints the
  audit's metadata + the `omega spawn-worker` command an operator would run
  to actually dispatch it (confirmed by running it live: prints "Audit:
  ...", "Phases: ...", "To dispatch as a worker session: omega spawn-worker
  ..."). So the WS stream this wave adds is real infrastructure (mirrors
  `install_stream_loop`'s process-group-kill contract faithfully) but the
  underlying command it drives is fast and side-effect-free, not a real
  multi-minute audit run — that stays a `spawn-worker` dispatch, out of scope
  for this wave.
- `omega doctor` has options `--pre-reset` / `--fix` / `--deep` / `-h` —
  `--json` does NOT exist. `--fix` mutates, `--deep` burns quota on a live
  Codex auth check — `GET /v1/doctor` MUST run bare `omega doctor` only.
  Real stdout format (from `crates/omega-cli/src/main.rs::cmd_doctor`):
  `println!("  {} {:16} {}", c.health.glyph(), c.name, c.detail);` — glyph is
  exactly `[+]`/`[!]`/`[x]` (`omega_core::doctor::Health::glyph()`). Check
  lines are indented by EXACTLY two spaces before the glyph; the trailing
  overall-summary line (`"[+] all systems healthy"` etc.) has ZERO leading
  spaces — that is the unambiguous parser discriminant, since `{:16}` does
  NOT truncate/delimit a `c.name` longer than 16 chars (e.g. "binary
  provenance" is 18 chars, so name and detail are separated only by the
  format string's single literal space, not a fixed column) — never split
  name/detail on a fixed-width assumption; the parser must capture
  `{health, text}` per checked line (glyph + full remainder), and derive
  `overall` by aggregating check healths (mirrors
  `omega_core::doctor::overall()`), never by re-parsing the trailing summary
  line. `omega doctor` exits 1 on Fail (`std::process::exit(1)` in
  `cmd_doctor`) — that is a NORMAL outcome (matches `omega_cli::run`'s own
  non-zero-exit-is-not-an-error philosophy), never treated as a spawn/502
  error by this endpoint.
- `omega usage` (no flags) is a passive cache read: `omega_core::monitor::
  UsageSnapshot::read() -> Result<Option<Self>>` reads
  `~/.omega/state/usage.json`, zero network, zero subprocess.
  `GET /v1/usage` calls this directly in-process (spawn_blocking), never
  shells to `omega usage`. `--check` (live OAuth fetch + Telegram alert) is
  explicitly OUT OF SCOPE — it has side effects (an alert send) a passive GET
  must never trigger. NOTE: `UsageSnapshot::omega_usage_path()` hardcodes
  `dirs::home_dir()` (NOT `OMEGA_HOME`/`OMEGA_STATE_DIR`) — a hermetic test
  overrides via the `HOME` env var itself (Linux `dirs` reads `$HOME`),
  LOCK-guarded like every other global-env test in this crate.
  `UsageSnapshot` has no data when the cache file is absent (`None`) — the
  endpoint must render that as a real, structured "no data yet" response,
  never a 404/500.
- `omega backup [--out PATH] [--include-memory]` archives `~/.omega` +
  crontab to a `.tgz`; real, non-destructive (reads only, writes one new
  file), but a real side effect — `POST /v1/backup` always passes `--out` at
  a caller-chosen/temp path in tests, NEVER runs unbounded against the
  operator's real `~/.omega` inside a test.
- `omega --version` prints `"omega 0.1.9\n"` — `GET /v1/box-info` returns
  this trimmed verbatim (no brittle parse) alongside the gateway's own
  `CARGO_PKG_VERSION`, `hostname` (shelled — no hostname crate in this
  crate's Cargo.toml, matches the existing shell-out convention), and a
  process-uptime computed from a `once_cell`/`OnceLock<Instant>` captured at
  gateway startup.
- `routes_dispatch.rs`'s project-allowlist pattern (validate `project` against
  `omega_core::projects::discover(&home)` BEFORE any subprocess/filesystem
  touch beyond the read-only discover walk) is reused verbatim by Task A
  (files root) and Task B (audit project dir) — `DiscoveredProject::path` is
  the per-project root, never `$HOME` itself (browsing all of `$HOME` would
  reach `~/.ssh`, `~/.omega/secrets`, violating R-ENV).
- `routes_agents.rs::install_stream_loop` is the canonical disconnect-safe
  WS-streams-a-subprocess pattern (process_group(0) + kill_on_drop + BOTH
  `tokio::select!` arms — the mpsc frame channel AND the socket's own read
  side — must be watched, a send-failure-only disconnect check misses a
  client that goes quiet, see wave5's P1 fix) — Task B's audit-stream mirrors
  its STRUCTURE (same discipline `omega_cli.rs` documents mirroring
  `rmux.rs`'s shape while deviating on error semantics), not a forced shared
  abstraction over already-shipped, tested code.
