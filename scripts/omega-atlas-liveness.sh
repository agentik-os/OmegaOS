#!/usr/bin/env bash
# OmegaOS Atlas liveness watchdog (cron, every 2 min).
# ───────────────────────────────────────────────────────────────────────────
# WHY: the master Telegram bot (Atlas) runs a SINGLE poll loop. Every inbound
# update is processed with `await` INSIDE that loop, and some handler paths
# (voice transcription, file download) do an un-timed `fetch` with no watchdog.
# One stalled fetch blocks the loop forever: the bun process stays alive (the
# pollProgress/pollReports setIntervals keep the event loop up), so systemd
# still reports the unit `active` and `omega doctor` (which only checks the
# systemd state) reports "healthy". Atlas goes DEAF but nothing restarts it —
# the operator finds it "locked" again and again.
#
# This probe closes that blind spot WITHOUT touching the 261KB bot core:
# getWebhookInfo (which does NOT conflict with getUpdates long-polling) exposes
# pending_update_count. A healthy bot drains pending to 0 within a second or two.
# If pending stays > 0 and is NOT draining across a ~25s window AND the bot is
# not legitimately busy (no claude/codex child), the poll loop is stuck → restart.
#
# Guards against false positives / restart loops:
#   - busy-guard: never restart while the master has a live claude/codex child
#     (a legit long task can hold pending > 0 for minutes).
#   - drain-guard: only restart if pending2 >= pending1 (not consuming at all).
#   - cooldown: at most one restart per 15 min (stamp file).
set -uo pipefail

TG="$HOME/.omega/telegram.toml"
[ -f "$TG" ] || exit 0
TOKEN="$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG" 2>/dev/null | head -1)"
[ -n "$TOKEN" ] || exit 0

API="https://api.telegram.org/bot${TOKEN}"
pending() { curl -s --max-time 10 "${API}/getWebhookInfo" | grep -oP '"pending_update_count":\K[0-9]+'; }

P1="$(pending)"
# No answer from Telegram (network) or nothing pending → nothing to do.
[ -n "${P1:-}" ] || exit 0
[ "$P1" -gt 0 ] 2>/dev/null || exit 0

# Resolve the master PID via systemd (user scope). Bail quietly if unavailable.
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=${XDG_RUNTIME_DIR}/bus}"
MPID="$(systemctl --user show -p MainPID --value omega-tg-bot.service 2>/dev/null)"
[ -n "${MPID:-}" ] && [ "$MPID" != "0" ] || exit 0

# busy-guard: a live claude/codex child means the bot IS working a real task.
if pgrep -P "$MPID" >/dev/null 2>&1; then
    if ps --ppid "$MPID" -o comm= 2>/dev/null | grep -qiE 'claude|codex|bun|node|whisper|ffmpeg'; then
        exit 0
    fi
fi

# Give a healthy bot time to drain; a deaf one will not move.
sleep 25
P2="$(pending)"
[ -n "${P2:-}" ] || exit 0
[ "$P2" -gt 0 ] 2>/dev/null || exit 0      # drained → it was just busy, healthy
[ "$P2" -ge "$P1" ] 2>/dev/null || exit 0  # still draining (P2<P1) → consuming, healthy
# re-check busy-guard after the wait (a task may have started meanwhile)
if ps --ppid "$MPID" -o comm= 2>/dev/null | grep -qiE 'claude|codex|whisper|ffmpeg'; then
    exit 0
fi

# Cooldown: at most one restart per 15 min.
FLAG="$HOME/.omega/state/atlas-liveness-restart"
mkdir -p "$HOME/.omega/state"
if [ -f "$FLAG" ]; then
    AGE=$(( $(date +%s) - $(stat -c %Y "$FLAG" 2>/dev/null || echo 0) ))
    [ "$AGE" -lt 900 ] && exit 0
fi

echo "[$(date '+%F %T')] atlas-liveness: DEAF (pending ${P1}→${P2}, pid ${MPID}, no agent child) → restarting omega-tg-bot"
systemctl --user restart omega-tg-bot.service && : > "$FLAG"

# Alert the operator through the canonical funnel (best-effort).
MSG="━━━━━━━━━━━━
<b>Ω  ATLAS AUTO-HEAL</b>
━━━━━━━━━━━━
 🔄 Atlas was alive but not consuming (${P1} msg stuck). Poll loop restarted."
[ -x "$HOME/.omega/bin/omega-alert-send.sh" ] && bash "$HOME/.omega/bin/omega-alert-send.sh" "$MSG" >/dev/null 2>&1 || true
