# Workflow: Certify a build

**Modes:** `INTAKE`, `TRACE`, `PLAN`, `TEST`, `VERDICT`
**Produces:** the requirement-to-evidence matrix, executed evidence, the defect
ledger and the quality verdict handed to Security {OS}.

## Trigger

Builder {OS} ran `finalize` and handed over a build artifact with its evidence
ledger and BG01 to BG20 results. Also triggered by a request to certify an
inherited product that was never certified.

## Preconditions

- The build artifact is pinned by version or commit.
- Blueprint requirements and Design contracts are pinned and reachable.
- A test environment exists that is not production, or the use of production
  has been explicitly approved.

## Steps

1. **Pin every input.** Build artifact, Blueprint version, Design handoff
   version, Stepper plan verdict. Record the fingerprints. Certifying against a
   moving target certifies nothing.
2. **Read Builder's evidence, do not inherit it.** It says what Builder proved
   about its own steps. It is context, not certification. Where a fresh run
   contradicts it, the fresh run wins and the contradiction is a finding.
3. **Build the traceability matrix.** Every critical requirement to the
   evidence that would prove it. Produce both gap lists: requirements with no
   planned evidence, and evidence attached to no requirement.
4. **Model the risk.** Rank by consequence and uncertainty. Regulated,
   irreversible, money-touching and data-destroying paths go first, whatever
   order the feature list is in.
5. **Write the plan, including what it excludes.** The exclusions are part of
   the deliverable; a plan that appears to cover everything is the one that
   misleads.
6. **Execute.** Functional, contract, integration, regression, exploratory,
   performance, accessibility, data migration, as the plan calls for. Record
   the command, the environment and the real output for each.
7. **Record blocked as blocked.** A 401, a 403, an environment that will not
   start, a missing credential: each is an uncovered surface with an owner,
   never a pass.
8. **Run the AI evaluations** where the product has model-driven behaviour, via
   the AI behaviour workflow.
9. **Triage every finding.** Defect or observation, and if a defect, then
   severity, impact, reproduction, workaround and owner.
10. **Return defects to Builder {OS}** as Stepper steps.
11. **Rule.** Issue the verdict with the residual risk and the uncovered
    surface named. Hand it, with the evidence, to Security {OS}.

## Completion test

By inspection of the quality record:

- every pinned input is recorded with its fingerprint;
- every critical requirement appears in the matrix, mapped either to executed
  evidence or to a named uncovered surface;
- every planned test is marked run (with its real output) or not run (with a
  reason);
- every defect carries severity, impact, reproduction, workaround and owner;
- the verdict is one of `CONFORMS`, `CONFORMS WITH KNOWN DEFECTS` or `DOES NOT
  CONFORM`, and names what was not covered;
- the handoff to Security {OS} exists.

A verdict of `CONFORMS` on a matrix containing an unmapped critical requirement
fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a requirement has no testable acceptance criterion | record it uncertifiable, raise a decision request to Blueprint {OS}, never invent the criterion |
| the environment cannot be brought up | report the surface uncovered with the blocker and its owner, narrow the verdict accordingly |
| time runs out mid plan | narrow the scope transparently, list the uncovered surface in the verdict, never silently drop tests |
| a defect cannot be reproduced | record it unreproduced with the observed conditions; it stays open |
| a finding looks exploitable | route it to Security {OS} before publishing it anywhere |
| someone asks for a pass on a failing critical requirement | refuse; the exception path is Release {OS} with a named acceptance authority, not a rewritten verdict |
