# Prepare and issue a board pack

Produces a board pack issued at the stated notice period, in which every paper
names the decision it asks for.

## Trigger

The notice deadline for the next meeting, counted backwards from the meeting
date. The trigger is the deadline, not the meeting, which is the whole point:
a pack triggered by the meeting is always late.

## Inputs

- The board calendar, and this meeting's stated purpose.
- The notice period from the terms of reference.
- The open action register from the previous meeting.
- Management reporting and the numbers, with their definitions, from KPI &
  Analytics {OS}.
- The risk register and its owners.
- Any reserved matter arising, from the rights recorded by Deal Structuring
  {OS}.
- Declared interests and conflicts, so related party items can be flagged in
  advance.

## Steps

1. Compute the paper deadline: the notice period plus the assembly time,
   backwards from the meeting date. Publish it to every paper author.
2. Request each paper against that deadline, naming the author and the decision
   the paper must ask for.
3. As papers arrive, test each one: does it state a decision, does it state the
   options, does it state what happens if the board does nothing. A paper that
   asks for no decision moves to the written section.
4. Check the numbers against the previous pack. Any contradiction becomes an
   agenda item, and is never reconciled silently to make the pack look tidy.
5. Flag every item where a director has a declared interest, and note who must
   withdraw.
6. Place the open action register at the front, before any new business.
7. At the deadline, assemble what exists. If a paper is missing, do not wait:
   propose deferring its item and record the deferral with its reason.
8. **Human approval gate:** the chair approves issue. The pack is a formal
   communication to directors and does not go out on the OS's authority.
9. Issue the pack, and record the issue timestamp against the notice deadline.
   The timestamp is the evidence that the notice rule held.
10. Confirm receipt, and record any director who received it late, since late
    receipt is a governance fact and not an administrative one.
11. Emit `board.pack.published`.

## Completion test

The pack was issued at or before the notice deadline, the issue timestamp is
recorded, every paper in the presented section names the decision it asks for,
the open action register is at the front, related party items are flagged, and
any deferred item carries a written reason.

## Failure modes

| Failure | What happens |
|---|---|
| a paper is late | its item is proposed for deferral, and the pack goes out on time without it |
| an author says the paper needs one more day | the deadline holds, since the notice period protects the directors' reading time, not the author's |
| a paper asks for no decision | it moves to the written section and takes no meeting time |
| the numbers contradict the last pack | the contradiction becomes an agenda item with its own decision |
| a conflict is spotted at assembly | it is flagged in the pack and the withdrawal is planned before the meeting, not during it |
| the pack cannot be issued on time at all | the meeting is proposed for deferral, and the failure is recorded for the effectiveness review |
