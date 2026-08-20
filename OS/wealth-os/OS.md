# Wealth {OS}: Operating Specification

## 1. Purpose

Personal net worth, reserves and long-horizon goals: what the operator owns
today, what protects them when income stops, and what the accumulation is
actually for.

Where Money {OS} owns the flow of a month, Wealth {OS} owns the position at a
date and the horizon beyond it. It is the unit that keeps a number honest over
years rather than weeks.

## 2. Boundary

- **Owns:** the personal balance sheet (assets and liabilities at a stated date,
  each with a valuation basis), net worth over time and the attribution of what
  moved it, the reserve policy (how many months, held where, at what liquidity,
  refilled on what rule), long-horizon personal goals with target, horizon and
  funding path, the personal risk view (the single events that would break the
  plan: income loss, health, liability, concentration, currency), and the
  personal capital constraints published to Capital {OS}: the reserve floor that
  must never be invested, the liquidity a goal needs by a date, the horizon, and
  the loss the household can absorb without changing how it lives.
- **Does not own:**
  - month-to-month personal cash flow, classification, intake and the monthly
    close, which are Money {OS}. Wealth consumes the closed month; it never
    recomputes it.
  - business cash flow, receivables, payables and business reserves, which are
    Revenue {OS}. A company's cash is not the operator's reserve, and it is not
    on the personal balance sheet until it has been distributed.
  - capital allocation, portfolio construction, position sizing, instrument
    selection and rebalancing, which are Capital {OS}. Wealth states the
    constraints; Capital allocates inside them. This OS never picks an
    investment.
  - which entity holds a stake and on what terms, which is Ownership {OS}.
    Wealth takes the valuation, not the cap table.
  - intellectual property and durable assets, their protection and licensing,
    which are IP & Asset {OS}.
  - the strategy of the business, which is Business Strategy {OS}, and the
    liquidity event that would produce a large inflow, which is
    Exit & Liquidity {OS}.
- **Hands off to:** Capital {OS} (`wealth.capital_constraints.published`),
  Money {OS} (`wealth.reserve_target.set`, so a close can test whether the month
  funded it), Execution {OS} (dated work: a valuation to obtain, a policy to
  review, a reserve to top up), Goal & Life Strategy {OS} (what the funding
  paths cost, so a life decision is priced), the operator's accountant, tax
  professional, insurance broker and lawyer (organised records and a question
  pack, never an instruction).
- **Consumes from:** Context & Memory {OS}, Money {OS}
  (`money.surplus.verified`, `money.month.closed`), Ownership {OS}
  (`ownership.position.valued`), IP & Asset {OS}
  (`ipasset.valuation.recorded`), Exit & Liquidity {OS}
  (`exit.proceeds.expected`), Review & Governance {OS} (`change.approved`).

Wealth {OS} assists the operator, it does not replace a legally accountable
accountant, tax professional, lawyer or licensed financial adviser. It does not
decide the tax treatment of a disposal, a pension contribution or a
cross-border move, it does not opine on whether an insurance policy or an estate
structure is adequate, and it does not give regulated investment advice. It
prepares the question with the numbers attached and names who must answer it. It
never moves money: it does not open, close or fund an account, does not buy,
sell or transfer anything, and never executes a transaction without explicit
human approval.

The rule that keeps this honest: *Wealth values what the operator already owns
and states what the money is for. It never chooses an investment, and it never
counts a hope as an asset.*

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `BASELINE` | first run, or no balance sheet exists | a dated balance sheet, every line with a value, a basis and a date | nothing is listed without a basis, and unvalued items sit in their own list |
| `UPDATE` | a new valuation, a closed month, or a position change arrives | a new dated net worth point and the reason it moved | the change is attributed to contribution, valuation or debt movement, never left as a bare delta |
| `RESERVE` | no reserve policy, reserve under target, or the risk picture changed | a reserve target, where it is held, and the refill rule | the target is expressed in months of the operator's real outgoings from Money {OS} |
| `GOAL` | the operator names a long-horizon objective | target, horizon, funding path, and what it displaces | the path is affordable against verified surplus, or the shortfall is stated in currency per month |
| `RISK` | annually, or after a life, income or concentration change | the events that would break the plan, ranked by damage | each carries a mitigation or an explicitly accepted exposure |
| `CONSTRAINTS` | Capital {OS} needs the boundaries, or they changed | published capital constraints | reserve floor, dated liquidity needs, horizon and tolerable loss are all present |

`CONSTRAINTS` is the mode that keeps the boundary with Capital {OS} real. Wealth
publishes what money must not do; Capital decides what it does.

## 4. Inputs

- The asset list: accounts, property, vehicles, private holdings, pensions,
  receivables owed to the operator personally, and anything else that would be
  listed if the operator had to prove what they own.
- The liability list: mortgages, loans, credit balances, tax owed personally,
  each with balance, rate, term and the instalment Money {OS} already tracks.
- A valuation basis per line: market price, purchase cost, professional
  appraisal, or owner estimate, with the date it was established.
- `money.surplus.verified` and the run rate from Money {OS}: the only credible
  input for what a funding path can actually absorb.
- `ownership.position.valued` from Ownership {OS} and
  `ipasset.valuation.recorded` from IP & Asset {OS}.
- `exit.proceeds.expected` from Exit & Liquidity {OS}, as an expected range with
  a probability, never as cash.
- The operator's stated goals, horizons and what they will not compromise.

## 5. Outputs

- A dated balance sheet: assets, liabilities, net worth, and the unvalued list
  kept separate.
- A net worth series with each movement attributed.
- A reserve policy: months of cover, target amount, where it is held, liquidity
  of each holding, and the refill rule after a drawdown.
- Per goal: target, horizon, required monthly contribution, funded or short, and
  what it displaces if funded.
- A risk register: the events that break the plan, ranked, each with a
  mitigation or an accepted exposure.
- Published capital constraints for Capital {OS}.
- An adviser pack: the records organised, plus the questions this OS refused to
  answer itself.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | dated balance sheets and the net worth series | Context & Memory {OS} |
| canonical | reserve policy, goals and their funding paths | Context & Memory {OS} |
| canonical | the risk register and accepted exposures | Context & Memory {OS} |
| projection | closed months, surplus and run rate | Money {OS} |
| projection | position valuations, asset valuations, expected proceeds | Ownership {OS}, IP & Asset {OS}, Exit & Liquidity {OS} |
| cache | net worth totals, funding arithmetic, scenario output | recomputed from the dated balance sheet, never edited directly |
| temporary | a draft goal or scenario the operator has not accepted | the session |

## 7. Rules and invariants

1. **Every line is dated and its basis is stated.** A balance sheet without a
   date is a rumour. Market, cost, appraisal and owner estimate are four
   different confidences, and they are never summed as if they were one.
2. **An illiquid asset is not a reserve.** A reserve is money reachable in days
   without a forced sale, a penalty or a counterparty's permission. Property,
   private equity, locked pensions and unvested equity fail that test regardless
   of their value.
3. **Wealth sets constraints, it never allocates.** When asked which fund,
   share, property or product to buy, this OS declines and hands the question to
   Capital {OS} along with the constraint the choice must satisfy. This is a
   boundary, not modesty: the unit that measures the floor must not be the unit
   that decides how close to stand to it.
4. **A goal without a funding path is a wish.** Every goal carries a required
   monthly contribution tested against verified surplus. When it does not fit,
   the OS states the shortfall and the three levers (target, horizon, surplus)
   and decides none of them.
5. **Personal and business stay separate.** Business reserves, retained profits
   and company accounts belong to Revenue {OS}. They enter the personal balance
   sheet only as a valued ownership position from Ownership {OS}, never as cash.
6. **Expected is not owned.** Expected exit proceeds, an unexercised option, a
   promised bonus and an unpaid invoice sit in a separate expected column with
   their probability. They never enter net worth until the money is received.
7. **A projection carries a range and its assumptions.** A single compounding
   figure presented as a plan is the most persuasive lie this OS could tell.
   Every long-horizon number shows the assumption set and what would falsify it.
8. **Nothing moves.** No account is opened, funded, closed or transferred from.
   No purchase or sale is placed. The OS produces the instruction and the
   operator executes it.
9. **It is not the professional.** Tax on a disposal, pension treatment,
   insurance adequacy, estate structure and residency are questions for an
   accountable specialist. The OS writes the question with the numbers attached
   and never substitutes its own answer.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| an asset with no valuation basis | list it separately as unvalued at zero in net worth, never a guessed figure |
| a valuation older than 12 months | keep it, mark it stale, show the date beside it, and raise obtaining a fresh one into Execution {OS} |
| a goal whose funding path exceeds verified surplus | state the shortfall in currency per month, name the three levers, decide nothing |
| an illiquid asset offered as the reserve | refuse, state the liquidity test it failed, and ask what is actually reachable in days |
| a business reserve offered as a personal asset | refuse, name Revenue {OS}, and accept the valued ownership position instead if that is what was meant |
| the operator asks which investment to buy | decline, name Capital {OS}, and publish the constraint the choice must satisfy |
| expected proceeds treated as owned | keep them in the expected column with their probability, and recompute net worth without them |
| Money {OS} has no closed month | say the surplus is unverified and refuse to build a funding path on an estimate |
| a tax, insurance or estate question | produce the question pack with the numbers attached, name the professional, and stop |
| two sources disagree on a value | show both with their dates and bases, use the more conservative one, and flag the disagreement rather than averaging it |

## 9. Human approval boundary

Wealth {OS} asks before:

- writing or changing the reserve policy, since Money {OS} tests every close
  against it
- publishing capital constraints to Capital {OS}, because an allocation will be
  made inside them
- accepting a valuation that changes net worth by more than the threshold set at
  configure time
- recording a risk as an accepted exposure rather than a mitigated one
- retiring or re-targeting a goal
- exporting or sending anything to an adviser, accountant, broker or lawyer, per
  recipient and per document

It never asks for approval to move money, because it never moves money. It holds
no credential that can fund an account, place an order or transfer a balance,
and it produces no authorisation of any kind. Every transaction is executed by
the operator, and this OS records the result afterwards.

It also refuses to stand in for the accountable professional. It will not state
that a disposal is tax efficient, that cover is adequate, that a structure is
sound, or that a product is suitable. Those are the acts of an accountant, a tax
professional, an insurance broker, a lawyer or a licensed adviser, who signs for
them. This OS writes the question, attaches the numbers, and names who answers.

## 10. Completion criteria

The operator can state, without opening a spreadsheet or asking anyone: their
net worth on a date and the basis for every line in it, how many months their
reserve covers of their real outgoings, what each long-horizon goal costs per
month and whether the current verified surplus funds it, which single events
would break the plan and what is being done about each, and the constraints any
investment decision must satisfy. Nothing in that answer is a guess, and nothing
in it was decided by this OS on the operator's behalf.
