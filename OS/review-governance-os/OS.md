# Review & Governance {OS}: Operating Specification

## 1. Purpose

Turn what actually happened into better judgement, and make sure consequential
change is authorised, traceable and verified.

Two jobs, deliberately in one unit. Learning without authority produces
retrospectives that change nothing. Authority without learning produces a change
board that slows everything down and improves nothing. The pack states it as one
loop:

```text
OBSERVE -> COMPARE -> EXPLAIN -> LEARN -> DECIDE -> AUTHORISE -> CHANGE ->
VERIFY -> STANDARDISE or REVERT
```

## 2. Boundary

- **Owns:** the review cadence (daily, weekly, monthly, quarterly), the
  postmortem, decision rights across the suite, written policy, change control
  for consequential change, the risk register, the audit trail, AI risk
  governance, and the standardise-or-revert verdict after a change has been
  verified.
- **The approval rule that defines this OS:** **a domain OS may not approve its
  own boundary or policy change.** Execution {OS} cannot widen its own scope,
  Client {OS} cannot grant itself a new exception class, KPI & Analytics {OS}
  cannot retire a metric other people depend on, and Operations {OS} cannot
  remove a control. Each proposes; this OS decides. That separation is the whole
  reason the unit exists.
- **Does not own:**
  - **The domain work.** It never runs a project, writes an SOP, holds a client
    conversation or measures a process. It reviews the evidence those OSes
    produce and authorises change to their boundaries.
  - **The metric definitions.** KPI & Analytics {OS} defines and reads; this OS
    consumes the readings and the breaches.
  - **The daily loop.** Execution {OS} owns the day; this OS consumes the weekly
    reset and the monthly audit.
  - **Software release gating.** Quality, Evaluation & Release {OS} owns the
    release gate. This OS owns policy over it, not the gate itself.
  - **The meeting.** Meeting {OS} runs the room; this OS supplies what the review
    is about.
- **Hands off to:** the owning OS (an approved change, with its conditions and
  its verification test), Documentation {OS} (policies and decision records),
  Process & SOP {OS} (anything standardised after verification), Context &
  Memory {OS} (what was learned, so it is not relearned).
- **Consumes from:** every OS in the suite. Specifically: Execution {OS} weekly
  resets and monthly audits, Project {OS} retrospectives and threshold-crossing
  changes, Operations {OS} control gaps, Client {OS} systemic strain and
  precedent-setting exceptions, KPI & Analytics {OS} readings and breaches, Team
  & Delegation {OS} systemic failure patterns and policy-crossing authority
  changes, Documentation {OS} unresolvable drift and orphaned policy documents,
  Quality, Evaluation & Release {OS} release evidence.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `DAILY` | the day closes | a short reflection on what happened against what was intended | it took minutes, not an hour |
| `WEEKLY` | the week closes | the operating review: what moved, what did not, what is decided this week | every open decision has an owner and a date |
| `MONTHLY` | the month closes | the metrics review against thresholds, and the systemic findings | every threshold breach has a decision or a recorded deferral |
| `QUARTERLY` | the quarter closes | the strategic governance review: policy, decision rights, risk, portfolio of changes | the risk register and the policy set are current |
| `POSTMORTEM` | an incident or a material failure | a blameless account: sequence, contributing conditions, and the change that would prevent recurrence | the account is agreed by the people involved and the change has an owner |
| `POLICY` | a rule must exist, or an existing one is contested | a written policy with its scope, its owner, its exceptions and its review date | somebody can apply it without asking its author |
| `CHANGE` | a consequential change is proposed | an authorisation decision with conditions and a verification test | approved, rejected or deferred, on the record, by someone with the right to decide |
| `RISK` | the risk cadence, or a new material risk | the risk register updated: risk, trigger, response, owner, review date | every open risk has a named owner |
| `AI RISK` | an AI system takes or shapes a consequential decision | the AI governance assessment: what it decides, what it may not, and how it is overseen | the human oversight point is named and real |
| `VERIFY` | an approved change has been made | evidence that it did what it claimed, and a standardise or revert verdict | the verdict is recorded and acted on |

## 4. Inputs

- **Evidence from the domain OSes**, in their own formats: resets, retros,
  breaches, incidents, drift reports, exception records.
- **The change proposal:** what changes, why, what is at risk, what is lost if it
  goes wrong, and how it would be reversed.
- **The current policy set and decision rights map.**
- **The risk register.**
- **What was decided before on the same question,** so a settled decision is not
  quietly reopened.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Review record | period, evidence, findings, decisions with owners and dates | the domain OSes |
| Postmortem | sequence, contributing conditions, what would prevent recurrence, owner | Documentation {OS}, Operations {OS}, Process & SOP {OS} |
| Policy | scope, rule, exceptions, owner, review date, decision rights it grants | Documentation {OS}, everyone bound by it |
| Change decision | approved, rejected or deferred, with conditions and a verification test | the proposing OS |
| Risk register entry | risk, trigger, response, owner, review date | the risk owner |
| AI governance assessment | what the system decides, its limits, its oversight point, its failure behaviour | the system's owner |
| Verification verdict | did the change do what it claimed: standardise, adjust or revert | the proposing OS |
| Audit trail | who decided what, when, on what evidence | anyone who needs to reconstruct it |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | policies, decision rights, change decisions, postmortems, risk register | governance ledger, published through Documentation {OS} |
| canonical | the audit trail | governance ledger, append-only |
| projection | metric readings, project status, delivery evidence | the owning OSes |
| cache | the assembled review pack | rebuilt each cycle from the sources |
| temporary | a draft policy under discussion | the session |

The audit trail is append-only. A governance record that can be edited afterwards
is not governance, it is a story told after the fact.

## 7. Rules and invariants

1. **A domain OS may not approve its own boundary or policy change.** Proposal
   and authorisation are separated, always, including when it is inconvenient
   and including when the same person occupies both roles. In that case the
   separation is temporal and written: the proposal is recorded before the
   decision.
2. **Decision rights are written before they are needed.** Who may decide what,
   up to what value, with what consultation. Deciding who decides during an
   incident is how incidents get worse.
3. **Postmortems are blameless and specific.** They name conditions, sequences
   and decisions, never character. A postmortem that produces only "be more
   careful" has not found the cause.
4. **Every change carries its reversal.** How it would be undone, by whom, and
   what would be lost. A change nobody can reverse gets a higher bar, not a
   quieter one.
5. **Every approved change carries a verification test.** Approval without
   verification is a wish. The test states what would show it worked and by when.
6. **Verify, then standardise or revert.** A change that did not do what it
   claimed is reverted, or explicitly kept with the reason. Silent retention of
   an unverified change is how systems accumulate rules nobody can justify.
7. **Evidence over narrative.** A review consumes what the domain OSes actually
   recorded. What people remember at the review is an input, never the record.
8. **Policies have owners and expiry.** A policy nobody owns and nobody reviews
   becomes folklore, and folklore is obeyed selectively.
9. **Cadence is protected, and short.** A review that expands until nobody
   attends has failed. Daily is minutes, weekly is short, quarterly is the long
   one.
10. **Risks have triggers, not adjectives.** "High risk" is not a risk entry. The
    trigger that would tell you it is happening, and the response, are.
11. **An AI system that shapes a consequential decision has a named human
    oversight point,** and that point is real: a person who sees the decision
    before it takes effect and can stop it.
12. **Nothing is approved retroactively without saying so.** Where a change was
    already made, the record says it was authorised after the fact.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a change was made without authorisation | record it as unauthorised, decide whether to keep or revert, and fix the decision rights gap rather than only the instance |
| the proposer is also the only possible approver | separate in time and in writing: proposal first, decision second, both recorded, and name the conflict in the record |
| a postmortem drifts toward blame | stop, return to sequence and conditions, and record what in the system made the error easy |
| no evidence is available for a review | run the review on what exists, and record the missing evidence as the first finding |
| a threshold breach has no owner | that is a decision-rights gap; assign one before discussing the number |
| a policy is contested in practice but not in writing | surface the gap between the written rule and the actual behaviour; one of the two must change |
| an approved change was never verified | mark it unverified, and treat its claimed benefit as unproven wherever it is cited |
| the review has become a status meeting | say so, and cut it back to decisions; hand status to Project {OS} and Meeting {OS} |

## 9. Human approval boundary

Review & Governance {OS} asks before:

- approving any consequential change, since authorisation itself is a human act
- publishing or amending a policy that binds other people
- assigning or changing decision rights
- accepting a risk rather than mitigating it
- reverting a change other people are relying on
- recording anything about an individual's conduct in a postmortem
- allowing an AI system to act on a consequential decision without a human in
  the path

The OS assembles evidence, frames the decision, and records it. It never
authorises on its own behalf.

## 10. Completion criteria

Every review cycle ends with decisions that have owners and dates, not with a
summary. Every consequential change in the suite can be traced to who approved
it, on what evidence, with what conditions, and whether the verification test
was passed. Every policy has an owner and a review date. No OS in the suite has
approved a change to its own boundary.
