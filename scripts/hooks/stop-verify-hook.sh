#!/usr/bin/env bash
# stop-verify-hook.sh — Stop hook: nudge the agent to verify before finishing.
#
# If the session edited code (Write/Edit) but shows no verification (build/test
# pass), emit a reminder. Dependency-free (no ~/.aisb). Shipped by OmegaOS.
# Enforces Law L1 (runtime is truth) + L4 (done means 100%, verified).

# Only nudge inside a project tree.
case "$PWD" in
    *"/VibeCoding/"*|*"/projects/"*) ;;
    *) exit 0 ;;
esac

TRANSCRIPT="$1"
if [ -n "$TRANSCRIPT" ] && echo "$TRANSCRIPT" | grep -qE "Write|Edit"; then
    if ! echo "$TRANSCRIPT" | grep -qiE "verified|verify|build.*(pass|ok|0 error)|test.*pass|cargo (build|test)|npm run build"; then
        echo "REMINDER (Law L1/L4): verify your changes before saying done — run the build/tests, observe real output. 92% is not done."
    fi
fi
exit 0
