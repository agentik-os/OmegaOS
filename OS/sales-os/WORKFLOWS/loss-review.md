# Loss review

Produces the honest record of why a deal was lost: the reason the buyer gave,
the reason you believe, the stage where it was really lost, and the unit that
owns the fix.

## Trigger

An opportunity is marked closed lost, including the losses that feel obvious
and the ones that went quiet rather than saying no.

## Steps

1. **Sales {OS}** records the stated reason: what the buyer actually said,
   quoted. Produces the stated reason field. A buyer who went silent is
   recorded as silent, not as a price loss.
2. **Sales {OS}** records the believed reason, separately. Produces the
   believed reason field. The two fields are never merged, because merging
   them is how a business spends a year fixing a price problem that was
   really a positioning problem.
3. **Sales {OS}** identifies the stage at which the deal was really lost,
   which is usually earlier than the stage it was sitting in. Produces the
   loss stage.
4. **Sales {OS}** classifies the owning unit: Positioning {OS} if the claim
   did not land, Offer {OS} if the shape was wrong, Pricing {OS} if the number
   was wrong, Delivery & Customer Success {OS} if a reference or a capability
   was missing, or Sales {OS} if the process failed. Produces the owner.
5. **Sales {OS}** checks whether qualification should have caught it. Produces
   a yes or no, and where yes, the specific dimension that was assumed
   favourably instead of recorded as unknown.
6. **Sales {OS}** extracts the objection that decided it, if there was one,
   and emits it to Offer {OS} for stress testing. Produces the objection
   record.
7. **Growth {OS}** receives the loss pattern with conversion by stage.
   **Pricing {OS}** receives any loss where the deciding factor was price,
   whether or not a discount was requested.

## Completion test

The review is complete when:

- the stated reason and the believed reason are both populated, and they are
  two distinct fields even when they agree
- the stated reason is a quotation or is explicitly recorded as none given
- the loss stage is named, and it is a stage the opportunity actually passed
- exactly one owning unit is named, and that unit has received the record
- the qualification check is answered, and where the answer is yes, the
  favourably assumed dimension is named
- the deciding objection, if any, has been emitted to Offer {OS}
- no field contains a rationalisation that cannot be traced to something the
  buyer said or did

## Failure and abort

- **The buyer gave no reason.** Record none given. Do not substitute the most
  comfortable hypothesis. A pipeline of losses all attributed to price, with
  no buyer ever having said price, is a fiction that survives because nobody
  wrote down that it was never said.
- **The believed reason cannot be separated from the stated one.** Record them
  as identical and note that no independent assessment was made. That is
  weaker evidence, and it should look weaker in the record.
- **The loss stage is disputed inside the team.** Record both candidate
  stages and escalate to a human. Do not average them.
- **The owning unit would be Sales {OS} itself.** Record it anyway. A loss
  review that never blames the sales process is not a review, it is a defence,
  and Growth {OS} will optimise the wrong stage on the strength of it.
- **The opportunity is being reopened rather than reviewed.** Complete the
  loss review first, then open a new opportunity. A resurrected deal that
  skipped its review carries the same unexamined defect back into the
  pipeline.
