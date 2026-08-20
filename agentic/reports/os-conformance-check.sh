#!/usr/bin/env bash
# os-conformance-check.sh - Agentik Runtime conformance checker for every OmegaOS "OS" unit.
#
# Answers one question deterministically: can the Agentik Runtime (install /
# configure / run / compose / update / evaluate / permissions) actually operate
# on each OS that ships in OmegaOS today?
#
# Pure read-only. Emits JSON on stdout. No network, no mutation.
#
# Usage:  bash os-conformance-check.sh > matrix.json
set -uo pipefail

SKILLS="${OMEGA_SKILLS:-$HOME/.omega/skills}"
REPO="${OMEGA_SRC:-$HOME/Station/SideBusiness/OmegaOS}/skills"
LIB="${AGENTIK_SKILLS:-$HOME/.omega/repos/Agentik-Skills}"
REGISTRY="$SKILLS/_os-suite-registry.json"

python3 - "$SKILLS" "$REPO" "$LIB" "$REGISTRY" <<'PY'
import json, os, sys

skills, repo, lib, registry_path = sys.argv[1:5]

# --- the OS units: every *-os directory plus the explicit builder ---
units = sorted(
    d for d in os.listdir(skills)
    if os.path.isdir(os.path.join(skills, d)) and (d.endswith("-os") or d == "personal-os-builder")
)

# --- registry (the declared suite SSOT) ---
registry_slugs, registry_deps = set(), {}
if os.path.exists(registry_path):
    reg = json.load(open(registry_path))
    for o in reg.get("os", []):
        registry_slugs.add(o["slug"])
        registry_deps[o["slug"]] = {
            "consumes": o.get("consumes_primarily_from", []),
            "hands_off": o.get("hands_off_primarily_to", []),
        }

# --- library index: any dir named <unit> anywhere in the library repo ---
lib_index = set()
if os.path.isdir(lib):
    for root, dirs, _ in os.walk(lib):
        if ".git" in root:
            continue
        lib_index.update(dirs)

def has_f(u, *p):  return os.path.isfile(os.path.join(skills, u, *p))
def has_d(u, *p):  return os.path.isdir(os.path.join(skills, u, *p))

def yaml_key(u, key):
    """Presence of a top-level key in config/os.yaml (no yaml dep needed)."""
    f = os.path.join(skills, u, "config", "os.yaml")
    if not os.path.isfile(f):
        return False
    try:
        return any(l.startswith(key + ":") for l in open(f, encoding="utf-8", errors="replace"))
    except Exception:
        return False

rows = []
for u in units:
    slug_guess = u[:-3] if u.endswith("-os") else u
    man_path = os.path.join(skills, u, "MANIFEST.json")
    man, man_err = {}, None
    if os.path.isfile(man_path):
        try:
            man = json.load(open(man_path))
        except Exception as e:
            man_err = str(e)

    in_repo = os.path.isdir(os.path.join(repo, u))
    in_lib  = u in lib_index
    slug    = man.get("slug")
    in_reg  = (slug or slug_guess) in registry_slugs

    # --- Runtime pillar readiness (the doc's six verbs + permissions) ---
    p_install    = in_repo or in_lib                      # reachable by a fresh install
    p_configure  = has_f(u, "config", "os.yaml")          # declarative config contract
    p_run        = has_f(u, "SKILL.md")                   # an entrypoint to invoke
    p_compose    = in_reg                                 # dependencies resolvable
    p_update     = bool(man.get("version")) and has_f(u, "CHANGELOG.md")
    p_evaluate   = has_d(u, "evals")
    p_permission = yaml_key(u, "requires_human_approval_for")

    pillars = {
        "install": p_install, "configure": p_configure, "run": p_run,
        "compose": p_compose, "update": p_update, "evaluate": p_evaluate,
        "permissions": p_permission,
    }
    ready = sum(1 for v in pillars.values() if v)

    rows.append({
        "unit": u,
        "tier": "full" if os.path.isfile(man_path) else "thin",
        "ssot": {"omegaos_repo": in_repo, "agentik_skills_library": in_lib,
                 "orphan": not (in_repo or in_lib)},
        "registry": {"listed": in_reg, "slug": slug or slug_guess,
                     "deps": registry_deps.get(slug or slug_guess)},
        "manifest": {
            "present": os.path.isfile(man_path), "parse_error": man_err,
            "version": man.get("version"), "schema_version": man.get("schema_version"),
            "slug": slug,
            "declares_dependencies": "dependencies" in man or "requires" in man,
            "declares_permissions": "permissions" in man,
            "declares_targets": "targets" in man or "capabilities" in man,
        },
        "files": {
            "SKILL.md": has_f(u, "SKILL.md"), "README.md": has_f(u, "README.md"),
            "INSTALL.md": has_f(u, "INSTALL.md"), "CHANGELOG.md": has_f(u, "CHANGELOG.md"),
            "MASTER.md": has_f(u, "MASTER.md"),
            "OMEGA_INTEGRATION.md": has_f(u, "OMEGA_INTEGRATION.md"),
            "config/os.yaml": has_f(u, "config", "os.yaml"),
        },
        "dirs": {d: has_d(u, d) for d in
                 ("evals", "examples", "memory", "system", "protocols",
                  "schemas", "agents", "references", "knowledge", "scripts", "runtime")},
        "pillars": pillars,
        "pillars_ready": ready,
        "verdict": "RUNTIME-READY" if ready == 7 else ("PARTIAL" if ready >= 4 else "NOT-READY"),
    })

summary = {
    "total_os_units": len(rows),
    "tier_full": sum(1 for r in rows if r["tier"] == "full"),
    "tier_thin": sum(1 for r in rows if r["tier"] == "thin"),
    "orphans": sorted(r["unit"] for r in rows if r["ssot"]["orphan"]),
    "not_in_registry": sorted(r["unit"] for r in rows if not r["registry"]["listed"]),
    "pillar_coverage": {
        p: sum(1 for r in rows if r["pillars"][p])
        for p in ("install", "configure", "run", "compose", "update", "evaluate", "permissions")
    },
    "verdicts": {
        v: sum(1 for r in rows if r["verdict"] == v)
        for v in ("RUNTIME-READY", "PARTIAL", "NOT-READY")
    },
}

print(json.dumps({"summary": summary, "os": rows}, indent=2))
PY
