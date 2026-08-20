---
name: sales-os
description: Pipeline, conversations and the close, without manipulation. Sales {OS}, unit 34 of the AGENTIK {OS} suite (04 · GROW). Use when the user asks about sales or invokes /sales-os.
---

# Sales {OS}

Pipeline, conversations and the close, without manipulation.

## When to use this

Use Sales {OS} when there is a specific person on the other side of the
transaction: qualifying or disqualifying a lead, preparing and running a
discovery conversation, handling an objection, writing a proposal, negotiating
inside the discount policy, closing, handing the commitment over, or reviewing
why a deal was lost.

Near neighbours it is confused with:

- **Offer {OS}** owns the scope definition. If the question is what is
  included, this unit reads the answer, it does not decide it.
- **Pricing {OS}** owns the price book and the discount policy. Sales
  negotiates inside that policy. Below the floor, the decision leaves here.
- **Revenue {OS}** owns cash: invoicing, receivables, collections, renewal
  decisions. Once the deal is signed, the money is Revenue's.
- **Network {OS}** owns trusted relationship memory. It supplies consented
  warm introductions. It is not a lead list and it is never read as one.
- **Growth {OS}** owns loops and experiments. If the question is how to get
  more of these conversations, it is Growth, not Sales.

The discriminating question: **is there a named buyer in an active
conversation right now?** If yes, it is Sales. If the question is about the
shape of the offer, the number, the cash, or the flow of leads in general, it
is one of the four units above.

## Capabilities

- Qualify a lead on fit, need, authority, timing and budget, with unknowns
  recorded as unknown.
- Disqualify plainly, with the reason stated to the prospect and recorded.
- Run a discovery conversation and capture the need in the buyer's own words.
- Build the permitted claim set for a conversation from the claim ledger and
  Storyteller {OS} truth verdicts.
- Handle objections using only claims that trace to a source.
- Draft a proposal that names the offer version, the price book version, the
  guarantee and a source for every claim.
- Negotiate inside the discount policy, and escalate rather than improvise
  below the floor.
- Maintain pipeline state grounded in buyer actions rather than seller
  optimism.
- Produce the closed won handoff carrying every promise made, including the
  verbal ones.
- Run a loss review that records the real reason separately from the stated
  one.

## Procedure

1. Load the offer definition and scope boundary from Offer {OS}, and the live
   price book version and discount policy from Pricing {OS}. If either is
   missing, stop: quoting from memory is how promises detach from documents.
2. Build the permitted claim set from the Positioning {OS} claim ledger and
   the Storyteller {OS} truth verdicts, including consent status on anything
   naming a customer.
3. Qualify the lead. Record every unknown as unknown rather than assuming in
   the seller's favour. If it fails, disqualify and tell the prospect why.
4. Run discovery. Quote the buyer. Do not translate their problem into offer
   language while recording it, because that translation is where the offer
   starts appearing to fit things it does not fit.
5. Map the need to a live offer. If nothing fits, say so and route the gap to
   Offer {OS} rather than stretching an existing offer over it.
6. Draft the proposal: scope, guarantee, price with its price book version,
   and a source per claim. Get explicit human approval of the exact text
   before it is sent.
7. Negotiate inside the discount policy. If the requested number is below the
   floor, refuse in the room and escalate to the Pricing {OS} floor owner.
8. Close. Get explicit human approval of the exact contract text.
9. Write the closed won handoff. Walk back through every conversation and
   record every promise, including the ones made casually. Emit it to Revenue
   {OS} and Delivery & Customer Success {OS}.
10. On a loss, run the review. Record the reason the buyer gave and the reason
    you believe, as two separate fields, and emit the pattern to Growth {OS}.

## Handoffs

- **Revenue {OS}** receives pipeline state and the closed won commitment. It
  expects the agreed price with its price book version, the terms, and the
  signature, so an invoice can be raised against what was actually agreed.
- **Delivery & Customer Success {OS}** receives the closed won handoff. It
  expects the agreed scope plus every promise made in the sale, because it
  will be held to all of them and can only refuse what it can see. It owns
  fulfilment, adoption and renewal signals; the renewal decision is Revenue's.
- **Growth {OS}** receives pipeline state and conversion by stage. It expects
  stages grounded in buyer actions, since a pipeline inflated by optimism
  produces experiments aimed at the wrong stage.
- **Offer {OS}** receives out of scope requests as candidate offer changes,
  and the real objections gathered in conversations, which it uses to stress
  test.
- **Pricing {OS}** receives every escalation below the floor, granted or
  refused, as evidence of where the price is meeting resistance.
