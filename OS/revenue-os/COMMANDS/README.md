# Revenue {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install revenue-os` | Installs this OS into your environment | Once, first |
| `agentik configure revenue-os` | Collects the minimum context it needs | After install |
| `agentik run revenue-os` | Starts the OS | Every session |
| `agentik doctor revenue-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update revenue-os` | Updates to the latest version | When a release lands |
| `agentik eval revenue-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/revenue` | Opens the business money view | the ledger and CRM | cash, receivables, pipeline and the next thing that needs a decision |
| `/pipeline` | Reviews the pipeline and the forecast | commitments from Sales {OS} | pipeline state and a forecast range with its assumptions |
| `/lead` | Creates or inspects a commercial record for a lead | a person or organisation with context | a CRM record with source and stage evidence |
| `/proposal` | Builds the commercial logic of a proposal | scope and price book | a proposal with terms, figures and what happens if scope moves |
| `/invoice` | Drafts and issues an invoice | a billable event | an invoice draft, issued only on approval of the exact figures |
| `/collections` | Manages overdue receivables | the ageing view | a per-invoice action and an unsent, respectful message |
| `/business-cashflow` | Analyses business cash and runway | reconciled inflows and outflows | a cash position with runway dates and what changes it |
| `/receipt-business` | Stages a business receipt or photo | a document | a staged extraction with its source and confidence |
| `/contract` | Stages contract data as a commercial record | a signed contract | contract terms, dates and obligations, staged for confirmation |
| `/revenue-close` | Runs the monthly close | the period's records | reconciled accounts and a named exception list |
| `/revenue-scenario` | Models revenue and cash | a decision and its variables | scenarios as ranges with falsifiable assumptions |
| `/renewal` | Takes the renewal decision | delivery signals and contract terms | the decision, with the signals it accepted or overrode |

---

## The money view

### `/revenue`

Open the business money view. Cash on hand, what is owed to you, what you owe,
the pipeline, and the single thing that most needs a decision today.

```text
/revenue
```

**When to reach for it:** first thing, and whenever you are about to spend.
**Returns:** a position where every number names which number it is. Cash
received is never shown as revenue booked.

## Pipeline and commercial records

### `/pipeline`

Review the pipeline as it stands and produce a forecast that admits what it
assumes.

```text
/pipeline
/pipeline --horizon 90d
```

**When to reach for it:** weekly, and before any commitment that depends on
money arriving.
**Returns:** stage-by-stage pipeline plus a forecast range. Never a single
number alone: a forecast that states no assumptions is a wish.

### `/lead`

Create or inspect the commercial record for a lead: who they are, where they
came from, and what is actually evidenced about the opportunity.

```text
/lead "Meridian Health" --source "inbound, article on pricing"
```

**When to reach for it:** when a real opportunity appears and needs a record
rather than a memory.
**Returns:** a CRM record with source and stage evidence. A lead is a person or
organisation with context, not a counter.

### `/proposal`

Build the commercial logic of a proposal: the figures, the terms, and what
happens when scope moves.

```text
/proposal "Meridian Health" --offer retainer
```

**When to reach for it:** after Offer {OS} has fixed the scope and Pricing {OS}
has fixed the price. This command applies both; it sets neither.
**Returns:** the commercial construction: figures, payment terms, and the
consequence of a scope change, stated before it happens.

## Billing and collections

### `/invoice`

Draft an invoice against a billable event, then issue it once a human has
approved it.

```text
/invoice "Meridian Health" --milestone "phase 2 accepted"
/invoice --list overdue
```

**When to reach for it:** the moment something becomes billable, not at month
end out of habit.
**Returns:** a draft. An invoice is a legal document, so it is never issued
until a human has approved the exact figures, the exact line items and the
exact recipient.

### `/collections`

Work the overdue list. Per invoice, per customer, by age, with a respectful
message drafted for each.

```text
/collections
/collections "Meridian Health"
```

**When to reach for it:** on a fixed cadence. A receivable ages whether or not
anyone looks at it.
**Returns:** the ageing view and one action per invoice: chase, reschedule,
escalate, or write off. The message is unsent until you approve the text, and a
write-off always needs an explicit decision.

### `/receipt-business`

Stage a business receipt or a photo of one as evidence, with its source
attached.

```text
/receipt-business ./receipts/2026-08-hosting.jpg
```

**When to reach for it:** when the document exists, before the number is needed.
**Returns:** a staged extraction with the source region and a confidence level.
It is evidence, not an entry: low confidence stays staged until you confirm it.

### `/contract`

Stage a signed contract as a commercial record: parties, term, dates,
obligations and the scope it points at.

```text
/contract ./contracts/meridian-2026.pdf
```

**When to reach for it:** on signature, before the first invoice.
**Returns:** the staged terms for confirmation. Any later change to a signed
contract requires an explicit human decision.

## Cash, close and decisions

### `/business-cashflow`

Analyse what the business actually has, what is landing, and how long it lasts.

```text
/business-cashflow --horizon 6m
```

**When to reach for it:** before hiring, before committing, before a large spend.
**Returns:** a cash position reconciled to cash received, runway with dates, and
the two or three variables that move it most.

### `/revenue-close`

Run the monthly close: reconcile every account and name what does not
reconcile.

```text
/revenue-close --period 2026-07
```

**When to reach for it:** at each period end.
**Returns:** a closed month with an exception list. A period with unreconciled
accounts is not closed silently; closing it anyway takes an explicit decision.

### `/revenue-scenario`

Model a decision before you make it.

```text
/revenue-scenario "raise the retainer 20 percent, lose two clients"
```

**When to reach for it:** before any commitment whose downside you cannot
absorb.
**Returns:** scenarios as ranges, each assumption named and falsifiable, and the
assumption that most changes the answer.

### `/renewal`

Take the renewal decision, using the signals Delivery & Customer Success {OS}
sent.

```text
/renewal "Meridian Health"
```

**When to reach for it:** before the term ends, with enough time for the answer
to be no.
**Returns:** the decision (renew, renegotiate, expand or let go) citing which
delivery signals it accepted and which it overrode. Delivery owns the signals,
Revenue owns the decision, and a decision that contradicts the signals must say
so and record why.

---

## Deprecated aliases

The legacy Revenue pack also shipped four commands that now belong to other
units in the suite. They still resolve, and they route rather than execute. They
are not Revenue {OS} commands, and Revenue does not own what they produce.

| Alias | Routes to | Why it moved |
|---|---|---|
| `/offer` | Offer {OS} | Offer owns what you sell and its scope boundary |
| `/positioning` | Positioning {OS} | Positioning owns the category and the claim |
| `/pricing` | Pricing {OS} | Pricing owns what you charge, separately from what you sell |
| `/sales-call` | Sales {OS} | Sales owns the pipeline and the close |

Invoking an alias hands the request to the owning unit and says so. Revenue
consumes their outputs: the price book, the discount policy and the scope
boundary arrive here as inputs, and are never redefined here.

---

## Command summary

| Command | Does |
|---|---|
| `/revenue` | opens the business money view |
| `/pipeline` | reviews the pipeline and the forecast |
| `/lead` | creates or inspects a lead's commercial record |
| `/proposal` | builds the commercial logic of a proposal |
| `/invoice` | drafts and issues an invoice on approval |
| `/collections` | works overdue receivables |
| `/business-cashflow` | analyses business cash and runway |
| `/receipt-business` | stages a business receipt as evidence |
| `/contract` | stages contract data as a commercial record |
| `/revenue-close` | runs the monthly close |
| `/revenue-scenario` | models revenue and cash decisions |
| `/renewal` | takes the renewal decision |
