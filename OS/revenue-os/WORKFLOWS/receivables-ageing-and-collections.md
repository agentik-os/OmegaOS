# Receivables ageing and collections

Age every receivable from the calendar, act on each one, and never let silence
be mistaken for an absence of debt.

## Trigger

The collections cadence fires (weekly by default), or an invoice passes its
payment terms. The ageing itself runs whether or not anybody triggers it: a
receivable ages on the calendar, not on attention.

## Steps

1. **Revenue {OS}** recomputes days outstanding for every open invoice from its
   contractual terms and today's date. Produces: the ageing view.
2. **Revenue {OS}** buckets each receivable: current, overdue, materially
   overdue, and at risk. Produces: buckets with a per-customer total.
3. **Revenue {OS}** checks each overdue invoice against known disputes,
   unapplied cash and delivery state. An invoice for work that Delivery flagged
   as not accepted is pulled out of collections and escalated instead.
   Produces: a filtered collections list plus an escalation list.
4. **Revenue {OS}** proposes one action per invoice: chase, reschedule with new
   terms, escalate, or write off. Produces: an action per line, with a reason.
5. **Revenue {OS}** drafts the message for each chase, respectful, factual, and
   escalating only in firmness with age. Produces: unsent messages.
6. **Human** approves the exact text of every message that will be sent, and
   separately approves any write-off. A write-off is never bundled into a batch
   approval. Produces: approvals, per item.
7. **User** sends the approved messages. **Revenue {OS}** records the contact
   date against the invoice. Produces: a contact history per receivable.
8. **Revenue {OS}** applies any payment received to its invoice and removes it
   from the ageing view. Produces: the reconciled receivable.
9. **Revenue {OS}** emits the updated receivable status to KPI & Analytics {OS},
   Business Strategy {OS} and Growth {OS}, and flags any customer whose
   receivable pattern should inform the renewal decision.

## Completion test

Every open invoice carries a days-outstanding figure computed from its terms,
a bucket, and exactly one recorded action for this cycle. No invoice sits in the
ageing view with no action and no reason. Any receivable removed from the view
is removed by a matched payment, an approved reschedule, or an approved
write-off, never by editing the view.

## Failure and abort

- Payment terms are missing from the contract record: age from the invoice date,
  mark the ageing provisional, and raise the missing term as a blocker.
- Unapplied cash exists that could match an overdue invoice: stop before
  chasing. Chasing a customer who has already paid costs more than a day of
  delay. Resolve the allocation first.
- Delivery has not accepted the work the invoice bills for: abort collections on
  that invoice and escalate to a human. It is a delivery conversation, not a
  payment conversation.
- Approval on a message text is withheld: nothing is sent for that invoice this
  cycle, and the invoice stays in the view with the reason recorded.
- A write-off is proposed without an explicit decision: it does not happen. The
  receivable stays open and continues to age.
