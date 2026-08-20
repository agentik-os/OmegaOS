# Post-payment handoff

Move a signed commitment into delivery through the gate, with every promise made
in the sale itemised and checked against the offer scope before anyone builds
anything.

## Trigger

Sales {OS} reports a closed won commitment. The trigger starts the gate check;
it does not start the work.

## Steps

1. **Revenue {OS}** confirms `contract.signed`. Produces: the contract record.
2. **Revenue {OS}** confirms `payment.reconciled`, meaning cash received and
   matched, not an invoice issued. Produces: the payment reconciliation.
3. **Delivery & Customer Success {OS}** verifies the gate sequence is complete
   and in order. If any link is missing it stops here and names it. Produces: a
   gate verdict.
4. **Sales {OS}** transfers the signed commitment, the agreed scope, and every
   promise made during the sale, including the ones made verbally. Produces: the
   raw handoff.
5. **Delivery & Customer Success {OS}** itemises each promise into the promise
   register, one row per promise, each carrying its source in the sale.
   Produces: the promise register.
6. **Delivery & Customer Success {OS}** checks every promise against the scope
   boundary published by Offer {OS}. Each promise gets one of three verdicts: in
   scope, outside scope, or ambiguous. Produces: a scope verdict per promise.
7. **Delivery & Customer Success {OS}** escalates every outside-scope and
   ambiguous promise to a human before any work is scheduled, with three
   options: price it as a change, refuse it and tell the customer, or amend the
   offer. Produces: an escalation set.
8. **Human** resolves each escalation. Produces: a decision per promise.
9. **Delivery & Customer Success {OS}** records `handoff.accepted` only when
   every promise has a verdict and every escalation has a decision, and names
   the delivery owner and the customer owner. Produces: the accepted handoff.
10. **Delivery & Customer Success {OS}** emits the accepted handoff to Revenue
    {OS} so the commercial record and the delivery obligation agree, and opens
    onboarding.

## Completion test

`handoff.accepted` exists, its timestamp is later than both `contract.signed`
and `payment.reconciled`, and the promise register has zero rows without a scope
verdict and zero outside-scope rows without a recorded human decision. Any work
item whose creation time precedes `handoff.accepted` is a gate breach and fails
this workflow regardless of the outcome of the engagement.

## Failure and abort

- Payment is not reconciled: hold. The contract alone is not a start condition.
  Report which link is missing and route it to Revenue {OS}.
- The customer expects something absent from the handoff: it was not sold. Raise
  it as a sales to delivery gap, do not quietly add it to the plan.
- A promise sits outside the offer scope: escalate before it is work. Delivering
  it first and negotiating later converts a commercial question into a
  relationship problem.
- The scope boundary from Offer {OS} is missing: every promise is ambiguous.
  Stop, request the boundary, and do not substitute a delivery opinion for it.
- Sales cannot produce the verbal promises: record that the register is
  incomplete, name it in the first customer conversation, and treat any
  surfacing promise as an escalation rather than a discovery.
