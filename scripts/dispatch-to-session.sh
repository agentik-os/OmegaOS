#!/usr/bin/env zsh
# OmegaOS — Dispatch prompt to a worker session
# Usage: dispatch-to-session.sh <SESSION_NAME> "prompt" [workdir]

set -euo pipefail

SESSION="${1:?Usage: dispatch-to-session.sh <SESSION> \"prompt\" [workdir]}"
PROMPT="${2:?Usage: dispatch-to-session.sh <SESSION> \"prompt\" [workdir]}"
WORKDIR="${3:-$(pwd)}"

OMEGA_STATE="${OMEGA_STATE:-$HOME/.omega/state}"
OMEGA_LOGS="${OMEGA_LOGS:-$HOME/.omega/logs}"
OMEGA_AGENT="${OMEGA_AGENT:-claude}"

mkdir -p "$OMEGA_STATE" "$OMEGA_LOGS"

TIMESTAMP=$(date -Iseconds)

# Autonomy banner for dispatched workers
FULL_PROMPT="[DISPATCHED] Third Law: decide and proceed, never wait for confirmation.

${PROMPT}

When done: omega done ${SESSION} done_clean \"<summary>\""

# Create worker session
omega new "$SESSION" \
  --dir "$WORKDIR" \
  --cmd "${OMEGA_AGENT} -p '${FULL_PROMPT}'" 2>&1

echo "✓ Worker dispatched: $SESSION"
echo "  Workdir: $WORKDIR"

# Log dispatch
echo "[$TIMESTAMP] DISPATCH worker=$SESSION workdir=$WORKDIR prompt=\"${PROMPT:0:120}\"" \
  >> "${OMEGA_LOGS}/dispatch.log"
