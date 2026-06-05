#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — proactive Claude OAuth token refresh (headless)
# ───────────────────────────────────────────────────────────────────────────
# Run on a cron (every 30 min). If the Claude OAuth token expires soon, refresh
# it via the shipped claude-oauth helper BEFORE an agent hits a 401. If the
# refresh fails (refresh_token dead), alert the operator on Telegram to /login.
# Idempotent, no TTY required. Installed by install.sh; cron marker
# OMEGA-CRON-TOKEN-REFRESH-v1.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
CRED="$HOME/.claude/.credentials.json"
OAUTH="$OMEGA_DIR/bin/claude-oauth.sh"
THRESHOLD=1800   # refresh when < 30 min to expiry
stamp() { date -u +%Y-%m-%dT%H:%M:%SZ; }

[ -f "$CRED" ] || { echo "$(stamp) no credential at $CRED — skip"; exit 0; }
[ -x "$OAUTH" ] || { echo "$(stamp) oauth helper missing — skip"; exit 0; }

exp=$(python3 -c "import json,sys;d=json.load(open('$CRED'));o=d.get('claudeAiOauth',d);print(int(o['expiresAt'])//1000)" 2>/dev/null) || exit 0
[ -z "${exp:-}" ] && exit 0
now=$(date +%s); left=$((exp - now))

# ── omega-mc gateway token sync ──────────────────────────────────────────────
# The Mission-Control gateway gets CLAUDE_CODE_OAUTH_TOKEN from its .env at
# container-create time (baked env). Every /login or refresh ROTATES the access
# token, leaving the gateway (and every agent it spawns) with a dead token →
# dashboard agents show "Not logged in • Please run /login". Reconcile on every
# cron pass: if the live token differs from .env, rewrite it and recreate the
# gateway so the fleet follows the credential, hands-free.
MC_DIR="$OMEGA_DIR/repos/omega-mc"
sync_mc() {
    [ -f "$MC_DIR/.env" ] || return 0
    local live cur
    live=$(python3 -c "import json;d=json.load(open('$CRED'));print(d.get('claudeAiOauth',d).get('accessToken',''))" 2>/dev/null) || return 0
    [ -n "$live" ] || return 0
    cur=$(grep -E '^CLAUDE_CODE_OAUTH_TOKEN=' "$MC_DIR/.env" 2>/dev/null | head -1 | cut -d= -f2-)
    [ "$cur" = "$live" ] && return 0
    sed -i "s|^CLAUDE_CODE_OAUTH_TOKEN=.*|CLAUDE_CODE_OAUTH_TOKEN=$live|" "$MC_DIR/.env"
    if (cd "$MC_DIR" && sudo -n docker compose up -d --no-build >/dev/null 2>&1); then
        echo "$(stamp) omega-mc: token rotated → .env synced + gateway recreated"
    else
        echo "$(stamp) omega-mc: .env synced (gateway recreate failed — retry next run)"
    fi
}
sync_mc

if [ "$left" -ge "$THRESHOLD" ]; then
    echo "$(stamp) token healthy (${left}s left) — no action"
    exit 0
fi

echo "$(stamp) token expires in ${left}s (< ${THRESHOLD}s) — refreshing"
res=$(bash "$OAUTH" try-refresh 2>&1)
echo "$(stamp) try-refresh: $res"

if echo "$res" | grep -q '"ok"[[:space:]]*:[[:space:]]*true'; then
    echo "$(stamp) refresh OK"
    sync_mc   # the refresh just rotated the token → follow it into omega-mc now
    exit 0
fi

# Refresh failed → alert the operator (canonical Alerts-topic sender) to re-login.
if bash "$OMEGA_DIR/bin/omega-alert-send.sh" "⚠️ AISB: the Claude token refresh failed. Open /account → 🔐 Login to re-authenticate (otherwise the agents will start hitting 401)." 2>/dev/null; then
    echo "$(stamp) refresh failed — operator alerted on Telegram"
else
    echo "$(stamp) refresh failed — no telegram config to alert"
fi
exit 0
