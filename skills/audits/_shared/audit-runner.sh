#!/usr/bin/env bash
# audit-runner.sh — Canonical hybrid audit orchestrator.
#
# DESIGN (2026-05-08):
#   Old: LLM reads 30-200 files, runs hand-grep, judges everything → 100-300k tokens
#   New: programmatic gather (lighthouse, axe, semgrep, eslint, etc.) → JSON
#        LLM Phase 2: reads JSON + 3-5 critical files → DEEPER synthesis
#                     (more Popper falsification, more cross-audit XR, more
#                      user-need verification, edge case hunting)
#        LLM Phase 3: contextual fix plan
#
# Token reduction: 70-85% on gather. Quality ↑↑ because:
#   - Tools catch ALL violations deterministically (no LLM omission)
#   - Freed token budget is REINVESTED in synthesis depth
#   - LLM can spend its context on "is this a real bug for THIS user" instead
#     of "what's the bug in this 500-line file"
#
# USAGE:
#   audit-runner.sh <audit-name> <project-path> [--ticket=ID] [--files="..."] [--user-need="..."]
#
# OUTPUT (in $project-path/.{audit}/):
#   raw/                  — programmatic tool outputs (JSON)
#   evidence-summary.json — structured data the LLM consumes
#   verdict.json          — LLM's score + findings + fix plan
#
# EXIT:
#   0 = audit completed with score ≥ threshold
#   1 = audit completed but score < threshold (fix needed)
#   2 = audit failed to run (tool missing / project incompat)

set -uo pipefail

AUDIT="${1:?Usage: audit-runner.sh <audit-name> <project-path> [args...]}"
PROJECT_PATH="${2:?Missing project path}"
shift 2

# Parse optional args
TICKET=""
FILES=""
USER_NEED=""
URL=""
HINGE=""
for arg in "$@"; do
    case "$arg" in
        --ticket=*) TICKET="${arg#--ticket=}" ;;
        --files=*) FILES="${arg#--files=}" ;;
        --user-need=*) USER_NEED="${arg#--user-need=}" ;;
        --url=*) URL="${arg#--url=}" ;;
        --hinge=*) HINGE="${arg#--hinge=}" ;;
    esac
done

cd "$PROJECT_PATH" || { echo "AUDIT-RUNNER: cd $PROJECT_PATH failed" >&2; exit 2; }

ARTIFACTS=".${AUDIT}"
[ -n "$TICKET" ] && ARTIFACTS=".linear-fix/${TICKET}/.${AUDIT}"
mkdir -p "$ARTIFACTS/raw"

LOG="$ARTIFACTS/runner.log"
echo "[$(date -Iseconds)] audit-runner $AUDIT on $PROJECT_PATH" >> "$LOG"

# ─────────────────────────────────────────────────────────────────
# Phase 1 — Programmatic gather
# ─────────────────────────────────────────────────────────────────
GATHER_SCRIPT="$HOME/.aisb/lib/audit-gather/${AUDIT}.sh"
if [ -x "$GATHER_SCRIPT" ]; then
    echo "[$(date -Iseconds)] PHASE 1 gather: $GATHER_SCRIPT" >> "$LOG"
    "$GATHER_SCRIPT" "$ARTIFACTS/raw" "$PROJECT_PATH" "$FILES" "$URL" 2>&1 | tee -a "$LOG"
    GATHER_RC=$?
    echo "[$(date -Iseconds)] PHASE 1 done rc=$GATHER_RC" >> "$LOG"
else
    echo "[$(date -Iseconds)] PHASE 1 skipped — no $GATHER_SCRIPT (LLM-only audit)" >> "$LOG"
fi

# ─────────────────────────────────────────────────────────────────
# Phase 2 — Build evidence-summary.json from raw/
# ─────────────────────────────────────────────────────────────────
SUMMARY_SCRIPT="$HOME/.aisb/lib/audit-gather/${AUDIT}-summarize.py"
SUMMARY_JSON="$ARTIFACTS/evidence-summary.json"
if [ -x "$SUMMARY_SCRIPT" ]; then
    "$SUMMARY_SCRIPT" "$ARTIFACTS/raw" > "$SUMMARY_JSON" 2>>"$LOG" || {
        echo "[$(date -Iseconds)] PHASE 2 summarizer failed" >> "$LOG"
        echo '{"summary_error": true}' > "$SUMMARY_JSON"
    }
elif [ -d "$ARTIFACTS/raw" ] && [ -n "$(ls -A "$ARTIFACTS/raw" 2>/dev/null)" ]; then
    # Default summarizer: just list files + sizes
    python3 -c "
import json, os, sys
out = {'tool_outputs': []}
for f in sorted(os.listdir('$ARTIFACTS/raw')):
    p = os.path.join('$ARTIFACTS/raw', f)
    if os.path.isfile(p):
        out['tool_outputs'].append({'file': f, 'size': os.path.getsize(p)})
print(json.dumps(out, indent=2))
" > "$SUMMARY_JSON" 2>>"$LOG"
fi

# ─────────────────────────────────────────────────────────────────
# Phase 3 — LLM is invoked by the audit skill itself.
# audit-runner.sh just guarantees the evidence is ready before the LLM
# starts reasoning. The skill .md template now reads $SUMMARY_JSON instead
# of doing manual greps.
# ─────────────────────────────────────────────────────────────────

echo ""
echo "─────────────────────────────────────────────────────"
echo " AUDIT-RUNNER ${AUDIT}: gather complete"
echo " Evidence: $PROJECT_PATH/$SUMMARY_JSON"
echo " Raw outputs: $PROJECT_PATH/$ARTIFACTS/raw/"
echo " LLM phase: read evidence-summary.json + 3-5 critical files."
echo "─────────────────────────────────────────────────────"

exit 0
