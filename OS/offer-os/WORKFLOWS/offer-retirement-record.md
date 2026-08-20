# Offer retirement record

Produces the record that withdraws an offer from sale and gives every customer
still held on it a named destination.

## Trigger

An offer is being discontinued or replaced, or `/offer-cost-check` returned
not viable and the business chose withdrawal over revision.

## Steps

1. **Offer {OS}** marks the offer `frozen`. Produces an immediate stop on new
   sales, emitted to Sales {OS} and Content {OS}. Freezing precedes retiring,
   always, so no proposal goes out while the migration is being designed.
2. **Revenue {OS}** supplies the list of live customers on the offer, with
   contract end dates and payment status. Produces the population to migrate.
3. **Delivery & Customer Success {OS}** supplies the delivery state of each
   live customer: in flight, complete, or at risk. Produces the constraint set
   for the migration.
4. **Offer {OS}** names a destination per customer: migrate to a named live
   offer, run to contract end on the retired version, or exit with terms.
   Produces the migration table, one row per customer, no row left blank.
5. **Offer {OS}** states what happens to the guarantee attached to the retired
   version. Produces an explicit guarantee disposition per row, because the
   obligation attached to the version sold survives the retirement.
6. **A human** approves the retirement, the effective date and the migration
   table, row by row.
7. **Sales {OS}** and **Delivery & Customer Success {OS}** receive the
   approved record. **Offer {OS}** marks the offer `retired` and emits the
   updated offer set.

## Completion test

The retirement is complete when:

- the offer is `retired` and no live proposal references it
- every live customer from step 2 appears exactly once in the migration table
- every migration row names a destination and a guarantee disposition
- the effective date is in the future relative to the approval timestamp
- Sales {OS} confirms no in flight proposal quotes the retired offer
- Delivery & Customer Success {OS} confirms no in flight engagement lost its
  scope boundary in the migration

A retirement with one unassigned customer is not complete. It is a customer
holding an obligation nobody owns.

## Failure and abort

- **A live customer has no viable destination.** Refuse to retire. Keep the
  offer `frozen` and escalate the specific customer to a human. Freezing
  stops the bleeding; retiring without a destination creates an orphan.
- **Revenue cannot produce the live customer list.** Abort at step 2, name
  Revenue {OS} as the blocking unit. Retiring an offer against an unknown
  population is not a decision, it is a hope.
- **An in flight proposal quotes the offer.** Hold the retirement until Sales
  {OS} has withdrawn or re-quoted it. The prospect saw a specific version.
- **Guarantee still claimable after the effective date.** The guarantee window
  outlives the retirement. Record the residual liability and hand it to
  Revenue {OS} rather than closing the record.
- **Human approval withheld.** The offer stays `frozen`, not `live`, and not
  `retired`. Frozen is a legitimate resting state; a half executed retirement
  is not.
