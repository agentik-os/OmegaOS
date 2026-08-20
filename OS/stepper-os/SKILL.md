---
name: stepper-os
description: The dependency-aware step graph and its deterministic verification gate. Stepper {OS}, unit 23 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about stepper or invokes /stepper-os.
---

# Stepper {OS}

The dependency-aware step graph and its deterministic verification gate.

## When to use this

Use Stepper {OS} when:

- a Blueprint is complete (and a Design handoff exists where there is UX) and
  the question becomes what gets built, in what order;
- an agent or a team keeps declaring things done that are not;
- work is being lost across sessions because the plan lives in a chat;
- several workers need to run in parallel without writing the same files;
- you need a defensible answer to "how much of this is actually finished".

Do not use it when:

- the product is not defined. Decomposing an undefined product produces steps
  nobody can verify. Go to Blueprint {OS}.
- the work is a single change with one obvious correct form. A graph of one
  node is overhead.
- the question is whether the built thing is correct or safe. That is Quality &
  Evaluation {OS} and Security {OS}.

The near neighbour people confuse it with is Builder {OS}. Stepper decides what
is next and whether it is done. Builder does the work. An agent that keeps its
own TODO list beside Stepper has created a second plan, and the second plan is
always the one that is wrong.

## Capabilities

- Compiles a frozen Blueprint plus a Design handoff into modules, epics, slices
  and steps, each with typed references back to both sources.
- Enforces the step contract: objective, constraints, mechanically verifiable
  definition of done, do not touch.
- Validates schema, uniqueness, reference resolution and DAG acyclicity, and
  warns on a UI-touching step that cites no design reference.
- Ranks READY candidates and emits a safe execution wave that respects scope
  locks and the work-in-progress limit.
- Emits a self-contained agent brief per step.
- Runs deterministic verification (file, grep, command, review gate) by argv,
  and closes a step only on a full pass.
- Bounds the repair loop and escalates at the ceiling instead of thrashing.
- Reconciles interrupted attempts on resume, from an append-only event log.
- Issues the plan-completion release gate at a target priority.

## Procedure

1. **Init.** `omega-stepper init` in the project, then declare both upstream
   sources in `stepper.yaml`: `blueprint` and, where there is UX, `design`.
2. **Compile.** Turn the frozen handoffs into the spec tree. Every step gets
   its four blocks and its `blueprint_references` and `design_references`.
3. **Validate.** `omega-stepper validate`. Fix every schema, reference and DAG
   error. Read the warnings; a UI step with no design reference means someone
   is about to guess.
4. **Freeze and hand off.** Print `BUILD READY` and hand the graph to Builder
   {OS}.
5. **Recover state, never memory.** Every session opens with
   `resume`, `status`, `plan`.
6. **Claim one step.** `start <STEP-ID>` prints the brief. One step, one
   contract. It refuses anything not READY.
7. **Verify to close.** `done <STEP-ID>`. PASS moves to the next wave. FAIL
   opens a repair attempt against the printed evidence.
8. **Escalate at the ceiling.** When `max_fix_attempts` is reached, stop and
   hand it to a human with the accumulated evidence.
9. **Gate reviews explicitly.** `review <step> <role> PASS --by <name>`, only
   for reviews actually performed.
10. **Close the plan.** `release-check` PASS at the target priority is the only
    terminal success Stepper issues, and it is a statement about the plan.

## Handoffs

| Receives from | What arrives |
|---|---|
| Blueprint {OS} (20) | the frozen pack, pinned: requirements, contracts, acceptance criteria |
| Design {OS} (21) | `design-handoff.json` at `STEPPER_READY`: flows, surfaces, states, component contracts, work-unit seeds |
| Prototype {OS} (22) | verdicts that settle or reopen an assumption a step depends on |

| Hands to | What it expects |
|---|---|
| Builder {OS} (24) | the frozen graph at `BUILD READY`, the step contracts, the agent briefs, and the obligation to write evidence back rather than keep a competing TODO list |
| Quality & Evaluation {OS} (25), after Builder | the plan-completion verdict as one input to certification, never as certification itself |
