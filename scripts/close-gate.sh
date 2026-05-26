#!/usr/bin/env zsh
# OmegaOS — Quality gate: check if a session can be safely closed
# Usage: close-gate.sh check-worker <session>
#        close-gate.sh check-oracle <session>
#        close-gate.sh ack-worker <worker> <oracle>
#        close-gate.sh ack-oracle <oracle>
#
# Exit codes:
#   0 = safe to close
#   1 = blocked (upstream not acked)
#   2 = blocked (still running)

set -euo pipefail

ACTION="${1:?Usage: close-gate.sh <check-worker|check-oracle|ack-worker|ack-oracle> <session>}"
SESSION="${2:?Missing session name}"

OMEGA_STATE="${OMEGA_STATE:-$HOME/.omega/state}"

case "$ACTION" in
  check-worker)
    DONE_FILE="${OMEGA_STATE}/worker-${SESSION}.done.json"

    if [[ ! -f "$DONE_FILE" ]]; then
        # Check if session is still alive
        if omega list 2>/dev/null | grep -q "$SESSION"; then
            echo "BLOCKED: worker $SESSION still running (no done.json)"
            exit 2
        else
            echo "SAFE: worker $SESSION gone (no session, no done.json)"
            exit 0
        fi
    fi

    # Check for ack
    ACK_FILE="${OMEGA_STATE}/oracle-worker-${SESSION}.acked.json"
    if [[ -f "$ACK_FILE" ]]; then
        echo "SAFE: worker $SESSION done + acked"
        exit 0
    else
        echo "BLOCKED: worker $SESSION done but not acked by oracle"
        exit 1
    fi
    ;;

  check-oracle)
    DONE_FILE="${OMEGA_STATE}/${SESSION}.done.json"

    if [[ ! -f "$DONE_FILE" ]]; then
        if omega list 2>/dev/null | grep -q "$SESSION"; then
            echo "BLOCKED: oracle $SESSION still running"
            exit 2
        else
            echo "SAFE: oracle $SESSION gone"
            exit 0
        fi
    fi

    # Check aisb_reported flag
    if python3 -c "
import json, sys
d = json.load(open('$DONE_FILE'))
sys.exit(0 if d.get('aisb_reported', False) else 1)
" 2>/dev/null; then
        echo "SAFE: oracle $SESSION done + reported"
        exit 0
    else
        echo "BLOCKED: oracle $SESSION done but not reported to AISB"
        exit 1
    fi
    ;;

  ack-worker)
    ORACLE="${3:?Usage: close-gate.sh ack-worker <worker> <oracle>}"
    DONE_FILE="${OMEGA_STATE}/worker-${SESSION}.done.json"

    if [[ ! -f "$DONE_FILE" ]]; then
        echo "ERROR: No done.json for worker $SESSION"
        exit 3
    fi

    ACK_FILE="${OMEGA_STATE}/oracle-worker-${SESSION}.acked.json"
    cat > "$ACK_FILE" <<EOF
{
  "worker": "$SESSION",
  "oracle": "$ORACLE",
  "acked_at": "$(date -Iseconds)",
  "done_source": "$DONE_FILE"
}
EOF
    echo "✓ Worker $SESSION acked by oracle $ORACLE"
    ;;

  ack-oracle)
    DONE_FILE="${OMEGA_STATE}/${SESSION}.done.json"

    if [[ ! -f "$DONE_FILE" ]]; then
        echo "ERROR: No done.json for oracle $SESSION"
        exit 3
    fi

    # Set aisb_reported flag
    python3 -c "
import json
with open('$DONE_FILE', 'r') as f:
    d = json.load(f)
d['aisb_reported'] = True
d['aisb_reported_at'] = '$(date -Iseconds)'
with open('$DONE_FILE', 'w') as f:
    json.dump(d, f, indent=2)
"
    echo "✓ Oracle $SESSION marked as reported"
    ;;

  *)
    echo "Unknown action: $ACTION"
    echo "Usage: close-gate.sh <check-worker|check-oracle|ack-worker|ack-oracle> <session>"
    exit 1
    ;;
esac
