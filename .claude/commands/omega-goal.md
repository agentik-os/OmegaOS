---
description: Set a /goal-style success condition for the current session and loop until met
argument-hint: <verifiable success condition>
---

This is a OmegaOS-flavored wrapper around the native Claude Code `/goal` command. It adds two behaviors on top:

1. **Persists the goal** to `~/.omega/state/session-<id>.goal` so it survives session restarts (rmux respawn, OAuth re-auth, etc.).
2. **Reports completion** by writing a `.done.json` to `~/.omega/state/oracle-<session>.done.json` so the AISB patrol sees the result via the normal pipeline.

Then invoke the real:
```
/goal $ARGUMENTS
```

After Claude completes the goal, append the result to the done.json with status=done_clean and the verification evidence (build output, test results, screenshots — whatever proves the condition is met).
