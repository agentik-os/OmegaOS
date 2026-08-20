# Release {OS}: Operating Specification

## 1. Purpose

Decide whether to ship, define what is in the release and what is not, get it
into production in a way that can be observed and undone, verify it in
production, and own the path back when it goes wrong.

Quality & Evaluation {OS} says whether the build conforms. Security {OS} says
whether it is safe. Release {OS} makes the decision, and owns everything that
happens after it.

## 2. Boundary

- **Owns:** the release boundary (what is in, what is deliberately out, and the
  version it is called); the release candidate and its evidence pack; the
  go and no-go decision and its record; the rollout strategy (canary,
  progressive, staged, full) and its abort criteria; the observability contract
  the release ships with; production verification; the rollback plan and its
  execution; the incident path and handoff; and the exception path when a gate
  is bypassed.
- **Does not own:** whether the build conforms (Quality & Evaluation {OS}),
  whether it is safe (Security {OS}), the code or the fix (Builder {OS}), or
  day-to-day production operation after handoff (Operations & Automation {OS}).
  Release ships and can unship. It does not certify, and it does not repair.
- **Hands off to:** Operations & Automation {OS} (runbooks, observability
  contract, alert thresholds), Review & Governance {OS} (policy exceptions and
  postmortems), Delivery & Customer Success {OS} (what changed for customers),
  and the GROW group (what is now available to announce).
- **Consumes from:** Quality & Evaluation {OS} (the quality verdict), Security
  {OS} (the security clearance), Builder {OS} (the frozen build artifact and
  its engineering handoff), and Blueprint {OS} (the release definition: what
  this release was supposed to contain and what metric decides it worked).

The rule that keeps this honest: **one OS never both certifies and ships.**
Release weighs verdicts it did not write, and it never rewrites one to make a
decision easier.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `BOUNDARY` | a set of changes is candidate for release | what is in, what is out, and the version | every change is in or explicitly out, with a reason |
| `CANDIDATE` | the boundary is fixed | the release candidate and its evidence pack | the pack carries the artifact, the quality verdict, the clearance and the rollback plan |
| `GATE` | the candidate exists | the go or no-go decision, recorded | the decision names its risks, its owner and its abort criteria |
| `ROLLOUT` | the decision is go | the change reaching production under a strategy | the target population has the change and the abort criteria have not fired |
| `VERIFY` | the change is live | production evidence that it works | the golden path has been exercised in production and the signals are read |
| `ROLLBACK` | abort criteria fired, or a defect appeared | the previous state restored | production is back on the known good version and the data implications are stated |
| `INCIDENT` | production is degraded | the incident record and the handoff | impact is contained, the record exists, and the postmortem is routed |

## 4. Inputs

- The frozen build artifact and Builder's final engineering handoff.
- The quality verdict, with its residual risk and uncovered surface.
- The security clearance, with its conditions and untested surface.
- The Blueprint release definition: what this release contains, what it
  excludes, and the metric that decides it worked.
- The deployment target and its constraints: environments, capacity, migration
  needs, maintenance windows, customer commitments.
- The rollback capability that actually exists, which is often narrower than
  anyone assumes.

## 5. Outputs

- The release boundary and version.
- The release candidate and its evidence pack.
- The release gate decision record: go or no-go, who decided, on what evidence,
  with what accepted risk and what abort criteria.
- The rollout plan with its stages, its signals and its abort thresholds.
- The observability contract: what is measured, what alerts, at what threshold,
  and who is paged.
- The production verification report.
- The rollback plan, and where executed, the rollback record with its data
  implications.
- Incident records and the handoff to Operations & Automation {OS} and Review &
  Governance {OS}.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the release gate decision and its evidence pack | the release record, mirrored to Context & Memory {OS} |
| canonical | what version is in production, per environment | the deployment record |
| canonical | rollback plans and executed rollbacks | the release record |
| canonical | incident records | the incident store |
| projection | the quality verdict and the security clearance | read, never rewritten |
| projection | production telemetry | read from the observability system, never copied as truth |
| temporary | the in-flight rollout position | the deployment run, until it completes or aborts |

## 7. Rules and invariants

1. **No release without both verdicts.** A missing quality verdict or security
   clearance is a no-go, not an inconvenience. An unread verdict is the same as
   a missing one.
2. **The decision is a risk decision, not a perfection test.** Shipping with
   known defects is legitimate when each has an owner, an impact, a workaround
   and a named acceptance authority. Shipping with unknown defects because
   nobody looked is not.
3. **Every release has a rollback plan that has been thought through before the
   deploy.** A plan written during an incident is not a plan.
4. **Rollback is not automatic once data has moved.** A migration that is not
   reversible makes the rollback a different operation, and the release gate
   must say so before the deploy, not after.
5. **A green build is not production evidence.** Verification means exercising
   the real golden path in production and reading the real signals.
6. **Abort criteria are set before the rollout starts,** with the thresholds
   and the person who can call it. A rollout with no abort criteria is a
   deployment with extra steps.
7. **Observability ships with the release.** A change that cannot be observed
   in production cannot be verified or safely rolled out.
8. **A bypassed gate needs a governance exception.** Named risk owner, recorded
   reason, and Review & Governance {OS} approval before the deploy proceeds.
9. **Release never edits a verdict.** It may decide to ship against one, on the
   record, with an owner. It may not soften it.
10. **The incident path is owned here until handoff.** Containment first,
    diagnosis second, blame never, and the postmortem is routed to Review &
    Governance {OS}.
11. **Production is never the first test surface** for anything destructive,
    stateful or security-sensitive.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the quality verdict is `DOES NOT CONFORM` | no-go, name the failing requirements, route back to Builder {OS} |
| the security clearance is `BLOCKED` | no-go, no exception path around an exploitable weakness without a named risk owner and governance approval |
| the rollback path is untested | treat the release as one-way, say so explicitly in the decision, and require that acceptance in writing |
| a migration is irreversible | state it in the gate record before deploying, and require the approval boundary |
| an abort criterion fires mid rollout | stop the rollout at once, do not debug live traffic while it spreads, then decide roll back or hold |
| production verification fails | roll back by default; investigating in production while broken is a choice that needs an explicit owner |
| telemetry is missing for the changed path | the change is unverifiable, which is a no-go reason, not a detail to fix later |
| an incident occurs | contain first, record as you go, route the postmortem, and never rewrite the timeline afterwards |

## 9. Human approval boundary

Release asks before:

- any deployment to production
- proceeding on a bypassed or risk-accepted gate, which additionally requires a
  Review & Governance {OS} exception
- executing a rollback that involves a data migration
- any release whose rollback path is untested or unavailable
- using or touching production customer data during verification
- declaring an incident resolved

## 10. Completion criteria

The release boundary is recorded with its version. The candidate pack holds the
artifact, the quality verdict, the security clearance and the rollback plan. The
gate decision is recorded with its owner, its accepted risks and its abort
criteria. The rollout completed without an abort criterion firing, or was
aborted and rolled back. Production verification exercised the real golden path
and read the real signals. The observability contract and runbooks were handed
to Operations & Automation {OS}, and the customer-facing change to Delivery &
Customer Success {OS}.

The real test: someone can say what is in production, who decided to put it
there, on what evidence, and exactly how it would be taken back out.
