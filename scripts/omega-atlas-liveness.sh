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
#   - cooldown: at most one restart per 15 min, PER UNIT.
#   - circuit breaker: after 3 restarts in 60 min the unit is left untouched
#     until an operator explicitly resets it with `--reset <unit>`.
#   - ONE shared 25s window for all bots, so covering N bots costs ~30s total
#     and never overruns the 2-minute cron interval.
set -uo pipefail

API_BASE="${OMEGA_TG_API_BASE:-https://api.telegram.org/bot}"
DRAIN_SECONDS="${OMEGA_TG_DRAIN_SECONDS:-25}"
COOLDOWN_SECONDS="${OMEGA_TG_COOLDOWN_SECONDS:-900}"
WINDOW_SECONDS="${OMEGA_TG_WINDOW_SECONDS:-3600}"
MAX_RESTARTS="${OMEGA_TG_MAX_RESTARTS:-3}"
LOCK_STALE_SECONDS="${OMEGA_TG_LOCK_STALE_SECONDS:-120}"
STATE_DIR="${OMEGA_TG_STATE_DIR:-$HOME/.omega/state}"
ALERT_BIN="${OMEGA_TG_ALERT_BIN:-$HOME/.omega/bin/omega-alert-send.sh}"
if ! [[ "$DRAIN_SECONDS" =~ ^[0-9]+$ && "$COOLDOWN_SECONDS" =~ ^[0-9]+$ \
    && "$WINDOW_SECONDS" =~ ^[0-9]+$ && "$MAX_RESTARTS" =~ ^[1-9][0-9]*$ \
    && "$LOCK_STALE_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
    echo "tg-liveness: invalid numeric circuit-breaker configuration" >&2
    exit 2
fi

valid_unit() {
    [[ "$1" =~ ^omega-tg-(bot|agent-[a-zA-Z0-9_-]+)\.service$ ]]
}

reset_circuit() {
    local requested="${1:-}"
    mkdir -p "$STATE_DIR"
    if [ "$requested" = "all" ]; then
        local state
        for state in \
            "$STATE_DIR"/tg-liveness-circuit-omega-tg-* \
            "$STATE_DIR"/tg-liveness-restarts-omega-tg-* \
            "$STATE_DIR"/tg-liveness-restart-omega-tg-* \
            "$STATE_DIR"/tg-liveness-lock-omega-tg-*; do
            if [ -d "$state" ]; then rmdir -- "$state" 2>/dev/null || true
            elif [ -e "$state" ]; then rm -f -- "$state"
            fi
        done
        echo "tg-liveness: all Telegram circuit breakers reset"
        return 0
    fi
    [[ "$requested" == *.service ]] || requested="${requested}.service"
    valid_unit "$requested" || {
        echo "usage: omega-atlas-liveness.sh --reset <omega-tg-bot.service|omega-tg-agent-ID.service|all>" >&2
        return 2
    }
    local stem="${requested%.service}"
    rm -f -- \
        "$STATE_DIR/tg-liveness-circuit-$stem" \
        "$STATE_DIR/tg-liveness-restarts-$stem" \
        "$STATE_DIR/tg-liveness-restart-$stem"
    rmdir -- "$STATE_DIR/tg-liveness-lock-$stem" 2>/dev/null || true
    echo "tg-liveness: circuit breaker reset for $requested"
}

if [ "${1:-}" = "--reset" ]; then
    reset_circuit "${2:-}"
    exit $?
fi

pending() {  # $1 = token
    [[ "$1" =~ ^[0-9]+:[A-Za-z0-9_-]+$ ]] || return 1
    printf 'url = "%s"\n' "${API_BASE}${1}/getWebhookInfo" \
        | curl -s --max-time 10 --config - \
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
    valid_unit "$unit" || continue
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
sleep "$DRAIN_SECONDS"

atomic_state() { # $1 target, $2 content
    local target="$1" content="$2" tmp
    mkdir -p "$STATE_DIR"
    tmp="$(mktemp "$STATE_DIR/.tg-liveness-state.XXXXXX")" || return 1
    chmod 0600 "$tmp"
    if ! printf '%s' "$content" > "$tmp" || ! mv -f -- "$tmp" "$target"; then
        rm -f -- "$tmp"
        return 1
    fi
}

send_alert() { # $1 message
    [ -x "$ALERT_BIN" ] && bash "$ALERT_BIN" "$1" >/dev/null 2>&1 || true
}

handle_suspect() (
    unit="$1" token="$2" label="$3" p1="$4" mpid="$5"
    valid_unit "$unit" || { echo "tg-liveness: refusing invalid unit $unit" >&2; return 2; }
    mkdir -p "$STATE_DIR"
    stem="${unit%.service}"
    lock="$STATE_DIR/tg-liveness-lock-$stem"
    if ! mkdir "$lock" 2>/dev/null; then
        lock_age=$(( $(date +%s) - $(stat -c %Y "$lock" 2>/dev/null || echo 0) ))
        if [ "$lock_age" -ge "$LOCK_STALE_SECONDS" ] && rmdir "$lock" 2>/dev/null && mkdir "$lock" 2>/dev/null; then
            echo "[$(date '+%F %T')] tg-liveness: recovered stale probe lock for $unit"
        else
            echo "[$(date '+%F %T')] tg-liveness: another probe owns $unit; skipping"
            return 0
        fi
    fi
    trap 'rmdir "$lock" 2>/dev/null || true' EXIT

    circuit="$STATE_DIR/tg-liveness-circuit-$stem"
    history="$STATE_DIR/tg-liveness-restarts-$stem"
    legacy_flag="$STATE_DIR/tg-liveness-restart-$stem"
    if [ -f "$circuit" ]; then
        echo "[$(date '+%F %T')] tg-liveness: CIRCUIT OPEN for $unit; no restart (repair, then run --reset $unit)"
        return 0
    fi

    p2="$(pending "$token")"
    [ -n "${p2:-}" ] || return 0
    [ "$p2" -gt 0 ] 2>/dev/null || return 0      # drained → was just busy, healthy
    [ "$p2" -ge "$p1" ] 2>/dev/null || return 0  # still draining → consuming, healthy
    busy "$mpid" && return 0                      # a task started during the wait

    now="$(date +%s)"
    cutoff=$(( now - WINDOW_SECONDS ))
    recent="$(awk -v cutoff="$cutoff" '$1 ~ /^[0-9]+$/ && $1 >= cutoff { print $1 }' "$history" 2>/dev/null)"
    restart_count="$(printf '%s\n' "$recent" | awk 'NF { n++ } END { print n + 0 }')"
    last_restart="$(printf '%s\n' "$recent" | tail -n 1)"
    if [ -n "$last_restart" ] && [ $(( now - last_restart )) -lt "$COOLDOWN_SECONDS" ]; then
        return 0
    fi
    # Honour the legacy cooldown stamp during the first run after upgrading.
    if [ ! -s "$history" ] && [ -f "$legacy_flag" ]; then
        legacy_age=$(( now - $(stat -c %Y "$legacy_flag" 2>/dev/null || echo 0) ))
        [ "$legacy_age" -lt "$COOLDOWN_SECONDS" ] && return 0
    fi

    if [ "$restart_count" -ge "$MAX_RESTARTS" ]; then
        atomic_state "$circuit" "OPEN $now"$'\n' || return 1
        echo "[$(date '+%F %T')] tg-liveness: CIRCUIT OPEN for $unit after $restart_count restart(s) in ${WINDOW_SECONDS}s; no restart"
        send_alert "━━━━━━━━━━━━
<b>Ω  TELEGRAM CIRCUIT OPEN</b>
━━━━━━━━━━━━
 ⛔ ${label} stayed deaf after ${restart_count} restart(s) in ${WINDOW_SECONDS}s. Automatic restarts are stopped. Repair the root cause, then run: <code>omega-atlas-liveness.sh --reset ${unit}</code>."
        return 0
    fi

    echo "[$(date '+%F %T')] tg-liveness: DEAF ${label} (pending ${p1}→${p2}, pid ${mpid}, no agent child) → restarting ${unit}"
    if systemctl --user restart "$unit"; then
        next_history="${recent}${recent:+$'\n'}${now}"$'\n'
        atomic_state "$history" "$next_history" || return 1
        atomic_state "$legacy_flag" "$now"$'\n' || return 1
        send_alert "━━━━━━━━━━━━
<b>Ω  TELEGRAM AUTO-HEAL</b>
━━━━━━━━━━━━
 🔄 ${label} was alive but not consuming (${p1} msg stuck). Poll loop restarted ($(( restart_count + 1 ))/${MAX_RESTARTS} in the current window)."
    else
        echo "[$(date '+%F %T')] tg-liveness: restart FAILED for $unit" >&2
        send_alert "OmegaOS Telegram auto-heal could not restart ${label} (${unit}). Manual intervention required."
        return 1
    fi
)

while IFS=$'\t' read -r unit token label p1 mpid; do
    [ -n "${unit:-}" ] || continue
    handle_suspect "$unit" "$token" "$label" "$p1" "$mpid"
done <<< "$SUSPECT"
