---
name: revenue-os
description: Operate a conversational revenue brain and governed database for offers, positioning, pricing, leads, sales pipeline, customers, contracts, invoicing, payments, expenses, cash flow, reserves, forecasting, retention and expansion. Contains 24 specialist agents, 40 skills, 10 protocols and 14 JSON schemas. Use for CRM work, pricing and offer design, sales calls, invoicing and collections, cash flow forecasting, monthly revenue close, or renewal and expansion planning. Trigger words: revenue, offer, pricing, pipeline, invoice, collections, cash flow, forecast, CRM, sales call, proposal, renewal; FR: chiffre d'affaires, tarification, pipeline commercial, facture, recouvrement, prevision de tresorerie, CRM, appel de vente, renouvellement.
---

# Revenue {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/revenue` | dashboard | Open revenue brain |
| `/offer` | offer | Create or audit an offer |
| `/positioning` | offer | Define category and differentiation |
| `/pricing` | offer | Build pricing architecture |
| `/pipeline` | pipeline | Review CRM and forecast |
| `/lead` | pipeline | Create or analyze a lead |
| `/sales-call` | sales | Prepare or debrief a call |
| `/proposal` | sales | Create proposal and commercial logic |
| `/invoice` | billing | Create or inspect invoice |
| `/collections` | billing | Manage overdue receivables |
| `/business-cashflow` | finance | Analyze business cash flow |
| `/receipt-business` | ingest | Stage business receipt/photo |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

## Suite contract

Unit 38 of the AGENTIK {OS} suite (04 · GROW). Inside the suite this pack is
narrowed: Revenue {OS} owns BUSINESS cash flow, CRM, billing and receivables and
NEVER personal money. Personal money is Wealth {OS} and Money {OS}. The only
thing that crosses that line is a verified owner distribution, and it crosses in
one direction, outward, after verification. The formal specification is
[OS.md](OS.md).

Four commands the legacy pack shipped now belong elsewhere in the suite and
survive here only as deprecated aliases that route to the owning unit:
`/offer` routes to Offer {OS}, `/positioning` routes to Positioning {OS},
`/pricing` routes to Pricing {OS}, and `/sales-call` routes to Sales {OS}.

### When to use this

Reach for Revenue {OS} when the question is about business money that already
exists or is owed: what is in the pipeline and what the forecast really says,
what has been invoiced, what is overdue, what the business can afford, what the
month closed at, what a decision does to cash, and whether a contract renews.

Near neighbours it is confused with:

- **Wealth {OS}** and **Money {OS}** own personal money. A transaction submitted
  to the wrong one is rejected, not reclassified.
- **Sales {OS}** owns the pipeline up to the close. Revenue takes the closed won
  commitment and everything downstream of it.
- **Pricing {OS}** owns what is charged. Revenue applies the price book and
  reports the discount history back; it does not set price.
- **Offer {OS}** owns what is sold and its scope boundary. Revenue bills against
  that scope and never redefines it.
- **Delivery & Customer Success {OS}** owns fulfilment, adoption and the renewal
  SIGNALS. Revenue owns the renewal DECISION.

The discriminating question: **is money changing hands, or being owed, on the
business side?** If yes, this OS. If it is personal money, it is the wrong OS
and the request is rejected with the right destination named.

### Capabilities

- Maintain the CRM customer and account record as one commercial truth.
- Review pipeline state and produce a forecast as a range with stated assumptions.
- Stage a contract or a receipt from its source document, with confidence shown.
- Draft an invoice for human approval on the exact figures, then issue it.
- Age receivables from the terms and the calendar, independent of who is looking.
- Draft respectful collections messages, unsent, escalating by age.
- Analyse business cash flow and runway, reconciled to cash actually received.
- Run the monthly close and name every exception rather than smoothing it.
- Model revenue and cash scenarios with falsifiable assumptions.
- Take the renewal decision on the delivery signals, and record the reasoning.
- Emit a verified owner distribution, the only artifact crossing to Wealth {OS}.

### Procedure

1. Classify the request and reject anything personal, naming Wealth {OS} or
   Money {OS} as the destination. Record nothing from a rejected request.
2. Retrieve the required inputs: agreed scope from Sales {OS}, price book from
   Pricing {OS}, scope boundary from Offer {OS}, delivery signals from Delivery
   & Customer Success {OS}. Name any that are missing and stop rather than infer.
3. Ingest and stage any source document with its origin and a confidence level.
   Low confidence stays staged.
4. Separate cash received from revenue booked from revenue billed from revenue
   committed, and label every number with which one it is.
5. Choose the smallest sufficient mode and run it.
6. Produce the artifact with its reconciliation source: invoice, ageing view,
   cash position, close pack, scenario, or renewal decision.
7. Present anything irreversible for explicit approval on the exact figures or
   the exact text. Nothing is sent or issued before that.
8. Reconcile after execution: payment to invoice, invoice to contract, contract
   to scope. Unmatched items become named exceptions, never guesses.
9. Emit the handoffs below, and only those.

### Handoffs

- **KPI & Analytics {OS}** receives cash position and receivable status. It
  expects each metric to carry its definition and reconciliation source.
- **Business Strategy {OS}** receives cash position and receivable status. It
  expects forecasts as ranges with the assumptions attached.
- **Growth {OS}** receives cash position and receivable status, plus revenue and
  retention by cohort. It expects cohorts defined the same way twice running.
- **Delivery & Customer Success {OS}** receives the renewal decision. It expects
  the decision to cite which of its signals were accepted or overridden.
- **Sales {OS}** receives the renewal decision, and hands Revenue the closed won
  commitment and agreed scope in return.
- **Wealth {OS}** receives a verified owner distribution and nothing else. It
  expects an amount and a date, never business transaction history.
