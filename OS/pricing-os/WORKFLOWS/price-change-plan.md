# Price change plan

Produces the record that moves a price: old number, new number, effective
date, modelled revenue impact, the grandfathering decision, and the approved
text every affected customer will read.

## Trigger

Willingness to pay evidence, unit economics or a `/pricing-review` variance
report indicates a price is wrong, and a change is being considered rather
than debated.

## Steps

1. **Pricing {OS}** states the reason for the change in one sentence, and the
   evidence behind it. Produces the change rationale. A change whose rationale
   is "it has been a while" is stopped here.
2. **Revenue {OS}** supplies every customer currently on the affected line,
   with contract term, end date and signed price. Produces the affected
   population.
3. **Pricing {OS}** separates that population: customers whose signed contract
   term the change would break, and customers who could move. Produces two
   lists. The first list is excluded from the change and honoured to term.
4. **Pricing {OS}** models the revenue impact: the gain at the new price, the
   expected churn, and the net across a stated horizon. Produces the impact
   estimate with its churn assumption written down and falsifiable.
5. **Pricing {OS}** makes the grandfathering decision per segment: move now,
   move at renewal, hold for a stated period, or hold indefinitely. Produces a
   decision per segment with a notice period.
6. **Pricing {OS}** drafts the customer communication: what changes, when,
   why, and what the customer can do about it. Produces the exact text.
7. **A human** approves the new price, the grandfathering decision and the
   exact communication text, as three separate approvals.
8. **Pricing {OS}** stamps a new price book version and emits it to Sales
   {OS}, Revenue {OS} and Growth {OS}. **Revenue {OS}** applies the new price
   at the dates the grandfathering decision specifies.

## Completion test

The change is complete when:

- every customer from step 2 appears in exactly one segment of the
  grandfathering decision, with a date
- no customer whose signed term the change would break has been moved
- the impact estimate names its churn assumption and the horizon it covers
- the new price book version is live and Sales {OS} quotes only from it
- three separate human approvals are recorded: the number, the grandfathering
  decision, and the exact customer text
- no customer has received the communication before all three approvals exist
- Revenue {OS} confirms the applied prices match the grandfathering decision,
  customer by customer

The last line is the real test. A change is not done when it is announced, it
is done when the billing matches what was announced.

## Failure and abort

- **A signed contract would be broken.** Exclude that contract, honour it to
  term, and surface it by name in the grandfathering decision. Never resolve
  this by reinterpreting the contract.
- **Affected population unavailable from Revenue.** Abort at step 2. Changing
  a price against an unknown population is not a decision, and the surprises
  arrive one angry email at a time.
- **Churn assumption cannot be grounded.** Model the change at two explicit
  churn rates, a tolerable one and an intolerable one, and present both.
  Abstain from a single point estimate rather than inventing confidence.
- **Grandfathering decision withheld.** Stop. Do not ship the price change
  without it: a price change without a grandfathering decision creates two
  truths for the same customer, and the second truth gets invented on a call.
- **Communication text not approved.** Nothing is sent. The new price may be
  live for new customers only, and the existing population stays untouched
  until the text is approved.
