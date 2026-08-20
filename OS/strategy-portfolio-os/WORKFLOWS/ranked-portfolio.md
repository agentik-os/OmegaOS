# Workflow: Ranked portfolio

**Produces:** the scored and ranked portfolio: every active and proposed bet
with a status, an owner, a resource cost, kill criteria and a stated opportunity
cost, plus the explicit not-doing list.

## Trigger

More candidates exist than the period can carry, or nobody can say how many
things are actually running. Also triggered by a new candidate arriving
(`brainstorm.concept.selected`, `opportunity.named`,
`market.validation.completed`) into a portfolio that was already full.

Runs the portfolio council protocol.

## Steps

1. **Confirm the kernel.** Ranking without a guiding policy produces a
   preference list, not a portfolio. If no kernel exists, run the strategy
   kernel workflow first, or record explicitly that the ranking is provisional
   and why.
2. **Inventory everything.** Every active project, every proposed bet, and every
   commitment nobody counts: maintenance, support, obligations to other people,
   and half-finished work still consuming attention. The uncounted items are
   where the missing capacity always is.
3. **Expose hidden maintenance.** For each running item, the recurring cost it
   imposes per week whether or not anyone advances it. An item with a large
   maintenance cost and no thesis is a candidate for the kill review, not the
   ranking.
4. **Write the thesis per bet.** Why this may work, in one sentence, falsifiably.
   A bet whose thesis cannot be stated is not ready to be scored.
5. **Score each item.** Strategic fit against the guiding policy, evidence score
   with its E1 to E5 basis, upside, learning value, resource cost in time,
   capital and people, and downside. Scores are cache, never evidence: they
   order a conversation, they do not settle it.
6. **Attach kill criteria.** For every item that could be funded: the observable
   condition, with a threshold or a date, under which it stops. An item with no
   kill criteria cannot be funded at this step.
7. **Run reversibility analysis.** For each top candidate, name the cheapest
   reversible experiment that would buy the same information as the
   irreversible commitment. Where one exists, it is ranked ahead of the
   commitment.
8. **Check capacity before you rank.** Total the hours, the people and the
   capital that genuinely exist this period, consuming `health.capacity.assessed`
   and `capital.reallocation.proposed`. Subtract the maintenance from step 3
   first. What remains is what can be allocated.
9. **Rank, and state the opportunity cost.** Order the candidates. For every
   item you propose to fund, name what it excludes: which other candidate loses
   the hours, the people or the money. A funded bet with no stated cost is
   returned, not published.
10. **Choose the disposition per item:** fund, experiment, hold, pause or kill.
    Where two candidates cannot be separated on the available evidence, say so
    and name the single piece of evidence that would separate them, its owner
    and its cost, rather than breaking the tie on taste.
11. **Publish the not-doing list.** Every exclusion with the reason and the
    condition that would reopen it, plus the low-cost options being deliberately
    preserved. An exclusion with no reopening condition becomes a permanent
    blind spot.
12. **Route governance.** Consequential funding, pause or kill decisions go out
    as `strategy.change.requested` and wait for `change.approved` before
    `portfolio.item.funded`, `portfolio.item.paused`, `portfolio.item.killed` or
    `allocation.changed` is emitted.
13. **Record.** Write the portfolio items, bets and allocations to canonical
    state through Context & Memory {OS}, each with owner, completion evidence
    and review trigger.

## Completion test

- Every active and proposed item appears, including maintenance and obligations.
- Every item carries a status, an owner, a resource cost and, if fundable, kill
  criteria with a threshold or a date.
- Every funded item has a written thesis and a stated opportunity cost naming
  what it excludes.
- The total allocation fits the capacity remaining after maintenance, and any
  overcommitment is reported as an overcommitment rather than absorbed.
- The not-doing list is published, each entry with a reason and a reopening
  condition.
- No consequential portfolio event was emitted before its `change.approved`
  returned.
