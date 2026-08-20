# Onboard a funded position

Produces the position record: what reporting is owed, in what form, by whom and
by when, plus the data rights, the contacts and the baseline the position will
be measured against.

## Trigger

A commitment has been approved by Capital {OS} and the money has actually moved.
Approval alone does not start this workflow: an unfunded approval is still
Capital's business.

## Inputs

- The allocation decision record from Capital {OS}, events
  `capital.allocation.approved` and `capital.reserve.committed`, giving the
  amount and the reserve held.
- The agreed terms from Deal Structuring {OS}, event `structure.terms.agreed`,
  which determine the information rights that can actually be enforced.
- The thesis and kill criteria from Investment Thesis {OS}.
- The named contacts at the position: who signs, who reports, who to call when
  reporting stops.
- The metrics as at funding, from diligence or from the position itself.

## Steps

1. Confirm funding, not approval. Record the funding date and the amount that
   actually settled. If they differ from the approved amount, raise the
   difference with Capital {OS} before opening the record.
2. Mirror the commitment amount and the reserve held from Capital {OS} into the
   position record as read only. This OS never edits either.
3. Read the terms and extract the information rights literally: what the owner
   is entitled to receive, in what form, at what frequency, and what happens
   when it does not arrive. Rights that are not in the terms are requests, and
   are recorded as requests.
4. Build the reporting calendar from those rights: each period with a due date.
5. Name the person who personally owes each report. A department does not owe a
   report. If nobody can be named, that is the first finding of this workflow
   and it is raised now, not at the first missed period.
6. Record the escalation path: who is contacted at one missed period, and who at
   two.
7. Capture the baseline: the metrics as at funding that the thesis will be
   tested against. A baseline captured three months later is not a baseline.
8. Reference the thesis and its kill criteria. Do not restate them here: cite
   the Investment Thesis {OS} record so there is one copy that can change.
9. Set the opening mark at cost, with method `cost` and the funding evidence,
   dated at funding. **Human approval gate:** even the opening mark is written
   to the book only on the owner's approval.
10. Record the support expectations, if any were promised during the deal, and
    charge them against the support capacity budget. A promise made in a deal
    room is a capacity commitment.
11. Emit `portfolio.position.opened` and confirm the first reporting due date
    with the named contact in writing. **Human approval gate:** the outbound
    message to the position is sent by the owner, not by this OS.

## Completion test

Open the record and answer four questions without looking anywhere else: what
are we owed, in what form, by when, and by whom by name. Then check the calendar
has a first due date in the future, the baseline has values as at the funding
date, and the opening mark carries method, evidence and date. If the reporting
obligation cannot be quoted from the terms, the record shows it as a request
rather than a right.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| onboarded on approval, not funding | position record exists with no settled amount | hold the record open, record the expected funding date, do not emit `portfolio.position.opened` |
| no named reporter | "the company will send updates" | raise it now as the first finding, request a name before the first period |
| rights assumed rather than read | reporting expectations exceed anything the terms grant | reclassify the excess as requests, keep them visible, do not enforce them as rights |
| baseline captured late | first metrics recorded a quarter after funding | record what exists, label the baseline reconstructed, and note that thesis checkpoints inherit that weakness |
| thesis restated locally | kill criteria copied into the position record | replace with a citation of the Investment Thesis {OS} record |
| deal-room promises off ledger | support committed during negotiation and never logged | charge them to the capacity budget at onboarding |
