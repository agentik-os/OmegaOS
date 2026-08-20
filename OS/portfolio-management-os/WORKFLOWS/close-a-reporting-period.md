# Close a reporting period

Produces the closed period: every position reported or escalated, every mark
carrying a method and evidence, the portfolio triaged, and an approved report in
which realised and unrealised are separated.

## Trigger

The reporting period ends. Also runs early when a marking event lands mid period
that is large enough to change a class, for example a priced round, a covenant
breach or a write down at another holder.

## Inputs

- The reporting calendar and the due dates for this period.
- Raw submissions from each position: management accounts, KPI packs, cap table
  updates.
- Marking evidence: priced rounds, secondary transactions, comparable sets,
  impairment triggers, each with its own source.
- Thesis checkpoint results due this period, from Investment Thesis {OS}.
- Board packs and escalations from Board {OS}, where a board seat exists.
- The support ledger and the capacity budget for the period.

## Steps

1. Build the chase list from the calendar: due, received, outstanding. Chase the
   named person, and record the chase itself, because the chase history is the
   evidence behind any later escalation.
2. Receive submissions and retain every raw file unchanged. Normalise into the
   comparable series as a derivation, so the normalisation can be redone when
   the mapping is found to be wrong.
3. Mark each position missing this period as unsupported: the prior mark stands,
   labelled, and it is never quietly moved to fill the gap.
4. Escalate every position at two consecutive missed periods. Move it to watch
   at minimum, notify the owner, and state in the report that the mark is
   unsupported by current reporting. **Human approval gate:** any escalation
   that leaves the owner's desk, to the company's board or to a third party, is
   sent by the owner.
5. Set marks. For each position choose the method, attach the evidence, set the
   date, and name who set it. Reject any request for a value with no method.
   Where evidence conflicts, present both sources and state the method chosen
   and why, rather than averaging them into one number.
6. **Human approval gate:** the owner approves each mark before it is written to
   the book. Emit `portfolio.mark.updated` on approval, never before.
7. Assess impairment wherever a trigger occurred in this period, whatever it
   does to the aggregate. On approval, emit `portfolio.position.impaired`.
8. Run triage: classify every position compounding, watch or impaired against
   the evidence in front of you, and record the date of every class change with
   the evidence that caused it.
9. Reconcile the support ledger: capacity stated, capacity spent per position,
   capacity remaining. Any help delivered off the ledger is written in now, with
   the position that consumed it named.
10. Read the thesis checkpoints due this period. Where a checkpoint result
    contradicts the class you just assigned, resolve the contradiction in
    writing rather than in favour of the more comfortable one.
11. Build the report. Realised and unrealised are separated in every total,
    chart and headline. Every mark shows its method and date. The face of the
    report states that marks are management estimates and not audited figures.
12. **Human approval gate:** the owner approves the send, per audience.
    Stakeholder reporting may be a regulated communication depending on the
    jurisdiction and the recipient class, so this is an explicit decision every
    period and never a standing permission. Emit
    `portfolio.report.published` after the send is approved.

## Completion test

Every position in the book resolves to one of exactly three states for the
period: reported, chased and unsupported, or escalated. Every mark in the book
names a method, an evidence item and a date. Every class in the triage table
carries the evidence and the date it was set. The support ledger reconciles to
the stated capacity. Opening the report at any total shows realised and
unrealised as separate figures, and the report has an approval record against
the send.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| quiet position carried at cost | second missed period and the mark unchanged with no label | escalate, move to watch, label the mark unsupported in the report |
| mark moved to save the period | value changed with no new evidence | reject the change, show prior mark, new mark and triggering evidence side by side, keep the prior mark if there is none |
| conflicting evidence averaged | a priced round and a comparable set blended into one number | present both, choose one method, state why |
| normalisation overwrites the source | raw submission discarded after mapping | retain the raw file, treat the series as a recomputable derivation |
| support delivered off ledger | hours and intros nobody recorded | write them in at close, name the position that consumed them, report the capacity overrun |
| checkpoint contradicted by class | thesis checkpoint failed while the position sits in compounding | resolve in writing, and state which evidence overrode which |
| report sent on standing permission | last period's approval reused | require an explicit approval per audience per period |
