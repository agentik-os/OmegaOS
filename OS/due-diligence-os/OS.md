# Due Diligence {OS}: Operating Specification

## 1. Purpose

Verify what is being claimed about an opportunity before the point of no
return, using sources that are not the seller.

Scope the work to the questions whose answers could change the decision, run
the workstreams, log every answer with its source and date, and register
findings that each carry a severity and an explicit consequence for the deal.

## 2. Boundary

- **Owns:** the diligence plan and its budget, the information request list and
  its chase state, the commercial, financial, legal, technical, people and
  customer reference workstreams, the evidence log, the findings register with
  severity and consequence, the red flag escalation path, and the closing
  report with its conditions to completion.
- **Does not own:** the prediction that the bet will work (Investment Thesis
  {OS}), the terms, instruments or price adjustments that findings argue for
  (Deal Structuring {OS}), the amount committed and its approval (Capital
  {OS}), the sourcing and screening that produced the opportunity (Deal Flow
  {OS}), the negotiation calendar and the closing sequence on a named target
  (Acquisition {OS}), and the quality of earnings opinion, the legal opinion or
  the audit, which belong to named professionals.
- **Hands off to:** Deal Structuring {OS} (findings with their consequences),
  Capital {OS} (the verified basis and the conditions attached to any
  commitment), Acquisition {OS} (the conditions that must be satisfied before
  completion), Investment Thesis {OS} (verified facts that claims may rest on),
  and Context & Memory {OS} (the evidence log and the findings register).
- **Consumes from:** Deal Flow {OS} (`dealflow.opportunity.qualified`),
  Investment Thesis {OS} (`thesis.drafted`, `thesis.kill_criteria.set`),
  Acquisition {OS} (`acquisition.loi.prepared`,
  `acquisition.exclusivity.entered`), Capital {OS} (`capital.policy.set`), and
  Review & Governance {OS} (`change.approved`).

**Most often confused with Investment Thesis {OS}.** Diligence verifies what is
claimed to be true today and returns a finding with a source; the thesis states
what must become true in the future and returns a claim with kill criteria. If
the question can be settled by a document, a system extract, a public record or
a third party call, it is diligence. If it can only be settled by waiting, it
is a thesis claim. This OS is also not Deal Structuring {OS}: diligence
produces findings, structuring decides what those findings do to the terms. A
finding says the customer concentration is 61 percent in one account with a
30 day termination clause. What that does to price, escrow or a condition
precedent is decided in Deal Structuring {OS}, not here.

**The money constraint.** This OS coordinates the accountant who signs the
quality of earnings, the lawyer who gives the legal opinion, the auditor and
the regulated adviser. It never replaces them and never issues a legal, tax or
accounting conclusion of its own: where one is required, it records the
question, names the profession that must answer it, and marks the item open
until that professional's written answer arrives. Nothing it produces is
investment advice or a regulated recommendation. It never transmits an
information request, a finding, a red flag or a diligence report to a
counterparty, an adviser or any third party without explicit human approval,
and it never signs, executes or funds anything.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `PLAN` | an opportunity is qualified and a commitment is contemplated | the scoped diligence plan with a time and cost budget, and the decision relevance of every question | every question is traced to a decision it could change, and the budget is stated |
| `REQUEST` | a plan exists | the information request list, with owner, format, due date and chase state | the list is issued to the human who will send it, and every item has an owner and a date |
| `WORKSTREAM` | requests are out, or a stream can start on public sources | per stream working papers across commercial, financial, legal, technical, people and customer references | each stream states its questions answered, unanswered and refused |
| `EVIDENCE` | any answer arrives | the evidence log entry: the answer, its source, its date, its confidence and whether the source is the seller | every entry carries a source that is named, dated and classified |
| `FINDINGS` | evidence contradicts a claim or reveals a risk | the findings register: severity plus the explicit consequence for price, structure, condition or walk | every finding has a severity and a consequence, and none is filed as a note |
| `ESCALATE` | a finding meets the red flag threshold | a stopping escalation to the decision maker, with the evidence attached | the human has decided to continue, restructure, or stop, and that decision is recorded |
| `CLOSE` | the plan's questions are answered, budget is spent, or a stop is called | the diligence report and the conditions that must be satisfied before completion | every question is marked answered, unanswered or refused, and each condition has an owner |

Most people start in `PLAN` far too late, after a request list has already been
sent. If a list is already out, run `PLAN` anyway and use it to decide which of
the answers you are waiting for actually matter: an unscoped request list is
the main reason diligence runs out of time before it reaches the questions that
decide the deal.

## 4. Inputs

- The opportunity, its stage and the intended decision, from Deal Flow {OS} or
  from Acquisition {OS} for a named target.
- The thesis claims that depend on present-day facts, from Investment Thesis
  {OS}, which is where decision relevance comes from.
- The time and cost budget for diligence, set by the user against the
  commitment being contemplated and the policy in Capital {OS}.
- Seller supplied material: data room documents, management accounts,
  management assertions and presentations, all classified as seller sourced.
- Independent sources: statutory filings, registries, court and lien searches,
  system extracts observed directly, customer and supplier references, employee
  conversations, and third party market data.
- The written work of the named professionals: the accountant's quality of
  earnings, the lawyer's report, the auditor's opinion, the technical
  specialist's assessment, each attached as their document rather than
  summarised into a fact.

## 5. Outputs

- The diligence plan, with each question's decision relevance and the budget,
  stored under `DILIGENCE/<target>/plan.md`.
- The information request list with chase state, `DILIGENCE/<target>/requests/`.
- Per stream working papers, `DILIGENCE/<target>/workstreams/<stream>.md`.
- The evidence log: one entry per answer with source, date, confidence and
  seller or independent classification.
- The findings register: severity, evidence reference, and the consequence for
  price, structure, condition or walk.
- Red flag escalations with their decision record.
- The diligence report, including an explicit list of what could not be
  verified, and the conditions to completion with owners.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the diligence plan and its decision relevance mapping | Context & Memory {OS} |
| canonical | the evidence log with sources, dates and classifications | Context & Memory {OS} |
| canonical | the findings register with severities and consequences | Context & Memory {OS} |
| canonical | red flag escalations and the human decision on each | Context & Memory {OS} |
| canonical | conditions to completion and their owners | Context & Memory {OS} |
| projection | the thesis claims being verified | Investment Thesis {OS} |
| projection | the deal calendar and exclusivity dates | Acquisition {OS} |
| projection | the contemplated commitment and its policy limits | Capital {OS} |
| cache | request list chase status and overdue counts | recomputed from request dates |
| temporary | working notes from a call before the evidence entry is written | the session, promoted into the evidence log or discarded |

## 7. Rules and invariants

1. **Decision relevance scopes the work.** Every question on the plan names the
   decision it could change and how. A question whose answer cannot change
   price, structure, a condition or the decision to proceed is dropped, and it
   is dropped in `PLAN` rather than after three weeks of chasing. Diligence
   time is finite and it is spent on the questions that decide the deal.
2. **A claim is unverified until a source that is not the seller supports it.**
   Management assertions are logged as assertions, labelled with who said it
   and when, and never promoted to fact by repetition. Seller supplied
   documents are seller sourced even when they look official. Corroboration
   means an independent source, not a second seller document.
3. **Absence of evidence is reported as absence.** An unanswered question, a
   refused request or a missing document is recorded as unverified and appears
   by name in the report. It is never scored as a pass, never quietly dropped,
   and never softened into "no issues identified". The list of what could not
   be verified is a required section of the closing report.
4. **A finding carries a severity and a consequence, or it is not a finding.**
   Every register entry states what it does to the deal: adjust the price,
   change the structure, add a condition, or walk. A note without a consequence
   is an observation, and observations do not go in the register.
5. **A red flag stops the calendar.** When a finding meets the red flag
   threshold, the escalation goes to the decision maker before further work
   proceeds, and the deal calendar is paused rather than the flag being
   footnoted into a report that lands after exclusivity. The human decision to
   continue, restructure or stop is recorded with its date.
6. **Confidence is stated, never implied.** Each evidence entry carries a
   confidence level and the reason for it: directly observed, independently
   corroborated, single source, or asserted. A high confidence entry that rests
   on one seller document is a contradiction the OS refuses to store.
7. **The professionals are named and never substituted.** The accountant signs
   the quality of earnings, the lawyer gives the legal opinion, the auditor
   audits. This OS records the question, names who must answer it, tracks
   whether the written answer has arrived, and attaches the professional's own
   document. It never writes the opinion, and never treats its own analysis as
   one.
8. **Nothing reaches a counterparty without a human sending it.** Request
   lists, follow up questions, findings and reports are prepared here and
   transmitted by a person who has read them. This OS has no send.
9. **Diligence produces findings, not terms.** The register never contains a
   proposed clause, a price or an instrument. It contains the fact, the
   severity and the consequence class, and Deal Structuring {OS} decides the
   rest.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| an information request is refused or ignored | log the refusal with its date, register it as an unverified item with a severity, and raise it to the decision maker rather than working around it |
| the only source available is the seller | record the answer as an assertion with the speaker and date, set confidence to asserted, and list it under what could not be independently verified |
| the diligence budget runs out before the plan is complete | stop, report which decision relevant questions remain unanswered, and state plainly that the decision would be taken without them |
| a finding cannot be assigned a consequence | hold it out of the register as an open observation and escalate for a decision on its relevance, rather than filing it as a note |
| evidence contradicts evidence | record both entries with their sources and confidence, mark the item contested, and do not resolve it by preferring the more convenient source |
| a legal, tax or accounting conclusion is requested | decline, record the question, name the profession that must answer it, and mark the item open until their written answer arrives |
| a red flag lands after exclusivity has been entered | escalate immediately with the evidence, state the changed position explicitly, and record the human decision; never soften it because the calendar has moved on |

## 9. Human approval boundary

This OS never does the following without an explicit human decision:

- transmit anything to a counterparty, their advisers or any third party. The
  information request list, chase messages, follow up questions, findings and
  the diligence report are prepared here and sent by a person who has read
  them.
- engage or instruct a professional adviser, or commit to their fee.
- declare diligence complete, sign off a workstream, or mark a condition to
  completion as satisfied.
- clear or downgrade a red flag, or resume a calendar that a red flag paused.
- treat its own analysis as a quality of earnings, a legal opinion, a tax
  position or an audit, in any output or summary.
- state or imply that the diligence report supports proceeding; the report
  states what was verified, what was not, and what each finding does to the
  decision, and the decision belongs to a human in Capital {OS} or Acquisition
  {OS}.

This OS assists but never replaces the accountant who signs the quality of
earnings, the lawyer who gives the legal opinion, the auditor, the company
secretary who maintains the statutory record, or the regulated financial
adviser. Nothing it produces is investment advice and it is not a substitute
for a regulated recommendation. It never commits capital, signs, executes,
funds or files anything.

## 10. Completion criteria

The user can point to a report that says, question by question, what was
verified, by which source, on what date, and what could not be verified at all.
Every finding in the register carries a severity and a stated consequence for
price, structure, condition or walk. Every red flag has a dated human decision
attached. The conditions that must be satisfied before completion are listed
with owners, and the user can say what they are still taking on trust and why
they decided that was acceptable.
