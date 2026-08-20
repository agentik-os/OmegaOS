# Workflow: Deploy a governed automation

Take a scored candidate to production with controls, a runbook, observability, a
gate and an owner.

## Trigger

- A candidate cleared its score and its arithmetic and has a named owner.
- An existing automation is being materially changed and must be re-gated.

## Steps

1. **Write the blueprint:** inputs, steps, decision points, outputs, and the
   owner. Every step names the system it touches.
2. **Route every observed exception.** Handled automatically, escalated to a
   named human, or explicitly rejected with a reason. An exception with no route
   is a silent failure with a delayed arrival.
3. **Gate every irreversible step.** Sending, publishing, paying, deleting and
   signing start gated. A gate is removed only against execution statistics that
   are shown.
4. **Design the idempotency key first.** A repeated run must not double the
   effect. This is designed before retries, because a retry policy over a non
   idempotent step is a mechanism for causing the incident twice.
5. **Design retries with a ceiling and an escalation.** Unbounded retry is not
   resilience, it is a slow outage with a bill.
6. **Design deduplication and rate limits** against the real contracts from Tool
   & Integration {OS}, including their failure semantics.
7. **Design observability:** what is emitted per run, what threshold alerts, who
   receives it, and the explicit rule that silence means failure.
8. **Write the runbook,** including the manual path. Test it by having someone
   who did not build the automation follow it once.
9. **Route a consequential change to Review & Governance {OS}** and wait for the
   approval. An internal blueprint approval is not a governance approval, and
   the order is: change requested, change approved, blueprint approved, run
   started.
10. **Hand to Builder {OS}** if software must be written, and to Quality &
    Evaluation {OS} to test and gate the rollout.
11. **Deploy behind the gate,** with the owner named and reachable.
12. **Verify monitoring is receiving real data** before calling it live. A
    dashboard that has never received an event is not observability.
13. **Reconcile after the first production period** against the baseline: did the
    business outcome move in the direction the arithmetic predicted?

## Completion test

- Every observed exception has a route in the blueprint.
- Every irreversible step has a gate, or shown statistics that replaced it.
- An idempotency key exists and a deliberate double run has been tested to
  produce a single effect.
- The retry policy has a finite ceiling and an escalation at the end of it.
- Alerting exists, and silence is defined as a failure state.
- A person who did not build it has followed the runbook, including the manual
  path, successfully.
- A consequential change carries a Review & Governance {OS} approval that
  precedes the blueprint approval.
- Monitoring has received real production data.
- A reconciliation against the baseline has been scheduled with a date and an
  owner.

An automation is not live because it is deployed. It is live when it is watched,
recoverable by hand, and reconciled against the outcome it claims.
