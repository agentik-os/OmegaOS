#!/usr/bin/env bash
# install-open-design.sh — opt-in installer for the Open Design daemon.
# Clones upstream (pinned), starts the Docker daemon, serves it tailnet-only.
# NOT run by OmegaOS install.sh (heavy external dep — ZernFlow/higgsfield boundary).
#
#   OMEGA_SKIP_TS_SERVE=1  skip the tailscale serve step
#   OPEN_DESIGN_PORT=7456  host port (default 7456)
set -uo pipefail
declare -F ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
declare -F info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
declare -F warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }
export GIT_TERMINAL_PROMPT=0

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO_URL="https://github.com/nexu-io/open-design.git"
PIN="${OPEN_DESIGN_PIN:-7d7c56a}"
DST="$OMEGA_DIR/repos/open-design"
PORT="${OPEN_DESIGN_PORT:-7456}"

command -v docker >/dev/null 2>&1 || { warn "docker not found — Open Design needs Docker. Aborting."; exit 0; }

# 1) clone (shallow) or fast-forward
mkdir -p "$OMEGA_DIR/repos" "$OMEGA_DIR/secrets"
if [[ -d "$DST/.git" ]]; then
    ( cd "$DST" && git fetch --depth 1 origin "$PIN" 2>/dev/null && git checkout -q "$PIN" 2>/dev/null ) || info "clone present; pin fetch skipped"
else
    git clone --depth 1 "$REPO_URL" "$DST" 2>&1 | tail -1 || { warn "clone failed"; exit 0; }
fi
ok "Open Design source at $DST ($(cd "$DST" && git rev-parse --short HEAD 2>/dev/null))"

# 2) .env with a generated token (mirrored to secrets, never the repo)
DEPLOY="$DST/deploy"
TS_HOST="$(tailscale status --json 2>/dev/null | python3 -c "import json,sys;print(json.load(sys.stdin)['Self']['DNSName'].rstrip('.'))" 2>/dev/null || echo localhost)"
if [[ ! -f "$DEPLOY/.env" ]]; then
    cp "$DEPLOY/.env.example" "$DEPLOY/.env"
    TOKEN="$(openssl rand -hex 32)"
    sed -i "s|^OD_API_TOKEN=.*|OD_API_TOKEN=$TOKEN|" "$DEPLOY/.env"
    sed -i "s|^OPEN_DESIGN_ALLOWED_ORIGINS=.*|OPEN_DESIGN_ALLOWED_ORIGINS=https://$TS_HOST:$PORT|" "$DEPLOY/.env"
    grep -q '^OPEN_DESIGN_DISABLE_API_AUTH=' "$DEPLOY/.env" \
        && sed -i "s|^OPEN_DESIGN_DISABLE_API_AUTH=.*|OPEN_DESIGN_DISABLE_API_AUTH=1|" "$DEPLOY/.env" \
        || echo "OPEN_DESIGN_DISABLE_API_AUTH=1" >> "$DEPLOY/.env"
    grep '^OD_API_TOKEN=' "$DEPLOY/.env" > "$OMEGA_DIR/secrets/open-design.env" && chmod 600 "$OMEGA_DIR/secrets/open-design.env"
    ok "deploy/.env written (token mirrored to secrets)"
fi

# 3) pull + up
( cd "$DEPLOY" && docker compose pull 2>&1 | tail -1 && docker compose up -d 2>&1 | tail -1 ) || { warn "docker compose up failed"; exit 0; }

# 4) tailscale serve (tailnet-only)
if [[ "${OMEGA_SKIP_TS_SERVE:-0}" != "1" ]] && command -v tailscale >/dev/null 2>&1; then
    tailscale serve --bg --https=${PORT} "http://127.0.0.1:${PORT}" 2>/dev/null && ok "served tailnet-only at https://$TS_HOST:$PORT"
fi

# 5) install the CLI + skill live
if [[ -f "$(dirname "$0")/../../scripts/omega-design" ]]; then
    cp -f "$(dirname "$0")/../../scripts/omega-design" "$OMEGA_DIR/bin/omega-design"
    chmod +x "$OMEGA_DIR/bin/omega-design"
    ln -sf "$OMEGA_DIR/bin/omega-design" "${INSTALL_DIR:-$HOME/.local/bin}/omega-design" 2>/dev/null || true
fi

# 6) verify
i=0; until curl -sf -o /dev/null "http://127.0.0.1:${PORT}/api/health" || [ $i -ge 30 ]; do sleep 2; i=$((i+1)); done
if curl -sf -o /dev/null "http://127.0.0.1:${PORT}/api/health"; then
    ok "Open Design healthy → view: https://$TS_HOST:$PORT"
else
    warn "Open Design did not report healthy yet (check: docker logs open-design)"
fi
