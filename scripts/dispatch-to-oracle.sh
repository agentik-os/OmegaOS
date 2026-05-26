#!/usr/bin/env zsh
# OmegaOS — Dispatch mission to an oracle session
# Usage: dispatch-to-oracle.sh <PROJECT> "mission text"

set -euo pipefail

PROJECT="${1:?Usage: dispatch-to-oracle.sh <PROJECT> \"mission\"}"
MISSION="${2:?Usage: dispatch-to-oracle.sh <PROJECT> \"mission\"}"

OMEGA_STATE="${OMEGA_STATE:-$HOME/.omega/state}"
OMEGA_LOGS="${OMEGA_LOGS:-$HOME/.omega/logs}"
OMEGA_AGENT="${OMEGA_AGENT:-claude}"

mkdir -p "$OMEGA_STATE" "$OMEGA_LOGS"

# Find available oracle name (multi-oracle safe)
find_oracle_name() {
    local project="$1"
    local base="oracle-${project}"

    # Check if base oracle exists and is alive
    if omega list 2>/dev/null | grep -q "^.*${base}$"; then
        # Base oracle exists, find next index
        local max_idx=1
        for sess in $(omega list 2>/dev/null | grep "oracle-${project}" | awk '{print $NF}'); do
            if [[ "$sess" =~ oracle-${project}-([0-9]+) ]]; then
                local idx="${match[1]}"
                (( idx > max_idx )) && max_idx=$idx
            fi
        done
        echo "oracle-${project}-$((max_idx + 1))"
    else
        echo "$base"
    fi
}

ORACLE_NAME=$(find_oracle_name "$PROJECT")
TIMESTAMP=$(date -Iseconds)

# Write context file
cat > "${OMEGA_STATE}/${ORACLE_NAME}.context.json" <<EOF
{
  "oracle": "$ORACLE_NAME",
  "project": "$PROJECT",
  "mission": $(printf '%s' "$MISSION" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))"),
  "dispatched_at": "$TIMESTAMP",
  "status": "running"
}
EOF

# Build oracle prompt
ORACLE_PROMPT="## Mission
${MISSION}

## Role: ORACLE for ${PROJECT}
You are the Oracle. Analyze, decompose, dispatch workers, verify quality.
Use \`omega spawn-worker <task> \"<prompt>\"\` to create workers.
Use \`omega done <session> done_clean \"<summary>\"\` when complete.

## Laws
1. Code lies. Only runtime tells the truth.
2. Be a researcher, not a sycophant. Challenge flawed premises.
3. Decide and proceed. Never wait for confirmation."

# Create oracle session with omega CLI
omega new "$ORACLE_NAME" --cmd "${OMEGA_AGENT} -p '${ORACLE_PROMPT}'" 2>&1

echo "✓ Oracle dispatched: $ORACLE_NAME"
echo "  Project: $PROJECT"
echo "  Mission: ${MISSION:0:80}..."

# Log dispatch
echo "[$TIMESTAMP] DISPATCH oracle=$ORACLE_NAME project=$PROJECT mission=\"${MISSION:0:120}\"" \
  >> "${OMEGA_LOGS}/dispatch.log"
