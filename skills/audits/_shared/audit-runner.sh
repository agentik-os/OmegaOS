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
#   audit-runner.sh <audit-name> <project-path> \
#     --user-need="..." --hinge="..." [--ticket=ID] [--files="..."]
#   audit-runner.sh <audit-name> <project-path> \
#     --user-need="..." --hinge="..." --finalize [--threshold=70]
#
# OUTPUT (in $project-path/.{audit}/):
#   raw/                  — programmatic tool outputs (JSON)
#   evidence-summary.json — structured data the LLM consumes
#   verdict.json          — LLM's score + findings + fix plan
#
# EXIT:
#   Gather mode: 0 = evidence is valid and ready for the LLM
#   Finalize mode: 0 = verdict score meets threshold, 1 = below threshold
#   Any mode: 2 = invalid contract, gather failure, or invalid/missing evidence

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
FINALIZE=0
THRESHOLD="70"
for arg in "$@"; do
    case "$arg" in
        --ticket=*) TICKET="${arg#--ticket=}" ;;
        --files=*) FILES="${arg#--files=}" ;;
        --user-need=*) USER_NEED="${arg#--user-need=}" ;;
        --url=*) URL="${arg#--url=}" ;;
        --hinge=*) HINGE="${arg#--hinge=}" ;;
        --threshold=*) THRESHOLD="${arg#--threshold=}" ;;
        --finalize) FINALIZE=1 ;;
        *)
            echo "AUDIT-RUNNER: unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ ! -d "$PROJECT_PATH" ]; then
    echo "AUDIT-RUNNER: project path is not a directory: $PROJECT_PATH" >&2
    exit 2
fi
if [ -z "$USER_NEED" ]; then
    echo "AUDIT-RUNNER: --user-need is required" >&2
    exit 2
fi
if [ -z "$HINGE" ]; then
    echo "AUDIT-RUNNER: --hinge is required" >&2
    exit 2
fi
if ! python3 - "$THRESHOLD" <<'PY'
import math
import sys

try:
    value = float(sys.argv[1])
except ValueError:
    raise SystemExit(1)
raise SystemExit(0 if math.isfinite(value) and 0 <= value <= 100 else 1)
PY
then
    echo "AUDIT-RUNNER: --threshold must be a number from 0 to 100" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OMEGA_ROOT="${OMEGA_DIR:-$HOME/.omega}"
REGISTRY="$OMEGA_ROOT/skills/audits/registry.toml"
if [ ! -f "$REGISTRY" ]; then
    REGISTRY="$SCRIPT_DIR/../registry.toml"
fi
if [ ! -f "$REGISTRY" ]; then
    echo "AUDIT-RUNNER: canonical audit registry not found" >&2
    exit 2
fi

CANONICAL_AUDIT="$(
    python3 - "$REGISTRY" "$AUDIT" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as handle:
    registry = tomllib.load(handle)
requested = sys.argv[2].strip()
ids = {entry["id"] for entry in registry.get("audits", [])}
candidates = (requested, f"{requested}audit")
for candidate in candidates:
    if candidate in ids:
        print(candidate)
        raise SystemExit(0)
raise SystemExit(1)
PY
)" || {
    echo "AUDIT-RUNNER: unknown audit '$AUDIT' in $REGISTRY" >&2
    exit 2
}

cd "$PROJECT_PATH" || {
    echo "AUDIT-RUNNER: cd $PROJECT_PATH failed" >&2
    exit 2
}

# Keep the audit protocol's established artifact alias (`.code`, `.sec`, ...)
# while recording the canonical registry id inside every evidence envelope.
ARTIFACTS=".${AUDIT}"
[ -n "$TICKET" ] && ARTIFACTS=".linear-fix/${TICKET}/.${AUDIT}"
mkdir -p "$ARTIFACTS/raw"

LOG="$ARTIFACTS/runner.log"
echo "[$(date -Iseconds)] audit-runner $CANONICAL_AUDIT on $PROJECT_PATH" >> "$LOG"

if [ "$FINALIZE" -eq 1 ]; then
    VERDICT_JSON="$ARTIFACTS/verdict.json"
    if [ ! -f "$VERDICT_JSON" ]; then
        echo "AUDIT-RUNNER: missing final verdict: $PROJECT_PATH/$VERDICT_JSON" >&2
        exit 2
    fi
    FINAL_SCORE="$(
        python3 - "$VERDICT_JSON" "$CANONICAL_AUDIT" "$USER_NEED" "$HINGE" <<'PY'
import json
import math
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    verdict = json.load(handle)
if not isinstance(verdict, dict):
    raise SystemExit(1)
if verdict.get("skill_used") != sys.argv[2]:
    raise SystemExit(1)
user_need = verdict.get("user_need_match")
if not isinstance(user_need, dict) or user_need.get("quote") != sys.argv[3]:
    raise SystemExit(1)
if not isinstance(user_need.get("addressed"), bool):
    raise SystemExit(1)
if verdict.get("confidence") not in {"low", "medium", "high"}:
    raise SystemExit(1)
if not isinstance(verdict.get("falsifiable_tests"), list):
    raise SystemExit(1)
if not isinstance(verdict.get("hinge_findings"), list):
    raise SystemExit(1)
if not isinstance(verdict.get("issues_found_and_fixed"), list):
    raise SystemExit(1)
if not isinstance(verdict.get("confidence_basis"), str):
    raise SystemExit(1)
hinge = sys.argv[4]
hinge_findings = verdict["hinge_findings"]
if hinge_findings and not any(
    isinstance(finding, dict)
    and isinstance(finding.get("location"), str)
    and (
        hinge in finding["location"]
        or finding["location"] in hinge
        or verdict.get("hinge") == hinge
    )
    for finding in hinge_findings
):
    raise SystemExit(1)
score = verdict.get("score")
if isinstance(score, bool) or not isinstance(score, (int, float)):
    raise SystemExit(1)
score = float(score)
if not math.isfinite(score) or not 0 <= score <= 100:
    raise SystemExit(1)
print(score)
PY
    )" || {
        echo "AUDIT-RUNNER: verdict.json is invalid or does not match this audit contract" >&2
        exit 2
    }
    echo "[$(date -Iseconds)] FINALIZE score=$FINAL_SCORE threshold=$THRESHOLD" >> "$LOG"
    if python3 - "$FINAL_SCORE" "$THRESHOLD" <<'PY'
import sys
raise SystemExit(0 if float(sys.argv[1]) >= float(sys.argv[2]) else 1)
PY
    then
        echo "AUDIT-RUNNER $CANONICAL_AUDIT: PASS score=$FINAL_SCORE threshold=$THRESHOLD"
        exit 0
    fi
    echo "AUDIT-RUNNER $CANONICAL_AUDIT: NEEDS WORK score=$FINAL_SCORE threshold=$THRESHOLD"
    exit 1
fi

# ─────────────────────────────────────────────────────────────────
# Phase 1 — Programmatic gather
# ─────────────────────────────────────────────────────────────────
GATHER_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${AUDIT}.sh"
if [ ! -x "$GATHER_SCRIPT" ]; then
    GATHER_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${CANONICAL_AUDIT}.sh"
fi
if [ -x "$GATHER_SCRIPT" ]; then
    echo "[$(date -Iseconds)] PHASE 1 gather: $GATHER_SCRIPT" >> "$LOG"
    "$GATHER_SCRIPT" "$ARTIFACTS/raw" "$PROJECT_PATH" "$FILES" "$URL" 2>&1 | tee -a "$LOG"
    GATHER_RC="${PIPESTATUS[0]}"
    echo "[$(date -Iseconds)] PHASE 1 done rc=$GATHER_RC" >> "$LOG"
    if [ "$GATHER_RC" -ne 0 ]; then
        echo "AUDIT-RUNNER: gather failed with exit code $GATHER_RC" >&2
        exit 2
    fi
    GATHER_MODE="programmatic"
else
    echo "[$(date -Iseconds)] PHASE 1 LLM-only: no gather script" >> "$LOG"
    GATHER_MODE="llm-only"
fi

# ─────────────────────────────────────────────────────────────────
# Phase 2 — Build evidence-summary.json from raw/
# ─────────────────────────────────────────────────────────────────
SUMMARY_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${AUDIT}-summarize.py"
if [ ! -x "$SUMMARY_SCRIPT" ]; then
    SUMMARY_SCRIPT="$OMEGA_ROOT/lib/audit-gather/${CANONICAL_AUDIT}-summarize.py"
fi
SUMMARY_JSON="$ARTIFACTS/evidence-summary.json"
if [ -x "$SUMMARY_SCRIPT" ]; then
    if ! "$SUMMARY_SCRIPT" "$ARTIFACTS/raw" > "$SUMMARY_JSON" 2>>"$LOG"; then
        echo "[$(date -Iseconds)] PHASE 2 summarizer failed" >> "$LOG"
        echo "AUDIT-RUNNER: evidence summarizer failed" >&2
        exit 2
    fi
else
    python3 - "$ARTIFACTS/raw" "$SUMMARY_JSON" <<'PY' 2>>"$LOG"
import json
import os
import sys

raw_dir, output = sys.argv[1:3]
tool_outputs = []
for name in sorted(os.listdir(raw_dir)):
    path = os.path.join(raw_dir, name)
    if os.path.isfile(path):
        tool_outputs.append({"file": name, "size": os.path.getsize(path)})
with open(output, "w", encoding="utf-8") as handle:
    json.dump({"tool_outputs": tool_outputs}, handle, indent=2)
    handle.write("\n")
PY
fi

if ! python3 - "$SUMMARY_JSON" "$CANONICAL_AUDIT" "$USER_NEED" "$HINGE" "$GATHER_MODE" "$REGISTRY" <<'PY' 2>>"$LOG"
import json
import os
import sys

path, audit, user_need, hinge, mode, registry = sys.argv[1:7]
with open(path, encoding="utf-8") as handle:
    evidence = json.load(handle)
if not isinstance(evidence, dict):
    raise SystemExit(1)
document = {
    "schema_version": 1,
    "audit": audit,
    "user_need": user_need,
    "hinge": hinge,
    "provenance": {
        "runner": "omega-audit-runner",
        "gather_mode": mode,
        "registry": os.path.abspath(registry),
    },
    "evidence": evidence,
}
temporary = f"{path}.tmp"
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
os.replace(temporary, path)
PY
then
    echo "AUDIT-RUNNER: evidence-summary.json is invalid" >&2
    exit 2
fi

# ─────────────────────────────────────────────────────────────────
# Phase 3 — LLM is invoked by the audit skill itself.
# audit-runner.sh just guarantees the evidence is ready before the LLM
# starts reasoning. The skill .md template now reads $SUMMARY_JSON instead
# of doing manual greps.
# ─────────────────────────────────────────────────────────────────

echo ""
echo "─────────────────────────────────────────────────────"
echo " AUDIT-RUNNER ${CANONICAL_AUDIT}: evidence ready"
echo " Evidence: $PROJECT_PATH/$SUMMARY_JSON"
echo " Raw outputs: $PROJECT_PATH/$ARTIFACTS/raw/"
echo " LLM phase: produce verdict.json, then rerun with --finalize."
echo "─────────────────────────────────────────────────────"

exit 0
