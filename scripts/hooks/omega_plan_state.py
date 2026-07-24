"""Shared transcript analyzer for the OmegaOS finish hooks.

ONE parser, two consumers — the Stop finish-guard (stop-verify-hook.sh) and the
SessionStart contract (omega-session-contract.sh). They ask the same question
("what does this session's tracked plan look like right now?") and a second
implementation would drift from the first within a week (R-KARPATHY: no parallel
re-implementations of an existing pattern).

Consumed as a module (`import omega_plan_state`) with OMEGA_HOOKS_DIR on
sys.path, and never executed directly.
"""

import json
import os
import re

# Evidence that something actually RAN. Deliberately excludes the bare words
# "verify"/"verified": the SessionStart contract contains "VERIFY against
# runtime", that text is stored in the transcript as a hook attachment, and a
# whole-line scan for it silently disabled the check for every session. An
# agent's claim that it verified something is not evidence (L1).
VERIFY_RE = re.compile(
    r"build (?:passed|succeeded)|0 errors?\b|tests? passed|test result: ok|"
    r"cargo (?:build|test|check|clippy)|npm (?:run build|test)|pnpm (?:build|test)|"
    r"bun (?:run build|test)|yarn (?:build|test)|pytest|go (?:build|test)|"
    r"HTTP/[\d.]+ 200|\b200 OK\b|passed;\s*0 failed",
    re.I,
)

SHELL_TOOLS = ("Bash", "shell", "exec_command", "run_terminal_cmd", "local_shell")
MUTATION_TOOLS = ("Write", "Edit", "NotebookEdit", "apply_patch")
DONE_STATES = ("completed", "done", "deleted", "cancelled")


def _walk_tool_uses(obj):
    if isinstance(obj, dict):
        if obj.get("type") == "tool_use":
            yield obj
        for v in obj.values():
            yield from _walk_tool_uses(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _walk_tool_uses(v)


def _evidence(rec):
    """(ran_a_command, text) — the parts of a record that count as runtime proof.

    Shell COMMANDS and tool RESULTS only. Hook attachments, injected context and
    assistant prose are excluded on purpose.
    """
    if not isinstance(rec, dict):
        return False, ""
    if rec.get("type") == "attachment" or "attachment" in rec:
        return False, ""
    ran = False
    out = []

    def visit(obj):
        nonlocal ran
        if isinstance(obj, dict):
            t = obj.get("type")
            if t == "tool_use" and (obj.get("name") or "") in SHELL_TOOLS:
                ran = True
                inp = obj.get("input")
                if isinstance(inp, dict):
                    out.append(str(inp.get("command") or ""))
            elif t == "tool_result":
                out.append(json.dumps(obj.get("content") or ""))
            else:
                for v in obj.values():
                    visit(v)
        elif isinstance(obj, list):
            for v in obj:
                visit(v)

    visit(rec)
    return ran, "\n".join(out)


def analyze(transcript_path):
    """Reconstruct the session's plan + work signals from its transcript.

    Returns None when there is nothing observable (missing/unreadable file), so
    callers can fail open — a hook must never break a session.

    Keys: open_items, total_tasks, plan_ever, edited, verified, mutations,
    tool_calls.
    """
    if not transcript_path or not os.path.isfile(transcript_path):
        return None

    created, status_of, todos = [], {}, None
    edited = verified = plan_ever = False
    mutations = tool_calls = 0

    try:
        with open(transcript_path, "r", errors="replace") as fh:
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
                for tu in _walk_tool_uses(rec):
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
                        if name in MUTATION_TOOLS:
                            edited = True
                            mutations += 1
                            # Evidence is only evidence for the code that existed
                            # when it was gathered. A new edit invalidates it.
                            verified = False
                if not verified:
                    ran, text = _evidence(rec)
                    if ran or VERIFY_RE.search(text):
                        verified = True
    except Exception:
        return None

    open_items = []
    total = 0
    if todos is not None:
        total = len(todos)
        for t in todos:
            if not isinstance(t, dict):
                continue
            if (t.get("status") or "").lower() not in DONE_STATES:
                open_items.append(
                    t.get("content") or t.get("step") or t.get("subject") or "(task)"
                )
    elif created:
        total = len(created)
        # Task ids are handed out sequentially as "1", "2", … in creation order.
        for i, subject in enumerate(created, start=1):
            if (status_of.get(str(i)) or "pending").lower() not in DONE_STATES:
                open_items.append(subject)

    return {
        "open_items": open_items,
        "total_tasks": total,
        "plan_ever": plan_ever,
        "edited": edited,
        "verified": verified,
        "mutations": mutations,
        "tool_calls": tool_calls,
    }


def int_env(key, default):
    try:
        return int(os.environ.get(key) or default)
    except ValueError:
        return default
