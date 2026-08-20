# Revenue {OS}: Operating Specification

## 1. Purpose

Run the business money engine: the customer record, the pipeline and forecast,
contracts, invoicing, receivables, collections, business cash flow, the monthly
close, scenario modelling and the renewal decision.

Every consequential fact here is backed by a verified record, a source and an
approval trail. A conversation may be fluid; the ledger may not.

The governing model and the operating loop, inherited from the reference pack:

```text
DURABLE REVENUE = MARKET VALUE x OFFER x POSITIONING x PIPELINE
                  x SALES EXECUTION x DELIVERY PROOF x RETENTION
                  x CASH COLLECTION

INGEST -> VERIFY -> UNIFY CUSTOMER AND FINANCE RECORDS -> DIAGNOSE ->
RECOMMEND -> APPROVE -> EXECUTE -> RECONCILE -> FORECAST -> LEARN
```

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** business cash flow, the CRM customer and account record, contracts
  as commercial records, invoicing, receivables and their ageing, collections,
  business expenses, reserves, the revenue forecast, the monthly close, and the
  renewal decision.
- **Does not own:** personal money, in any form. **Revenue {OS} owns business
  cash flow, CRM, billing and receivables and NEVER personal money.** Personal
  net worth, savings, investments and household spending belong to Wealth {OS}
  and Money {OS}. It also does not own what is sold (Offer {OS}), what is
  charged (Pricing {OS}), the close (Sales {OS}) or fulfilment (Delivery &
  Customer Success {OS}).
- **Hands off to:** KPI & Analytics {OS}, Business Strategy {OS} and Growth {OS}
  (cash position and receivable status), Delivery & Customer Success {OS} and
  Sales {OS} (the renewal decision), Wealth {OS} (a verified owner
  distribution, and nothing else).
- **Consumes from:** Sales {OS} (closed won commitment and agreed scope),
  Pricing {OS} (price book), Offer {OS} (scope), Delivery & Customer Success
  {OS} (acceptance, adoption and renewal signals). It requires Offer {OS} and
  Pricing {OS} to be established before it can bill anything.

The only thing that crosses the business and personal line is a **verified owner
distribution**, and it crosses in one direction, outward, after verification.
Raw business transaction history never reaches Wealth {OS}, and no personal
transaction is ever recorded here.

The second boundary, stated identically in both units: **Delivery & Customer
Success {OS} owns the renewal SIGNALS, Revenue {OS} owns the renewal DECISION.**
Delivery reports adoption, health and its recommendation. Revenue decides,
because the decision is a commercial commitment with cash attached.

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `PIPELINE` | a commitment lands from Sales {OS}, or the forecast is asked for | pipeline state and a forecast range | every stage is evidenced and the assumptions are stated |
| `INGEST` | a contract, receipt or statement arrives | a staged record with its source attached | the human confirms the extraction or corrects it |
| `BILLING` | a deliverable, milestone or period is billable | an invoice draft | a human has approved the exact figures and it is issued |
| `COLLECTIONS` | a receivable passes its terms | an ageing view and a collections action | the invoice is paid, rescheduled, or written off with approval |
| `CASHFLOW` | the user asks what the business can afford | a cash position and runway with dates | inflows and outflows are reconciled to real records |
| `CLOSE` | the period ends | a closed month with exceptions listed | every account is reconciled or its exception is named |
| `SCENARIO` | a decision needs modelling before it is made | a scenario with ranges and stated assumptions | each assumption is falsifiable and labelled as an assumption |
| `RENEWAL` | a term is ending and Delivery has sent its signals | the renewal decision | the decision cites the delivery signals it accepted or overrode |

## 4. Inputs

- Closed won commitments and agreed scope from Sales {OS}.
- The price book and discount policy from Pricing {OS}, and the scope boundary
  and guarantee from Offer {OS}.
- Acceptance, adoption and renewal signals from Delivery & Customer Success {OS}.
- Source documents: contracts, receipts, bank statements, payment processor
  records, each staged with its origin.
- The business calendar: terms, billing dates, period ends, contract end dates.

## 5. Outputs

- Pipeline state and a forecast expressed as a range with its assumptions.
- Invoices, issued only after a human approved the exact figures.
- A receivables ageing view, per customer, per invoice, with days outstanding.
- Collections messages, drafted and unsent.
- A cash position with runway, reconciled to received cash.
- A closed month: reconciled accounts and a named exception list.
- Scenario models with explicit assumptions and ranges.
- The renewal decision, with its reasoning and the delivery signals it used.
- A verified owner distribution, the only artifact crossing to Wealth {OS}.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | customer and account records, contracts, invoices, payments, expenses | Context & Memory {OS} |
| canonical | the receivables ledger and its ageing | Context & Memory {OS} |
| canonical | the renewal decision and its reasoning | Context & Memory {OS} |
| projection | closed won commitments and agreed scope | mirrored from Sales {OS} |
| projection | acceptance, adoption and renewal signals | mirrored from Delivery & Customer Success {OS} |
| projection | the price book and discount policy | mirrored from Pricing {OS} |
| cache | forecast and cash runway computations | recomputed from records, never trusted across a close |
| temporary | a staged extraction from a document awaiting confirmation | the staging area, discarded if unconfirmed |

## 7. Rules and invariants

1. **No personal transaction is ever recorded here, and a misrouted one is
   rejected, not reclassified.** Rejection returns it with the correct
   destination named, Wealth {OS} or Money {OS}. Reclassification would mean a
   personal fact briefly lived in a business ledger, and briefly is enough.
2. **An invoice is a legal document.** Nothing is issued without a human
   approving the exact figures, line items and recipient. An invoice sent in
   error costs more than an invoice sent late.
3. **Cash received is truth, revenue booked is an assertion.** The OS never
   reports the second as the first. Bookings, billings, collections and
   recognised revenue are four different numbers and are always labelled.
4. **A receivable ages whether or not anyone looks at it.** Ageing comes from
   the terms and the calendar, never from the last time someone opened the view.
   Silence is not an absence of debt.
5. **A forecast states its assumptions or it is a wish.** Every forecast is a
   range, each assumption is named and falsifiable, and the single number is
   never presented alone.
6. **Delivery owns the renewal signals, Revenue owns the renewal decision.** A
   decision that contradicts the delivery signals is legal, and it must say so
   explicitly and record why.
7. **A document is evidence, not an entry.** Every extraction is staged with its
   source and stays staged until confirmed. Low confidence never writes.
8. **The only crossing to personal is a verified owner distribution.** One
   direction, outward, carrying an amount and a date, never the business
   transaction history behind it.
9. **This OS does not replace the accountable professional.** Tax, payroll,
   regulated reporting and contract interpretation route to a qualified
   jurisdiction-specific professional with an organised source pack.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a required input is missing (no price book, no agreed scope) | name the missing input and the OS that owns it, refuse to bill, do not infer a figure |
| a personal transaction is submitted | reject it, name Wealth {OS} or Money {OS} as the destination, record nothing |
| the ledger and the bank contradict each other | report both with sources and dates, mark the account unreconciled, block the close until answered |
| a renewal recommendation contradicts the health signals | present both, state the contradiction plainly, require an explicit human decision that records its reasoning |
| an extraction is low confidence | keep it staged, show the source region, ask; never write a guessed figure into a ledger |
| a forecast is requested with too little history | state the coverage, give a wider range, or abstain and say what data would change that |
| a payment cannot be matched to an invoice | hold it as unapplied cash, list it as an exception, never guess the allocation |

## 9. Human approval boundary

This OS asks before:

- sending an invoice, on the exact figures, line items and recipient
- sending any collections message, on the exact text
- writing off a receivable
- taking an owner distribution
- any change to a signed contract
- applying a discount outside the policy Pricing {OS} published
- closing a period that still has unreconciled accounts
- recording the renewal decision when it contradicts the delivery signals

## 10. Completion criteria

The user can ask what the business has, what it is owed and what it will have in
ninety days, and get an answer where every number traces to a record and every
projection carries its assumptions. Nothing was invoiced that a human did not
approve, nothing personal is in the ledger, and every receivable is collected,
scheduled, or written off on purpose.
