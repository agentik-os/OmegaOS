#!/usr/bin/env bash
# stop-verify-hook.sh — Stop hook: nudge the agent to verify before finishing.
#
# Reads the Stop-hook JSON on stdin (Claude Code convention), resolves the
# transcript FILE, and if the session edited code (Write/Edit) without any
# verification signal, emits a reminder. Dependency-free (no ~/.aisb).
# Enforces Law L1 (runtime is truth) + L4 (done means 100%, verified).

# Only nudge inside a project tree.
# (~/Station is the live projects root — the original VibeCoding/projects
# gate matched nothing on this layout, so the hook never fired.)
case "$PWD" in
    *"/VibeCoding/"*|*"/projects/"*|*"/Station/"*) ;;
    *) exit 0 ;;
esac

# Stop-hook input is JSON on stdin: { transcript_path, ... }. Resolve the file.
IN=""; [ ! -t 0 ] && IN=$(cat 2>/dev/null)
TP=$(echo "$IN" | jq -r '.transcript_path // empty' 2>/dev/null)
[ -z "$TP" ] && TP="$1"            # fallback: positional arg
[ -f "$TP" ] || exit 0             # no readable transcript → nothing to check

# Did the session edit code? (grep the FILE contents, not the path.)
if grep -qE '"name":"(Write|Edit|NotebookEdit)"' "$TP" 2>/dev/null; then
    if ! grep -qiE "verified|verify|build.*(pass|ok|0 error)|test.*pass|cargo (build|test)|npm run build" "$TP" 2>/dev/null; then
        echo "REMINDER (Law L1/L4): verify your changes before saying done — run the build/tests, observe real output. 92% is not done."
    fi
fi
exit 0
