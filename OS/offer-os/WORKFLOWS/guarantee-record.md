# Guarantee record

Produces a guarantee whose worst case cost has been computed before it is
offered to anybody, together with the ceiling above which it is withdrawn from
new sales.

## Trigger

A guarantee is being added, changed or removed on an offer, or the fulfilment
cost model from Delivery & Customer Success {OS} changed by enough to move the
worst case.

## Steps

1. **Offer {OS}** states the guarantee condition: the precise, observable
   event that entitles a customer to the remedy. Produces a condition
   statement with no evaluative words in it.
2. **Offer {OS}** states the remedy: refund, rework, extension, credit, and
   the window in which it can be claimed. Produces a remedy statement.
3. **Delivery & Customer Success {OS}** supplies the fulfilment cost per unit
   and the historical rate at which the condition has actually occurred.
   Produces a cost figure and an incidence rate, or an explicit unknown.
4. **Offer {OS}** computes the worst case cost: every eligible customer in the
   claim window invoking the remedy at once. Produces a worst case figure, not
   an expected value.
5. **Offer {OS}** compares the worst case against available cash, using the
   cash position from Revenue {OS}. Produces an affordable or not affordable
   verdict.
6. **Offer {OS}** writes the ceiling: the incidence rate, claim volume or cash
   position at which the guarantee is withdrawn from new sales. Produces a
   ceiling with a named trigger metric and an owner.
7. **A human** approves the guarantee text, the remedy and the ceiling.
   **Offer {OS}** attaches the guarantee to the offer version and emits it to
   Sales {OS} and Delivery & Customer Success {OS}.

## Completion test

The guarantee record is complete when:

- the condition is observable and could be adjudicated by a third party
  reading it, with no judgement call left to the seller
- the remedy names an amount or an action and a claim window
- a worst case cost figure exists, and it is arithmetically derived from the
  incidence rate and the fulfilment cost, both dated
- the worst case is affordable against the current cash position, or the
  guarantee is not published
- a ceiling exists with a named trigger metric and a named owner
- a human approval of the exact guarantee wording is recorded
- Sales {OS} and Delivery & Customer Success {OS} both hold the same version

## Failure and abort

- **Incidence rate unknown.** Abstain. Output the guarantee as unmodellable
  and name the missing quantity. Do not substitute an assumed rate: a
  guarantee you cannot afford to honour is a lie with a delay, and an invented
  incidence rate is exactly how one gets published.
- **Worst case exceeds available cash.** Refuse to publish. Offer the two
  legitimate paths: reduce the remedy, or narrow the condition. Do not publish
  a guarantee that is affordable only if few people claim it.
- **Cost model absent from Delivery.** Abort at step 3, name Delivery &
  Customer Success {OS} as the blocking unit.
- **Guarantee change requested on a live offer with existing customers.** The
  change applies to new sales only. Existing customers keep the guarantee
  attached to the version they bought, and the workflow produces both records
  rather than overwriting one.
- **Human approval withheld.** No guarantee is attached, and the offer
  publishes without one rather than with a draft.
