---
name: portfolio-management-os
description: Run the portfolio after the deal: reporting, support, reallocation. Portfolio Management {OS}, unit 62 of the AGENTIK {OS} suite (07 · CAPITAL). Use when the user asks about portfolio management or invokes /portfolio-management-os.
---

# Portfolio Management {OS}

Run what is already owned: reporting, marks, support, triage and the follow-on
recommendation.

## When to use this

Reach for Portfolio Management {OS} once money has actually moved:

- A commitment has funded and nobody has written down what reporting you are
  owed, by whom, and when.
- Quarter end has arrived and you are chasing five founders for numbers in five
  different formats.
- You need to state what a position is worth and you want the method and the
  evidence attached rather than a number you would struggle to defend.
- A position has gone quiet for two periods and you are still carrying it at
  cost.
- You are giving away hours, intros and reputation across the portfolio with no
  idea which positions are consuming them.
- A position is raising and you need a recommendation, with the thesis
  checkpoint attached, to hand to whoever decides the amount.
- You owe stakeholders a report and it must separate cash received from marks.

Near neighbours, and the one line that separates them:

- **Capital {OS}:** it decides how much goes where before or at commitment;
  this OS runs what is already owned and recommends without ever setting an
  amount.
- **Board {OS}:** it governs inside the company (agendas, resolutions, minutes,
  directors' duties); this OS reports to the owner about the position from
  outside.
- **Investment Thesis {OS}:** it owns the falsifiable claims and runs the
  checkpoints; this OS cites the checkpoint result, it does not restate the
  thesis.
- **Exit & Liquidity {OS}:** it runs the sale; this OS marks a position exit
  ready and hands over.
- **Deal Structuring {OS}:** it wrote the information rights; this OS enforces
  them and never renegotiates them.

## Capabilities

- Open a position record at funding with reporting expectations, data rights,
  named contacts and a baseline.
- Build the reporting calendar per position from the rights actually agreed in
  the terms.
- Chase, receive and normalise periodic reporting into one comparable series
  across a portfolio that reports in incompatible formats.
- Set a valuation mark with a named method, its evidence and its date, and
  refuse a bare number.
- Detect and escalate non-reporting at the second consecutive missed period,
  and label the affected mark unsupported.
- Log support requests against a stated capacity budget so help is allocated,
  not absorbed by whoever asks loudest.
- Triage the portfolio into compounding, watch and impaired, each with the
  evidence that produced the class.
- Assess impairment on a triggering event in the period the trigger occurs.
- Produce a follow-on or stand down recommendation carrying the thesis
  checkpoint result, with no amount attached.
- Publish the portfolio report with realised and unrealised separated in every
  view, gated on human approval before any send.
- Close a position at exit or write off with the final realised outcome set
  against the last mark before it.

## Procedure

1. On `capital.allocation.approved` and funding, open the position record.
   Import the commitment amount and the reserve from Capital {OS} as read only.
2. Read the agreed terms from Deal Structuring {OS} and write down exactly what
   information the owner is entitled to, in what form and by when. Name the
   person who owes it.
3. Capture the baseline: the metrics as at funding, and the thesis reference
   from Investment Thesis {OS}. Emit `portfolio.position.opened`.
4. Each period, run `COLLECT`: chase against the calendar, receive submissions,
   retain the raw file, and normalise into the comparable series.
5. Run `MARK` for every position: choose the method, attach the evidence, set
   the date. A position with no fresh evidence keeps its prior mark, labelled
   unsupported for the period, and is never quietly moved.
6. Escalate any position at two consecutive missed periods. Move it to watch at
   minimum and state the unsupported mark in the report.
7. Run `TRIAGE`: classify every position compounding, watch or impaired against
   the evidence, and record the date of any class change.
8. Assess impairment wherever a trigger occurred in the period. Record it when
   the evidence exists, not when it is comfortable, and emit
   `portfolio.position.impaired` after human approval.
9. Run `SUPPORT` against the stated capacity budget. Every request gets an
   owner, a capacity cost and a recorded outcome. Nothing is delivered off the
   ledger.
10. When a position raises or a stand down is due, request the checkpoint result
    from Investment Thesis {OS}, then produce the recommendation with that
    result attached and no amount. Emit `portfolio.followon.recommended` and
    hand it to Capital {OS}.
11. Run `REPORT`: build the owner or stakeholder report with realised and
    unrealised separated everywhere. Obtain explicit human approval for the
    send, then emit `portfolio.report.published`.
12. At exit or write off, close the record: realised outcome, last mark before
    it, and the gap between them, so the marking method itself is measured.

## Handoffs

- **Capital {OS}** receives the follow-on or stand down recommendation and
  expects the thesis checkpoint result, the current mark with its method, the
  reserve already held, and no amount. Capital sets the amount.
- **Investment Thesis {OS}** receives evidence that a checkpoint is due or that
  the thesis is drifting, and expects the reporting series that shows it.
- **Board {OS}** receives a matter that belongs inside company governance and
  expects it framed as an agenda item or an escalation, not as an owner report
  written more forcefully.
- **Exit & Liquidity {OS}** receives a position marked exit ready and expects
  the current mark, the method behind it, and the information rights that apply
  to a sale process.
- **The owner and stakeholders** receive the portfolio report and expect
  realised separated from unrealised, and every mark accompanied by its method
  and date. The send happens on their explicit approval.
