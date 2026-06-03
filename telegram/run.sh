#!/usr/bin/env bash
# Launch the OmegaOS Telegram bot (Bun + claude-agent-sdk) using the config
# written by `omega telegram setup` (~/.omega/telegram.toml). This is the new,
# lighter Telegram layer (replaces the native Rust bridge). The bot's Claude
# session IS the AISB Master: it carries the 13 agents in its system prompt and
# dispatches to project oracles via the `omega` CLI (see src/config.ts).
set -euo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
CFG="$OMEGA_DIR/telegram.toml"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[[ -f "$CFG" ]] || { echo "No $CFG — run: omega telegram setup <BOT_TOKEN> <CHAT_ID>"; exit 1; }

# Minimal TOML read (token + allow-list) → the env the bot expects.
tok="$(grep -E '^[[:space:]]*bot_token' "$CFG" | head -1 | sed -E 's/.*= *"([^"]*)".*/\1/')"
users="$(grep -E '^[[:space:]]*allow_user_ids' "$CFG" | head -1 | sed -E 's/.*= *\[([^]]*)\].*/\1/' | tr -d ' ')"
[[ -n "$tok" ]] || { echo "bot_token missing in $CFG"; exit 1; }

export TELEGRAM_BOT_TOKEN="$tok"
export TELEGRAM_ALLOWED_USERS="$users"
export CLAUDE_WORKING_DIR="${CLAUDE_WORKING_DIR:-$HOME}"
# Single source of truth for the Master prompt when OmegaOS shipped it.
[[ -f "$OMEGA_DIR/agents/_master-runtime.md" ]] && export OMEGA_SYSTEM_PROMPT_FILE="$OMEGA_DIR/agents/_master-runtime.md"
[[ -z "${OMEGA_SYSTEM_PROMPT_FILE:-}" && -f "$OMEGA_DIR/agents/aisb-master.md" ]] && export OMEGA_SYSTEM_PROMPT_FILE="$OMEGA_DIR/agents/aisb-master.md"
# Voice transcription (optional) — reuse a configured OpenAI key if present.
[[ -z "${OPENAI_API_KEY:-}" && -f "$OMEGA_DIR/credentials/openai.key" ]] && export OPENAI_API_KEY="$(cat "$OMEGA_DIR/credentials/openai.key")"

command -v bun >/dev/null 2>&1 || { echo "bun is required — install: curl -fsSL https://bun.sh/install | bash"; exit 1; }

cd "$HERE"
[[ -d node_modules ]] || { echo "Installing bot deps (first run)…"; bun install; }
exec bun run src/index.ts
