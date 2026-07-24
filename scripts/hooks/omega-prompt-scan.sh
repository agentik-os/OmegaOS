#!/usr/bin/env bash
# omega-prompt-scan.sh — UserPromptSubmit hook: catch a multi-ask prompt at the
# moment it arrives, not after the tail tasks were already dropped.
#
# WHY HERE AND NOT AT STOP TIME: refusing a stop because "you should have fanned
# out" is useless — the work is already done serially, the turn is already spent.
# The only moment a fan-out decision can still be made is when the prompt lands.
# So this hook never blocks; it injects context (R-ORCH's trigger, L6's ENUMERATE
# step) exactly when it can still change what happens.
#
# Deliberately quiet: it only speaks when the prompt genuinely looks like several
# asks. A single-ask prompt gets nothing, because a nudge that fires every turn
# is a nudge that gets ignored.
#
# Output: {"hookSpecificOutput":{"hookEventName":"UserPromptSubmit",
#          "additionalContext":"…"}} — or nothing at all.

set -uo pipefail

[[ "${OMEGA_PROMPT_SCAN:-on}" == "off" ]] && exit 0

IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)
[ -z "$IN" ] && exit 0

command -v python3 >/dev/null 2>&1 || exit 0

OMEGA_HOOK_INPUT="$IN" python3 - <<'PY'
import json, os, re, sys, unicodedata

try:
    payload = json.loads(os.environ.get("OMEGA_HOOK_INPUT") or "{}")
except Exception:
    sys.exit(0)
if not isinstance(payload, dict):
    sys.exit(0)

prompt = payload.get("prompt") or payload.get("user_prompt") or ""
if not isinstance(prompt, str) or len(prompt) < 60:
    sys.exit(0)          # too short to be a multi-part mission

# Accent-insensitive: the operator types fast and drops diacritics, so "mets a
# jour" must match the same rule as "mets à jour".
low = unicodedata.normalize("NFD", prompt.lower())
low = "".join(c for c in low if unicodedata.category(c) != "Mn")

signals = 0
detail = []

# 1. An explicit enumeration of 3+ items. On its own this IS a multi-ask prompt,
#    so it scores the full threshold — no second signal needed.
#    Counted both as list lines AND inline ("do 1. x 2. y 3. z"), because a fast
#    operator types the whole list on one line.
enumerated = max(
    len(re.findall(r"(?m)^\s*(?:[-*•]|\d+[.)])\s+\S", prompt)),
    len(re.findall(r"(?:(?<=\s)|^)\d+[.)]\s+\S", prompt)),
)
if enumerated >= 3:
    signals += 2
    detail.append("%d enumerated items" % enumerated)

# 2. Additive connectors — the "et aussi" pattern that quietly doubles a mission.
#    FR and EN, because the operator writes in both.
connectors = re.findall(
    r"\b(?:et aussi|puis|ensuite|egalement|en plus|par ailleurs|aussi stp|"
    r"and also|then also|after that|additionally|as well as)\b", low)
if connectors:
    signals += 1
    detail.append("%d additive connector(s)" % len(connectors))

# 3. Several distinct imperatives — separate deliverables, not one elaborated ask.
verbs = set(re.findall(
    r"\b(?:fix|build|create|add|update|refactor|migrate|deploy|test|audit|review|"
    r"verify|document|analyze|analyse|implement|remove|rename|publish|ship|check|"
    r"corrige|construis|cree|ajoute|mets? a jour|met a jour|refactorise|migre|"
    r"deploie|teste|audite|verifie|documente|implemente|supprime|renomme|publie|"
    r"fais|lance|regarde|ameliore|nettoie|installe|configure)\b", low))
if len(verbs) >= 3:
    signals += 2
    detail.append("%d distinct action verbs" % len(verbs))
elif len(verbs) == 2:
    signals += 1
    detail.append("2 distinct action verbs")

# 4. Sheer length. A 300+ character prompt is far more often several asks than
#    one, but it is a weak signal on its own — it only ever confirms another.
if len(prompt) >= 300:
    signals += 1
    detail.append("%d chars" % len(prompt))

if signals < 2:
    sys.exit(0)

note = (
    "## OmegaOS — this prompt looks like SEVERAL asks (%s)\n\n"
    "Before touching anything: ENUMERATE them out loud, one line each, in the operator's "
    "own order (L6 step 1). The asks that get silently dropped are always the last ones.\n"
    "Then TaskCreate one task per ask (R-PLAN) — the finish-guard will refuse to end this "
    "session while any of them is still open, so the plan is what protects you.\n"
    "Then check disjointness: if 3+ of those tasks touch different files, DISPATCH them in "
    "parallel now (R-ORCH) — Workflow, Agent, or `omega spawn-worker` per file scope. "
    "Grinding them one at a time until the turn runs out is the failure L6 names.\n\n"
    "If it turns out to be a single ask, ignore this."
) % ", ".join(detail)

print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": note,
}}))
PY
exit 0
