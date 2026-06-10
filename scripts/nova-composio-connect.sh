#!/usr/bin/env bash
# Connecte une app à Nova via Composio — catalogue: ~/.omega/nova-apps.json
#   sans argument        -> affiche le menu (apps groupées + statut connecté)
#   nova-composio-connect.sh <slug>   -> lance la connexion de l'app
# OAuth2 : imprime l'URL d'autorisation à ouvrir.  api_key : exige la clé dans
# nova-secrets.env (champ "env" du catalogue).  Multi-compte : relancer par compte.
set -uo pipefail

OMEGA="${OMEGA_DIR:-$HOME/.omega}"
CATALOG="$OMEGA/nova-apps.json"
export PATH="$HOME/Linux/bin:$HOME/.local/bin:$PATH"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a

if [ -z "${COMPOSIO_API_KEY:-}" ]; then
  echo "❌ COMPOSIO_API_KEY manquante — colle-la dans $OMEGA/nova-secrets.env d'abord." >&2
  exit 1
fi
[ -f "$CATALOG" ] || { echo "❌ Catalogue introuvable: $CATALOG" >&2; exit 1; }

# --- sans argument : afficher le menu ---------------------------------------
if [ $# -eq 0 ]; then
  echo "📲 Apps connectables par Nova (Composio)"
  python3 - "$CATALOG" <<'PY'
import json,sys
apps=json.load(open(sys.argv[1]))["apps"]
cats={}
for a in apps: cats.setdefault(a["category"],[]).append(a)
for cat,items in cats.items():
    print(f"\n— {cat} —")
    for a in items:
        mark="🟢" if a.get("connected") else "⚪"
        kind={"oauth2":"OAuth","api_key":"clé API","dcr_oauth":"OAuth-MCP"}.get(a["auth"],a["auth"])
        need=f"  (clé: {a['env']})" if a["auth"]=="api_key" else ""
        print(f"  {mark} {a['label']:<34} {a['slug']:<16} [{kind}]{need}")
print("\n→ Connecter : nova-composio-connect.sh <slug>")
PY
  exit 0
fi

# --- avec argument : connecter une app --------------------------------------
APP="$1"
read -r AUTH ENVVAR < <(python3 - "$CATALOG" "$APP" <<'PY'
import json,sys
apps=json.load(open(sys.argv[1]))["apps"]
m={a["slug"]:a for a in apps}
a=m.get(sys.argv[2])
if not a: print("__MISSING__ -"); sys.exit(0)
print(a["auth"], a.get("env","-"))
PY
)

if [ "$AUTH" = "__MISSING__" ]; then
  echo "❌ '$APP' absent du catalogue. Liste : nova-composio-connect.sh (sans argument)." >&2
  exit 1
fi

composio login --api-key "$COMPOSIO_API_KEY" >/dev/null 2>&1 || true

case "$AUTH" in
  api_key)
    KEY="${!ENVVAR:-}"
    if [ -z "$KEY" ]; then
      echo "🔑 '$APP' a besoin d'une clé API. Ajoute '$ENVVAR=...' dans $OMEGA/nova-secrets.env puis relance." >&2
      exit 2
    fi
    echo "→ Connexion de '$APP' (clé API depuis \$$ENVVAR)…"
    composio add "$APP"
    ;;
  *)
    echo "→ Connexion de '$APP' à Composio. Ouvre l'URL d'autorisation qui s'affiche :"
    composio add "$APP"
    ;;
esac
