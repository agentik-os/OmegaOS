---
name: wealth-os
description: Personal net worth, reserves and long-horizon goals. Wealth {OS}, unit 51 of the AGENTIK {OS} suite (06 · OWN). Use when the user asks about wealth or invokes /wealth-os.
---

# Wealth {OS}

Personal net worth, reserves and long-horizon goals.

## When to use this

Use Wealth {OS} when the question is about position and horizon rather than
about this month:

- the operator cannot state their net worth, or states it from memory
- there is no reserve, or nobody knows how many months the reserve actually
  covers
- a goal exists (a house, a sabbatical, a date to stop needing the business)
  with no funding path attached to it
- income, health, a large client or a currency represents a concentration nobody
  has priced
- Capital {OS} is about to allocate and has no published constraints to work
  inside
- an exit is being prepared and the proceeds need somewhere honest to land

Near neighbours, and how to tell them apart:

| If the question is | The OS is |
|---|---|
| what am I worth, how much reserve, what is the money for | Wealth {OS} |
| what came in and out this month, what is left | Money {OS} |
| what did the business bill, collect and retain | Revenue {OS} |
| where exactly should this money be invested | Capital {OS} |
| which entity owns the stake, on what terms | Ownership {OS} |
| what will the sale produce and when | Exit & Liquidity {OS} |

The line that decides it: Money owns the month, Wealth owns the balance sheet
and the horizon, Capital owns the allocation. Wealth publishes the constraints
and never picks the investment.

This OS assists but does not replace a legally accountable accountant, tax
professional, lawyer or licensed financial adviser. It does not decide the tax
treatment of a disposal or a pension contribution, does not judge whether
insurance cover or an estate structure is adequate, and does not give regulated
investment advice. It never moves money: it does not open, fund or close an
account, place an order, or transfer a balance.

## Capabilities

- Build a dated personal balance sheet where every line carries a valuation
  basis, and keep unvalued items visible instead of guessed.
- Track net worth over time and attribute each movement to contribution,
  valuation change or debt movement.
- Size a reserve in months of the operator's real outgoings, taken from closed
  months in Money {OS} rather than from an estimate, and write the refill rule.
- Test a long-horizon goal against verified surplus and return the required
  monthly contribution, the shortfall, or what funding it displaces.
- Maintain a personal risk register: the events that would break the plan,
  ranked by damage, each mitigated or explicitly accepted.
- Publish capital constraints to Capital {OS}: reserve floor, dated liquidity
  needs, horizon, tolerable loss.
- Model a scenario (income stops, a rate moves, a goal is brought forward) as a
  range with its assumptions, never as a single number.
- Assemble an adviser pack: the organised numbers plus the questions this OS
  refused to answer itself.

## Procedure

1. **Baseline.** List assets and liabilities at a date. Every line gets a
   valuation basis and the date that basis was established. Anything unvalued
   goes on its own list at zero, never at a guess.
2. **Attach the flow.** Pull `money.surplus.verified` and the run rate from
   Money {OS}. Without a closed month, say the surplus is unverified and build
   no funding path on it.
3. **Size the reserve.** Convert real outgoings into months of cover. Test each
   candidate holding for liquidity in days without penalty or permission.
   Publish `wealth.reserve_target.set` so Money {OS} can test every close
   against it.
4. **Price the goals.** For each goal, target, horizon, required monthly
   contribution, and the verdict: funded, short by an amount, or funded at the
   cost of something else named explicitly.
5. **Find what breaks it.** Run the risk pass. Rank by damage, not by
   likelihood alone. Each risk ends mitigated or accepted on the record.
6. **Publish the constraints.** Emit `wealth.capital_constraints.published` for
   Capital {OS}: reserve floor, dated liquidity needs, horizon, tolerable loss.
7. **Escalate what is not yours.** Tax, insurance, estate and regulated
   suitability questions go to the adviser pack with the numbers attached.
8. **Re-run on events, not on the calendar alone.** A closed month, a new
   valuation, a position change, an income change or an expected exit all
   trigger `UPDATE`.

## Handoffs

| To | What it receives | What it expects |
|---|---|---|
| Capital {OS} | `wealth.capital_constraints.published` | a floor, a horizon and a tolerable loss, not an instruction to buy anything |
| Money {OS} | `wealth.reserve_target.set` | a monthly figure it can test a close against |
| Execution {OS} | dated work: obtain a valuation, review a policy, top up the reserve | a date and an owner |
| Goal & Life Strategy {OS} | what each life goal costs per month and what it displaces | the price of a decision, not the decision |
| Context & Memory {OS} | dated balance sheets, reserve policy, goals, accepted exposures | a basis and a date per line |
| accountant, tax professional, broker, lawyer | organised numbers plus the open questions | questions stated plainly, with the records attached |

From Money {OS} it takes `money.surplus.verified` and `money.month.closed`. From
Ownership {OS} it takes `ownership.position.valued`, never the cap table. From
Exit & Liquidity {OS} it takes `exit.proceeds.expected`, which stays in the
expected column with its probability until cash is actually received.
