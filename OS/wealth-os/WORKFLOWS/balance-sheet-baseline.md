# Workflow: Balance sheet baseline

Produce the first dated personal balance sheet, with a valuation basis on every
line and the unvalued kept visibly unvalued.

**Trigger:** `/wealth-baseline` on first run, or after a structural change that
makes the old sheet wrong: an inheritance, a separation, a move to another
country, an entity restructure, a closed exit.

**Owner:** the operator supplies the facts. The OS refuses to fill a gap with a
plausible number.

## Steps

1. **Fix the date.** Every balance sheet is as at a date. Choose it before
   collecting anything, and use balances true on that date rather than today's.
2. **List assets.** Cash and accounts, property, vehicles, private holdings,
   pensions and long-term savings, personal receivables, and durable assets
   worth listing. Ask what would be listed if the operator had to prove what
   they own.
3. **List liabilities.** Mortgages, loans, credit balances, personal tax owed,
   personal guarantees given. For each: balance, rate, term, and the instalment
   Money {OS} already tracks in its obligation register.
4. **Attach a valuation basis to every line.** Market price, purchase cost,
   professional appraisal, or owner estimate, each with the date that basis was
   established. Bases are never mixed silently: four confidences are not one.
5. **Route the positions.** A stake in a company is taken from Ownership {OS} as
   `ownership.position.valued`. An intellectual property asset is taken from
   IP & Asset {OS} as `ipasset.valuation.recorded`. This OS does not value them
   itself.
6. **Refuse business cash.** A company bank balance is not a personal asset. It
   enters only as a valued ownership position, and Revenue {OS} keeps the rest.
7. **Separate the expected.** Expected exit proceeds, unvested equity,
   unexercised options, a promised bonus and an unpaid personal invoice go into
   an expected column with a probability. They are not in net worth.
8. **Park the unvalued.** Anything with no defensible basis goes on the unvalued
   list at zero, named, with what it would take to value it. Raise that work
   into Execution {OS}.
9. **Compute and attribute.** Net worth equals valued assets minus liabilities.
   Show the totals per basis type, so the reader can see how much of the number
   rests on an owner estimate.
10. **Publish.** Emit `wealth.networth.updated` and write the dated sheet to
    Context & Memory {OS}.

## Completion test

The baseline is done when:

- the sheet carries a single as-at date and every line's basis carries its own
  date
- no line has an amount without a basis, and every unvalued item is on the
  unvalued list at zero
- no business account balance appears as a personal asset
- nothing from the expected column is inside net worth
- the totals per basis type are shown, so the confidence of the headline number
  is visible rather than implied
- `wealth.networth.updated` has been emitted
