# Stepper {OS}: Operating Specification

## 1. Purpose

Turn a frozen product definition and a resolved design into a dependency-aware
graph of atomic, independently verifiable steps, then own the truth about which
of them are done.

Stepper is the execution operating system around coding agents, not a coding
agent. It owns three things a coding agent must never own: the sequence
(planner), the truth (tracker), and the definition of done (deterministic
verifier). A step is DONE because a check passed, never because someone said so.

## 2. Boundary

- **Owns:** the decomposition into modules, epics, slices and steps; the step
  contract (objective, constraints, mechanically verifiable definition of done,
  do not touch); the dependency DAG and its acyclicity; the ready set and the
  safe execution wave; scope locks and work-in-progress limits; the attempt
  and repair lifecycle with its bounded ceiling; review gates; the append-only
  event log; and the release gate that says every step at the target priority
  is done.
- **Does not own:** what the product is (Blueprint {OS}), how it looks and
  behaves (Design {OS}), the code itself (Builder {OS}), whether the built
  product conforms to its contracts (Quality & Evaluation {OS}), or whether it
  is safe (Security {OS}) or shippable (Release {OS}). Stepper's release gate
  says the plan is complete, which is a different claim from the product is
  ready.
- **Hands off to:** Builder {OS}, at `BUILD READY`, with the graph, the step
  contracts and the agent briefs. Builder executes Stepper's program and writes
  evidence back into it.
- **Consumes from:** the frozen Blueprint {OS} handoff (what and why), the
  Design {OS} handoff (flows, screens, states), and Prototype {OS} verdicts
  that settle or reopen an assumption a step depends on.

Two upstream sources govern every step, and both are declared in `stepper.yaml`
under `sources:`. Each step names the exact documents that bind it, through
`blueprint_references` and `design_references`. A UI-touching step citing no
design reference is a validation warning, because it means someone will guess.

The rule that keeps this honest: **Stepper stops at `BUILD READY`.** It plans
and it verifies; it does not implement.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `COMPILE` | a frozen Blueprint, plus a Design handoff where there is UX | `stepper.yaml` and the spec tree of modules, epics, slices, steps | `validate` passes: schema, uniqueness, references, acyclic DAG |
| `PLAN` | a validated graph | the ranked READY candidates and the safe wave | the wave respects locks and the work-in-progress limit |
| `EXECUTE` | a step is READY and claimed | one step advanced through its lifecycle | `done` passes every deterministic check |
| `REPAIR` | a step failed verification | a bounded repair loop against printed evidence | the check passes, or the attempt ceiling escalates to a human |
| `REVIEW` | a step requires a role gate | a recorded review verdict | the gate is PASS by a named reviewer, or FAIL with reasons |
| `RESUME` | a session was interrupted | reconciled state | interrupted RUNNING or VERIFYING attempts are back to FAILED and re-offered |
| `RELEASE-CHECK` | every step at the target priority claims DONE | the plan-completion verdict | PASS, which is the only terminal success this OS issues |

## 4. Inputs

- The frozen Blueprint handoff, pinned by version and checksum.
- The Design handoff at readiness `STEPPER_READY`, where the product has a UX
  surface.
- Prototype verdicts, where a step depends on an assumption that was tested.
- The repository itself: what already exists constrains decomposition.
- Execution policy: work-in-progress limit, `max_fix_attempts`, target
  priority for release, and which roles gate which steps.

## 5. Outputs

- `stepper.yaml`: the manifest, including the declared upstream `sources:`.
- The spec tree: `stepper/{modules,epics,slices,steps}/*.yaml`. One file per
  unit, each step carrying its four blocks and its typed references.
- The agent brief per step: a self-contained markdown document a coding agent
  can execute without reading the rest of the plan.
- `.stepper/state.json` and the append-only `.stepper/events.jsonl`: runtime
  truth, restart safe.
- The plan and status reports: ranked candidates, safe wave, weighted and raw
  progress, per-status counts.
- The release-check verdict, at the target priority.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the step graph and every step contract | `stepper/**/*.yaml`, versioned with the repository |
| canonical | step status, attempts, locks, review verdicts | `.stepper/state.json` |
| canonical | the event history | `.stepper/events.jsonl`, append only |
| projection | Blueprint and Design records a step cites | typed references, resolved at validate time, never copied |
| projection | progress percentages and reports | recomputed from state |
| cache | the ranked plan for an unchanged graph | invalidated on any status write |
| temporary | one agent's working context on a claimed step | the session |

## 7. Rules and invariants

1. **DONE is never self-reported.** `done` runs the deterministic checks and
   closes the step only when every one passes. A coding agent's opinion is an
   input, never the verdict.
2. **Dependencies are authoritative.** `start` refuses a step that is not READY.
   There is no override that skips the graph.
3. **Every step carries four blocks.** Objective, constraints, mechanically
   verifiable definition of done, and do not touch. A step missing any of them
   is red and blocking. The fourth is the one everyone forgets and the one that
   prevents a 900 line diff.
4. **The definition of done is a command, not a sentence.** File, grep,
   command or review-gate checks, executed by argv, never through a shell.
5. **Both upstream sources bind.** `blueprint_references` and
   `design_references` are typed, resolve to real documents at validate time,
   and a UI-touching step with no design reference is warned about.
6. **The DAG is acyclic and validated.** A cycle is a refusal at validate, not
   a runtime surprise.
7. **The repair loop is bounded.** `execution.max_fix_attempts` is a ceiling on
   repeated failure, not on success. Hitting it escalates to a human instead of
   thrashing.
8. **State survives restarts.** `resume` reconciles interrupted attempts back
   to FAILED so the planner re-offers them. Conversational memory is not state.
9. **Scope locks serialise writers.** Two steps that write the same files never
   enter the same wave. Overlapping scope is serialised, never merged.
10. **A Blueprint or Design change is a decision request.** Stepper never
    silently redesigns to make a step easier to plan.
11. **Stepper never writes product code.** That is Builder {OS}, executing this
    graph.
12. **Release-check is about the plan.** PASS means every step at the target
    priority is DONE. It is not a statement about quality, security or
    shippability, and it is never presented as one.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a reference does not resolve to a real document | validate fails, name the step and the missing path |
| a UI step cites no design reference | validate warns, name the step, do not silently pass |
| the graph contains a cycle | refuse to validate, print the cycle |
| a step is claimed but its dependency is not DONE | `start` refuses and names the blocking dependency |
| verification fails | print the failing check and its real output, open a repair attempt |
| the repair ceiling is reached | stop repairing, escalate to a human with the accumulated evidence |
| a session died mid attempt | `resume` returns it to FAILED and the planner re-offers it |
| a step contract cannot be written verifiably | mark the step blocked on definition, escalate upstream, do not invent a check that always passes |
| a step turns out to need a Blueprint change | block the step, raise the decision request, keep the rest of the wave moving |

## 9. Human approval boundary

Stepper asks before:

- freezing the graph as `BUILD READY`
- changing the target priority for the release gate
- raising `max_fix_attempts` after a ceiling escalation
- overriding a review gate, which is never done by an agent on its own
- accepting a step whose definition of done is a human review rather than a
  deterministic check
- unblocking a step that was blocked on an upstream decision

## 10. Completion criteria

`omega-stepper validate` passes: schema, uniqueness, resolving references, and
an acyclic DAG. Every step carries its four blocks and both reference sets.
Every UI-touching step cites a design reference. The graph is frozen and
`BUILD READY` is printed, with the ready set non-empty.

The terminal success, later, is `omega-stepper release-check` PASS at the
target priority. That says the plan is complete. Quality & Evaluation {OS},
Security {OS} and Release {OS} decide whether the product ships.
