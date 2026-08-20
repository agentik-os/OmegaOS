# Delivery & Customer Success {OS}: Operating Specification

## 1. Purpose

Fulfil the promise that was sold, drive adoption of what was delivered, and earn
the renewal on demonstrated value rather than on activity.

This OS owns the whole customer journey after commercial commitment: handoff,
onboarding, discovery, success planning, delivery, scope, communication,
acceptance, adoption, value proof, renewal signals, referral and offboarding.

The governing model and the operating loop, inherited from the reference pack:

```text
CUSTOMER VALUE = RIGHT PROMISE x CLEAR SUCCESS PLAN x RELIABLE DELIVERY
                 x ADOPTION x PROOF x TRUST

SIGNED COMMITMENT -> HANDOFF -> ONBOARD -> DISCOVER -> SUCCESS PLAN ->
DELIVER -> ACCEPT -> ADOPT -> PROVE VALUE -> RENEW or EXPAND or REFER -> LEARN
```

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** the sales to delivery handoff, onboarding, the success plan and its
  measures, delivery against agreed scope, customer communication, scope change
  handling, escalation, adoption, acceptance evidence, value proof, the renewal
  SIGNALS and recommendation, case study consent, and offboarding.
- **Does not own:** what was sold (Offer {OS}), what it cost (Pricing {OS}), the
  close (Sales {OS}), the invoice or the money (Revenue {OS}), and **the renewal
  decision**. It also does not own the published case study: it owns the
  customer's consent and the evidence, Storyteller {OS} and Content {OS} own the
  telling.
- **Hands off to:** Revenue {OS} and Growth {OS} (acceptance record and adoption
  signal), Revenue {OS} (the renewal recommendation), Storyteller {OS} and
  Content {OS} (consented case study material).
- **Consumes from:** Sales {OS} (the signed commitment and every promise made in
  the sale), Offer {OS} (the scope boundary), Revenue {OS} (contract and payment
  reconciliation), Quality & Evaluation {OS} (acceptance evidence). It requires
  Revenue {OS}, because the post-payment gate cannot clear without it.

Stated identically in both units: **Delivery & Customer Success {OS} owns
fulfilment, adoption and renewal SIGNALS. Revenue {OS} owns the renewal
DECISION.** This OS produces the health picture and a recommendation; it does
not commit the business to a term, because a term is cash.

Every promise made in the sale arrives in the handoff from Sales {OS} and
becomes a delivery obligation, checked against the Offer {OS} scope boundary. A
promise that is not in the handoff was not sold. A promise in the handoff that
sits outside the offer scope is an escalation before it is work.

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `HANDOFF` | a commitment is signed and the post-payment gate has cleared | an accepted handoff with every promise itemised | context, promises, risks and ownership have all transferred |
| `ONBOARD` | the handoff is accepted | an onboarding plan and first value date | the customer has done the first thing that matters to them |
| `PLAN` | onboarding discovery is complete | a success plan with the customer's own measures | the customer agreed the measures, in writing |
| `DELIVER` | the success plan is agreed | delivered work and transparent status updates | the customer accepted the deliverable with evidence |
| `RISK` | scope moves, a date slips, or health drops | a change request or an escalation plan | it is priced, refused, or escalated, never absorbed silently |
| `ADOPT` | delivery is accepted but usage is not happening | an adoption intervention with a measurable target | usage moved, or the reason it did not is named |
| `VALUE` | a review point, or renewal approaches | value proof and a business review | every claim is attributed, and what cannot be attributed says so |
| `RENEW` | the term is ending | a renewal recommendation with its signals | the recommendation and the signals are consistent, or the conflict is stated |

## 4. Inputs

- The signed commitment and every promise made in the sale, from Sales {OS}.
- The scope boundary and guarantee from Offer {OS}.
- Contract and payment reconciliation from Revenue {OS}, which gates the start.
- Acceptance evidence from Quality & Evaluation {OS}.
- The customer's own definition of success, in their words, gathered in
  discovery rather than assumed from the sale.
- Usage, health and sentiment signals from wherever the engagement runs.

## 5. Outputs

- An accepted handoff: every promise itemised, with its scope verdict.
- An onboarding plan with a first value date.
- A success plan carrying the customer's own measures, agreed in writing.
- Status updates that state risk before the deadline, not after.
- Priced change requests, and refusals that say why.
- Acceptance records evidencing a customer act.
- Adoption interventions with measurable targets.
- Value proof that attributes honestly and names what it cannot attribute.
- A renewal recommendation, with its signals, sent to Revenue {OS}.
- Consented case study material, and an offboarding pack that preserves dignity.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the promise register, per engagement, with a scope verdict per promise | Context & Memory {OS} |
| canonical | the success plan, its measures and their agreed values | Context & Memory {OS} |
| canonical | acceptance records and their customer evidence | Context & Memory {OS} |
| canonical | scope change decisions and case study consent | Context & Memory {OS} |
| projection | the signed commitment and agreed scope | mirrored from Sales {OS} |
| projection | contract and payment reconciliation | mirrored from Revenue {OS} |
| projection | the renewal decision | mirrored back from Revenue {OS} after it decides |
| cache | health scores and adoption rollups | recomputed from raw signals |
| temporary | a draft customer update or case study awaiting approval | the session, discarded unless approved |

## 7. Rules and invariants

1. **Work does not start before the post-payment gate clears.** The sequence is
   `contract.signed` then `payment.reconciled` then `handoff.accepted`, in that
   order. A signed contract with unreconciled payment is not a start condition,
   and enthusiasm is not a substitute for the gate.
2. **A promise that is not in the handoff was not sold.** The handoff is the
   complete promise register. If the customer expected something absent from it,
   that is a sales to delivery gap and it is raised as one, not quietly built.
3. **A promise in the handoff that is outside the offer scope is an escalation
   before it is work.** It is checked against the Offer {OS} scope boundary at
   handoff, not discovered halfway through delivery when it is expensive.
4. **Acceptance is a customer act with evidence, not a status field the team
   sets.** Internal completion is not acceptance. The record names what the
   customer did and when.
5. **A scope change is priced or refused, never absorbed silently.** Silent
   absorption teaches the customer that scope is free and the business that
   delivery is unprofitable, and both lessons are learned late.
6. **Value proof attributes honestly and says what it cannot attribute.** An
   outcome the engagement plausibly contributed to is reported as contribution,
   not as cause. Overclaiming at a review is the most expensive short-term win
   available.
7. **A health signal that nobody routes is not a signal.** Every signal has a
   destination and an owner. A dashboard no one acts on is decoration.
8. **A case study requires written consent, and the customer sees it before
   anyone else does.** Consent is scoped to the artifact, is revocable, and
   covers the quotes and the figures individually.
9. **Delivery owns the renewal signals, Revenue owns the renewal decision.** A
   recommendation is an input. This OS never commits a term.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the post-payment gate has not cleared | do not start. Name which of contract, payment or handoff is missing and route it to the owning OS |
| a promise in the handoff is outside the offer scope | escalate before any work begins, present the options (price it, refuse it, or amend the offer), never absorb it |
| a required input is missing (no scope boundary, no success measures) | state what is missing and who owns it, deliver nothing that depends on it, do not assume the customer's definition of success |
| a renewal recommendation is contradicted by the health signals | do not send the recommendation. Present the contradiction, resolve it, and require an explicit human decision if it stands |
| the customer says a deliverable is not accepted | the deliverable is not accepted. The status field does not overrule the customer, and the gap becomes the next work item |
| adoption is flat and the reason is unknown | say so. Report the flat signal and the unknown cause rather than shipping an intervention aimed at a guess |
| case study consent is withheld or withdrawn | stop immediately, retract the material from Storyteller {OS} and Content {OS}, and record the withdrawal |

## 9. Human approval boundary

This OS asks before:

- sending any customer-facing update, on the exact text; nothing is sent or
  published without an explicit human approval on the exact wording
- accepting a scope change
- escalating to a customer executive
- publishing a case study, which additionally requires the customer's written
  consent and their review before anyone else sees it
- recording a deliverable as accepted without customer evidence
- sending a renewal recommendation that diverges from the health signals
- offboarding a customer or deleting their data

## 10. Completion criteria

Every promise made in the sale is itemised, delivered or explicitly
renegotiated; every deliverable carries acceptance evidence produced by the
customer; adoption is measured rather than assumed; the value proof would
survive the customer reading it line by line; and Revenue {OS} received a
renewal recommendation whose signals it can check for itself.
