#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# omega-git-merge.sh — merge all per-worker omega/* branches back into the base
# ───────────────────────────────────────────────────────────────────────────
# The oracle runs this AFTER all workers are terminal + ground-truth verified.
# Merges every local omega/* branch into <base> (default: current branch), one by
# one. On a conflict it ABORTS that merge and reports it (a conflict is a real code
# issue for the oracle to resolve), leaving the tree clean. Never force-pushes,
# never pushes at all — the ship step owns the remote.
#
#   omega-git-merge [base-branch] [dir]
# ═══════════════════════════════════════════════════════════════════════════
set -uo pipefail
dir="${2:-.}"; cd "$dir" || { echo "[ERR] bad dir: $dir" >&2; exit 1; }
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "[ERR] not a git repo" >&2; exit 1; }
base="${1:-$(git rev-parse --abbrev-ref HEAD)}"

# Refuse to merge onto a dirty tree (would entangle uncommitted work).
if [ -n "$(git status --porcelain)" ]; then
  echo "[ABORT] working tree is dirty — commit/stash before merging." >&2; exit 2
fi
git checkout "$base" >/dev/null 2>&1 || { echo "[ERR] cannot checkout base $base" >&2; exit 1; }

branches="$(git for-each-ref --format='%(refname:short)' refs/heads/omega/ 2>/dev/null)"
[ -n "$branches" ] || { echo "[OK] no omega/* worker branches to merge."; exit 0; }

merged=0; conflicts=""
while IFS= read -r b; do
  [ -n "$b" ] || continue
  if git merge --no-ff --no-edit "$b" >/dev/null 2>&1; then
    merged=$((merged+1)); echo "[OK] merged $b"
  else
    git merge --abort >/dev/null 2>&1 || true
    conflicts="${conflicts}${b}\n"
    echo "[CONFLICT] $b — needs manual resolution (left unmerged)"
  fi
done <<< "$branches"

echo "──"
echo "[SUMMARY] merged=$merged onto $base"
if [ -n "$conflicts" ]; then
  echo -e "[CONFLICTS] resolve these branches then re-run:\n$conflicts" >&2
  exit 3
fi
echo "[DONE] all worker branches merged cleanly into $base (not pushed — ship step owns the remote)."
