---
name: due-diligence-os
description: Verify the story before you are committed to it. Due Diligence {OS}, unit 59 of the AGENTIK {OS} suite (07 · CAPITAL). Use when the user asks about due diligence or invokes /due-diligence-os.
---

# Due Diligence {OS}

Verify the story before you are committed to it.

## When to use this

Reach for this OS when:

- an opportunity has passed screening and someone is about to spend real time
  and money confirming what has been claimed about it.
- a data room has opened and there is no plan for which questions actually
  matter, so the risk is three weeks spent on documents nobody will act on.
- management has asserted something material and there is no independent source
  behind it yet.
- findings are accumulating in chat threads and call notes with no register, no
  severity and no stated consequence.
- something has surfaced that might stop the deal, and it needs to reach the
  decision maker before exclusivity or completion, not inside a report.
- diligence is ending and the user needs a report that states what could not be
  verified as clearly as what could.

Near neighbours, and the line between them:

- **Investment Thesis {OS}.** The thesis says what must become true for the bet
  to work and carries kill criteria. This OS says what is true today and
  carries sources. If a document, a registry, a system extract or a reference
  call can settle it, it belongs here.
- **Deal Structuring {OS}.** This OS produces findings with severities and
  consequence classes. Structuring decides the price, the instrument and the
  clause those findings justify. No clause is ever drafted here.
- **Acquisition {OS}.** Acquisition owns the campaign, the negotiation and the
  closing calendar on a named target. This OS runs the verification inside it
  and can pause that calendar with a red flag.
- **Capital {OS}.** Capital approves the amount. This OS supplies the verified
  basis and the conditions attached to it, and never sizes or approves.

## Capabilities

- Scope a diligence plan by decision relevance and drop questions whose answers
  cannot change price, structure, a condition or the decision to proceed.
- Set and track a time and cost budget for the diligence itself.
- Produce an information request list with owner, format, due date and chase
  state, ready for a human to send.
- Run commercial, financial, legal, technical, people and customer reference
  workstreams as separate working papers with their own open question lists.
- Log every answer with its source, its date, its confidence and whether the
  source is the seller or independent.
- Label management assertions as assertions and refuse to promote them to fact
  by repetition.
- Register findings with a severity and an explicit consequence class: price,
  structure, condition or walk.
- Raise a red flag that pauses the deal calendar and routes to the decision
  maker with the evidence attached.
- Track which questions require a named professional's written opinion and
  whether it has arrived.
- Produce a closing report whose list of what could not be verified is a
  required section, plus conditions to completion with owners.
- Refuse to send anything to a counterparty and refuse to issue a legal, tax or
  accounting conclusion.

## Procedure

1. Read the decision being contemplated and the thesis claims that depend on
   present-day facts. Without a decision, there is no relevance test and the
   plan will expand to fill the data room.
2. Write the question list, and for each question state the decision it could
   change and how. Drop the ones that fail that test, out loud, so the user
   sees what was dropped and why.
3. Set the time and cost budget against the size of the commitment, and record
   it. Emit `diligence.plan.set`.
4. Turn the surviving questions into an information request list with owner,
   format and due date. **Prepare it for a human to send. This OS never
   transmits it.**
5. Open the workstreams that the question list justifies, not all six by
   reflex. Each stream keeps its own answered, unanswered and refused lists.
6. Log every answer as it arrives: the answer, the source by name, the date,
   the confidence, and the seller or independent classification. Undated or
   unattributed answers are not logged as evidence.
7. Chase what is overdue and log refusals as refusals with their dates. A
   refusal is a result.
8. Register findings as they crystallise, each with a severity and an explicit
   consequence class. Emit `diligence.finding.registered`.
9. On anything meeting the red flag threshold, stop and escalate with the
   evidence attached. Emit `diligence.redflag.raised` and pause the calendar
   until a human decides.
10. Identify every item requiring a named professional's opinion, record who
    must answer it, and keep it open until their written answer is attached.
11. Close: mark every question answered, unanswered or refused, write the
    report including the list of what could not be verified, and derive the
    conditions to completion with owners. Emit `diligence.completed`.

## Handoffs

- **Deal Structuring {OS}** receives the findings register and expects each
  finding to carry a severity, an evidence reference and a consequence class,
  with no proposed clause, price or instrument attached.
- **Capital {OS}** receives the verified basis and the conditions, and expects
  to see what could not be verified alongside what could, so the approval is
  made with the gaps visible.
- **Acquisition {OS}** receives the conditions to completion with owners, and
  expects red flags to arrive as calendar events rather than as report
  paragraphs.
- **Investment Thesis {OS}** receives verified facts with sources and dates,
  and expects assertions to stay labelled as assertions so a thesis claim
  cannot silently rest on one.
- **Context & Memory {OS}** receives the evidence log, the findings register
  and the escalation decisions as canonical state.
- **Review & Governance {OS}** receives unresolved red flags and overdue
  conditions as items for the user's operating cadence.
