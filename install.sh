#!/usr/bin/env bash
# OmegaOS Installer
# One-command setup for the agentic terminal operating system
#
# Usage: curl -sSL https://raw.githubusercontent.com/agentik-os/OmegaOS/main/install.sh | bash
#    or: ./install.sh (from cloned repo)

set -euo pipefail

# ── Prompt-proofing (NEVER hang on an invisible prompt) ──────────────────────
# The npx installer runs this script behind a full-screen animation that holds
# the TTY, so ANY interactive prompt (git asking for credentials on /dev/tty,
# apt asking a question, a tool reading stdin) becomes an INVISIBLE infinite
# hang — the progress bar just freezes. We make every step non-interactive:
#   - git fails fast instead of prompting for credentials (private/missing repo)
#   - apt/dpkg never ask questions
#   - ssh never drops to an interactive auth prompt
export GIT_TERMINAL_PROMPT=0
export GIT_SSH_COMMAND="${GIT_SSH_COMMAND:-ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new}"
export DEBIAN_FRONTEND=noninteractive
export DEBIAN_PRIORITY=critical
export NEEDRESTART_MODE=a
export PIP_NO_INPUT=1
export CI="${CI:-1}"   # many installers go non-interactive when CI is set

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

# Bootstrap the OS packages a BARE VPS lacks. Without this, the rustup curl
# below dies with `command not found` on a fresh Ubuntu/Debian image, and even
# with Rust the rmux build fails for want of a C toolchain (`cc`) + pkg-config.
# Idempotent: only installs what is actually missing.
bootstrap_os_packages() {
    local need=()
    command -v git >/dev/null 2>&1 || need+=("git")
    command -v curl >/dev/null 2>&1 || need+=("curl")
    # NOTE: the C toolchain + pkg-config are NOT bootstrapped here anymore — they
    # are only needed to COMPILE rmux/omega, which the prebuilt fast path skips.
    # `ensure_build_toolchain` (below) installs them lazily, only in the
    # source-build branches of Phase 3/4. This keeps the common (prebuilt) install
    # from pulling ~200MB of build-essential it never uses.
    # rsync is used in Phase 5 to install the PDF generator (and other asset
    # copies). A bare VPS lacks it, and under `set -euo pipefail` the missing
    # `rsync` aborts the whole install mid-way (proven: `line 255: rsync: command
    # not found` → install rc=127, leaving skills/commands/shell-integration
    # uninstalled). Bootstrap it here with the rest of the build prerequisites.
    command -v rsync >/dev/null 2>&1 || need+=("rsync")
    # jq registers the tracking/verify hooks into settings.json (Phase 6.5). Absent
    # → doctor reports "hooks present but not registered". Cheap to include here.
    command -v jq >/dev/null 2>&1 || need+=("jq")
    if [[ ${#need[@]} -eq 0 ]]; then
        ok "Runtime prerequisites present (git, curl, rsync, jq)"
        return 0
    fi
    info "Installing runtime prerequisites: ${need[*]}"
    local SUDO=""
    [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
    if command -v apt-get >/dev/null 2>&1; then
        $SUDO apt-get update -qq && $SUDO apt-get install -y curl git ca-certificates rsync jq
    elif command -v dnf >/dev/null 2>&1; then
        $SUDO dnf install -y curl git ca-certificates rsync jq
    elif command -v yum >/dev/null 2>&1; then
        $SUDO yum install -y curl git ca-certificates rsync jq
    elif command -v pacman >/dev/null 2>&1; then
        $SUDO pacman -Sy --noconfirm curl git ca-certificates rsync jq
    elif command -v apk >/dev/null 2>&1; then
        $SUDO apk add --no-cache curl git ca-certificates rsync jq
    else
        err "No supported package manager (apt/dnf/yum/pacman/apk)."
        err "Install these manually, then re-run ./install.sh: ${need[*]}"
        exit 1
    fi
    ok "Runtime prerequisites installed"
}
bootstrap_os_packages

# The C toolchain + pkg-config are ONLY needed to compile rmux/omega from source
# (the prebuilt fast path skips this entirely). Installed lazily, just-in-time,
# from the source-build branches of Phase 3/4 — so a prebuilt install never pays
# for build-essential. Idempotent: a no-op when a compiler is already present.
ensure_build_toolchain() {
    # (1) C toolchain + pkg-config
    if { command -v cc >/dev/null 2>&1 || command -v gcc >/dev/null 2>&1; } && command -v pkg-config >/dev/null 2>&1; then
        ok "Build toolchain present (cc + pkg-config)"
    else
        info "Installing build toolchain (compiling from source)..."
        local SUDO=""; [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
        if   command -v apt-get >/dev/null 2>&1; then $SUDO apt-get update -qq && $SUDO apt-get install -y build-essential pkg-config
        elif command -v dnf     >/dev/null 2>&1; then $SUDO dnf install -y gcc gcc-c++ make pkgconf-pkg-config
        elif command -v yum     >/dev/null 2>&1; then $SUDO yum install -y gcc gcc-c++ make pkgconfig
        elif command -v pacman  >/dev/null 2>&1; then $SUDO pacman -Sy --noconfirm base-devel pkgconf
        elif command -v apk     >/dev/null 2>&1; then $SUDO apk add --no-cache build-base pkgconf
        else err "No supported package manager for the build toolchain — install a C compiler + pkg-config, then re-run."; exit 1
        fi
        ok "Build toolchain installed"
    fi
    # (2) Rust (rustup) — only reached on the source-build path.
    if ! command -v cargo >/dev/null 2>&1; then
        info "Rust not found. Installing via rustup..."
        command -v curl >/dev/null 2>&1 || { err "curl is required to bootstrap Rust but is missing; install curl and re-run."; exit 1; }
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
        ok "Rust installed: $(rustc --version)"
    fi
}

# Optional fluidity dep: mosh. Plain SSH waits a full network round-trip before
# echoing each keystroke and ships output as TCP segments — on a far VPS (tens
# of ms RTT) typing feels laggy and agent streaming arrives in chunks, no matter
# how fast the box is. mosh adds predictive LOCAL echo + a UDP diff protocol, so
# typing is instant and streaming is smooth at any latency. Best-effort: a
# missing package must NEVER abort the install (it is a comfort upgrade, not a
# build requirement). Connect with: mosh <host> -- rmux attach
install_mosh_optional() {
    command -v mosh-server >/dev/null 2>&1 && { ok "mosh present (low-latency SSH available)"; return 0; }
    local SUDO=""; [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
    if   command -v apt-get >/dev/null 2>&1; then $SUDO apt-get install -y mosh 2>/dev/null
    elif command -v dnf     >/dev/null 2>&1; then $SUDO dnf install -y mosh 2>/dev/null
    elif command -v yum     >/dev/null 2>&1; then $SUDO yum install -y mosh 2>/dev/null
    elif command -v pacman  >/dev/null 2>&1; then $SUDO pacman -Sy --noconfirm mosh 2>/dev/null
    elif command -v apk     >/dev/null 2>&1; then $SUDO apk add --no-cache mosh 2>/dev/null
    fi
    command -v mosh-server >/dev/null 2>&1 \
        && ok "mosh installed — connect with 'mosh <host> -- rmux attach' for lag-free typing/streaming" \
        || info "mosh not installed (optional) — plain SSH still works; install mosh for smoother remote typing"
    return 0
}
install_mosh_optional

# A UTF-8 native locale must EXIST or mosh-server degrades ("needs a UTF-8 native
# locale to run") and Claude's box-drawing/accents render as mojibake. A bare VPS
# often ships without en_US.UTF-8 generated. Guarantee it system-wide so EVERY
# session — root + future users — inherits UTF-8 (this is what the Termius mosh
# command's `-l LANG=en_US.UTF-8` relies on). Best-effort; never aborts.
ensure_utf8_locale() {
    if locale -a 2>/dev/null | grep -qiE 'en_US\.utf-?8'; then ok "UTF-8 locale present (en_US.UTF-8)"; else
        local SUDO=""; [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
        # Every command here is best-effort: on a minimal image the `locales`
        # package (and its en_US source) may be absent, so locale-gen/localedef
        # can fail. Under `set -euo pipefail` an unguarded failure would ABORT
        # the whole install (proven in a clean ubuntu:24.04 container) — so each
        # is `|| true`. A missing UTF-8 locale degrades to C.UTF-8, never fatal.
        if command -v locale-gen >/dev/null 2>&1; then
            $SUDO sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen 2>/dev/null || true
            grep -q '^en_US.UTF-8 UTF-8' /etc/locale.gen 2>/dev/null || echo 'en_US.UTF-8 UTF-8' | $SUDO tee -a /etc/locale.gen >/dev/null 2>&1 || true
            $SUDO locale-gen 2>/dev/null || true
        elif command -v localedef >/dev/null 2>&1; then
            $SUDO localedef -i en_US -f UTF-8 en_US.UTF-8 2>/dev/null || true
        fi
        locale -a 2>/dev/null | grep -qiE 'en_US\.utf-?8' \
            && ok "UTF-8 locale generated (en_US.UTF-8)" \
            || info "Could not generate en_US.UTF-8 — falls back to C.UTF-8 (mosh: use -l LANG=C.UTF-8)"
    fi
    # Make it the system default so root + future users get UTF-8 without per-shell setup.
    if [[ ! -s /etc/default/locale ]] || ! grep -q 'LANG=.*UTF-8' /etc/default/locale 2>/dev/null; then
        local SUDO=""; [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
        { echo 'LANG=en_US.UTF-8' | $SUDO tee /etc/default/locale >/dev/null 2>&1 \
            && ok "System default locale set (LANG=en_US.UTF-8 for all users)"; } || true
    fi
    return 0
}
ensure_utf8_locale

# Bun runtime: the SST companion layer (Phase 6.9) installs global skills via the
# `skills` CLI (bunx/npx) — planning-with-files, design packs, claude-mem,
# superpowers. On a BARE VPS without bun/node those are skipped, so SST never
# lands. Bootstrap bun (single static binary, fast) so SST works on any fresh
# machine. Best-effort: never aborts the install if it fails.
install_bun_optional() {
    { command -v bun >/dev/null 2>&1 || command -v bunx >/dev/null 2>&1 \
        || command -v npx >/dev/null 2>&1; } && { ok "JS runtime present (bun/npx — SST companion can install)"; return 0; }
    info "Installing bun (for the SST companion skills layer)…"
    curl -fsSL https://bun.sh/install | bash >/dev/null 2>&1 || true
    # bun installs to ~/.bun/bin — make it visible to this run + future shells.
    [[ -d "$HOME/.bun/bin" ]] && export PATH="$HOME/.bun/bin:$PATH"
    command -v bun >/dev/null 2>&1 \
        && ok "bun installed (SST companion skills will install)" \
        || info "bun not installed (optional) — SST companion skills will be skipped; install bun/node later to enable them"
    return 0
}
install_bun_optional

# Rust is needed ONLY to compile from source (the prebuilt fast path skips it).
# `ensure_build_toolchain` installs rustup lazily in the Phase 3/4 source-build
# branches if cargo is absent — so a prebuilt install never downloads a toolchain.
if command -v cargo &>/dev/null; then
    ok "Rust found: $(rustc --version 2>/dev/null || echo present)"
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

# ─── Phase 2.5: Prebuilt binaries (fast path) ────────────────────────────────
# Try to download prebuilt omega + rmux from GitHub Releases for THIS platform
# and skip the ~8-minute source compile. Any miss (no release, no asset for this
# arch, checksum/extract failure, binary won't run here) falls through to the
# source build in Phases 3-4 — so a fresh clone always reproduces the system
# (Law 0), just faster when a release exists. Force source with OMEGA_FROM_SOURCE=1.
PREBUILT_OK=""
maybe_install_prebuilt() {
    [[ -n "${OMEGA_FROM_SOURCE:-}" ]] && { info "OMEGA_FROM_SOURCE set — building from source"; return 0; }
    command -v curl >/dev/null 2>&1 || { info "curl absent — building from source"; return 0; }
    command -v tar  >/dev/null 2>&1 || return 0

    local os arch triple
    os="$(uname -s)"; arch="$(uname -m)"
    case "$os/$arch" in
        Linux/x86_64)              triple="x86_64-unknown-linux-gnu" ;;
        Linux/aarch64|Linux/arm64) triple="aarch64-unknown-linux-gnu" ;;
        Darwin/arm64)              triple="aarch64-apple-darwin" ;;
        Darwin/x86_64)             triple="x86_64-apple-darwin" ;;
        *) info "No prebuilt for $os/$arch — building from source"; return 0 ;;
    esac

    local tag
    tag="$(curl -fsSL "https://api.github.com/repos/agentik-os/OmegaOS/releases/latest" 2>/dev/null \
            | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')" || true
    [[ -n "$tag" ]] || { info "No published release yet — building from source"; return 0; }

    local base tarball tmp
    base="https://github.com/agentik-os/OmegaOS/releases/download/$tag"
    tarball="omega-$triple.tar.gz"
    tmp="$(mktemp -d)" || return 0

    info "Fetching prebuilt $tarball ($tag)..."
    if ! curl -fsSL "$base/$tarball" -o "$tmp/$tarball"; then
        info "Prebuilt $tarball not in release $tag — building from source"; rm -rf "$tmp"; return 0
    fi
    # Checksum: if the .sha256 sidecar exists it MUST match (tamper/partial guard).
    if curl -fsSL "$base/$tarball.sha256" -o "$tmp/$tarball.sha256" 2>/dev/null; then
        local want got
        want="$(awk '{print $1}' "$tmp/$tarball.sha256" 2>/dev/null)" || true
        got="$( { sha256sum "$tmp/$tarball" 2>/dev/null || shasum -a 256 "$tmp/$tarball" 2>/dev/null; } | awk '{print $1}')" || true
        if [[ -n "$want" && "$want" != "$got" ]]; then
            err "Prebuilt checksum mismatch — refusing prebuilt, building from source"; rm -rf "$tmp"; return 0
        fi
    fi
    if ! tar xzf "$tmp/$tarball" -C "$tmp" 2>/dev/null; then
        info "Prebuilt extract failed — building from source"; rm -rf "$tmp"; return 0
    fi
    [[ -f "$tmp/omega" && -f "$tmp/rmux" ]] || { info "Prebuilt missing binaries — building from source"; rm -rf "$tmp"; return 0; }

    mkdir -p "$INSTALL_DIR"
    install -m 0755 "$tmp/omega" "$INSTALL_DIR/omega" || { rm -rf "$tmp"; return 0; }
    install -m 0755 "$tmp/rmux"  "$INSTALL_DIR/rmux"  || { rm -rf "$tmp"; return 0; }
    ln -sf "$INSTALL_DIR/omega" "$INSTALL_DIR/omg"
    rm -rf "$tmp"

    # Sanity: the downloaded binaries actually run on THIS host (right libc/arch).
    # rmux is tmux-style — its version flag is `-V` (NOT --version, which exits 1).
    if "$INSTALL_DIR/omega" --version >/dev/null 2>&1 && "$INSTALL_DIR/rmux" -V >/dev/null 2>&1; then
        PREBUILT_OK=1
        ok "Prebuilt omega + rmux installed ($tag, $triple) — skipped the source build"
    else
        info "Prebuilt binaries did not run here — building from source"
        rm -f "$INSTALL_DIR/omega" "$INSTALL_DIR/rmux"
    fi
    return 0
}
maybe_install_prebuilt

# ─── Phase 3: Build rmux ─────────────────────────────────────────────────────

step "Phase 3: Building rmux"

RMUX_BUILD_DIR="/tmp/omega-rmux-build"
if [[ -f "$INSTALL_DIR/rmux" ]]; then
    ok "rmux already installed at $INSTALL_DIR/rmux"
else
    if [[ -d "$RMUX_BUILD_DIR" ]]; then
        rm -rf "$RMUX_BUILD_DIR"
    fi
    ensure_build_toolchain
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
if [[ -n "${PREBUILT_OK:-}" ]]; then
    ok "omega CLI already installed from prebuilt — skipping source build"
else
    ensure_build_toolchain
    info "Building omega CLI..."
    # --locked: build against the committed Cargo.lock so a fresh clone resolves the
    # exact same transitive deps (reproducible builds). Falls back to an unlocked
    # build only if the lockfile is somehow absent/out of sync.
    cargo build --release --locked 2>&1 | tail -3 || cargo build --release 2>&1 | tail -3
    mkdir -p "$INSTALL_DIR"
    cp target/release/omega "$INSTALL_DIR/omega"
    ln -sf "$INSTALL_DIR/omega" "$INSTALL_DIR/omg"   # short alias: omg == omega
    ok "omega CLI installed to $INSTALL_DIR/omega"
fi

# ─── Phase 5: Configuration ──────────────────────────────────────────────────

step "Phase 5: Configuring OmegaOS"

# Single OmegaOS home. Defined sub-dirs so future installs land in the RIGHT
# place (see docs/ARCHITECTURE.md): repos/ = cloned github repos (e.g. omega-mc),
# tools/ = third-party tools/binaries an agent installs, prompts/ = runtime
# prompt scratch, lib/ + bin/ = audit runtime. No dual ~/.aisb home.
mkdir -p "$OMEGA_DIR"/{state,logs,locks,repos,tools,prompts,lib,bin}
mkdir -p "$OMEGA_DIR/credentials/accounts"

# ─── Provisioning credential store (for /omega-new-project) ─────────────────
# ~/.omega/provisioning/ holds the tokens that auto-create + wire external
# services (Vercel/Convex/GitHub/Clerk/Stripe) for a fresh project. We seed
# the templates ONCE and NEVER clobber a real file the user has filled in.
mkdir -p "$OMEGA_DIR/provisioning"
chmod 700 "$OMEGA_DIR/provisioning"
if [[ -f "$OMEGA_SRC/config/provisioning.sample" && ! -f "$OMEGA_DIR/provisioning/services.env" ]]; then
    cp "$OMEGA_SRC/config/provisioning.sample" "$OMEGA_DIR/provisioning/services.env"
    chmod 600 "$OMEGA_DIR/provisioning/services.env"
fi
if [[ -f "$OMEGA_SRC/config/clerk-pool.sample" && ! -f "$OMEGA_DIR/provisioning/clerk-pool.env" ]]; then
    cp "$OMEGA_SRC/config/clerk-pool.sample" "$OMEGA_DIR/provisioning/clerk-pool.env"
    chmod 600 "$OMEGA_DIR/provisioning/clerk-pool.env"
fi

# ─── Phase 5a: Credential Migration ─────────────────────────────────────────
# Move existing per-provider credentials into ~/.omega/credentials/<provider>.json
# and replace the legacy path with a symlink. Idempotent: if the legacy path is
# already a symlink, do nothing. If the canonical file already exists, the
# legacy file is backed up with .pre-omega suffix.

migrate_creds() {
    local provider="$1"
    local legacy="$2"
    local canonical="$OMEGA_DIR/credentials/${provider}.json"

    # Already a symlink? Nothing to do.
    if [ -L "$legacy" ]; then
        ok "$provider creds: already symlinked"
        return
    fi

    # Legacy file exists as a real file?
    if [ -f "$legacy" ]; then
        mkdir -p "$(dirname "$legacy")"
        if [ -f "$canonical" ]; then
            # Both exist — keep canonical, backup the legacy duplicate.
            mv "$legacy" "${legacy}.pre-omega"
            info "$provider creds: backed up legacy duplicate to ${legacy}.pre-omega"
        else
            mv "$legacy" "$canonical"
            ok "$provider creds: migrated $legacy -> $canonical"
        fi
    fi

    # Ensure parent dir of legacy exists so the symlink can be created.
    mkdir -p "$(dirname "$legacy")"

    # Create the symlink (target may not exist yet — that is fine; the LLM
    # will write through it on first login).
    if [ ! -e "$legacy" ] && [ ! -L "$legacy" ]; then
        ln -s "$canonical" "$legacy"
        ok "$provider creds: linked $legacy -> $canonical"
    fi
}

migrate_creds "claude" "$HOME/.claude/.credentials.json"
migrate_creds "codex"  "$HOME/.codex/auth.json"
migrate_creds "gemini" "$HOME/.gemini/oauth_creds.json"

if [[ ! -f "$OMEGA_DIR/config.toml" ]]; then
    cp config/default.toml "$OMEGA_DIR/config.toml"
    ok "Config created: $OMEGA_DIR/config.toml"
else
    ok "Config already exists: $OMEGA_DIR/config.toml"
fi

# Clock timezone hint. The on-screen clock follows the system zone by default;
# a headless VPS is usually UTC, so the operator (elsewhere) sees a wrong wall
# time. We can't auto-detect where the human is — just flag it and show the knob.
SYS_TZ="$( (timedatectl show -p Timezone --value 2>/dev/null) || cat /etc/timezone 2>/dev/null || echo "${TZ:-}" )"
if [[ -z "$SYS_TZ" || "$SYS_TZ" == "Etc/UTC" || "$SYS_TZ" == "UTC" ]]; then
    info "Clock is UTC. For your wall time, set 'timezone = \"Europe/Paris\"' (your IANA zone) in $OMEGA_DIR/config.toml"
else
    ok "Clock follows system timezone: $SYS_TZ (override with 'timezone =' in config.toml)"
fi

# Install agent templates (Master AISB system prompt + the 13 agent prompts)
AGENTS_DIR="$OMEGA_DIR/agents"
mkdir -p "$AGENTS_DIR/aisb"
cp agents/*.md "$AGENTS_DIR/" 2>/dev/null || true
cp -r agents/aisb/*.md "$AGENTS_DIR/aisb/" 2>/dev/null || true
ok "Agent templates installed to $AGENTS_DIR/"

# Install PDF generator (templates + engine — deps installed on first use)
PDFGEN_SRC="$OMEGA_SRC/tools/pdfgen"
PDFGEN_DST="$OMEGA_DIR/pdfgen"
if [[ -d "$PDFGEN_SRC" ]]; then
    mkdir -p "$PDFGEN_DST"
    if command -v rsync >/dev/null 2>&1; then
        rsync -a --exclude='node_modules' --exclude='.next' --exclude='output' "$PDFGEN_SRC/" "$PDFGEN_DST/"
    else
        # Defense in depth: rsync is bootstrapped in Phase 2, but never let its
        # absence abort the whole install (set -e). cp + prune the heavy dirs.
        cp -a "$PDFGEN_SRC/." "$PDFGEN_DST/"
        rm -rf "$PDFGEN_DST/node_modules" "$PDFGEN_DST/.next" "$PDFGEN_DST/output"
    fi
    ok "PDF generator installed to $PDFGEN_DST/ (deps auto-install on first 'omega pdf')"
else
    info "PDF generator source not found — skipping (can be added later)"
fi

# Bridge config dir: a minimal Claude config (no hooks) so the Telegram
# bridge's `claude --print` calls run fast (~4s vs ~11s with full settings).
# Credentials are symlinked so OAuth still works.
BRIDGE_CFG="$OMEGA_DIR/claude-bridge-config"
mkdir -p "$BRIDGE_CFG"
echo '{}' > "$BRIDGE_CFG/settings.json"
# Ensure the symlink target exists so the bridge never reads through a dangling
# link before `claude` login writes real creds (NEVER clobber an existing file).
[[ -e "$OMEGA_DIR/credentials/claude.json" ]] || : > "$OMEGA_DIR/credentials/claude.json"
ln -sf "$OMEGA_DIR/credentials/claude.json" "$BRIDGE_CFG/.credentials.json"
ok "Bridge config dir created (fast hookless claude --print)"

# Install OAuth helper (fallback for non-interactive token refresh)
OAUTH_SRC="$OMEGA_SRC/docs/reference/oauth/claude-oauth.sh"
OAUTH_DST_DIR="$OMEGA_DIR/bin"
OAUTH_DST="$OAUTH_DST_DIR/claude-oauth.sh"
mkdir -p "$OAUTH_DST_DIR"
if [[ -f "$OAUTH_SRC" ]]; then
    cp "$OAUTH_SRC" "$OAUTH_DST"
    chmod +x "$OAUTH_DST"
    ok "OAuth helper installed: $OAUTH_DST"
else
    info "OAuth helper script not found — skipping (login via Telegram /login still works)"
fi

# Install audit skills (Quality Arsenal)
AUDITS_SRC="$OMEGA_SRC/skills/audits"
AUDITS_DST="$OMEGA_DIR/skills/audits"
if [[ -d "$AUDITS_SRC" ]]; then
    mkdir -p "$AUDITS_DST"
    cp -r "$AUDITS_SRC"/* "$AUDITS_DST/"
    # Vendored shared shell tools must stay executable after copy (cp -r does not
    # guarantee the bit on every fs). The audits invoke _shared/*.sh
    # (hinge-analyzer.sh, grep-loop.sh, …).
    [[ -d "$AUDITS_DST/_shared" ]] && chmod +x "$AUDITS_DST/_shared/"*.sh 2>/dev/null || true
    ok "Quality Arsenal installed: $(ls -d "$AUDITS_DST"/*/ 2>/dev/null | wc -l) audit skills → $AUDITS_DST/"

    # Make each audit invocable as a Claude Code slash command (/codeaudit, etc.).
    # The full SKILL.md is large, so the stub points the agent at the installed
    # protocol file rather than inlining it. Idempotent + non-fatal.
    AUDIT_CMD_DST="$HOME/.claude/commands"
    mkdir -p "$AUDIT_CMD_DST"
    AUDIT_STUBS=0
    for skill_md in "$AUDITS_DST"/*/SKILL.md; do
        [[ -f "$skill_md" ]] || continue
        name="$(basename "$(dirname "$skill_md")")"
        # Ship each audit under BOTH its canonical /<name> (kept — the CLAUDE.md
        # keyword detection maps to these) AND a namespaced /omg-<name> alias.
        # Alias, never rename — both always work (non-breaking).
        for cmd in "$name" "omg-$name"; do
            cat > "$AUDIT_CMD_DST/$cmd.md" <<EOF
# /$cmd

Run the full $name protocol. Read and follow the complete forensic instructions in:

\`$AUDITS_DST/$name/SKILL.md\`

Execute every phase exactly as written — no streamlined or custom variant.
EOF
        done
        AUDIT_STUBS=$((AUDIT_STUBS + 1))
    done
    ok "Audit slash commands installed ($AUDIT_STUBS audits → /<name> + /omg-<name> in $AUDIT_CMD_DST/)"

    # Quality Arsenal RUNTIME. The audit SKILLs invoke the hybrid orchestrator
    # by ABSOLUTE path under the single OmegaOS home: ~/.omega/lib/audit-runner.sh
    # (the "mandatory FIRST step"), its ~/.omega/lib/audit-gather/<audit>.sh|.py
    # gatherers, ~/.omega/lib/safe-npm-build.sh (build mutex), and
    # ~/.omega/bin/audit-notify.sh. Vendored under _shared/ and placed where the
    # skills expect them, or every audit breaks on a fresh clone (Law 0). Single
    # home — no dual ~/.aisb (consolidated). Runtime scratch goes to ~/.omega/state.
    if [[ -d "$AUDITS_DST/_shared" ]]; then
        mkdir -p "$OMEGA_DIR/lib/audit-gather" "$OMEGA_DIR/bin" "$OMEGA_DIR/state"
        if [[ -f "$AUDITS_DST/_shared/audit-runner.sh" ]]; then
            cp "$AUDITS_DST/_shared/audit-runner.sh" "$OMEGA_DIR/lib/audit-runner.sh"
            chmod +x "$OMEGA_DIR/lib/audit-runner.sh"
        fi
        if [[ -f "$AUDITS_DST/_shared/safe-npm-build.sh" ]]; then
            cp "$AUDITS_DST/_shared/safe-npm-build.sh" "$OMEGA_DIR/lib/safe-npm-build.sh"
            chmod +x "$OMEGA_DIR/lib/safe-npm-build.sh"
        fi
        if [[ -d "$AUDITS_DST/_shared/audit-gather" ]]; then
            cp -r "$AUDITS_DST/_shared/audit-gather/." "$OMEGA_DIR/lib/audit-gather/"
            chmod +x "$OMEGA_DIR/lib/audit-gather/"*.sh 2>/dev/null || true
        fi
        if [[ -f "$AUDITS_DST/_shared/audit-notify.sh" ]]; then
            cp "$AUDITS_DST/_shared/audit-notify.sh" "$OMEGA_DIR/bin/audit-notify.sh"
            chmod +x "$OMEGA_DIR/bin/audit-notify.sh"
        fi
        ok "Audit runtime installed (~/.omega/lib/audit-runner.sh + audit-gather/ + safe-npm-build.sh + ~/.omega/bin/audit-notify.sh)"
    fi
else
    info "Audit skills not found — skipping"
fi

# Install the design skills (generative UI/UX, aesthetics, image-to-code).
# Curated set: taste-skill, minimalist-ui, industrial-brutalist-ui,
# high-end-visual-design, image-to-code, design-system, stitch-design-taste,
# ui-ux-pro-max. Mirrors the audits loop: copy → ~/.omega/skills/design/ +
# generate /<name> AND /omg-<name> slash stubs.
DESIGN_SRC="$OMEGA_SRC/skills/design"
DESIGN_DST="$OMEGA_DIR/skills/design"
if [[ -d "$DESIGN_SRC" ]]; then
    mkdir -p "$DESIGN_DST"
    cp -r "$DESIGN_SRC"/* "$DESIGN_DST/"
    find "$DESIGN_DST" -name "*.sh" -exec chmod +x {} + 2>/dev/null || true
    DESIGN_CMD_DST="$HOME/.claude/commands"
    mkdir -p "$DESIGN_CMD_DST"
    DESIGN_STUBS=0
    for skill_md in "$DESIGN_DST"/*/SKILL.md; do
        [[ -f "$skill_md" ]] || continue
        name="$(basename "$(dirname "$skill_md")")"
        for cmd in "$name" "omg-$name"; do
            cat > "$DESIGN_CMD_DST/$cmd.md" <<EOF
# /$cmd

Run the $name design skill. Read and follow the complete instructions in:

\`$DESIGN_DST/$name/SKILL.md\`

Use every reference, template, and script it provides.
EOF
        done
        DESIGN_STUBS=$((DESIGN_STUBS + 1))
    done
    ok "Design skills installed ($DESIGN_STUBS → /<name> + /omg-<name> in $DESIGN_DST/)"
else
    info "Design skills not found — skipping"
fi

# Install the orchestration planner skill (engine-native).
# OmegaOS slash commands are namespaced `/omg-*` to avoid colliding with the
# user's other commands (e.g. a pre-existing prose `/planner`).
PLANNER_SRC="$OMEGA_SRC/skills/planner"
PLANNER_DST="$OMEGA_DIR/skills/planner"
OMG_CMD_DST="$HOME/.claude/commands"
mkdir -p "$OMG_CMD_DST"
if [[ -d "$PLANNER_SRC" ]]; then
    mkdir -p "$PLANNER_DST"
    cp -r "$PLANNER_SRC"/* "$PLANNER_DST/"
    cat > "$OMG_CMD_DST/omg-planner.md" <<EOF
# /omg-planner

Generate a typed .planner/tracker.json and drive it with the OmegaOS engine.
Read and follow the full protocol in:

\`$PLANNER_DST/SKILL.md\`

The engine (\`omega plan-run\` / \`omega plan-status\`) enforces can't-skip + Guardian
verify. If \`omega\` is missing, fall back to: \`bun $PLANNER_DST/fallback/plan.ts\`.
EOF
    ok "Planner skill installed → $PLANNER_DST/ (+ /omg-planner stub)"
else
    info "Planner skill not found — skipping"
fi

# Install the end-to-end new-project skill (engine-driven). Ships as repo asset
# and installs BOTH /omg-new-project (canonical) and /omega-new-project (the name
# the TUI [N] New Project menu already dispatches) — same skill, no collision.
NEWPROJ_SRC="$OMEGA_SRC/skills/new-project"
NEWPROJ_DST="$OMEGA_DIR/skills/new-project"
if [[ -d "$NEWPROJ_SRC" ]]; then
    mkdir -p "$NEWPROJ_DST"
    cp -r "$NEWPROJ_SRC"/* "$NEWPROJ_DST/"
    for cmd in omg-new-project omega-new-project; do
        cat > "$OMG_CMD_DST/$cmd.md" <<EOF
# /$cmd

End-to-end OmegaOS new project: guiding (client/works) → provision → scaffold →
vision/PRD → /omg-planner → \`omega plan-run\` (engine executes with can't-skip +
Guardian verify). Read and follow the full protocol in:

\`$NEWPROJ_DST/SKILL.md\`
EOF
    done
    ok "New-project skill installed → $NEWPROJ_DST/ (+ /omg-new-project, /omega-new-project stubs)"
else
    info "New-project skill not found — skipping"
fi

# Install the OmegaOS pipeline skills (vision, prd) the new-project flow delegates
# to — shipped as /omg-* so a FRESH install is self-contained (the pipeline no
# longer depends on the user's personal /vision /prd existing). Does NOT touch any
# pre-existing /vision /prd the user may already have.
for psk in vision prd; do
    PSK_SRC="$OMEGA_SRC/skills/$psk"
    PSK_DST="$OMEGA_DIR/skills/$psk"
    if [[ -d "$PSK_SRC" ]]; then
        mkdir -p "$PSK_DST"
        cp -r "$PSK_SRC"/* "$PSK_DST/"
        cat > "$OMG_CMD_DST/omg-$psk.md" <<EOF
# /omg-$psk

OmegaOS-shipped \`$psk\` step of the new-project pipeline. Read and follow the
full protocol in:

\`$PSK_DST/SKILL.md\`
EOF
        ok "Pipeline skill installed → $PSK_DST/ (+ /omg-$psk stub)"
    fi
done

# Install the Linear feedback-resolution skill (self-contained, engine-native).
# Ships the launcher (SKILL.md) + the full ported protocol (RULES.md) to
# ~/.omega/skills/linear/, and installs BOTH /omg-linear (canonical, namespaced)
# AND /linear (the keyword the CLAUDE.md trigger table maps to). Alias, never a
# rename — both always work. No maintainer-private deps: the audit gate runs
# through the shipped /omg-audit Quality Arsenal, NOT any private selector/gate
# script. The stub points the agent at the installed RULES.md (single source of
# truth) so a fresh `git clone && ./install.sh` reproduces the whole pipeline.
LINEAR_SRC="$OMEGA_SRC/skills/linear"
LINEAR_DST="$OMEGA_DIR/skills/linear"
if [[ -d "$LINEAR_SRC" ]]; then
    mkdir -p "$LINEAR_DST"
    cp -r "$LINEAR_SRC"/* "$LINEAR_DST/"
    for cmd in omg-linear linear; do
        cat > "$OMG_CMD_DST/$cmd.md" <<EOF
# /$cmd

OmegaOS Linear feedback-resolution pipeline (v2, Workflow-driven). TRIGGER-GUARDED:
only act on Linear when the user explicitly signals it (the word "linear", "fix linear",
"regler les feedbacks", a ticket id, or a linear.app URL) — bare "feedback"/"ticket"
never triggers it, and never mention Linear unless the user did.

Read and follow the full protocol (the single source of truth) in:

\`$LINEAR_DST/RULES.md\`

The launcher is \`$LINEAR_DST/SKILL.md\`. Resolve tickets end-to-end via the OmegaOS
Workflow primitive (\`/dynamic\`): triage → surgical fix → BEFORE/AFTER evidence →
the /omg-audit gate (\`omega audit select\` → run each /omg-<name>audit, 100/100 each)
→ strict Fix-Verification comment → move each ticket to the neutral "In Review" /
"Omega Review" state for the operator. A human marks Done; the agent NEVER self-marks Done.
EOF
    done
    ok "Linear skill installed → $LINEAR_DST/ (+ /omg-linear, /linear stubs)"
else
    info "Linear skill not found — skipping"
fi

# Install the one-time Linear app-setup wizard (installs the in-app feedback
# widget + Linear labels + API routes into the user's OWN project). Ships to
# ~/.omega/skills/linear-setup/ with /omg-linear-setup (+ /linear-setup alias).
LINEAR_SETUP_SRC="$OMEGA_SRC/skills/linear-setup"
LINEAR_SETUP_DST="$OMEGA_DIR/skills/linear-setup"
if [[ -d "$LINEAR_SETUP_SRC" ]]; then
    mkdir -p "$LINEAR_SETUP_DST"
    cp -r "$LINEAR_SETUP_SRC"/* "$LINEAR_SETUP_DST/"
    for cmd in omg-linear-setup linear-setup; do
        cat > "$OMG_CMD_DST/$cmd.md" <<EOF
# /$cmd

One-time Linear feedback-system setup for the CURRENT project: installs the
in-app feedback widget (screenshot + page URL + element selector + console
capture), the Linear label groups the pipeline keys off, and the API route(s)
that turn a widget report into a Linear issue. Auto-detects the project's stack,
auth provider, UI library, and src/ layout. Run this ONCE per project; after it,
use /omg-linear to resolve the tickets. Read and follow the full wizard in:

\`$LINEAR_SETUP_DST/SKILL.md\`
EOF
    done
    ok "Linear-setup wizard installed → $LINEAR_SETUP_DST/ (+ /omg-linear-setup, /linear-setup stubs)"
else
    info "Linear-setup skill not found — skipping"
fi

# Install OMEGA.md master system prompt
if [[ -f "$OMEGA_SRC/OMEGA.md" ]]; then
    cp "$OMEGA_SRC/OMEGA.md" "$OMEGA_DIR/OMEGA.md"
    ok "OMEGA.md installed (universal agent instructions)"
fi

# Install reference docs (architecture, integration plans, install guide).
# Ship the whole docs/ tree to ~/.omega/docs/ so users can `omega docs`
# (and so Claude sessions launched in $HOME can read them).
if [[ -d "$OMEGA_SRC/docs" ]]; then
    mkdir -p "$OMEGA_DIR/docs"
    cp -r "$OMEGA_SRC/docs/"* "$OMEGA_DIR/docs/" 2>/dev/null || true
    ok "Reference docs installed → $OMEGA_DIR/docs/ ($(ls "$OMEGA_DIR/docs/"*.md 2>/dev/null | wc -l) markdown files)"
fi

# Ship OmegaOS-specific Claude Code slash commands (.claude/commands/omega-*.md)
# These become available in EVERY Claude session globally as /omega-status,
# /omega-dispatch, etc. (Claude Code v2.1+ auto-discovers user-scope commands).
CLAUDE_CMD_DST="$HOME/.claude/commands"
if [[ -d "$OMEGA_SRC/.claude/commands" ]]; then
    mkdir -p "$CLAUDE_CMD_DST"
    # Each OmegaOS command ships under TWO names: its canonical /omg-<name>
    # (short, namespaced — avoids colliding with the user's other commands)
    # AND its legacy /omega-<name>. This is an ALIAS, never a rename — renaming
    # would break references (patrol.rs dispatches /omega-curate; the TUI [N]
    # New Project menu dispatches /omega-new-project). Both always work.
    # new-project is skipped here: its engine-integrated stubs (/omg-new-project
    # + /omega-new-project) are generated above from skills/new-project, so both
    # names point at the can't-skip engine version, not the prose copy.
    for src in "$OMEGA_SRC/.claude/commands/"omega-*.md; do
        [[ -f "$src" ]] || continue
        bn="$(basename "$src")"
        [[ "$bn" == "omega-new-project.md" ]] && continue
        cp -f "$src" "$CLAUDE_CMD_DST/$bn"               # /omega-<name> (legacy, unchanged)
        cp -f "$src" "$CLAUDE_CMD_DST/omg-${bn#omega-}"  # /omg-<name>   (canonical alias)
    done
    # /dynamic — native Dynamic Workflows trigger — keep + add /omg-dynamic alias.
    if [[ -f "$OMEGA_SRC/.claude/commands/dynamic.md" ]]; then
        cp -f "$OMEGA_SRC/.claude/commands/dynamic.md" "$CLAUDE_CMD_DST/dynamic.md"
        cp -f "$OMEGA_SRC/.claude/commands/dynamic.md" "$CLAUDE_CMD_DST/omg-dynamic.md"
    fi
    # /projects — also expose the bare name the user reaches for (in addition to
    # /omega-projects + /omg-projects shipped by the loop above). `omega projects`
    # exists as a CLI subcommand; this makes the Claude slash command match it.
    if [[ -f "$OMEGA_SRC/.claude/commands/omega-projects.md" ]]; then
        cp -f "$OMEGA_SRC/.claude/commands/omega-projects.md" "$CLAUDE_CMD_DST/projects.md"
    fi
    SHIPPED=$(ls "$CLAUDE_CMD_DST/"omg-*.md 2>/dev/null | wc -l)
    if [[ "$SHIPPED" -gt 0 ]]; then
        ok "OmegaOS slash commands installed (/omg-* canonical + /omega-* aliases — $SHIPPED /omg-* in $CLAUDE_CMD_DST/)"
    fi
fi

# Export operational rules to ~/.omega/rules/
# Two passes:
#   1. Copy the canonical .md files from the repo (covers disk-only rules)
#   2. Run `omega rules export` (covers rules registered in code)
# The code-rules will overwrite any disk-rule with the same filename, so the
# binary stays the source of truth when both define the same id.
info "Exporting operational rules..."
mkdir -p "$OMEGA_DIR/rules"
if [[ -d "$OMEGA_SRC/rules" ]]; then
    cp "$OMEGA_SRC/rules"/*.md "$OMEGA_DIR/rules/" 2>/dev/null || true
fi
"$INSTALL_DIR/omega" rules export 2>/dev/null || true
RULES_COUNT=$(ls "$OMEGA_DIR/rules" 2>/dev/null | wc -l)
ok "Rules exported to $OMEGA_DIR/rules/ ($RULES_COUNT files)"

# Copy agent prompts to ~/.omega/agents/
cp -r "$OMEGA_SRC/agents/"* "$AGENTS_DIR/" 2>/dev/null || true
ok "Agent prompts installed"

# Sync rules + OMEGA.md into all LLM config directories
info "Syncing to LLM config directories..."
"$INSTALL_DIR/omega" sync 2>/dev/null || true
ok "LLM configs synced (Claude, Gemini, Codex)"

# Schedule the self-improvement patrol (curator auto-trigger + trajectory
# pruning). Idempotent — only adds the cron line if it's not already there.
# The patrol watches ~/.omega/state/oracle-*.done.json and dispatches a
# curator worker the first time each mission finishes.
mkdir -p "$OMEGA_DIR/logs"
# Each cron line ends with a unique literal marker so the idempotency check is
# exact (grep -qF on the marker, not a broad substring like "omega patrol" that
# would also match "omega patrol-supervisor" or a renamed command).
PATROL_CRON="* * * * * $INSTALL_DIR/omega patrol --once >> $OMEGA_DIR/logs/omega-patrol.log 2>&1   # OMEGA-CRON-PATROL-v1"
USAGE_CRON="*/10 * * * * $INSTALL_DIR/omega usage --check >> $OMEGA_DIR/logs/omega-usage.log 2>&1   # OMEGA-CRON-USAGE-v1"
if command -v crontab >/dev/null 2>&1; then
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-PATROL-v1"; then
        ok "Self-improvement patrol already scheduled"
    else
        ( crontab -l 2>/dev/null; echo "$PATROL_CRON" ) | crontab -
        ok "Self-improvement patrol scheduled (every minute → curator auto-trigger)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-USAGE-v1"; then
        ok "Token-budget usage alert already scheduled"
    else
        ( crontab -l 2>/dev/null; echo "$USAGE_CRON" ) | crontab -
        ok "Token-budget usage alert scheduled (every 10 min → 80%/90% Telegram alert)"
    fi
else
    info "crontab not available — run 'omega patrol' + 'omega usage --check' manually or via your scheduler"
fi

# Install OPTIONAL rmux keybinding config — user has to opt-in via:
#   omega install-bindings
mkdir -p "$OMEGA_DIR"
if [[ -f config/rmux.conf.omega ]]; then
    cp config/rmux.conf.omega "$OMEGA_DIR/rmux.conf.omega"
    ok "rmux config available at $OMEGA_DIR/rmux.conf.omega (run 'omega install-bindings' to activate)"
fi

# Hook into user's rmux config (idempotent — only adds source line if absent)
RMUX_CONF="${RMUX_CONF:-$HOME/.rmux.conf}"
RMUX_SOURCE_LINE="source-file $OMEGA_DIR/rmux.conf.omega"
if [[ -f "$RMUX_CONF" ]]; then
    if ! grep -qF "rmux.conf.omega" "$RMUX_CONF" 2>/dev/null; then
        echo "" >> "$RMUX_CONF"
        echo "# OmegaOS keybindings (Option+Z launches session manager)" >> "$RMUX_CONF"
        echo "$RMUX_SOURCE_LINE" >> "$RMUX_CONF"
        ok "Added OmegaOS source line to $RMUX_CONF"
    else
        ok "$RMUX_CONF already sources OmegaOS bindings"
    fi
else
    echo "# Auto-generated by OmegaOS installer" > "$RMUX_CONF"
    echo "$RMUX_SOURCE_LINE" >> "$RMUX_CONF"
    ok "Created $RMUX_CONF with OmegaOS bindings"
fi

# System-wide rmux config so EVERY user — root and any future account — gets the
# same hardened session (mouse/scroll/clipboard/escape-time/truecolor + Option+Z),
# not just the installing user. rmux reads /etc/rmux.conf on server start (proven
# empirically), so we drop a world-readable copy under /etc/omega and source it
# from /etc/rmux.conf idempotently. Best-effort: needs root; if unavailable the
# per-user ~/.rmux.conf above still covers the installing user.
SUDO=""; [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
if [[ -f config/rmux.conf.omega ]] && { [[ -z "$SUDO" ]] || sudo -n true 2>/dev/null; }; then
    if $SUDO mkdir -p /etc/omega 2>/dev/null && $SUDO cp config/rmux.conf.omega /etc/omega/rmux.conf.omega 2>/dev/null; then
        $SUDO chmod 644 /etc/omega/rmux.conf.omega 2>/dev/null
        GLOBAL_SRC="source-file /etc/omega/rmux.conf.omega"
        if [[ -f /etc/rmux.conf ]] && grep -qF "/etc/omega/rmux.conf.omega" /etc/rmux.conf 2>/dev/null; then
            ok "/etc/rmux.conf already sources OmegaOS config (all users)"
        else
            printf '\n# OmegaOS — shared session config for all users (root + future accounts)\n%s\n' "$GLOBAL_SRC" | $SUDO tee -a /etc/rmux.conf >/dev/null 2>&1 \
                && ok "System-wide rmux config wired (/etc/rmux.conf → all users)" \
                || info "Could not write /etc/rmux.conf (per-user config still applies)"
        fi
    fi
else
    info "Skipped system-wide rmux config (no root) — per-user ~/.rmux.conf still applies"
fi

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

# Detect shell + the files it ACTUALLY reads.
# zsh can relocate its config via ZDOTDIR (e.g. "clean home" setups that point it
# at ~/Linux/zsh). Writing to ~/.zshrc would then be silently ignored — so we ask
# zsh itself for the effective ZDOTDIR. PATH goes in the env file (read by EVERY
# zsh: login, interactive, and non-interactive/scripts); aliases go in the rc.
SHELL_NAME="$(basename "${SHELL:-bash}")"
case "$SHELL_NAME" in
    zsh)
        ZSH_DIR="$(zsh -c 'print -r -- ${ZDOTDIR:-$HOME}' 2>/dev/null)"
        [ -d "$ZSH_DIR" ] || ZSH_DIR="$HOME"
        ENV_FILE="$ZSH_DIR/.zshenv"
        RC_FILE="$ZSH_DIR/.zshrc"
        ;;
    bash) ENV_FILE="$HOME/.bashrc";                  RC_FILE="$HOME/.bashrc" ;;
    fish) ENV_FILE="$HOME/.config/fish/config.fish"; RC_FILE="$ENV_FILE" ;;
    *)    ENV_FILE="$HOME/.profile";                 RC_FILE="$HOME/.profile" ;;
esac

# Add ~/.local/bin to PATH (in the env file so omega/omg resolve in every context)
EXPORT_LINE='export PATH="$HOME/.local/bin:$PATH"'
if ! grep -qF '.local/bin' "$ENV_FILE" 2>/dev/null; then
    { echo ""; echo "# OmegaOS"; echo "$EXPORT_LINE"; } >> "$ENV_FILE"
    ok "Added $INSTALL_DIR to PATH in $ENV_FILE"
else
    ok "PATH already includes $INSTALL_DIR"
fi

# Claude Code: render in the terminal's NORMAL screen, not the alternate screen.
# Inside an rmux pane this lets the WHOLE conversation flow into rmux's scrollback
# (history-limit 500000) so you can scroll the full session in the panel — instead
# of Claude's fullscreen buffer capping the scrollback at ~one screen. Set in the
# env file so EVERY Claude Code invocation (any pane, login or not) inherits it.
# Tradeoff: no fixed-position fullscreen UI (Claude becomes a scrolling transcript).
if ! grep -qF 'CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN' "$ENV_FILE" 2>/dev/null; then
    { echo ""; echo "# OmegaOS — full conversation scrolls in the rmux panel"; \
      echo "export CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1"; } >> "$ENV_FILE"
    ok "Claude Code → normal-screen (full conversation scrolls in the rmux panel)"
else
    ok "Claude Code normal-screen already set"
fi

# Add omega alias for session manager (interactive rc)
ALIAS_LINE='alias om="omega menu"'
if ! grep -qF 'alias om=' "$RC_FILE" 2>/dev/null; then
    echo "$ALIAS_LINE" >> "$RC_FILE"
    ok "Added 'om' alias for omega menu"
fi

# ─── Phase 6.5: Self-containment (hooks, identity, persistent service, headless) ──
# Everything here makes a fresh install reproduce the LIVE system with zero
# dependency on the retired ~/.aisb legacy layer.

step "Phase 6.5: Self-containment"

# (a) Agent-tracking + verify hooks → ~/.omega/hooks, registered in settings.json.
HOOKS_DST="$OMEGA_DIR/hooks"
mkdir -p "$HOOKS_DST"
if [[ -d "$OMEGA_SRC/scripts/hooks" ]]; then
    cp -f "$OMEGA_SRC/scripts/hooks/"*.sh "$HOOKS_DST/" 2>/dev/null && chmod +x "$HOOKS_DST/"*.sh 2>/dev/null
    CLAUDE_SETTINGS="$HOME/.claude/settings.json"
    mkdir -p "$HOME/.claude"
    [[ -f "$CLAUDE_SETTINGS" ]] || echo '{}' > "$CLAUDE_SETTINGS"
    if command -v jq >/dev/null 2>&1; then
        TMP="$(mktemp)"
        jq --arg track "$HOOKS_DST/track-tool-use.sh" --arg verify "$HOOKS_DST/stop-verify-hook.sh" '
          .hooks = (.hooks // {})
          | .hooks.PostToolUse = ((.hooks.PostToolUse // []) | map(select(((.hooks[0].command // "") | test("track-tool-use")) | not)) + [{"matcher":"*","hooks":[{"type":"command","command":$track}]}])
          | .hooks.Stop = ((.hooks.Stop // []) | map(select(((.hooks[0].command // "") | test("stop-verify")) | not)) + [{"hooks":[{"type":"command","command":$verify}]}])
        ' "$CLAUDE_SETTINGS" > "$TMP" 2>/dev/null && mv "$TMP" "$CLAUDE_SETTINGS" && ok "Hooks installed + registered (PostToolUse track + Stop verify)" || { rm -f "$TMP"; info "Hook merge skipped (jq error) — hooks copied to $HOOKS_DST"; }
    else
        info "jq not found — hooks copied to $HOOKS_DST; install jq to auto-register them in settings.json"
    fi
fi

# (b) Identity baseline (never overwrites an existing SOUL.md).
if [[ -f "$OMEGA_SRC/agents/identity/SOUL.template.md" && ! -f "$OMEGA_DIR/SOUL.md" ]]; then
    cp "$OMEGA_SRC/agents/identity/SOUL.template.md" "$OMEGA_DIR/SOUL.md"
    ok "Identity template → $OMEGA_DIR/SOUL.md (customize it; keep private data in MEMORY.md)"
fi

# (c) Telegram interface → the OmegaOS command bot (button-driven, omega CLI).
# This is the operator's phone control center: every command opens an inline
# keyboard of sub-actions, each running an `omega` action (/status /sessions
# /projects /audits /account /model /dispatch /killall /clean …). It is the
# SINGLE Telegram poller (Telegram allows one getUpdates consumer per bot token),
# shipped to ~/.omega and run as a persistent systemd --user service. Connect it
# with:  omega-tg-up <BOT_TOKEN> <YOUR_TELEGRAM_USER_ID>
loginctl enable-linger "${USER:-$(id -un)}" 2>/dev/null || true
# Retire the old native Rust bridge unit (would 409-conflict on the bot token).
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user disable --now omega-telegram.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/omega-telegram.service" \
          "$HOME/.config/systemd/user/default.target.wants/omega-telegram.service" 2>/dev/null || true
    systemctl --user daemon-reload 2>/dev/null || true
fi
if [[ -f "$OMEGA_SRC/telegram-bot/omega-tg-bot.ts" ]]; then
    mkdir -p "$OMEGA_DIR/telegram-bot"
    cp -f "$OMEGA_SRC/telegram-bot/omega-tg-bot.ts" "$OMEGA_DIR/telegram-bot/omega-tg-bot.ts"
    # connect helper on PATH
    if [[ -f "$OMEGA_SRC/scripts/omega-tg-up.sh" ]]; then
        cp -f "$OMEGA_SRC/scripts/omega-tg-up.sh" "$OMEGA_DIR/bin/omega-tg-up.sh"
        chmod +x "$OMEGA_DIR/bin/omega-tg-up.sh"
        ln -sf "$OMEGA_DIR/bin/omega-tg-up.sh" "$INSTALL_DIR/omega-tg-up" 2>/dev/null || true
    fi
    BUN_BIN="$(command -v bun || true)"; [[ -z "$BUN_BIN" && -x "$HOME/.bun/bin/bun" ]] && BUN_BIN="$HOME/.bun/bin/bun"
    # The command bot needs Bun specifically (uses Bun.$/Bun.spawn). install_bun_optional
    # may have skipped it when node/npx was already present, so ensure it here.
    if [[ -z "$BUN_BIN" ]]; then
        info "Installing bun (required by the Telegram command bot)…"
        # bun.sh/install needs unzip — ensure it (the #1 reason bun bootstrap fails).
        command -v unzip >/dev/null 2>&1 || { command -v apt-get >/dev/null 2>&1 && sudo apt-get install -y unzip >/dev/null 2>&1 || true; }
        curl -fsSL https://bun.sh/install | bash >/dev/null 2>&1 || true
        [[ -x "$HOME/.bun/bin/bun" ]] && { BUN_BIN="$HOME/.bun/bin/bun"; export PATH="$HOME/.bun/bin:$PATH"; }
        # last resort: npm-provided bun (no unzip needed)
        [[ -z "$BUN_BIN" ]] && command -v npm >/dev/null 2>&1 && { npm install -g bun >/dev/null 2>&1 || true; BUN_BIN="$(command -v bun || true)"; }
    fi
    if command -v systemctl >/dev/null 2>&1 && [[ -n "$BUN_BIN" ]]; then
        SD_DIR="$HOME/.config/systemd/user"; mkdir -p "$SD_DIR"
        cat > "$SD_DIR/omega-tg-bot.service" <<EOF
[Unit]
Description=OmegaOS Telegram command bot (omega CLI control center)
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
WorkingDirectory=%h/.omega/telegram-bot
ExecStart=$BUN_BIN %h/.omega/telegram-bot/omega-tg-bot.ts
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload 2>/dev/null || true
        if [[ -f "$OMEGA_DIR/telegram.toml" ]] && grep -q 'bot_token *= *"[0-9]' "$OMEGA_DIR/telegram.toml" 2>/dev/null; then
            systemctl --user enable --now omega-tg-bot.service 2>/dev/null || true
            ok "Telegram command bot enabled + started — open your bot, tap /menu (button-driven commands)"
        else
            systemctl --user enable omega-tg-bot.service 2>/dev/null || true
            ok "Telegram command bot installed — connect it:  omega-tg-up <BOT_TOKEN> <YOUR_TELEGRAM_USER_ID>"
        fi
    else
        info "Telegram command bot shipped, but bun is missing — install bun, then: omega-tg-up <BOT_TOKEN> <USER_ID>"
    fi
else
    info "Telegram command bot source not found — skipping (optional phone interface)"
fi

# (d) Claude Code agent binary — omega needs it to spawn agents.
if ! command -v claude >/dev/null 2>&1; then
    info "Claude Code CLI absent — omega needs it to spawn agents. Attempting install..."
    timeout 180 "$INSTALL_DIR/omega" install claude 2>/dev/null || info "Run 'omega install claude' (or install Claude Code manually), then authenticate with 'claude'."
fi

# (e+f) Browser stack (Xvfb + Playwright + Chromium) for PDF generation and the
# visual Quality Arsenal audits (uiux/flow/a11y/perf, browser-tester) + CDP.
#
# DEFERRED BY DEFAULT. The Chromium download (~150MB) + `playwright install-deps`
# (dozens of apt libs) was the single biggest chunk of install time, yet it is
# needed only when you actually generate a PDF or run a visual audit — not for
# core omega, orchestration, agents, or Telegram. So we DON'T pull it at install
# time. Opt in with OMEGA_WITH_BROWSER=1 (or run the one-liner below later); a
# fresh box is usable in seconds instead of minutes.
if [[ "${OMEGA_WITH_BROWSER:-0}" == "1" && "${OMEGA_SKIP_BROWSER:-0}" != "1" ]]; then
    if ! command -v Xvfb >/dev/null 2>&1 && command -v apt-get >/dev/null 2>&1; then
        sudo apt-get install -y xvfb >/dev/null 2>&1 && ok "Xvfb installed (headless PDF/Playwright)" || info "For headless PDF/browser: 'sudo apt-get install xvfb'"
    fi
    if command -v npm >/dev/null 2>&1; then
        command -v playwright >/dev/null 2>&1 || {
            info "Installing Playwright CLI..."
            npm install -g playwright >/dev/null 2>&1 && ok "Playwright CLI installed" || info "Playwright CLI install failed"
        }
        if command -v playwright >/dev/null 2>&1; then
            playwright install chromium >/dev/null 2>&1 && ok "Chromium installed (Playwright + CDP ready)" || info "Chromium download failed — run 'playwright install chromium'"
            if command -v apt-get >/dev/null 2>&1; then
                sudo env "PATH=$PATH" playwright install-deps chromium >/dev/null 2>&1 && ok "Chromium system deps installed" || info "For Chromium libs: 'sudo playwright install-deps chromium'"
            fi
        fi
    else
        info "npm not found — install Node.js, then 'npm i -g playwright && playwright install chromium'"
    fi
else
    info "Browser stack deferred (saves ~150MB + apt deps). PDF/visual audits need it:"
    info "  re-run with OMEGA_WITH_BROWSER=1 ./install.sh  — or:  npm i -g playwright && playwright install --with-deps chromium"
fi

# ─── Phase 6.9: Companion tools + skills (SST multi-LLM, best-effort, opt-out) ──
# planning-with-files, higgsfield (CLI+skills), claude-mem, superpowers,
# mempalace, remotion, + the best-practice reference. Sourced so its OK/WARN
# lines fold into this install run; never fatal. Skip: OMEGA_SKIP_COMPANION=1.
# DEFERRED BY DEFAULT (opt-in OMEGA_WITH_COMPANION=1). These are extra skill
# packs that fetch over the network (bun/npm/git) — useful, but not core, and a
# slow/stalled fetch here was a big chunk of install time. Core OmegaOS (omega,
# rmux, the Quality Arsenal audits, agents, rules, doctrine) is fully installed
# without them; add them anytime with OMEGA_WITH_COMPANION=1 ./install.sh.
if [[ "${OMEGA_WITH_COMPANION:-0}" == "1" && "${OMEGA_SKIP_COMPANION:-0}" != "1" && -f "$OMEGA_SRC/scripts/install-companion-tools.sh" ]]; then
    # shellcheck source=/dev/null
    source "$OMEGA_SRC/scripts/install-companion-tools.sh" || info "companion tools step had warnings (non-fatal)"
else
    info "Companion skill packs deferred (planning-with-files, claude-mem, superpowers, …). Add: OMEGA_WITH_COMPANION=1 ./install.sh"
fi

# ─── Phase 6.95: Telegram interface → OmegaMC (Agentik-Telegram, Go + Docker) ───
# OmegaMC (agentik-os/agentik-telegram) is the OmegaOS Telegram control plane: it
# routes Telegram messages to named AISB agents — each running Claude Code in its
# own Docker container — with @agent routing + a Mission Control web UI. This is
# the "talk to your agents from your phone" layer (RESTORES the powerful Go+Docker
# OmegaMC; supersedes the interim single-session Bun bot). The heavy image builds
# (chromium + embedding model) are DEFERRED to connect-time, exactly like the
# browser stack: cloning is cheap; `omega-mc-up` builds the three local images and
# `docker compose up -d` on demand (the images are not published to GHCR because
# the agent image bundles the Claude Code binary, so we build from source). The
# bring-up reads the bot token from ~/.omega/telegram.toml and the Claude OAuth
# token from ~/.omega/credentials/claude.json. Skip entirely: OMEGA_SKIP_DASHBOARD=1.
if [[ "${OMEGA_SKIP_DASHBOARD:-0}" != "1" ]]; then
    MC_DIR="$OMEGA_DIR/repos/omega-mc"
    mkdir -p "$OMEGA_DIR/repos"
    if [[ -d "$MC_DIR/.git" ]]; then
        ok "OmegaMC present ($MC_DIR — update: git -C $MC_DIR pull && omega-mc-up --rebuild)"
    elif timeout 120 git clone --depth 1 https://github.com/agentik-os/agentik-telegram.git "$MC_DIR" >/dev/null 2>&1; then
        ok "OmegaMC cloned → $MC_DIR"
    else
        rm -rf "$MC_DIR" 2>/dev/null || true
        info "OmegaMC clone skipped — repo unreachable (no git auth?). Later: git clone https://github.com/agentik-os/agentik-telegram.git $MC_DIR"
    fi
    # Ship the bring-up helper (generates .env from OmegaOS state → builds images
    # → docker compose up) onto the omega bin PATH.
    if [[ -f "$OMEGA_SRC/scripts/omega-mc-up.sh" ]]; then
        cp -f "$OMEGA_SRC/scripts/omega-mc-up.sh" "$OMEGA_DIR/bin/omega-mc-up.sh"
        chmod +x "$OMEGA_DIR/bin/omega-mc-up.sh"
        # symlink onto PATH ($INSTALL_DIR is added to PATH above) so `omega-mc-up` just works
        ln -sf "$OMEGA_DIR/bin/omega-mc-up.sh" "$INSTALL_DIR/omega-mc-up" 2>/dev/null || true
    fi
    # Seed the AISB 13-agent roster as the active config if absent.
    if [[ -d "$MC_DIR/config" && ! -f "$MC_DIR/config/omega-mc.yaml" && -f "$MC_DIR/config/omega-aisb.yaml" ]]; then
        cp "$MC_DIR/config/omega-aisb.yaml" "$MC_DIR/config/omega-mc.yaml"
    fi
    if [[ -d "$MC_DIR/.git" ]]; then
        # OmegaMC is the OPTIONAL multi-agent backend (Claude-per-container + Mission
        # Control web UI). It is NOT auto-started: it would poll the same bot token as
        # the command bot (Phase c), and Telegram allows ONE poller per token. Run it
        # on a SEPARATE bot token, or stop omega-tg-bot first, then: omega-mc-up.
        info "OmegaMC (optional multi-agent backend) installed → $MC_DIR. It needs its OWN bot token (don't share the command bot's). Bring up: omega-mc-up <BOT_TOKEN> <CHAT_ID>  (needs Docker)"
    fi
fi

# ─── Done ─────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}  OmegaOS v${OMEGA_VERSION} installed successfully!${NC}"
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo ""
echo "  Quick start:"
echo "    omega                   # Launch the TUI session manager"
echo "    omega list              # List all sessions"
echo "    omega master            # Attach the AISB Master (the 13-agent brain)"
echo ""
echo "  Install optional CLI agents (each is a one-line installer):"
echo "    omega install hermes    # Hermes from Nous Research"
echo "    omega install pi        # Pi from earendil-works"
echo "    omega install codex     # Codex from OpenAI"
echo "    omega install gemini    # Gemini from Google"
echo "    omega install glm       # GLM from Z.AI"
echo "    omega install --help"
echo ""
echo "  Quality Arsenal (23 forensic audits):"
echo "    omega audit list              # Show all 23 Quality Arsenal audits"
echo "    omega audit select \"fix auth\"  # See which audits apply"
echo ""
echo "  Optional Telegram bridge (talk to AISB from your phone):"
echo "    omega telegram setup <BOT_TOKEN> <CHAT_ID> --user-id <YOUR_USER_ID>"
echo "    omega telegram run"
echo ""
echo "  Optional global keybinding (popup omega from any rmux session):"
echo "    omega install-bindings  # binds Ctrl+Space, Ctrl-B z, Ctrl-B o"
echo ""
echo "  Config files: $OMEGA_DIR/config.toml + $OMEGA_DIR/providers.toml (created when you add a provider)"
echo ""
echo "  Restart your shell or run:  source $RC_FILE"
echo ""
