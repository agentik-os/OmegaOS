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

# Bootstrap the OS packages a BARE VPS lacks. Without this, the rustup curl
# below dies with `command not found` on a fresh Ubuntu/Debian image, and even
# with Rust the rmux build fails for want of a C toolchain (`cc`) + pkg-config.
# Idempotent: only installs what is actually missing.
bootstrap_os_packages() {
    local need=()
    command -v git >/dev/null 2>&1 || need+=("git")
    command -v curl >/dev/null 2>&1 || need+=("curl")
    { command -v cc >/dev/null 2>&1 || command -v gcc >/dev/null 2>&1; } || need+=("C-toolchain")
    command -v pkg-config >/dev/null 2>&1 || need+=("pkg-config")
    if [[ ${#need[@]} -eq 0 ]]; then
        ok "Build prerequisites present (git, curl, cc, pkg-config)"
        return 0
    fi
    info "Installing build prerequisites: ${need[*]}"
    local SUDO=""
    [[ "$(id -u)" -ne 0 ]] && command -v sudo >/dev/null 2>&1 && SUDO="sudo"
    if command -v apt-get >/dev/null 2>&1; then
        $SUDO apt-get update -qq && $SUDO apt-get install -y curl git ca-certificates build-essential pkg-config
    elif command -v dnf >/dev/null 2>&1; then
        $SUDO dnf install -y curl git ca-certificates gcc gcc-c++ make pkgconf-pkg-config
    elif command -v yum >/dev/null 2>&1; then
        $SUDO yum install -y curl git ca-certificates gcc gcc-c++ make pkgconfig
    elif command -v pacman >/dev/null 2>&1; then
        $SUDO pacman -Sy --noconfirm curl git ca-certificates base-devel pkgconf
    elif command -v apk >/dev/null 2>&1; then
        $SUDO apk add --no-cache curl git ca-certificates build-base pkgconf
    else
        err "No supported package manager (apt/dnf/yum/pacman/apk)."
        err "Install these manually, then re-run ./install.sh: ${need[*]}"
        exit 1
    fi
    ok "Build prerequisites installed"
}
bootstrap_os_packages

# Check for Rust
if ! command -v cargo &>/dev/null; then
    info "Rust not found. Installing via rustup..."
    command -v curl >/dev/null 2>&1 || { err "curl is required to bootstrap Rust but is missing; install curl and re-run."; exit 1; }
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
# --locked: build against the committed Cargo.lock so a fresh clone resolves the
# exact same transitive deps (reproducible builds). Falls back to an unlocked
# build only if the lockfile is somehow absent/out of sync.
cargo build --release --locked 2>&1 | tail -3 || cargo build --release 2>&1 | tail -3
mkdir -p "$INSTALL_DIR"
cp target/release/omega "$INSTALL_DIR/omega"
ln -sf "$INSTALL_DIR/omega" "$INSTALL_DIR/omg"   # short alias: omg == omega
ok "omega CLI installed to $INSTALL_DIR/omega"

# ─── Phase 5: Configuration ──────────────────────────────────────────────────

step "Phase 5: Configuring OmegaOS"

mkdir -p "$OMEGA_DIR"/{state,logs,locks}
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
    rsync -a --exclude='node_modules' --exclude='.next' --exclude='output' "$PDFGEN_SRC/" "$PDFGEN_DST/"
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
else
    info "Audit skills not found — skipping"
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
PATROL_CRON="* * * * * $INSTALL_DIR/omega patrol --once >> $OMEGA_DIR/logs/omega-patrol.log 2>&1   # OMEGA self-improvement patrol + curator"
USAGE_CRON="*/10 * * * * $INSTALL_DIR/omega usage --check >> $OMEGA_DIR/logs/omega-usage.log 2>&1   # OMEGA token-budget 80/90% alert"
if command -v crontab >/dev/null 2>&1; then
    if crontab -l 2>/dev/null | grep -q "omega patrol"; then
        ok "Self-improvement patrol already scheduled"
    else
        ( crontab -l 2>/dev/null; echo "$PATROL_CRON" ) | crontab -
        ok "Self-improvement patrol scheduled (every minute → curator auto-trigger)"
    fi
    if crontab -l 2>/dev/null | grep -q "omega usage"; then
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

# (c) Persistent Telegram bridge — systemd --user service (replaces legacy aisb-bot.service).
if command -v systemctl >/dev/null 2>&1 && [[ -f "$OMEGA_SRC/config/systemd/omega-telegram.service" ]]; then
    SD_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SD_DIR"
    cp -f "$OMEGA_SRC/config/systemd/omega-telegram.service" "$SD_DIR/omega-telegram.service"
    loginctl enable-linger "$USER" 2>/dev/null || true
    systemctl --user daemon-reload 2>/dev/null || true
    if [[ -f "$OMEGA_DIR/telegram.toml" ]]; then
        # Token present → enable + start. (`omega telegram setup` also does this.)
        systemctl --user enable --now omega-telegram.service 2>/dev/null || true
        ok "Telegram bridge service enabled + started (persistent, Restart=always)"
    else
        # Do NOT enable yet: with no token the bridge would crash-loop every 10s.
        # `omega telegram setup` enables + starts it once the token is written.
        ok "Telegram bridge unit installed — run 'omega telegram setup <TOKEN> <CHAT_ID>' (it enables + starts the service)"
    fi
else
    info "systemd --user unavailable — run 'omega telegram run' under rmux, or use your own supervisor"
fi

# (d) Claude Code agent binary — omega needs it to spawn agents.
if ! command -v claude >/dev/null 2>&1; then
    info "Claude Code CLI absent — omega needs it to spawn agents. Attempting install..."
    "$INSTALL_DIR/omega" install claude 2>/dev/null || info "Run 'omega install claude' (or install Claude Code manually), then authenticate with 'claude'."
fi

# (e) Headless rendering for PDF generation / Playwright audits (best-effort).
if ! command -v Xvfb >/dev/null 2>&1 && command -v apt-get >/dev/null 2>&1; then
    sudo apt-get install -y xvfb >/dev/null 2>&1 && ok "Xvfb installed (headless PDF/Playwright)" || info "For headless PDF/browser: 'sudo apt-get install xvfb'"
fi

# (f) Playwright + Chromium — the browser engine the Quality Arsenal audits
# (uiux/flow/a11y/perf, browser-tester) and any CDP/DevTools automation drive
# via the Playwright CLI from Bash. Chromium ships the DevTools Protocol, so
# this single install covers BOTH Playwright AND CDP/DevTools — there is no
# separate package. Best-effort and opt-out (OMEGA_SKIP_BROWSER=1) because the
# Chromium download is ~150MB; never fatal to the install.
if [[ "${OMEGA_SKIP_BROWSER:-0}" != "1" ]]; then
    if command -v npm >/dev/null 2>&1; then
        if command -v playwright >/dev/null 2>&1; then
            ok "Playwright CLI already present ($(playwright --version 2>/dev/null || echo installed))"
        else
            info "Installing Playwright CLI (browser automation for the audits)..."
            npm install -g playwright >/dev/null 2>&1 \
                && ok "Playwright CLI installed" \
                || info "Playwright CLI install failed — run 'npm install -g playwright' manually"
        fi
        if command -v playwright >/dev/null 2>&1; then
            # Idempotent: a no-op when the Chromium build is already cached.
            playwright install chromium >/dev/null 2>&1 \
                && ok "Chromium installed (Playwright + CDP/DevTools ready)" \
                || info "Chromium download failed — run 'playwright install chromium' manually"
            # System libraries Chromium needs (apt + sudo; preserve PATH so the
            # root shell finds the global playwright bin). Non-fatal.
            if command -v apt-get >/dev/null 2>&1; then
                sudo env "PATH=$PATH" playwright install-deps chromium >/dev/null 2>&1 \
                    && ok "Chromium system deps installed" \
                    || info "For Chromium libs: 'sudo playwright install-deps chromium'"
            fi
        fi
    else
        info "npm not found — skipping Playwright (install Node.js, then 'npm i -g playwright && playwright install chromium')"
    fi
fi

# ─── Phase 6.9: Companion tools + skills (SST multi-LLM, best-effort, opt-out) ──
# planning-with-files, higgsfield (CLI+skills), claude-mem, superpowers,
# mempalace, remotion, + the best-practice reference. Sourced so its OK/WARN
# lines fold into this install run; never fatal. Skip: OMEGA_SKIP_COMPANION=1.
if [[ -f "$OMEGA_SRC/scripts/install-companion-tools.sh" ]]; then
    # shellcheck source=/dev/null
    source "$OMEGA_SRC/scripts/install-companion-tools.sh" || info "companion tools step had warnings (non-fatal)"
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
echo "  Config files: $OMEGA_DIR/config.toml + $OMEGA_DIR/providers.toml"
echo ""
echo "  Restart your shell or run:  source $RC_FILE"
echo ""
