# Renewal decision

Turn the signals Delivery & Customer Success {OS} produced into one commercial
decision with cash attached, and record why.

## Trigger

A contract term is approaching its end, at a lead time long enough for the
answer to be no, or Delivery & Customer Success {OS} publishes a renewal
recommendation ahead of that date.

## Steps

1. **Delivery & Customer Success {OS}** publishes the renewal recommendation
   with its adoption, health and acceptance signals. Produces: the signals and a
   recommendation.
2. **Revenue {OS}** loads the commercial record: contract terms, realised
   revenue, discount history, payment behaviour and receivable pattern.
   Produces: the commercial picture.
3. **Revenue {OS}** compares the two. It states plainly where the delivery
   signals and the commercial record agree and where they diverge. Produces: an
   agreement and divergence statement.
4. **Revenue {OS}** applies the current price book from Pricing {OS} to the
   renewal term, including any change since the original signature. Produces:
   renewal figures.
5. **Revenue {OS}** models the options as scenarios with ranges: renew as is,
   renegotiate, expand, or let go, each with its cash consequence. Produces:
   scenario set.
6. **Human** takes the decision. If the decision contradicts the delivery
   signals, the contradiction is stated explicitly and the reasoning is recorded
   with it. Produces: the decision and its reasoning.
7. **Revenue {OS}** writes the renewal decision to canonical state and emits it
   to Delivery & Customer Success {OS} and Sales {OS}, citing which signals it
   accepted and which it overrode. Produces: the emitted decision.
8. **Sales {OS}** or **Revenue {OS}** executes the decision: a renewal contract,
   a renegotiation, an expansion, or a wind-down handed back to Delivery for
   offboarding. Produces: the executed commercial change.
9. **Revenue {OS}** schedules the first invoice of the new term, which enters
   the invoice to cash workflow.

## Completion test

A renewal decision record exists before the contract end date, it names the
delivery signals it used, and where it diverges from the delivery
recommendation it carries an explicit contradiction statement with reasoning.
A term that ends with no decision record, or a decision that silently disagrees
with the signals, fails this workflow.

## Failure and abort

- Delivery signals are absent: do not decide on the commercial record alone.
  Request the signals, state the missing input, and hold the decision. If the
  date forces a call, record the decision as made without delivery evidence.
- The recommendation contradicts the health signals inside Delivery's own
  output: reject the recommendation, return it, and require the contradiction to
  be resolved before Revenue decides.
- The price book has changed and the customer was never told: stop. That is a
  human conversation, and pricing surprise at renewal is a trust event.
- The decision is let go: hand the wind-down to Delivery & Customer Success {OS}
  for offboarding, and confirm every open receivable is collected or written off
  before the relationship closes.
