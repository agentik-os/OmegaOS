#!/usr/bin/env bash
# OmegaOS TTS bench installer — local voice engines for Nova (CPU-only VPS).
# Idempotent. One engine failing must NOT kill the others (no set -e): each
# engine gets its own uv venv under ~/.omega/tts/venvs/<engine>; the daemon
# reports per-engine availability so a partial install degrades gracefully.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TTS_DIR="$OMEGA_DIR/tts"
PY="${OMEGA_TTS_PYTHON:-python3}"
FAILED=""

info() { printf '\033[36m[tts]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[tts]\033[0m %s\n' "$*"; }

mkdir -p "$TTS_DIR/venvs" "$TTS_DIR/workers" "$TTS_DIR/models/piper" "$TTS_DIR/out" "$TTS_DIR/logs"
cp -f "$SRC_DIR/ttsd.py" "$TTS_DIR/ttsd.py"
cp -f "$SRC_DIR/workers/"*.py "$TTS_DIR/workers/"

# uv makes the venv builds fast and reproducible; fall back to pip+venv.
UV="$(command -v uv || true)"
mkvenv() { # mkvenv <name> <pip-args…>
    local name="$1"; shift
    local venv="$TTS_DIR/venvs/$name"
    if [[ -n "$UV" ]]; then
        [[ -x "$venv/bin/python" ]] || "$UV" venv "$venv" --python "$PY" >/dev/null 2>&1
        "$UV" pip install --python "$venv/bin/python" --quiet "$@"
    else
        [[ -x "$venv/bin/python" ]] || "$PY" -m venv "$venv"
        "$venv/bin/pip" install --quiet --upgrade pip && "$venv/bin/pip" install --quiet "$@"
    fi
}

# ffmpeg (OGG/Opus conversion) + espeak-ng (kokoro phonemizer) are the only
# system deps; skip silently when sudo isn't available non-interactively.
if command -v apt-get >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    command -v ffmpeg >/dev/null 2>&1 || sudo apt-get install -y -qq ffmpeg >/dev/null 2>&1
    command -v espeak-ng >/dev/null 2>&1 || sudo apt-get install -y -qq espeak-ng >/dev/null 2>&1
fi

info "pocket (Kyutai Pocket TTS — CPU real-time, French)…"
mkvenv pocket pocket-tts scipy || { warn "pocket install failed"; FAILED="$FAILED pocket"; }

info "kokoro (Kokoro 82M — light, French ff_siwis)…"
mkvenv kokoro kokoro soundfile || { warn "kokoro install failed"; FAILED="$FAILED kokoro"; }

info "piper (Piper — fastest, fr_FR-siwis)…"
if mkvenv piper piper-tts; then
    "$TTS_DIR/venvs/piper/bin/python" -m piper.download_voices --data-dir "$TTS_DIR/models/piper" fr_FR-siwis-medium >/dev/null 2>&1 \
        || warn "piper voice download failed (will retry lazily at first use)"
else
    warn "piper install failed"; FAILED="$FAILED piper"
fi

info "chatterbox (Resemble — top quality, heavy; CPU torch)…"
mkvenv chatterbox chatterbox-tts --extra-index-url https://download.pytorch.org/whl/cpu \
    || { warn "chatterbox install failed"; FAILED="$FAILED chatterbox"; }

# systemd user service (Linux). The daemon is stdlib-only → system python3.
if command -v systemctl >/dev/null 2>&1; then
    SD_DIR="$HOME/.config/systemd/user"; mkdir -p "$SD_DIR"
    cat > "$SD_DIR/omega-ttsd.service" <<EOF
[Unit]
Description=OmegaOS TTS gateway daemon (voice engines for Nova)
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
ExecStart=$(command -v python3) %h/.omega/tts/ttsd.py
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user enable omega-ttsd.service 2>/dev/null || true
    systemctl --user restart omega-ttsd.service 2>/dev/null || true
fi

if [[ -n "$FAILED" ]]; then
    warn "done with failures:$FAILED (daemon still serves the engines that installed)"
    exit 1
fi
info "done — daemon on 127.0.0.1:8765 (GET /engines to check status)"
