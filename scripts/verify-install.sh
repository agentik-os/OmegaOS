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

# 4. Full-scrollback retention persisted in the sourced rmux config, at the
#    deepened 500k default (guards against a regression to the old 100k).
if grep -q "history-limit 500000" config/rmux.conf.omega 2>/dev/null; then ok "rmux history-limit persisted (500000)"; else bad "history-limit not 500000 in rmux.conf.omega"; fi
# 4-bis. The startup history-limit call (omega binary) matches the 500k default.
# (The old Rust telegram bridge was retired — replaced by the Bun bot omega-tg-bot.ts —
# so this now lives only in main.rs.)
if [ "$(grep -ro '"history-limit", "500000"' crates/omega-cli/src/ 2>/dev/null | wc -l)" -ge 1 ]; then ok "omega startup history-limit = 500000 (main.rs)"; else bad "startup history-limit not 500000 in main.rs"; fi

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

# 4e. Terminal UX hardening (crash-test findings) persisted in the sourced rmux
#     config: mouse scroll/select, OSC52 clipboard, snappy escape-time, truecolor.
#     These are the defaults rmux gets wrong for an all-day agent session.
RC=config/rmux.conf.omega
while IFS= read -r opt; do
  [ -z "$opt" ] && continue
  if grep -qF "$opt" "$RC" 2>/dev/null; then ok "rmux: '$opt' persisted"; else bad "rmux: '$opt' missing from rmux.conf.omega"; fi
done <<'OPTS'
set -g mouse on
set -g set-clipboard external
set -g allow-passthrough on
set -g escape-time 10
set -sa terminal-features ",*:RGB"
set -g focus-events on
OPTS

# 4f. Optional low-latency SSH (mosh) bootstrapped best-effort by install.sh.
#     Predictive local echo + UDP diffs → lag-free typing/streaming on a far VPS.
if grep -q "install_mosh_optional" install.sh; then ok "mosh (low-latency SSH) bootstrapped by install.sh"; else bad "mosh bootstrap step missing from install.sh"; fi

# 4g. System-wide rmux config so EVERY user (root + future accounts) gets the
#     hardened session via /etc/rmux.conf, not just the installing user.
if grep -q "/etc/omega/rmux.conf.omega" install.sh && grep -q "/etc/rmux.conf" install.sh; then ok "system-wide rmux config (/etc/rmux.conf) wired for all users"; else bad "system-wide rmux config step missing from install.sh"; fi

# 4h. UTF-8 locale guaranteed so mosh never degrades + TUI renders correctly,
#     for root + future users (what the Termius mosh -l LANG=… relies on).
if grep -q "ensure_utf8_locale" install.sh; then ok "UTF-8 locale guaranteed by install.sh"; else bad "UTF-8 locale guarantee missing from install.sh"; fi

# 4i. Claude Code set to normal-screen so the full conversation scrolls in the
#     rmux panel (flows into the 500k scrollback) instead of Claude's fullscreen
#     buffer. Written to the shell env file by install.sh for every session.
if grep -q "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN" install.sh; then ok "Claude normal-screen (full scroll in rmux panel) wired by install.sh"; else bad "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN not set by install.sh"; fi

# 4j. Omega chrome theme (Monogram model) persisted in the sourced rmux config:
#     dark chrome + cyan accent for focus, dim passives, reserved amber/red
#     alert channel. Sentinels = the state-differentiation load-bearers.
RC=config/rmux.conf.omega
while IFS= read -r opt; do
  [ -z "$opt" ] && continue
  if grep -qF "$opt" "$RC" 2>/dev/null; then ok "rmux theme: '$opt' persisted"; else bad "rmux theme: '$opt' missing from rmux.conf.omega"; fi
done <<'OPTS'
set -g status-style "bg=#16161a,fg=#7f7f7f"
set -g window-status-current-style "fg=#000000,bg=#00ffff,bold"
set -g window-status-bell-style "fg=#000000,bg=#ff6e6e,bold"
set -g pane-active-border-style "fg=#00ffff"
set -g message-style "bg=#ffa500,fg=#000000,bold"
OPTS

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
if [ -f telegram-bot/omega-tg-bot.ts ] && [ -f scripts/omega-tg-up.sh ] && grep -q "omega-tg-bot.ts" install.sh && grep -q "omega-tg-up" install.sh; then ok "Telegram command bot shipped + wired (omega-tg-bot + omega-tg-up + systemd service)"; else bad "Telegram command bot (omega-tg-bot.ts / omega-tg-up.sh) not shipped/wired in install.sh"; fi
if [ -f scripts/omega-mc-up.sh ] && grep -q "omega-mc-up.sh" install.sh && grep -q "agentik-telegram" install.sh; then ok "OmegaMC optional multi-agent backend shipped + wired (omega-mc-up + agentik-telegram clone)"; else bad "OmegaMC (omega-mc-up.sh / agentik-telegram clone) not shipped/wired in install.sh"; fi
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
# Browser engine for the Quality Arsenal audits (uiux/flow/a11y/perf, browser-tester)
# + CDP/DevTools automation: install.sh must provision Playwright AND its Chromium
# (Chromium ships the DevTools Protocol, so one install covers both).
if grep -q "playwright install chromium" install.sh && grep -q "OMEGA_SKIP_BROWSER" install.sh; then ok "Playwright + Chromium (CDP/DevTools) provisioned by install.sh"; else bad "install.sh does not install Playwright/Chromium — browser audits would fail on a fresh box"; fi
# Quality Arsenal audit SKILLS shipped + wired (the registry is 23 audits; the
# skill dirs must match so a fresh install can actually run them — excludes the
# 3 non-audit dirs _shared / audit-orchestrator / audit-tracker).
N_AUDITS=$(ls -d skills/audits/*/ 2>/dev/null | grep -vE '/(_shared|audit-orchestrator|audit-tracker)/$' | wc -l)
if [ "$N_AUDITS" -ge 23 ] && grep -q "skills/audits" install.sh; then ok "Quality Arsenal audit skills shipped + wired ($N_AUDITS)"; else bad "audit skills missing or not wired in install.sh ($N_AUDITS dirs, need >=23)"; fi
# Design skills (generative UI/UX) shipped + wired.
N_DESIGN=$(ls -d skills/design/*/ 2>/dev/null | wc -l)
if [ "$N_DESIGN" -ge 8 ] && grep -q "skills/design" install.sh; then ok "Design skills shipped + wired ($N_DESIGN)"; else bad "design skills missing or not wired in install.sh ($N_DESIGN dirs, need >=8)"; fi
# Design skills must not leak the maintainer's identity/paths.
if grep -rqE 'Gareth|/home/hacker' skills/design/ 2>/dev/null; then bad "design skills leak Gareth/home path"; else ok "design skills clean (no Gareth/home leak)"; fi
# PDF generator shipped + wired (all branded PDF output depends on it).
if [ -d tools/pdfgen ] && grep -q "tools/pdfgen" install.sh; then ok "pdfgen shipped + wired in install.sh"; else bad "pdfgen not shipped/wired in install.sh"; fi
# OAuth fallback helper shipped + wired (non-interactive token refresh).
if [ -f docs/reference/oauth/claude-oauth.sh ] && grep -q "claude-oauth.sh" install.sh; then ok "OAuth helper shipped + wired in install.sh"; else bad "oauth helper not shipped/wired in install.sh"; fi
# Reference docs tree shipped + wired (`omega docs` + $HOME Claude sessions read it).
if [ -d docs ] && [ -f docs/ARCHITECTURE.md ] && grep -q "OMEGA_SRC/docs" install.sh; then ok "reference docs shipped + wired in install.sh"; else bad "docs tree not shipped/wired in install.sh"; fi

# 10b. No SKILL.md leaks a maintainer-private ~/.claude or /home/hacker path that
#      install.sh does NOT create. OmegaOS ships on a blank VPS: an audit skill that
#      shells out to `~/.claude/lib/hinge-analyzer.sh` or reads `~/.claude/DEPRECATED.md`
#      silently breaks on a fresh clone. install.sh only provisions ~/.claude/commands/,
#      ~/.claude/settings.json, and ~/.claude/.credentials.json — anything else under
#      ~/.claude/ (lib/data/agents/resources/rules/projects/…) or under /home/hacker/ is a leak.
#      Documented blank-VPS warnings ("never reference ~/.claude/…") are prose, not leaks.
leaks=$(grep -rhnE '(~/\.claude/|/home/hacker/)[A-Za-z0-9_./-]+' skills/audits/*/SKILL.md skills/audits/_shared/* 2>/dev/null \
  | grep -vE '~/\.claude/(commands/|settings\.json|\.credentials\.json)' \
  | grep -vE '~/\.claude/\.\.\.' \
  | grep -vE '(blank-VPS|never reference|never reach|forbids|ships them|shipped INSIDE|vendored next)')
if [ -z "$leaks" ]; then
  ok "no SKILL.md references an unshipped ~/.claude or /home/hacker path"
else
  bad "SKILL.md files reference ~/.claude/home paths install.sh does NOT create (blank-VPS clone breaks):"
  printf '%s\n' "$leaks" | sed 's/^/      /'
fi

# 10c. Quality Arsenal RUNTIME parity. The audit SKILLs hard-code the hybrid
#      orchestrator by absolute path under the SINGLE home: ~/.omega/lib/audit-runner.sh
#      + audit-gather/ + safe-npm-build.sh + ~/.omega/bin/audit-notify.sh. This gate
#      proves both halves: (a) the runtime is VENDORED in _shared/, and (b) install.sh
#      SHIPS it to ~/.omega. Either half missing = audits break on a fresh clone (Law 0).
#      Also asserts the dual ~/.aisb home is gone (consolidated into ~/.omega).
miss=""
[ -f skills/audits/_shared/audit-runner.sh ]   || miss="$miss audit-runner.sh"
[ -d skills/audits/_shared/audit-gather ]      || miss="$miss audit-gather/"
[ -f skills/audits/_shared/audit-notify.sh ]   || miss="$miss audit-notify.sh"
[ -f skills/audits/_shared/safe-npm-build.sh ] || miss="$miss safe-npm-build.sh"
if [ -n "$miss" ]; then bad "audit runtime not vendored in _shared:$miss"; else ok "audit runtime vendored in _shared (runner + gather + notify + safe-npm-build)"; fi
if grep -q 'OMEGA_DIR/lib/audit-runner\.sh' install.sh && grep -q 'OMEGA_DIR/bin/audit-notify\.sh' install.sh; then
  ok "install.sh ships the ~/.omega audit runtime (skills' absolute paths resolve)"
else
  bad "install.sh does NOT install the ~/.omega audit runtime — audit SKILLs break on fresh clone"
fi
# 10c-bis. No skill or agent may reference the retired ~/.aisb dual-home (consolidated → ~/.omega).
if grep -rqE '/\.aisb/' skills/ agents/ 2>/dev/null; then
  bad "a skill/agent still references the retired ~/.aisb home (should be ~/.omega):"; grep -rlE '/\.aisb/' skills/ agents/ | sed 's/^/      /'
else
  ok "no skill/agent references the retired ~/.aisb home (single ~/.omega home)"
fi

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

# 11. New self-healing assets shipped + wired (token-refresh, shared-credential, scrollback alias).
if [ -f scripts/omega-token-refresh.sh ] && grep -q "omega-token-refresh.sh" install.sh; then ok "token-refresh helper shipped + wired in install.sh"; else bad "token-refresh helper not shipped/wired in install.sh"; fi
if grep -q "OMEGA-CRON-TOKEN-REFRESH-v1" install.sh; then ok "token-refresh cron scheduled by install.sh"; else bad "token-refresh cron not in install.sh"; fi
if [ -f scripts/omega-self-heal.sh ] && grep -q "omega-self-heal.sh" install.sh && grep -q "OMEGA-CRON-SELFHEAL-v1" install.sh; then ok "self-heal daemon shipped + wired + cron scheduled"; else bad "self-heal daemon (omega-self-heal.sh) not shipped/wired in install.sh"; fi
if grep -q "OMEGA_CREDENTIALS_LINK" install.sh; then ok "shared-credential override (OMEGA_CREDENTIALS_LINK) wired in install.sh"; else bad "shared-credential override not in install.sh"; fi
if grep -q "alias om=" install.sh && grep -q "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN" install.sh; then ok "shell integration (om alias + scrollback) wired in install.sh"; else bad "shell integration not in install.sh"; fi
if [ -f docs/GETTING-STARTED.md ] && grep -q "GETTING-STARTED.md" install.sh; then ok "getting-started guide shipped + installed to ~/.omega"; else bad "getting-started guide (docs/GETTING-STARTED.md) not shipped/wired in install.sh"; fi
if grep -q "Commands::Guide" crates/omega-cli/src/main.rs && grep -q "GETTING-STARTED.md" crates/omega-cli/src/main.rs; then ok "omega guide command wired (embedded fallback)"; else bad "omega guide command missing from CLI"; fi
if grep -q "agent-bot resurrect" telegram-bot/omega-tg-bot.ts; then ok "agent-bot units resurrected at bot startup (reinstall-safe)"; else bad "agent-bot resurrection loop missing from bot startup"; fi

echo "═══════════════════════════════════"
if [ "$fail" -eq 0 ]; then
  printf '\033[32mINSTALL PARITY OK — a fresh install reproduces this system.\033[0m\n'
else
  printf '\033[31mINSTALL PARITY FAILED — fix the above before declaring done (Law 0).\033[0m\n'
fi
exit "$fail"
