#!/usr/bin/env bash
# Ferme les sessions de travail DÉTACHÉES des users de dev (vibe, lab) + tunnels.
# PROTÈGE toujours : la connexion courante (root), l'infra (omega-mc, bots Telegram,
# daemons sécurité agentik-*, tailscaled, sshd, systemd). Usage: close-sessions.sh [--dry]
set -uo pipefail
DRY="${1:-}"
act() { if [ "$DRY" = "--dry" ]; then echo "  [dry] fermerait pid $*"; else kill "$@" 2>/dev/null && echo "  ✅ fermé pid $*"; fi; }

echo "== Tunnels cloudflared =="
for p in $(pgrep -x cloudflared 2>/dev/null); do act "$p"; done

echo "== Sessions mosh détachées (vibe + lab ; root/connexion courante protégés) =="
for p in $(pgrep -x mosh-server -U vibe 2>/dev/null; pgrep -x mosh-server -U lab 2>/dev/null); do act "$p"; done

echo "== Daemons rmux (vibe + lab) =="
for p in $(pgrep -f 'rmux --__internal' -U vibe 2>/dev/null; pgrep -f 'rmux --__internal' -U lab 2>/dev/null); do act "$p"; done

echo "(root, infra omega-mc/bots/sécurité/tailscale/sshd : non touchés)"
