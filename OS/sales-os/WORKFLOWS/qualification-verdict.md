# Qualification verdict

Produces a decision to pursue or to disqualify, with every dimension either
answered or explicitly marked unknown, and the reason stated to the prospect
either way.

## Trigger

A lead arrives from any source: inbound signal from Content {OS}, a consented
warm introduction from Network {OS}, or direct outbound.

## Steps

1. **Sales {OS}** records the source and, for an introduction, the consent
   record from Network {OS}. Produces the provenance line. An introduction
   with no consent record does not proceed: Network {OS} owns trusted
   relationship memory and is not a lead list.
2. **Sales {OS}** reads the live offer definitions from Offer {OS}. Produces
   the set of things this prospect could actually buy.
3. **Sales {OS}** assesses fit: does the prospect resemble the buyer the offer
   was written for. Produces a fit answer, or an explicit unknown.
4. **Sales {OS}** assesses need: is there a problem the offer addresses, in
   the prospect's own description. Produces a need answer, or an unknown.
5. **Sales {OS}** assesses authority: who decides, and have they been named.
   Produces a decision maker, or an unknown.
6. **Sales {OS}** assesses timing: is there a real reason to act in a period
   the business can serve. Produces a timing answer, or an unknown.
7. **Sales {OS}** assesses budget against the live price book from Pricing
   {OS}. Produces an answer, or an unknown. The price book is read, never
   improvised around.
8. **Sales {OS}** issues the verdict. Pursue, gather more on named unknowns,
   or disqualify. Produces the verdict record.
9. On disqualification, **a human** approves the exact message and **Sales
   {OS}** tells the prospect plainly, with the reason. Produces a
   disqualification record and a revisit condition if one exists.

## Completion test

The qualification is complete when:

- the source and, where applicable, the consent record are both recorded
- each of the five dimensions carries an answer or the literal value unknown,
  and no dimension carries an assumption in the seller's favour
- the verdict is one of pursue, gather, or disqualify, with no fourth state
- on pursue, a next action and an owner exist with a date
- on gather, the specific unknown to be resolved is named, and the opportunity
  does not advance a stage until it is
- on disqualify, the prospect has been told, and the reason recorded matches
  the reason given
- no budget answer was produced from a number outside the live price book

## Failure and abort

- **No consent record on an introduction.** Abort at step 1. Ask Network {OS}
  for consent. Do not contact the person, and do not treat a shared contact as
  an implied permission.
- **No live offer definition.** Abort at step 2. Qualifying against an offer
  that does not exist yet produces a pipeline of deals nobody can fulfil.
- **A dimension is unknown and being assumed favourably.** Refuse the
  assumption and record the unknown. An unknown budget is not a budget, and a
  pipeline built on that substitution forecasts revenue that never arrives.
- **The prospect fails qualification but wants to buy.** Disqualify anyway,
  and say why. A bad fit close costs the refund, the support load, the
  reference that never comes and the story they tell afterwards.
- **The prospect fits nothing in the offer set.** Disqualify for this cycle
  and route the shape of what they needed to Offer {OS} as market evidence.
  Do not stretch an existing offer over it to keep the deal alive.
