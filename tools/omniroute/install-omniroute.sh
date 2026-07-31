#!/usr/bin/env bash
# install-omniroute.sh — opt-in installer for OmniRoute, the OmegaOS BYOK/API
# gateway (diegosouzapw/OmniRoute, MIT). ONE OpenAI-compatible /v1 fronting 290+
# providers (40+ free) with smart routing, fallback, and token compression.
# NOT run by OmegaOS install.sh (external service — ZernFlow/open-design boundary).
#
#   OMEGA_SKIP_TS_SERVE=1   skip the tailscale serve step
#   OMNIROUTE_PORT=20128    host port
set -uo pipefail
declare -F ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
declare -F info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
declare -F warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
PORT="${OMNIROUTE_PORT:-20128}"
IMAGE="diegosouzapw/omniroute:latest"
command -v docker >/dev/null 2>&1 || { warn "docker not found — OmniRoute needs Docker. Aborting."; exit 0; }

# 1) pull + run (localhost-bound; keys stay AES-256 encrypted in the data volume)
docker pull "$IMAGE" 2>&1 | tail -1
if docker ps -a --format '{{.Names}}' | grep -qx omniroute; then
    docker start omniroute >/dev/null 2>&1 || true
    ok "OmniRoute container present — started"
else
    docker run -d --name omniroute --restart unless-stopped \
        -p "127.0.0.1:${PORT}:20128" -v omniroute-data:/app/data "$IMAGE" >/dev/null 2>&1 \
        && ok "OmniRoute started on 127.0.0.1:${PORT}" || { warn "docker run failed"; exit 0; }
fi

# 2) tailscale serve the dashboard (tailnet-only)
TS_HOST="$(tailscale status --json 2>/dev/null | python3 -c "import json,sys;print(json.load(sys.stdin)['Self']['DNSName'].rstrip('.'))" 2>/dev/null || true)"
URL="http://localhost:${PORT}"; [ -n "$TS_HOST" ] && URL="https://$TS_HOST:${PORT}"
if [[ "${OMEGA_SKIP_TS_SERVE:-0}" != "1" ]] && command -v tailscale >/dev/null 2>&1; then
    tailscale serve --bg --https=${PORT} "http://127.0.0.1:${PORT}" 2>/dev/null && ok "dashboard served tailnet-only at $URL"
fi

# 3) install the CLI
if [[ -f "$(dirname "$0")/../../scripts/omega-omniroute" ]]; then
    cp -f "$(dirname "$0")/../../scripts/omega-omniroute" "$OMEGA_DIR/bin/omega-omniroute"
    chmod +x "$OMEGA_DIR/bin/omega-omniroute"
    ln -sf "$OMEGA_DIR/bin/omega-omniroute" "${INSTALL_DIR:-$HOME/.local/bin}/omega-omniroute" 2>/dev/null || true
fi

# 4) install the omniroute skill (agent doctrine) into the active namespace
if [[ -f "$(dirname "$0")/../../skills/omniroute/SKILL.md" ]]; then
    mkdir -p "$OMEGA_DIR/skills/omniroute"
    cp -f "$(dirname "$0")/../../skills/omniroute/SKILL.md" "$OMEGA_DIR/skills/omniroute/SKILL.md"
    ln -sfn "$OMEGA_DIR/skills/omniroute" "$HOME/.claude/skills/omniroute" 2>/dev/null || true
fi

# 5) verify
i=0; until curl -sf -o /dev/null "http://127.0.0.1:${PORT}/v1/models" || [ $i -ge 30 ]; do sleep 2; i=$((i+1)); done
if curl -sf -o /dev/null "http://127.0.0.1:${PORT}/v1/models"; then
    ok "OmniRoute healthy → dashboard: $URL  |  /v1 for tools: http://127.0.0.1:${PORT}/v1"
    info "Use its /v1 for API-based LLM calls (embeddings, BYOK, cheap fan-out). NEVER the subscription CLIs."
else
    warn "OmniRoute did not report healthy yet (check: docker logs omniroute)"
fi
