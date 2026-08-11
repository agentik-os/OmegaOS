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
- [x] Wiring — server.rs route table (every new route ABOVE route_layer),
      protocol.rs schema_test, lib.rs module decls — done incrementally per
      task, each reviewed as part of it. Final review re-verified independently:
      all 39 route paths extracted from `server.rs:118-215` (including the
      multi-line `.route(` forms) are UNIQUE — zero duplicates, zero collisions
      with pre-existing routes; all 7 wave6 routes sit at lines 156-160 and
      204-207, i.e. ABOVE the `route_layer` at `server.rs:211`; only
      `/v1/health` + `/v1/pair` are outside `protected`. Proven live, not just
      read: every wave6 route returns 401 with no token AND with a garbage
      token. `AppState` is constructed ONLY via `AppState::new` (zero struct
      literals anywhere in src/ or tests/), so Task C's `started_at` field
      could not break any existing construction; `Instant` being `Copy`, the
      live `uptime_secs` tracked the real process (240s after 4 min, and reset
      to 17s after a deliberate restart).
- [x] Final opus whole-branch review — RUN THE BINARY against live requests.
      NOT CLEAN: found one real security bug the per-task reviews missed, in
      Task A's traversal guard — see below.
- [x] Runtime verify (L1): rebuild release, live-check every endpoint against
      a real paired token; capture real evidence.

## Final whole-branch review (opus) — live run against the real binary

Method: clean `cargo build --release -p omega-gateway`, then the real
`target/release/omega-gatewayd serve` started as a background process on a
free port under a fully isolated scratch env (`HOME`, `OMEGA_HOME`,
`OMEGA_GATEWAY_DIR`, `OMEGA_BACKUP_DIR` all tempdirs; `OMEGA_BIN` pointed at
the REAL `~/.local/bin/omega` so every subprocess call was real integration,
never a fake script). Real device paired the documented way
(`omega-gatewayd pair` -> 8-char code -> `POST /v1/pair` -> 64-char bearer).
`HOME` had to be overridden as well as `OMEGA_HOME`: `omega_cli::omega_bin()`,
`UsageSnapshot::read()` and `omega backup` all resolve `dirs::home_dir()`
(the real `$HOME`), NOT `OMEGA_HOME` — and `omega backup` both WRITES
`crontab.bak` into the state dir and tars it, so left on the real `$HOME` a
single `POST /v1/backup` would have written into the operator's real
`~/.omega` and archived ~16 GB of it. All three real subcommands were
dry-run under the scratch `HOME` first and proven bounded (doctor 0.21s,
audit run 0.016s, backup 0.020s / 1.6 KB) before being driven over HTTP.

### BUG FOUND + FIXED — `6725751`

`resolve_scoped_path` (`routes_files.rs:116`) still leaked an existence
oracle through 403-vs-404, one directory level ABOVE the one `6a42db8`
closed. That earlier fix classified a non-resolving leaf by canonicalizing
the leaf's immediate PARENT and nothing further, so `../<dir>/leaf` answered
403 when `<dir>` existed outside the project root and 404 when it did not.
An authenticated caller could therefore enumerate arbitrary DIRECTORY paths
anywhere on the box from the status code alone. Proven LIVE against the
running binary with a real token (this is exactly what a code re-read had
already missed twice):

    ../../../../../../../../home/vibe/.ssh/probe             -> 403
    ../../../../../../../../home/vibe/.no-such-dir-xyz/probe -> 404
    ../../../../../../../../home/vibe/.omega/secrets/probe   -> 403
    ../../../../../../../../etc/ssl/probe                    -> 403
    ../../../../../../../../etc/no-such-dir-xyz/probe        -> 404

Fix: the status may depend ONLY on whether the NEAREST EXISTING ancestor is
inside the root, never on what exists outside it — so on a leaf that does not
resolve, walk UP the ancestor chain to the first entry that DOES resolve and
classify on that (inside -> 404, outside -> 403). The walk terminates because
`joined` is absolute, so the chain ends at `/`, which always canonicalizes.
Regression tests proven fail-before/pass-after at BOTH layers (2 unit tests
failed before the fix; a third pins that the walk does not over-broaden a
legitimate in-root miss into 403, plus an HTTP-level test). Re-confirmed
LIVE on the rebuilt binary: all six probes above now return 403 uniformly,
while in-root misses (`nope.txt`, `src/nope.txt`, `no-such-subdir/nope.txt`)
still return 404.

### Live evidence, per endpoint (real bearer token, real server)

- `GET /v1/files` root -> 200, real entries incl. `.git`/`src` dirs before
  files, alphabetical within group; `path=src` -> 200 `main.rs` size 30.
- `GET /v1/files/read` -> 200 `{"content":"hello from demo-proj\nline two\n"}`.
- Traversal battery, all blocked, ZERO `/etc/passwd` content ever returned:
  `../../../etc` -> 403; absolute `/etc/passwd` -> 400 "path must not be
  absolute"; symlink `escape-link -> /etc/passwd` -> 403; URL-encoded
  `%2e%2e%2f` -> 403; `project=../../etc` -> 400 unknown project.
- `GET /v1/audits` -> 200, real 23-audit catalog over the wire, domains
  rendered as LABELS (`SEO`, `DX`, not Debug `Seo`/`Dx`).
- `POST /v1/audit` -> 200 for `codeaudit` (23 phases / 420); 400 unknown
  kind; 400 unknown project.
- `GET /v1/audit/stream` WS -> real 101 upgrade, 7 real `Line` frames from
  the real `omega audit run` subprocess + a real `Exit{success:true,code:0}`
  frame, clean close 1000. Pre-upgrade rejection verified: bad kind -> plain
  400, bad project -> plain 400, no/bad token -> 401 (never 101-then-error).
- `GET /v1/doctor` -> 200, 18 real checks, `overall:"warn"`, matching the
  real `omega doctor` run directly on this box. Exercised the real
  exit-code path and the >16-char name case live ("binary provenance",
  18 chars, parsed as one `text` with no fixed-width split).
- `GET /v1/usage` -> 200 `{"available":false}` with no cache in the scratch
  `$HOME`; after seeding a cache file, 200 `available:true` with real
  parsed fields. BOTH branches proven live.
- `GET /v1/box-info` -> 200, real `hostname` "Agentik-os", real
  `omega 0.1.9`, gateway `0.1.0`, `uptime_secs` tracking the real process.
- `POST /v1/backup` -> 200, a REAL 1688-byte tgz written under the scratch
  `OMEGA_BACKUP_DIR`; `tar tzf` confirms it archived the SCRATCH `.omega`
  only. Operator's real `~/.omega` mtime byte-identical before and after, no
  `crontab.bak` written there, no stray archive in the operator's home.
- Every route above with NO token -> 401; with a garbage token -> 401. The
  unauthenticated `POST /v1/backup` wrote no file (401 before the handler).

Gates at final HEAD, fresh: `cargo test -p omega-gateway` = **351 passed, 0
failed, 0 ignored** across 34 test binaries (347 carried in + 4 new
regression tests); `cargo clippy -p omega-gateway --all-targets --no-deps --
-D warnings` = clean, exit 0; release build clean.
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
