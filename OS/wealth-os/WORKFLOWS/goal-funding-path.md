# Workflow: Goal funding path

Price a long-horizon goal in money per month, test it against verified surplus,
and say plainly what it displaces.

**Trigger:** `/wealth-goal "<goal>"`, the moment a goal is spoken, or a review
where an existing goal's horizon, target or surplus has moved.

**Owner:** the operator chooses the lever. The OS states the arithmetic and the
tradeoff, and picks nothing.

## Steps

1. **Make the goal a number and a date.** A target amount and a horizon. If the
   operator has neither, work backward from what the goal buys (a deposit, a
   year of outgoings, a business bought out) rather than accepting an adjective.
2. **Establish the starting point.** What is already allocated to this goal
   today, from the dated balance sheet, excluding anything counted toward the
   reserve. Reserve money funds no goal.
3. **Take the verified surplus.** `money.surplus.verified` from Money {OS}, over
   the last three closed months, not a single good month and not an estimate.
4. **Compute the required contribution.** Target minus starting point, over the
   months to the horizon, with any growth assumption stated explicitly and
   expressed as a range rather than a rate presented as a fact.
5. **Test affordability.** Compare against surplus already committed to the
   reserve and to other goals. Return one of three verdicts: funded; short by an
   amount per month; funded only by displacing something, named.
6. **Show the levers when short.** Lower the target, extend the horizon, or
   raise the surplus, each quantified. State what each costs. Choose none of
   them.
7. **Record the liquidity requirement.** A goal with a date creates a dated
   liquidity need, which is a constraint on Capital {OS}, not a suggestion. Add
   it to the constraint set.
8. **Check it against the risk register.** If an unmitigated risk would consume
   the goal's funding, say so here rather than letting two documents disagree.
9. **Publish.** Emit `wealth.goal.funded_path`, and republish
   `wealth.capital_constraints.published` if the dated liquidity need changed.
10. **Route what is not ours.** Tax treatment of the vehicle used, product
    suitability and any regulated question go into the adviser pack with the
    numbers attached.

## Completion test

The goal is priced when:

- target, horizon, starting point and required monthly contribution are all
  stated in currency
- the verdict is one of funded, short by an amount, or funded by displacing a
  named commitment
- every growth assumption is stated as a range with what would falsify it, and
  no single compounding figure is presented as a plan
- reserve money was excluded from the starting point
- the dated liquidity need reached the constraint set for Capital {OS}
- `wealth.goal.funded_path` has been emitted, and no lever was chosen by the OS
