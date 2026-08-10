# omega-gateway wave 4 — progress ledger

Branch: `omega-gateway-wave4` (worktree `~/.omega/worktrees/omega-gateway-wave4`)
Base: `origin/main` @ c8ea843

## Plan (enumerated, operator order)

- [x] Task A — Live color terminal stream + interactive key input (routes_sessions.rs, rmux.rs, protocol.rs) — commit e4be4be, adversarial review CLEAN
- [x] Task B — Chat history pagination at scale (chat_store.rs, routes_chat.rs, protocol.rs) — commit ab9f28b, adversarial review CLEAN (2 nits recorded, non-blocking)
- [x] Task C — Deposit files from the app (new routes_deposit.rs + deposit.rs, protocol.rs) — commit 5a08a99, adversarial security review CLEAN (3 nits recorded, non-blocking, inherited from the reference bot)
- [x] Task D — Dispatch hardening (routes_dispatch.rs) — commit abf0f33, adversarial review CLEAN (3 nits recorded, non-blocking), concurrency test stress-tested 25x with 0 flakes
- [x] Wiring — server.rs route table, protocol.rs schema_test, lib.rs module decl (done incrementally per task, each schema_test/route addition reviewed as part of its task)
- [x] Final opus whole-branch review — first pass: NOT READY, found 4 blocking bugs via live runtime testing (multipart 2MiB axum wall silently defeating the 50MiB cap; same-second filename collision destroying both payloads; secret-detection truncation-order bypass; send-keys dash-prefix argv bug recurring from task D). All 4 fixed in commit 4223d14, each fix TDD'd (regression test proven to fail pre-fix, pass post-fix) and independently re-verified live by the controller against the real release binary + real rmux CLI (not just unit tests).
- [x] Runtime verify (L1): release build + live checks for all 4 features, twice — once per-task with fake rmux/omega, once after the bug-fix pass with a fresh scratch env, including a direct real-rmux comparison for B4 (old argv shape errors, new shape exits 0)
- [x] Rebase on origin/main, leave clean, report

Tasks are SERIALIZED (not parallel fan-out) because A/B/C/D all touch shared files
(protocol.rs, server.rs, lib.rs) — R-SCOPE (one writer per file) forbids concurrent
delegates on those. Each task still gets a fresh implementer + fresh reviewer per
the SDD contract; they just run one task at a time.

## Ground truth gathered before implementing

- rmux capture-pane: `run(&["capture-pane", "-p", "-t", session, "-S", &start])`.
  ANSI mode adds `-e` per the mission brief. `rmux_bin()` env override `OMEGA_RMUX_BIN`.
- routes_sessions stream: R-STREAM loop, never exits except dead socket; per-frame
  diff dedupe against `last`.
- Chat WS: `ChatStreamServerMsg`/`ClientMsg` tagged enums, `valid_chat_id` (16 lowercase
  hex) guard reused from routes_chat.rs — same guard needed for the new REST routes.
- Deposit ground truth: `~/.omega/telegram-bot/inbox-bot.ts` — SECRETISH regex
  `(\.(p8|pem|key|env|p12|jks|keystore|ppk|crt|pfx)$)|(^|[._-])(id_rsa|id_ed25519)|credential|secret|token|passwd|private[._-]?key`,
  boxes `["Home","AltReality","Omega","Box"]`, hardlink-then-copy-fallback `place()`,
  original kept in `~/.omega/inbox/`, `fanout_secrets=true` or `--force`/`!share` override,
  `~/.omega/deposit.toml` fanout config. Gateway equivalent: `OMEGA_DEPOSIT_DIR` env
  override (mirrors `OMEGA_GATEWAY_DIR`/`OMEGA_HOME` pattern in config.rs) so tests never
  touch the real `~/.omega/deposit`.
- Dispatch hardening ground truth: `routes_dispatch.rs` already validates project via
  `omega_core::projects::discover`; roster validation should use
  `omega_core::agents::Agent::all()` (same source `routes_agents.rs` uses) BEFORE spawn.
  Chat concurrency pattern to mirror: `AppState.chat_permits: Arc<Semaphore>` sized
  `MAX_CONCURRENT_CHAT_TURNS`, `try_acquire_owned()` -> busy short-circuit.
- Test idiom: `static LOCK: tokio::sync::Mutex<()>` (async tests touching env vars) or
  `std::sync::Mutex<()>` (sync tests), fake-bin scripts with `#!/usr/bin/env bash`,
  argv capture files, `install_fake_home`/`install_fake_omega`/`install_fake_rmux` helpers.

## Log

(updated as each task closes)
