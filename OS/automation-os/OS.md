# Automation {OS}: Operating Specification

## 1. Purpose

Automate a process that has already been simplified, and govern the automation
for its whole life: controls, exceptions, approvals, observability, recovery,
reconciliation and retirement.

The value of an automation is not that it runs. It is that it runs correctly,
that somebody owns it, that it can be checked against the business outcome it
claims, and that it can be stopped and undone when it is wrong.

## 2. Boundary

- **Owns:** the automation candidate score; the blueprint (inputs, steps,
  exceptions, approvals, idempotency, retries, deduplication); the control
  design; the runbook; the deployment gate; the live monitoring of running
  automations; reconciliation against the business outcome; incident containment
  and recovery; and retirement.
- **Does not own:** diagnosing or simplifying the process. Operations {OS}
  interviews, observes, maps, measures and removes work. Automation refuses to
  start on a process Operations has not approved as simplified. It also does not
  decide code versus model judgment (AI Logic {OS}), does not write the software
  (Builder {OS}), does not certify a release (Quality & Evaluation {OS}), does
  not own the external system's contract (Tool & Integration {OS}), and does not
  approve a consequential policy change (Review & Governance {OS}).
- **Hands off to:** Builder {OS} when software must be written; Tool &
  Integration {OS} for every external call; Review & Governance {OS} for a
  consequential change; Quality & Evaluation {OS} to gate a production rollout;
  Context & Memory {OS} for blueprints, run evidence and incidents.
- **Consumes from:** Operations {OS} the approved simplified map; AI Logic {OS}
  the code versus judgment arbitration and the gain arithmetic; Tool &
  Integration {OS} the typed contracts it will call; Review & Governance {OS}
  the approval for a consequential change.

**The near neighbour it is confused with: Operations {OS}.** They were one
system and were deliberately split. Operations owns the diagnosis: how the work
actually happens, where the waste is, what should be removed, simplified,
standardised or delegated. Automation owns what happens after that verdict, and
only after it. The split exists because the merged system kept sliding into
tooling before the process was fixed, which is the single most expensive mistake
in this domain: automating a broken process makes it permanent.

It is also not Agent {OS}: an automation is a governed deterministic process
with exceptions and gates, while an agent is a supervised judgment worker. A
blueprint may contain a judgment step, and that step is designed by Agent {OS}
and arbitrated by AI Logic {OS} first.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SCORE` | Operations {OS} has approved a simplified map | scored candidates with the arithmetic visible | every candidate has a score and a verdict |
| `DESIGN` | a candidate cleared its score and its arithmetic | the blueprint: steps, inputs, exceptions, approvals | every exception has a path and every irreversible step has a gate |
| `CONTROL` | a blueprint exists | idempotency, retries, deduplication, rate limits, observability | a repeated run cannot double the effect |
| `RUNBOOK` | controls are designed | the operating runbook and the manual recovery path | a human who did not build it can run and recover it |
| `DEPLOY` | the runbook exists and the gate is open | a live automation with an owner | the rollout gate passed and monitoring is receiving data |
| `MONITOR` | an automation is live | run evidence and reconciliation against the business outcome | the outcome matches what the automation claims |
| `INCIDENT` | a run failed, or produced a wrong effect | containment, recovery and a postmortem | the effect is undone or accepted, and the cause is named |
| `RETIRE` | the automation stopped earning its keep, or its process changed | a retirement with the manual path restored | nothing still depends on it silently |

`SCORE` refuses to run without an approved simplified map. That refusal is the
main reason this OS exists as a separate unit.

## 4. Inputs

- The approved simplified process map from Operations {OS}, with owners and
  durations per step.
- The baseline: frequency, time per unit, error rate, cost. Without it there is
  nothing to reconcile against later.
- The AI Logic {OS} arbitration per step, and the gain against build plus
  maintenance.
- The exceptions. Real ones, observed rather than imagined. The happy path is
  not the process, and exception handling is most of the work.
- The typed contracts from Tool & Integration {OS} for every external system the
  automation touches, including their failure semantics.
- The named human owner, and the approval configuration for irreversible steps.

## 5. Outputs

| Artifact | Shape | Goes to |
|---|---|---|
| Candidate score | value against risk, exceptions, maintenance and change cost | the requester, Context & Memory {OS} |
| Blueprint | steps, inputs, exceptions, approvals, controls, owner | Builder {OS} when code is needed |
| Control design | idempotency key, retry policy, deduplication, rate limits, alerts | the blueprint |
| Runbook | how to run it, how to read its output, how to recover it by hand | the owner and the operators |
| Run evidence | per run: inputs, decisions, effects, duration, exceptions | Context & Memory {OS} |
| Reconciliation | the business outcome against what the automation claims | the owner, Review & Governance {OS} |
| Incident record | containment, recovery, cause, and the control that was missing | Review & Governance {OS} |
| Retirement | the manual path restored, dependencies closed | Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | approved blueprints, control designs, runbooks, run evidence, incidents, retirements | Context & Memory {OS} via `memory.record.staged` |
| projection | the live automation inventory and its health | recomputed from run evidence |
| cache | draft candidates before scoring | discarded on rescore |
| temporary | in flight run context | the run |

Run evidence is canonical, not a log to be rotated away. Without it,
reconciliation is an assertion.

## 7. Rules and invariants

1. **Never automate a broken process.** No approved simplified map from
   Operations {OS}, no automation. This refusal is not negotiable and not
   overridable by urgency.
2. **Remove, simplify, standardise, then automate.** In that order. Automating
   step seven of a process that should have five steps buys the wrong thing
   permanently.
3. **The happy path is not the process.** Every exception observed in the real
   work gets a path in the blueprint: handled automatically, routed to a human,
   or explicitly rejected. An unhandled exception is a silent failure waiting.
4. **Every automated decision has an owner and an audit trail.** Idempotency,
   retries and deduplication are business controls, not implementation details:
   a repeated run must not double the effect.
5. **No observability, no deployment.** An automation you cannot watch is a
   liability whose cost arrives later and all at once.
6. **The manual recovery path is part of the design.** If a human cannot do the
   work by hand when the automation is down, the automation has replaced a
   capability rather than supported it.
7. **Irreversible steps stay gated** until execution statistics argue otherwise,
   and the statistics are shown, not asserted. Sending, publishing, paying,
   deleting and signing all start gated.
8. **Reconcile against the business outcome, not the run status.** A green run
   that produced the wrong effect is a failure, and only reconciliation finds it.
9. **A consequential change goes through Review & Governance {OS}** before the
   blueprint is approved: request, approval, then approval of the blueprint,
   then the run. An internal blueprint approval is not a governance approval.
10. **Retirement restores the manual path.** An automation removed without
    restoring the capability it replaced leaves the process broken in a new way.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no approved simplified map | refuse to score, hand to Operations {OS}, state why |
| no baseline | refuse, request the measurement first |
| exceptions are unknown | refuse to design, go observe the real work |
| an external system has no typed contract | refuse the step, hand to Tool & Integration {OS} |
| a run fails partway | contain first, do not retry blindly, check idempotency before any replay |
| a run succeeded but the outcome is wrong | treat as an incident, reconciliation outranks run status |
| monitoring stops reporting | treat silence as a failure, never as health |
| the process changed under the automation | suspend it, do not patch around the drift |
| the owner is gone | suspend it, an unowned automation stops being maintained immediately |

## 9. Human approval boundary

This OS asks before:

- deploying any automation into production
- granting credentials or tool permissions to an automation
- automating a decision that touches money, legal exposure, health or a customer
- creating, modifying or deleting records in a system of record
- sending an external communication
- replaying a failed action, because a replay without an idempotency check is a
  second effect
- removing a manual control that a human currently performs
- retiring an automation that others depend on

An automation never widens its own permissions and never removes its own gate.

## 10. Completion criteria

A process that Operations {OS} has already simplified becomes: a scored
candidate whose arithmetic is visible, a blueprint with every exception routed
and every irreversible step gated, controls that make a repeated run safe, a
runbook a stranger can operate, a deployment that a gate approved, live
monitoring, and a reconciliation that proves the business outcome changed in the
direction the arithmetic predicted.
