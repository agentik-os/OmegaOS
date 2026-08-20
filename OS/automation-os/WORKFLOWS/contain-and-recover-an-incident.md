# Workflow: Contain and recover an incident

Stop the effect, recover safely, and name the control that was missing.

## Trigger

- A run failed partway through.
- A run reported success and produced the wrong effect.
- An external system returned errors, or stopped answering.
- Monitoring went silent, which is a failure and not health.

## Steps

1. **Contain before diagnosing.** Suspend the automation so it stops producing
   effects. Diagnosis on a system that is still running produces a longer
   incident and a moving target.
2. **Establish the blast radius from run evidence:** which runs, which records,
   which external systems, which customers. Count them; do not estimate them.
3. **Do not retry.** A replay before the idempotency check is how one incident
   becomes two. Retrying is a decision made later in this workflow, never a
   reflex at the start.
4. **Check idempotency explicitly.** Was the failed step idempotent, and does
   the key cover the partial state that exists now? If either answer is no, the
   recovery is manual and it is done through the runbook.
5. **Decide undo or accept**, per affected record, and ask a human when the
   effect is irreversible or touches money, legal exposure, health or a customer.
6. **Recover through the runbook's manual path,** which is why it exists. If the
   runbook does not cover this case, record that as a defect of the runbook, not
   only of the automation.
7. **Reconcile after recovery** against the business outcome. The incident is not
   over when the automation runs again; it is over when the records are right.
8. **Name the cause and, separately, the control that was missing.** These are
   different findings: the cause explains this incident, the missing control
   explains why nothing caught it.
9. **Add the missing control to the blueprint,** and re-gate the automation
   before it resumes.
10. **Update the runbook** with the case it did not cover.
11. **Route the postmortem to Review & Governance {OS}** and stage the incident
    record to Context & Memory {OS}.
12. **Resume only behind the gate,** with a bounded observation period and a
    named watcher.

## Completion test

- The automation was suspended before diagnosis began.
- The blast radius is counted from run evidence, not estimated.
- No replay happened before an explicit idempotency check.
- Every irreversible or customer affecting undo carries a recorded human
  decision.
- The records are reconciled against the business outcome, not just the run
  status.
- The cause and the missing control are recorded as two separate findings.
- The missing control is in the blueprint and the runbook covers the case.
- The postmortem reached Review & Governance {OS}.
- The resumption happened behind the gate with a named watcher and an end date
  for the observation period.

An incident closed without naming the missing control is an incident that has
been scheduled to happen again.
