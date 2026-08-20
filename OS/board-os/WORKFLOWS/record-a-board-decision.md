# Record a board decision

Produces a decision record that will still be defensible in two years:
attendance, the decision, dissent, declared conflicts, and the actions it
created with owners and dates.

## Trigger

The board reaches a decision, including a decision to defer, to decline, or to
delegate. Every decision is recorded, not only the ones that feel historic.

## Inputs

- The agenda item and the paper that asked for the decision.
- The attendance list, including who joined late or left early.
- The interests and conflicts register.
- The delegated authority matrix, to confirm this decision belongs to the board.
- Any reserved matter that this decision triggers, from Deal Structuring {OS}.

## Steps

1. Confirm the decision class sits with the board under the authority matrix. If
   it does not, record the gap and escalate before the vote, not after it.
2. Confirm the item's conflicts before the discussion opens. Record the
   declaration and whether the director participated or withdrew.
3. Record who was present for this specific item, since attendance changes
   through a meeting and a decision is only valid for those in the room.
4. State the decision in one sentence that a stranger could act on, including
   what was explicitly not decided.
5. Record the vote or the consensus, and record dissent by name where a director
   asks for it. A unanimous record that hides a dissent is a false record.
6. Record the reason: the two or three facts the board relied on. In two years
   the reason is the only defence available.
7. Create the actions the decision requires, each with a named owner and a date.
   An action with neither is not recorded as an action.
8. Check whether the decision triggers a reserved matter, a shareholder consent
   or a notification. If so, route it to the company secretary and the lawyer
   rather than resolving it in the meeting.
9. Draft the resolution wording where a formal resolution is needed, marked as a
   draft.
   **Human approval gate:** a resolution is adopted by humans, never recorded as
   adopted by this OS.
10. Fold the decision into the draft minutes, and route the minutes for approval.
    **Human approval gate:** minutes are approved by the board and the approved
    version goes to the company secretary, who holds the statutory record.
11. Communicate outward only what the board has agreed may be communicated.
    **Human approval gate:** any communication of a board decision outside the
    board is approved first.
12. Emit `board.resolution.recorded` once the resolution is approved, and
    `board.minutes.approved` when the minutes are.

## Completion test

For this decision, the record shows: the item, the authority class, who was
present for it, every declared conflict and whether the director withdrew, the
decision in one actionable sentence, any dissent by name where requested, the
facts relied on, and every action with an owner and a date. The minutes carrying
it are marked draft until humans approve them, and the approved version is with
the company secretary.

## Failure modes

| Failure | What happens |
|---|---|
| the decision class is not in the authority matrix | it is escalated and the gap recorded before the vote, and the matrix amendment is proposed |
| a conflict surfaces after the vote | it is recorded and reported immediately to the chair and the company secretary, and is never cured by editing the minutes |
| a director asks for dissent to be removed | the dissent stays, and the request itself is recorded |
| an action has no owner | it is not recorded as an action, and the gap is raised in the room |
| the decision triggers a consent nobody noticed | it is routed to the lawyer and the company secretary, and completion of the action waits for their answer |
| minutes are wanted as final immediately | they stay marked draft until approved, whatever the pressure, since the statutory record is not this OS's to create |
