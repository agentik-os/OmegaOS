#!/usr/bin/env bash
# blueprint-check.sh — le contrôle MÉCANIQUE des invariants d'un blueprint.
#
# Pourquoi il existe : les invariants de la doctrine vivaient uniquement en prose
# dans les fichiers de référence, donc leur respect dépendait du jugement de celui
# qui déroulait le skill. Le premier vrai run (Club OS) l'a montré : un index Convex
# sur un champ tableau, deux tables obligatoires absentes et trois tables sans flux
# ont été trouvés à la lecture, pas par un contrôle. Un autre agent, ou le même un
# jour de fatigue, produit un blueprint qui franchit les trois gates avec un schéma
# cassé. Ce script est le filet.
#
# Usage:  blueprint-check.sh <dossier-du-blueprint> [--quiet] [--gates-only]
# Sortie: 0 = tout passe · 1 = au moins un ÉCHEC · 2 = usage
#
# Read-only. N'écrit jamais dans le blueprint.
set -uo pipefail

BP="${1:-}"; shift || true
QUIET=0; GATES_ONLY=0
for a in "$@"; do
  case "$a" in
    --quiet) QUIET=1;;
    --gates-only) GATES_ONLY=1;;
  esac
done

if [[ -z "$BP" || ! -d "$BP" ]]; then
  echo "usage: blueprint-check.sh <dossier-du-blueprint> [--quiet] [--gates-only]" >&2
  exit 2
fi
BP="${BP%/}"
NAME="$(basename "$BP")"
JSON="$BP/blueprint.json"

FAIL=0; WARN=0; PASS=0
c_ok(){   PASS=$((PASS+1)); [[ $QUIET -eq 1 ]] || printf '  \033[32m✓\033[0m %s\n' "$*"; }
c_no(){   FAIL=$((FAIL+1)); printf '  \033[31m✗\033[0m %s\n' "$*"; }
c_warn(){ WARN=$((WARN+1)); printf '  \033[33m!\033[0m %s\n' "$*"; }
head_(){  [[ $QUIET -eq 1 ]] || printf '\n\033[1m%s\033[0m\n' "$*"; }

[[ $QUIET -eq 1 ]] || printf '\033[1m═══ blueprint-check · %s ═══\033[0m\n' "$NAME"

# ── 1. blueprint.json ───────────────────────────────────────────────────────────
head_ "1. Le manifeste"
if [[ ! -f "$JSON" ]]; then
  c_no "blueprint.json absent — le blueprint n'a pas de manifeste"
  echo; printf '\033[31mÉCHEC\033[0m — rien de vérifiable sans manifeste.\n'; exit 1
fi
if ! python3 -c "import json,sys; json.load(open('$JSON'))" 2>/dev/null; then
  c_no "blueprint.json illisible (JSON invalide)"
  echo; printf '\033[31mÉCHEC\033[0m\n'; exit 1
fi
c_ok "blueprint.json présent et valide"

# Un champ par LIGNE, jamais séparé par des espaces : les valeurs du template en
# contiennent ("personnel | professionnel"), et un read positionnel décalerait tout
# le reste en silence.
mapfile -t META < <(python3 - "$JSON" <<'PY'
import json,sys
d=json.load(open(sys.argv[1])); g=d.get("gates",{})
def b(k): return str(g.get(k,{}).get("franchi",False)).lower()
for v in (d.get("branche","?"), d.get("primitive") or "", d.get("statut","?"),
          b("phase2_primitive"), b("phase3_business"), b("phase5_parite"),
          len(d.get("questions_ouvertes",[]))):
    print(v)
PY
)
BR="${META[0]:-?}"; PRIM="${META[1]:-}"; STAT="${META[2]:-?}"
G2="${META[3]:-false}"; G3="${META[4]:-false}"; G5="${META[5]:-false}"; QOPEN="${META[6]:-0}"
[[ "$QOPEN" =~ ^[0-9]+$ ]] || QOPEN=0

[[ "$BR" == "personnel" || "$BR" == "professionnel" ]] \
  && c_ok "branche tranchée: $BR" \
  || c_no "branche non tranchée (attendu: personnel ou professionnel, lu: $BR)"

# ── 2. Les trois gates ──────────────────────────────────────────────────────────
head_ "2. Les trois gates"
[[ "$G2" == "true" ]] && c_ok "gate phase 2 — la primitive" || c_no "gate phase 2 NON franchi — la primitive"
[[ "$G3" == "true" ]] && c_ok "gate phase 3 — le business"  || c_no "gate phase 3 NON franchi — le business"
[[ "$G5" == "true" ]] && c_ok "gate phase 5 — la parité"    || c_no "gate phase 5 NON franchi — la parité"

# La primitive ne doit jamais être un objet de réseau social. On teste chaque MOT de
# la phrase, parce qu'elle s'écrit « la personne (people + edges) » et pas « personne ».
PRIM_LC="$(printf '%s' "$PRIM" | tr '[:upper:]' '[:lower:]')"
if [[ -z "$PRIM" ]]; then
  c_no "primitive non renseignée dans blueprint.json"
elif printf '%s' "$PRIM_LC" | grep -qwE 'post|posts|content|contenu|message|messages|thread|threads'; then
  c_no "primitive interdite: « $PRIM » — c'est un réseau social, reprendre la phase 2"
else
  c_ok "primitive: $PRIM"
fi

if [[ $GATES_ONLY -eq 1 ]]; then
  echo; [[ $FAIL -eq 0 ]] && { printf '\033[32mGATES OK\033[0m\n'; exit 0; } || { printf '\033[31mGATES: %d échec(s)\033[0m\n' "$FAIL"; exit 1; }
fi

# ── 3. Les phases ont du contenu réel ───────────────────────────────────────────
head_ "3. Les phases portent du contenu"
PHASES=(00-vision 01-market 02-primitive 03-business 04-flux 05-parite 06-features
        07-automatisations 08-ia 09-data 10-stax 11-positionnement 12-gtm 13-release)
for p in "${PHASES[@]}"; do
  d="$BP/$p"
  if [[ ! -d "$d" ]]; then c_no "$p — dossier absent"; continue; fi
  # Un README seul = le template non rempli. On veut au moins un AUTRE fichier.
  n="$(find "$d" -type f ! -name README.md 2>/dev/null | wc -l)"
  if [[ "$n" -eq 0 ]]; then
    c_no "$p — vide (seul le README du template)"
  else
    # Un fichier quasi vide ne compte pas non plus.
    big="$(find "$d" -type f ! -name README.md -size +200c 2>/dev/null | wc -l)"
    [[ "$big" -gt 0 ]] && c_ok "$p — $n fichier(s)" || c_warn "$p — $n fichier(s) mais tous sous 200 octets"
  fi
done

# ── 4. Le schéma Convex — les invariants qui cassent en silence ─────────────────
head_ "4. Le schéma Convex (phase 09)"
SCHEMA="$BP/09-data/schema.ts"
if [[ ! -f "$SCHEMA" ]]; then
  c_no "09-data/schema.ts absent — /stack n'aura rien à reprendre"
else
  c_ok "schema.ts présent"

  TABLES="$(grep -oE '^  [a-zA-Z_][a-zA-Z0-9_]*: defineTable' "$SCHEMA" | sed 's/^  //; s/: defineTable//')"
  NTAB="$(printf '%s\n' "$TABLES" | grep -c . || true)"
  FIRST="$(printf '%s\n' "$TABLES" | head -1)"
  c_ok "$NTAB tables, première = $FIRST"

  # La primitive DOIT être la première table (doctrine: si on hésite, la phase 2 a échoué).
  # Le nom de table apparaît dans la phrase de la primitive, ex. « la personne (people + edges) ».
  FIRST_L="$(printf '%s' "$FIRST" | tr '[:upper:]' '[:lower:]')"
  if printf '%s' "$PRIM_LC" | grep -qw "$FIRST_L"; then
    c_ok "la primitive est bien la première table ($FIRST)"
  else
    c_warn "première table « $FIRST » absente de la primitive déclarée « $PRIM » — vérifier que c'est le même objet"
  fi

  # entries et syntheses sont OBLIGATOIRES (doctrine 4 et 5)
  printf '%s\n' "$TABLES" | grep -qx "entries" \
    && c_ok "table entries présente (les signaux)" \
    || c_no "table entries ABSENTE — obligatoire: sans elle, aucune lecture en travers"
  printf '%s\n' "$TABLES" | grep -qx "syntheses" \
    && c_ok "table syntheses présente (les sorties IA, séparées)" \
    || c_no "table syntheses ABSENTE — obligatoire: ne jamais mélanger l'observé et l'interprété"

  # Le champ tenant sur chaque table
  TENANT="$(grep -oE '(tenantId|clubId|orgId|workspaceId|accountId): v\.string\(\)' "$SCHEMA" | head -1 | cut -d: -f1)"
  if [[ -n "$TENANT" ]]; then
    NT="$(grep -cE "^    ${TENANT}: v\.string\(\)" "$SCHEMA" || true)"
    if [[ "$NT" -ge $((NTAB - 3)) ]]; then
      c_ok "champ tenant « $TENANT » sur $NT/$NTAB tables"
    else
      c_warn "champ tenant « $TENANT » sur seulement $NT/$NTAB tables — le rétrofit est un enfer"
    fi
  else
    c_warn "aucun champ tenant détecté — normal seulement si le produit ne sera jamais vendu"
  fi

  # ── LE contrôle que le run Club OS a dû faire à la main ──
  # Un index Convex sur un champ v.array(...) se construit mais ne filtre pas comme
  # attendu : bug silencieux, découvert en production.
  ARRAY_FIELDS="$(grep -oE '^    [a-zA-Z_][a-zA-Z0-9_]*: v\.array\(' "$SCHEMA" | sed 's/^    //; s/: v\.array($//' | sort -u)"
  BAD=""
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if grep -qE "\.index\(\"[^\"]+\", \[[^]]*\"${f}\"" "$SCHEMA"; then
      BAD="$BAD $f"
    fi
  done <<< "$ARRAY_FIELDS"
  if [[ -n "$BAD" ]]; then
    c_no "index sur un champ TABLEAU:$BAD — l'index se construit, la requête ne filtre pas. Passer par un searchIndex"
  else
    c_ok "aucun index posé sur un champ tableau"
  fi

  # Chaque table devrait apparaître dans un flux (phase 04)
  FLUX="$BP/04-flux"
  if [[ -d "$FLUX" ]]; then
    ORPH=""
    while IFS= read -r t; do
      [[ -z "$t" ]] && continue
      grep -rqi "\b${t}\b" "$FLUX" 2>/dev/null || ORPH="$ORPH $t"
    done <<< "$TABLES"
    if [[ -n "$ORPH" ]]; then
      c_warn "tables absentes de tout flux:$ORPH — chacune doit gagner son flux ou sortir du schéma"
    else
      c_ok "chaque table apparaît dans au moins un flux"
    fi
  fi
fi

# ── 5. La couverture des couches ────────────────────────────────────────────────
head_ "5. La couverture parité"
FEAT="$(find "$BP/06-features" -type f -name '*.md' ! -name README.md 2>/dev/null | head -1)"
if [[ -z "$FEAT" ]]; then
  c_no "06-features vide — impossible de vérifier la couverture"
else
  # Les couches sont marquées par les pastilles 🔵 socle · 🟠 parité · 🟣 différenciant
  NPAR="$(grep -o '🟠' "$FEAT" | wc -l)"
  NDIF="$(grep -o '🟣' "$FEAT" | wc -l)"
  NSOC="$(grep -o '🔵' "$FEAT" | wc -l)"
  TOT=$((NPAR + NDIF + NSOC))
  if [[ "$TOT" -eq 0 ]]; then
    c_warn "aucune pastille de couche trouvée — les features ne sont pas classées socle/parité/différenciant"
  elif [[ "$NPAR" -eq 0 ]]; then
    c_no "ZÉRO ligne de parité sur $TOT — un blueprint 100% différenciant décrit une démo, la phase 5 a été ratée"
  else
    PCT=$(( NDIF * 100 / TOT ))
    c_ok "couches: $NSOC socle · $NPAR parité · $NDIF différenciant ($TOT lignes)"
    if [[ "$PCT" -gt 30 ]]; then
      c_warn "différenciant à ${PCT}% — au-dessus de la fourchette saine 15-30%. Le travail sans précédent domine, la v1 doit être pauvre en différenciant"
    else
      c_ok "différenciant à ${PCT}% — dans la fourchette saine"
    fi
  fi
fi

# La matrice de parité doit trancher, pas décrire
PARF="$(find "$BP/05-parite" -type f -name '*.md' ! -name README.md 2>/dev/null | head -1)"
if [[ -n "$PARF" ]]; then
  NV=$(grep -cE 'CONSTRUIRE|ACHETER|DIFFÉRER|DIFFERER|REFUSER' "$PARF" || true)
  if [[ "$NV" -lt 30 ]]; then
    c_warn "seulement $NV verdicts dans la matrice de parité — la liste de référence en compte ~60"
  else
    c_ok "$NV verdicts tranchés dans la matrice"
  fi
fi

# ── 6. Les livrables de la phase 14 ─────────────────────────────────────────────
head_ "6. Les livrables"
SYNTH="$(find "$BP" -maxdepth 1 -name '*-blueprint.md' 2>/dev/null | head -1)"
if [[ -n "$SYNTH" ]]; then
  NSEC="$(grep -cE '^## [0-9]+\.' "$SYNTH" || true)"
  [[ "$NSEC" -ge 18 ]] && c_ok "synthèse présente, $NSEC sections" || c_no "synthèse à $NSEC sections — les 18 sections sont attendues"
else
  c_no "aucune synthèse <nom>-blueprint.md à la racine"
fi

ART="$(find "$BP/99-artefacts" -type f -name '*.html' 2>/dev/null | head -1)"
if [[ -n "$ART" ]]; then
  c_ok "artefact HTML présent"
  grep -qE '(src|href)="https?://' "$ART" && c_no "l'artefact charge un host EXTERNE — il doit être self-contained" || c_ok "artefact self-contained"
else
  c_no "aucun artefact HTML dans 99-artefacts — la phase 14 demande DEUX livrables"
fi

[[ "$QOPEN" -eq 3 ]] && c_ok "exactement 3 questions ouvertes" \
  || c_no "$QOPEN question(s) ouverte(s) — le skill en demande exactement 3"

# ── 7. R-NODASH ─────────────────────────────────────────────────────────────────
head_ "7. Kill pass (R-NODASH)"
NDASH=$(grep -rlP '[\x{2013}\x{2014}]' "$BP" --include='*.html' 2>/dev/null | wc -l)
[[ "$NDASH" -eq 0 ]] && c_ok "aucun em ou en dash dans les livrables HTML" \
  || c_no "$NDASH fichier(s) HTML portent un em ou en dash — les remplacer par une ponctuation humaine"

# ── verdict ─────────────────────────────────────────────────────────────────────
echo
printf '\033[1m─── %s ───\033[0m\n' "$NAME"
printf '  %d contrôles passés · %d avertissements · %d échecs\n' "$PASS" "$WARN" "$FAIL"
if [[ "$FAIL" -eq 0 ]]; then
  printf '\033[32mBLUEPRINT OK\033[0m — les invariants tiennent.\n'
  exit 0
else
  printf '\033[31mBLUEPRINT INCOMPLET\033[0m — %d invariant(s) violé(s), à corriger avant de construire.\n' "$FAIL"
  exit 1
fi
