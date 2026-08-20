# Workflow: Business model canvas

**Produces:** the model written out in one page: who is served, what they get,
how it reaches them, how money arrives, what it costs to deliver, and what the
model depends on to keep working.

## Trigger

An idea, a plan, a deck or a running business exists, and there is no single
written statement of how it creates, delivers and captures value. Also runs when
the business has changed shape and the written model no longer describes it.

## Steps

1. **Recover what is already established.** Pull segment profiles from Customer
   Discovery {OS}, market evidence and sizing from Market Research {OS}, and any
   previous canvas or viability assessment. Record the date of each. A projection
   older than the last business change is treated as stale and labelled.
2. **List the segments actually served**, not the segments addressable. Each one
   named as a group you could put on a contact list. A segment nobody in the
   business can name three real examples of is marked as hypothetical.
3. **Write the value proposition per segment.** One per segment, never one shared
   sentence across three kinds of buyer. State the job they are hiring you for
   and the alternative they use today, including doing nothing.
4. **State the delivery mechanism.** What physically reaches the customer, who
   performs the work, and what has to happen for one unit to be delivered. If
   delivery requires a human, name the role and the hours.
5. **List the channels** through which a customer arrives, and for each, whether
   its acquisition cost has been measured, estimated, or never priced. A channel
   entered at zero cost is written down as an assumption in the same line, not
   left implied.
6. **State the revenue mechanics in outline:** who pays, for what, when. Detail
   belongs to the revenue mechanics workflow; the canvas states the shape.
7. **State the cost structure in outline:** the fixed base per period, and what
   varies with volume. Detail belongs to the unit economics workflow.
8. **Name the key resources and external dependencies:** the people, the assets,
   the platforms, the suppliers, the licences and the single points of failure
   the model would stop working without.
9. **Mark every unknown as unknown.** Do not fill an empty block with something
   plausible. An empty block is information; a plausible invention is a defect
   that survives into every downstream number.
10. **Label every number that appears** as measured, benchmark or assumed.
11. **Trace the chain.** Every segment must reach a value proposition, a delivery
    mechanism and a way it pays. Any segment that does not is either not served
    or not a segment; say which.
12. **Register the assumptions.** Each assumed number and each hypothetical
    segment becomes a claim with an owner and an impact-if-wrong. Emit
    `business_model.assumption.registered`.
13. **Emit `business_model.canvas.drafted`** and write the canvas to canonical
    state.

## Completion test

- Every segment traces to a value proposition, a delivery mechanism and a
  payment path.
- Every value proposition names the alternative it beats, including doing
  nothing where that is the real competitor.
- The delivery mechanism says who does the work, not only what the customer
  receives.
- Every channel is either priced or explicitly registered as an unpriced
  assumption.
- No block is filled with a plausible invention: unknowns read as unknown.
- Every number carries measured, benchmark or assumed.
- A reader outside the business could restate how the company makes money in two
  sentences, from this page alone.
