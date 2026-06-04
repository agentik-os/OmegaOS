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

if [ "$left" -ge "$THRESHOLD" ]; then
    echo "$(stamp) token healthy (${left}s left) — no action"
    exit 0
fi

echo "$(stamp) token expires in ${left}s (< ${THRESHOLD}s) — refreshing"
res=$(bash "$OAUTH" try-refresh 2>&1)
echo "$(stamp) try-refresh: $res"

if echo "$res" | grep -q '"ok"[[:space:]]*:[[:space:]]*true'; then
    echo "$(stamp) refresh OK"
    exit 0
fi

# Refresh failed → alert the operator on Telegram to re-login via the bot.
TG=$(grep -E '^[[:space:]]*bot_token' "$OMEGA_DIR/telegram.toml" 2>/dev/null | head -1 | cut -d'"' -f2)
CHAT=$(grep -E '^[[:space:]]*chat_id' "$OMEGA_DIR/telegram.toml" 2>/dev/null | head -1 | grep -oE '[0-9]+' | head -1)
if [ -n "${TG:-}" ] && [ -n "${CHAT:-}" ]; then
    curl -s "https://api.telegram.org/bot${TG}/sendMessage" \
        --data-urlencode "chat_id=${CHAT}" \
        --data-urlencode "text=⚠️ AISB : le refresh du token Claude a échoué. Ouvre /account → 🔐 Login pour te reconnecter (sinon les agents tomberont en 401)." >/dev/null 2>&1
    echo "$(stamp) refresh failed — operator alerted on Telegram"
else
    echo "$(stamp) refresh failed — no telegram config to alert"
fi
exit 0
