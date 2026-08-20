---
name: automation-os
description: Governed automation of a process that was simplified first. Automation {OS}, unit 67 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about automation or invokes /automation-os.
---

# Automation {OS}

Automate a process that has already been simplified, and govern it for its whole
life: controls, exceptions, approvals, observability, recovery, reconciliation
and retirement.

## When to use this

Use it when:

- Operations {OS} has mapped, measured and simplified a process, and the
  remaining work is genuinely repetitive.
- An automation exists but nobody can say whether it is producing the right
  effect.
- A run failed and the effect needs containing before anything is retried.
- An automation has no owner, no runbook, or no way to be watched.
- Somebody wants to replay a failed action and nobody has checked idempotency.
- An automation has outlived the process it was built for.

Do not use it when the process is still broken, undefined or unmeasured. In that
case the answer is Operations {OS}, and this OS will refuse.

**Near neighbours, and why this is not them.** Operations {OS} owns the
diagnosis and the simplification; this OS starts only after that verdict, and
the split exists because merged, the system kept reaching for tooling before the
process was fixed. AI Logic {OS} decides whether a step is a rule or a judgment
and whether the automation is worth building at all. Agent {OS} designs a
supervised judgment worker; an automation is a governed deterministic process.
Builder {OS} writes the software. Quality & Evaluation {OS} certifies the
release. Tool & Integration {OS} owns the contract with every external system a
blueprint calls.

## Capabilities

- Refuse to start on a process Operations {OS} has not approved as simplified.
- Score automation candidates: value against risk, exceptions, maintenance and
  change cost, with the arithmetic visible.
- Design a blueprint whose every observed exception has a path.
- Design controls: idempotency keys, retry policy, deduplication, rate limits,
  alerting thresholds.
- Write a runbook a person who did not build it can operate and recover from.
- Gate a deployment on observability actually receiving data.
- Reconcile run status against the business outcome, and treat a green run with
  a wrong effect as a failure.
- Contain and recover an incident without a blind retry.
- Route a consequential change through Review & Governance {OS} before approving
  the blueprint.
- Retire an automation and restore the manual path it replaced.

## Procedure

1. **Demand the approved simplified map.** No map from Operations {OS}, no work.
   State the refusal plainly rather than starting anyway.
2. **Demand the baseline:** frequency, time per unit, error rate, cost. This is
   what reconciliation will later compare against.
3. **Get the AI Logic {OS} arbitration** per step, and its gain against build
   plus maintenance. If the arithmetic does not clear, stop and say so.
4. **Enumerate the real exceptions** from observed work, not from the described
   process. Exception handling is most of the design, and the happy path is the
   part that never fails.
5. **Design the blueprint:** inputs, steps, decision points, exception routes,
   approvals, outputs, owner. Every exception is handled, routed to a human, or
   explicitly rejected.
6. **Design the controls.** Idempotency key first: a repeated run must not
   double the effect. Then retries with their ceiling, deduplication, rate
   limits, and the alert that fires when the automation goes quiet.
7. **Design the observability** and its failure meaning: silence is a failure,
   never health.
8. **Write the runbook,** including the manual path. If a human cannot do the
   work by hand when this is down, the design is not finished.
9. **Route a consequential change to Review & Governance {OS}** and wait for the
   approval before approving the blueprint.
10. **Gate the deployment.** Quality & Evaluation {OS} tests it; monitoring must
    be receiving real data before it is called live.
11. **Reconcile after the first production period** against the baseline and the
    business outcome, not against the run status.
12. **Review live automations periodically** and retire what no longer earns its
    keep, restoring the manual path.

## Handoffs

| Direction | Counterpart | Contract |
|---|---|---|
| in | Operations {OS} | `operations.map.approved`, the simplified process, without which nothing starts |
| in | AI Logic {OS} | `ailogic.arbitration.decided`, code versus judgment and the gain arithmetic |
| in | Tool & Integration {OS} | `integration.contract.published`, typed contracts with failure semantics |
| in | Review & Governance {OS} | `change.approved` for a consequential change |
| out | Builder {OS} | a blueprint that needs software written |
| out | Quality & Evaluation {OS} | a rollout to test and gate |
| out | Review & Governance {OS} | `automation.change.requested`, `automation.review.requested`, incidents |
| out | Context & Memory {OS} | blueprints, run evidence, reconciliations, incidents, retirements |

The one handoff this OS never accepts is a process that has not been simplified.
That refusal is the whole point of the boundary with Operations {OS}.
