#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — end-of-mission notifier (done.json → Telegram)
# ───────────────────────────────────────────────────────────────────────────
# Watches ~/.omega/state/oracle-*.done.json. When an oracle finishes (it writes
# done.json via `omega done <session> <status> <summary>`), this relays the report
# to Telegram — to the project's forum TOPIC in the hub group if one is mapped, else
# the operator DM. Idempotent: each done.json is notified once, marked ONLY on a
# confirmed send (a failed send retries next tick). 1-min cron (OMEGA-CRON-DONE-
# NOTIFY-v1). The single, restart-proof notification path — catches EVERY oracle
# (Telegram-, TUI-, or Atlas-dispatched). Closing the oracle is owned by `omega done`.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STATE="$OMEGA_DIR/state"
TG_TOML="$OMEGA_DIR/telegram.toml"
GFILE="$OMEGA_DIR/telegram-groups.json"

[ -d "$STATE" ] || exit 0
TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
DM="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] && [ -n "${DM:-}" ] || exit 0   # no telegram configured → nothing to do
# Forum hub group (topics live HERE, not in the DM).
HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"

# Send to a chat (+optional topic thread). HTML, with a plain-text retry on parse
# error so a report is never silently dropped. Returns success/failure.
send_tg() { # $1=chat_id  $2=text  $3=thread(optional)
    local a=(--data-urlencode "chat_id=$1" --data-urlencode "text=$2" --data-urlencode "parse_mode=HTML")
    [ -n "${3:-}" ] && a+=(--data-urlencode "message_thread_id=$3")
    local r; r="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${a[@]}")"
    if ! printf '%s' "$r" | grep -q '"ok":true'; then
        local p=(--data-urlencode "chat_id=$1" --data-urlencode "text=$2")
        [ -n "${3:-}" ] && p+=(--data-urlencode "message_thread_id=$3")
        r="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${p[@]}")"
    fi
    printf '%s' "$r" | grep -q '"ok":true'
}

# Topic id in the hub for a project (telegram-groups.json: {topics:{"<id>":"<proj>"}}).
# Tries the exact project, then the session-suffix-stripped name (oracle-<p>-<n>).
topic_for() {
    [ -f "$GFILE" ] || return 0
    python3 - "$GFILE" "$1" <<'PY' 2>/dev/null
import json,sys,re
try:
    g=json.load(open(sys.argv[1])); want=sys.argv[2].lower()
    cands=[want, re.sub(r'-[0-9]+$','',want)]
    for tid,name in (g.get("topics") or {}).items():
        if str(name).lower() in cands: print(tid); break
except Exception: pass
PY
}

for f in "$STATE"/oracle-*.done.json; do
    [ -e "$f" ] || continue
    marker="${f}.notified"
    [ -f "$marker" ] && continue

    # Extract every field robustly (one python read; NUL-safe, no fragile word-split).
    eval "$(python3 - "$f" <<'PY' 2>/dev/null
import json,sys,shlex
try:
    d=json.load(open(sys.argv[1])); ship=d.get("ship") or {}
    def q(k,v): print(f"{k}={shlex.quote(str(v))}")
    q("STATUS", d.get("status","done"))
    q("ORACLE", d.get("oracle","?"))
    q("PROJECT", d.get("project","?"))
    q("SUMMARY", " ".join(str(d.get("summary","")).split())[:2800])
    q("COMMIT", ship.get("commit") or "")
    q("DEPLOY", ship.get("deploy_url") or "")
    print("OK=1")
except Exception:
    print("OK=0")
PY
)"
    [ "${OK:-0}" = "1" ] || continue

    case "$STATUS" in
        done_clean) icon="✅";; failed) icon="❌";; blocked) icon="🚧";; pending) icon="⏳";; *) icon="⏹";;
    esac
    esc() { printf '%s' "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }
    extra=""
    [ -n "$COMMIT" ] && extra="${extra}
Commit: <code>$(esc "$COMMIT")</code>"
    [ -n "$DEPLOY" ] && extra="${extra}
🌐 $(esc "$DEPLOY")"
    msg="<b>${icon} Oracle $(esc "$ORACLE") — $(esc "$PROJECT")</b>
Mission terminée (<b>${STATUS}</b>).

$(esc "$SUMMARY")${extra}"

    # Route: a mapped project topic → the HUB group + that thread; else → the operator DM.
    thread="$(topic_for "$PROJECT")"
    if [ -n "$thread" ] && [ -n "${HUB:-}" ]; then
        target="$HUB"; via="topic ${thread}"
    else
        target="$DM"; thread=""; via="DM"
    fi
    if send_tg "$target" "$msg" "$thread"; then
        : > "$marker"
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) notified ${ORACLE} (${STATUS}) → ${via}"
    else
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) SEND FAILED ${ORACLE} → ${via} — will retry"
    fi
done
