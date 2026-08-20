# Offer definition

Produces the artifact every other GROW unit depends on: a versioned offer with
a promise, a named outcome, deliverables, a scope boundary, exclusions, a
guarantee and a proof set.

## Trigger

A new thing is about to be sold, or an existing offer has been frozen by
`/offer-cost-check` or by a positioning change and has to be redefined.

## Steps

1. **Offer {OS}** reads the positioning statement and the claim ledger from
   Positioning {OS}. Produces the claim the offer must honour. If no claim
   exists, the workflow aborts here and names Positioning {OS}.
2. **Offer {OS}** reads the job to be done from Customer Discovery {OS} and
   the demand evidence from Validation {OS}. Produces the buyer's own
   description of the problem, quoted, not paraphrased into seller language.
3. **Offer {OS}** drafts the promise as a named outcome. Produces a one
   sentence promise whose subject is the buyer's result, not the seller's
   activity.
4. **Offer {OS}** enumerates the deliverables and assigns each an acceptance
   form. Produces a deliverable list where every line states the condition
   under which it is done.
5. **Offer {OS}** draws the scope boundary and writes the exclusions. Produces
   the scope boundary artifact plus an exclusion list with at least one entry.
6. **Delivery & Customer Success {OS}** supplies the fulfilment cost model for
   the drafted scope. Produces hours, headcount, tooling and support load per
   unit sold.
7. **Offer {OS}** runs the economics comparison. Produces a viable or not
   viable verdict with the gap quantified.
8. **Offer {OS}** assembles the proof set. Produces one proof item per
   promise, each naming its source, with any customer named carrying a consent
   status from Network {OS}.
9. **Sales {OS}** supplies the objections the offer will actually meet.
   **Offer {OS}** answers each or revises the offer. Produces an objection log
   and a revision list.
10. **A human** reviews and approves the exact wording. **Offer {OS}**
    publishes, versions and emits the definition to Pricing {OS}, Sales {OS},
    Revenue {OS}, Delivery & Customer Success {OS} and Content {OS}.

## Completion test

The offer definition is complete when all of the following hold and are
checkable by reading the artifact alone:

- the promise names an outcome, and no deliverable line is the whole promise
- every deliverable has an acceptance condition stated without a conversation
- the exclusion list has at least one entry
- the guarantee, if present, has a modelled worst case cost and a ceiling
- every proof item names a source, and every named customer has consent
- the economics verdict is viable, with the fulfilment cost model dated
- a human approval of the exact published wording is recorded
- a version stamp exists and the emission list names all five consuming units

Any single item unmet leaves the offer in `draft`, not in `live`.

## Failure and abort

- **No positioning claim.** Abort at step 1. Report Positioning {OS} as the
  blocking unit. Do not invent a claim to unblock the draft.
- **No fulfilment cost model.** Continue to step 9 but block publication. The
  offer stays `draft` and the report states plainly that the economics are
  unverified.
- **Not viable economics.** Freeze the offer for new sales, emit the three
  options (narrow the scope, change the guarantee, hand it to Pricing {OS}),
  and stop. Offer {OS} does not choose between them.
- **Offer contradicts the claim.** Halt, present both statements side by side,
  escalate to a human. Neither artifact is edited unilaterally.
- **Human approval withheld.** The offer stays `draft`. Nothing is emitted,
  and no consuming unit is told an offer exists.
