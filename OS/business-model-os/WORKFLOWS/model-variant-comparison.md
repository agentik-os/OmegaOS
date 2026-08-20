# Workflow: Model variant comparison

**Produces:** two or three model shapes compared on the same unit, the same bar
and identical assumptions, with what must be true for each one to win.

## Trigger

The argument has moved from whether the business works to which shape it should
take: subscription versus per-project, self-serve versus assisted, take rate
versus listing fee, one-off versus recurring, direct versus partner-delivered.

## Steps

1. **Name the variants, at most three.** More than three is not a comparison, it
   is a survey, and it is answered by cutting the weakest two first on a stated
   reason.
2. **Fix one unit across all variants.** If the natural unit differs by shape (a
   project in one, an active seat in another), choose a common unit both can be
   expressed in and state the conversion. A comparison across two different units
   is not a comparison.
3. **Fix one bar across all variants**, the same one the viability assessment
   uses.
4. **Build one shared assumption set**: retention, acquisition cost per channel,
   delivery cost, price level, conversion. These are held identical unless a
   variant genuinely changes them.
5. **Name every assumption that genuinely differs by shape**, and say why the
   shape changes it. A subscription may plausibly carry different churn than a
   one-off sale; it does not plausibly carry a different cost of electricity.
   Each genuine difference is registered as its own claim, because it is now
   doing the work of choosing.
6. **Run the unit economics for each variant** on that shared set: contribution,
   acquisition cost, retention, payback, lifetime value.
7. **Run the breakeven for each variant** and set each against the same plausible
   pipeline volume.
8. **State what is identical between variants.** Anything the same across all of
   them is not a reason to choose, and saying so out loud removes most of the
   heat from the argument.
9. **State what must be true for each variant to win**, as a number: the
   retention it needs, the volume it needs, the price it needs, the delivery cost
   it needs. This is the real output, more than the ranking.
10. **Name the switching cost.** If the business is already running one of these
    shapes, moving to another has a cost in customers, contracts, systems and
    calendar time. A variant that wins on economics and loses on switching cost
    is reported as both.
11. **Do not declare a winner where the difference is inside the noise of the
    assumptions.** If the gap between two variants is smaller than the error in
    the assumptions driving it, say the comparison cannot separate them and name
    which claim, once settled, would.
12. **Register the deciding claims** and emit `business_model.assumption.registered`
    so Validation {OS} can settle the difference rather than the room settling it
    by preference.
13. **Write the comparison to canonical state** and hand the surviving shape to
    the viability assessment.

## Completion test

- At most three variants, on one unit, against one bar.
- The assumption set is shared, and every assumption that differs by shape is
  named, justified and registered as a claim.
- Each variant carries contribution, payback, lifetime value and a breakeven
  volume computed the same way.
- What is identical between the variants is stated explicitly.
- Each variant states, as a number, what must be true for it to win.
- Switching cost is stated where a shape is already running.
- Where the difference is inside the noise of the assumptions, no winner is
  declared and the deciding claim is named instead.
