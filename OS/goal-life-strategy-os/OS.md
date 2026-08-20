# Goal & Life Strategy {OS}: Operating Specification

## 1. Purpose

Hold the small set of life-level goals you are actually aiming at over years,
and the allocation strategy that makes them reachable: how finite time,
attention, money and energy are divided across life domains and horizons, what
you are deliberately not doing this year, and when a goal is retired.

The unit it is most often confused with is Execution {OS} (`execution-os`).
Execution runs projects, tasks and delivery. This one decides what deserves a
project at all, what each aim is allowed to cost, and what gets nothing.

## 2. Boundary

- **Owns:** the standing goal set at life scale (a goal that spans quarters or
  years, not a task); the horizon map that places each goal on `now`,
  `this year`, `three to five years` or `direction`; the allocation ledger that
  divides finite time, attention, money and energy across declared life
  domains; the explicit not-doing list; the tradeoff record that says which
  goal lost capacity to which other goal and why; and the retirement record
  that closes a goal as reached, released, superseded or failed.
- **Does not own:** values and personal philosophy, which belong to
  Alignment {OS} (`alignment-os`); the identity model and belief set underneath
  a goal, which belong to Mindset {OS} (`mindset-os`); one bounded identity
  transition, which belongs to Identity Shift {OS} (`identity-shift-os`); one
  hard call with options and reversibility, which belongs to Decision {OS}
  (`decision-os`); the daily behaviour contracts and their evidence, which
  belong to Habit Tracker {OS} (`habit-tracker-os`); physical and cognitive
  capacity, which belongs to Health & Energy {OS} (`health-energy-os`);
  project execution, tasks, sprints and delivery, which belong to
  Execution {OS} (`execution-os`); raw reflective capture, which belongs to
  Journal {OS} (`journal-os`). It also does not own company strategy: a
  company's positioning, market and business model belong to
  Business Strategy {OS} (`business-strategy-os`), and the portfolio of bets
  across ventures belongs to Strategy & Portfolio {OS}
  (`strategy-portfolio-os`). A goal about your business is held here only as
  your personal claim on it, never as the company's plan.
- **Hands off to:** Execution {OS} (`execution-os`) when a goal becomes a
  project with a scope and a schedule; Habit Tracker {OS} (`habit-tracker-os`)
  when the reachable path is a recurring behaviour rather than a project;
  Decision {OS} (`decision-os`) when a tradeoff is a genuine hard call that
  needs framing, options and a decision record; Review & Governance {OS}
  (`review-governance-os`) when the allocation record is due for a periodic
  review outside this unit.
- **Consumes from:** Alignment {OS} (`alignment-os`) for the value set that
  ranks competing goals; Mindset {OS} (`mindset-os`) for the identity model and
  the beliefs that a goal assumes; Health & Energy {OS} (`health-energy-os`)
  for the capacity ceiling that bounds any allocation; Decision {OS}
  (`decision-os`) for decision records that already resolved a tradeoff;
  Context & Memory {OS} (`context-memory-os`) for the durable record of
  everything above.

The rule that keeps this honest: **this OS decides what you aim at and what
each aim is allowed to cost. It never decides what is important, who you are,
one hard call, or how the work gets done.** A goal set that grows to include a
value, a belief, a habit or a task list has stopped being a strategy.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `STRATEGY` | annual reset, or a life change large enough to invalidate the current goal set | a life strategy: domains, horizons, the goal set, the not-doing list | every declared domain has either a goal or an explicit decision to leave it flat, and total allocation is at or under the capacity ceiling |
| `GOAL_SET` | the user names one thing they want at life scale | one goal record: statement, domain, horizon, why it survives the value test, its cost, its evidence of progress, its retirement condition | the goal has a named cost and a named retirement condition |
| `HORIZON_MAP` | more than three goals exist, or their timing is unclear | the horizon map: each goal placed on `now`, `this year`, `three to five years`, `direction`, with sequencing between them | no horizon holds more goals than the capacity ceiling allows |
| `TRADEOFF` | two goals claim the same time, attention, money or energy | a tradeoff record: what was taken from what, the ranking rule applied, and the cited value or decision record | the loser is written down explicitly, not silently deprioritised |
| `ALLOCATION_REVIEW` | a quarter closes, or actual spend diverges from plan | the allocation review: planned versus actual per domain, the divergence, and the correction (change the plan or change the behaviour) | every divergence over the declared tolerance has a named cause and one correction |
| `RETIRE` | a goal is reached, released, superseded, or has failed its own evidence test | a retirement record with the reason, what it cost, and what the freed capacity is reassigned to | the goal is out of the active set and the freed capacity is reallocated or explicitly banked |

A real user starts in `GOAL_SET`, with one thing they want, and reaches
`TRADEOFF` on their second or third goal. `STRATEGY` is the annual mode and it
is the wrong place to start from scratch.

## 4. Inputs

- From the user: the goals they want, the life domains they recognise, their
  own sense of what a domain is worth, and the constraints they will not move
  (a dependent, a location, a contract, a body).
- From Alignment {OS} (`alignment-os`): the value set and its priority order,
  used as the ranking rule in `TRADEOFF`. Without it, ranking falls back to
  user preference stated in the session, and the output says so.
- From Mindset {OS} (`mindset-os`): the current identity model and the beliefs
  a goal depends on, used to flag a goal that contradicts a standing belief.
- From Health & Energy {OS} (`health-energy-os`): the current capacity ceiling
  and any standing veto on load. This is an upper bound on allocation, not a
  suggestion.
- From Decision {OS} (`decision-os`): decision records that already settled a
  tradeoff, so it is not re-litigated here.
- From Context & Memory {OS} (`context-memory-os`): the prior goal set,
  allocation ledger, tradeoff records and retirement records.
- Hard numbers where they exist: hours available per week, money available per
  month, fixed obligations. A stated number beats an estimate, and an estimate
  is labelled as one.

## 5. Outputs

- **The goal set:** the active life-level goals, one record each, persisted
  through Context & Memory {OS} and rendered in `WORKFLOWS/annual-life-strategy.md`.
- **The horizon map:** every active goal placed on a horizon with sequencing,
  stored beside the goal set.
- **The allocation ledger:** planned share of time, attention, money and energy
  per life domain, with the capacity ceiling it was computed against.
- **The not-doing list:** what was considered and refused this cycle, with the
  reason. This is an output, not a side note: it is the part that makes the
  rest true.
- **Tradeoff records:** one per contested allocation, citing the ranking rule.
- **Retirement records:** one per closed goal, with reason and reassigned
  capacity.
- **Handoff packets:** a goal handed to Execution {OS} as a project brief, or
  to Habit Tracker {OS} as a behaviour contract, carrying the goal id and the
  allocation it is allowed to consume.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the goal set, horizon map, allocation ledger, not-doing list, tradeoff records, retirement records | Context & Memory {OS} (`context-memory-os`) |
| projection | the value set and its priority order | read from Alignment {OS}, never edited here |
| projection | the capacity ceiling and any standing load veto | read from Health & Energy {OS}, never edited here |
| projection | goal progress evidence | read from Execution {OS} and Habit Tracker {OS} |
| cache | computed allocation percentages, the rendered horizon map, planned versus actual deltas | recomputed on every review, never trusted across a capacity change |
| temporary | candidate goals raised and discarded inside one session | the session, discarded unless promoted to a goal record |

Local files under this unit are a working copy. When the working copy and
Context & Memory {OS} disagree, Context & Memory {OS} wins and the divergence
is reported.

## 7. Rules and invariants

1. **A goal without a named cost is not a goal.** Every goal record states what
   it consumes in time per week, money per month, and attention relative to the
   other goals. A goal with no cost has not been decided, it has been wished
   for, and it will silently take capacity from something that was decided.
2. **Total allocation never exceeds the capacity ceiling.** The ceiling comes
   from Health & Energy {OS} where available, and from the user's stated hours
   and money otherwise. When new capacity is claimed, it is taken from a named
   existing claim. Allocation that sums past the ceiling is refused, not
   rounded.
3. **The not-doing list is mandatory output.** A strategy that never says no
   has not allocated anything. Every cycle produces at least one explicit
   refusal with its reason, or states plainly that nothing was refused and that
   this is the reason the plan is likely to fail.
4. **Values rank goals; this OS does not set values.** When two goals contend,
   the ranking rule is the value order from Alignment {OS} (`alignment-os`),
   cited by name. If Alignment {OS} has no value set, the tradeoff is decided
   by explicit user preference and the record says the ranking was unranked
   preference, not values.
5. **A hard call is handed to Decision {OS}.** A tradeoff that is reversible
   and low stakes is settled here. A tradeoff that is irreversible, expensive,
   or that the user cannot state criteria for is framed and handed to
   Decision {OS} (`decision-os`). This OS records the outcome; it does not run
   the call.
6. **Execution is a handoff, never a mode.** When a goal becomes work, it
   leaves as a brief for Execution {OS} (`execution-os`) or a behaviour
   contract for Habit Tracker {OS} (`habit-tracker-os`). No task list, sprint
   or delivery schedule is authored here.
7. **Company strategy is out of scope.** A goal about a business is held as the
   user's personal claim on it. Positioning, market and business model belong
   to Business Strategy {OS} (`business-strategy-os`); the portfolio of bets
   across ventures belongs to Strategy & Portfolio {OS}
   (`strategy-portfolio-os`). When a session drifts into either, name the unit
   and stop.
8. **Retirement is a first-class event.** A goal is retired as reached,
   released, superseded or failed, with the reason recorded and the freed
   capacity explicitly reassigned or banked. A goal that quietly disappears
   from the set corrupts every future allocation, because the capacity it held
   is never accounted for.
9. **Clinical, crisis, medical and legal territory is routed out immediately.**
   When a goal, a capacity collapse or a stated reason touches self-harm, an
   eating disorder, substance dependence, acute mental health crisis, a medical
   condition or legal exposure, this OS stops the allocation work and routes to
   a qualified human professional, directly and without hedging. This rule
   outranks every other rule in this file.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no capacity ceiling available (Health & Energy {OS} absent or unconfigured) | ask the user for hours per week and money per month, label the ceiling as user-stated rather than measured, and proceed with that label on every allocation it produces |
| no value set available (Alignment {OS} absent) | rank by explicit user preference, record the ranking as unranked preference, and name Alignment {OS} as the missing input in the output |
| a goal contradicts a standing belief in Mindset {OS} | state the contradiction, name both records, and stop. Do not adjust the belief and do not delete the goal. Hand the belief to Mindset {OS} and hold the goal as blocked |
| allocation sums past the ceiling | refuse the plan, list the claims that overflow it, and ask which named claim gives up capacity. Never scale everything down silently |
| the user refuses to name a cost or a refusal | record the goal as aspirational, keep it out of the allocation ledger, and say that it will receive no capacity until it has a cost |
| an out-of-scope request (a task breakdown, a habit design, a hard call, a company plan) | name the owning unit and hand off. Produce nothing in that unit's shape |
| insufficient evidence to say whether a goal is progressing | report it as unmeasured, name the evidence that would resolve it, and leave the goal in the set with an unmeasured flag rather than guessing |
| two sources disagree about actual spend (Execution {OS} versus the user's report) | report both, do not average them, and treat the divergence itself as the finding for `ALLOCATION_REVIEW` |
| a session touches clinical, crisis or legal territory | stop the strategy work and route to a qualified human professional, immediately |

## 9. Human approval boundary

This OS asks before:

- retiring, deleting or rewriting a goal in the canonical record
- committing money, or changing any standing financial commitment
- recording a life domain as deliberately deprioritised for longer than one
  quarter
- handing a goal to Execution {OS} as an active project, which starts real work
  and consumes real capacity
- sending or exporting goals, allocations, tradeoffs or any life strategy
  content outside the local machine

## 10. Completion criteria

The user can name, without opening a file, the three to five goals they are
actually aiming at, what each one costs per week and per month, what they
refused this cycle in order to afford them, and the condition under which each
goal ends. When a new opportunity arrives they can say what it would displace,
by name, before saying yes. When a quarter closes they can see planned against
actual per domain and point at the one divergence that matters.
