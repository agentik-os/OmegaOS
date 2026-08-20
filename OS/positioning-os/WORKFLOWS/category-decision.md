# Category decision

Decide which category you compete in, with the demand evidence that says the
category exists, or with an explicit declaration that you are inventing one and
will have to pay for the demand.

## Trigger

A new venture has no category on record, a claim keeps failing distinctiveness
against every rival on the same ground, sales calls reveal buyers comparing you
against something you did not expect, or the operator proposes a category
change.

## Steps

1. **Positioning {OS}** asks the operator for the last five real buying
   conversations and produces the list of alternatives those buyers named,
   including doing nothing.
2. **Positioning {OS}** runs `/position-map` over that list and produces the
   competitive set: each alternative with its published claim, its exclusion,
   and the source and date of the quote.
3. **Market Research {OS}** supplies demand evidence for each candidate
   category label: whether anyone searches for it, buys against it, or budgets
   for it. Positioning {OS} records the evidence with its source.
4. **Positioning {OS}** produces the candidate categories, each with three
   facts: who else is in it, what demand evidence exists, and what the buyer
   would compare you against inside it.
5. **Positioning {OS}** marks any candidate with no demand evidence as
   invented, and states what creating that demand costs: the education burden,
   the longer sales cycle, and the fact that early content will be teaching a
   category rather than selling a product.
6. **Operator** chooses a category, or chooses to invent one with the cost
   acknowledged in writing.
7. **Positioning {OS}** records the decision, the evidence behind it, and the
   review condition: the signal that would say the category choice was wrong.
8. **Positioning {OS}** replays every live ledger claim against the new
   category and produces the list of claims that no longer make sense in it.
9. **Human** approves the category change and the resulting claim retirements
   before anything is emitted.
10. **Positioning {OS}** emits the updated statement to every downstream unit
    and flags the change as a category change, not a wording change.

## Completion test

A single category is recorded with: the alternatives buyers actually named, a
demand verdict citing its evidence or an explicit invented marking with its
cost acknowledged by the operator, a review condition, and a per claim verdict
for every live ledger entry replayed against it. A category recorded without a
demand verdict, or an invented category with no acknowledged cost, fails.

## Failure and abort

- Fewer than three real buying conversations available: abort, and run customer
  discovery first. Category decisions taken from desk research alone are the
  ones that get discovered wrong in a sales call a quarter later.
- Demand evidence unavailable or unverifiable: do not infer demand from
  plausibility. Mark the category invented and continue on that basis, or stop.
- Two categories are equally supported: present both with their evidence and
  their downstream consequences, and stop. This is a human decision, and the OS
  does not pick.
- Step 8 finds live claims already published under the old category: the
  emission is withheld until a human approves the retirement plan, because
  changing category silently orphans claims already in market.
