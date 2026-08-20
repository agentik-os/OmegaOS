# Channel verdict

Decide whether a channel is scaled, held or killed, on cost per retained
cohort rather than on the volume it produces.

## Trigger

A channel's retention window closes for a cohort, a spend ceiling is reached,
or `/growth-review` reaches the channel on its cadence.

## Steps

1. **The OS assembles the cohort** for the channel: who was acquired, when, at
   what acquisition cost, from Growth's own spend records and Sales {OS}.
2. **The OS pulls retained revenue for that cohort** from Revenue {OS}, and
   adoption and health signals from Delivery & Customer Success {OS}. If the
   retention window has not closed, the verdict is provisional and says so.
3. **The OS computes cost per retained cohort**: total acquisition cost against
   revenue that survived the window, not revenue booked at signup.
4. **The OS compares the channel against the loop map**: does this channel feed
   a loop, or does it only fill a funnel that must be refilled with money every
   cycle.
5. **The OS checks spend dependence**: model the channel at half the current
   spend and at zero. A channel whose output collapses proportionally with spend
   is a purchase, not a channel, and is labelled as such.
6. **The OS issues a recommendation**: scale, hold, or kill, each with the
   figures behind it and the confidence given the cohort size.
7. **Scale requires human approval** on the spend, with an explicit ceiling.
   The proposal goes to the owning OS to execute.
8. **Kill produces a kill record** (`/kill`) stating the channel, the economics,
   the date, and the condition under which revisiting it would be justified.
9. **Hold produces a next review date and the specific number being watched.**

## Completion test

The channel has a recommendation of scale, hold or kill. The recommendation
carries: the cohort and its size, the acquisition cost, the retained revenue
with its source in Revenue {OS}, the cost per retained cohort, a statement of
whether the channel feeds a loop or a funnel, and the modelled behaviour at
half and at zero spend.

A recommendation to scale that is not accompanied by an approved spend ceiling
fails this test. A kill without a revisit condition also fails it, because an
unqualified kill is re-argued from scratch within a year.

## Failure and abort

- **Retention window still open:** issue the verdict as provisional, name the
  date it becomes final, and do not approve new spend against it.
- **Cohort too small to distinguish the channel from noise:** report that, hold
  by default, and state the cohort size that would settle it.
- **Acquisition cost incomplete:** name the missing cost component and treat the
  computed figure as a floor, never as the cost.
- **Revenue and Delivery signals disagree** about whether the cohort is
  retained: report both, do not average them, and escalate the definition
  conflict to KPI & Analytics {OS}.
- **Human approval refused on a scale proposal:** record the refusal, keep the
  channel at its current ceiling, and set the next review date.
