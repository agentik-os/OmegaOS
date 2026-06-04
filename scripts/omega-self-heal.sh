#!/usr/bin/env bash
# OmegaOS self-heal daemon (cron, every few hours).
# ───────────────────────────────────────────────────────────────────────────
# Runs `omega doctor --fix` to apply the safe mechanical fixes it knows how to
# resolve (duplicate Telegram pollers, dead bot service, stale usage cache,
# expired oauth). If warnings REMAIN after the fix pass, it sends ONE branded
# Telegram alert to the operator so they can tap /status → "🛠 Fix it" (which
# dispatches an oracle). It does NOT auto-dispatch an oracle itself — that costs
# tokens and should stay a human decision (R-BUDGET).
#
# Scheduled by install.sh (OMEGA-CRON-SELFHEAL-v1) so a fresh clone self-heals.
set -uo pipefail

OMEGA="${OMEGA_BIN:-$HOME/.local/bin/omega}"
[ -x "$OMEGA" ] || OMEGA="$(command -v omega 2>/dev/null || echo omega)"

# 1) Apply mechanical fixes (prints what it did + the post-fix check list).
OUT="$("$OMEGA" doctor --fix 2>&1)"
echo "$OUT"

# 2) Count warnings/fails REMAINING after the fix pass.
REMAIN="$("$OMEGA" doctor 2>&1 | grep -cE '^\s*\[[!x]\]')"
[ "${REMAIN:-0}" -gt 0 ] 2>/dev/null || { echo "self-heal: clean"; exit 0; }

# 3) Best-effort Telegram alert (same creds the usage monitor uses).
TG="$HOME/.omega/telegram.toml"
[ -f "$TG" ] || exit 0
TOKEN="$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG" 2>/dev/null | head -1)"
CHAT="$(grep -oP 'allow_user_ids\s*=\s*\[\s*\K[0-9]+' "$TG" 2>/dev/null | head -1)"
[ -n "$CHAT" ] || CHAT="$(grep -oP 'chat_id\s*=\s*\K[0-9]+' "$TG" 2>/dev/null | head -1)"
[ -n "$TOKEN" ] && [ -n "$CHAT" ] || exit 0

# Cooldown: at most one self-heal alert per 6h (don't nag).
FLAG="$HOME/.omega/state/selfheal-alert-sent"
if [ -f "$FLAG" ]; then
    AGE=$(( $(date +%s) - $(stat -c %Y "$FLAG" 2>/dev/null || echo 0) ))
    [ "$AGE" -lt 21600 ] && exit 0
fi
mkdir -p "$HOME/.omega/state" && : > "$FLAG"

RULE="━━━━━━━━━━━━"
MSG="${RULE}
<b>Ω  AUTO-HEAL</b>
${RULE}
 🩺 Pass exécuté — <b>${REMAIN}</b> alerte(s) restante(s).
 Tape /status → <b>🛠 Fix it</b> pour envoyer un oracle."

curl -fsS --max-time 15 "https://api.telegram.org/bot${TOKEN}/sendMessage" \
    --data-urlencode "chat_id=${CHAT}" \
    --data-urlencode "text=${MSG}" \
    --data-urlencode "parse_mode=HTML" \
    --data-urlencode "disable_web_page_preview=true" >/dev/null 2>&1 || true
