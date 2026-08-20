# Workflow: Named opportunity

**Produces:** one opportunity record naming who can act, what changed, why now,
what window closes it, and the date it expires if nothing else fires.

## Trigger

A movement has been confirmed with a direction and a rate, and a specific actor
could plausibly do something about it before it stops being available.

## Steps

1. **Restate the confirmed movement** in one sentence, with its direction, its
   rate and its measurement window. If the movement is still a candidate, stop:
   an opportunity named on unconfirmed movement is a guess with a deadline
   attached.
2. **Name the actor.** A specific party who can actually reach this: you, this
   team, this company, this product line. Test reachability honestly against
   what they have (distribution, capability, capital, permission). An
   opportunity nobody in the room can take is a fact about the world, and it is
   recorded as a fact rather than dressed up as an opportunity.
3. **Write what changed.** The behaviour, from what to what, over which window,
   with the supporting observations attached. This is the movement, restated as
   the thing the actor can exploit.
4. **Write why now.** The hardest of the four and the one that exposes fake
   opportunities: what makes this moment different from three years ago, and
   different from three years from now. Acceptable answers name an enabling
   change (a cost floor crossed, a platform opened, a regulation landed, a
   default shifted, a supply appeared, an incumbent withdrew). "Interest is
   high" is not a why now.
5. **Write the closing condition.** What specifically would shut this window: a
   dominant entrant, a platform closing access, a price floor, the behaviour
   becoming the default and the advantage evaporating, a regulation, a supply
   drying up. Name the observable that would tell you it fired, and add it to
   the watch so the sweep can see it.
6. **Set the expiry date.** The date the opportunity is re-examined and, absent
   fresh evidence, retired, even if no closing condition fired. An opportunity
   without an expiry date is not named here. This is the single rule that stops
   a plan carrying dead opportunities for years.
7. **Assign an owner and a review date.** The owner is the person who will act
   or decide not to. The review date is at or before the expiry, never after.
8. **State what this opportunity is not.** It is not a concept, not a product
   definition, not a market size, and not a funded bet. Name the units that own
   those next steps so the handoff is explicit rather than assumed.
9. **Record and emit `opportunity.named`** to Brainstorm {OS} (which will
   generate concepts against it), Strategy & Portfolio {OS} (which will weigh it
   against other bets) and Market Research {OS} (which will size the market
   behind it).
10. **Route approval before any external publication.** An opportunity brief
    leaving the organisation goes through the human approval boundary first.

## Completion test

- All four parts are present: who, what changed, why now, what window.
- The actor is specific, and their ability to reach it is stated rather than
  assumed.
- "What changed" cites the confirmed movement record, with its rate and window.
- "Why now" names an enabling change, not a level of enthusiasm.
- A closing condition is written with an observable that the watch can detect.
- An expiry date exists and is a date, not a season.
- An owner and a review date on or before the expiry are recorded.
- The record explicitly disclaims concept, product definition, market size and
  funding, and names the OS that owns each.
- No brief left the organisation without an approval record.
