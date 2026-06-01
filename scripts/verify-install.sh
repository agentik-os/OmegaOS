#!/usr/bin/env bash
# verify-install.sh — INSTALL PARITY gate (CLAUDE.md Law 0).
#
# Run after ANY OmegaOS improvement. Fails if a fresh
# `git clone … && ./install.sh` would NOT reproduce the current system, or if
# a secret leaked into the repo. Binary changes ship automatically because
# install.sh builds from source; this guards the things that DON'T:
# new assets, uncommitted work, unpushed commits, and secrets.
#
# Usage:  ./scripts/verify-install.sh
# Exit 0 = install is complete and safe. Non-zero = fix before declaring done.

set -u
cd "$(dirname "$0")/.." || exit 2
fail=0
ok()   { printf '  \033[32m✓\033[0m %s\n' "$1"; }
bad()  { printf '  \033[31m✗ %s\033[0m\n' "$1"; fail=1; }

echo "═══ OmegaOS install-parity check ═══"

# 1. install.sh builds the binary from source (so all Rust features ship).
if grep -q "cargo build --release" install.sh && grep -q "target/release/omega" install.sh; then
  ok "binary built from source (all Rust features ship automatically)"
else
  bad "install.sh does NOT build omega from source"
fi

# 2. Agent prompts (oracle.md v2 etc.) are installed.
if grep -q "agents/\*.md" install.sh; then ok "agents/*.md installed"; else bad "agents/*.md not copied by install.sh"; fi

# 3. Slash commands installed.
if grep -q "omega-\*.md" install.sh; then ok "omega-* slash commands installed"; else bad "omega-* commands not copied"; fi

# 4. Full-scrollback retention persisted in the sourced rmux config.
if grep -q "history-limit" config/rmux.conf.omega 2>/dev/null; then ok "rmux history-limit persisted"; else bad "history-limit missing from rmux.conf.omega"; fi

# 4b. Alt+Up/Down scroll bindings use the quoted-string if-shell form. rmux 0.3.1
#     rejects the brace `{ … }` command-list form ("bind-key does not accept a
#     parsed command-list argument") and silently drops the binding — so scroll
#     never works on a fresh install. Guard against the brace form regressing.
if grep -qE "bind-key -n M-Up .*\{ *send-keys" config/rmux.conf.omega 2>/dev/null; then
  bad "rmux Alt+Up uses brace syntax rmux 0.3.1 rejects (use quoted if-shell args)"
elif grep -qE "bind-key -n M-Up +if-shell .*'send-keys -X scroll-up'" config/rmux.conf.omega 2>/dev/null; then
  ok "rmux scroll bindings use rmux-0.3-compatible quoted syntax"
else
  bad "rmux Alt+Up/Down scroll bindings missing from rmux.conf.omega"
fi

# 4c. Status-bar clock is localized via `omega clock` (not the rmux server's
#     UTC strftime), so the bar shows the operator's wall time on a headless VPS.
if grep -q "omega clock" config/rmux.conf.omega 2>/dev/null; then ok "rmux status clock localized via omega clock"; else bad "rmux status clock not wired to omega clock"; fi

# 4d. timezone knob documented for the operator in the shipped config template.
if grep -q "^# timezone" config/default.toml 2>/dev/null; then ok "timezone config documented in default.toml"; else bad "timezone knob missing from default.toml"; fi

# 5. Self-improvement crons scheduled.
if grep -q "omega patrol" install.sh; then ok "patrol/usage crons scheduled"; else bad "crons missing from install.sh"; fi

# 6. NO SECRETS tracked. Telegram-token pattern + hardcoded bot_token + .omega ignored.
if git grep -nIE "[0-9]{9,10}:[A-Za-z0-9_-]{30,}" -- . >/dev/null 2>&1; then
  bad "possible token committed to the repo!"
else
  ok "no telegram-token pattern in tracked files"
fi
if grep -qE "^\.omega/|^\.omega$" .gitignore 2>/dev/null; then ok ".omega/ (secrets) gitignored"; else bad ".omega/ not gitignored"; fi

# 7. Working tree clean → a fresh clone is complete.
if [ -z "$(git status --porcelain)" ]; then ok "working tree clean (fresh clone = complete)"; else bad "uncommitted changes — a fresh install would NOT get them"; fi

# 8. Local branch in sync with origin (latest pushed).
git fetch -q origin 2>/dev/null || true
if [ -n "$(git rev-parse @ 2>/dev/null)" ] && [ "$(git rev-parse @ 2>/dev/null)" = "$(git rev-parse @{u} 2>/dev/null)" ]; then
  ok "local in sync with origin (latest pushed)"
else
  bad "local commits not pushed to origin"
fi

# 9. Self-containment assets (reset-survival): persistent service, hooks, identity.
if [ -f config/systemd/omega-telegram.service ] && grep -q "omega-telegram.service" install.sh; then ok "persistent Telegram service shipped + installed"; else bad "omega-telegram.service not shipped/wired in install.sh"; fi
if ls scripts/hooks/*.sh >/dev/null 2>&1 && grep -q "scripts/hooks" install.sh; then ok "tracking + verify hooks shipped + installed"; else bad "hooks not shipped/wired"; fi
if [ -f agents/identity/SOUL.template.md ] && grep -q "SOUL.template" install.sh; then ok "SOUL identity template shipped + installed"; else bad "SOUL template not shipped/wired"; fi
if grep -q "usage --check" install.sh; then ok "native billing cron (omega usage --check) scheduled"; else bad "native billing cron missing from install.sh"; fi

# 10. Orchestration engine (Gate+Driver+Guardian) + planner skill shipped.
if [ -f crates/omega-core/src/executor.rs ] && [ -f crates/omega-core/src/guardian.rs ]; then ok "orchestration engine (executor + guardian) in source"; else bad "engine source (executor/guardian) missing"; fi
if grep -qE "PlanRun|PlanStatus|plan-run|plan_run" crates/omega-cli/src/main.rs; then ok "omega plan-run/plan-status CLI present"; else bad "engine CLI commands missing from main.rs"; fi
if [ -f skills/planner/SKILL.md ] && [ -f skills/planner/fallback/plan.ts ] && grep -q "omg-planner" install.sh; then ok "/omg-planner skill + Bun fallback shipped + installed"; else bad "planner skill not shipped/wired in install.sh"; fi
if [ -f skills/new-project/SKILL.md ] && grep -q "skills/new-project" install.sh && grep -q "omg-new-project" install.sh; then ok "/omg-new-project end-to-end skill shipped + installed"; else bad "new-project skill not shipped/wired in install.sh"; fi
# Linear feedback-resolution skill + one-time setup wizard shipped + wired,
# self-contained (RULES.md present, no maintainer-private audit-selector dep).
if [ -f skills/linear/SKILL.md ] && [ -f skills/linear/RULES.md ] && grep -q "skills/linear" install.sh && grep -q "omg-linear" install.sh; then ok "/omg-linear skill (+ RULES.md) shipped + installed"; else bad "linear skill not shipped/wired in install.sh"; fi
if [ -f skills/linear-setup/SKILL.md ] && grep -q "skills/linear-setup" install.sh && grep -q "omg-linear-setup" install.sh; then ok "/omg-linear-setup wizard shipped + installed"; else bad "linear-setup skill not shipped/wired in install.sh"; fi
# Linear skill must not LEAK a maintainer-private path (it may mention them in
# negative "does NOT read ~/.claude" prose; only a real ~/.claude/ or /home/hacker
# path token is a leak). Check for the home-dir literal specifically.
if ! grep -q "/home/hacker" skills/linear/RULES.md skills/linear/SKILL.md; then ok "linear skill has no hardcoded maintainer path"; else bad "linear skill leaks /home/hacker"; fi
# Pipeline self-containment: OmegaOS ships its OWN /omg-vision + /omg-prd, so a
# fresh install does not depend on the user's personal /vision /prd. The
# new-project skill must delegate to the /omg-* versions, not the bare ones.
if [ -f skills/vision/SKILL.md ] && [ -f skills/prd/SKILL.md ] && grep -q "omg-\$psk" install.sh; then ok "/omg-vision + /omg-prd shipped (pipeline self-contained)"; else bad "vision/prd not shipped as /omg-* (fresh install pipeline would break)"; fi
if grep -qE '^[0-9]\. .*/omg-vision' skills/new-project/SKILL.md && grep -qE '^[0-9]\. .*/omg-prd' skills/new-project/SKILL.md; then ok "new-project pipeline delegates to /omg-vision + /omg-prd"; else bad "new-project still calls bare /vision or /prd"; fi
# OmegaOS slash commands are /omg-* namespaced (no collision with other commands).
if ! grep -q '"\$OMG_CMD_DST/planner.md"' install.sh && ! grep -q '/planner.md"' install.sh; then ok "no bare /planner stub (uses /omg-planner — no collision)"; else bad "install.sh still writes a bare /planner stub (collides)"; fi
# Every OmegaOS command exposed as /omg-* (canonical) AND /omega-* (legacy alias — non-breaking).
if grep -q 'omg-${bn#omega-}' install.sh && grep -q 'omg-dynamic' install.sh; then ok "/omg-* aliases generated for all OmegaOS commands (legacy /omega-* kept)"; else bad "/omg-* alias loop missing from install.sh"; fi
# Companion tools + skills (SST multi-LLM) shipped and sourced by install.sh.
if [ -f scripts/install-companion-tools.sh ] && grep -q "install-companion-tools.sh" install.sh; then ok "companion tools (planning-with-files/higgsfield/claude-mem/superpowers/mempalace/remotion) shipped + wired"; else bad "companion-tools installer not shipped/wired in install.sh"; fi

# ── Behavioral gates (runtime truth, not text-greps) ─────────────────────────
# The checks above grep install.sh for the right STRINGS; these prove the system
# actually builds and the shipped config actually parses. Skip with VERIFY_FAST=1
# (the fast curator path); CI and the L0 gate run the real thing.
if [ "${VERIFY_FAST:-0}" != "1" ] && command -v cargo >/dev/null 2>&1; then
  if cargo check --workspace --locked >/dev/null 2>&1; then
    ok "workspace compiles against the committed Cargo.lock (reproducible)"
  else
    bad "cargo check --locked failed — lockfile out of sync or code broken"
  fi
  if cargo test -p omega-core --lib shipped_default_toml_deserializes >/dev/null 2>&1; then
    ok "shipped config/default.toml deserializes into OmegaConfig"
  else
    bad "config/default.toml does not deserialize — fresh install would discard it"
  fi
else
  ok "behavioral build/deserialize gates skipped (VERIFY_FAST or no cargo)"
fi

echo "═══════════════════════════════════"
if [ "$fail" -eq 0 ]; then
  printf '\033[32mINSTALL PARITY OK — a fresh install reproduces this system.\033[0m\n'
else
  printf '\033[31mINSTALL PARITY FAILED — fix the above before declaring done (Law 0).\033[0m\n'
fi
exit "$fail"
