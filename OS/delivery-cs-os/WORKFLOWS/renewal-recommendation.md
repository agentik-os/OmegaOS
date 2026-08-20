# Renewal recommendation

Assemble the health, adoption and value signals into one recommendation, and
hand the decision to Revenue {OS}.

## Trigger

A contract term approaches its end, at a lead time long enough for the answer to
be no. Also fires when health drops far enough that the renewal is in doubt
before the calendar says so.

## Steps

1. **Delivery & Customer Success {OS}** compiles the health signals: adoption,
   engagement, support load, sentiment, sponsor stability, and each one's raw
   source. Produces: the signal set.
2. **Delivery & Customer Success {OS}** compiles the value proof against the
   success plan measures, separating attributed outcomes from contributions and
   from what cannot be attributed at all. Produces: the value proof.
3. **Delivery & Customer Success {OS}** reviews the promise register: every
   promise delivered, renegotiated, or still open. An open promise at renewal is
   stated first, not buried. Produces: the promise status.
4. **Delivery & Customer Success {OS}** forms a recommendation: renew, expand,
   renegotiate, or let go. Produces: a draft recommendation.
5. **Delivery & Customer Success {OS}** tests the recommendation against its own
   signals. If the recommendation contradicts them, it is not sent: the
   contradiction is surfaced for resolution first. Produces: a consistency
   verdict.
6. **Human** approves the recommendation, and separately approves any
   recommendation that still diverges from the signals, recording why.
   Produces: approval and reasoning.
7. **Delivery & Customer Success {OS}** emits the renewal recommendation with
   its full signal set to Revenue {OS}. Produces: the emitted recommendation.
8. **Revenue {OS}** takes the renewal decision and returns it. Delivery owns the
   signals, Revenue owns the decision. Produces: the decision.
9. **Delivery & Customer Success {OS}** executes its half of the decision:
   continue delivery on the new term, prepare an expansion plan, support the
   renegotiation with evidence, or run offboarding.

## Completion test

Revenue {OS} holds a recommendation containing every signal it was based on,
sent before the contract end date, and either consistent with those signals or
carrying an explicit, approved statement of the divergence and its reasoning. A
term that ends with no recommendation on record, or a recommendation whose
signals contradict it silently, fails this workflow.

## Failure and abort

- The success plan measures were never agreed: there is no defensible value
  proof. Say so, recommend on health and delivery alone, and state the gap as a
  limitation of the recommendation.
- Adoption is flat and the cause is unknown: report the flat signal and the
  unknown cause. Do not recommend renewal on the strength of a relationship
  while the usage evidence says nothing.
- The recommendation is contradicted by the health signals: stop. Resolve the
  contradiction or send it with an explicit approved divergence. Never smooth it.
- An open promise remains from the original sale: it goes at the top of the
  recommendation. Renewing over an undelivered promise buys one term and costs
  the relationship.
- The decision is let go: hand the engagement to offboarding, and confirm with
  Revenue {OS} that every open receivable is resolved before the relationship
  closes.
