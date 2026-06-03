#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-mc-up.sh — bring up the OmegaMC / Agentik-Telegram control plane
# ───────────────────────────────────────────────────────────────────────────
# OmegaMC (agentik-os/agentik-telegram, Go + Docker) is the OmegaOS Telegram
# interface: it routes Telegram messages to named AISB agents, each running
# Claude Code in its own container, with @agent routing + a Mission Control
# web UI. This script makes "connect Telegram → it works" a single command:
#
#   1. Generate repos/omega-mc/.env from live OmegaOS state
#      (bot token ← ~/.omega/telegram.toml, Claude OAuth ← credentials/claude.json,
#       DOCKER_GID ← the host docker group). Vault passphrase + web password are
#       generated ONCE and preserved across runs.
#   2. Ensure config/omega-mc.yaml exists (AISB 13-agent roster).
#   3. Build the three local images if missing (gateway, agent-base, agent) —
#      they are NOT published to GHCR (the agent image bundles the Claude Code
#      binary, which can't be redistributed), so we build from source.
#   4. docker compose up -d  (restart: unless-stopped → survives reboot).
#
# Idempotent: safe to re-run to refresh the Claude OAuth token or after edits.
# Usage:  omega-mc-up.sh [--rebuild] [<BOT_TOKEN> [<CHAT_ID>]]
#   --rebuild           force image rebuilds
#   <BOT_TOKEN> <CHAT_ID>  one-shot connect: write them to telegram.toml, then up
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail

# Some launch contexts (npx-spawned install, `docker exec`) have no USER var; set
# -u would then abort on every "$USER". Default it from the real uid.
: "${USER:=$(id -un)}"

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
MC_DIR="${OMEGA_MC_DIR:-$OMEGA_DIR/repos/omega-mc}"
TG_TOML="$OMEGA_DIR/telegram.toml"
CLAUDE_CREDS="$OMEGA_DIR/credentials/claude.json"
GW_IMAGE="ghcr.io/agentik-os/omega-mission-control:latest"
BASE_IMAGE="ghcr.io/agentik-os/omega-mission-control-agent-base:latest"
AGENT_IMAGE="omega-mc-agent:latest"
REBUILD=0; [[ "${1:-}" == "--rebuild" ]] && { REBUILD=1; shift; }
ARG_TOKEN="${1:-}"; ARG_CHAT="${2:-}"

die() { echo "omega-mc-up: $*" >&2; exit 1; }

# One-shot connect: persist a freshly-supplied bot token + chat id to telegram.toml
# (validated against Telegram getMe so we fail loudly on a dead token).
if [[ -n "$ARG_TOKEN" ]]; then
    if command -v curl >/dev/null 2>&1; then
        curl -sf "https://api.telegram.org/bot${ARG_TOKEN}/getMe" >/dev/null 2>&1 \
            || die "bot token rejected by Telegram (getMe 401) — get a fresh token from @BotFather"
    fi
    umask 077
    cat > "$TG_TOML" <<EOF
bot_token = "${ARG_TOKEN}"
chat_id = ${ARG_CHAT:-0}
allow_user_ids = [${ARG_CHAT:-}]
relay_session = "aisb-master"
label = ""
enabled = true
EOF
    chmod 0600 "$TG_TOML"
    echo "omega-mc-up: bot token validated + saved to $TG_TOML"
fi

[[ -d "$MC_DIR/.git" ]] || die "OmegaMC not cloned at $MC_DIR — run install.sh or: git clone https://github.com/agentik-os/agentik-telegram.git $MC_DIR"

# ── Docker provisioning (so OmegaMC works for ANYONE, on a bare box) ────────
# OmegaMC needs a working Docker engine and the invoking user able to reach the
# socket. On a fresh VPS neither exists, so provision them here (connect-time),
# keeping install.sh fast. sudo is used for the privileged bits only.
SUDO=""; [[ $EUID -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
ensure_docker() {
    if ! command -v docker >/dev/null 2>&1; then
        echo "omega-mc-up: Docker not found — installing (get.docker.com)…"
        if command -v curl >/dev/null 2>&1; then curl -fsSL https://get.docker.com | $SUDO sh >/dev/null 2>&1 || true; fi
        command -v docker >/dev/null 2>&1 || die "Docker install failed — install Docker manually, then re-run"
    fi
    # daemon up (systemd or sysv); harmless if already running
    $SUDO systemctl enable --now docker >/dev/null 2>&1 || $SUDO service docker start >/dev/null 2>&1 || true
    # current user in the docker group (applies to NEW logins; this run uses sg/sudo)
    if [[ $EUID -ne 0 ]] && ! id -nG "$USER" | grep -qw docker; then
        getent group docker >/dev/null 2>&1 || $SUDO groupadd docker >/dev/null 2>&1 || true
        $SUDO usermod -aG docker "$USER" >/dev/null 2>&1 \
            && echo "omega-mc-up: added $USER to the docker group (log out/in for it to apply to your shell)"
    fi
}
ensure_docker

# Pick a docker invocation that actually reaches the daemon in THIS process:
# direct (group already active / root) → sg docker (group set but not in this
# login) → sudo (last resort).
DOCK_MODE=direct
if ! docker version >/dev/null 2>&1; then
    if command -v sg >/dev/null 2>&1 && id -nG "$USER" | grep -qw docker && sg docker -c "docker version" >/dev/null 2>&1; then
        DOCK_MODE=sg
    elif $SUDO docker version >/dev/null 2>&1; then
        DOCK_MODE=sudo
    fi
fi
dock() {
    case "$DOCK_MODE" in
        sg)   sg docker -c "docker $(printf '%q ' "$@")" ;;
        sudo) $SUDO docker "$@" ;;
        *)    docker "$@" ;;
    esac
}
dock version >/dev/null 2>&1 || die "cannot reach the Docker daemon (tried direct, sg docker, sudo)"

# ── 1. Gather secrets from live OmegaOS state ──────────────────────────────
[[ -f "$TG_TOML" ]] || die "no $TG_TOML — connect Telegram first: omega telegram setup <TOKEN> <CHAT_ID>"
TG_TOKEN=$(grep -oP 'bot_token\s*=\s*"\K[^"]+' "$TG_TOML" || true)
[[ -n "$TG_TOKEN" ]] || die "no bot_token in $TG_TOML"
ALLOW_ID=$(grep -oP 'allow_user_ids\s*=\s*\[\s*\K[0-9]+' "$TG_TOML" 2>/dev/null || true)
CHAT_ID=$(grep -oP 'chat_id\s*=\s*\K[0-9]+' "$TG_TOML" 2>/dev/null || true)
OPERATOR="${ALLOW_ID:-$CHAT_ID}"

OAUTH=""
if [[ -f "$CLAUDE_CREDS" ]] && command -v jq >/dev/null 2>&1; then
  OAUTH=$(jq -r '.claudeAiOauth.accessToken // empty' "$CLAUDE_CREDS" 2>/dev/null || true)
fi
[[ -n "$OAUTH" || -n "${ANTHROPIC_API_KEY:-}" ]] || \
  echo "omega-mc-up: WARNING — no Claude OAuth token in $CLAUDE_CREDS and no ANTHROPIC_API_KEY; agents will fail auth until one is set." >&2

DGID=$(getent group docker | cut -d: -f3 || echo 988)

# ── 2. Generate .env (preserve generated secrets across runs) ──────────────
ENV_FILE="$MC_DIR/.env"
prev() { [[ -f "$ENV_FILE" ]] && grep -oP "^$1=\K.*" "$ENV_FILE" 2>/dev/null | head -1 || true; }
VAULT=$(prev OMEGA_MC_VAULT_PASSPHRASE); [[ -n "$VAULT" ]] || VAULT=$(openssl rand -hex 24)
WEBPW=$(prev OMEGA_MC_WEB_PASSWORD);     [[ -n "$WEBPW" ]] || WEBPW=$(openssl rand -hex 12)

umask 077
cat > "$ENV_FILE" <<EOF
# OmegaMC env — generated by omega-mc-up.sh from OmegaOS state. Secret file (0600),
# gitignored. Re-run omega-mc-up.sh to refresh the Claude OAuth token.
OMEGA_MC_TELEGRAM_TOKEN=${TG_TOKEN}
CLAUDE_CODE_OAUTH_TOKEN=${OAUTH}
ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
OMEGA_MC_WEB_PASSWORD=${WEBPW}
OMEGA_MC_VAULT_PASSPHRASE=${VAULT}
OPENAI_API_KEY=${OPENAI_API_KEY:-}
TZ=${TZ:-Europe/Paris}
DOCKER_GID=${DGID}
EOF
chmod 0600 "$ENV_FILE"

# ── 3. Ensure active config (AISB roster), operator-locked (R-TGSEC) ───────
CFG="$MC_DIR/config/omega-mc.yaml"
if [[ ! -f "$CFG" ]]; then
  if [[ -f "$MC_DIR/config/omega-aisb.yaml" ]]; then cp "$MC_DIR/config/omega-aisb.yaml" "$CFG"
  elif [[ -f "$MC_DIR/config/omega-mc.example.yaml" ]]; then cp "$MC_DIR/config/omega-mc.example.yaml" "$CFG"
  else die "no config template in $MC_DIR/config"; fi
fi
if [[ -n "$OPERATOR" ]]; then
  sed -i -E "s/^( *allow_from:) *\[.*/\1 [${OPERATOR}]/" "$CFG"
  sed -i -E "s/^( *main_chat_id:) *[0-9].*/\1 ${OPERATOR}/" "$CFG"
fi

# ── 4. Build local images if missing (or --rebuild) ────────────────────────
have() { dock image inspect "$1" >/dev/null 2>&1; }
cd "$MC_DIR"
if [[ $REBUILD -eq 1 ]] || ! have "$BASE_IMAGE"; then
  echo "omega-mc-up: building agent-base (chromium + nix + embedding model — several minutes)…"
  dock build -f Dockerfile.agent-base -t "$BASE_IMAGE" .
fi
if [[ $REBUILD -eq 1 ]] || ! have "$AGENT_IMAGE"; then
  echo "omega-mc-up: building agent image…"
  dock compose build agent
fi
if [[ $REBUILD -eq 1 ]] || ! have "$GW_IMAGE"; then
  echo "omega-mc-up: building gateway image…"
  dock build -f Dockerfile -t "$GW_IMAGE" .
fi

# ── 5. Up ───────────────────────────────────────────────────────────────────
echo "omega-mc-up: starting stack…"
dock compose up -d
echo "omega-mc-up: ✓ OmegaMC up. Mission Control → http://localhost:8080  (logs: docker compose -f $MC_DIR/docker-compose.yml logs -f)"
