# Workflow: Challenge a flow

**Mode:** `FLOW`
**Produces:** a before and after path for the named journeys, every edge state
declared, and the deleted, merged, deferred or demoted steps recorded with
reasons.

## Trigger

A journey is expensive to build, risky to get wrong, frequently used, or has
simply never been questioned. Also triggered when Prototype {OS} refutes an
interaction assumption, or when Quality & Evaluation {OS} reports a defect
cluster concentrated in one flow.

## Preconditions

- The flow exists as `FLOW-###` in the design pack, or can be reconstructed
  from the shipped product.
- The Blueprint requirements the flow serves are identified by ID.

## Steps

1. **State the user's actual goal.** Not the screen sequence: the outcome the
   person wants. Flows get long because the goal was never written down.
2. **Draw the current path.** Every step, every decision, every wait, every
   place the user must remember something the system already knows.
3. **Count the cost.** Steps, decisions, context switches, waits, and required
   prior knowledge. This is the number the redesign has to beat.
4. **Score the flow.** Value, frequency, risk, urgency, reversibility. A
   low-value, low-frequency, high-cost flow is a candidate for deletion, not
   optimisation.
5. **Attack it.** For each step ask: can it be removed, defaulted, deferred,
   merged, inferred from data the system holds, or made reversible instead of
   confirmed. Record every rejection with its reason so it is not re-proposed.
6. **Name every edge state.** Empty, loading, stale, partial, offline,
   permission denied, conflict, failure, cancellation, and what recovery looks
   like from each. This is where challenged flows usually break.
7. **Draw the new path.** With its own cost count, and the specific tradeoff it
   accepts.
8. **Record the decision.** A `DDEC-###` with problem, evidence, options,
   decision, tradeoffs, consequences, reversal trigger and owner.
9. **Re-trace.** Update the traceability for every requirement the flow serves,
   and every surface, state and component contract the change touches.
10. **Route what you cannot decide.** A product conflict goes to Blueprint
    {OS}. An empirical question goes to Prototype {OS}.

## Completion test

```bash
omega-designer handoff design-handoff.json     # must still pass
```

And, by inspection: the after path has a lower measured cost than the before
path or an explicitly recorded reason for being longer; every step of the after
path declares its edge states; a `DDEC-###` records the decision with a
reversal trigger; every requirement the flow served still traces to a surface,
a state and an acceptance test.

A challenge that produces a prettier flow with no cost count, no edge states
and no decision record has not run this workflow.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the shorter path breaks a Blueprint invariant | keep the invariant, record the constraint as the reason the flow stays long |
| two after paths are equally defensible | send the difference to Prototype {OS} as one question, do not pick on taste alone |
| the flow serves a requirement nobody can explain | ask Blueprint {OS}, and do not delete the flow on silence |
| removing a step removes a legal or consent surface | stop, escalate, record the constraint against the flow permanently |
