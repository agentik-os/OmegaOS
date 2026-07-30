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

# A verification is a pair: an admissible verification command followed by its
# successful tool result. Merely opening a shell, or writing "tests passed" in
# assistant prose, is never evidence.
VERIFY_COMMAND_RE = re.compile(
    r"(?:^|[\s;&|])(?:"
    r"cargo\s+(?:build|test|check|clippy|fmt\s+--check)\b|"
    r"(?:npm|pnpm|bun|yarn)\s+(?:run\s+)?(?:build|test|check|lint|typecheck)\b|"
    r"(?:python(?:3)?\s+-m\s+)?(?:pytest|unittest)\b|"
    r"go\s+(?:build|test|vet)\b|"
    r"(?:make|just)\s+(?:test|check|verify|build)\b|"
    r"(?:bash\s+)?(?:\./)?[\w./-]*verify[\w.-]*\.sh\b|"
    r"curl\s+[^\n]*(?:--fail|-f)\b"
    r")",
    re.I,
)

VERIFY_SUCCESS_RE = re.compile(
    r"Process exited with code 0|Command exited with code 0|"
    r"\bexit(?:ed)?(?: with)?(?: code)?[=: ]+0\b|"
    r"\"exit_code\"\s*:\s*0\b|test result:\s*ok|"
    r"\b\d+\s+passed\b|\bOK\b|Finished\s+\S+\s+profile|"
    r"HTTP/[\d.]+\s+2\d\d|\b2\d\d OK\b|passed;\s*0 failed",
    re.I,
)

VERIFY_FAILURE_RE = re.compile(
    r"Process exited with code [1-9]\d*|Command exited with code [1-9]\d*|"
    r"\bexit(?:ed)?(?: with)?(?: code)?[=: ]+[1-9]\d*\b|"
    r'"exit_code"\s*:\s*[1-9]\d*\b|test result:\s*failed|'
    r"\b[1-9]\d*\s+failed\b|"
    r"Traceback \(most recent call last\)",
    re.I,
)

SHELL_TOOLS = (
    "Bash",
    "shell",
    "exec",
    "exec_command",
    "run_terminal_cmd",
    "local_shell",
)
MUTATION_TOOLS = ("Write", "Edit", "NotebookEdit", "apply_patch")
DONE_STATES = ("completed", "done", "deleted", "cancelled")
CUSTOM_CMD_RE = re.compile(r'\bcmd\s*:\s*("(?:\\.|[^"\\])*")', re.S)


def _decode_tool_input(obj):
    raw = obj.get("input")
    if raw is None:
        raw = obj.get("arguments")
    if isinstance(raw, str):
        try:
            raw = json.loads(raw)
        except Exception:
            if obj.get("name") == "exec":
                match = CUSTOM_CMD_RE.search(raw)
                if match:
                    try:
                        return {"cmd": json.loads(match.group(1))}
                    except Exception:
                        pass
            return {}
    return raw if isinstance(raw, dict) else {}


def _walk_tool_uses(obj):
    if isinstance(obj, dict):
        if obj.get("type") in ("tool_use", "function_call", "custom_tool_call") and obj.get(
            "name"
        ):
            yield {
                "name": obj.get("name"),
                "input": _decode_tool_input(obj),
                "call_id": obj.get("id") or obj.get("call_id"),
            }
        for v in obj.values():
            yield from _walk_tool_uses(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _walk_tool_uses(v)


def _walk_tool_results(obj):
    if isinstance(obj, dict):
        result_type = obj.get("type")
        if result_type in (
            "tool_result",
            "function_call_output",
            "custom_tool_call_output",
        ):
            yield {
                "call_id": obj.get("tool_use_id") or obj.get("call_id"),
                "content": obj.get("content")
                if result_type == "tool_result"
                else obj.get("output"),
                "is_error": bool(obj.get("is_error")),
            }
        for value in obj.values():
            yield from _walk_tool_results(value)
    elif isinstance(obj, list):
        for value in obj:
            yield from _walk_tool_results(value)


def _command_from_input(inp):
    return str(inp.get("command") or inp.get("cmd") or "")


def _is_verification_command(command):
    return bool(command and VERIFY_COMMAND_RE.search(command))


def _result_succeeded(result):
    if result.get("is_error"):
        return False
    text = json.dumps(result.get("content") or "", ensure_ascii=False)
    if VERIFY_FAILURE_RE.search(text):
        return False
    return bool(VERIFY_SUCCESS_RE.search(text))


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
    pending_verifications = {}

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
                            pending_verifications.clear()
                        elif name in SHELL_TOOLS:
                            command = _command_from_input(inp)
                            if _is_verification_command(command):
                                call_id = tu.get("call_id")
                                if call_id:
                                    pending_verifications[str(call_id)] = command
                for result in _walk_tool_results(rec):
                    call_id = result.get("call_id")
                    if call_id is None:
                        continue
                    command = pending_verifications.pop(str(call_id), None)
                    if command is not None:
                        verified = _result_succeeded(result)
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
