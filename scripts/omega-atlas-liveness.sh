#!/usr/bin/env bash
# OmegaOS Telegram liveness watchdog (cron, every 2 min).
# ───────────────────────────────────────────────────────────────────────────
# WHY: every OmegaOS Telegram bot — the master (Atlas) AND each project agent
# bot — runs the SAME 261KB core with a SINGLE poll loop. Every inbound update
# is processed with `await` INSIDE that loop, and some handler paths (voice
# transcription, file download) do an un-timed `fetch` with no watchdog. One
# stalled fetch blocks the loop forever: the bun process stays alive (the
# pollProgress/pollReports setIntervals keep the event loop up), so systemd
# still reports the unit `active` and `omega doctor` (which only checks the
# systemd state) reports "healthy". The bot goes DEAF but nothing restarts it —
# the operator finds it "locked" again and again.
#
# This probe closes that blind spot WITHOUT touching the bot core:
# getWebhookInfo (which does NOT conflict with getUpdates long-polling) exposes
# pending_update_count. A healthy bot drains pending to 0 within a second or two.
# If pending stays > 0 and is NOT draining across a ~25s window AND the bot is
# not legitimately busy (no claude/codex child), the poll loop is stuck → restart.
#
# COVERAGE: master + every agent bot in agent-bots.json. It used to watch the
# master ONLY, which left the project bots running the exact un-timed-fetch code
# this watchdog exists for, with nothing to recover them (found 2026-07-23: six
# agent bots had been up since 2026-07-09, unprotected).
#
# Guards against false positives / restart loops:
#   - busy-guard: never restart a bot with a live claude/codex child (a legit
#     long task can hold pending > 0 for minutes).
#   - drain-guard: only restart if pending2 >= pending1 (not consuming at all).
#   - cooldown: at most one restart per 15 min, PER UNIT (own stamp file).
#   - ONE shared 25s window for all bots, so covering N bots costs ~30s total
#     and never overruns the 2-minute cron interval.
set -uo pipefail

API_BASE="https://api.telegram.org/bot"
pending() {  # $1 = token
    curl -s --max-time 10 "${API_BASE}${1}/getWebhookInfo" \
        | grep -oP '"pending_update_count":\K[0-9]+'
}

# ── Build the watch list: "unit<TAB>token<TAB>label" ─────────────────────────
WATCH=""

TG="$HOME/.omega/telegram.toml"
if [ -f "$TG" ]; then
    MASTER_TOKEN="$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG" 2>/dev/null | head -1)"
    [ -n "${MASTER_TOKEN:-}" ] && WATCH="omega-tg-bot.service	${MASTER_TOKEN}	Atlas"
fi

# Agent bots: one entry per key in agent-bots.json ({"<name>": {"token": ...}}).
BOTS_JSON="$HOME/.omega/agent-bots.json"
if [ -f "$BOTS_JSON" ] && command -v python3 >/dev/null 2>&1; then
    AGENTS="$(python3 - "$BOTS_JSON" <<'PY' 2>/dev/null
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(0)
if isinstance(d, dict):
    for name, cfg in d.items():
        if isinstance(cfg, dict) and cfg.get("token"):
            print("omega-tg-agent-%s.service\t%s\t%s" % (name, cfg["token"], name))
PY
)"
    [ -n "${AGENTS:-}" ] && WATCH="${WATCH}
${AGENTS}"
fi

[ -n "${WATCH//[[:space:]]/}" ] || exit 0

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=${XDG_RUNTIME_DIR}/bus}"

# busy-guard: a live agent child means the bot IS working a real task.
busy() {  # $1 = main pid
    # python/omega-transcribe = the local STT (faster-whisper) child transcribing a
    # voice note: legitimate work (2026-08-11: the guard missed it and the watchdog
    # kept restarting the dentistrygpt bot mid-transcription, every cooldown window).
    ps --ppid "$1" -o comm= 2>/dev/null | grep -qiE 'claude|codex|whisper|ffmpeg|python|omega-transcribe'
}

# ── Pass 1: snapshot pending for every bot; keep only the suspicious ones ────
SUSPECT=""
while IFS=$'\t' read -r unit token label; do
    [ -n "${unit:-}" ] && [ -n "${token:-}" ] || continue
    p1="$(pending "$token")"
    [ -n "${p1:-}" ] || continue
    [ "$p1" -gt 0 ] 2>/dev/null || continue
    mpid="$(systemctl --user show -p MainPID --value "$unit" 2>/dev/null)"
    [ -n "${mpid:-}" ] && [ "$mpid" != "0" ] || continue
    busy "$mpid" && continue
    SUSPECT="${SUSPECT}${unit}	${token}	${label}	${p1}	${mpid}
"
done <<< "$WATCH"

[ -n "${SUSPECT//[[:space:]]/}" ] || exit 0

# ── One shared drain window, then re-probe only the suspects ────────────────
sleep 25

while IFS=$'\t' read -r unit token label p1 mpid; do
    [ -n "${unit:-}" ] || continue
    p2="$(pending "$token")"
    [ -n "${p2:-}" ] || continue
    [ "$p2" -gt 0 ] 2>/dev/null || continue      # drained → was just busy, healthy
    [ "$p2" -ge "$p1" ] 2>/dev/null || continue  # still draining → consuming, healthy
    busy "$mpid" && continue                     # a task started during the wait

    # Cooldown: at most one restart per 15 min, per unit.
    FLAG="$HOME/.omega/state/tg-liveness-restart-${unit%.service}"
    mkdir -p "$HOME/.omega/state"
    if [ -f "$FLAG" ]; then
        AGE=$(( $(date +%s) - $(stat -c %Y "$FLAG" 2>/dev/null || echo 0) ))
        [ "$AGE" -lt 900 ] && continue
    fi

    echo "[$(date '+%F %T')] tg-liveness: DEAF ${label} (pending ${p1}→${p2}, pid ${mpid}, no agent child) → restarting ${unit}"
    systemctl --user restart "$unit" && : > "$FLAG"

    # Alert the operator through the canonical funnel (best-effort).
    MSG="━━━━━━━━━━━━
<b>Ω  TELEGRAM AUTO-HEAL</b>
━━━━━━━━━━━━
 🔄 ${label} was alive but not consuming (${p1} msg stuck). Poll loop restarted."
    [ -x "$HOME/.omega/bin/omega-alert-send.sh" ] && bash "$HOME/.omega/bin/omega-alert-send.sh" "$MSG" >/dev/null 2>&1 || true
done <<< "$SUSPECT"
