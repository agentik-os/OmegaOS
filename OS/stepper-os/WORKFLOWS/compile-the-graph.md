# Workflow: Compile the graph

**Mode:** `COMPILE`
**Produces:** a validated spec tree and a frozen dependency graph at
`BUILD READY`.

## Trigger

Blueprint {OS} printed `BLUEPRINT COMPLETE, STEPPER READY`, and, where the
product has a UX surface, Design {OS} reached readiness `STEPPER_READY`. Also
triggered by a Blueprint delta large enough to change the shape of the plan.

## Preconditions

- Both upstream handoffs are pinned by version and checksum, and readable from
  the repository.
- The repository is clean enough to reason about: Stepper decomposes against
  what already exists.
- `omega-stepper` is installed and its engine venv has been created by a first
  run.

## Steps

1. **Init.** `omega-stepper init --name <project>`.
2. **Declare both sources.** In `stepper.yaml`, under `sources:`, the
   `blueprint` root and the `design` root. A missing design source on a product
   with UX is not an omission to fix later; it is why steps get guessed.
3. **Read what exists.** Inventory the repository. A step that recreates
   something already built is waste, and a step that ignores an existing
   abstraction creates a parallel implementation.
4. **Decompose top down.** Modules, then epics, then slices, then steps. Stop
   splitting when a step is independently verifiable and independently
   revertible.
5. **Write the four blocks per step.** Objective, constraints, mechanically
   verifiable definition of done, and do not touch. A step whose definition of
   done cannot be expressed as a command is a step whose scope is still wrong.
6. **Attach both reference sets.** `blueprint_references` and
   `design_references`, typed: document, sections, ids. Every UI-touching step
   cites a design reference.
7. **Declare dependencies and scope.** What must be DONE first, and which files
   the step writes. File scope is what lets the planner run a safe wave.
8. **Validate.** `omega-stepper validate`. Fix every error. Read every warning;
   the design-reference warning is the expensive one to ignore.
9. **Plan once, as a check.** `omega-stepper plan` should return a non-empty
   ready set. An empty ready set on a fresh graph means the dependencies are
   wrong.
10. **Freeze and hand off.** Print `BUILD READY` and hand the graph, the
    contracts and the briefs to Builder {OS}.

## Completion test

```bash
omega-stepper validate        # no errors
omega-stepper plan            # non-empty ready set
```

And, by inspection: every step carries four filled blocks, every step's
definition of done is executable by argv, every UI-touching step cites a design
reference, the DAG is acyclic, and no two steps in the same wave declare
overlapping file scope.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a reference does not resolve | validate fails, fix the path or the pin, never delete the reference to pass |
| a requirement cannot be decomposed into verifiable steps | block on definition, raise a decision request to Blueprint {OS} |
| a UI step has no design contract to cite | raise a flow or surface request to Design {OS}, do not invent the interface in the step |
| the graph has a cycle | refuse, print the cycle, split the step that closes it |
| the ready set is empty | the dependencies are wrong; fix them rather than starting a step out of order |
| a step's definition of done can only be a human review | mark it a review gate explicitly, with the role, and get that accepted at the approval boundary |
