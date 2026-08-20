# Workflow: Reserve sizing and funding

Turn "we should have some savings" into a target in months, a place to hold it,
and a rule that refills it after it is used.

**Trigger:** `/wealth-reserve`, no reserve policy exists, the reserve has been
drawn below target, or the risk picture changed (income concentration, health,
a new dependant, a currency exposure).

**Owner:** the operator approves the policy, because Money {OS} will test every
monthly close against it.

## Steps

1. **Get real outgoings, not estimated ones.** Take the last three closed months
   from Money {OS}. If no month is closed, stop and say the figure would be an
   estimate; a reserve sized on a guess protects nothing.
2. **Separate committed from discretionary.** Committed outgoings (housing,
   instalments, insurance, essential subscriptions) set the floor. Discretionary
   spending sets the comfortable figure. Present both.
3. **Set the months of cover.** Base it on how long the operator's income would
   take to replace, not on a round number. Concentrated income (one client, one
   employer, one market) needs more months, and the OS says so in months rather
   than in adjectives.
4. **Apply the liquidity test to every candidate holding.** Reachable in days,
   without a penalty, without a forced sale, without another party's permission.
   Property, private equity, locked pensions and unvested equity fail it
   regardless of value.
5. **Place the reserve.** Name where it is held and in which currency. If the
   operator's outgoings are in one currency and the reserve in another, state
   the exposure rather than netting it away.
6. **State the gap.** Target minus what qualifies today, in currency and in
   months, plus the time to close it at the verified surplus from Money {OS}.
7. **Write the refill rule.** What counts as a legitimate draw, and what happens
   after one: the monthly amount, the priority against goals, and the point at
   which goal funding pauses to restore cover.
8. **Approve and publish.** On the operator's approval, emit
   `wealth.reserve_target.set` so every close in Money {OS} reports whether the
   month funded it.
9. **Route the professional questions.** Where a reserve should sit for tax
   purposes, whether a product is suitable, and whether cover replaces reserve
   are questions for a tax professional, an adviser or a broker. They go into
   the adviser pack, unanswered here.

## Completion test

The reserve policy is done when:

- the target is expressed in months of outgoings taken from closed months, and
  both the committed floor and the comfortable figure are shown
- every holding counted toward the reserve passed the liquidity test, and the
  ones that failed are named with the reason
- the currency of the reserve is stated against the currency of the outgoings
- the gap is stated in currency, in months, and in time to close at the verified
  surplus
- the refill rule names what pauses when the reserve is drawn
- `wealth.reserve_target.set` has been emitted with the operator's approval
