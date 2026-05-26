#!/usr/bin/env zsh
# OmegaOS — Mark a worker session as done
# Usage: worker-mark-done.sh <STATUS> "summary" [commit_hash]
# Called by workers when they finish their task.

set -euo pipefail

STATUS="${1:?Usage: worker-mark-done.sh <done_clean|pending|failed> \"summary\" [commit]}"
SUMMARY="${2:?Usage: worker-mark-done.sh <STATUS> \"summary\" [commit]}"
COMMIT="${3:-}"

OMEGA_STATE="${OMEGA_STATE:-$HOME/.omega/state}"

# Validate status
case "$STATUS" in
  done_clean|pending|failed) ;;
  *) echo "ERROR: Invalid status '$STATUS'. Use: done_clean, pending, failed" >&2; exit 1 ;;
esac

# Detect session name
if [[ -n "${OMEGA_SESSION:-}" ]]; then
    SESSION="$OMEGA_SESSION"
elif [[ -n "${RMUX_SESSION:-}" ]]; then
    SESSION="$RMUX_SESSION"
else
    SESSION="unknown-$(date +%s)"
fi

TIMESTAMP=$(date -Iseconds)

mkdir -p "$OMEGA_STATE"

# Atomic write: tmp then rename
DONE_TMP="${OMEGA_STATE}/.worker-${SESSION}.done.json.tmp"
DONE_FILE="${OMEGA_STATE}/worker-${SESSION}.done.json"

cat > "$DONE_TMP" <<EOF
{
  "session": "$SESSION",
  "status": "$STATUS",
  "summary": $(printf '%s' "$SUMMARY" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))"),
  "commit": $(if [[ -n "$COMMIT" ]]; then echo "\"$COMMIT\""; else echo "null"; fi),
  "finished_at": "$TIMESTAMP",
  "todos_total": 0,
  "todos_completed": 0,
  "pending_actions": [],
  "written_by": "worker-mark-done.sh"
}
EOF

mv "$DONE_TMP" "$DONE_FILE"

echo "✓ Done signal written: $DONE_FILE"
echo "  Status: $STATUS"
echo "  Summary: ${SUMMARY:0:80}"
