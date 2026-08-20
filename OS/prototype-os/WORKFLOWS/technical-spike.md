# Workflow: Technical spike

**Mode:** `SPIKE`, closing with `TEARDOWN`
**Produces:** a yes, no or blocked answer to one feasibility question, with the
evidence that produced it, and no surviving code.

## Trigger

The implementation plan depends on an approach nobody has run: an unfamiliar
platform limit, a third-party integration whose real behaviour is undocumented,
a performance assumption, a model capability, a migration path. Stepper {OS}
cannot decompose a step whose feasibility is unknown, so the spike runs first.

## Preconditions

- The question is technical and binary or measurable, not a matter of taste.
- A sandbox exists: disposable directory, disposable environment, no production
  credentials and no production data.
- A time ceiling is set. A spike without one becomes an implementation.

## Steps

1. **State the exact unknown.** Not "can we use this library" but "does this
   library sustain 200 concurrent streams on the target instance without
   dropping frames". Write the threshold now.
2. **Name what happens on each answer.** If yes, the plan proceeds as written.
   If no, this is the alternative. A spike whose two outcomes lead to the same
   plan is not worth running.
3. **Create the sandbox.** Disposable, dated, isolated from every production
   credential and dataset. Synthetic or sampled anonymised data only.
4. **Build the smallest thing that exercises the risk.** No framework, no
   structure, no tests beyond what proves the point, no error handling. This is
   the one place in the suite where that is correct.
5. **Instrument before running.** Timings, error output, resource use. A spike
   that produces an impression instead of a number has not answered anything.
6. **Run it more than once.** A single run is an anecdote. Record the spread,
   not just the best result.
7. **Record what surprised you.** The undocumented behaviour is usually worth
   more than the answer to the original question, and it belongs in the verdict.
8. **Rule.** Yes, no, or blocked with the blocker named. Compare against the
   threshold set in step 1.
9. **Write the consequences.** Which Blueprint records, Design decisions or
   planned steps this answer changes, by ID.
10. **Tear down.** Delete the sandbox, revoke keys, remove the environment,
    record the teardown.

## Completion test

By inspection of `prototypes/<id>/verdict.json`:

- the question names a measurable threshold recorded before the first run;
- more than one run is recorded, with the spread;
- the verdict is yes, no or blocked, with the blocker named where blocked;
- the surprises section is present, even when empty and marked so;
- at least one downstream record is named as unblocked or newly blocked;
- the sandbox path no longer exists and the teardown record says so.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the spike hits the time ceiling with no answer | report blocked with what was learned, propose either a longer bounded spike with a new ceiling or a decision without evidence |
| it works, and it looks like the start of the feature | delete it anyway, and hand the design to Stepper {OS}; a spike has no tests, no error handling and no review |
| the only way to test it is against production | refuse, propose a staging or synthetic equivalent, escalate if that is genuinely impossible |
| the result depends on a credential nobody will issue | record blocked on access, name the owner, do not fabricate a workaround |
| the answer invalidates an approved architecture decision | stop, route to Blueprint {OS} as a decision request with the evidence attached |
