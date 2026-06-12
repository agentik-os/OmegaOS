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

# Single source of truth: the workspace Cargo.toml version. Launched via
# curl|bash there is no Cargo.toml in $PWD yet — re-derived after the Phase 4
# clone (cd "$OMEGA_SRC"), so the epilogue never shows a stale hardcoded number.
# `|| true`: under set -euo pipefail a missing Cargo.toml makes grep exit 2,
# pipefail propagates it into the assignment, and errexit killed the documented
# curl|bash path RIGHT HERE — instantly, before any output (proven empirically).
OMEGA_VERSION="$(grep -m1 '^version' Cargo.toml 2>/dev/null | cut -d'"' -f2 || true)"
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
REPO_URL="https://github.com/agentik-os/OmegaOS"
RMUX_REPO="https://github.com/agentik-os/rmux"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
err()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }
step()  { echo -e "\n${BOLD}==> $*${NC}"; }

# Privileged command runner — the ONE way this script touches sudo that can
# ever PROMPT (sole exception: the system-wide rmux config block in Phase 5
# uses raw `sudo` but self-gates on `sudo -n true`, so it never prompts). Tries
# passwordless `sudo -n` first; if sudo wants a password AND a real terminal is
# attached (stdin + stderr are TTYs — i.e. NOT hidden behind the npx animation),
# falls back to ONE visible interactive prompt (the pre-1.5 behavior; sudo
# caches the timestamp so later calls stay silent). Headless with no cached
# credential → explicit, actionable error + rc 1, so each caller decides:
# required steps abort with context, optional steps degrade. Never an invisible
# hang, never a silent errexit death.
omega_sudo() {
    if [[ "$(id -u)" -eq 0 ]]; then "$@"; return; fi
    if ! command -v sudo >/dev/null 2>&1; then
        err "Privilege needed for '$1' but sudo is not installed — re-run as root or install sudo first."
        return 1
    fi
    if sudo -n true 2>/dev/null; then
        sudo -n "$@"; return
    fi
    if [[ -t 0 && -t 2 ]]; then
        info "sudo needs your password (for: $*)"
        sudo "$@"; return
    fi
    err "sudo needs a password and no TTY is available — run 'sudo -v' first, or rerun in a terminal. (skipped: sudo $*)"
    return 1
}

# Portable timeout: GNU coreutils `timeout` is guaranteed on Linux; macOS only
# grew a native /usr/bin/timeout in macOS 13, and older boxes have at best
# brew's `gtimeout`. Exit code 127 from a missing binary silently disabled the
# claude auto-install + the Agentik-Skills/OmegaMC clones on such Macs (the
# `||`/`elif` guards turned it into misleading "no auth?" skip messages).
# Fallback: run un-timed — strictly better than not running at all (git is
# already prompt-proofed via GIT_TERMINAL_PROMPT=0, so clones fail fast).
omega_timeout() {
    if command -v timeout >/dev/null 2>&1; then timeout "$@"
    elif command -v gtimeout >/dev/null 2>&1; then gtimeout "$@"
    else shift; "$@"
    fi
}

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
    # Privileged installs go through omega_sudo (top of file): passwordless when
    # cached, ONE visible prompt when a terminal is attached, and an explicit
    # error when headless. Every branch records failure in pkg_rc so a failed
    # bootstrap can NEVER fall through to the "ok" line below (the old
    # `sudo -n … && …` apt list survived errexit and printed a false ok; the
    # single-command dnf/yum/pacman/apk branches died with no message at all).
    local pkg_rc=0
    if command -v apt-get >/dev/null 2>&1; then
        { omega_sudo apt-get update -qq && omega_sudo apt-get install -y curl git ca-certificates rsync jq; } || pkg_rc=$?
    elif command -v dnf >/dev/null 2>&1; then
        omega_sudo dnf install -y curl git ca-certificates rsync jq || pkg_rc=$?
    elif command -v yum >/dev/null 2>&1; then
        omega_sudo yum install -y curl git ca-certificates rsync jq || pkg_rc=$?
    elif command -v pacman >/dev/null 2>&1; then
        omega_sudo pacman -Sy --noconfirm curl git ca-certificates rsync jq || pkg_rc=$?
    elif command -v apk >/dev/null 2>&1; then
        omega_sudo apk add --no-cache curl git ca-certificates rsync jq || pkg_rc=$?
    elif command -v brew >/dev/null 2>&1; then
        # macOS with Homebrew. brew is non-interactive and must not run as root.
        brew install "${need[@]}" || pkg_rc=$?
    elif [[ "$(uname -s)" == "Darwin" ]]; then
        # macOS without Homebrew: git/curl are hard requirements (ship with the
        # Xcode Command Line Tools); rsync/jq only degrade optional features
        # (jq → hook auto-registration, reported by `omega doctor`) — warn and
        # continue instead of killing the whole install.
        local critical=()
        command -v git  >/dev/null 2>&1 || critical+=("git")
        command -v curl >/dev/null 2>&1 || critical+=("curl")
        if [[ ${#critical[@]} -gt 0 ]]; then
            err "Missing ${critical[*]} — install the Xcode Command Line Tools (xcode-select --install) or Homebrew, then re-run."
            exit 1
        fi
        info "Optional tools missing (${need[*]}) — continuing; install later with: brew install ${need[*]}"
        return 0
    else
        err "No supported package manager (apt/dnf/yum/pacman/apk/brew)."
        err "Install these manually, then re-run ./install.sh: ${need[*]}"
        exit 1
    fi
    if [[ $pkg_rc -ne 0 ]]; then
        err "Runtime prerequisite install failed (${need[*]}) — fix the sudo/package error above, then re-run ./install.sh"
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
        local tc_rc=0
        if   command -v apt-get >/dev/null 2>&1; then { omega_sudo apt-get update -qq && omega_sudo apt-get install -y build-essential pkg-config; } || tc_rc=$?
        elif command -v dnf     >/dev/null 2>&1; then omega_sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config || tc_rc=$?
        elif command -v yum     >/dev/null 2>&1; then omega_sudo yum install -y gcc gcc-c++ make pkgconfig || tc_rc=$?
        elif command -v pacman  >/dev/null 2>&1; then omega_sudo pacman -Sy --noconfirm base-devel pkgconf || tc_rc=$?
        elif command -v apk     >/dev/null 2>&1; then omega_sudo apk add --no-cache build-base pkgconf || tc_rc=$?
        elif [[ "$OS" == "Darwin" ]]; then
            # macOS source build (e.g. Intel — no prebuilt asset in the release):
            # pkg-config comes from Homebrew (mirrors bootstrap_os_packages' brew
            # handling — non-interactive, never root); cc ships with the Xcode
            # Command Line Tools, which a working brew already implies.
            if command -v brew >/dev/null 2>&1; then
                brew install pkg-config || tc_rc=$?
            else
                err "macOS source build needs Homebrew for pkg-config — install it from https://brew.sh, then re-run ./install.sh"
                exit 1
            fi
        else err "No supported package manager for the build toolchain — install a C compiler + pkg-config, then re-run."; exit 1
        fi
        if [[ $tc_rc -ne 0 ]]; then
            err "Build toolchain install failed — fix the error above (or install a C compiler + pkg-config manually), then re-run ./install.sh"
            exit 1
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
    # Genuinely optional: ANY failure (sudo, package manager, missing package)
    # degrades to an info line — the old bare `$SUDO … 2>/dev/null` statements
    # killed the WHOLE install under errexit AND hid the only diagnostic.
    if   command -v apt-get >/dev/null 2>&1; then omega_sudo apt-get install -y mosh || info "mosh skipped (sudo unavailable)"
    elif command -v dnf     >/dev/null 2>&1; then omega_sudo dnf install -y mosh || info "mosh skipped (sudo unavailable)"
    elif command -v yum     >/dev/null 2>&1; then omega_sudo yum install -y mosh || info "mosh skipped (sudo unavailable)"
    elif command -v pacman  >/dev/null 2>&1; then omega_sudo pacman -Sy --noconfirm mosh || info "mosh skipped (sudo unavailable)"
    elif command -v apk     >/dev/null 2>&1; then omega_sudo apk add --no-cache mosh || info "mosh skipped (sudo unavailable)"
    fi
    command -v mosh-server >/dev/null 2>&1 \
        && ok "mosh installed — connect with 'mosh <host> -- rmux attach' for lag-free typing/streaming (NOTE: mosh does not sync mouse-mode, so wheel-scroll/click/drag are dead over mosh — use plain SSH when you want the mouse; keyboard scroll PgUp/PgDn & Alt+Up/Down works over both)" \
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
    # macOS is natively UTF-8 and /etc/default/locale is a Debian-ism: writing it
    # via sudo prompted for a password BEHIND the npx animation — an invisible,
    # infinite hang at ~20% (proven on a real Mac install). Nothing to do here.
    if [[ "$(uname -s)" == "Darwin" ]]; then ok "UTF-8 locale native (macOS)"; return 0; fi
    if locale -a 2>/dev/null | grep -qiE 'en_US\.utf-?8'; then ok "UTF-8 locale present (en_US.UTF-8)"; else
        # Every command here is best-effort: on a minimal image the `locales`
        # package (and its en_US source) may be absent, so locale-gen/localedef
        # can fail. Under `set -euo pipefail` an unguarded failure would ABORT
        # the whole install (proven in a clean ubuntu:24.04 container) — so each
        # is `|| true`. A missing UTF-8 locale degrades to C.UTF-8, never fatal.
        if command -v locale-gen >/dev/null 2>&1; then
            omega_sudo sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen 2>/dev/null || true
            grep -q '^en_US.UTF-8 UTF-8' /etc/locale.gen 2>/dev/null || echo 'en_US.UTF-8 UTF-8' | omega_sudo tee -a /etc/locale.gen >/dev/null 2>&1 || true
            omega_sudo locale-gen 2>/dev/null || true
        elif command -v localedef >/dev/null 2>&1; then
            omega_sudo localedef -i en_US -f UTF-8 en_US.UTF-8 2>/dev/null || true
        fi
        locale -a 2>/dev/null | grep -qiE 'en_US\.utf-?8' \
            && ok "UTF-8 locale generated (en_US.UTF-8)" \
            || info "Could not generate en_US.UTF-8 — falls back to C.UTF-8 (mosh: use -l LANG=C.UTF-8)"
    fi
    # Make it the system default so root + future users get UTF-8 without per-shell setup.
    if [[ ! -s /etc/default/locale ]] || ! grep -q 'LANG=.*UTF-8' /etc/default/locale 2>/dev/null; then
        { echo 'LANG=en_US.UTF-8' | omega_sudo tee /etc/default/locale >/dev/null 2>&1 \
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

    # Freshness gate: a checkout that already has a local release build is a
    # dev/operator box — that build reflects HEAD, while the prebuilt reflects
    # the (older) release tag. Installing the prebuilt here silently DOWNGRADES
    # the binary that was just built from newer commits at the same version
    # number (stale-binary bug, 2026-06-10: doctor showed 21/26 rules because
    # the v0.1.5 release artifact replaced a fresh post-release build). Fresh
    # boxes have no target/release and keep the fast path.
    if [[ -x "$OMEGA_SRC/target/release/omega" ]]; then
        info "Local release build present — preferring source build over prebuilt $tag"
        return 0
    fi

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

# ─── Telegram command bot installer (invoked right after Phase 4) ────────────
# Defined here, run BEFORE the long Phase 5. Phase 5 is set -e and ~556 lines;
# any non-zero command there used to abort the whole install and silently skip
# the bot. Installing the bot first makes it survive a later abort. Idempotent.
# TG_BOT_WARN carries a launchd-failed-to-start warning (headless/SSH Mac) into
# the final summary, where the npx animation can't swallow it.
TG_BOT_WARN=""
install_command_bot() {
    [[ -f "$OMEGA_SRC/telegram-bot/omega-tg-bot.ts" ]] || { info "Telegram command bot source not found — skipping"; return 0; }
    loginctl enable-linger "${USER:-$(id -un)}" 2>/dev/null || true
    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user disable --now omega-telegram.service 2>/dev/null || true
        rm -f "$HOME/.config/systemd/user/omega-telegram.service" \
              "$HOME/.config/systemd/user/default.target.wants/omega-telegram.service" 2>/dev/null || true
        systemctl --user daemon-reload 2>/dev/null || true
    fi
    # bin/ + logs/ are created HERE (not only in Phase 5): this function runs
    # right after Phase 4 precisely so the bot survives a Phase 5 abort — it
    # must not depend on Phase 5's mkdir for its own copy targets.
    mkdir -p "$OMEGA_DIR/telegram-bot" "$OMEGA_DIR/bin" "$OMEGA_DIR/logs"
    cp -f "$OMEGA_SRC/telegram-bot/omega-tg-bot.ts" "$OMEGA_DIR/telegram-bot/omega-tg-bot.ts"
    if [[ -f "$OMEGA_SRC/scripts/omega-tg-up.sh" ]]; then
        cp -f "$OMEGA_SRC/scripts/omega-tg-up.sh" "$OMEGA_DIR/bin/omega-tg-up.sh"
        chmod +x "$OMEGA_DIR/bin/omega-tg-up.sh"
        ln -sf "$OMEGA_DIR/bin/omega-tg-up.sh" "$INSTALL_DIR/omega-tg-up" 2>/dev/null || true
    fi
    local BUN_BIN; BUN_BIN="$(command -v bun || true)"; [[ -z "$BUN_BIN" && -x "$HOME/.bun/bin/bun" ]] && BUN_BIN="$HOME/.bun/bin/bun"
    if [[ -z "$BUN_BIN" ]]; then
        info "Installing bun (required by the Telegram command bot)…"
        command -v unzip >/dev/null 2>&1 || { command -v apt-get >/dev/null 2>&1 && omega_sudo apt-get install -y unzip >/dev/null 2>&1 || true; }
        curl -fsSL https://bun.sh/install | bash >/dev/null 2>&1 || true
        [[ -x "$HOME/.bun/bin/bun" ]] && { BUN_BIN="$HOME/.bun/bin/bun"; export PATH="$HOME/.bun/bin:$PATH"; }
        [[ -z "$BUN_BIN" ]] && command -v npm >/dev/null 2>&1 && { npm install -g bun >/dev/null 2>&1 || true; BUN_BIN="$(command -v bun || true)"; }
    fi
    # BOTH init systems exec a tiny wrapper that resolves bun at RUNTIME: freezing
    # the install-time $BUN_BIN into the unit meant one bun reinstall (new path)
    # → eternal respawn throttle under KeepAlive=true / Restart=always (proven on
    # macOS; the identical hazard existed in the Linux units). The wrapper
    # survives bun moving, so it is written unconditionally.
    local TG_WRAPPER="$OMEGA_DIR/bin/tg-bot-launch.sh"
    mkdir -p "$OMEGA_DIR/logs" "$OMEGA_DIR/bin"
    cat > "$TG_WRAPPER" <<EOF
#!/usr/bin/env bash
# OmegaOS Telegram bot launcher (systemd/launchd → here → bun). Resolves bun at
# runtime so a bun reinstall never bricks the service.
OMEGA_DIR="\${OMEGA_DIR:-$OMEGA_DIR}"
# Size-cap log rotation: launchd never rotates StandardOut/ErrPath, so under
# KeepAlive=true these grow unbounded. Rotate at every launch (KeepAlive
# restarts make launch-time rotation effective): >5MB → COPY to <name>.1 then
# truncate in place (copytruncate). NOT mv: launchd opens these fds at job
# spawn BEFORE this wrapper runs, so mv re-points the live fds at .1 and the
# rotating generation streams uncapped there. copytruncate keeps the fds on
# the truncated original — .1 holds the snapshot at rotation, the live file
# restarts at 0 and is re-capped at every respawn. (Under systemd output goes
# to the journal, these files don't exist, and the loop is a no-op.)
for _log in "\$OMEGA_DIR/logs/tg-bot.log" "\$OMEGA_DIR/logs/tg-bot.err.log"; do
    if [ -f "\$_log" ] && [ "\$(wc -c < "\$_log")" -gt 5242880 ]; then
        cp "\$_log" "\$_log.1" && : > "\$_log"
    fi
done
for cand in "\$(command -v bun 2>/dev/null)" "\$HOME/.bun/bin/bun" /opt/homebrew/bin/bun /usr/local/bin/bun; do
    [ -n "\$cand" ] && [ -x "\$cand" ] && exec "\$cand" "\$OMEGA_DIR/telegram-bot/omega-tg-bot.ts"
done
echo "tg-bot-launch: bun not found (PATH, ~/.bun/bin, /opt/homebrew/bin, /usr/local/bin)" >&2
exit 127
EOF
    chmod +x "$TG_WRAPPER"
    if command -v systemctl >/dev/null 2>&1 && [[ -n "$BUN_BIN" ]]; then
        local SD_DIR="$HOME/.config/systemd/user"; mkdir -p "$SD_DIR"
        cat > "$SD_DIR/omega-tg-bot.service" <<EOF
[Unit]
Description=OmegaOS Telegram command bot (omega CLI control center)
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
WorkingDirectory=%h/.omega/telegram-bot
ExecStart=%h/.omega/bin/tg-bot-launch.sh
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload 2>/dev/null || true
        # ALWAYS enable + start: the bot waits for a token and auto-connects the
        # moment `omega telegram setup` / omega-tg-up writes telegram.toml.
        systemctl --user enable --now omega-tg-bot.service 2>/dev/null || true
        ok "Telegram command bot installed + running (waits for token). Connect: OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <CHAT_ID>  (env form keeps the token out of the process list)  OR  omega-tg-up <TOKEN> <YOUR_USER_ID>"
    elif [[ "$(uname -s)" == "Darwin" && -n "$BUN_BIN" ]]; then
        # macOS: no systemd — install a launchd LaunchAgent instead, same
        # semantics (always running, waits for the token, auto-restarts).
        # Without this the bot never runs on a Mac: telegram.toml gets written
        # but every message goes unanswered (proven on a real Mac install).
        local LA_DIR="$HOME/Library/LaunchAgents"
        local LA_LABEL="os.omega.tg-bot"
        local LA_PLIST="$LA_DIR/$LA_LABEL.plist"
        mkdir -p "$LA_DIR"
        # launchd execs the same runtime bun-resolving wrapper written above.
        cat > "$LA_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>$LA_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$TG_WRAPPER</string>
    </array>
    <key>WorkingDirectory</key><string>$OMEGA_DIR/telegram-bot</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>OMEGA_DIR</key><string>$OMEGA_DIR</string>
        <key>PATH</key><string>$HOME/.local/bin:$HOME/.bun/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
    <key>StandardOutPath</key><string>$OMEGA_DIR/logs/tg-bot.log</string>
    <key>StandardErrorPath</key><string>$OMEGA_DIR/logs/tg-bot.err.log</string>
</dict>
</plist>
EOF
        # Idempotent (re)load: bootout any old instance, then bootstrap; fall
        # back to legacy load -w on older macOS. Capture launchctl's stderr —
        # over SSH with no GUI session `bootstrap gui/$UID` fails, and that
        # failure must be LOUD (warn + final summary), not a suppressed info
        # line the npx animation swallows while claiming "Telegram connected".
        launchctl bootout "gui/$(id -u)/$LA_LABEL" 2>/dev/null || true
        local LC_OUT=""
        if ! LC_OUT="$(launchctl bootstrap "gui/$(id -u)" "$LA_PLIST" 2>&1)"; then
            LC_OUT="$LC_OUT $(launchctl load -w "$LA_PLIST" 2>&1 || true)"
        fi
        if launchctl print "gui/$(id -u)/$LA_LABEL" >/dev/null 2>&1; then
            TG_BOT_WARN=""
            ok "Telegram command bot installed + running via launchd (waits for token). Connect: OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <CHAT_ID>  (env form keeps the token out of the process list)"
        else
            TG_BOT_WARN="Telegram bot NOT started: launchd has no GUI session for this user (typical over SSH / headless). After a GUI login, run: launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/os.omega.tg-bot.plist — or run it now in the foreground: omega telegram run  (logs: $OMEGA_DIR/logs/tg-bot.err.log)"
            warn "$TG_BOT_WARN"
            [[ -n "${LC_OUT// /}" ]] && warn "launchctl said: $LC_OUT" || true
        fi
    else
        info "Telegram command bot shipped, but bun/systemd missing — start later: omega-tg-up <BOT_TOKEN> <USER_ID>"
    fi
}

# Deposit bot — a private inbox (telegram-bot/inbox-bot.ts): the operator sends
# photos/notes from their phone, an agent reads them in ~/.omega/inbox/. Same
# shipping shape as the command bot, minus the cleanup/bun-install (already done
# by install_command_bot, which runs first); config lives in ~/.omega/deposit.toml.
install_inbox_bot() {
    [[ -f "$OMEGA_SRC/telegram-bot/inbox-bot.ts" ]] || { info "Deposit bot source not found — skipping"; return 0; }
    mkdir -p "$OMEGA_DIR/telegram-bot" "$OMEGA_DIR/inbox" "$OMEGA_DIR/bin"
    # Legacy cleanup — a hand-rolled inbox-bot.service (pre-ship deployment:
    # ~/.omega/inbox-bot/ + token.env) polls the same bot token as the shipped
    # omega-inbox-bot.service; two pollers on one token = Telegram 409. Same
    # pattern as the omega-telegram.service cleanup in install_command_bot.
    # One-time token migration FIRST (token.env: INBOX_BOT_TOKEN=… → the
    # deposit.toml `bot_token = "…"` the shipped bot reads), so the operator's
    # deposit inbox keeps working across the unit swap. Never clobbers an
    # existing deposit.toml.
    if [[ -f "$OMEGA_DIR/inbox-bot/token.env" && ! -f "$OMEGA_DIR/deposit.toml" ]]; then
        local LEGACY_TOKEN LEGACY_CHAT
        LEGACY_TOKEN="$(grep -m1 '^INBOX_BOT_TOKEN=' "$OMEGA_DIR/inbox-bot/token.env" 2>/dev/null | cut -d= -f2-)"
        # The legacy bot kept its operator lock in config.json ({"chat_id": N});
        # without migrating it the new bot would DROP every message ("no
        # chat_id/pair_code in deposit.toml") — the token alone is not enough.
        LEGACY_CHAT="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1])).get("chat_id",""))' "$OMEGA_DIR/inbox-bot/config.json" 2>/dev/null || true)"
        if [[ -n "$LEGACY_TOKEN" ]]; then
            printf 'bot_token = "%s"\n' "$LEGACY_TOKEN" > "$OMEGA_DIR/deposit.toml"
            [[ -n "$LEGACY_CHAT" ]] && printf 'chat_id = %s\n' "$LEGACY_CHAT" >> "$OMEGA_DIR/deposit.toml"
            chmod 600 "$OMEGA_DIR/deposit.toml"
            ok "Deposit bot: migrated legacy inbox-bot token.env → deposit.toml${LEGACY_CHAT:+ (chat_id kept)}"
        fi
    fi
    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user disable --now inbox-bot.service 2>/dev/null || true
        rm -f "$HOME/.config/systemd/user/inbox-bot.service" \
              "$HOME/.config/systemd/user/default.target.wants/inbox-bot.service" 2>/dev/null || true
        systemctl --user daemon-reload 2>/dev/null || true
    fi
    cp -f "$OMEGA_SRC/telegram-bot/inbox-bot.ts" "$OMEGA_DIR/telegram-bot/inbox-bot.ts"
    if [[ -f "$OMEGA_SRC/scripts/inbox-bot-up.sh" ]]; then
        cp -f "$OMEGA_SRC/scripts/inbox-bot-up.sh" "$OMEGA_DIR/bin/inbox-bot-up.sh"
        chmod +x "$OMEGA_DIR/bin/inbox-bot-up.sh"
        ln -sf "$OMEGA_DIR/bin/inbox-bot-up.sh" "$INSTALL_DIR/inbox-bot-up" 2>/dev/null || true
    fi
    local BUN_BIN; BUN_BIN="$(command -v bun || true)"; [[ -z "$BUN_BIN" && -x "$HOME/.bun/bin/bun" ]] && BUN_BIN="$HOME/.bun/bin/bun"
    # Runtime bun-resolving wrapper, written unconditionally — same constraint as
    # tg-bot-launch.sh above: a frozen install-time bun path under Restart=always
    # / KeepAlive=true turns one bun reinstall into an eternal respawn throttle.
    local IB_WRAPPER="$OMEGA_DIR/bin/inbox-bot-launch.sh"
    mkdir -p "$OMEGA_DIR/logs" "$OMEGA_DIR/bin"
    cat > "$IB_WRAPPER" <<EOF
#!/usr/bin/env bash
# OmegaOS Deposit bot launcher (systemd/launchd → here → bun). Resolves bun at runtime.
OMEGA_DIR="\${OMEGA_DIR:-$OMEGA_DIR}"
for _log in "\$OMEGA_DIR/logs/inbox-bot.log" "\$OMEGA_DIR/logs/inbox-bot.err.log"; do
    if [ -f "\$_log" ] && [ "\$(wc -c < "\$_log")" -gt 5242880 ]; then cp "\$_log" "\$_log.1" && : > "\$_log"; fi
done
for cand in "\$(command -v bun 2>/dev/null)" "\$HOME/.bun/bin/bun" /opt/homebrew/bin/bun /usr/local/bin/bun; do
    [ -n "\$cand" ] && [ -x "\$cand" ] && exec "\$cand" "\$OMEGA_DIR/telegram-bot/inbox-bot.ts"
done
echo "inbox-bot-launch: bun not found" >&2; exit 127
EOF
    chmod +x "$IB_WRAPPER"
    if command -v systemctl >/dev/null 2>&1 && [[ -n "$BUN_BIN" ]]; then
        local SD_DIR="$HOME/.config/systemd/user"; mkdir -p "$SD_DIR"
        cat > "$SD_DIR/omega-inbox-bot.service" <<EOF
[Unit]
Description=OmegaOS Deposit bot (photo/note inbox → ~/.omega/inbox)
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
WorkingDirectory=%h/.omega/telegram-bot
ExecStart=%h/.omega/bin/inbox-bot-launch.sh
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload 2>/dev/null || true
        # ALWAYS enable + start: the bot idles without a token and auto-connects
        # the moment inbox-bot-up writes deposit.toml.
        systemctl --user enable --now omega-inbox-bot.service 2>/dev/null || true
        ok "Deposit bot installed + running (waits for token). Connect: OMEGA_DEPOSIT_TOKEN=<TOKEN> inbox-bot-up  OR  inbox-bot-up <TOKEN>"
    elif [[ "$(uname -s)" == "Darwin" && -n "$BUN_BIN" ]]; then
        local LA_DIR="$HOME/Library/LaunchAgents"
        local LA_LABEL="os.omega.inbox-bot"
        local LA_PLIST="$LA_DIR/$LA_LABEL.plist"
        mkdir -p "$LA_DIR"
        # launchd execs the same runtime bun-resolving wrapper written above.
        cat > "$LA_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>$LA_LABEL</string>
    <key>ProgramArguments</key>
    <array><string>$IB_WRAPPER</string></array>
    <key>WorkingDirectory</key><string>$OMEGA_DIR/telegram-bot</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>OMEGA_DIR</key><string>$OMEGA_DIR</string>
        <key>PATH</key><string>$HOME/.local/bin:$HOME/.bun/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
    <key>StandardOutPath</key><string>$OMEGA_DIR/logs/inbox-bot.log</string>
    <key>StandardErrorPath</key><string>$OMEGA_DIR/logs/inbox-bot.err.log</string>
</dict>
</plist>
EOF
        launchctl bootout "gui/$(id -u)/$LA_LABEL" 2>/dev/null || true
        launchctl bootstrap "gui/$(id -u)" "$LA_PLIST" 2>/dev/null || launchctl load -w "$LA_PLIST" 2>/dev/null || true
        if launchctl print "gui/$(id -u)/$LA_LABEL" >/dev/null 2>&1; then
            ok "Deposit bot installed + running via launchd (waits for token). Connect: inbox-bot-up <TOKEN>"
        else
            info "Deposit bot shipped; launchd has no GUI session (headless/SSH). Start later after GUI login, or: inbox-bot-up <TOKEN>"
        fi
    else
        info "Deposit bot shipped, but bun/systemd missing — start later: inbox-bot-up <BOT_TOKEN>"
    fi
}

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
# curl|bash path: the version parse at the top of the script ran before the
# clone existed — re-derive from the cloned Cargo.toml (still the single source
# of truth; "unknown" only if the workspace manifest is unparseable).
[[ -n "$OMEGA_VERSION" ]] || OMEGA_VERSION="$(grep -m1 '^version' Cargo.toml 2>/dev/null | cut -d'"' -f2 || true)"
[[ -n "$OMEGA_VERSION" ]] || OMEGA_VERSION="unknown"
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

# Install the Telegram command bot NOW (before the long, fragile Phase 5) so a
# set -e abort later can never skip the operator's phone interface.
install_command_bot || info "command bot setup had warnings (non-fatal)"
install_inbox_bot || info "deposit bot setup had warnings (non-fatal)"

# TTS voice bench (Nova's voices: Pocket/Chatterbox/Kokoro/Piper + ElevenLabs
# proxy via the omega-ttsd gateway). Engine venvs + models are multi-GB — the
# installer is idempotent and per-engine fault-tolerant; a failure here never
# blocks the rest of the install.
if [[ -f "$OMEGA_SRC/tools/tts/install-tts.sh" ]]; then
    bash "$OMEGA_SRC/tools/tts/install-tts.sh" || info "TTS bench setup had warnings (non-fatal — omega-ttsd serves whatever installed)"
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

# Install PDF generator (templates + engine — deps installed on first use).
# Canonical home is ~/.omega/skills/pdfgen — what the omega binary's lookup
# (find_pdfgen_dir) prefers and what R-PDF documents; ~/.omega/pdfgen is the
# LEGACY location, kept alive as a compatibility symlink for old references.
PDFGEN_SRC="$OMEGA_SRC/tools/pdfgen"
PDFGEN_DST="$OMEGA_DIR/skills/pdfgen"
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
    # Migrate the legacy real dir to a symlink. The SOURCE tree there is a pure
    # install copy, but RUNTIME artifacts are not: `omega pdf` runs with the
    # resolved pdfgen dir as CWD, so user-generated PDFs (relative --out) and
    # output/ live INSIDE the legacy dir — rescue those before discarding.
    if [[ -d "$OMEGA_DIR/pdfgen" && ! -L "$OMEGA_DIR/pdfgen" ]]; then
        find "$OMEGA_DIR/pdfgen" -maxdepth 1 -name '*.pdf' -exec mv -n {} "$PDFGEN_DST/" \; 2>/dev/null || true
        if [[ -d "$OMEGA_DIR/pdfgen/output" ]]; then
            mkdir -p "$PDFGEN_DST/output"
            cp -an "$OMEGA_DIR/pdfgen/output/." "$PDFGEN_DST/output/" 2>/dev/null || true
        fi
        rm -rf "$OMEGA_DIR/pdfgen"
    fi
    ln -sfn "$PDFGEN_DST" "$OMEGA_DIR/pdfgen"
    ok "PDF generator installed to $PDFGEN_DST/ (legacy ~/.omega/pdfgen → symlink; deps auto-install on first 'omega pdf')"
else
    info "PDF generator source not found — skipping (can be added later)"
fi

# Bridge config dir: a minimal Claude config (no hooks) for FAST `claude -p`
# calls (~4s vs ~11s with full settings + hooks). OPT-IN — nothing consumes it
# automatically; point a headless call at it with:
#   CLAUDE_CONFIG_DIR=~/.omega/claude-bridge-config claude -p "…"
# Credentials are symlinked so OAuth still works.
BRIDGE_CFG="$OMEGA_DIR/claude-bridge-config"
mkdir -p "$BRIDGE_CFG"
echo '{}' > "$BRIDGE_CFG/settings.json"
# Ensure the symlink target exists so the bridge never reads through a dangling
# link before `claude` login writes real creds (NEVER clobber an existing file).
[[ -e "$OMEGA_DIR/credentials/claude.json" ]] || : > "$OMEGA_DIR/credentials/claude.json"
# Optional shared-credential override (multi-operator / multi-machine setups):
#   OMEGA_CREDENTIALS_LINK=/Shared/claude/credentials.json ./install.sh
# points the local credential at a shared, auto-refreshed source instead of a
# per-box file, so the Telegram bridge never drifts to a stale copy. No-op if unset.
if [[ -n "${OMEGA_CREDENTIALS_LINK:-}" && -e "$OMEGA_CREDENTIALS_LINK" ]]; then
    ln -sfn "$OMEGA_CREDENTIALS_LINK" "$OMEGA_DIR/credentials/claude.json"
    ok "Claude credential linked to shared source: $OMEGA_CREDENTIALS_LINK"
fi
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

# Install proactive token-refresh helper (cron-driven; pre-empts agent 401s by
# refreshing the Claude OAuth token before it expires, alerting via Telegram on failure).
TOKREFRESH_SRC="$OMEGA_SRC/scripts/omega-token-refresh.sh"
if [[ -f "$TOKREFRESH_SRC" ]]; then
    cp "$TOKREFRESH_SRC" "$OMEGA_DIR/bin/omega-token-refresh.sh"
    chmod +x "$OMEGA_DIR/bin/omega-token-refresh.sh"
    ok "Token-refresh helper installed: $OMEGA_DIR/bin/omega-token-refresh.sh"
fi

# Install the end-of-mission notifier (oracle done.json → Telegram report). The
# single, restart-proof notification path: catches every finished oracle (Telegram,
# TUI, or Atlas-dispatched) and relays its report to the project's topic / main chat.
DONENOTIFY_SRC="$OMEGA_SRC/scripts/omega-done-notify.sh"
if [[ -f "$DONENOTIFY_SRC" ]]; then
    cp "$DONENOTIFY_SRC" "$OMEGA_DIR/bin/omega-done-notify.sh"
    chmod +x "$OMEGA_DIR/bin/omega-done-notify.sh"
    ok "End-of-mission notifier installed: $OMEGA_DIR/bin/omega-done-notify.sh"
fi

STUCK_SRC="$OMEGA_SRC/scripts/omega-stuck-oracle-alert.sh"
if [[ -f "$STUCK_SRC" ]]; then
    cp "$STUCK_SRC" "$OMEGA_DIR/bin/omega-stuck-oracle-alert.sh"
    chmod +x "$OMEGA_DIR/bin/omega-stuck-oracle-alert.sh"
    ok "Stuck-oracle alert installed: $OMEGA_DIR/bin/omega-stuck-oracle-alert.sh"
fi

# Canonical alert sender — ONE funnel to the Telegram "Alerts 🚨" topic for every
# operational alert (stuck oracle / self-heal / token refresh). Auto-recreates the
# topic if deleted (undeletable invariant), falls back to the operator DM.
ALERTSEND_SRC="$OMEGA_SRC/scripts/omega-alert-send.sh"
if [[ -f "$ALERTSEND_SRC" ]]; then
    cp "$ALERTSEND_SRC" "$OMEGA_DIR/bin/omega-alert-send.sh"
    chmod +x "$OMEGA_DIR/bin/omega-alert-send.sh"
    ok "Alert sender installed: $OMEGA_DIR/bin/omega-alert-send.sh"
fi

for hs in omega-atlas-brief omega-clean-projects; do
    if [[ -f "$OMEGA_SRC/scripts/$hs.sh" ]]; then
        cp "$OMEGA_SRC/scripts/$hs.sh" "$OMEGA_DIR/bin/$hs.sh"; chmod +x "$OMEGA_DIR/bin/$hs.sh"
        ln -sf "$OMEGA_DIR/bin/$hs.sh" "$INSTALL_DIR/$hs" 2>/dev/null || true
    fi
done
[[ -f "$OMEGA_DIR/bin/omega-atlas-brief.sh" ]] && ok "Atlas briefing + project cleaner installed"

# Install the self-heal daemon (cron → `omega doctor --fix`; alerts Telegram if
# warnings remain so the operator can tap /status → Fix it). Keeps the system
# green unattended (duplicate pollers, dead bot service, stale cache, oauth).
SELFHEAL_SRC="$OMEGA_SRC/scripts/omega-self-heal.sh"
if [[ -f "$SELFHEAL_SRC" ]]; then
    cp "$SELFHEAL_SRC" "$OMEGA_DIR/bin/omega-self-heal.sh"
    chmod +x "$OMEGA_DIR/bin/omega-self-heal.sh"
    ok "Self-heal daemon installed: $OMEGA_DIR/bin/omega-self-heal.sh"
fi

# Install the git branch-per-worker orchestration helpers (oracles isolate parallel
# workers on omega/* branches then merge them back — safe, never force/push). On PATH.
for gh in omega-git-branch omega-git-merge; do
    if [[ -f "$OMEGA_SRC/scripts/$gh.sh" ]]; then
        cp "$OMEGA_SRC/scripts/$gh.sh" "$OMEGA_DIR/bin/$gh.sh"
        chmod +x "$OMEGA_DIR/bin/$gh.sh"
        ln -sf "$OMEGA_DIR/bin/$gh.sh" "$INSTALL_DIR/$gh" 2>/dev/null || true
    fi
done
[[ -f "$OMEGA_DIR/bin/omega-git-merge.sh" ]] && ok "Git branch orchestration installed (omega-git-branch + omega-git-merge)"

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

# Install the maintenance skills (cleanup, project-tidy) — VPS/disk cleanup +
# project tidying (docs/ + agentic/ convention, doc↔app coherence). Portable
# scripts (no machine-specific paths). Mirrors the design loop: copy →
# ~/.omega/skills/<name>/ + /<name> and /omg-<name> slash stubs.
for MSK in cleanup project-tidy ramflush; do
    MSK_SRC="$OMEGA_SRC/skills/$MSK"
    MSK_DST="$OMEGA_DIR/skills/$MSK"
    if [[ -d "$MSK_SRC" ]]; then
        mkdir -p "$MSK_DST"
        cp -r "$MSK_SRC"/* "$MSK_DST/"
        find "$MSK_DST" -name "*.sh" -exec chmod +x {} + 2>/dev/null || true
        MCMD="$HOME/.claude/commands"; mkdir -p "$MCMD"
        for cmd in "$MSK" "omg-$MSK"; do
            cat > "$MCMD/$cmd.md" <<EOF
# /$cmd

Run the $MSK maintenance skill. Read and follow the complete instructions in:

\`$MSK_DST/SKILL.md\`

Use every reference, template, and script it provides.
EOF
        done
        ok "Maintenance skill installed: $MSK → ~/.omega/skills/$MSK/ (/$MSK + /omg-$MSK)"
    else
        info "Maintenance skill $MSK not found — skipping"
    fi
done

# Install the marketing / go-to-market suite (R-MARKETING) + the Higgsfield
# visual-identity pair (R-VISUAL-ID) — 10 vendored third-party skills adapted to
# OmegaOS conventions — plus the OmegaOS-native `diagram` visual skill. Prompt-only
# except market-research (gooseworks API, user creds), the higgsfield pair, and
# diagram (external `d2` / mermaid-cli renderers — all installed lazily at RUNTIME
# by the skill itself, NEVER auto-installed here). Mirrors the maintenance loop:
# copy → ~/.omega/skills/<name>/ + /<name> and /omg-<name> slash stubs.
for GTMK in launch-strategy product-marketing-context cold-email social-content ad-creative content-strategy market-research marketing-strategist higgsfield-soul-id higgsfield-generate diagram; do
    GTMK_SRC="$OMEGA_SRC/skills/$GTMK"
    GTMK_DST="$OMEGA_DIR/skills/$GTMK"
    if [[ -d "$GTMK_SRC" ]]; then
        mkdir -p "$GTMK_DST"
        cp -r "$GTMK_SRC"/* "$GTMK_DST/"
        find "$GTMK_DST" -name "*.sh" -exec chmod +x {} + 2>/dev/null || true
        GTMK_CMD="$HOME/.claude/commands"; mkdir -p "$GTMK_CMD"
        for cmd in "$GTMK" "omg-$GTMK"; do
            cat > "$GTMK_CMD/$cmd.md" <<EOF
# /$cmd

Run the $GTMK skill. Read and follow the complete instructions in:

\`$GTMK_DST/SKILL.md\`

Use every reference and template it provides.
EOF
        done
        ok "Marketing/Visual skill installed: $GTMK → ~/.omega/skills/$GTMK/ (/$GTMK + /omg-$GTMK)"
    else
        info "Marketing/Visual skill $GTMK not found — skipping"
    fi
done

# Install the llm-council skill — a 100% Claude Code-native multi-model Claude
# deliberation council (inspired by Karpathy's LLM Council), implemented on the
# OmegaOS Workflow primitive: four Claude models answer in parallel, peer-review
# each other anonymously, an Opus president synthesizes the verdict WITH the
# dissent. NO API KEYS — members run as in-process Workflow sub-agents through the
# existing Claude Code session. Three trigger stubs: /llm-council, /omg-llm-council,
# /council-llm-style entrypoint `/council` (multi-model second opinion).
LLMC_SRC="$OMEGA_SRC/skills/llm-council"
LLMC_DST="$OMEGA_DIR/skills/llm-council"
if [[ -d "$LLMC_SRC" ]]; then
    mkdir -p "$LLMC_DST"
    cp -r "$LLMC_SRC"/* "$LLMC_DST/"
    find "$LLMC_DST" -name "*.sh" -exec chmod +x {} + 2>/dev/null || true
    LLMC_CMD="$HOME/.claude/commands"; mkdir -p "$LLMC_CMD"
    for cmd in llm-council omg-llm-council council omg-council; do
        cat > "$LLMC_CMD/$cmd.md" <<EOF
# /$cmd

Convene the multi-model LLM Council. Read and follow the complete protocol in:

\`$LLMC_DST/SKILL.md\`

Run the council with the **Workflow tool** exactly as the skill prescribes: four Claude
models (Opus 4.8, Sonnet 4.6, Haiku 4.5, Fable 5) answer the SAME question independently
and in parallel, then peer-review each other ANONYMOUSLY (Response A/B/C, identities hidden),
and an Opus 4.8 president synthesizes the final verdict WITH the dissent. The members are
in-process Workflow \`agent(prompt, { model })\` sub-agents on the existing Claude Code
session — **no API keys, no external providers, no extra cost**. Use the embedded,
copy-pasteable Workflow JS in the SKILL.md as the implementation.
EOF
    done
    ok "Council skill installed: llm-council → ~/.omega/skills/llm-council/ (/llm-council + /omg-llm-council + /council + /omg-council)"
else
    info "Council skill llm-council not found — skipping"
fi

# Install the browser-use skill — agentic CLOUD browser automation via the
# official browser-use-sdk (Browser Use cloud API). Markdown + thin wrapper +
# run.py only: the Python venv (~/.omega/skills/browser-use/.venv) and the pip
# install are a RUNTIME OPT-IN created lazily on first run — install.sh NEVER
# pip-installs or creates the venv, and BROWSER_USE_API_KEY is read at runtime
# from ~/.omega/secrets/integrations.env, never the repo (R-BROWSER / R-SEC).
BUSE_SRC="$OMEGA_SRC/skills/browser-use"
BUSE_DST="$OMEGA_DIR/skills/browser-use"
if [[ -d "$BUSE_SRC" ]]; then
    mkdir -p "$BUSE_DST"
    cp -r "$BUSE_SRC"/* "$BUSE_DST/"
    # The wrapper is named `browser-use` (no .sh extension), so the generic *.sh
    # chmod patterns miss it — set the exec bit explicitly (cp -r drops it).
    chmod +x "$BUSE_DST/browser-use" 2>/dev/null || true
    find "$BUSE_DST" -name "*.sh" -exec chmod +x {} + 2>/dev/null || true
    BUSE_CMD="$HOME/.claude/commands"; mkdir -p "$BUSE_CMD"
    for cmd in browser-use omg-browser-use; do
        cat > "$BUSE_CMD/$cmd.md" <<EOF
# /$cmd

Run the browser-use skill — an agentic cloud browser task from natural language.
Read and follow the complete instructions in:

\`$BUSE_DST/SKILL.md\`

browser-use is for LLM-AGENTIC browsing of UIs we don't control (unknown steps);
for deterministic E2E of our own apps use Playwright/acceptance instead (R-BROWSER).
It needs the paid Browser Use cloud + BROWSER_USE_API_KEY in
~/.omega/secrets/integrations.env; the venv + pip install are a runtime opt-in.
EOF
    done
    ok "browser-use skill installed: browser-use → ~/.omega/skills/browser-use/ (/browser-use + /omg-browser-use)"
else
    info "browser-use skill browser-use not found — skipping"
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

# Install the autonomous browser-acceptance + self-heal gate (the build's terminal
# phase). Ships the SKILL protocol + the Playwright sweep harness (sweep.cjs) so a
# fresh install can prove "it WORKS" (every route 200, clean console/network, authed
# golden path persists) and auto-heal what it finds.
ACCEPT_SRC="$OMEGA_SRC/skills/acceptance"
ACCEPT_DST="$OMEGA_DIR/skills/acceptance"
if [[ -d "$ACCEPT_SRC" ]]; then
    mkdir -p "$ACCEPT_DST"
    cp -r "$ACCEPT_SRC"/* "$ACCEPT_DST/"
    cat > "$OMG_CMD_DST/omg-acceptance.md" <<EOF
# /omg-acceptance

Autonomous browser-acceptance + self-heal gate — the terminal phase of a build.
Playwright-sweeps every route (200 + render), captures every console error and
failed network request, walks the authenticated golden path with a real persisted
write, then AUTONOMOUSLY fixes what it finds and re-runs until green. Read and
follow the full protocol in:

\`$ACCEPT_DST/SKILL.md\`

Sweep harness: \`node $ACCEPT_DST/sweep.cjs\` (configured via BASE_URL / ROUTES /
PROTECTED_ROUTES / GOLDEN_PATH / CLERK_* / TEST_* env).
EOF
    ok "Acceptance skill installed → $ACCEPT_DST/ (+ /omg-acceptance stub + sweep.cjs)"
else
    info "Acceptance skill not found — skipping"
fi

# Install the OmegaOS pipeline skills (vision, prd, brand-identity) the new-project
# flow delegates to — shipped as /omg-* so a FRESH install is self-contained (the
# pipeline no longer depends on the user's personal /vision /prd /brand-identity
# existing). Does NOT touch any pre-existing ones the user may already have.
for psk in vision prd brand-identity; do
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
# Failure-tolerant (a broken binary must not abort the install) but VISIBLE:
# the old silent `|| true` could print "ok … (0 files)" with nothing exported.
"$INSTALL_DIR/omega" rules export 2>/dev/null \
    || warn "omega rules export failed — code-registered rules not exported (re-run later: omega rules export)"
RULES_COUNT=$(ls "$OMEGA_DIR/rules" 2>/dev/null | wc -l)
ok "Rules exported to $OMEGA_DIR/rules/ ($RULES_COUNT files)"

# Copy agent prompts to ~/.omega/agents/
cp -r "$OMEGA_SRC/agents/"* "$AGENTS_DIR/" 2>/dev/null || true
ok "Agent prompts installed"

# Sync rules + OMEGA.md into all LLM config directories. Failure-tolerant but
# visible (same constraint as rules export above — no silent empty sync).
info "Syncing to LLM config directories..."
if "$INSTALL_DIR/omega" sync 2>/dev/null; then
    ok "LLM configs synced (Claude, Gemini, Codex)"
else
    warn "omega sync failed — LLM configs NOT synced (re-run later: omega sync)"
fi

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
TOKREFRESH_CRON="*/30 * * * * $OMEGA_DIR/bin/omega-token-refresh.sh >> $OMEGA_DIR/logs/omega-token-refresh.log 2>&1   # OMEGA-CRON-TOKEN-REFRESH-v1"
DONENOTIFY_CRON="* * * * * $OMEGA_DIR/bin/omega-done-notify.sh >> $OMEGA_DIR/logs/omega-done-notify.log 2>&1   # OMEGA-CRON-DONE-NOTIFY-v1"
STUCK_CRON="* * * * * $OMEGA_DIR/bin/omega-stuck-oracle-alert.sh >> $OMEGA_DIR/logs/omega-stuck-alert.log 2>&1   # OMEGA-CRON-STUCK-ALERT-v1"
BRIEF_CRON="0 8 * * * $OMEGA_DIR/bin/omega-atlas-brief.sh >> $OMEGA_DIR/logs/omega-atlas-brief.log 2>&1   # OMEGA-CRON-ATLAS-BRIEF-v1"
SELFHEAL_CRON="0 */3 * * * $OMEGA_DIR/bin/omega-self-heal.sh >> $OMEGA_DIR/logs/omega-self-heal.log 2>&1   # OMEGA-CRON-SELFHEAL-v1"
# Telegram media inbox purge: images received via the bot land in state/tg-media
# for the dispatched oracle to Read; they are transient — purge anything older
# than 7 days, daily. Strictly scoped to that one directory (cannot touch projects).
TGMEDIA_CRON="30 4 * * * find $OMEGA_DIR/state/tg-media -type f -mtime +7 -delete 2>/dev/null   # OMEGA-CRON-TGMEDIA-PURGE-v1"
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
        ok "Token-budget usage alert scheduled (every 10 min → 80/85/90/95% Telegram alert)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-TOKEN-REFRESH-v1"; then
        ok "Claude token-refresh already scheduled"
    elif [[ -f "$OMEGA_DIR/bin/omega-token-refresh.sh" ]]; then
        ( crontab -l 2>/dev/null; echo "$TOKREFRESH_CRON" ) | crontab -
        ok "Claude token-refresh scheduled (every 30 min → pre-empts agent 401s)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-DONE-NOTIFY-v1"; then
        ok "End-of-mission notifier already scheduled"
    elif [[ -f "$OMEGA_DIR/bin/omega-done-notify.sh" ]]; then
        ( crontab -l 2>/dev/null; echo "$DONENOTIFY_CRON" ) | crontab -
        ok "End-of-mission notifier scheduled (every minute → oracle done.json → Telegram)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-STUCK-ALERT-v1"; then
        ok "Stuck-oracle alert already scheduled"
    elif [[ -f "$OMEGA_DIR/bin/omega-stuck-oracle-alert.sh" ]]; then
        ( crontab -l 2>/dev/null; echo "$STUCK_CRON" ) | crontab -
        ok "Stuck-oracle alert scheduled (every minute)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-ATLAS-BRIEF-v1"; then
        ok "Atlas briefing already scheduled"
    elif [[ -f "$OMEGA_DIR/bin/omega-atlas-brief.sh" ]]; then
        ( crontab -l 2>/dev/null; echo "$BRIEF_CRON" ) | crontab -
        ok "Atlas briefing scheduled (daily 08:00 UTC)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-SELFHEAL-v1"; then
        ok "Self-heal daemon already scheduled"
    elif [[ -f "$OMEGA_DIR/bin/omega-self-heal.sh" ]]; then
        ( crontab -l 2>/dev/null; echo "$SELFHEAL_CRON" ) | crontab -
        ok "Self-heal daemon scheduled (every 3h → omega doctor --fix + alert)"
    fi
    if crontab -l 2>/dev/null | grep -qF "# OMEGA-CRON-TGMEDIA-PURGE-v1"; then
        ok "Telegram media purge already scheduled"
    else
        ( crontab -l 2>/dev/null; echo "$TGMEDIA_CRON" ) | crontab -
        ok "Telegram media purge scheduled (daily 04:30 → state/tg-media files >7 days)"
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
        jq --arg track "$HOOKS_DST/track-tool-use.sh" --arg verify "$HOOKS_DST/stop-verify-hook.sh" --arg guard "$HOOKS_DST/omega-audit-guard.sh" '
          .hooks = (.hooks // {})
          | .hooks.PostToolUse = ((.hooks.PostToolUse // []) | map(select(((.hooks[0].command // "") | test("track-tool-use")) | not)) + [{"matcher":"*","hooks":[{"type":"command","command":$track}]}])
          | .hooks.Stop = ((.hooks.Stop // []) | map(select(((.hooks[0].command // "") | test("stop-verify")) | not)) + [{"hooks":[{"type":"command","command":$verify}]}])
          | .hooks.PreToolUse = ((.hooks.PreToolUse // []) | map(select(((.hooks[0].command // "") | test("omega-audit-guard")) | not)) + [{"matcher":"Bash","hooks":[{"type":"command","command":$guard}]}])
        ' "$CLAUDE_SETTINGS" > "$TMP" 2>/dev/null && mv "$TMP" "$CLAUDE_SETTINGS" && ok "Hooks installed + registered (PreToolUse guard + PostToolUse track + Stop verify)" || { rm -f "$TMP"; info "Hook merge skipped (jq error) — hooks copied to $HOOKS_DST"; }
    else
        info "jq not found — hooks copied to $HOOKS_DST; install jq to auto-register them in settings.json"
    fi
fi

# (b) Identity baseline (never overwrites an existing SOUL.md).
if [[ -f "$OMEGA_SRC/agents/identity/SOUL.template.md" && ! -f "$OMEGA_DIR/SOUL.md" ]]; then
    cp "$OMEGA_SRC/agents/identity/SOUL.template.md" "$OMEGA_DIR/SOUL.md"
    ok "Identity template → $OMEGA_DIR/SOUL.md (customize it; keep private data in MEMORY.md)"
fi


# (c) Telegram command bot — already installed right after Phase 4 (survives a
# Phase 5 abort). Idempotent backstop: re-run (picks up bun if Phase 5 installed it).
install_command_bot || true
install_inbox_bot || true

# (d) Claude Code agent binary — omega needs it to spawn agents.
if ! command -v claude >/dev/null 2>&1; then
    info "Claude Code CLI absent — omega needs it to spawn agents. Attempting install..."
    omega_timeout 180 "$INSTALL_DIR/omega" install claude 2>/dev/null || info "Run 'omega install claude' (or install Claude Code manually), then authenticate with 'claude'."
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
        omega_sudo sh -c 'apt-get install -y xvfb >/dev/null 2>&1' && ok "Xvfb installed (headless PDF/Playwright)" || info "For headless PDF/browser: 'sudo apt-get install xvfb'"
    fi
    if command -v npm >/dev/null 2>&1; then
        command -v playwright >/dev/null 2>&1 || {
            info "Installing Playwright CLI..."
            npm install -g playwright >/dev/null 2>&1 && ok "Playwright CLI installed" || info "Playwright CLI install failed"
        }
        if command -v playwright >/dev/null 2>&1; then
            playwright install chromium >/dev/null 2>&1 && ok "Chromium installed (Playwright + CDP ready)" || info "Chromium download failed — run 'playwright install chromium'"
            if command -v apt-get >/dev/null 2>&1; then
                omega_sudo env "PATH=$PATH" sh -c 'playwright install-deps chromium >/dev/null 2>&1' && ok "Chromium system deps installed" || info "For Chromium libs: 'sudo playwright install-deps chromium'"
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
step "Phase 6.9: Companion tools"
if [[ "${OMEGA_WITH_COMPANION:-0}" == "1" && "${OMEGA_SKIP_COMPANION:-0}" != "1" && -f "$OMEGA_SRC/scripts/install-companion-tools.sh" ]]; then
    # shellcheck source=/dev/null
    source "$OMEGA_SRC/scripts/install-companion-tools.sh" || info "companion tools step had warnings (non-fatal)"
else
    info "Companion skill packs deferred (planning-with-files, claude-mem, superpowers, …). Add: OMEGA_WITH_COMPANION=1 ./install.sh"
fi

# ─── Phase 6.92: Nova — personal-assistant layer (opt-in) ────────────────────
# Nova is the operator's PERSONAL companion: the fallback persona
# (agents/companion.md — shipped with the other agent prompts above), proactive
# cron touchpoints (nova-report morning/evening/nudge → nova-send), an
# always-alive terminal-session watchdog (nova-godmode), and a Composio app
# connector (nova-composio-connect + the nova-apps.json catalogue).
# OPT-IN (OMEGA_WITH_NOVA=1), like the browser stack: Nova needs her OWN
# Telegram bot token + per-operator secrets (~/.omega/nova-secrets.env,
# NOVA_CHAT_ID at minimum), so a default install must not schedule crons that
# can only fail. Secrets and the live catalogue stay local-only (gitignored).
step "Phase 6.92: Nova personal-assistant layer"
if [[ "${OMEGA_WITH_NOVA:-0}" == "1" ]]; then
    for nsk in nova-report.sh nova-send.sh nova-godmode.sh nova-composio-connect.sh \
               nova-call-sync.py nova-call-kb.py nova-self-improve.sh nova-notify.sh; do
        if [[ -f "$OMEGA_SRC/scripts/$nsk" ]]; then
            cp -f "$OMEGA_SRC/scripts/$nsk" "$OMEGA_DIR/bin/$nsk"
            chmod +x "$OMEGA_DIR/bin/$nsk"
        fi
    done
    # Seed Nova's self-improvement anatomy into the life store ONCE — these are
    # HER files afterwards (she edits them), never clobbered by a re-install.
    # shellcheck source=/dev/null
    [[ -f "$OMEGA_DIR/nova-secrets.env" ]] && source "$OMEGA_DIR/nova-secrets.env" 2>/dev/null || true
    NOVA_LIFE_DIR="${NOVA_HOME:-$HOME/Station/LifeStyle}"
    mkdir -p "$NOVA_LIFE_DIR" "$OMEGA_DIR/backups/nova"
    if [[ -f "$OMEGA_SRC/config/nova-anatomy.template.md" && ! -f "$NOVA_LIFE_DIR/ANATOMY.md" ]]; then
        cp "$OMEGA_SRC/config/nova-anatomy.template.md" "$NOVA_LIFE_DIR/ANATOMY.md"
    fi
    if [[ ! -f "$NOVA_LIFE_DIR/SELF-IMPROVEMENT.md" ]]; then
        cat > "$NOVA_LIFE_DIR/SELF-IMPROVEMENT.md" <<'EOF'
# SELF-IMPROVEMENT — journal de mes modifications de moi-même

> Tenu par Nova. Une entrée par modification : date, composant, quoi, pourquoi
> (le problème OBSERVÉ), comment vérifier. Les backups vivent dans
> `~/.omega/backups/nova/`.
EOF
    fi
    # Seed the app catalogue + the secrets template ONCE — never clobber the
    # operator's live copies (they hold per-operator connection state / keys).
    if [[ -f "$OMEGA_SRC/config/nova-apps.sample.json" && ! -f "$OMEGA_DIR/nova-apps.json" ]]; then
        cp "$OMEGA_SRC/config/nova-apps.sample.json" "$OMEGA_DIR/nova-apps.json"
        chmod 600 "$OMEGA_DIR/nova-apps.json"
    fi
    if [[ ! -f "$OMEGA_DIR/nova-secrets.env" ]]; then
        cat > "$OMEGA_DIR/nova-secrets.env" <<'EOF'
# Nova secrets — local-only, NEVER committed. Read by the nova-*.sh scripts.
# NOVA_CHAT_ID=<your numeric Telegram user id>     (required: reports + send)
# NOVA_BOT_TOKEN=<token of Nova's OWN bot from @BotFather>  (required: reports + send
#                — Nova needs her own bot, separate from the Atlas command bot;
#                alternative: put the token alone in $NOVA_HOME/.bot, mode 600)
# NOVA_OPERATOR_NAME=<how Nova calls you>           (optional)
# NOVA_HOME=<life store dir>                        (default ~/Station/LifeStyle)
# NOVA_SELF=<Nova's own territory>                  (default ~/Station/Nova — enables the daily studio)
# NOVA_SOUL_ID=<your trained Higgsfield Soul id(s)> (optional: studio visuals)
# NOVA_VISUAL_STYLE=<her visual style, free text>   (optional: studio visuals)
# COMPOSIO_API_KEY=                                 (app connections via Composio)
EOF
        chmod 600 "$OMEGA_DIR/nova-secrets.env"
    fi
    # Cron touchpoints (TZ of the box). Idempotency greps the script+mode
    # substring rather than only the OMEGA-CRON-NOVA-* marker, so a pre-marker
    # hand-installed Nova line is never duplicated into a double-firing pair.
    NOVA_MORNING_CRON="0 7 * * * $OMEGA_DIR/bin/nova-report.sh morning >> $OMEGA_DIR/logs/nova-report.log 2>&1   # OMEGA-CRON-NOVA-MORNING-v1"
    NOVA_EVENING_CRON="0 21 * * * $OMEGA_DIR/bin/nova-report.sh evening >> $OMEGA_DIR/logs/nova-report.log 2>&1   # OMEGA-CRON-NOVA-EVENING-v1"
    NOVA_NUDGE_CRON="0 11,15,18 * * * $OMEGA_DIR/bin/nova-report.sh nudge >> $OMEGA_DIR/logs/nova-report.log 2>&1   # OMEGA-CRON-NOVA-NUDGE-v1"
    NOVA_GODMODE_CRON="* * * * * $OMEGA_DIR/bin/nova-godmode.sh >> $OMEGA_DIR/logs/nova-godmode.log 2>&1   # OMEGA-CRON-NOVA-GODMODE-v1"
    # Voice-call loop + weekly self-improvement. All three scripts no-op
    # silently until their per-operator config exists (NOVA_CHAT_ID, the call
    # agent in state/nova-call.json, ELEVENLABS_API_KEY) — safe to schedule on
    # a fresh box.
    NOVA_CALLSYNC_CRON="* * * * * /usr/bin/python3 $OMEGA_DIR/bin/nova-call-sync.py # OMEGA-CRON-NOVA-CALL-SYNC-v1"
    NOVA_CALLKB_CRON="0 6 * * * /usr/bin/python3 $OMEGA_DIR/bin/nova-call-kb.py # OMEGA-CRON-NOVA-CALL-KB-v1"
    NOVA_SELFIMPROVE_CRON="0 19 * * 0 /bin/bash $OMEGA_DIR/bin/nova-self-improve.sh # OMEGA-CRON-NOVA-SELF-IMPROVE-v1"
    NOVA_STUDIO_CRON="0 12 * * * $OMEGA_DIR/bin/nova-report.sh studio >> $OMEGA_DIR/logs/nova-report.log 2>&1   # OMEGA-CRON-NOVA-STUDIO-v1"
    if command -v crontab >/dev/null 2>&1; then
        crontab -l 2>/dev/null | grep -qF "nova-report.sh morning" || { ( crontab -l 2>/dev/null; echo "$NOVA_MORNING_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-report.sh evening" || { ( crontab -l 2>/dev/null; echo "$NOVA_EVENING_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-report.sh nudge"   || { ( crontab -l 2>/dev/null; echo "$NOVA_NUDGE_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-godmode.sh"        || { ( crontab -l 2>/dev/null; echo "$NOVA_GODMODE_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-call-sync.py"      || { ( crontab -l 2>/dev/null; echo "$NOVA_CALLSYNC_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-call-kb.py"        || { ( crontab -l 2>/dev/null; echo "$NOVA_CALLKB_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-self-improve.sh"   || { ( crontab -l 2>/dev/null; echo "$NOVA_SELFIMPROVE_CRON" ) | crontab -; }
        crontab -l 2>/dev/null | grep -qF "nova-report.sh studio"  || { ( crontab -l 2>/dev/null; echo "$NOVA_STUDIO_CRON" ) | crontab -; }
        ok "Nova layer installed (8 scripts + crons: reports + godmode + call-sync + call-KB + weekly self-improve + daily studio + notify). Configure NOVA_CHAT_ID + NOVA_BOT_TOKEN (her OWN @BotFather bot): \$EDITOR $OMEGA_DIR/nova-secrets.env"
    else
        ok "Nova layer installed (7 scripts; crontab unavailable — schedule nova-report.sh/nova-godmode.sh/nova-call-sync.py in your scheduler)"
    fi
else
    info "Nova personal-assistant layer deferred (needs its own bot token + NOVA_CHAT_ID). Add: OMEGA_WITH_NOVA=1 ./install.sh"
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
# Agentik-Skills — the canonical SSOT skills library (agentik-os/Agentik-Skills).
# Skills here mirror into ~/.omega/skills/<name>/ and `omega sync` symlinks them
# into every LLM (Claude ~/.claude/skills, Gemini/Codex via OMEGA.md). Private repo
# → needs gh/git auth; best-effort, like the dashboard.
step "Phase 6.95: Telegram interface (OmegaMC)"
SKILLS_REPO_DIR="$OMEGA_DIR/repos/Agentik-Skills"
mkdir -p "$OMEGA_DIR/repos"
if [[ -d "$SKILLS_REPO_DIR/.git" ]]; then
    ok "Agentik-Skills present ($SKILLS_REPO_DIR — update: git -C $SKILLS_REPO_DIR pull)"
elif command -v gh >/dev/null 2>&1 && omega_timeout 120 gh repo clone agentik-os/Agentik-Skills "$SKILLS_REPO_DIR" >/dev/null 2>&1; then
    ok "Agentik-Skills cloned → $SKILLS_REPO_DIR (canonical skills source)"
elif omega_timeout 120 git clone --depth 1 https://github.com/agentik-os/Agentik-Skills.git "$SKILLS_REPO_DIR" >/dev/null 2>&1; then
    ok "Agentik-Skills cloned → $SKILLS_REPO_DIR"
else
    rm -rf "$SKILLS_REPO_DIR" 2>/dev/null || true
    # Honest about the gap: this is a PRIVATE repo, so without auth a fresh box
    # lacks every skill family that lives only there — say which, and how to fix.
    info "Agentik-Skills unavailable (private repo, no gh/git auth) — the Motion/animation skill families (hyperframes, gsap, three, animejs, lottie, waapi, css-animations, typegpu, remotion) + art-director design packs will be MISSING. Get access from the agentik-os maintainer, then: gh auth login && gh repo clone agentik-os/Agentik-Skills $SKILLS_REPO_DIR && ./install.sh"
fi

# Mirror the cloned library into the live skill store (Agentik-Skills mirror
# step). The repo is the canonical SSOT skills library; without this mirror its
# skills existed only where someone hand-copied them, so a fresh install
# silently lacked 16+ live skills. Every dir holding a SKILL.md — top level
# (<skill>/) AND categorized (<Category>/<skill>/) — lands in
# ~/.omega/skills/<skill>/. Skills the OmegaOS repo itself ships are SKIPPED:
# the Phase 5 vendored copies are canonical here, the mirror only fills gaps.
if [[ -d "$SKILLS_REPO_DIR/.git" ]]; then
    SKMIRROR=0
    while IFS= read -r sk_md; do
        sk_dir="$(dirname "$sk_md")"
        sk_name="$(basename "$sk_dir")"
        [[ -d "$OMEGA_SRC/skills/$sk_name" ]] && continue   # OmegaOS-vendored = canon
        mkdir -p "$OMEGA_DIR/skills/$sk_name"
        if command -v rsync >/dev/null 2>&1; then
            rsync -a "$sk_dir/" "$OMEGA_DIR/skills/$sk_name/" 2>/dev/null || true
        else
            cp -r "$sk_dir/." "$OMEGA_DIR/skills/$sk_name/" 2>/dev/null || true
        fi
        SKMIRROR=$((SKMIRROR + 1))
    done < <(find "$SKILLS_REPO_DIR" -mindepth 2 -maxdepth 3 -name SKILL.md -not -path '*/.git/*' 2>/dev/null)
    ok "Agentik-Skills mirrored → $OMEGA_DIR/skills/ ($SKMIRROR skills from the SSOT library)"
fi

if [[ "${OMEGA_SKIP_DASHBOARD:-0}" != "1" ]]; then
    MC_DIR="$OMEGA_DIR/repos/omega-mc"
    mkdir -p "$OMEGA_DIR/repos"
    if [[ -d "$MC_DIR/.git" ]]; then
        ok "OmegaMC present ($MC_DIR — update: git -C $MC_DIR pull && omega-mc-up --rebuild)"
    elif omega_timeout 120 git clone --depth 1 https://github.com/agentik-os/agentik-telegram.git "$MC_DIR" >/dev/null 2>&1; then
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
        info "OmegaMC (optional multi-agent backend) installed → $MC_DIR. It needs its OWN bot token (don't share the command bot's). Bring up the dashboard (no token needed — the command bot owns Telegram): omega-mc-up  (needs Docker)"
    fi
fi

# ─── Done ─────────────────────────────────────────────────────────────────────

# Persist the onboarding guide where the user can ALWAYS find it again:
# the npx wrapper hides install output behind its animation, so the epilogue
# below may never be seen — `omega guide` + ~/.omega/GETTING-STARTED.md are
# the durable copies (single source: docs/GETTING-STARTED.md in the repo).
if [[ -f "$OMEGA_SRC/docs/GETTING-STARTED.md" ]]; then
    cp -f "$OMEGA_SRC/docs/GETTING-STARTED.md" "$OMEGA_DIR/GETTING-STARTED.md"
fi
# Same parity for the full operator manual (vocabulary, cockpits, skill
# catalog, LLM-agent contract): GUIDE.md is the document we tell people to
# read next — a fresh box must have it locally, not just on GitHub.
if [[ -f "$OMEGA_SRC/GUIDE.md" ]]; then
    cp -f "$OMEGA_SRC/GUIDE.md" "$OMEGA_DIR/GUIDE.md"
fi

echo ""
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}  OmegaOS v${OMEGA_VERSION} installed successfully!${NC}"
echo -e "${GREEN}${BOLD}═══════════════════════════════════════════${NC}"
echo ""
# Surface the headless-Mac launchd failure HERE too: the npx animation swallows
# mid-install lines, but the final summary is always shown.
if [[ -n "${TG_BOT_WARN:-}" ]]; then
    warn "$TG_BOT_WARN"
    echo ""
fi
# A daemon that was already running predates the binary we just installed —
# its in-memory code keeps any old bug (e.g. the pre-v0.1.8 rename-session
# runtime loss) alive until it is restarted. Sessions survive nothing across
# a daemon restart, so this is a deliberate operator action, never automatic.
if pgrep -u "$(id -u)" -f "rmux --__internal-daemon" >/dev/null 2>&1; then
    warn "An rmux daemon from a PREVIOUS install is still running — the updated rmux only takes effect after you restart it. When your current sessions are done: rmux kill-server   (this closes all sessions), then just run 'omega' again."
    echo ""
fi
echo -e "  ${BOLD}Your 5-minute setup — in order:${NC}"
echo ""
echo -e "  ${BOLD}0.${NC} Reload your shell:        source $RC_FILE"
echo -e "  ${BOLD}1.${NC} Connect Claude (required): claude   →  then type /login and follow the URL"
echo -e "  ${BOLD}2.${NC} Telegram remote (recommended):"
echo "       @BotFather → /newbot → copy the token; @userinfobot → your numeric id"
echo "       OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <YOUR_ID> --user-id <YOUR_ID>"
echo "       (the env prefix keeps your token out of the process list)"
echo "       Project topics: group + Topics ON + bot admin → /setupgroup → /sync"
echo -e "  ${BOLD}3.${NC} Service keys (optional, for auto-provisioning new apps):"
echo "       \$EDITOR $OMEGA_DIR/provisioning/services.env   (Vercel/GitHub/Convex/Stripe/OpenAI)"
echo -e "  ${BOLD}4.${NC} Add a project:  omega → Projects → [N] New Project   (or drop a repo in ~/Station/...)"
echo -e "  ${BOLD}5.${NC} Verify:         omega doctor    — every line should be [+]"
echo ""
echo -e "  ${BOLD}Then:${NC}  omega                                  # the TUI"
echo "        omega dispatch <Project> \"<mission>\"  # your first mission"
echo ""
echo -e "  📖 ${BOLD}Full guide (every step explained):${NC}  omega guide"
echo "     (also saved at $OMEGA_DIR/GETTING-STARTED.md)"
echo ""
