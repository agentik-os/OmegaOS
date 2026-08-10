# SDD Progress — omega-gateway-surface

Plan: docs/superpowers/plans/2026-08-10-omega-gateway-omega-surface.md

## Status
- [x] Task 1: Add omega-core dependency, prove clean link — commit dc39327
- [x] Task 2: GET /v1/rules — commit 237e867, reviewed CLEAN
- [x] Task 3: GET /v1/agents — commit e3772bd, reviewed CLEAN
- [x] Task 4: GET /v1/skills — commit b0d2346, reviewed CLEAN (380 skills live-verified)
- [x] Task 5: GET /v1/projects — commit 2000ccc, reviewed CLEAN (49 projects, path/score confirmed absent)
- [x] Task 6: omega_cli.rs subprocess wrapper — commit 68a81ae, reviewed CLEAN (argv-only, non-zero exit != Err verified)
- [x] Task 7: GET /v1/oracles — commit 233ad7a, reviewed CLEAN (key=session=ledger's "oracle" field verbatim, no double-prefix; live oracle-dentistrygpt-3 confirmed 14/15)
- [x] Task 8: POST /v1/dispatch — commit bad9510, reviewed CLEAN (security: argv-only proven with live hostile-payload injection test, validate-before-spawn proven live with unreachable OMEGA_BIN, exact-string project match, auth-gated). Sanctioned cross-task touch: config.rs (new home_dir() override) + routes_projects.rs (call-site swap only, Task 5 behavior unchanged).
- [x] Task 9: Final wiring + live-daemon verification pass — controller-run, no fix needed, no empty commit

Execute task-by-task per subagent-driven-development, TDD (failing test first), runtime-verify each endpoint against the live daemon before marking a task done.

## Task 1 — done
Commit dc39327. Build-time delta: 10.81s (before) -> 17.48s clean gateway build with
omega-core's own tree pre-built (~6.7s steady-state added cost); no-op incremental
rebuild unaffected (0.14s). cargo test -p omega-gateway: 149 passed. Assessment:
bounded/acceptable, no operator escalation needed on build cost.

**Clippy gate correction (controller-verified, L1):** `cargo clippy -p omega-gateway
--all-targets -- -D warnings` (the plan's literal command) now fails — NOT from any
code this plan writes, but because clippy's `-D warnings` extra-rustc-args apply to
the whole compiled unit graph in that invocation, and `omega-core` carries ~64
pre-existing clippy findings (manual_strip, trim_split_whitespace, unnecessary_map_or,
etc. in skill_registry.rs/sysinfo.rs/session.rs) that were never linted before because
nothing in omega-gateway's dependency graph compiled omega-core under clippy. Fixing
omega-core's own debt is out of this plan's file scope (R-KARPATHY surgical changes,
R-SCOPE) and not something Task 1 should silently absorb.
**Fix (verified clean, 7.54s):** every task from here on runs clippy as
`cargo clippy -p omega-gateway --all-targets --no-deps -- -D warnings` — `--no-deps`
scopes clippy's lints to the omega-gateway crate only (still compiles omega-core
normally, just doesn't lint it), which is the standard, minimal, non-source-touching
fix. This does not lower the quality bar for anything this plan writes; it only stops
unrelated pre-existing dependency debt from blocking this plan's own gate. Recorded
here transparently per L5 (narrow scope, don't silently lower the floor). Flagging
omega-core's own clippy debt as a separate future cleanup item for the operator.

## Task 9 — done (controller-run)
`server.rs` re-read top to bottom: all 6 new routes (`/v1/rules`, `/v1/agents`,
`/v1/skills`, `/v1/projects`, `/v1/oracles`, `/v1/dispatch`) sit inside the
`protected` block, strictly above `.route_layer(...require_device)`; none landed
in the pre-guard `/v1/health`/`/v1/pair` block. `cargo test -p omega-gateway`:
172 passed, 0 failed. `cargo clippy -p omega-gateway --all-targets --no-deps --
-D warnings`: clean. `cargo build --release -p omega-gateway`: succeeds, 1m36s.

Full live-daemon pass (scratch `omega-gatewayd` release binary, real `$HOME`,
temp `OMEGA_GATEWAY_DIR`, paired a fresh device):
- `/v1/rules` -> 7 laws, 52 rules, L0 and R-CLI present.
- `/v1/agents` -> 8 agents, claude available:true.
- `/v1/skills?q=audit&limit=3` -> 3 results, total:382, all match "audit".
- `/v1/projects` -> 49 projects, first entry Kommu/Station/SideBusiness, no
  path/score leaked.
- `/v1/oracles` -> 1 live entry, oracle-dentistrygpt-3, 14/15 tasks, matches
  the real on-disk ledger.
- `/v1/dispatch` unknown-project POST (OMEGA_BIN pointed at an unreachable
  path as a safety guard) -> real 400 `{"error":"unknown project: ..."}`,
  never touched OMEGA_BIN (proves validate-before-spawn holds against a real
  release binary, not just the test harness). No-auth POST -> 401.
No real oracle was ever dispatched during verification. Scratch dir + daemon
process cleaned up after the pass.
Step 1 found nothing to fix -> no empty Task 9 commit, per the plan's own
Step 4 fallback.
