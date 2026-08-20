#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP="$(mktemp -d)"
cleanup() {
  if [[ -f "$TMP/fallback/state/omega-tg-bot.pid" ]]; then
    pid="$(tr -cd '0-9' < "$TMP/fallback/state/omega-tg-bot.pid")"
    [[ -n "$pid" ]] && kill "$pid" 2>/dev/null || true
  fi
  rm -rf -- "$TMP"
}
trap cleanup EXIT
mkdir -p "$TMP/bin" "$TMP/systemd/telegram-bot" "$TMP/fallback/telegram-bot"
cp "$ROOT/telegram-bot/omega-tg-bot.ts" "$TMP/systemd/telegram-bot/omega-tg-bot.ts"
cp "$ROOT/telegram-bot/omega-tg-bot.ts" "$TMP/fallback/telegram-bot/omega-tg-bot.ts"

cat > "$TMP/bin/curl" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$MOCK_CURL_ARGS"
env | grep '^OMEGA_TG_TOKEN=' >> "$MOCK_CURL_ENV" || true
cat >> "$MOCK_CURL_STDIN"
exit 0
EOF
cat > "$TMP/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *"list-unit-files"* ]]; then [[ "$MOCK_SYSTEMD" == 1 ]]; exit; fi
printf '%s\n' "$*" >> "$MOCK_SYSTEMCTL_CALLS"
exit 0
EOF
cat > "$TMP/bin/bun" <<'EOF'
#!/usr/bin/env bash
trap 'exit 0' TERM INT
while :; do /bin/sleep 1; done
EOF
chmod +x "$TMP/bin/"*
: > "$TMP/curl-args"; : > "$TMP/curl-env"; : > "$TMP/curl-stdin"; : > "$TMP/systemctl-calls"

COMMON=(
  PATH="$TMP/bin:/usr/bin:/bin"
  MOCK_CURL_ARGS="$TMP/curl-args"
  MOCK_CURL_ENV="$TMP/curl-env"
  MOCK_CURL_STDIN="$TMP/curl-stdin"
  MOCK_SYSTEMCTL_CALLS="$TMP/systemctl-calls"
)

env "${COMMON[@]}" MOCK_SYSTEMD=1 OMEGA_DIR="$TMP/systemd" \
  OMEGA_TG_TOKEN='123:SECRET_TOKEN' bash "$ROOT/scripts/omega-tg-up.sh" 456 >/dev/null
if grep -q 'SECRET_TOKEN' "$TMP/curl-args"; then echo "token leaked into curl argv" >&2; exit 1; fi
if grep -q 'OMEGA_TG_TOKEN' "$TMP/curl-env"; then echo "token leaked into curl environment" >&2; exit 1; fi
grep -q 'SECRET_TOKEN' "$TMP/curl-stdin"
[[ "$(stat -c '%a' "$TMP/systemd/telegram.toml")" == 600 ]]
grep -q -- '--user restart omega-tg-bot.service' "$TMP/systemctl-calls"
if grep -q 'pkill' "$ROOT/scripts/omega-tg-up.sh"; then echo "broad pkill remains" >&2; exit 1; fi

# Non-systemd fallback owns one exact PID. A second run terminates only that
# marked process and replaces its pidfile, never a pattern-matched bot fleet.
env "${COMMON[@]}" MOCK_SYSTEMD=0 OMEGA_DIR="$TMP/fallback" \
  OMEGA_TG_TOKEN='123:SECRET_TOKEN' bash "$ROOT/scripts/omega-tg-up.sh" 456 >/dev/null
first_pid="$(tr -cd '0-9' < "$TMP/fallback/state/omega-tg-bot.pid")"
kill -0 "$first_pid"
env "${COMMON[@]}" MOCK_SYSTEMD=0 OMEGA_DIR="$TMP/fallback" \
  OMEGA_TG_TOKEN='123:SECRET_TOKEN' bash "$ROOT/scripts/omega-tg-up.sh" 456 >/dev/null
second_pid="$(tr -cd '0-9' < "$TMP/fallback/state/omega-tg-bot.pid")"
[[ "$first_pid" != "$second_pid" ]]
if kill -0 "$first_pid" 2>/dev/null; then echo "previous exact PID still alive" >&2; exit 1; fi
kill -0 "$second_pid"

echo "ecosystem_tg_up: ok"
