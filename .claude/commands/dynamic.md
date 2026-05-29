---
description: Run this task with Claude Code's native Dynamic Workflows — plan, fan out parallel sub-agents in-process, adversarially verify, then synthesize. The OmegaOS trigger for the Workflow tool (the successor to the old /team multi-subagent pattern). An oracle puts `/dynamic` on line 1 of a worker prompt to make that worker use Dynamic Workflows.
argument-hint: <task to run as a dynamic workflow>
---

# /dynamic — Dynamic Workflows

You are explicitly authorized to use Claude Code's native **Dynamic Workflows** (the `Workflow`
tool, Opus 4.8) for this task. **This `/dynamic` invocation IS the opt-in** — go ahead and use
the `Workflow` tool.

## Do this
1. **Plan** — decompose the task into the largest set of independent units.
2. **Fan out** — author a `Workflow` script that spawns parallel sub-agents (or pipeline
   stages) over those units, **in-process** (no rmux pane). Scale to the task — a few to hundreds.
3. **Verify adversarially** — for every non-trivial finding/output, dispatch ≥3 skeptic graders
   that try to **falsify** it (Popper); keep only majority-confirmed, cited results.
4. **Loop when unbounded** — loop-until-dry / loop-until-count / loop-until-budget for discovery
   of unknown size.
5. **Synthesize** — reconcile the sub-agent outputs into the answer **yourself**; never paste a
   sub-agent summary as the verdict.
6. **Report** — return the verified result + what was actually checked.

## When NOT to fan out
- Trivial single-step task → just do it (a workflow is overkill).
- Long isolated file mutation → that's a worker (`omega spawn-worker`), not in-process fan-out.

## OmegaOS context
- This is the canonical way to invoke Dynamic Workflows here — see Rule **R-ORCH** (Workflow is
  the primary orchestration primitive: Workflow > Agent > spawn-worker).
- Verify before reporting (Law **L1** runtime-is-truth, **L4** done-means-100%).

---

Task to run as a dynamic workflow:

$ARGUMENTS
