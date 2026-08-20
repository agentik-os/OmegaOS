# Workflow: Compile the design handoff

**Mode:** `FULL`
**Produces:** the 15 part Design Definition Pack and a validated
`design-handoff.json` at readiness `STEPPER_READY`.

## Trigger

Blueprint {OS} printed `BLUEPRINT COMPLETE, STEPPER READY` and the product has
a UX or UI surface. Also triggered by a `REVISION` large enough that recompiling
is cheaper than patching.

## Preconditions

- A frozen Blueprint handoff exists, pinned by version and checksum.
- `omega-designer intake` passes on the Blueprint intake file.
- Brand constraints and any existing component library are reachable.

## Steps

1. **Recover and normalise.** Read the pinned Blueprint. Build the coverage map
   from requirements and actors to user outcomes. Missing action semantics,
   permissions or data states become questions for Blueprint, not inventions
   here.
2. **Set the experience thesis.** One sentence. Then the primary user question
   for every major surface. A surface whose user question is unclear is a
   surface that will be redesigned twice.
3. **Score and challenge the flows.** Value, frequency, risk, urgency,
   reversibility. Delete, merge, defer or demote every flow that does not earn
   its cost. Record the before and after paths.
4. **Decide information architecture and shell.** Entities, collections, tasks,
   global utilities. Choose the shell per surface: route, hub and drill, STAX
   panel rail, split view, canvas, chat first, focused editor, or a justified
   hybrid. Record the rejected shells and why.
5. **Compile journeys and state machines.** Happy, alternate, recovery,
   permission, empty, loading, stale, offline, conflict and destructive paths,
   each with entry and exit conditions, transitions, system responses,
   undo or compensation, and the success signal.
6. **Define the interaction system.** Command, menu, shortcut, selection, drag
   and drop, paste, focus, notification and progressive disclosure contracts.
   For AI products, run the `ai-app` pass here.
7. **Define the visual system.** Semantic tokens, typography roles, spacing and
   density, radii, borders, elevation, icon rules, motion, data visualisation,
   light, dark and high contrast behaviour, with do and do not examples.
8. **Write the surface and component contracts.** Purpose, user question, entry
   points, layout regions, hierarchy, actions, content, data dependencies,
   states, permissions, responsive transformation, keyboard and touch
   behaviour, analytics, acceptance criteria. Map primitives to shadcn or Base
   UI and to STAX where selected. A custom component is defined only after
   proving no existing primitive fits.
9. **Write the accessibility, responsive, localisation and trust contracts.**
   These are contracts Quality & Evaluation {OS} will later test the build
   against, so they are written as assertions, not aspirations.
10. **Write the design evals.** `EVAL-###` cases the built product must pass.
11. **Validate and freeze.** `omega-designer handoff design-handoff.json`.
    Repair until it passes, set readiness `STEPPER_READY`, and hand to
    Stepper {OS}, or to Prototype {OS} first when a flow decision is still
    riskier than it is expensive to test.

## Completion test

```bash
omega-designer handoff design-handoff.json     # must pass
```

And, by inspection: every critical Blueprint requirement traces to a flow, a
surface, a state, a component contract and an acceptance test; every surface
declares empty, loading and error; every asynchronous state has a named
rendering; navigation has exactly one source of truth; no critical UNKNOWN or
CONFLICT is ownerless; readiness is `STEPPER_READY`.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a requirement has no action semantics | register `UNK-###`, return the question to Blueprint {OS}, continue on the rest |
| a destructive action has no undo path | specify confirmation plus consequence text plus recovery, record the tradeoff in a `DDEC-###` |
| a decision needs evidence nobody has | emit a Prototype {OS} question instead of designing on a guess |
| the handoff fails validation | readiness stays blocked, no `STEPPER_READY` claim, repair and revalidate |
| output limits force a split | mark INCOMPLETE, list finished and remaining sections, resume at the exact next section with IDs preserved |
