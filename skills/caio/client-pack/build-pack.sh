#!/usr/bin/env bash
# Assemble the CAIO ("Chief AI Officer On Demand") CLIENT PACK ZIP from this committed
# source + each chain skill's REAL assets/templates. Reproducible (L0): a fresh checkout
# regenerates the deliverable. The branded inner-guide PDF is rendered via the OmegaOS
# pdfgen (R-PDF: `omega pdf`), never a hand-rolled generator.
#   usage: bash build-pack.sh [OUTPUT_DIR]   (default: <repo>/agentic/caio-client-pack)
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # skills/caio/client-pack
SUITE="$(cd "$HERE/.." && pwd)"                          # skills/caio
REPO="$(cd "$SUITE/../.." && pwd)"                       # repo root
OUT_DIR="${1:-$REPO/agentic/caio-client-pack}"
mkdir -p "$OUT_DIR"

# (Re)render the branded inner-guide PDF via the OmegaOS pdfgen if data is present.
if command -v omega >/dev/null 2>&1 && [[ -f "$HERE/inner-guide.json" ]]; then
  omega pdf --template=whitepaper --data="$HERE/inner-guide.json" --out="$HERE/INNER-GUIDE.pdf" || \
    echo "[build-pack] WARN: pdfgen render failed — packing without a fresh PDF"
fi

TMP="$(mktemp -d)"; STAGE="$TMP/CAIO-Client-Pack"; mkdir -p "$STAGE"
cp "$HERE/README.md"      "$STAGE/README.md"
cp "$HERE/INNER-GUIDE.md" "$STAGE/00-INNER-GUIDE.md"
[[ -f "$HERE/INNER-GUIDE.pdf" ]] && cp "$HERE/INNER-GUIDE.pdf" "$STAGE/00-INNER-GUIDE.pdf"

# phase folder -> the chain skill whose assets/templates ship as that phase's client templates
phases="01-readiness:caio-ai-readiness-assessment \
02-discovery:caio-discovery-interview \
03-blueprint:caio-enterprise-workflow-architect \
04-build:caio-implementation-runbook \
05-enablement-transfer:caio-enablement-and-transfer \
06-run-optimize:caio-run-and-optimize"
for pair in $phases; do
  phase="${pair%%:*}"; skill="${pair##*:}"
  mkdir -p "$STAGE/$phase/templates"
  [[ -f "$HERE/$phase/PLAYBOOK.md" ]] && cp "$HERE/$phase/PLAYBOOK.md" "$STAGE/$phase/PLAYBOOK.md"
  if [[ -d "$SUITE/$skill/assets/templates" ]]; then
    cp -r "$SUITE/$skill/assets/templates/." "$STAGE/$phase/templates/" 2>/dev/null || true
  else
    echo "The blueprint deliverables are the 10 \`company-ai-os/\` documents the architect produces — see PLAYBOOK.md." \
      > "$STAGE/$phase/templates/_see-playbook.md"
  fi
done

OUT="$OUT_DIR/caio-client-pack.zip"
rm -f "$OUT"
if command -v zip >/dev/null 2>&1; then
  ( cd "$TMP" && zip -rq "$OUT" "CAIO-Client-Pack" )
else
  # zip(1) not installed — portable fallback via python3 zipfile (R-STACK: Python only when a dep demands it)
  python3 - "$TMP" "CAIO-Client-Pack" "$OUT" <<'PY'
import sys, os, zipfile
base, top, out = sys.argv[1], sys.argv[2], sys.argv[3]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for dp, _, fs in os.walk(os.path.join(base, top)):
        for f in fs:
            full = os.path.join(dp, f)
            z.write(full, os.path.relpath(full, base))
PY
fi
rm -rf "$TMP"
echo "PACK: $OUT"
