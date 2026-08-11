# omega-gateway wave7 — control-plane wave (A-E)

Merge-base: origin/main @ b642d72 (wave6 merged, ~351 tests).
Branch: omega-gateway-wave7.

Tasks A-E are SERIALIZED, not parallel fan-out: every task touches
protocol.rs, server.rs, lib.rs, schema_test.rs (R-SCOPE — one writer per
file). Each still gets a FRESH implementer + FRESH reviewer per the SDD
contract, one task at a time, its own commit(s).

## Ground truth gathered before implementing (controller read, not guessed)

- **Task A / Master chat — the mechanism is REAL but genuinely UNWIRED.**
  `omega aisb-chat` (main.rs:8747 `cmd_aisb_chat`) does NOT spawn a
  subprocess: it appends `{text, ts}` JSON to
  `~/.omega/state/aisb-local-inbox.jsonl`, then polls
  `~/.omega/state/aisb-conversation.log` for growth (up to 90s, 500ms
  ticks) and prints the delta. `aisb.rs::ensure_master` confirms the
  `aisb-master` rmux session is a PURE READ-ONLY VIEWER (`tail -F` on the
  conversation log) — "NEW MODEL (2026-05-28): the Telegram bot owns its
  own persistent Claude SDK subprocess — that is the brain." Grepped the
  ENTIRE box (crates/, ~/.omega/telegram-bot/*.ts incl. node_modules-free
  source, every .ts/.md/.toml under ~/.omega) for any reader of
  `aisb-local-inbox` / `local-inbox`: ZERO matches outside
  `cmd_aisb_chat` itself. Nothing on this box currently consumes that
  inbox file — the CLI's own local-chat feature is unwired to the real
  brain today. Decision: build the WS endpoint mirroring the CLI's exact
  file protocol (inbox write + conversation-log poll) — this is the real,
  documented, testable mechanism, and inventing a different call path
  (e.g. shelling a headless prompt) would break the "same brain, same
  response" contract and diverge further from what OmegaOS itself ships.
  Gate on `aisb-master` liveness first (never write the inbox if the
  session is dead — no auto-spawn, task said "not without care"); report
  the pre-existing wiring gap in the final report rather than papering
  over it.
- **Task B / oracle ops — `timeline` and `gate` are PURE IN-PROCESS reads,
  not CLI text to parse.** `omega_core::timeline::build(state_dir, oracle)
  -> Result<Option<OracleTimeline>>` and `omega_core::gate::{GateResult,
  Rubric}::read(state_dir, oracle) -> Result<Option<Self>>` are synchronous
  file reads already used by `cmd_timeline`/`cmd_gate` — calling them
  in-process (spawn_blocking, mirroring `routes_rules.rs` /
  `routes_oracles.rs`) is simpler and far more robust than shelling to
  `omega timeline`/`omega gate` and parsing rendered text, and several of
  the `GateResult`/`Rubric` fields already derive `Serialize`.
  `OracleTimeline`/`TimelineEvent` derive only `Debug, Clone` (no
  Serialize) — mapped field-by-field into new gateway protocol types
  rather than touching omega-core (surgical, R-KARPATHY). `reap` and
  `resurrect` remain real CLI subprocess wraps (`omega reap <session>`,
  `omega resurrect <oracle>`) — both mutate real state (scope
  claims/worktrees, spawns) so go through `omega_cli::run`, argv-only,
  fake-bin tested, NEVER run against a real session in tests or
  live-verify.
- **Task B / `orchestrate` is real, heavy, and long (`--timeout` default
  3600s), dispatches an actual oracle end-to-end.** WebSocket upgrades are
  GET-only per spec (browsers refuse to upgrade a POST) — the task brief's
  `POST /v1/orchestrate` literal path can't carry a WS upgrade, so this
  wave exposes `GET /v1/orchestrate/stream?project=&mission=&agent=`,
  mirroring `routes_audit.rs`'s `check`/`stream` pair 1:1 (project
  validated against `omega_core::projects::discover` before any spawn,
  pre-upgrade rejection on bad params, same disconnect-safe
  process-group-kill loop). Never run for real in tests/live-verify (fake
  OMEGA_BIN only) — this is exactly the "never launch a real mission in
  tests" instruction.
- **Task C / config — `ProvidersConfig` is a pure in-process typed
  store, and `omega config show` LEAKS EVERY PROVIDER API KEY IN
  PLAINTEXT** (`claude.api_key`, `codex.api_key`, `gemini.api_key`,
  `glm.api_key`, `pi.api_key`, `hermes.api_key`, `openrouter.api_key`
  — confirmed live: `omega config show` prints raw secrets for every
  configured provider). `omega_core::providers::ProvidersConfig::load()`
  /`::save()` (omega-core/src/providers.rs) are the real in-process
  read/write the CLI itself calls (`cmd_config`); `set_config_value`'s
  match arms are the authoritative key allowlist. SECURITY DECISION
  (beyond the literal task text, and load-bearing): `GET /v1/config`
  NEVER returns raw `api_key` values — every `*.api_key` field is
  redacted to a boolean `set: bool` (non-empty vs empty), never the
  secret itself, even though the caller is already device-authenticated.
  Rationale: a stolen/leaked device token would otherwise hand over
  EVERY provider credential on the box in one GET, a blast radius wildly
  disproportionate to what an app UI needs (it needs to know a key IS
  configured, never the value) — same posture as R-SECRETS-VAULT/R-ENV.
  `PUT /v1/config` still accepts `api_key` values (write-only — a client
  can SET a key it already possesses, it just can never READ one back
  over this API) and is validated against the same key allowlist
  `set_config_value` uses, in-process, then `cfg.save()` — no CLI
  subprocess at all for Task C. Hermetic test: `ProvidersConfig::path()`
  is `crate::config::omega_dir()` which DOES honor `$OMEGA_DIR` (confirmed
  in omega-core/src/config.rs), so tests set `OMEGA_DIR` to a scratch dir
  under a LOCK-guarded mutex like every other global-env test in this
  crate.
- **Task D / telegram — `OmegaTelegramConfig` is also a pure in-process
  typed store, no CLI shelling needed**, and it ALSO holds a plaintext
  secret (`bot_token`) that must never round-trip to a GET caller.
  `omega_core::monitor::OmegaTelegramConfig::{read,write}()` (used
  directly by `cmd_telegram`'s Status/Enable/Disable arms) hardcode
  `dirs::home_dir()` (NOT `$OMEGA_DIR`/`$OMEGA_HOME` — confirmed reading
  `monitor.rs::OmegaTelegramConfig::path()`), so hermetic tests override
  the `$HOME` env var itself, LOCK-guarded, same pattern wave6 used for
  `UsageSnapshot`. `GET /v1/telegram/status` redacts `bot_token` to a
  `set: bool` exactly like Task C's `api_key`; enable/disable read, flip
  `enabled`, write — in-process, no subprocess, so "fake-bin tested" from
  the task brief doesn't apply here (there is no bin to fake) — instead
  enable/disable are tested against a scratch `$HOME`, and live-verify
  reads the REAL status (safe, read-only) but NEVER calls enable/disable
  against the operator's real `~/.omega/telegram.toml`.
- **Task E / pdf — real CLI wrap, no shortcuts available.** `omega pdf
  --template=<t> --data=<path> --out=<path> [--send] [--caption=]` has no
  in-process library entry point in this workspace (pdfgen is its own
  tool under `tools/pdfgen/`, invoked as a subprocess by the CLI itself)
  — `POST /v1/pdf` shells to `omega_cli::run`, argv-only. `template` is
  validated against the literal known set (`whitepaper|audit|marketing|doc`)
  before spawning. `data` is client-supplied JSON: written to a
  SERVER-CHOSEN scratch path (never a client-supplied path passed to
  `--data`) mirroring `routes_box::backup`'s server-chosen-`--out`
  posture. `GET /v1/pdf/download?path=` is scoped to the pdf OUTPUT dir
  only (same canonicalize-and-prefix-check idiom `routes_files.rs`
  already carries, including its ancestor-walk fix for the
  outside-root-403-vs-404 leak wave6 found) — never an arbitrary
  filesystem path. `--send` is NEVER passed by this endpoint (it would
  push to the operator's real Telegram from an API call with no
  operator-side confirmation) — this endpoint only generates + returns a
  path/download link; sending stays a CLI/operator action.

## Task A — done

Built `GET /v1/master/chat` in a new `crates/omega-gateway/src/routes_master.rs`,
wired above the `route_layer` line in `server.rs` and added to `lib.rs`. The
handler mirrors `omega aisb-chat`'s (`cmd_aisb_chat`) exact file protocol: on
each inbound client text message it (1) checks `aisb-master` rmux-session
liveness via `rmux::list_sessions()` (spawn_blocking, same idiom
`routes_oracles::list` uses) — if not live, sends a `NotRunning` frame and
never touches the inbox, then keeps the loop open for more messages rather
than closing the socket; (2) if live, appends `{text, ts}` JSON to
`~/.omega/state/aisb-local-inbox.jsonl` (creating the file/parent dir if
missing), records the pre-append conversation-log byte length, then polls
`~/.omega/state/aisb-conversation.log` for growth (180 attempts * 500ms =
90s by default, overridable via `OMEGA_AISB_POLL_ATTEMPTS` /
`OMEGA_AISB_POLL_INTERVAL_MS` for tests); (3) on growth, sends `Reply{text}`
with the delta (leading newline noise trimmed); on no growth within budget,
sends `Timeout`. Both filesystem paths resolve via `dirs::home_dir()`
falling back to raw `$HOME`, deliberately NOT `crate::config::home_dir()`
(which additionally honors `$OMEGA_DIR`/`$OMEGA_HOME`) — a distinct
resolution kept intentionally duplicated in `routes_master.rs` rather than
reused, so this endpoint watches/writes the exact files a real `aisb-master`
setup uses, bit for bit matching the CLI. Added `MasterChatMsg` to
`protocol.rs` (`NotRunning` / `Reply{text}` / `Timeout`, same
`#[serde(tag="type", rename_all="snake_case")]` shape as `AuditStreamMsg`),
into the `Protocol` umbrella struct, and into `tests/schema_test.rs`'s type
list.

TDD: wrote the WS integration tests first (they compiled against the not-yet-
existing route and failed to build), then implemented `routes_master.rs` to
make them pass. Test delta: **+10** (6 unit tests inside `routes_master.rs`
covering `read_growth`/`log_len`/`append_to_inbox` edge cases — missing log,
no growth, delta trim, parent-dir creation, multi-line append; 4 WS
integration tests in the new `tests/master_chat_test.rs` — not-running never
touches the inbox, a live master's round trip writes the inbox and returns
the right `Reply` text, a live master that never grows the log times out
within a short overridden budget, and the pre-upgrade 401 with no token).
Crate total after Task A: **362 tests, 0 failed, 0 ignored**
(`cargo test -p omega-gateway`). `cargo clippy -p omega-gateway --all-targets
--no-deps -- -D warnings` clean.

Judgment calls beyond the literal brief: (1) on `NotRunning`, the loop stays
open (doesn't close the socket) so a client watching the master start up
mid-session doesn't need to reconnect — the brief allowed either choice and
named this the "simplest and safest" one. (2) `read_growth` returns `None`
(treated as "poll again") both when the log hasn't grown AND when it exists
but fails to parse as UTF-8 — never a crash/panic, per the brief's explicit
instruction. (3) Env-var poll overrides parse with `.ok().and_then(|v|
v.parse().ok())`, so a garbage value falls back to the CLI's real default
rather than erroring — matches this crate's general "degrade gracefully,
never 500 on a bad env var" posture (e.g. `routes_oracles::list`'s rmux
failure handling).

Honest gap, restated: nothing on this box currently reads
`aisb-local-inbox.jsonl` — confirmed by grepping every Rust crate and the
Bun Telegram bot's brain source before implementation started (see the
ground-truth section above). A real client hitting this endpoint today will
always receive `Timeout` (assuming `aisb-master` is even running), exactly
like the CLI's own local REPL does right now. This is a pre-existing wiring
gap in what OmegaOS ships, not a defect in this endpoint — the mechanism
built here is byte-for-byte the real, documented protocol, ready to work the
moment something starts consuming that inbox file.

### Task A — adversarial review-fix round

An independent adversarial reviewer found 3 real issues in the Task A code;
fixed with TDD (failing/reproducing test first where practical), each
surgical:

1. **UTF-8 char-boundary panic in `read_growth`** — `start` comes from a
   byte-length snapshot (`before_len`) taken on an EARLIER read; if the log
   is ever truncated and rewritten before the next read (a rotation, not the
   normal append-only path), a stale `before_len` can land mid-character and
   `content[start..]` panics. Fixed with `content.get(start..)`, treating
   `None` (non-boundary or out-of-range) the same as "no growth yet" — the
   exact idiom `routes_box.rs::parse_doctor_output` already uses for
   adversarial subprocess output. Regression unit test:
   `read_growth_none_on_stale_snapshot_mid_char_boundary` (log = `"aé"`,
   `before_len = 2`, a byte index inside `é`'s 2-byte encoding) — panicked
   before the fix, returns `None` after.
2. **No concurrency cap on `/v1/master/chat`** — any authenticated device
   could open unboundedly many concurrent WebSockets, each holding a
   connection for up to a 90s round trip and firing a `spawn_blocking` task
   every ~500ms tick. Added `AppState::master_chat_permits` (new
   `MAX_CONCURRENT_MASTER_CHATS = 4` in `server.rs`, mirroring
   `dispatch_permits`'s heavier/longer-held reasoning). `chat` now acquires
   ONE permit for the whole connection lifetime BEFORE upgrading (mirrors
   `routes_audit.rs::stream`'s reject-before-upgrade branch between
   `ws.on_upgrade(...)` and `code.into_response()`) and returns a bare 429 on
   exhaustion instead of upgrading; the permit moves into `master_chat_loop`
   and releases on drop. Test:
   `concurrency_cap_returns_429_when_master_chat_permits_exhausted` (4 live
   round-trips held open on a long poll budget, 5th connection attempt
   rejected with 429).
3. **No length cap on the inbound WS message** — unlike every other free-text
   input in this crate (`MAX_MISSION_LEN`, `MAX_SEND_KEYS_BYTES`), nothing
   bounded the client's WS text before it was JSON-encoded into the inbox
   file. Added `MAX_MASTER_CHAT_MESSAGE_LEN = 8000` in `routes_master.rs`; an
   oversized message never touches the inbox and gets a new `MasterChatMsg::
   Error{message}` frame (added to `protocol.rs`, already covered by
   `Protocol`'s single `master_chat_msg` field and `schema_test.rs`'s single
   `"MasterChatMsg"` entry — no per-variant wiring needed), and the loop
   stays open for the next message (same "don't close over one bad message"
   posture as `NotRunning`). Test:
   `oversized_message_rejected_with_error_frame_and_inbox_untouched` (8001-byte
   message → `Error` frame + untouched inbox, then a normal follow-up message
   on the same socket still gets served).

Test delta: **362 → 365** (+3: 1 unit regression test in `routes_master.rs`,
2 WS integration tests in `tests/master_chat_test.rs`).
`cargo test -p omega-gateway`: 365 passed, 0 failed, 0 ignored.
`cargo clippy -p omega-gateway --all-targets --no-deps -- -D warnings`: clean.

## Status
- [x] Task A — Master/AISB-chat WS
- [ ] Task B — Oracle mission ops (orchestrate/reap/resurrect/timeline/gate)
- [ ] Task C — Config GET/PUT
- [ ] Task D — Telegram bridge control
- [ ] Task E — PDF generation
- [ ] Final opus whole-branch live-binary review
- [ ] Rebase on origin/main, leave clean, report
