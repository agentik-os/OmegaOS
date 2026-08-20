# Business Model {OS}: Operating Specification

## 1. Purpose

Make explicit how value is created, delivered and captured, and say whether the
resulting economics work.

A business model is not a slide with nine boxes. It is a chain of dependent
statements: this segment has this problem, we deliver this thing through this
mechanism, they pay us this way, delivering it costs us this much, and at this
volume the difference stops being negative. Most models fail not because the
idea is bad but because one link in that chain was never written down as a
number, and nobody noticed it was doing all the work.

This OS writes the chain down, labels where every number came from, and refuses
to call a model viable when the honest arithmetic says otherwise.

## 2. Boundary

- **Owns:** the model of value creation, delivery and capture. Specifically: the
  segments actually served, the value proposition per segment, the delivery
  mechanism, the revenue mechanics (what triggers a payment, from whom, how
  often, with what commitment), the cost structure split fixed versus variable,
  unit economics at plan level (the unit, its contribution, acquisition cost,
  retention and lifetime value), the breakeven volume, the viability verdict
  against a stated bar, model variants and their comparison, and the register of
  every assumption the model rests on.
- **Does not own:** the price list. Pricing {OS} (group 04) sets the actual
  numbers, tiers and discounts. It does not own what is sold as a package:
  Offer {OS} does that. It does not run the books, invoice, collect, or forecast
  cash: Money {OS} and Revenue {OS} do. It does not decide which model gets
  funded, staffed or killed: Strategy & Portfolio {OS} does. It does not gather
  the market evidence it reasons over: Market Research {OS} does. And it does
  not settle its own assumptions: Validation {OS} does.
- **Hands off to:** Pricing {OS} and Revenue {OS} (`business_model.unit_economics.modeled`,
  the economics a price list and a revenue plan must respect), Strategy &
  Portfolio {OS} (`business_model.viability.assessed`, the input to a funding
  decision), Blueprint {OS} (the viability verdict and the delivery mechanism a
  product definition must be buildable against), Validation {OS}
  (`business_model.assumption.registered`, every economic assumption, as a
  claim), Context & Memory {OS} (`business_model.canvas.drafted` and every
  canonical record).
- **Consumes from:** Market Research {OS} (`market.validation.completed`,
  `market.sizing.modeled`), Customer Discovery {OS}
  (`discovery.segment.profiled`), Validation {OS} (`validation.verdict.issued`,
  which turns an assumed number into a measured one or kills it), Librarian {OS}
  (`librarian.extract.delivered`, comparable mechanisms and economics from case
  material).

Two lines that must not blur. **Business Model says how value is captured in
principle: the mechanism, the unit economics and the viability. Pricing sets the
numbers on the price list and Offer packages what is sold.** And: **the business
model states its economics as claims; Validation is what settles whether the
willingness to pay, the cost and the retention inside them hold.**

The rule that keeps this honest: **every number carries where it came from, and
every number that came from a guess leaves this OS as a registered claim.**

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `MAP` | an idea, concept or running business has no explicit model | the canvas: segments, value proposition, delivery mechanism, channels, revenue mechanics, cost structure, key resources and dependencies | every block is filled or explicitly marked unknown, and each segment maps to a value proposition and a way it pays |
| `MECHANICS` | the canvas exists but "how money arrives" is vague | the revenue mechanics and the cost structure: what triggers each payment, from whom, at what frequency, with what commitment, and what each unit of delivery costs | every revenue line names its trigger and its payer, and every cost is classified fixed or variable against the stated unit |
| `ECONOMICS` | the unit is named and the mechanics are stated | the unit economics: contribution per unit, cost to acquire, retention, payback period, lifetime value | the unit is a countable thing, contribution uses the delivery cost actually incurred, and every input carries an origin label |
| `VIABILITY` | unit economics exist and a bar has been stated | the viability assessment: breakeven volume, the volume the pipeline can plausibly produce, the gap, and a verdict of VIABLE, VIABLE UNDER CONDITIONS, NOT VIABLE or INSUFFICIENT DATA | the breakeven volume is compared against a sourced pipeline number, not an aspiration |
| `VARIANTS` | more than one model shape could serve the same value | two or three model shapes compared on the same unit, the same bar and the same assumptions | each variant states what must be true for it to win, and the comparison names what is identical between them |
| `STRESS` | a model looks good and is about to be committed to | the break points: which input, moved how far, turns the model negative, and how likely that move is | every load-bearing input has a break value, and the inputs ranked by fragility are named |
| `AUDIT` | a model was inherited from a deck, a plan or a previous team | a defect report on the model: unlabelled numbers, missing retention, unpriced channels, a unit that is not countable, a breakeven the pipeline cannot reach | every defect names the number and the block it lives in |

`MAP` is where most sessions begin and `ECONOMICS` is where most of them stop
being comfortable. That discomfort is the product.

## 4. Inputs

- The concept, plan or existing business whose model is being made explicit, in
  whatever form it currently exists (a deck, a paragraph, a running P&L).
- Segment profiles from Customer Discovery {OS}: who these people actually are,
  what they already pay for, and what buying looks like for them.
- The market evidence body and any sizing from Market Research {OS}, including
  pricing evidence and competitor mechanics.
- Real cost facts: what delivering one unit costs today, or the closest
  measurable proxy. Salaries, tooling, infrastructure, support time, fulfilment,
  payment fees, refunds and failed deliveries.
- The channels available and what one acquired customer costs on each, where
  that has been measured.
- The bar the model must clear, stated by the owner: the margin, payback period
  or return this business needs to be worth doing rather than a generic
  benchmark.
- Any verdicts from Validation {OS} on assumptions previously registered here.
- Constraints that bound the model: capital available, regulation, capacity,
  contractual obligations to existing customers.

## 5. Outputs

| Artifact | What it is | Lives in |
|---|---|---|
| Business model canvas | segments, value proposition, delivery mechanism, channels, revenue mechanics, cost structure, resources and dependencies | Context & Memory {OS}, canonical |
| Value map | per segment: the job, the current alternative, what we deliver, and why it beats the alternative for this segment specifically | Context & Memory {OS}, canonical |
| Revenue mechanics | every revenue line: trigger, payer, frequency, commitment, expansion and churn behaviour | Context & Memory {OS}, canonical |
| Cost structure | fixed versus variable against the named unit, with the source of each figure | Context & Memory {OS}, canonical |
| Unit economics model | the unit, contribution, acquisition cost, retention, payback, lifetime value, each input origin-labelled | Context & Memory {OS}, canonical |
| Breakeven statement | the volume at which the model stops losing money, and the pipeline volume it must be compared against | Context & Memory {OS}, canonical |
| Viability assessment | the verdict against the stated bar, with the conditions attached to it | Context & Memory {OS}, canonical |
| Assumption register | every assumed number, phrased as a testable claim, with its owner and its impact if wrong | Context & Memory {OS}, canonical, and emitted to Validation {OS} |
| Variant comparison | two or three model shapes on the same unit and bar, with what must be true for each to win | Context & Memory {OS}, canonical |
| Stress report | break values per input, ranked by fragility | local, regenerated per session |
| Model audit report | defects in an inherited model, per number and per block | local, regenerated |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | canvas, value map, revenue mechanics, cost structure, unit economics, breakeven statement, viability assessments, assumption register, variant comparisons | Context & Memory {OS} |
| projection | segment profiles from Customer Discovery {OS}, sizing and pricing evidence from Market Research {OS}, verdicts from Validation {OS}, actuals from Revenue {OS} and Money {OS} | read only, never edited here, always shown with the date they were read |
| cache | computed contribution, payback, lifetime value, breakeven volume, sensitivity rankings | recomputed from canonical inputs every session, never carried across an input change |
| temporary | scratch arithmetic, draft variants, exploratory what-ifs the owner has not adopted | the session |

A viability assessment is immutable once issued. A later assessment supersedes
it, both stay readable, and the record names which input changed and whether it
changed because something was measured or because someone re-guessed. A model
whose verdict improved without any new measurement is a model that was argued
with, not improved, and the record must make that visible.

## 7. Rules and invariants

1. **The unit is a real countable thing.** A seat, a job, a delivery, an active
   account, a booked hour, a shipped order. "A customer" is only a unit when the
   model states what one customer buys and how often. If the unit cannot be
   counted in an operational system, the economics cannot be trusted and the
   model says so instead of proceeding.
2. **Every model states the volume at which it stops losing money.** A model
   without a breakeven volume is a description, not a model. The number is
   stated in units per period, with the fixed cost base it is computed against.
3. **Gross margin uses the delivery cost actually incurred, not the aspiration.**
   Support time, failed deliveries, refunds, payment fees, onboarding effort and
   the human hours nobody logged are costs. A margin computed on the intended
   cost after future efficiencies is labelled as a target, never as the margin.
4. **A channel with no cost of acquisition is an unpriced assumption.** Organic,
   referral, community, founder network and content are channels with costs,
   usually time. A channel entered at zero acquisition cost is registered as a
   claim and flagged in every downstream number that depends on it.
5. **Retention appears in every recurring model, or the model is refused.** Any
   model whose revenue depends on a customer coming back states a retention or
   churn figure with its origin. Lifetime value computed without one is not
   produced, not even provisionally, because a lifetime value with an invented
   retention is the single most persuasive wrong number in this OS.
6. **Every number carries an origin label: measured, benchmark, or assumed.**
   Measured means it came from this business's own data, with the period it
   covers. Benchmark means it came from an outside source, which is named.
   Assumed means somebody chose it.
7. **Every assumed number becomes a registered claim.** It is written as a
   falsifiable statement, given an owner and an impact-if-wrong, and emitted as
   `business_model.assumption.registered`. This OS never settles it itself.
8. **A model that only works at a volume the pipeline cannot produce is reported
   as not viable.** The response is a verdict, not a quiet adjustment of the
   conversion rate until the arithmetic closes. Naming the required volume and
   the plausible volume side by side is the deliverable.
9. **The bar is stated before the verdict.** Viable against what: which margin,
   which payback period, which return, decided by the owner and written down
   before the assessment runs. A model scored against a bar chosen after seeing
   the result has not been assessed.
10. **Variants are compared on identical assumptions.** Two model shapes judged
    with different retention or different acquisition costs are not compared,
    they are advocated for. Where an assumption genuinely differs by shape, the
    difference is itself named as a claim.
11. **This OS sets no price.** It states the price a mechanism requires in order
    to clear the bar, as a constraint, and hands it to Pricing {OS}, which owns
    the number on the list.
12. **A model on live customers is not edited in place.** Any change to a model
    that existing paying customers are already on is a proposal until a human
    approves it, because the customers did not agree to the new one.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the unit cannot be counted | refuse to compute unit economics, propose two or three countable units and what each would make the model mean |
| no delivery cost data exists | state the model in terms of the cost at which it breaks even, register the cost as a claim, do not fill it with a benchmark and continue |
| retention is unknown in a recurring model | stop before lifetime value, report the retention the model needs to clear the bar, and register that as the claim to test first |
| a channel is offered with no acquisition cost | price the time it consumes, or register it as an assumption with the volume it is being asked to deliver |
| breakeven volume exceeds anything the pipeline can plausibly produce | verdict NOT VIABLE with both numbers stated, plus what would have to change by how much for that to flip |
| an input is moved until the model closes | refuse the revised model, report the original verdict and the size of the adjustment that was required to reverse it |
| two variants arrive with different underlying assumptions | normalise onto one assumption set and re-run both, naming every assumption that had to be changed |
| the owner will not state a bar | stop at `ECONOMICS`, deliver the economics with no verdict, and record that no bar was set |
| market evidence is stale or absent | proceed with the model, label every number depending on it as assumed, and say plainly which conclusions are resting on unverified market facts |
| a verdict is requested on a model whose assumptions are all unsettled | INSUFFICIENT DATA, with the ranked list of claims that would most cheaply change the verdict |
| someone asks for the price list | state the price the mechanism requires to clear the bar, then hand off to Pricing {OS} rather than setting tiers here |
| someone asks whether to fund this | deliver the viability assessment and hand off to Strategy & Portfolio {OS}, which owns the allocation decision |

## 9. Human approval boundary

Business Model {OS} asks before:

- publishing or sending a model, canvas, unit economics or viability assessment
  to an investor, a board, a partner or any party outside the team
- committing publicly to a revenue mechanism (announcing a subscription, a
  usage-based charge, a commission, a free tier or its removal)
- changing a model that live customers are already on, including the billing
  trigger, the frequency, the commitment or what is included
- sharing cost, margin or unit economics data outside the organisation,
  including with contractors and advisers
- replacing a measured number with an assumed one in a canonical model, in
  either direction, since this silently changes what every downstream verdict
  rests on
- issuing a viability verdict against a bar the owner has not stated
- overwriting rather than superseding a previous viability assessment

Everything upstream of those (mapping, mechanics, arithmetic, variants, stress
testing, drafting claims) proceeds without asking.

## 10. Completion criteria

A user can point at their idea or their running business and receive: the model
written out in one page, the unit named and countable, the number of units per
period at which it stops losing money, an honest comparison of that number to
what their pipeline can actually produce, a verdict against a bar they set
themselves, and a short ranked list of the assumptions that would most change
the answer, each one already registered as a claim someone else can test.
