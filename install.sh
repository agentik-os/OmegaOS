#!/usr/bin/env bash
# OmegaOS Installer
# One-command setup for the agentic terminal operating system
#
# Usage: curl -sSL https://raw.githubusercontent.com/agentik-os/OmegaOS/main/install.sh | bash
#    or: ./install.sh (from cloned repo)

set -euo pipefail

OMEGA_VERSION="0.1.0"
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
REPO_URL="https://github.com/agentik-os/OmegaOS"
RMUX_REPO="https://github.com/agentik-os/rmux"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }
step()  { echo -e "\n${BOLD}==> $*${NC}"; }

# ─── Phase 1: Environment Detection ──────────────────────────────────────────

step "Phase 1: Environment Detection"

OS="$(uname -s)"
ARCH="$(uname -m)"
info "OS: $OS  Arch: $ARCH"

# Detect if running from cloned repo
if [[ -f "Cargo.toml" ]] && grep -q "omega-core" Cargo.toml 2>/dev/null; then
    OMEGA_SRC="$(pwd)"
    info "Building from local source: $OMEGA_SRC"
else
    OMEGA_SRC=""
fi

# ─── Phase 2: Dependencies ───────────────────────────────────────────────────

step "Phase 2: Checking Dependencies"

# Check for Rust
if ! command -v cargo &>/dev/null; then
    info "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    ok "Rust installed: $(rustc --version)"
else
    ok "Rust found: $(rustc --version)"
fi

# Check for git
if ! command -v git &>/dev/null; then
    err "git is required but not installed."
    exit 1
fi
ok "git found"

# Check for an AI agent CLI (optional, warn if missing)
if command -v claude &>/dev/null; then
    ok "Claude Code CLI found"
elif command -v codex &>/dev/null; then
    ok "Codex CLI found"
else
    info "No AI agent CLI found (claude/codex). You can add one later."
    info "Set agent_command in ~/.omega/config.toml"
fi

# ─── Phase 3: Build rmux ─────────────────────────────────────────────────────

step "Phase 3: Building rmux"

RMUX_BUILD_DIR="/tmp/omega-rmux-build"
if [[ -f "$INSTALL_DIR/rmux" ]]; then
    ok "rmux already installed at $INSTALL_DIR/rmux"
else
    if [[ -d "$RMUX_BUILD_DIR" ]]; then
        rm -rf "$RMUX_BUILD_DIR"
    fi
    info "Cloning rmux..."
    git clone --depth 1 "$RMUX_REPO" "$RMUX_BUILD_DIR"
    info "Building rmux (this may take a few minutes)..."
    cd "$RMUX_BUILD_DIR"
    cargo build --release 2>&1 | tail -3
    mkdir -p "$INSTALL_DIR"
    cp target/release/rmux "$INSTALL_DIR/rmux"
    cd -
    rm -rf "$RMUX_BUILD_DIR"
    ok "rmux installed to $INSTALL_DIR/rmux"
fi

# ─── Phase 4: Build OmegaOS ──────────────────────────────────────────────────

step "Phase 4: Building OmegaOS"

if [[ -z "$OMEGA_SRC" ]]; then
    OMEGA_SRC="/tmp/omega-build"
    if [[ -d "$OMEGA_SRC" ]]; then
        rm -rf "$OMEGA_SRC"
    fi
    info "Cloning OmegaOS..."
    git clone --depth 1 "$REPO_URL" "$OMEGA_SRC"
fi

cd "$OMEGA_SRC"
info "Building omega CLI..."
cargo build --release 2>&1 | tail -3
mkdir -p "$INSTALL_DIR"
cp target/release/omega "$INSTALL_DIR/omega"
ok "omega CLI installed to $INSTALL_DIR/omega"

# ─── Phase 5: Configuration ──────────────────────────────────────────────────

step "Phase 5: Configuring OmegaOS"

mkdir -p "$OMEGA_DIR"/{state,logs,locks}

if [[ ! -f "$OMEGA_DIR/config.toml" ]]; then
    cp config/default.toml "$OMEGA_DIR/config.toml"
    ok "Config created: $OMEGA_DIR/config.toml"
else
    ok "Config already exists: $OMEGA_DIR/config.toml"
fi

# Install scripts
SCRIPT_DIR="$OMEGA_DIR/scripts"
mkdir -p "$SCRIPT_DIR"
cp scripts/*.sh "$SCRIPT_DIR/"
chmod +x "$SCRIPT_DIR"/*.sh
ok "Scripts installed to $SCRIPT_DIR/"

# Install agent templates
AGENTS_DIR="$OMEGA_DIR/agents"
mkdir -p "$AGENTS_DIR"
cp agents/*.md "$AGENTS_DIR/"
ok "Agent templates installed to $AGENTS_DIR/"

# Install shell completions
SHELL_NAME="$(basename "${SHELL:-bash}")"
case "$SHELL_NAME" in
    zsh)
        COMP_DIR="${HOME}/.zsh/completions"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/omega" completions zsh > "$COMP_DIR/_omega"
        ok "Zsh completions installed"
        ;;
    bash)
        COMP_DIR="${HOME}/.local/share/bash-completion/completions"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/omega" completions bash > "$COMP_DIR/omega"
        ok "Bash completions installed"
        ;;
    fish)
        COMP_DIR="${HOME}/.config/fish/completions"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/omega" completions fish > "$COMP_DIR/omega.fish"
        ok "Fish completions installed"
        ;;
esac

# ─── Phase 6: Shell Integration ──────────────────────────────────────────────

step "Phase 6: Shell Integration"

# Detect shell
SHELL_NAME="$(basename "${SHELL:-bash}")"
case "$SHELL_NAME" in
    zsh)  RC_FILE="$HOME/.zshrc" ;;
    bash) RC_FILE="$HOME/.bashrc" ;;
    fish) RC_FILE="$HOME/.config/fish/config.fish" ;;
    *)    RC_FILE="$HOME/.profile" ;;
esac

# Add to PATH if not already there
EXPORT_LINE='export PATH="$HOME/.local/bin:$PATH"'
if ! grep -qF '.local/bin' "$RC_FILE" 2>/dev/null; then
    echo "" >> "$RC_FILE"
    echo "# OmegaOS" >> "$RC_FILE"
    echo "$EXPORT_LINE" >> "$RC_FILE"
    ok "Added $INSTALL_DIR to PATH in $RC_FILE"
else
    ok "PATH already includes $INSTALL_DIR"
fi

# Add omega alias for session manager
ALIAS_LINE='alias om="omega menu"'
if ! grep -qF 'alias om=' "$RC_FILE" 2>/dev/null; then
    echo "$ALIAS_LINE" >> "$RC_FILE"
    ok "Added 'om' alias for omega menu"
fi

# ─── Done ─────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}  OmegaOS v${OMEGA_VERSION} installed successfully!${NC}"
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo ""
echo "  Quick start:"
echo "    omega            # Launch session manager TUI"
echo "    omega new test   # Create a new session"
echo "    omega list       # List all sessions"
echo "    omega dispatch MyProject 'Fix the auth bug'"
echo ""
echo "  Configuration: $OMEGA_DIR/config.toml"
echo ""
echo "  Restart your shell or run: source $RC_FILE"
echo ""
