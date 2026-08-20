# Workflow: Finalise the build

**Modes:** `GATE`, then `FINALIZE`
**Produces:** BG01 to BG20 evaluated with evidence, and the frozen final
engineering and operations handoff that Quality & Evaluation {OS} receives.

## Trigger

`omega-stepper release-check` reports PASS at the target priority: every
planned step is DONE through the verifier. That says the plan is complete, and
this workflow decides whether the build is coherent.

## Preconditions

- No step is FAILED, BLOCKED or open.
- The working tree is clean and every branch is integrated.
- Builder state validates.

## Steps

1. **Confirm the plan is genuinely closed.** `omega-stepper release-check` and
   `omega-stepper status`. A step closed without recorded evidence is reopened
   here, not waved through.
2. **Run the gates.** `omega-builder gate`. Every gate BG01 to BG20 returns
   PASS, FAIL or UNEVALUATED with the evidence that produced it.
3. **Treat UNEVALUATED as a finding.** A gate that could not run is not a gate
   that passed. Name why it could not run and who owns fixing that.
4. **Repair failures as steps, not as patches.** A failing gate becomes a step
   in the Stepper graph with its own contract and verification. Fixing it
   informally leaves the fix with no evidence.
5. **Reconcile documentation with behaviour.** Read the changed surfaces and
   the shipped documentation together. Where they disagree, the documentation
   is wrong until proven otherwise.
6. **Assemble the traceability.** Step to requirement to acceptance criterion,
   for every critical requirement. This is what Quality & Evaluation {OS} will
   audit first, and gaps found here are cheaper than gaps found there.
7. **Record the candidate.** `omega-builder set-release` with the frozen
   candidate and the Stepper release result.
8. **Check engineering readiness.** `omega-builder release-check`: Stepper
   release PASS and BG01 to BG20 PASS together.
9. **Ask before finalising.** `finalize` freezes the handoff, and it is an
   approval boundary.
10. **Finalise and hand off.** `omega-builder finalize`, then hand the artifact,
    the evidence ledger, the gate results and the traceability to Quality &
    Evaluation {OS}. Builder makes no readiness claim about the product.

## Completion test

```bash
omega-stepper release-check              # PASS at the target priority
omega-builder gate <state.json>          # every gate PASS, none UNEVALUATED
omega-builder release-check <state.json> # engineering readiness PASS
omega-builder validate <state.json>      # state validates
git status                               # clean
```

And, by inspection: the final handoff file exists and is frozen; every critical
requirement traces to at least one step and one acceptance criterion; every
closed step has recorded check evidence.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a gate is UNEVALUATED | report it as unevaluated, never as passing, and name the owner of the blocker |
| a gate fails | open a Stepper step for the fix, return to the step transaction workflow, do not patch around the gate |
| a requirement traces to no step | stop, raise it with Blueprint {OS} and Stepper {OS}; a requirement nobody built is not a documentation problem |
| the working tree is not clean | reconcile it first; a handoff frozen over uncommitted work is not reproducible |
| someone asks to finalise with a known failing gate | refuse; the exception path runs through Release {OS} and Review & Governance {OS}, with a named risk owner, not through Builder |
| Quality & Evaluation {OS} returns defects | they arrive as new Stepper steps, and this workflow runs again from the top |
