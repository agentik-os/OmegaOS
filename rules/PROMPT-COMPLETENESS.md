# PROMPT-COMPLETENESS — Every task in the prompt gets done, verified, and reported

**Category:** Universal
**Added:** 2026-05-28

## Rule

When the user sends a prompt, the agent MUST:

1. **List every distinct task** in the prompt at the start (a todo list).
   A single prompt often contains 3+ separate requests — capture them ALL.

2. **Work each task** to completion. Use sub-agents / workers / `/goal`
   loops when available (Claude Code has them; OpenAI/Gemini may not — adapt
   to the runtime). Never stop at "most of it done".

3. **Self-verify at the end**: re-read the original prompt, go through each
   task, and confirm it was actually done (not just intended). If an item was
   missed, GO BACK and do it — the mission isn't complete until 100%.

4. **Report a recap**: a checklist of every task with its status (done /
   partial / blocked), plus proposed next steps and improvement opportunities.

## Why

Observed failure mode: the user sends a 3-part prompt, the agent does 1-2
parts and forgets the rest. When later asked "did you do X?", the agent
correctly answers "no" — proving it KNEW the task existed but skipped the
verification step. The fix is a mandatory list-then-verify loop.

## The loop

```
RECEIVE prompt
  → LIST all tasks (TodoWrite / TaskCreate)
  → for each task: execute (sub-agent if heavy)
  → SELF-VERIFY: re-read prompt, check each task actually done
  → if any incomplete → loop back, finish it
  → REPORT: checklist + next steps + improvements
```

Tokens are unlimited. Time is not a constraint. Quality (100% completion)
is the only constraint. Keep going until every task is verified done.

## Runtime adaptation

- **Claude Code**: use TaskCreate/TaskUpdate, Agent sub-agents, `/goal`, `/loop`
- **OpenAI Codex / Gemini**: no native sub-agent tools — do tasks sequentially
  in-process, but STILL list + verify + report
- Always degrade gracefully: the list-verify-report discipline applies to
  every runtime, even when orchestration primitives are absent

## Origin

User feedback (2026-05-28): "Il y a plein de fois où tu fais pas tout ce que
je te demande." The user noticed that when explicitly asked to verify,
the agent immediately knew what it had skipped. Therefore: make the
verification step mandatory, not on-demand.
