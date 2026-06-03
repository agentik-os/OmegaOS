#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-tg-up.sh — connect Telegram to the OmegaOS command bot, in one command.
# ───────────────────────────────────────────────────────────────────────────
# The OmegaOS command bot (telegram-bot/omega-tg-bot.ts) is the operator's phone
# control center: every command opens an inline-keyboard of sub-actions, each
# running an `omega` CLI action. This validates the bot token, writes it to
# ~/.omega/telegram.toml (operator-locked, R-TGSEC), and (re)starts the bot.
#
#   omega-tg-up <BOT_TOKEN> <YOUR_TELEGRAM_USER_ID>
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
TG="$OMEGA_DIR/telegram.toml"
BOT="$OMEGA_DIR/telegram-bot/omega-tg-bot.ts"
TOKEN="${1:-}"; USERID="${2:-}"

[[ -n "$TOKEN" ]] || { echo "usage: omega-tg-up <BOT_TOKEN> <YOUR_TELEGRAM_USER_ID>"; exit 1; }
if command -v curl >/dev/null 2>&1; then
  curl -sf "https://api.telegram.org/bot${TOKEN}/getMe" >/dev/null 2>&1 \
    || { echo "omega-tg-up: token rejected by Telegram (getMe) — get a fresh token from @BotFather"; exit 1; }
fi

umask 077
cat > "$TG" <<EOF
bot_token = "${TOKEN}"
chat_id = ${USERID:-0}
allow_user_ids = [${USERID:-}]
relay_session = "aisb-master"
enabled = true
EOF
chmod 0600 "$TG"

# (Re)start the bot — systemd service if present, else a detached process.
if command -v systemctl >/dev/null 2>&1 && systemctl --user list-unit-files omega-tg-bot.service >/dev/null 2>&1; then
  systemctl --user enable --now omega-tg-bot.service 2>/dev/null || true
  systemctl --user restart omega-tg-bot.service 2>/dev/null || true
elif command -v bun >/dev/null 2>&1 && [[ -f "$BOT" ]]; then
  pkill -f "omega-tg-bot.ts" 2>/dev/null || true
  ( cd "$OMEGA_DIR/telegram-bot" && OMEGA_DIR="$OMEGA_DIR" nohup bun "$BOT" >/tmp/omega-tg-bot.log 2>&1 & )
fi
echo "omega-tg-up: ✓ connected. DM the bot and tap /menu. For a project hub: create a supergroup (enable Topics), add the bot as ADMIN, then /setupgroup + /sync inside it."
