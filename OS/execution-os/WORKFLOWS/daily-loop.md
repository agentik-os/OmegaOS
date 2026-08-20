# Workflow: the daily loop

Produces a day that was deliberately opened, worked in protected blocks, closed
with proof, and connected to tomorrow.

## Trigger

The working day starts. The closing half triggers when the day ends, and also
on the next `BOOT` if a previous day was left open.

## Inputs

- Stated capacity (GREEN, AMBER, RED) and usable minutes.
- Open commitments carried from previous days.
- Incoming obligations: meeting actions owned by you, client promises, project
  next actions.
- The operator profile from Context & Memory {OS}.

## Steps

1. **Read the profile.** Local profile first, shipped default second. Do not
   ask for anything the profile already answers.
2. **Ask capacity.** Capacity and usable minutes. One question, not a survey.
   If the user does not answer, assume AMBER and say that you assumed it.
3. **Ask the must-win.** One outcome. If two are offered, ask which one makes
   the day count if the other does not happen.
4. **Pull open commitments.** List them with their current next action, oldest
   first. Anything deferred three or more times is flagged now, not later.
5. **Clarify the inbox.** Every captured item becomes a commitment, a project
   handoff, a delegation, reference, or is dropped. Nothing stays unclassified.
6. **Fit the day to the budget.** Sum the estimated minutes. If the sum exceeds
   usable minutes, name the overflow and ask what leaves the day. Do not
   silently accept an impossible day.
7. **Check the defined next.** Every committed item has one physical, startable
   action. Rewrite the ones that do not, or refuse the commitment.
8. **Render the daily command card.** Capacity, must-win, commitments in order,
   planned blocks. Short enough to read on a phone.
9. **Work in blocks.** One commitment per block, 25, 50 or 90 minutes. Capture
   anything that arrives mid-block instead of switching to it.
10. **Prove completions.** For each finished commitment, record the evidence and
    the acceptance test it satisfies. No evidence means the commitment stays
    open and is recorded as `TOUCHED`.
11. **Recover failures.** Anything not finished is classified: blocked,
    deferred, cancelled or delegated, each with a physical next action.
12. **Halt.** Day classification, energy, focus, friction, the day's proof, and
    tomorrow's first physical action.

## Completion test

- The ledger holds a boot record and a halt record for the same date.
- Committed minutes did not exceed usable minutes, or the overflow was
  explicitly removed and recorded.
- Every completed commitment has evidence attached and an acceptance test.
- Every unfinished commitment has a classification and one physical next action.
- Tomorrow's first physical action is written and is startable without thinking.

## Failure paths

| Situation | Response |
|---|---|
| the previous day was never halted | close it as `ABANDONED` with the reason unrecorded, say so, then boot today |
| the user refuses to name a must-win | pick the commitment with the nearest external deadline and state the choice |
| capacity is RED | cut to one commitment plus the must-win, and shrink the must-win rather than dropping the day |
| nothing shipped | classify honestly (`TOUCHED` or `ABANDONED`), record the friction, and carry the friction into the weekly reset |
