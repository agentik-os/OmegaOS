# Workflow: The step transaction

**Mode:** `EXECUTE`, with `REPAIR`, `REVIEW` and `INTEGRATE` inside it
**Produces:** one step closed through Stepper's verifier, with deterministic
evidence in the Builder ledger and the change integrated.

## Trigger

`omega-stepper plan` offers a READY step and there is capacity to work it. This
is the everyday unit of Builder.

## Preconditions

- Blueprint and Stepper fingerprints verified this session.
- No unreconciled attempt is open (`resume` has run).
- The working tree is either clean or reconciled and understood.

## Steps

1. **Claim.** Through Stepper (`start`), then mirror it into Builder state
   (`sync-step`, `claim`). The graph stays authoritative.
2. **Hydrate.** Read the step contract, the Blueprint references it cites, the
   design references for any surface it touches, the artifacts it depends on,
   and every prior failed attempt on this step. A repair that repeats a known
   dead end is a preventable cost.
3. **Preflight the contract.** Does it still hold against the repository as it
   is now. A contradiction found here costs minutes; found at verification it
   costs the attempt.
4. **Micro-plan.** Files, key signatures, order of work, all inside the
   contract. Anything outside the contract is a new step.
5. **Implement.** Exactly the objective, inside the constraints, never touching
   the do not touch paths. Do not restyle adjacent code, and do not refactor
   what the contract did not name.
6. **Verify with real commands.** Run the definition of done as written. Record
   each check with `record-check`: the command, and what it actually printed.
7. **Repair, bounded.** On failure, read the real output, name the failing
   assertion, make one aimed change, re-verify. Count every attempt against the
   ceiling. Before a third attempt on the same failure, get runtime evidence
   rather than another theory.
8. **Review.** Where the step declares a role gate, record the verdict with the
   real reviewer's name. An agent never signs a human gate.
9. **Integrate.** Land the change. On a conflict with another worker, do a real
   three-way merge; never discard the other side.
10. **Close.** Let Stepper's verifier close the step (`omega-stepper done`),
    then mirror the verdict with `mark-step`. Builder never marks its own step
    complete.
11. **Document.** A step that changed behaviour and changed no documentation is
    not finished.

## Completion test

```bash
omega-stepper done <STEP-ID>                 # PASS from the verifier
omega-builder status  <state.json>           # the step shows the mirrored DONE
omega-builder validate <state.json>          # no semantic invariant broken
```

And, by inspection: every check in the contract has a ledger entry holding a
real command and real output; the diff touches no do not touch path; the
working tree is clean; documentation matches behaviour.

A step marked DONE with no recorded check evidence fails this test and is
reopened.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the contract contradicts the Blueprint | block the step, raise a decision request upstream, keep other steps moving |
| the fix requires touching a do not touch path | stop, raise it as a scope question; widening the diff is the failure this block exists to prevent |
| verification needs a credential nobody issued | block with the owner named, never invent or borrow one from another project |
| the repair ceiling is reached | stop, escalate with every attempt and its evidence, leave the step FAILED |
| an integration conflict appears | serialise, merge for real, never reset or force |
| discovered work is genuinely required | create a new step in Stepper, do not smuggle it into this diff |
