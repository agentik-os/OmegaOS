# Offer {OS}: Operating Specification

## 1. Purpose

Define what is sold: the promise, the scope boundary, the deliberate
exclusions, the deliverables, the guarantee, the proof that makes the promise
credible, and the conditions under which the offer is retired.

An offer is where a claim becomes a commitment. Positioning {OS} says what you
stand for. Offer {OS} writes down what a customer receives for money, in terms
precise enough that Delivery & Customer Success {OS} can be held to them and
Sales {OS} cannot exceed them by accident.

## 2. Boundary

- **Owns:** the offer definition (promise, named outcome, deliverables), the
  scope boundary and its explicit exclusions, the guarantee and the cost
  ceiling that guarantee implies, the proof set required to make the promise
  credible, the offer lifecycle (draft, live, frozen, retired), and the
  migration path for customers sitting on a retired offer.
- **Does not own:** the price. **Offer {OS} owns what you sell, Pricing {OS}
  owns what you charge, and they are separate units on purpose:** an offer
  whose shape changes every time a number moves is not an offer, it is a
  negotiation. It also does not own fulfilment, which belongs to Delivery &
  Customer Success {OS}, and it does not own the category or the claim, which
  belong to Positioning {OS}.
- **Hands off to:** the `offer definition` to Pricing {OS}, Sales {OS},
  Revenue {OS}, Delivery & Customer Success {OS} and Content {OS}; the `scope
  boundary and guarantee` to Sales {OS} and Delivery & Customer Success {OS}.
- **Consumes from:** Positioning {OS} (the claim the offer must honour),
  Customer Discovery {OS} (the job to be done), Validation {OS} (evidence of
  demand), and Delivery & Customer Success {OS} (what fulfilment actually
  costs to deliver).

Requires: Positioning {OS}. An offer authored before the claim exists is a
guess about what somebody might buy.

The Delivery edge is the one people skip. An offer that cannot be fulfilled at
a sane cost is a liability with a signature on it, so fulfilment cost is a
declared input here, not a surprise found after the first ten sales.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `DRAFT` | a claim exists and a job to be done is named | a candidate offer definition | promise, outcome and deliverables are all written |
| `SCOPE` | a draft exists | the scope boundary and the exclusion list | at least one exclusion is stated and every deliverable has an acceptance form |
| `GUARANTEE` | scope is bounded | a guarantee with a modelled cost ceiling | the worst case cost is computed and affordable |
| `PROOF` | a promise is written | the proof set backing each promise | every promise maps to a proof item with a source |
| `STRESS` | the offer is complete but unpublished | an objection log and the revisions it forced | every objection is answered or the offer is changed |
| `PUBLISH` | stress test passed | a live, versioned offer definition | a human approved the exact text and the version is stamped |
| `RETIRE` | the offer is being withdrawn | a retirement record and a migration path | every live customer on it has a named destination |

`STRESS` is not optional. An offer that has never met a real objection has
only been read by people who already agree with it.

## 4. Inputs

- The positioning statement and the claim ledger from Positioning {OS}.
- The job to be done, in the customer's own words, from Customer Discovery
  {OS}.
- Demand evidence from Validation {OS}: what somebody actually paid for, tried
  to pay for, or refused.
- The fulfilment cost model from Delivery & Customer Success {OS}: hours,
  headcount, tooling and support load per unit sold.
- Objections gathered in real conversations, supplied by Sales {OS}.

## 5. Outputs

- An offer definition: promise, named outcome, deliverables, scope boundary,
  exclusions, guarantee, proof set, version and lifecycle state.
- A scope boundary artifact that Sales {OS} and Delivery & Customer Success
  {OS} can both check a specific request against, mechanically.
- A guarantee record: the condition, the remedy, the modelled worst case cost,
  and the ceiling above which it is withdrawn from new sales.
- A retirement record: reason, effective date, and a migration destination for
  every customer held on the retired offer.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the offer definitions and their lifecycle state | Context & Memory {OS} |
| canonical | guarantee records and their cost ceilings | Context & Memory {OS} |
| canonical | the objection log and the revisions it caused | Context & Memory {OS} |
| projection | the scope boundary Sales and Delivery check against | derived from the live offer definition |
| cache | the fulfilment cost model read from Delivery & Customer Success {OS} | refetched per review, never trusted across a version |
| temporary | drafts, candidate wordings, working notes | the session |

## 7. Rules and invariants

1. **An offer with no stated exclusion has an unbounded scope.** Silence is
   read by the buyer as inclusion and by the delivery team as scope creep. At
   least one exclusion is mandatory before an offer may leave `SCOPE`.
2. **A guarantee you cannot afford to honour is a lie with a delay.** It is
   modelled at its worst case, and its ceiling is written before publication.
3. **The offer names the outcome, not the activity.** "Twelve workshops" is an
   activity, "your team ships without me in the room" is an outcome, and
   buyers pay for the second while tolerating the first.
4. **Every promise becomes a Delivery obligation.** A promise that Delivery &
   Customer Success {OS} cannot check against the scope boundary is not a
   promise, it is an unpriced option the customer holds.
5. **The offer may not contradict the positioning claim.** If the claim says
   the category is speed and the offer sells a nine month engagement, one of
   them is wrong, and nothing publishes until Positioning {OS} resolves it.
6. **Fulfilment cost is an input, not a discovery.** An offer published
   without a cost model from Delivery is published blind, and the failure
   shows up as margin loss on the tenth customer, not as a flag on the first.
7. **Proof carries a source.** Every proof item names where it came from: a
   consented customer outcome, a measured result, a public artifact. An
   uncited proof item is removed, not softened.
8. **A live offer is versioned, never edited in place.** Customers bought a
   specific version, and its obligation survives the edit.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no positioning claim available | refuse to draft, name Positioning {OS} as the blocking unit, do not invent a claim |
| no fulfilment cost model from Delivery | draft, but block `PUBLISH` and state plainly that the economics are unverified |
| the offer contradicts the positioning claim | halt, present both statements side by side, escalate the contradiction to a human, change neither unilaterally |
| Delivery reports fulfilment cost exceeds the offer economics | freeze the offer for new sales, produce the three options (narrow the scope, change the guarantee, hand it to Pricing {OS}) and let a human pick |
| a guarantee whose worst case cost cannot be modelled | refuse the guarantee, output the abstention with the missing quantity named |
| a promise made in a sale is outside the scope boundary | reject it as out of scope, route the request to `DRAFT` as a candidate offer change, never absorb it silently |
| retirement requested while customers are live on the offer | refuse the retirement until every live customer has a named migration destination |

Abstention is a valid output. "The guarantee cannot be modelled because the
support load per customer is unknown" is a correct answer. A confident number
in its place is not.

## 9. Human approval boundary

Offer {OS} asks before:

- publishing an offer, or publishing any change to a live offer definition
- adding, changing or removing a guarantee
- retiring an offer that has live customers, including the migration path
- widening the scope boundary or removing an exclusion
- asserting a proof item that names a customer

No customer-facing offer text is published, sent or handed to Sales {OS} or
Content {OS} without an explicit human approval of the exact wording.

## 10. Completion criteria

A prospect can read the offer and know what they get, what they do not get,
what happens if it fails, and why the promise should be believed. Delivery &
Customer Success {OS} can take any specific request and answer in scope or out
of scope without asking a human. Nothing in the offer contradicts the claim,
and nothing costs more to fulfil than the business knew when it published it.
