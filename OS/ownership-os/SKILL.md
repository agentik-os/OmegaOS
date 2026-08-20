---
name: ownership-os
description: What you own, through which entity, and on what terms. Ownership {OS}, unit 52 of the AGENTIK {OS} suite (06 · OWN). Use when the user asks about ownership or invokes /ownership-os.
---

# Ownership {OS}

What you own, through which entity, and on what terms.

## When to use this

Reach for this OS when the question is structural rather than financial:

- You hold positions in more than one entity and cannot state your percentage in
  each without opening a folder.
- You signed a shareholder or operating agreement and want to know what you
  actually agreed to on drag, tag, preference, pre-emption and transfer.
- A grant is vesting and you need to know what is vested today, what is behind a
  cliff, and what accelerates on a change of control.
- A round, a transfer or a new partner is about to change a cap table and you
  want the before and after with the denominator stated.
- You are about to sell, and the buyer's counsel will ask who has to sign and
  which consents and waivers a transaction requires.
- You keep missing annual filings, register updates or beneficial-ownership
  declarations, and want a calendar with a name against every line.

Near neighbours, and how to tell them apart:

| If the question is | Use |
|---|---|
| what came into and went out of my personal accounts | Money {OS} |
| what my net worth is, what my reserves are, what my long-horizon goals need | Wealth {OS} |
| who owns the trademark, and how do I license it | IP & Asset {OS} |
| what should this business do next, and is it an asset or a job | Business Strategy {OS} |
| how do I prepare, time and run the sale | Exit & Liquidity {OS} |
| what does the business have invoiced, collected and outstanding | Revenue {OS} |
| how should I split capital across bets | Capital {OS} |
| how should I structure the company I am buying | Deal Structuring {OS} |

This OS assists a lawyer, a corporate secretary, an accountant and a tax
professional, and replaces none of them. It maintains a register and a calendar.
It does not give legal advice, does not file with a company registry, does not
draft or execute a share transfer or subscription, does not sign a resolution,
does not certify a statutory register, and does not opine on the tax treatment
of a structure or a distribution. Those acts are performed by a named licensed
professional. It also never executes a transaction and never moves money without
explicit human approval, and in fact never executes a transaction or moves money
at all.

## Capabilities

- Build an entity map across companies, partnerships, trusts and personal
  holdings, with jurisdiction and registration number per entity.
- Maintain a cap table per entity with class, quantity, percentage, denominator
  and verification state on every line.
- Extract a terms register from shareholder, operating and subscription
  agreements: voting, liquidation preference, drag, tag, anti-dilution,
  pre-emption, transfer restriction and information rights, each with its clause
  reference.
- Project vesting for any grant to any date, including cliff, acceleration
  triggers, exercise price and expiry.
- Reconcile the working register against source documents and produce a
  line-level discrepancy report.
- Build a consent map for a proposed transaction: who must sign, who must waive,
  and which clause creates the requirement.
- Maintain an obligations calendar per entity and jurisdiction, with a lead time
  and a responsible human on every entry.
- Prepare the question pack a lawyer, corporate secretary, accountant or tax
  adviser needs, so their time is spent answering rather than reconstructing.

## Procedure

1. **Enumerate entities before positions.** Ask for every legal entity the user
   is connected to, including dormant ones and ones held through a trust or a
   spouse. Record type, jurisdiction and registration number, or mark unknown.
2. **Attach positions to entities.** For each entity, capture holder,
   instrument, quantity and class. Compute no percentage yet.
3. **Establish the denominator.** For each entity, determine issued, outstanding
   and fully diluted counts, and whether an unallocated pool is inside the fully
   diluted figure. Only now compute percentages.
4. **Ingest source documents.** Extract the terms register clause by clause.
   Anything not found in the document is recorded as absent, with the document
   searched named.
5. **Mark verification state on every fact.** `verified` requires a document and
   a clause or page reference. `user-asserted` is recorded and labelled.
   Anything else is unknown and stays visibly unknown.
6. **Reconcile against the statutory register.** Where the working register and
   the statutory register disagree, the statutory register wins and the
   difference is raised, not silently corrected.
7. **Build the calendar.** For each entity and jurisdiction, list the filings and
   register-maintenance obligations, with dates, lead times and a named
   responsible human. Confirm the rules with the corporate secretary rather than
   inferring them.
8. **Flag and route.** Terms that are unusual, punitive, expiring or in conflict
   are flagged with the clause reference and routed to the named professional as
   a written question, not answered here.
9. **Emit.** Push valuations to Wealth {OS}, consent maps to Exit & Liquidity
   {OS}, entity placement questions to IP & Asset {OS}, and dated tasks to
   Execution {OS}, each only after the approval in section 9 of `OS.md`.

## Handoffs

| Receives | What it gets | What it expects |
|---|---|---|
| Wealth {OS} | `ownership.position.valued`: entity, instrument, quantity, percentage, denominator, verification state | a position it can place on the personal balance sheet, with its uncertainty attached. It values, this OS does not |
| Exit & Liquidity {OS} | a consent map: signatories, waivers required, and the clause creating each requirement | to know before it opens a process who can block it |
| IP & Asset {OS} | the entity map plus jurisdiction per entity | to decide which entity should hold a given asset or registration |
| Execution {OS} | `ownership.obligation.due`: obligation, date, lead time, jurisdiction, responsible human | a task with an owner and a deadline, not a reminder |
| Review & Governance {OS} | a proposed register change with its evidence | to return `change.approved` before the register mutates |
| The user's lawyer, corporate secretary, accountant or tax adviser | the register extract, the source documents and the specific written question | to answer rather than to reconstruct. This is the handoff that matters most, and it is a human one |
