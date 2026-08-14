#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — credential topology contract (install.sh + the post-login heal)
# ───────────────────────────────────────────────────────────────────────────
# Proves, mechanically, the filesystem fact install.sh used to get wrong:
# an ATOMIC write (write .tmp, then rename) onto a symlink does NOT follow the
# link, it REPLACES it. A symlink planted before the first login is therefore
# destroyed BY that login, and the canonical store it pointed at is never fed.
# Documented in docs/INSTALL-AND-CREDENTIALS.md ("GOTCHA"), and install.sh
# claimed the opposite ("the LLM will write through it on first login").
#
# What is covered:
#   1. rename(2) over a dangling symlink destroys it and never creates the target
#   2. install.sh's REAL migrate_creds() plants no dangling link on a fresh box
#   3. install.sh's REAL migrate_creds() still migrates + links an existing login
#   4. first login then heal: the canonical store is fed and the link restored
#   5. re-running the installer over a healed box changes nothing
#   6. the false write-through claim is gone from install.sh
#
# Checks 2, 3 and 5 execute the ACTUAL shipped migrate_creds() (extracted from
# install.sh), so reverting the fix fails this test. Check 4's heal step is a
# bash transcription of the CONTRACT implemented in Rust at
# crates/omega-core/src/credentials.rs (ensure_legacy_symlink) — it proves the
# topology those rules produce, NOT the Rust code itself.
#
# Runs in ~1s, entirely inside one mktemp -d, and never reads or writes the
# real HOME. Exits non-zero on any regression.
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_SH="$ROOT/install.sh"

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

# The real HOME is never a legal target. Everything below lives in $SANDBOX,
# and $HOME is never set, read, or written by this test.
case "$SANDBOX" in
    "" | "$HOME" | "$HOME"/*)
        echo "FATAL: refusing to run — sandbox '$SANDBOX' is not isolated from HOME" >&2
        exit 2
        ;;
esac

FAILURES=0
pass() { printf '  [+] %s\n' "$1"; }
fail() { printf '  [!] %s\n' "$1" >&2; FAILURES=$((FAILURES + 1)); }

TOKEN='{"claudeAiOauth":{"refreshToken":"rt-test-token"}}'

# ── helpers ────────────────────────────────────────────────────────────────

# How Claude Code and Gemini actually persist credentials: a temp file next to
# the target, then rename(2) over it. This is the whole point of the test.
atomic_write() {
    local target="$1" content="$2"
    printf '%s' "$content" > "$target.tmp"
    mv "$target.tmp" "$target"
}

# The contract of CredentialStore::ensure_legacy_symlink for a non-codex
# provider (crates/omega-core/src/credentials.rs): a real file at the legacy
# path with no canonical yet is MOVED into the canonical store, then the legacy
# path is re-created as a symlink to it — and the link is only ever created
# once the canonical exists.
heal_contract_from_credentials_rs() {
    local legacy="$1" canonical="$2"
    if [ -L "$legacy" ]; then
        [ "$(readlink "$legacy")" = "$canonical" ] && return 0
        rm -f "$legacy"
    elif [ -f "$legacy" ]; then
        mkdir -p "$(dirname "$canonical")"
        [ -e "$canonical" ] || mv "$legacy" "$canonical"
        rm -f "$legacy"
    fi
    [ -f "$canonical" ] && ln -s "$canonical" "$legacy"
    return 0
}

# Run the REAL migrate_creds() out of install.sh, against sandbox paths only.
# ok/info/warn are the installer's own output helpers; stub them so the
# function body runs unmodified.
ok()   { :; }
info() { :; }
warn() { :; }
MIGRATE_CREDS_SRC="$(awk '/^migrate_creds\(\) \{$/,/^\}$/' "$INSTALL_SH")"
if [ -z "$MIGRATE_CREDS_SRC" ]; then
    echo "FATAL: could not extract migrate_creds() from $INSTALL_SH" >&2
    exit 2
fi
eval "$MIGRATE_CREDS_SRC"
if ! declare -F migrate_creds >/dev/null; then
    echo "FATAL: migrate_creds() did not define after extraction" >&2
    exit 2
fi

# A fresh, never-installed box: its own HOME-shaped tree and its own OMEGA_DIR.
new_box() {
    local box="$SANDBOX/$1"
    rm -rf "$box"
    mkdir -p "$box/home/.claude" "$box/omega"
    printf '%s' "$box"
}

echo "Credential topology contract ($SANDBOX)"

# ── 1. the filesystem fact the installer comment got wrong ─────────────────
echo "1. atomic write over a dangling symlink"
BOX="$(new_box box1)"
LEGACY="$BOX/home/.claude/.credentials.json"
CANONICAL="$BOX/omega/credentials/claude.json"
ln -s "$CANONICAL" "$LEGACY"
if [ -L "$LEGACY" ] && [ ! -e "$LEGACY" ]; then
    pass "precondition: legacy is a symlink to a target that does not exist"
else
    fail "precondition: expected a dangling symlink at $LEGACY"
fi
atomic_write "$LEGACY" "$TOKEN"
if [ -L "$LEGACY" ]; then
    fail "rename(2) followed the symlink — the whole premise of this test is wrong"
else
    pass "the symlink is GONE after the atomic write"
fi
if [ -f "$LEGACY" ] && [ "$(cat "$LEGACY")" = "$TOKEN" ]; then
    pass "legacy path is now a REAL file holding the credentials"
else
    fail "expected a real credential file at $LEGACY"
fi
if [ -e "$CANONICAL" ]; then
    fail "canonical store was fed by the login — it must NOT be ($CANONICAL)"
else
    pass "canonical store was NEVER created — the write-through claim was false"
fi

# ── 2. installer on a never-logged-in box ──────────────────────────────────
echo "2. install.sh migrate_creds() on a fresh box (no login yet)"
BOX="$(new_box box2)"
LEGACY="$BOX/home/.claude/.credentials.json"
CANONICAL="$BOX/omega/credentials/claude.json"
export OMEGA_DIR="$BOX/omega"
migrate_creds "claude" "$LEGACY"
if [ -L "$LEGACY" ]; then
    fail "installer planted a symlink to a missing target — the first login will destroy it"
else
    pass "no symlink planted while the canonical store is empty"
fi
# -e alone would pass on a DANGLING link (it follows the link), so test -L too.
if [ -e "$LEGACY" ] || [ -L "$LEGACY" ]; then
    fail "installer created something at $LEGACY on a never-logged-in box"
else
    pass "legacy path left absent, which is the honest state"
fi

# ── 3. installer on a box that logged in BEFORE OmegaOS ────────────────────
echo "3. install.sh migrate_creds() adopts a pre-existing login"
BOX="$(new_box box3)"
LEGACY="$BOX/home/.claude/.credentials.json"
CANONICAL="$BOX/omega/credentials/claude.json"
export OMEGA_DIR="$BOX/omega"
mkdir -p "$OMEGA_DIR/credentials"
atomic_write "$LEGACY" "$TOKEN"
migrate_creds "claude" "$LEGACY"
if [ -f "$CANONICAL" ] && [ "$(cat "$CANONICAL")" = "$TOKEN" ]; then
    pass "existing credentials migrated into the canonical store"
else
    fail "expected the pre-existing login at $CANONICAL"
fi
if [ -L "$LEGACY" ] && [ "$(readlink "$LEGACY")" = "$CANONICAL" ]; then
    pass "legacy path re-created as a symlink to the canonical store"
else
    fail "expected $LEGACY to be a symlink to $CANONICAL"
fi

# ── 4. the real sequence: install, then first login, then heal ─────────────
echo "4. install then first login then heal"
BOX="$(new_box box4)"
LEGACY="$BOX/home/.claude/.credentials.json"
CANONICAL="$BOX/omega/credentials/claude.json"
export OMEGA_DIR="$BOX/omega"
migrate_creds "claude" "$LEGACY"
atomic_write "$LEGACY" "$TOKEN"          # the user runs `claude` and logs in
if [ -e "$CANONICAL" ]; then
    fail "canonical store fed by the login itself — impossible, check the test"
else
    pass "after the login the canonical store is still empty (the open window)"
fi
heal_contract_from_credentials_rs "$LEGACY" "$CANONICAL"
if [ -f "$CANONICAL" ] && [ "$(cat "$CANONICAL")" = "$TOKEN" ]; then
    pass "heal moved the fresh credentials into the canonical store"
else
    fail "heal did not feed $CANONICAL"
fi
if [ -L "$LEGACY" ] && [ "$(readlink "$LEGACY")" = "$CANONICAL" ]; then
    pass "heal restored the legacy path as a symlink"
else
    fail "expected $LEGACY to be a symlink to $CANONICAL after the heal"
fi
if [ "$(cat "$LEGACY")" = "$TOKEN" ]; then
    pass "reading through the legacy path resolves to the canonical credentials"
else
    fail "reading $LEGACY did not resolve to the canonical credentials"
fi

# ── 5. re-running the installer over a healed box ──────────────────────────
echo "5. re-install over a healed topology is a no-op"
migrate_creds "claude" "$LEGACY"
if [ -L "$LEGACY" ] && [ "$(readlink "$LEGACY")" = "$CANONICAL" ]; then
    pass "symlink preserved"
else
    fail "re-install broke the healed symlink"
fi
if [ "$(cat "$CANONICAL")" = "$TOKEN" ]; then
    pass "credentials preserved"
else
    fail "re-install altered the canonical credentials"
fi
if [ -e "$LEGACY.pre-omega" ]; then
    fail "re-install created a spurious backup at $LEGACY.pre-omega"
else
    pass "no spurious backup created"
fi

# ── 6. the installer no longer claims the write-through ────────────────────
echo "6. install.sh states the verified truth"
if grep -q "write through it on first login" "$INSTALL_SH"; then
    fail "install.sh still claims the LLM writes through the symlink on first login"
else
    pass "the false write-through claim is gone"
fi
# Comment prose wraps; flatten the file (and strip comment markers) so this
# asserts the STATEMENT is present, not how it happens to be laid out.
INSTALL_PROSE="$(sed 's/^[[:space:]]*#[[:space:]]\{0,1\}//' "$INSTALL_SH" | tr '\n' ' ' | tr -s ' ')"
if printf '%s' "$INSTALL_PROSE" | grep -q "rename(2) does not follow a symlink"; then
    pass "install.sh records the rename(2) semantics it depends on"
else
    fail "install.sh no longer explains why it does not plant a dangling symlink"
fi

echo ""
if [ "$FAILURES" -ne 0 ]; then
    echo "FAILED — $FAILURES check(s) regressed" >&2
    exit 1
fi
echo "OK — credential topology contract holds"
