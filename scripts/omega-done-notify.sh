#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — end-of-mission notifier (done.json → Telegram)
# ───────────────────────────────────────────────────────────────────────────
# Watches ~/.omega/state/oracle-*.done.json. When an oracle finishes (it writes
# done.json via `omega done <session> <status> <summary>`), this relays the report
# to Telegram — to the project's forum TOPIC in the hub group if one is mapped, else
# the operator DM. Idempotent per STATUS: each done.json is notified once per status
# (the marker stores the notified status, so a gate-pending upgrade pending→done_clean
# re-arms exactly one corrected report), marked ONLY on a
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
# Forum hub group (topics live HERE, not in the DM) + the Atlas topic — where reports
# that don't belong to any project (e.g. OmegaOS-self) are posted instead of the DM.
HUB="$(python3 -c "import json;print(json.load(open('$GFILE')).get('hub',''))" 2>/dev/null || true)"
ATLAS="$(python3 -c "import json;print(json.load(open('$GFILE')).get('atlas_topic','') or '')" 2>/dev/null || true)"

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

    # Extract every field robustly (one python read; NUL-safe, no fragile word-split).
    eval "$(python3 - "$f" <<'PY' 2>/dev/null
import json,sys,shlex
try:
    d=json.load(open(sys.argv[1])); ship=d.get("ship") or {}
    def q(k,v): print(f"{k}={shlex.quote(str(v))}")
    q("STATUS", d.get("status","done"))
    q("ORACLE", d.get("oracle","?"))
    q("PROJECT", d.get("project","?"))
    q("MISSION", " ".join(str(d.get("mission","")).split())[:300])
    q("SUMMARY", " ".join(str(d.get("summary","")).split())[:2600])
    q("COMMIT", (ship.get("commit") or "")[:12])
    q("DEPLOY", ship.get("deploy_url") or "")
    dur=int(d.get("duration_secs") or 0)
    q("DUR", f"{dur//60}m{dur%60:02d}s" if dur else "")
    pa=d.get("pending_actions") or []
    q("PENDING", " • ".join(str(x) for x in pa)[:600])
    print("OK=1")
except Exception:
    print("OK=0")
PY
)"
    [ "${OK:-0}" = "1" ] || continue

    # Content-keyed skip: the marker records WHICH status was notified. An
    # in-place rewrite of the signal (the L4 gate-pending upgrade flips
    # pending → done_clean) changes the status and re-arms the notification —
    # this closes the read→send→mark race that a marker delete alone cannot
    # (a notifier mid-send could re-create the marker AFTER the upgrade's
    # invalidation, permanently suppressing the corrected done_clean).
    # Legacy empty markers (pre-content-key) count as matching so already-
    # notified missions are not re-sent on deploy.
    if [ -f "$marker" ]; then
        sent="$(cat "$marker" 2>/dev/null || true)"
        if [ -z "$sent" ] || [ "$sent" = "$STATUS" ]; then continue; fi
    fi

    # Symbol aesthetic — no emoji.
    case "$STATUS" in
        done_clean) sym="✓"; label="mission accomplie";;
        failed)     sym="✗"; label="mission échouée";;
        blocked)    sym="‖"; label="mission bloquée";;
        pending)    sym="…"; label="mission incomplète";;
        *)          sym="▪"; label="mission terminée";;
    esac
    esc() { printf '%s' "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }

    # ── Report card v3: status glyph + full bar + summary (long → expandable
    #    blockquote, collapsed by default) + subtle meta footer. Symbols only. ────
    sum_esc="$(esc "$SUMMARY")"
    if [ "${#SUMMARY}" -gt 280 ]; then
        body="<blockquote expandable>${sum_esc}</blockquote>"
    else
        body="${sum_esc}"
    fi
    msg="${sym} <b>$(esc "$PROJECT")</b> · ${label}
<code>██████████</code> 100%

${body}"
    [ -n "$PENDING" ] && msg="${msg}

<b>Reste :</b> $(esc "$PENDING")"
    # PROD VERIFY (R-PROD): probe the deploy URL before the report implies it's live.
    if [ -n "$DEPLOY" ]; then
        code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 8 "$DEPLOY" 2>/dev/null || echo 000)"
        if [ "$code" = "200" ] || [ "$code" = "301" ] || [ "$code" = "302" ]; then
            msg="${msg}

✓ <a href=\"$(esc "$DEPLOY")\">$(esc "$DEPLOY")</a> <code>${code}</code>"
        else
            msg="${msg}

⚠️ deploy injoignable (<code>${code}</code>) — $(esc "$DEPLOY")"
        fi
    fi
    foot="<i>$(esc "$ORACLE")"
    [ -n "$DUR" ]    && foot="${foot} · ${DUR}"
    foot="${foot}</i>"
    [ -n "$COMMIT" ] && foot="${foot} · <code>$(esc "$COMMIT")</code>"
    msg="${msg}

${foot}"

    # Route (2 levels): a mapped project TOPIC → hub+thread; else the operator DM.
    # NEVER the Atlas topic — Atlas is discussion + briefing only, never mission
    # reports (operator doctrine: alerts/tracking/reports live in their own topics,
    # Atlas stays a clean conversation channel). An unmapped-project report
    # (e2etest, OmegaOS-self before its topic existed) goes to the DM, not Atlas.
    thread="$(topic_for "$PROJECT")"
    if [ -n "$thread" ] && [ -n "${HUB:-}" ]; then
        target="$HUB"; via="topic ${thread}"
    else
        target="$DM"; thread=""; via="DM"
    fi
    if send_tg "$target" "$msg" "$thread"; then
        printf '%s' "$STATUS" > "$marker"
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) notified ${ORACLE} (${STATUS}) → ${via}"
    else
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) SEND FAILED ${ORACLE} → ${via} — will retry"
    fi
done
