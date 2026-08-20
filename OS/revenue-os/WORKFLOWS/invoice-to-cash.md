# Invoice to cash

Take a billable event all the way to reconciled cash, with a human approving the
document that leaves the building.

## Trigger

A deliverable is accepted, a milestone completes, or a billing period ends for a
customer whose contract and agreed scope are already staged in this OS.

## Steps

1. **Delivery & Customer Success {OS}** publishes the acceptance record for the
   deliverable or milestone. Produces: acceptance evidence.
2. **Revenue {OS}** resolves the billable amount from the price book published by
   Pricing {OS} and the scope published by Offer {OS}. It does not compute a
   price of its own. Produces: the billable line items.
3. **Revenue {OS}** checks the line items against the signed contract terms:
   currency, payment terms, tax treatment, purchase order reference. Produces: a
   conformance check or a named discrepancy.
4. **Revenue {OS}** drafts the invoice with every figure traceable to a record.
   Produces: an unissued invoice draft.
5. **Human** approves the exact figures, the exact line items and the exact
   recipient. An invoice is a legal document, so nothing is issued before this.
   Produces: approval.
6. **Revenue {OS}** issues the invoice, records it as billed, and starts the
   ageing clock from the contract terms rather than from the send date view.
   Produces: an issued invoice and a due date.
7. **Revenue {OS}** books the amount as billed revenue, and reports it as billed,
   never as cash. Produces: a labelled ledger entry.
8. **Revenue {OS}** watches for payment and matches each receipt to an invoice.
   An unmatched payment becomes unapplied cash on the exception list, never a
   guessed allocation. Produces: a reconciliation or an exception.
9. **Revenue {OS}** marks the invoice paid, moves the amount from billed to cash
   received, and emits the updated cash position and receivable status to KPI &
   Analytics {OS}, Business Strategy {OS} and Growth {OS}.

## Completion test

The invoice exists with an approval record naming a human and a timestamp
earlier than the issue time, and the payment is matched to that invoice by
amount and reference. Cash received equals the sum of matched payments, and no
invoice is marked paid without a matched receipt. An invoice issued with no
prior approval record fails this workflow even if the customer paid it.

## Failure and abort

- The price book or the agreed scope is missing: stop before drafting. Name the
  missing input and the OS that owns it. Do not infer a figure.
- The line items contradict the signed contract: halt, present both, and route
  any contract change through explicit human approval before proceeding.
- Approval is not given: the invoice stays a draft indefinitely. There is no
  timeout that issues it.
- A payment arrives that matches nothing: hold it as unapplied cash and list it
  as an exception. It blocks the close until resolved.
- The customer disputes the invoice: freeze collections on it, record the
  dispute with its date, and route the resolution to an explicit decision.
