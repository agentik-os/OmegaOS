# Goal & Life Strategy {OS}: Annual life strategy

**Produces:** the life strategy for the year: the declared life domains, the
horizon map across `now`, `this year`, `three to five years` and `direction`,
the active goal set with a cost and a retirement condition per goal, and the
year's not-doing list.

**Trigger:** the annual reset (a birthday, a year boundary, a chosen anchor
date), or a life change large enough to invalidate the current goal set: a
move, a separation, a birth, a job ending, a diagnosis, a large financial
change. Also on the phrase "what should I actually be aiming at".

**Runs in:** `STRATEGY`, calling `GOAL_SET`, `HORIZON_MAP`, `TRADEOFF` and
`RETIRE` as sub-passes.

**Takes:** last year's goal set, horizon map, allocation ledger and retirement
records (Context & Memory {OS}); the value set and priority order
(Alignment {OS}); the identity model and belief set (Mindset {OS}); the
capacity ceiling and standing vetoes (Health & Energy {OS}); decision records
from the year (Decision {OS}); the user's fixed constraints, the ones they will
not move.

## Steps

1. Close the year that ended before opening the next one. Walk last year's goal
   set item by item and mark each reached, released, superseded, failed or
   carried. Run `/goal-retire` on every one that is not carried, each with its
   reason and its capacity reassignment.
2. Print what the closed year actually cost: total hours and money per domain,
   from the four quarterly ledgers. This is the only honest input to next
   year's capacity estimate.
3. Declare the life domains for the coming year. Use the previous list unless
   the user changes it. A domain that receives nothing is kept in the list and
   marked flat, never deleted, so the refusal stays visible.
4. Load the value order from Alignment {OS} and print it. This is the ranking
   rule for every tradeoff below, and it is cited by name each time it is used.
5. Load the capacity ceiling from Health & Energy {OS}, plus any standing veto
   on load. Print the ceiling in hours per week and money per month. Every
   number produced after this step is bounded by it.
6. Collect candidate goals. No filtering yet, no cost yet. Include the carried
   goals from step 1.
7. For each candidate, run `/goal-set`: statement, domain, horizon, cost in
   hours per week and money per month, retirement condition. Drop any candidate
   the user will not put a cost on into an aspirational list, outside the
   allocation.
8. Check each surviving candidate against the Mindset {OS} belief set. Report
   every contradiction, name both records, and hold that goal blocked. Do not
   edit the belief here: hand it to Mindset {OS}.
9. Run `/horizon-map` over the survivors. Place each on a horizon and record
   the sequencing: which goal must land before another can start.
10. Sum the cost of the `now` and `this year` horizons against the ceiling. If
    it overflows, run `/tradeoff` per overflowing claim until the sum is at or
    under the ceiling.
11. Write the not-doing list: every candidate refused, every domain left flat,
    every tradeoff loser, each with its reason. This is the deliverable that
    makes the rest true.
12. State the year in one paragraph the user could say out loud: what they are
    aiming at, what it costs per week and per month, and what they are
    deliberately not doing to afford it.
13. Persist the goal set, horizon map, not-doing list, tradeoff records and
    retirement records through Context & Memory {OS}, after human approval for
    every retirement and every financial commitment.
14. Emit: project briefs to Execution {OS} for goals that are now work,
    behaviour contracts to Habit Tracker {OS} for goals whose path is
    recurring, framing packets to Decision {OS} for any tradeoff that stayed
    unresolved, and the strategy record to Review & Governance {OS}.

## Completion test

Every goal in the active set has a domain, a horizon, a cost in hours per week
and money per month, and a retirement condition. The summed cost of the `now`
and `this year` horizons is at or under the capacity ceiling. Every goal from
last year is either carried or has a retirement record. The not-doing list is
non-empty. Every declared domain is either allocated or explicitly marked flat.

## Failure

- No prior year on record: run steps 3 onward only, and label the output as a
  first strategy with no historical cost baseline.
- No value set from Alignment {OS}: run every tradeoff on explicit user
  preference, label each as unranked preference, and name Alignment {OS} in the
  output as the missing input that would make the ranking defensible.
- No capacity ceiling: ask the user for hours per week and money per month,
  label the ceiling user-stated, and carry that label onto the strategy.
- A goal contradicts a standing belief and the user will not look at the
  belief: hold the goal blocked, keep it out of the allocation, and record the
  contradiction. Do not silently allocate to a blocked goal.
- The user refuses to refuse anything: produce the goal set with an empty
  not-doing list, state plainly that nothing was cut, name the first claims
  expected to fail, and set the first quarterly review as the checkpoint.
- The session reaches clinical, crisis, medical or legal territory: stop the
  strategy work and route to a qualified human professional, immediately.
