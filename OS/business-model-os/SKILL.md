---
name: business-model-os
description: How value is created, delivered and captured, made explicit. Business Model {OS}, unit 19 of the AGENTIK {OS} suite (02 · DISCOVER & DECIDE). Use when the user asks about business model or invokes /business-model-os.
---

# Business Model {OS}

Write down how value is created, delivered and captured, then say whether the
arithmetic works.

## When to use this

Reach for Business Model when:

- You can describe what you are building but not how it makes money, or you can
  describe how it makes money but not what it costs to deliver.
- A plan says "we take 10%" or "we charge a subscription" and nothing behind it
  states what one customer is worth or what one costs to serve.
- You have revenue and it does not feel like it is working, and you need to know
  whether the problem is the price, the cost of delivery, the retention or the
  cost of acquisition.
- Someone asks for a lifetime value number and there is no retention figure
  anywhere in the business.
- You are choosing between model shapes: subscription versus per-project,
  self-serve versus assisted, marketplace take rate versus a listing fee.
- An investor, a board or a partner is about to see your economics and you want
  every number to survive being asked where it came from.
- You inherited a deck, a model or a spreadsheet and you need to know which of
  its numbers were measured and which were chosen.
- The model works on the slide and you want to know which single input, moved a
  little, turns it negative.

Near neighbours, and the line between them:

| Confused with | Difference |
|---|---|
| Pricing {OS} | Business Model says what price the mechanism requires to clear the bar, as a constraint. Pricing sets the actual list: tiers, packaging of value into price points, discounts, currencies, changes over time. Ask Business Model whether a price of that shape can work; ask Pricing what the number is. |
| Offer {OS} | Offer packages what is sold: the scope, the deliverables, the guarantee, the bonuses, the wording a buyer reads. Business Model is the machine behind the offer, and one model can carry several offers. |
| Revenue {OS} | Revenue owns the actual revenue: pipeline, bookings, recognition, forecasting, what came in this month. Business Model owns the shape of how revenue is supposed to arrive. When they disagree, Revenue is the fact and Business Model is the thing that needs updating. |
| Money {OS} | Money runs the books and the cash: accounts, runway, payables, taxes, what is actually in the bank. Business Model does plan-level arithmetic and never claims to be accounting. |
| Strategy & Portfolio {OS} | Strategy decides whether this model gets money, people and calendar time against every other candidate. Business Model tells it whether the model can work at all. A viability assessment is an input to the bet, never the bet. |
| Market Research {OS} | Market Research gathers and decides on the market and customer evidence body: sizing, segments, competition, demand and pricing evidence. Business Model consumes that evidence and reasons about the economics on top of it. It never runs the study itself. |
| Validation {OS} | Business Model states the economics as assumptions. Validation settles whether the willingness to pay, the delivery cost and the retention inside them actually hold. This OS registers claims and never grades its own homework. |

## Capabilities

- Draw the canvas from whatever exists: an idea, a paragraph, a deck, or a
  running business with no written model.
- Map each segment to the job it is hiring you for, the alternative it uses
  today, and why what you deliver beats that alternative for that segment.
- State the delivery mechanism concretely: what actually reaches the customer,
  who performs the work, and what has to happen for one unit to be delivered.
- Specify revenue mechanics per line: what triggers the payment, who pays, at
  what frequency, under what commitment, and how it expands or churns.
- Build the cost structure, split fixed versus variable against a named unit,
  with the source of every figure.
- Name a unit that can actually be counted, and refuse to compute economics on
  one that cannot.
- Compute contribution per unit, cost of acquisition, retention, payback period
  and lifetime value, with an origin label on every input.
- Compute the breakeven volume and compare it against the volume the pipeline
  can plausibly produce, rather than against ambition.
- Issue a viability verdict against a bar the owner stated first: VIABLE, VIABLE
  UNDER CONDITIONS, NOT VIABLE, INSUFFICIENT DATA.
- Compare two or three model shapes on identical assumptions and name what must
  be true for each to win.
- Stress the model: find which input, moved how far, turns it negative, and rank
  the inputs by fragility.
- Register every assumed number as a falsifiable claim with an owner and an
  impact-if-wrong, and hand it to Validation {OS}.
- Audit an inherited model and report, per number, what was measured and what
  was chosen.

## Procedure

1. **Recover.** Pull anything already established: segment profiles, market
   evidence and sizing, prior verdicts on assumptions registered here, and any
   previous viability assessment. Note the date of each. Never re-derive a
   number that has already been measured.
2. **Name the unit.** Decide what one unit of value is: a seat, a job, a
   delivery, an active account, a booked hour. Check that it can be counted in
   an operational system. If it cannot, stop and propose countable alternatives
   before touching any arithmetic.
3. **Map.** Fill the canvas: segments, value proposition per segment, delivery
   mechanism, channels, revenue mechanics, cost structure, key resources and
   external dependencies. Mark unknowns as unknown rather than plausible.
4. **Specify the mechanics.** For each revenue line, state the trigger, the
   payer, the frequency and the commitment. For each cost, classify it fixed or
   variable against the unit and record where the figure came from.
5. **Label every number.** Measured (from this business's data, with the period),
   benchmark (with the source named), or assumed (somebody chose it). Do this
   before any calculation, not after, because the label changes how much weight
   the output can carry.
6. **Compute the unit economics.** Contribution per unit using the delivery cost
   actually incurred. Acquisition cost per channel, including the time cost of
   channels people call free. Retention, or stop here if the model is recurring
   and retention is unknown. Then payback and lifetime value.
7. **Compute breakeven.** The volume per period at which the model stops losing
   money, against the stated fixed cost base.
8. **Get the bar.** Ask the owner what this model has to clear to be worth
   doing: which margin, which payback period, which return. Record it before
   assessing anything.
9. **Assess viability.** Compare breakeven against the volume the pipeline can
   plausibly produce, and the economics against the bar. Issue one verdict. If
   the required volume is not reachable, say NOT VIABLE and state what would
   have to change by how much, rather than adjusting an input until it closes.
10. **Stress it.** Move each load-bearing input until the model turns negative.
    Report the break value and how likely that move is. Rank the inputs by
    fragility so the owner knows which number is carrying the model.
11. **Register the assumptions.** Every assumed number becomes a claim: a
    falsifiable statement with an owner and an impact-if-wrong, ordered by how
    much the verdict moves if it is wrong. Emit them to Validation {OS}.
12. **Hand off.** Send the unit economics to Pricing {OS} and Revenue {OS} as a
    constraint, the viability assessment to Strategy & Portfolio {OS} and
    Blueprint {OS}, and write every canonical record to Context & Memory {OS}.
13. **Approve before it leaves.** Anything going to an investor, a board or a
    partner, any public commitment to a revenue mechanism, and any change to a
    model live customers are already on goes to the human approval boundary
    first.

## Handoffs

| To | Event | What it does with it |
|---|---|---|
| Pricing {OS} | `business_model.unit_economics.modeled` | sets the actual price list within the constraint the economics impose |
| Revenue {OS} | `business_model.unit_economics.modeled` | plans and forecasts revenue against the stated mechanics, and reports back when reality diverges |
| Strategy & Portfolio {OS} | `business_model.unit_economics.modeled`, `business_model.viability.assessed` | decides whether this model gets money, people and calendar time |
| Blueprint {OS} | `business_model.viability.assessed` | defines a product that is buildable against the delivery mechanism and the cost the model can bear |
| Validation {OS} | `business_model.assumption.registered` | turns each economic assumption into a signed test and returns a verdict |
| Context & Memory {OS} | `business_model.canvas.drafted` | makes the model durable and readable by every other OS across sessions |

Received from: Market Research {OS} (`market.validation.completed`,
`market.sizing.modeled`), Customer Discovery {OS}
(`discovery.segment.profiled`), Validation {OS} (`validation.verdict.issued`),
Librarian {OS} (`librarian.extract.delivered`).
