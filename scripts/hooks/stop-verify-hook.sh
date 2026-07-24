#!/usr/bin/env bash
# stop-verify-hook.sh — THE FINISH GUARD (Stop hook, Claude Code + Codex).
#
# Enforces L6 (finish the mission, never stop mid-workflow), R-PLAN (a tracked
# plan or it gets dropped) and L1/L4 (verify against runtime before "done").
#
# WHY THIS IS A REWRITE: the previous version only `echo`ed a reminder and exited
# 0. A Stop hook that exits 0 is INVISIBLE to the model — its stdout goes to the
# transcript, never into the conversation. That reminder had therefore never once
# reached an agent. Only exit code 2 (stderr → model, stop refused) actually
# changes behaviour, which is what this version does. Filename kept so every
# already-installed ~/.claude/settings.json inherits the fix with no migration.
#
# CONTRACT
#   stdin  : {"session_id":…, "transcript_path":…, "stop_hook_active":bool}
#   exit 0 : stop allowed (also the fail-open path — a guard must never break a session)
#   exit 2 : stop REFUSED, stderr is handed to the model as its next instruction
#
# LOOP SAFETY (R-LOOP): never blocks when the harness reports stop_hook_active,
# and never more than MAX_BLOCKS times per session — past the ceiling it stops
# nagging and hands control back to the operator.

set -uo pipefail

MAX_BLOCKS="${OMEGA_FINISH_GUARD_MAX_BLOCKS:-3}"
STATE_DIR="${OMEGA_DIR:-$HOME/.omega}/state/finish-guard"

# Escape hatch: a session that legitimately must end early (operator asked for a
# partial answer, a cron probe) exports OMEGA_FINISH_GUARD=off.
[[ "${OMEGA_FINISH_GUARD:-on}" == "off" ]] && exit 0

IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)

command -v python3 >/dev/null 2>&1 || exit 0   # fail-open, no interpreter

# The hook payload travels in an env var, NOT on stdin: the analyzer below is
# fed to python3 through a heredoc, which already occupies stdin.
read -r -d '' OMEGA_GUARD_SRC <<'PY'
import json, os, re, sys, hashlib

raw = os.environ.get("OMEGA_HOOK_INPUT", "")
try:
    payload = json.loads(raw) if raw.strip() else {}
except Exception:
    payload = {}
if not isinstance(payload, dict):
    payload = {}

# The harness sets this while it is already re-running the agent because of a
# previous block. Blocking again here is how a hook loops forever.
if payload.get("stop_hook_active"):
    sys.exit(0)

tp = payload.get("transcript_path") or ""
if not tp or not os.path.isfile(tp):
    sys.exit(0)                      # nothing observable → allow

created, status_of, todos = [], {}, None
edited = False
verified = False
mutations = 0      # Write/Edit/NotebookEdit/apply_patch calls
tool_calls = 0     # every tool call, plan tools excluded
plan_ever = False  # did this session EVER open a tracked plan?

VERIFY_RE = re.compile(
    r"verified|\bverify\b|build (?:passed|succeeded)|0 errors?\b|tests? passed|"
    r"cargo (?:build|test|check)|npm run build|pnpm build|bun run build|pytest|"
    r"HTTP/[\d.]+ 200|\b200 OK\b",
    re.I,
)

def walk(obj):
    """Yield every tool_use block in a transcript record."""
    if isinstance(obj, dict):
        if obj.get("type") == "tool_use":
            yield obj
        for v in obj.values():
            yield from walk(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from walk(v)

try:
    with open(tp, "r", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except Exception:
                continue
            # A sub-agent's own plan is ITS business, not this session's.
            if isinstance(rec, dict) and rec.get("isSidechain"):
                continue
            for tu in walk(rec):
                name = tu.get("name") or ""
                inp = tu.get("input") or {}
                if not isinstance(inp, dict):
                    inp = {}
                if name == "TaskCreate":
                    created.append(inp.get("subject") or "(untitled task)")
                    plan_ever = True
                elif name == "TaskUpdate":
                    tid = str(inp.get("taskId") or "")
                    st = inp.get("status")
                    if tid and st:
                        status_of[tid] = st
                    plan_ever = True
                elif name == "TodoWrite":
                    t = inp.get("todos")
                    if isinstance(t, list):
                        todos = t
                    plan_ever = True
                elif name == "update_plan":       # Codex
                    t = inp.get("plan") or inp.get("steps")
                    if isinstance(t, list):
                        todos = t
                    plan_ever = True
                else:
                    tool_calls += 1
                    if name in ("Write", "Edit", "NotebookEdit", "apply_patch"):
                        edited = True
                        mutations += 1
            if not verified and VERIFY_RE.search(line):
                # Cheap whole-record scan: build logs, test output and prod
                # probes land in tool_result text, not in a tool name.
                verified = True
except Exception:
    sys.exit(0)

open_items = []

# ── 1. The tracked plan still has open work ──────────────────────────────────
if todos is not None:
    for t in todos:
        if not isinstance(t, dict):
            continue
        st = (t.get("status") or "").lower()
        if st not in ("completed", "done", "deleted", "cancelled"):
            open_items.append(t.get("content") or t.get("step") or t.get("subject") or "(task)")
elif created:
    # Task ids are handed out sequentially as "1", "2", … in creation order.
    for i, subject in enumerate(created, start=1):
        st = (status_of.get(str(i)) or "pending").lower()
        if st not in ("completed", "deleted"):
            open_items.append(subject)

# ── 2. Real work, but no tracked plan was ever opened ────────────────────────
#
# This is the hole the first two checks cannot see: a session that never calls
# a plan tool has no open tasks to inspect, so a multi-part mission can lose its
# tail tasks silently. Thresholds are deliberately conservative — a chat turn or
# a one-file lookup does 0-2 tool calls and must never be blocked. A false
# positive costs one round-trip in which the agent enumerates what it did, which
# is the exact step (L6.1) that was being skipped.
def _int_env(key, default):
    try:
        return int(os.environ.get(key) or default)
    except ValueError:
        return default

MIN_MUTATIONS = _int_env("OMEGA_FINISH_GUARD_MIN_MUTATIONS", 3)
MIN_TOOL_CALLS = _int_env("OMEGA_FINISH_GUARD_MIN_TOOLS", 15)
planless_work = (not plan_ever) and (
    mutations >= MIN_MUTATIONS or tool_calls >= MIN_TOOL_CALLS
)

reason = None
if open_items:
    shown = "\n".join("  - " + str(s)[:110] for s in open_items[:8])
    more = "" if len(open_items) <= 8 else "\n  … and %d more" % (len(open_items) - 8)
    reason = (
        "STOP REFUSED — L6 (finish the mission). %d task(s) in your own tracked plan are "
        "still open:\n%s%s\n\n"
        "Do NOT summarize, do NOT ask whether to continue, do NOT re-report what is already "
        "done. Take the first open task, set it in_progress, finish it, verify it against "
        "runtime (L1), mark it completed, then move to the next one. If a task is genuinely "
        "blocked, finish every other unblocked task first (L4), then say plainly what is "
        "blocked and why. If 3+ open tasks are file-disjoint, fan them out now (R-ORCH) "
        "instead of grinding through them one at a time."
    ) % (len(open_items), shown, more)
else:
    faults = []
    if planless_work:
        faults.append(
            "· NO TRACKED PLAN (R-PLAN). This session ran %d tool call(s) and %d file "
            "mutation(s) without ever opening one. Nothing recorded what the prompt asked "
            "for, so nothing can prove the tail tasks were not dropped — which is exactly "
            "how they get dropped.\n"
            "  Do this now: re-read the ORIGINAL prompt, TaskCreate one task per distinct "
            "thing it asked for (in the operator's own order, one per ask), mark the ones "
            "you genuinely finished AND verified as completed, then execute every task that "
            "is left. If it turns out everything really is done, the plan costs you one "
            "message and proves it." % (tool_calls, mutations)
        )
    if edited and not verified:
        faults.append(
            "· UNVERIFIED EDITS (L1/L4). Code was edited but no verification ran: no build, "
            "no test, no runtime output anywhere in the transcript.\n"
            "  Run the real check now (cargo build / npm run build / the test suite / an "
            "HTTP probe of the deployed route), read the actual output, fix what it reports. "
            "A green diff is not a green build."
        )
    if faults:
        reason = "STOP REFUSED — the mission is not finishable yet:\n\n" + "\n\n".join(faults)

if not reason:
    sys.exit(0)

# ── Bounded blocking (R-LOOP) ────────────────────────────────────────────────
state_dir = os.environ.get("OMEGA_STATE_DIR") or "/tmp/omega-finish-guard"
try:
    os.makedirs(state_dir, exist_ok=True)
except Exception:
    sys.exit(0)

sid = payload.get("session_id") or hashlib.sha1(tp.encode()).hexdigest()[:16]
sid = re.sub(r"[^A-Za-z0-9_.-]", "_", str(sid))[:64]
counter = os.path.join(state_dir, sid + ".count")

try:
    n = int(open(counter).read().strip() or "0")
except Exception:
    n = 0

try:
    max_blocks = int(os.environ.get("OMEGA_MAX_BLOCKS") or "3")
except ValueError:
    max_blocks = 3

if n >= max_blocks:
    sys.exit(0)      # ceiling reached: stop nagging, hand control back (R-LOOP)

try:
    with open(counter, "w") as fh:
        fh.write(str(n + 1))
except Exception:
    pass

if n + 1 >= max_blocks:
    reason += (
        "\n\n(finish-guard: block %d/%d — the last one for this session. If you truly cannot "
        "finish, say so explicitly and tell the operator what is blocked; do not spin.)"
    ) % (n + 1, max_blocks)

sys.stderr.write(reason + "\n")
sys.exit(2)
PY

OMEGA_HOOK_INPUT="$IN" \
OMEGA_STATE_DIR="$STATE_DIR" \
OMEGA_MAX_BLOCKS="$MAX_BLOCKS" \
    python3 -c "$OMEGA_GUARD_SRC"
rc=$?

[[ $rc -eq 2 ]] && exit 2
exit 0
