# Run the exclusivity period

Produces a dated close plan with an owner and a deliverable every day, and
drives it to completion or to a clean abandon before the clock runs out.

## Trigger

An offer has been accepted and exclusivity is about to begin. The workflow runs
from the day before the clock starts, not from the first slipping deadline.

## Inputs

- The signed letter of intent and its dates, including the exclusivity length.
- Evidenced financing: a lender term sheet, committed investor funds, or cash.
- The diligence plan and its time budget from Due Diligence {OS}.
- The agreed instrument and terms from Deal Structuring {OS}.
- The approved commitment amount from Capital {OS}.
- The kill criteria from Investment Thesis {OS}.
- The names of the people who will actually do the work: lawyer, accountant,
  lender contact, the operator.

## Steps

1. Verify financing is evidenced, not indicated. If it is not, do not start the
   clock. Name the missing evidence and stop.
   **Human approval gate:** entering exclusivity is a human decision, and it
   commits the seller's time and the buyer's money.
2. Build the close plan backwards from completion: legal drafting, diligence
   workstreams, financing conditions, third party consents, completion
   accounts, employee and customer communication.
3. Assign an owner and a deliverable to every workstream, and put a date on
   every deliverable. A workstream without a named human is not a workstream.
4. Publish the plan to everyone on it, including the external advisers, so
   nobody's date is a surprise.
5. Run a short daily check: what was due today, what moved, what is at risk
   tomorrow.
6. On any slip, escalate the same day with the options and their cost in
   exclusivity days. Do not absorb a slip to protect the appearance of the plan.
7. On a red flag from Due Diligence {OS}, pause the calendar. The flag is
   resolved, priced into the terms through Deal Structuring {OS}, or the deal is
   abandoned. It is never carried into completion as a footnote.
   **Human approval gate:** continuing past a red flag is an explicit human
   decision, recorded with its reasoning.
8. Route every changed term back to Deal Structuring {OS} and re-test the offer
   hypothesis. A term traded to protect a date is the most expensive kind.
9. Two weeks before expiry, run the honest test: is completion achievable on the
   current evidence. If not, negotiate an extension deliberately or prepare the
   abandon.
10. Drive the closing checklist condition by condition until every condition has
    a named sign off.
    **Human approval gate:** completion itself is signed by humans, with the
    lawyer's confirmation that conditions are satisfied.
11. On completion, emit `acquisition.closed` and hand the day one transition
    pack to Portfolio Management {OS} and the governance structure to
    Board {OS}.
12. On abandon, tell the counterparty, record what fired and what it cost, emit
    `acquisition.abandoned`, and feed the lesson back into the buy box.

## Completion test

Either completion has occurred with every condition signed off by a named
person and the transition pack handed over, or the deal has been abandoned with
the counterparty informed and the record showing what fired, on what date, and
what the attempt cost. There is no third state at the end of the clock.

## Failure modes

| Failure | What happens |
|---|---|
| financing was indicated but not evidenced | the clock does not start, and the missing evidence is named |
| a workstream has no named owner | it is treated as not started, and it is reported as the plan's weakest point |
| a deliverable slips quietly | the daily check catches it, and same day escalation is mandatory |
| a red flag is raised late in the period | the calendar pauses, and continuing requires a recorded human decision |
| the seller adds a material term near completion | the offer hypothesis is re-run and the term is priced, never absorbed for the date |
| exclusivity is about to expire with conditions open | the extension or the abandon is decided deliberately, two weeks out, not on the last day |
| an adviser is unavailable at a critical date | the dependency is escalated immediately, since an unavailable lawyer is a schedule risk and not an administrative detail |
