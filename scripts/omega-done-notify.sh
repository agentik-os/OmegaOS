#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — end-of-mission notifier (done.json → Telegram)
# ───────────────────────────────────────────────────────────────────────────
# Watches ~/.omega/state/oracle-*.done.json. When an oracle finishes its mission
# (it writes done.json via `omega done <session> <status> <summary>`), this relays
# the report to Telegram — to the project's topic if one is mapped, else the main
# chat. Idempotent: each done.json is notified once (tracked by a .notified marker
# next to it). Runs on a 1-min cron (installed by install.sh, marker
# OMEGA-CRON-DONE-NOTIFY-v1). This is the single, restart-proof notification path —
# it catches EVERY oracle (dispatched from Telegram, the TUI, or Atlas).
#
# The oracle CLOSE is owned by the patrol/lifecycle (an ephemeral oracle is closed
# once it has a real done-signal). This script only NOTIFIES; it never kills.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STATE="$OMEGA_DIR/state"
TG_TOML="$OMEGA_DIR/telegram.toml"
GROUPS="$OMEGA_DIR/telegram-groups.json"

[ -d "$STATE" ] || exit 0
TOKEN="$(grep -E '^[[:space:]]*bot_token' "$TG_TOML" 2>/dev/null | head -1 | cut -d'"' -f2)"
CHAT="$(grep -E '^[[:space:]]*chat_id' "$TG_TOML" 2>/dev/null | head -1 | grep -oE '\-?[0-9]+' | head -1)"
[ -n "${TOKEN:-}" ] && [ -n "${CHAT:-}" ] || exit 0   # no telegram configured → nothing to do

send_tg() { # $1=text  $2=thread(optional)
    local args=(--data-urlencode "chat_id=${CHAT}" --data-urlencode "text=$1" --data-urlencode "parse_mode=HTML")
    [ -n "${2:-}" ] && args+=(--data-urlencode "message_thread_id=$2")
    local resp; resp="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${args[@]}")"
    # If HTML parsing failed (unbalanced tag in a model-written summary), retry as
    # plain text so the report is NEVER silently dropped.
    if ! printf '%s' "$resp" | grep -q '"ok":true'; then
        local plain=(--data-urlencode "chat_id=${CHAT}" --data-urlencode "text=$1")
        [ -n "${2:-}" ] && plain+=(--data-urlencode "message_thread_id=$2")
        resp="$(curl -s "https://api.telegram.org/bot${TOKEN}/sendMessage" "${plain[@]}")"
    fi
    printf '%s' "$resp" | grep -q '"ok":true'   # return success/failure for the caller
}

# Topic id for a project, if the bot registered one (telegram-groups.json: {topics:{"<id>":"<proj>"}}).
topic_for() {
    [ -f "$GROUPS" ] || return 0
    python3 - "$GROUPS" "$1" <<'PY' 2>/dev/null
import json,sys
try:
    g=json.load(open(sys.argv[1])); proj=sys.argv[2].lower()
    for tid,name in (g.get("topics") or {}).items():
        if str(name).lower()==proj: print(tid); break
except Exception: pass
PY
}

for f in "$STATE"/oracle-*.done.json; do
    [ -e "$f" ] || continue
    marker="${f}.notified"
    [ -f "$marker" ] && continue                       # already notified

    read -r status oracle project summary commit deploy < <(python3 - "$f" <<'PY' 2>/dev/null
import json,sys
try:
    d=json.load(open(sys.argv[1]))
    ship=d.get("ship") or {}
    def one(s): return " ".join(str(s).split())
    print(d.get("status","done"), d.get("oracle","?"), d.get("project","?"),
          "\x1f"+one(d.get("summary",""))[:2500], (ship.get("commit") or "-"), (ship.get("deploy_url") or "-"))
except Exception:
    print("ERR")
PY
)
    [ "${status:-ERR}" = "ERR" ] && continue
    # summary was joined after \x1f to keep spaces; re-extract it properly.
    summary="$(python3 -c "import json;d=json.load(open('$f'));print(' '.join(d.get('summary','').split())[:2500])" 2>/dev/null)"

    case "$status" in
        done_clean) icon="✅";; failed) icon="❌";; blocked) icon="🚧";; pending) icon="⏳";; *) icon="⏹";;
    esac
    extra=""
    [ "$commit" != "-" ] && [ -n "$commit" ] && extra="${extra}
Commit: <code>${commit}</code>"
    [ "$deploy" != "-" ] && [ -n "$deploy" ] && extra="${extra}
🌐 ${deploy}"

    msg="<b>${icon} Oracle ${oracle} — ${project}</b>
Mission terminée (<b>${status}</b>).

${summary}${extra}"

    # Route to the project topic. Fall back to the suffix-stripped name (oracle-<p>-<n>)
    # so an older done.json whose project still carries the session index still routes.
    thread="$(topic_for "$project")"
    [ -z "$thread" ] && thread="$(topic_for "$(printf '%s' "$project" | sed -E 's/-[0-9]+$//')")"
    if send_tg "$msg" "$thread"; then
        : > "$marker"   # mark notified ONLY on a confirmed send (else retry next tick)
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) notified ${oracle} (${status}) thread=${thread:-main}"
    else
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) SEND FAILED ${oracle} — will retry"
    fi
done
