#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# audit-notify.sh — Telegram notification helper for Quality Arsenal audits
#
# Usage: audit-notify.sh <audit-name> <event> [details]
#
# Events:
#   start     — audit begins (scope info)
#   progress  — phase milestone (every 3 phases)
#   iteration — fix-and-reaudit cycle (score trajectory)
#   verdict   — final score + path to verdict.md
#   abort     — audit aborted (reason)
#   sos       — 5-iter cap / lock collision / unrecoverable error
#
# Requires: telegram CLI at ~/.local/bin/telegram
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

AUDIT="${1:?Usage: audit-notify.sh <audit-name> <event> [details]}"
EVENT="${2:?Usage: audit-notify.sh <audit-name> <event> [details]}"
DETAILS="${3:-}"
CANONICAL_AUDIT="$AUDIT"
[[ "$CANONICAL_AUDIT" == *audit ]] || CANONICAL_AUDIT="${CANONICAL_AUDIT}audit"

# Telegram target — read from env, never hardcoded.
# If unset, notifications are skipped entirely (no DM to anyone).
TG_USER="${OMEGA_TELEGRAM_USER_ID:-}"
TELEGRAM="${HOME}/.local/bin/telegram"

# No target configured → skip silently (don't DM a maintainer, don't fail the audit)
if [[ -z "$TG_USER" ]]; then
    exit 0
fi

# Check telegram CLI exists
if [[ ! -x "$TELEGRAM" ]]; then
    echo "[audit-notify] WARNING: telegram CLI not found at $TELEGRAM. Notification skipped." >&2
    exit 0  # Don't fail the audit because notifications are down
fi

# Format message per event type
case "$EVENT" in
    start)
        MSG="🚦 /${AUDIT} started — ${DETAILS:-no scope specified}"
        ;;
    progress)
        MSG="📊 /${AUDIT} ${DETAILS:-progress update}"
        ;;
    iteration)
        MSG="🔁 /${AUDIT} ${DETAILS:-fix-and-reaudit cycle}"
        ;;
    verdict)
        MSG="🎯 /${CANONICAL_AUDIT} done — ${DETAILS:-see audits/.${CANONICAL_AUDIT}/verdict.md}"
        ;;
    abort)
        MSG="🛑 /${AUDIT} aborted — ${DETAILS:-unknown reason}"
        ;;
    sos)
        MSG="🆘 /${AUDIT} SOS — ${DETAILS:-unrecoverable error}"
        ;;
    *)
        MSG="📋 /${AUDIT} [${EVENT}] — ${DETAILS:-no details}"
        ;;
esac

# Send notification (fire-and-forget, don't block the audit)
"$TELEGRAM" notify --user "$TG_USER" "$MSG" 2>/dev/null || {
    echo "[audit-notify] WARNING: telegram send failed for /${AUDIT} ${EVENT}" >&2
}

# Log to audit's own telemetry
AUDIT_DIR="audits/.${CANONICAL_AUDIT}"
if [[ -d "$AUDIT_DIR" ]]; then
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) ${EVENT} ${DETAILS}" >> "${AUDIT_DIR}/notifications.log"
fi
