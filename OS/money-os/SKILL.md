---
name: money-os
description: Personal cash flow: what comes in, what goes out, what is left. Money {OS}, unit 50 of the AGENTIK {OS} suite (06 · OWN). Use when the user asks about money or invokes /money-os.
---

# Money {OS}

Personal cash flow: what comes in, what goes out, what is left.

## When to use this

Use Money {OS} when the question is about a month of the operator's own money:

- a pile of statements, exports or receipts needs to become records
- the month has ended and nobody has said what actually happened in it
- the operator does not know what is committed to leave the account in the next
  90 days
- the operator wants to know how many months the current balances cover
- a purchase is large enough that it should be checked against the month, the
  runway and the reserve before it is made
- the accountant is asking for records and the operator is rebuilding them by
  hand

Near neighbours, and how to tell them apart:

| If the question is | The OS is |
|---|---|
| what is left this month, and what is committed next | Money {OS} |
| what am I worth, how many months of reserve, what is the money for | Wealth {OS} |
| what did the business bill, collect and hold | Revenue {OS} |
| where should the surplus be invested | Capital {OS} |
| which entity received this, and on what terms | Ownership {OS} |

The line that decides it: Money owns flow, Wealth owns position, Revenue owns
the business. A personal ledger that contains business transactions is broken.

This OS assists but does not replace a legally accountable accountant or tax
professional. It categorises for personal clarity, which is not tax
classification, and it does not decide deductibility, tax treatment or the
content of any filing. It also never moves money: it writes the instruction and
a human executes it in their bank.

## Capabilities

- Stage a statement, export, receipt or photographed document into records with
  a source and an extraction confidence per field.
- Classify transactions against categories the operator defined, learning rules
  from confirmations rather than imposing a chart of accounts.
- Close a month: reconcile every account to its statement balance, produce in,
  out, left, and surplus or deficit by category.
- Maintain the recurring obligation register and project the next 90 days of
  committed outflow.
- Compute runway from closed-month burn and current balances, showing both.
- Produce an affordability read for a named decision: the effect on this month,
  on the runway, and on the reserve contribution Wealth {OS} asked for.
- Assemble an accountant pack: organised records plus the questions this OS
  refused to answer itself.

## Procedure

1. **Establish the accounts.** Which personal accounts exist, in which currency,
   with which balance on which date. Nothing else can be trusted before this.
2. **Intake.** Take the documents as they are. Every extracted line keeps its
   source and its confidence. Low confidence stays staged.
3. **Classify.** Work the staged queue. Suggest, never assume. A line the
   operator has categorised is never re-categorised by a later rule.
4. **Reconcile.** Compare each account to its statement balance. A gap stops the
   close and is reported in currency and direction.
5. **Close.** Produce the month: in, out, left, surplus or deficit, by category,
   with the unclassified residue named rather than distributed.
6. **Publish.** Emit `money.month.closed` and `money.surplus.verified` so
   Wealth {OS} can update reserves and goals against a fact.
7. **Look forward.** Refresh the obligation calendar and the runway figure, and
   raise anything dated in the next 90 days into Execution {OS}.
8. **Escalate what is not yours.** Tax, deductibility and filing questions go
   into the accountant pack with their records attached.

## Handoffs

| To | What it receives | What it expects |
|---|---|---|
| Wealth {OS} | `money.surplus.verified`, `money.month.closed`, run rate, debt service | a closed month, not a running estimate |
| Execution {OS} | dated money tasks: a payment due, a subscription to cancel, a document to request | a date, an amount and an account |
| Context & Memory {OS} | verified transactions and closed months | a source per record |
| the operator's accountant | organised records and a question pack | records they do not have to rebuild, and questions stated plainly |

From Revenue {OS} it accepts exactly one fact, `revenue.owner_distribution.verified`.
From Wealth {OS} it accepts `wealth.reserve_target.set`. It sends nothing to
Capital {OS} directly: constraints on investable money are Wealth's to publish.
