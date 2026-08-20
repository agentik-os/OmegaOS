# Workflow: Audit an existing interface

**Mode:** `AUDIT`
**Produces:** ranked findings by gate, plus a repair handoff Stepper {OS} can
plan against without a full redesign.

## Trigger

An interface exists (shipped product, inherited codebase, a design file, a
prototype) and someone needs to know whether it is sound. Also triggered before
a redesign, so the redesign is aimed at real defects rather than at taste.

## Preconditions

- The interface is reachable: a URL, a repository path, a design export, or
  screenshots with the flows they belong to.
- Where a Blueprint pack exists, it is pinned. Where none exists, the audit
  reports design coherence only and says so; it cannot judge conformance to a
  definition that was never written.

## Steps

1. **Inventory the surfaces.** Every screen, panel, modal and state that a user
   can reach, with the route or entry point that reaches it.
2. **Reconstruct the flows.** From the surfaces, rebuild the journeys actually
   available, not the ones the team believes exist.
3. **Run the gate catalogue.** For each surface and flow: coverage of critical
   requirements, failure and permission and latency and recovery behaviour,
   navigation source of truth, destructive action policy, AI state visibility,
   empty and loading and error states, mobile transformation, keyboard and focus
   restoration, token and component semantics, contrast, zoom and reflow.
4. **Cite everything.** Each finding names the surface, the state and the exact
   path or file. An uncited finding is discarded, not softened.
5. **Rank by consequence, not by ease.** A missing error state on a payment
   flow outranks a spacing inconsistency, however many spacing violations there
   are.
6. **Separate defect from preference.** A violated contract is a defect. A
   disagreement about aesthetics is a proposal, labelled as one.
7. **Write the repair handoff.** Each finding becomes a work unit seed with the
   surface, the contract it violates, the fix and the acceptance test. This is
   what Stepper {OS} plans.
8. **Route the rest.** Findings that are really product questions go to
   Blueprint {OS}. Findings about a shipped build's accessibility conformance
   are handed to Quality & Evaluation {OS}, which owns the certified audit.

## Completion test

```bash
omega-designer handoff design-handoff.json     # the repair handoff must validate
```

And, by inspection: every gate in the catalogue has been evaluated and reports a
verdict including the ones that pass, every finding carries a citation and a
consequence rank, every finding is either a work unit seed or an explicitly
routed handoff, and no finding is left with no owner.

An audit that returns only failures with no passing gates named is not
trustworthy, and neither is one that returns everything green.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a surface cannot be reached without credentials | report the surface as unaudited and name the blocker, never mark it clean |
| no Blueprint pack exists | audit coherence and contracts only, state plainly that conformance was not assessed |
| the interface contradicts its own design system | record it as a defect against the system, not as a new variant to adopt |
| the audit finds the product concept is wrong | stop expanding the audit, escalate to Blueprint {OS}, report what was covered |
