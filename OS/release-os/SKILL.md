---
name: release-os
description: Ship it: release boundaries, rollout, rollback and the incident path. Release {OS}, unit 27 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about release or invokes /release-os.
---

# Release {OS}

Ship it: release boundaries, rollout, rollback and the incident path.

## When to use this

Use Release {OS} when:

- a certified and cleared build has to reach production;
- a release decision is being made and nobody has written down what is in it;
- a rollout needs a strategy, abort criteria and someone who can call it;
- production is degraded and the way back has to be executed under pressure;
- a change shipped and nobody verified it against the real golden path.

Do not use it when:

- the question is whether the build conforms. That is Quality & Evaluation
  {OS}.
- the question is whether it is safe to expose. That is Security {OS}.
- the fix is what is needed. That is Builder {OS}, through Stepper {OS}.
- the product is already live and the question is steady-state operation. That
  is Operations & Automation {OS}, after this OS hands over.

The near neighbour people confuse it with is Quality & Evaluation {OS}. Quality
issues a verdict on conformance. Release makes a decision, weighing that
verdict, the security clearance, and the business context. Merging them
produces the failure this whole chain exists to prevent: the same party
certifying and shipping.

## Capabilities

- Fixes the release boundary: what is in, what is out, what it is called.
- Assembles the release candidate and its evidence pack.
- Runs the release gate and records the go or no-go decision with its owner,
  its accepted risks and its abort criteria.
- Plans and runs a rollout: canary, progressive, staged or full, with signals
  and abort thresholds set beforehand.
- Defines the observability contract the release ships with, and hands it to
  Operations & Automation {OS}.
- Verifies in production against the real golden path, not against a green
  build.
- Writes and executes rollback plans, including the case where data has moved
  and rollback is not symmetrical.
- Runs the incident path: contain, record, hand off, route the postmortem.
- Routes exceptions to Review & Governance {OS} when a gate is bypassed.

## Procedure

1. **Fix the boundary.** Every change is in or explicitly out, with a reason.
   The version is named. An unbounded release cannot be rolled back
   meaningfully, because nobody can say what would come out.
2. **Assemble the candidate.** Artifact, quality verdict, security clearance,
   Blueprint release definition, rollback plan, observability contract.
3. **Read the verdicts, do not summarise them away.** Residual risk, uncovered
   surface, conditions, untested surface. These are the inputs to the decision,
   and they belong in the record.
4. **Set the abort criteria first.** The signals, the thresholds, and the
   person who can call a stop, agreed before any traffic moves.
5. **Decide, and record.** Go or no-go, who decided, on what evidence, with
   what accepted risk and which acceptance authority per known defect. A
   bypassed gate requires a governance exception first.
6. **Roll out by stages.** Smallest blast radius first. Watch the agreed
   signals, not the dashboard that happens to be open.
7. **Verify in production.** Exercise the real golden path. Read the real
   signals. A deploy that succeeded is not a feature that works.
8. **Roll back on failure by default.** Debugging live while a bad change
   spreads is a decision that needs an explicit owner.
9. **Hand off.** Runbooks and the observability contract to Operations &
   Automation {OS}, the customer-facing change to Delivery & Customer Success
   {OS}, postmortems and exceptions to Review & Governance {OS}.

## Handoffs

| Receives from | What arrives |
|---|---|
| Quality & Evaluation {OS} (25) | the quality verdict, its residual risk and its uncovered surface |
| Security {OS} (26) | the security clearance, its conditions and its untested surface |
| Builder {OS} (24) | the frozen build artifact and the final engineering handoff |
| Blueprint {OS} (20) | the release definition and the metric that decides it worked |

| Hands to | What it expects |
|---|---|
| Operations & Automation {OS} | runbooks, the observability contract, alert thresholds, and who is paged |
| Review & Governance {OS} | policy exceptions before a bypassed gate proceeds, and postmortems after an incident |
| Delivery & Customer Success {OS} | what changed for customers, and what to say if it breaks |
| Builder {OS} (24) | defects found in production, as Stepper steps, never as hotfixes with no contract |
| the GROW group | what is now live and can be announced |
