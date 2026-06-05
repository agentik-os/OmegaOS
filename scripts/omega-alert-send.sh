#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — canonical alert sender (Telegram "Alerts 🚨" topic)
# ───────────────────────────────────────────────────────────────────────────
# ONE funnel for every operational alert (stuck oracle, self-heal, token
# refresh failure, …): they all land in the dedicated Alerts forum topic —
# NEVER in the Atlas topic (Atlas = briefing + oracle reports + off-project
# work) and never scattered in the DM.
#
# The Alerts topic is UNDELETABLE by design: if it is missing or was deleted
# in the group, this script recreates it on the fly, persists the new id in
# telegram-groups.json, and resends. Fallback chain: alerts topic → DM.
#
# Usage: omega-alert-send.sh "<HTML text>"
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
TG_TOML="$OMEGA_DIR/telegram.toml"
GFILE="$OMEGA_DIR/telegram-groups.json"
MSG="${1:-}"
[ -n "$MSG" ] || { echo "usage: omega-alert-send.sh \"<html text>\"" >&2; exit 2; }

TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
DM="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] || exit 0
HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"
ALERTS="$(python3 -c "import json;print(json.load(open('$GFILE')).get('alerts_topic','') or '')" 2>/dev/null || true)"

api() { curl -s --max-time 15 "https://api.telegram.org/bot${TOKEN}/$1" "${@:2}"; }

create_alerts_topic() { # → echoes new thread id (and persists it), empty on failure
    local r tid
    r="$(api createForumTopic -d "chat_id=$HUB" --data-urlencode "name=Alerts 🚨" -d "icon_color=16478047")"
    tid="$(printf '%s' "$r" | python3 -c "import json,sys;d=json.load(sys.stdin);print(d['result']['message_thread_id'] if d.get('ok') else '')" 2>/dev/null)"
    [ -n "$tid" ] || return 1
    python3 - "$tid" "$GFILE" <<'PY' 2>/dev/null
import json,sys
tid, p = sys.argv[1], sys.argv[2]
g=json.load(open(p))
# Drop any stale alerts mapping, then record the fresh one.
g["topics"]={k:v for k,v in (g.get("topics") or {}).items() if str(v).lower()!="alerts"}
g["alerts_topic"]=int(tid); g["topics"][tid]="alerts"
json.dump(g,open(p,"w"),indent=2)
PY
    echo "$tid"
}

send_to() { # $1=chat $2=thread("" = none) → echoes API response
    local a=(--data-urlencode "chat_id=$1" --data-urlencode "text=$MSG" --data-urlencode "parse_mode=HTML" --data-urlencode "disable_web_page_preview=true")
    [ -n "${2:-}" ] && a+=(--data-urlencode "message_thread_id=$2")
    api sendMessage "${a[@]}"
}

if [ -n "${HUB:-}" ]; then
    # No alerts topic on file → create it first (undeletable invariant).
    [ -n "${ALERTS:-}" ] || ALERTS="$(create_alerts_topic || true)"
    if [ -n "${ALERTS:-}" ]; then
        res="$(send_to "$HUB" "$ALERTS")"
        if printf '%s' "$res" | grep -q '"ok":true'; then exit 0; fi
        # Topic deleted in the group → recreate + resend (the auto-heal).
        if printf '%s' "$res" | grep -qiE 'thread not found|TOPIC_DELETED|TOPIC_ID_INVALID'; then
            ALERTS="$(create_alerts_topic || true)"
            if [ -n "${ALERTS:-}" ]; then
                res="$(send_to "$HUB" "$ALERTS")"
                printf '%s' "$res" | grep -q '"ok":true' && exit 0
            fi
        fi
    fi
fi
# Last resort: operator DM (never lose an alert).
[ -n "${DM:-}" ] && send_to "$DM" "" >/dev/null
exit 0
