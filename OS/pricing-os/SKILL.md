---
name: pricing-os
description: What to charge, how to package it and when to change it. Pricing {OS}, unit 32 of the AGENTIK {OS} suite (04 · GROW). Use when the user asks about pricing or invokes /pricing-os.
---

# Pricing {OS}

What to charge, how to package it and when to change it.

## When to use this

Use Pricing {OS} when the question resolves to a number: choosing a pricing
model, deciding what goes in which tier, building or versioning a price book,
gathering willingness to pay evidence, setting a discount policy and its
floor, executing a price increase, or deciding whether existing customers are
grandfathered.

Near neighbours it is confused with:

- **Offer {OS}** owns what you sell. If the question is what the customer
  receives, what is excluded or what is guaranteed, it is Offer.
- **Sales {OS}** negotiates inside the policy this unit sets. If the question
  is how to handle a specific prospect pushing on price, it is Sales, right up
  until the number leaves the price book, at which point it returns here.
- **Revenue {OS}** owns invoicing, collection and cash. If the money has
  already been agreed and the question is getting paid, it is Revenue.
- **Business Model {OS}** owns unit economics. If the question is whether the
  business can work at all at these margins, it is Business Model.

The discriminating question: **is this deciding a number and the rules around
it, or is it applying a number that is already decided?** Only the first is
Pricing.

## Capabilities

- Choose a pricing model and name the models rejected and why.
- Design tiers where each tier has a distinct buyer and a reason to move up.
- Build a versioned price book where every line carries evidence.
- Design and read a willingness to pay test against real buyers.
- Set a discount policy, its approval ladder, its floor and the floor's owner.
- Adjudicate a discount request against the policy, including refusal.
- Plan and execute a price change with an explicit grandfathering decision.
- Model the revenue impact of a change before it is announced.
- Reconcile realised discounts from Revenue {OS} against the stated policy.
- Flag any price sitting below unit economics as a named subsidy.

## Procedure

1. Read the offer definition from Offer {OS}. If the offer is still a draft,
   stop: a price on a moving shape is renegotiated in every call.
2. Read unit economics from Business Model {OS} and establish the cost to
   serve per unit sold.
3. Choose the pricing model by asking how the buyer experiences value, then
   record the models rejected and the reason for each.
4. Design the tiers. Each one gets a named buyer and a stated reason to move
   up from the tier below.
5. For every price point, find willingness to pay evidence in Market Research
   {OS}. Where there is none, open a test rather than publishing a guess.
6. Assemble the price book: price, unit, currency, effective date, evidence
   reference. Reject any line missing a field.
7. Set the discount policy: the permitted range, the approval ladder, the
   floor, and the floor's owner, who is never the person negotiating.
8. Publish the price book and the policy to Sales {OS}, Revenue {OS} and
   Growth {OS}, after human approval of the exact text.
9. On a cadence, pull realised revenue and discount history from Revenue {OS}
   and produce the variance report.
10. When a price moves, produce the change plan with the grandfathering
    decision, model the revenue impact, and get human approval before any
    customer is told.

## Handoffs

- **Sales {OS}** receives the price book and the discount policy. It expects a
  quotable number per offer line and an unambiguous floor, so a seller can
  answer without escalating inside the permitted range and knows exactly where
  they must stop.
- **Revenue {OS}** receives the price book and the discount policy so an
  invoice can be reconciled against what should have been charged. It returns
  realised revenue and discount history, which this unit consumes.
- **Growth {OS}** receives the price book as a variable it may propose
  experiments against. It does not change a price; it proposes one, and the
  decision stays here.
- **Offer {OS}** receives escalations when the honest answer to a pricing
  problem is that the shape of the thing sold has to change.
