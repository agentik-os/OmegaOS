# Workflow: the recurring meeting audit

Produces a keep, shrink, merge or kill verdict on a recurring meeting, backed by
what it actually produced and what it actually cost.

## Trigger

The recurring meeting reaches its review date, or three consecutive occurrences
produce no decision.

## Inputs

- The last several occurrences: agendas, decision records, action items.
- The attendee list and the duration, occurrence by occurrence.
- The closure rate of the actions the meeting created.
- The meeting's original stated purpose.

## Steps

1. **Count decisions per occurrence.** Not discussions, not updates. Decisions
   with a named decider.
2. **Count actions created and actions closed.** A meeting that creates actions
   nobody closes is a generator of guilt, not of work.
3. **Compute the cost.** Attendees multiplied by duration multiplied by
   occurrences, in person-hours, over the review period.
4. **Compare against the original purpose.** Meetings drift; the drift is
   usually from decision-making toward status reporting.
5. **Test the async alternative for the whole series.** If most items are status,
   the series can become a written report with a comment window.
6. **Test the attendee list.** Anyone who has neither spoken nor owned an action
   across the review period does not need a seat, only the record.
7. **Choose the verdict.**
   - Keep: it decides things, and the decisions are worth the person-hours.
   - Shrink: fewer people, or less time, or lower frequency.
   - Merge: another series covers the same decisions with the same people.
   - Kill: it produces no decisions, or the cost exceeds what it produces.
8. **Record the verdict with the evidence** and send it to Review & Governance
   {OS}. Killing a meeting other people rely on is a human decision, so it is
   proposed here and approved there.
9. **If kept, set the next review date.** No recurring meeting is exempt.

## Completion test

- Decisions per occurrence, actions created and actions closed are all counted.
- The cost over the review period is stated in person-hours.
- A verdict of keep, shrink, merge or kill is recorded with its evidence.
- If kept, a next review date exists.
- If killed or merged, everyone who relied on it has been told what replaces it.

## Failure paths

| Situation | Response |
|---|---|
| records are missing for most occurrences | that is itself the finding; recommend shrink and require records for one cycle before judging again |
| the meeting is social and everyone values it | record it as relational, keep it, and say plainly that it is not judged on decisions |
| the owner refuses to kill a meeting the evidence condemns | keep the verdict on the record, propose shrink as the fallback, and re-audit sooner |
| the actions are closed elsewhere and invisible here | connect the action list to Execution {OS} and Team & Delegation {OS}, then re-run the count |
