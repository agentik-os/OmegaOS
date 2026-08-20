# Workflow: Unit economics model

**Produces:** the economics of one countable unit: contribution, cost of
acquisition, retention, payback period and lifetime value, with the origin of
every input on the face of the model.

## Trigger

The canvas exists and someone needs to know whether one unit of this business
makes money. Also runs whenever a delivery cost, an acquisition cost or a
retention figure is measured for the first time and replaces an assumption.

## Steps

1. **Name the unit.** A seat, a job, a delivery, an active account, a booked
   hour, a shipped order. Write down where that unit is counted in an
   operational system today. If it is counted nowhere, stop: propose two or
   three countable units, state what each would make the model mean, and
   restart from the chosen one.
2. **State the period.** Per month, per quarter, per year. Every figure in the
   model uses the same period or is converted explicitly.
3. **Compute revenue per unit** from the revenue mechanics: the price paid, the
   frequency, and what proportion of units actually pay (trials, free tiers,
   failed payments and refunds all reduce it).
4. **Compute the variable cost per unit using the cost actually incurred.**
   Include support time, onboarding effort, fulfilment, infrastructure that
   scales with usage, payment fees, refunds and failed deliveries. If a future
   efficiency is being assumed, model it as a separate labelled target and keep
   the incurred cost as the base case.
5. **Compute contribution per unit:** revenue per unit minus variable cost per
   unit. State it in currency and as a margin, and say which delivery cost it
   was computed against.
6. **Compute cost of acquisition per channel.** For paid channels, spend divided
   by customers acquired, including the creative and management cost. For
   channels people call free (organic, referral, community, founder network,
   content), price the time consumed at a real rate. A channel left at zero is
   registered as a claim in this step, with the volume it is being asked to
   deliver.
7. **State retention or churn**, with its origin and the period it covers. In a
   recurring model, if this figure does not exist, stop here: report the
   retention the model would need to clear the bar and register it as the first
   claim to test. Do not produce a lifetime value.
8. **Compute payback period:** how many periods of contribution it takes to
   recover the acquisition cost. State it in real calendar time, since payback
   longer than the cash runway is a different problem than a poor ratio.
9. **Compute lifetime value** from contribution and retention, and state the
   time horizon it is truncated at. An untruncated lifetime value assumes a
   customer who never leaves.
10. **Label every input** measured, benchmark or assumed. Measured names the
    period and the system it came from. Benchmark names the source. Assumed
    names the person who chose it.
11. **Compare against the fixed base:** state how many units per period are
    needed to cover fixed costs. This hands directly to the viability
    assessment.
12. **Register every assumed input as a claim**, ordered by how much the model
    moves if it is wrong. Emit `business_model.assumption.registered`.
13. **Emit `business_model.unit_economics.modeled`** to Pricing {OS}, Revenue
    {OS} and Strategy & Portfolio {OS}, and write the model to canonical state.

## Completion test

- The unit is named and can be counted in a system that exists today.
- Every figure uses the same period, or its conversion is stated.
- Contribution is computed against the delivery cost actually incurred, and any
  efficiency target is separated from the base case.
- Every channel has an acquisition cost, or is registered as an unpriced
  assumption naming the volume expected of it.
- A recurring model carries a retention figure with an origin, and no lifetime
  value exists without one.
- Lifetime value states its time horizon.
- Every input reads measured, benchmark or assumed, and every assumed input
  appears in the claim register.
- The number of units per period needed to cover fixed costs is stated.
