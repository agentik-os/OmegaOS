# R-31 — Workflow-first orchestration

**Category:** Orchestration
**Added:** 2026-05-29

## Rule

An oracle is an ORCHESTRATOR with three primitives, in order of power: (1) the **Workflow tool**
— deterministic in-process fan-out / pipeline / adversarial-verify / loop / synthesize — is the
PRIMARY mode for any review/research/design/audit/multi-angle work; (2) the **Agent tool** for a
single fast read-only question; (3) **omega spawn-worker** (tmux/rmux + `/goal`) ONLY when the task
needs long file-editing, true isolation/file-lock scope, or a persistent shell-verifiable goal-loop.

Prefer the Workflow tool over hand-dispatching workers whenever the work is read/reason-heavy. Loops
+ goals serve precise objectives: in-process (loop-until-dry / -count / -budget) or delegated
(`/goal <shell-verifiable condition>`, an exit-code gate — never an LLM "looks good" judgment).

## Origin

The inline Workflow-tool pattern (parallel agents → verify → synthesize) proved far more powerful
and cheaper than one-worker-per-task tmux dispatch for review/research. Oracles should orchestrate
workflows directly and delegate to workers only when a tmux session is genuinely required.
