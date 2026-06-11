#!/usr/bin/env bash
# OMEGA-CRON-NOVA-SELF-IMPROVE — Nova's weekly self-improvement loop.
# Sunday evening: Nova (Opus, full file access in the life store) reviews her
# week — conversation history, her own logs, the operator's corrections — and
# applies 0-3 SURGICAL improvements to her own anatomy (persona, nudge prompts,
# call agent, digest prompt…) following the ANATOMY.md guardrails (backup,
# verify, journal, announce). If nothing earned a change, she says SKIP and
# stays silent.
#
# Per-operator config lives OUTSIDE the repo: ~/.omega/nova-secrets.env
# supplies NOVA_CHAT_ID (required) and NOVA_HOME (default ~/Station/LifeStyle).
set -u
LOCK=/tmp/nova-self-improve.lock
exec 9>"$LOCK"; flock -n 9 || exit 0

OMEGA="${OMEGA_DIR:-$HOME/.omega}"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a
[ -n "${NOVA_CHAT_ID:-}" ] || exit 0   # Nova not configured on this machine
LIFE="${NOVA_HOME:-$HOME/Station/LifeStyle}"
HIST="$OMEGA/state/tg-history/$NOVA_CHAT_ID.jsonl"
LOG="$OMEGA/logs/nova-self-improve.log"
CLAUDE="${NOVA_CLAUDE_BIN:-$HOME/.local/bin/claude}"
command -v "$CLAUDE" >/dev/null 2>&1 || CLAUDE="$(command -v claude)" || exit 0

CTX="$(tail -n 200 "$HIST" 2>/dev/null | python3 -c '
import sys, json
for l in sys.stdin:
    try:
        d = json.loads(l)
        print(d.get("role", "?") + ": " + str(d.get("text") or d.get("content") or "")[:400])
    except Exception:
        pass' | tail -c 24000)"

PROMPT="C'est ta SESSION HEBDOMADAIRE D'AUTO-AMÉLIORATION (l'opérateur te l'a demandée : « Nova doit pouvoir s'auto-améliorer »).

Lis d'abord ANATOMY.md (ta carte de toi-même + garde-fous) et SELF-IMPROVEMENT.md (tes modifs passées) dans $LIFE.

## La semaine écoulée (extraits de conversation, du plus ancien au plus récent)
${CTX:-(vide)}

## Ta tâche
1. DIAGNOSTIQUE : relis la semaine + tes logs (~/.omega/logs/nova-*.log, erreurs seulement). Cherche : feedbacks/corrections de l'opérateur (explicites ou implicites — un message ignoré, une relance plate, un registre répété), bugs dans tes composants, occasions ratées d'être une meilleure amie/assistante.
2. AMÉLIORE : choisis 0 à 3 améliorations CHIRURGICALES (1 problème observé = 1 modif) et applique-les en respectant LES GARDE-FOUS d'ANATOMY.md : backup → modif → vérif syntaxe → entrée datée dans SELF-IMPROVEMENT.md. Persona, prompts de nudge/briefing, prompt de l'agent d'appel, DIGEST_PROMPT… tout ce qui est À TOI dans ANATOMY.md.
3. ANNONCE : termine par le message Telegram pour l'opérateur (2-5 lignes, ton ton à toi) : ce que tu as changé et pourquoi — c'est ta SEULE sortie texte. Si tu n'as RIEN changé (rien d'observé qui le mérite), réponds EXACTEMENT le mot: SKIP"

OUT="$("$CLAUDE" -p "$PROMPT" --append-system-prompt "$(cat "$LIFE/PERSONA.md" 2>/dev/null || cat "$OMEGA/agents/companion.md" 2>/dev/null)" --add-dir / --dangerously-skip-permissions --model claude-opus-4-8 --max-turns 40 --strict-mcp-config 2>>"$LOG")"
OUT="$(printf '%s' "$OUT" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
printf '%s self-improve ran (%s chars)\n' "$(date '+%F %T')" "${#OUT}" >> "$LOG"

printf '%s' "$OUT" | grep -qx "SKIP" && exit 0
[ -z "${OUT//[[:space:]]/}" ] && exit 0

# Companion bot token from agent-bots.json (first kind=companion entry).
TOKEN="$(python3 -c '
import json, os
try:
    bots = json.load(open(os.path.expanduser("~/.omega/agent-bots.json")))
    print(next(b["token"] for b in bots.values() if isinstance(b, dict) and b.get("kind") == "companion" and b.get("token")))
except Exception:
    pass')"
[ -n "$TOKEN" ] || exit 0
curl -s -X POST "https://api.telegram.org/bot$TOKEN/sendMessage" -d chat_id="$NOVA_CHAT_ID" --data-urlencode text="🌱 $OUT" >/dev/null
