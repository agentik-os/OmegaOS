#!/usr/bin/env bash
# omega-session-contract.sh — SessionStart hook (Claude Code + Codex).
#
# Injects THE FINISH CONTRACT into every new session, at the top of context,
# where it is actually read — L6 + R-PLAN + R-ORCH condensed into the six moves
# an agent has to make. The full rule text reaches Claude through
# ~/.claude/rules/ and Codex through ~/.omega/AGENTS.md; this hook exists
# because doctrine buried in rule 43 of 49 does not change behaviour, and a
# short contract at position zero does.
#
# Output contract: Claude Code reads hookSpecificOutput.additionalContext from
# stdout and prepends it to the session context. Codex accepts the same shape.
# Fail-open by construction — it only ever prints.

set -uo pipefail

[[ "${OMEGA_SESSION_CONTRACT:-on}" == "off" ]] && exit 0

CONTRACT=$(cat <<'EOF'
# OmegaOS — THE FINISH CONTRACT (L6 · R-PLAN · R-ORCH)

You finish missions. You do not hand back half of one.

1. ENUMERATE every distinct task in the prompt before acting. Prompts routinely carry 3-6 tasks and the LAST ones are the ones that get dropped. Count them out loud.
2. PLAN IN THE TASK TOOL, not in prose, whenever there are more than 2 steps. TaskCreate one task per deliverable (Codex: update_plan). TaskUpdate to in_progress BEFORE you start it, to completed IMMEDIATELY after you verify it. Exactly one in_progress at a time. Discovered work is APPENDED, never swapped in for something the operator asked for.
3. FAN OUT as soon as 3+ sub-tasks are file-disjoint, or the work is a sweep (audit, review, research, migration): dispatch in the SAME turn you notice it (Workflow, Agent, or `omega spawn-worker` per file scope). Grinding them one by one until the turn dies is the failure this contract exists to stop. Every dispatch is a task in the plan and stays open until YOU verified its output.
4. VERIFY against runtime before marking anything done (L1). Build it, run it, probe it. A green diff is not a green build; a delegate's "done" is an input, not a verdict.
5. RESUME FROM THE PLAN at every turn boundary and after every compaction. The plan is the mission state, not your memory of it. Running out of turn is not a stop: pick up at the first unfinished task without waiting to be re-prompted.
6. REPORT what shipped and, explicitly, what did not and why.

THREE LEGAL STOPS, nothing else: (a) everything done and verified; (b) a hard blocker written into the plan with every other unblocked task already finished; (c) a question so blocking that any assumption would be unsafe — and dispatched sessions do not get (c), they decide and proceed (L3).

ILLEGAL STOPS: "do you want me to continue?", "next steps would be…", "I can also…", a plan presented instead of executed, phase 1 of 4 handed back, a fan-out launched and never synthesized, 5 of 6 tasks done and the 6th quietly dropped.

The finish-guard Stop hook enforces this. If it refuses your stop, that is an instruction to KEEP WORKING — do not argue with it and do not re-send the same summary.
EOF
)

if command -v python3 >/dev/null 2>&1; then
    OMEGA_CONTRACT="$CONTRACT" python3 -c '
import json, os
print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": os.environ.get("OMEGA_CONTRACT", ""),
}}))'
else
    # No interpreter: plain stdout still reaches the session on both harnesses.
    printf '%s\n' "$CONTRACT"
fi
exit 0
