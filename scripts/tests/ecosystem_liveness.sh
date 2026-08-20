#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf -- "$TMP"' EXIT
mkdir -p "$TMP/home/.omega" "$TMP/bin" "$TMP/state"

cat > "$TMP/home/.omega/telegram.toml" <<'EOF'
bot_token = "123:TEST_TOKEN"
allow_user_ids = [1]
EOF
cat > "$TMP/bin/curl" <<'EOF'
#!/usr/bin/env bash
cat >/dev/null
printf '%s\n' '{"pending_update_count":1}'
EOF
cat > "$TMP/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *" show "* ]]; then printf '%s\n' 4242; exit 0; fi
if [[ "$*" == *" restart "* ]]; then printf '%s\n' "$*" >> "$MOCK_RESTARTS"; exit 0; fi
exit 0
EOF
cat > "$TMP/bin/ps" <<'EOF'
#!/usr/bin/env bash
exit 1
EOF
cat > "$TMP/bin/sleep" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat > "$TMP/bin/date" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "+%s" ]]; then printf '%s\n' "$MOCK_NOW"; else printf '%s\n' "2026-08-11 12:00:00"; fi
EOF
cat > "$TMP/alert" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$1" >> "$MOCK_ALERTS"
EOF
chmod +x "$TMP/bin/"* "$TMP/alert"
: > "$TMP/restarts"
: > "$TMP/alerts"

run_probe() {
  local now="$1"
  HOME="$TMP/home" PATH="$TMP/bin:/usr/bin:/bin" MOCK_NOW="$now" \
    MOCK_RESTARTS="$TMP/restarts" MOCK_ALERTS="$TMP/alerts" \
    OMEGA_TG_STATE_DIR="$TMP/state" OMEGA_TG_ALERT_BIN="$TMP/alert" \
    OMEGA_TG_DRAIN_SECONDS=0 OMEGA_TG_COOLDOWN_SECONDS=10 \
    OMEGA_TG_WINDOW_SECONDS=1000 OMEGA_TG_MAX_RESTARTS=3 \
    bash "$ROOT/scripts/omega-atlas-liveness.sh" >/dev/null
}

run_probe 100
run_probe 111
run_probe 122
run_probe 133
[[ "$(wc -l < "$TMP/restarts")" -eq 3 ]]
grep -q '^OPEN ' "$TMP/state/tg-liveness-circuit-omega-tg-bot"
grep -q 'CIRCUIT OPEN' "$TMP/alerts"

# An open circuit never restarts until the explicit recovery command is used.
run_probe 144
[[ "$(wc -l < "$TMP/restarts")" -eq 3 ]]
HOME="$TMP/home" OMEGA_TG_STATE_DIR="$TMP/state" \
  bash "$ROOT/scripts/omega-atlas-liveness.sh" --reset omega-tg-bot.service >/dev/null
run_probe 200
[[ "$(wc -l < "$TMP/restarts")" -eq 4 ]]

echo "ecosystem_liveness: ok"
