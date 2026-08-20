# Workflow: Ship a release

**Modes:** `BOUNDARY`, `CANDIDATE`, `GATE`, `ROLLOUT`, `VERIFY`
**Produces:** a recorded go decision, a staged rollout that reached its target
population, and production verification against the real golden path.

## Trigger

Quality & Evaluation {OS} issued a verdict and Security {OS} issued a
clearance on a pinned build, and the change is wanted in production.

## Preconditions

- Both verdicts are on file and have been read, not summarised.
- A rollback plan exists and its limits are known.
- The observability contract covers every changed path.
- The approver for a production deployment is available.

## Steps

1. **Fix the boundary.** Every change in the candidate is in or explicitly out,
   with a reason, and the release has a version. Nobody can roll back what
   nobody can enumerate.
2. **Assemble the candidate.** Artifact fingerprint, quality verdict, security
   clearance, Blueprint release definition, rollback plan, observability
   contract. Anything missing is named, not skipped.
3. **Read the residual risk out loud.** The quality verdict's uncovered
   surface, the clearance's untested surface, the known defects and their
   acceptance authorities. These belong in the decision record, not in a
   footnote.
4. **Check the observability.** Every changed path has telemetry. A path that
   cannot be observed cannot be verified, and that is a no-go reason on its
   own.
5. **Set the abort criteria before any traffic moves.** The signals, the
   thresholds, the observation window per stage, and the person who can call a
   stop.
6. **Run the gate.** Go or no-go, recorded with the decider, the evidence, the
   accepted risks and their owners. A bypassed gate stops here until Review &
   Governance {OS} grants the exception.
7. **Deploy the first stage, smallest blast radius first.** Internal, then
   canary, then progressive. Ask for approval at every stage, including the
   ones that look routine.
8. **Watch the agreed signals for the agreed window.** Not the dashboard that
   happens to be open, and not for less time than agreed because it looks fine.
9. **Verify in production after each stage.** Exercise the real golden path,
   read the real responses. A green deployment is not a working feature.
10. **Advance or abort.** Abort criteria firing means stop immediately, then
    decide roll back or hold. Never continue spreading a change while
    investigating it.
11. **Complete and hand off.** Runbooks and the observability contract to
    Operations & Automation {OS}, the customer-facing change to Delivery &
    Customer Success {OS}.

## Completion test

By inspection of the release record:

- the boundary lists every change as in or out with a reason, and names the
  version;
- the candidate pack contains the artifact, both verdicts, the release
  definition, the rollback plan and the observability contract;
- the gate decision names the decider, the evidence, each accepted risk and its
  acceptance authority, and the abort criteria;
- each rollout stage records its population, its observation window and its
  signal readings;
- production verification records the paths exercised and their real responses;
- the handoff to Operations & Automation {OS} exists.

A release recorded as shipped with no production verification entry fails this
test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the quality verdict is `DOES NOT CONFORM` | no-go, route the failing requirements back to Builder {OS} |
| the security clearance is `BLOCKED` | no-go; there is no path around an exploitable weakness without a named risk owner and a governance exception |
| a changed path has no telemetry | no-go until it does; unverifiable is not shippable |
| the rollback path is untested | treat the release as one-way, state it in the decision record, require written acceptance |
| an abort criterion fires | stop the rollout at once, then run the rollback workflow or hold with an explicit owner |
| verification fails in production | roll back by default; investigating live is a decision that needs an owner on the record |
| someone asks to skip a stage to save time | refuse; stages exist to bound the blast radius, which is exactly what a hurry threatens |
