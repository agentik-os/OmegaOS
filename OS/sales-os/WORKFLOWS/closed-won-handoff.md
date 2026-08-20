# Closed won handoff

Produces the single artifact that carries a signed deal out of Sales {OS}: the
agreed scope, the price and terms, and every promise made in every
conversation, including the ones made verbally and casually.

## Trigger

A deal reaches closed won. The workflow runs immediately, before the details
of the conversations decay.

## Steps

1. **Sales {OS}** attaches the signed commitment: the exact contract text a
   human approved, the signature and its date. Produces the legal spine of the
   handoff.
2. **Sales {OS}** attaches the offer version sold and the price book version
   quoted. Produces the two references that make the scope and the number
   resolvable months later without asking anybody.
3. **Sales {OS}** walks back through every recorded conversation on the
   opportunity: call briefs, need records, objection responses, proposal
   revisions and email. Produces the promise list, one line per promise, with
   the conversation and date it was made in.
4. **Sales {OS}** classifies each promise against the scope boundary from
   Offer {OS}: inside scope, outside scope, or ambiguous. Produces a
   classified promise list. Nothing is deleted at this step, including the
   promises that should never have been made.
5. **Offer {OS}** adjudicates every ambiguous promise. Produces a binding in
   scope or out of scope ruling per line.
6. **Sales {OS}** escalates every out of scope promise to a human, with the
   two honest options: honour it as a recorded exception with a cost, or go
   back to the customer and correct it before delivery begins.
7. **A human** approves the handoff, including the disposition of every out of
   scope promise.
8. **Revenue {OS}** receives the commitment for invoicing and reconciliation.
   **Delivery & Customer Success {OS}** receives the scope, the guarantee and
   the full promise list, and fulfilment begins from that document.

## Completion test

The handoff is complete when:

- the signed contract text attached is byte identical to the text approved
- the offer version and the price book version are both named and both resolve
  to a published version
- every promise found in step 3 appears in the handoff, classified, with none
  removed
- no promise is left classified ambiguous
- every out of scope promise has a recorded human disposition
- Revenue {OS} confirms the price and terms match the signed commitment
- Delivery & Customer Success {OS} confirms it can answer in scope or out of
  scope for every line of the promise list without asking Sales

The last condition is the whole point. If Delivery has to phone the seller to
learn what was agreed, the handoff did not happen, a conversation did.

## Failure and abort

- **A promise is missing from the handoff.** Block the handoff. Do not treat
  it as an administrative detail to fix later: a promise made in a call and
  not written down here is the single most common source of delivery failure,
  and it always surfaces at the worst moment.
- **An out of scope promise was made.** Do not absorb it silently and do not
  quietly drop it. Escalate both options to a human and record which one was
  chosen. Silence here is a decision made by whoever is on the next call.
- **The price does not match the price book version quoted.** Halt. Revenue
  {OS} does not invoice against an unexplained number. Reconcile with Pricing
  {OS} and, if the number came from below the floor, record the escalation
  that authorised it.
- **The contract text differs from the approved text.** Halt and escalate. No
  customer-facing document leaves Sales {OS} without an explicit human
  approval of the exact text, and a post approval edit voids that approval.
- **Human approval withheld.** The deal stays closed won in Sales {OS} but
  nothing is emitted. Delivery does not begin from an unapproved handoff.
