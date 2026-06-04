#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-git-branch.sh — per-worker git branch lifecycle (OmegaOS)
# ───────────────────────────────────────────────────────────────────────────
# Branch-per-worker isolation so 10–100 workers can edit overlapping files in
# parallel; the oracle merges them back with omega-git-merge.sh. Safe by design:
# never force-anything, never touches a remote, only local branch ops.
#
#   omega-git-branch create <worker-name> [base-branch] [dir]
#       → creates + checks out omega/<worker-name>-<shortid> from base (default: current)
#   omega-git-branch list [dir]            → list local omega/* worker branches
#   omega-git-branch delete <branch> [dir] → delete a local omega/* branch (safe: only omega/*)
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail
action="${1:-}"; [ -n "$action" ] || { echo "usage: omega-git-branch {create|list|delete} …" >&2; exit 1; }

case "$action" in
  create)
    worker="${2:?worker-name required}"
    dir="${4:-.}"; cd "$dir"
    base="${3:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}"
    git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "[ERR] not a git repo: $dir" >&2; exit 1; }
    slug="$(printf '%s' "$worker" | tr '[:upper:] ' '[:lower:]-' | tr -cd 'a-z0-9-' | sed 's/^-*//;s/-*$//')"
    short="$(git rev-parse --short=8 HEAD 2>/dev/null || echo 00000000)"
    branch="omega/${slug:-worker}-${short}"
    # Unique-ify if it somehow exists.
    n=1; while git rev-parse --verify "$branch" >/dev/null 2>&1; do branch="omega/${slug}-${short}-${n}"; n=$((n+1)); done
    git checkout -b "$branch" "$base" >/dev/null 2>&1 || { echo "[ERR] could not branch from $base" >&2; exit 1; }
    echo "$branch"
    ;;
  list)
    dir="${2:-.}"; cd "$dir"
    git for-each-ref --format='%(refname:short)' refs/heads/omega/ 2>/dev/null || true
    ;;
  delete)
    branch="${2:?branch required}"; dir="${3:-.}"; cd "$dir"
    case "$branch" in omega/*) git branch -D "$branch" >/dev/null 2>&1 && echo "[OK] deleted $branch" ;;
      *) echo "[ERR] refusing to delete non-omega branch: $branch" >&2; exit 1 ;; esac
    ;;
  *) echo "[ERR] unknown action: $action" >&2; exit 1 ;;
esac
