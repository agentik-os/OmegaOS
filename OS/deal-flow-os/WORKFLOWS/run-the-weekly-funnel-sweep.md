# Run the weekly funnel sweep

Produces an honest funnel: every live item with a next action, an owner and a
date, every stalled item named as stalled, and every channel with a dated next
touch.

## Trigger

A fixed weekly slot. It is a calendar trigger, not a mood trigger, because the
failure this workflow prevents is exactly the one that never announces itself:
a pipeline that looks full and is not.

## Inputs

- The opportunity register, all live and parked items.
- The stage ageing thresholds from the screen.
- The source ledger.
- Any handoff records from the week, so items now owned by another OS are
  reported but not driven.

## Steps

1. Pull every live item and its last state change date.
2. Age each item against its stage threshold. Items past threshold are marked
   stalled. Nothing is quietly given more time.
3. For each stalled item, decide one of three things and record it: revive with
   a new dated next action, pass, or park with a stated reason and a review date.
4. For each live item, confirm the next action, the owner and the date are all
   present. Any item missing one of the three is treated as stalled until it has
   all three.
5. Remove from the live count anything that has been handed off. It is reported
   under the receiving OS, not counted twice.
6. Check the contacted rule: anything never actually contacted is a lead, not an
   opportunity, and moves out of the pipeline count.
7. Sweep the channels. Every source gets a dated next touch or is marked
   dormant with the date it went quiet.
8. Draft any pass messages the sweep produced. **Human approval gate:** every
   pass is reviewed and sent by a human.
9. Draft any outreach the channel sweep produced. **Human approval gate:** no
   outbound message leaves without human approval, and it goes under a named
   human's authority.
10. Publish the funnel state: live count, stalled count, parked count, and the
    items needing a decision this week.
11. Emit `dealflow.pipeline.reported`.

## Completion test

Every item in the register is in exactly one of four states: live with a next
action, an owner and a date; stalled with a decision recorded this week; parked
with a reason and a review date; or closed. The live count contains no item that
has never been contacted and no item owned by another OS.

## Failure modes

| Failure | What happens |
|---|---|
| the sweep is skipped for a period | the next sweep reports the gap explicitly and ages every item from its real last change date |
| an owner is unavailable | the item is stalled, not reassigned silently, and the reassignment is a recorded decision |
| the stalled list is larger than the live list | that is reported as the headline, not buried, since it means the screen is qualifying too much |
| a channel has been dormant for two consecutive sweeps | it is proposed for retirement with its lifetime qualified count attached |
| a parked item's review date passes unnoticed | the sweep surfaces overdue reviews before it reports anything else |
