# Price book

Produces the versioned, quotable list of every sellable thing, where each line
carries a price, a unit, a currency, an effective date and the evidence
standing behind the number.

## Trigger

An offer definition has been published by Offer {OS} and has no price, or the
existing price book has drifted far enough from evidence that it is being
rebuilt rather than amended.

## Steps

1. **Offer {OS}** supplies the published offer definitions, their scope
   boundaries and their guarantees. Produces the set of sellable things. A
   draft offer is excluded: pricing a moving shape produces a placeholder.
2. **Business Model {OS}** supplies unit economics: cost to serve per unit,
   gross margin target, cash cycle. Produces the floor arithmetic that the
   discount policy will later be built on.
3. **Pricing {OS}** chooses the pricing model per offer and records the models
   rejected with reasons. Produces the model decision record.
4. **Pricing {OS}** designs the tier structure. Produces tiers, each with a
   named buyer and a stated reason to move up. A tier with no distinct buyer
   is collapsed into its neighbour rather than shipped.
5. **Market Research {OS}** supplies willingness to pay evidence per price
   point. Produces observations: what comparable buyers paid, refused, or
   switched from.
6. **Pricing {OS}** assembles the book. Produces one line per sellable thing
   with price, unit, currency, effective date and evidence reference. Any line
   missing a field is rejected, not completed by inference.
7. **Pricing {OS}** marks every line whose evidence is an opinion rather than
   an observation as unevidenced, and opens a `/price-test` for each.
8. **A human** approves the exact price book text. **Pricing {OS}** stamps the
   version and emits the book to Sales {OS}, Revenue {OS} and Growth {OS}.

## Completion test

The price book is complete when, by reading the artifact alone:

- every published offer from step 1 appears exactly once
- no line is missing a price, a unit, a currency or an effective date
- every line carries an evidence reference, or is explicitly flagged
  unevidenced with an open test attached
- every price is at or above the cost to serve from step 2, or is recorded as
  a named subsidy with a duration and a reason
- every tier names its buyer and its reason to move up
- a human approval of the exact text is recorded with a timestamp
- a version stamp exists and Sales {OS}, Revenue {OS} and Growth {OS} all hold
  that same version

A book where the count of unevidenced lines is unknown has not passed. The
count may be greater than zero; it may not be unmeasured.

## Failure and abort

- **Offer still in draft.** Exclude that offer from the book and continue with
  the rest. Report it as unpriced with Offer {OS} named as the blocking unit.
- **No unit economics.** Abort at step 2. A book built without cost to serve
  cannot produce a defensible floor, and a floor invented later is a number
  nobody will hold.
- **No willingness to pay evidence for a price point.** Do not stop the book.
  Publish the line flagged unevidenced with an open test, so the gap is
  visible rather than laundered into a printed number.
- **A proposed price sits below cost to serve.** Halt that line, present the
  margin arithmetic beside the price, escalate to a human. If it is approved,
  it is recorded as a subsidy with a duration, never as a normal sale.
- **Human approval withheld.** No version is stamped and nothing is emitted.
  The previous book version stays live; there is no interim state where Sales
  quotes from an unapproved list.
