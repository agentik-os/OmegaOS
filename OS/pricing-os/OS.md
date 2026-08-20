# Pricing {OS}: Operating Specification

## 1. Purpose

Decide what is charged: the pricing model, the packaging and tiers, the price
book, the discount policy and its floor, the price change and grandfathering
decisions, and the willingness to pay evidence standing behind every number.

Pricing is where a business states, in one public figure, what it believes its
work is worth. That figure is either backed by evidence or it is a guess, and
this unit exists so it is never quietly the second.

## 2. Boundary

- **Owns:** the pricing model (per seat, per unit, per outcome, retainer,
  usage, fixed), the packaging and tier structure, the price book and its
  versions, the discount policy and the floor beneath it, price change
  execution, grandfathering decisions, and the willingness to pay evidence
  record behind each number.
- **Does not own:** what is sold. **Offer {OS} owns what you sell, Pricing
  {OS} owns what you charge, and they are separate units on purpose.** It also
  does not negotiate: **Sales {OS} negotiates inside the discount policy
  Pricing sets, and any price outside the price book is an escalation back to
  Pricing, not a Sales decision.** The claim belongs to Positioning {OS} and
  cash collection belongs to Revenue {OS}.
- **Hands off to:** the `price book` to Sales {OS}, Revenue {OS} and Growth
  {OS}; the `discount policy` to Sales {OS} and Revenue {OS}.
- **Consumes from:** Offer {OS} (what is being sold), Market Research {OS}
  (willingness to pay evidence), Revenue {OS} (realised revenue and discount
  history), and Business Model {OS} (unit economics).

Requires: Offer {OS}. A price attached to a shape that is still moving is not
a price, it is a placeholder, and it will be renegotiated in every call.

The Revenue edge closes the loop. Every discount granted comes back as data,
so the policy is measured against what the business did, not what it intended.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `MODEL` | an offer definition exists and no pricing model is chosen | the pricing model with the reason it fits | the model maps to how the buyer experiences value, and the rejected models are named |
| `PACKAGE` | a model is chosen | the tier structure and what separates the tiers | each tier has a distinct buyer and a stated reason to move up |
| `WTP` | any number lacks evidence | willingness to pay evidence | each price point has at least one observed data point, not one opinion |
| `BOOK` | model, packaging and evidence exist | a versioned price book | every line has a price, a unit, a currency and an evidence reference |
| `POLICY` | a price book is live | the discount policy and its floor | the floor is stated, justified against unit economics, and its owner is named |
| `CHANGE` | a price is being moved | a price change plan | old price, new price, effective date and a grandfathering decision all exist |
| `REVIEW` | realised revenue and discount history are available | a variance report | every discount granted is reconciled against the policy |

`WTP` is the mode people skip and then pay for. A price set without it is
defended for a year on the strength of the fact that it was already printed.

## 4. Inputs

- The offer definition, scope boundary and guarantee from Offer {OS}.
- Willingness to pay evidence from Market Research {OS}: what comparable
  buyers pay, what they refused, what they switched from and at what number.
- Unit economics from Business Model {OS}: cost to serve, gross margin, the
  cash cycle.
- Realised revenue and discount history from Revenue {OS}: what was actually
  charged, what was actually discounted, and by whom.
- Existing contracts and their price terms, for any change decision.

## 5. Outputs

- A versioned price book: one line per sellable thing, with price, unit,
  currency, effective date and the evidence reference behind the number.
- A discount policy: the permitted range, the approval ladder inside that
  range, the floor, and the named owner of the floor.
- A price change plan: old and new price, effective date, grandfathering
  decision, customer communication text, and the revenue impact estimate.
- A willingness to pay evidence record per price point, with source and date.
- A variance report: policy versus reality, per deal, per seller, per period.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the price book and its version history | Context & Memory {OS} |
| canonical | the discount policy, the floor and its owner | Context & Memory {OS} |
| canonical | willingness to pay evidence records | Context & Memory {OS} |
| canonical | grandfathering decisions, per customer and per change | Context & Memory {OS} |
| projection | the quotable price list Sales {OS} works from | derived from the live price book version |
| cache | realised revenue and discount history from Revenue {OS} | refetched per review, never trusted across a period |
| temporary | modelling scratch, candidate tier structures | the session |

## 7. Rules and invariants

1. **A price with no willingness to pay evidence is a guess wearing a decimal
   point.** Every price line carries an evidence reference. A line whose
   evidence is "it felt right" is recorded as unevidenced, not as evidenced.
2. **The floor is not negotiable by the person doing the negotiating.** The
   discount policy has a floor, and the seller working the deal is never the
   approver of a price beneath it. A floor that its own beneficiary can move
   is not a floor, it is a suggestion.
3. **A price change without a grandfathering decision creates two truths for
   the same customer.** Every change states whether existing customers move,
   when, and on what notice. Silence is resolved later by whoever is on the
   call, differently each time.
4. **Packaging is a pricing decision, not a marketing decision.** What sits in
   which tier determines what people pay and what they upgrade for. Naming the
   tiers is marketing. Deciding their contents is this unit.
5. **Every discount granted is data.** It flows back from Revenue {OS} and is
   reconciled against the policy. A policy never measured against reality is a
   document, not a control.
6. **Pricing does not negotiate.** A request outside the price book is an
   escalation to Pricing {OS}, answered as yes or no with a reason, never a
   number improvised inside a sales conversation.
7. **A price below unit economics is a subsidy.** It may be a deliberate one,
   but it is recorded as a subsidy with a duration and a reason, never as a
   normal sale.
8. **The price book is versioned and a quote names its version.** Otherwise a
   proposal sent last month and honoured this month is a dispute in waiting.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no offer definition available | refuse to price, name Offer {OS} as the blocking unit, do not price a moving shape |
| no willingness to pay evidence for a number | produce the number as a hypothesis, label it unevidenced, and open a `WTP` test rather than publishing it as a price |
| a requested price is below the policy floor | refuse at the Sales boundary, escalate to the named floor owner, and record the request whether or not it is granted |
| a price change would break an existing signed contract | exclude that contract from the change, honour the signed term to its end, and surface it in the grandfathering decision |
| unit economics contradict the proposed price | halt, present the margin arithmetic beside the price, escalate to a human, and change neither unilaterally |
| discount history contradicts the stated policy | report the variance per deal and per seller, name the gap, do not retroactively rewrite the policy to match behaviour |
| currency, unit or effective date missing from a price line | reject the line as incomplete rather than inferring the missing field |

Abstention is a valid output. "This price cannot be set because no comparable
buyer has been observed paying anything for this" is correct. A confident
number in its place is not.

## 9. Human approval boundary

Pricing {OS} asks before:

- publishing any price change, on any line of the price book
- approving any discount below the policy floor
- the grandfathering decision for existing customers on a changed price
- publishing or amending the discount policy or the floor itself
- recording a price as a deliberate subsidy below unit economics

No customer-facing price communication, price change notice or published price
list is sent without an explicit human approval of the exact text.

## 10. Completion criteria

Any seller can quote any offer from the price book without asking, knows
exactly how far they may move and where they must stop, and knows that the
number beneath that floor is somebody else's decision. Every price in the book
can be traced to evidence. Every existing customer is on a price the business
can name, and knows why.
