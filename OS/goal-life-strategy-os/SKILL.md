---
name: goal-life-strategy-os
description: Life-level goals and the strategy that makes them reachable. Goal & Life Strategy {OS}, unit 04 of the AGENTIK {OS} suite (01 · PERSONAL). Use when the user asks about goal & life strategy or invokes /goal-life-strategy-os.
---

# Goal & Life Strategy {OS}

Life-level goals, and the allocation of finite time, attention, money and
energy that makes them reachable.

## When to use this

In the user's own words:

- "I have too many things I want and I am making progress on none of them."
- "What should I actually be aiming at over the next three years?"
- "I said yes to this. What does it push out?"
- "I have been working on this goal for two years. Is it still mine?"
- "I want to get in shape, build the business, and be around more. Pick."
- "The quarter is over. Where did my time actually go against what I planned?"
- "How much of my money and my week is this goal allowed to take?"

Near neighbours, and the one line that separates them:

| If the ask is really about | It belongs to | Discriminator |
|---|---|---|
| what is important to you | Alignment {OS} (`alignment-os`) | a value ranks goals, it is not a goal. "Freedom over status" is Alignment |
| who you hold yourself to be | Mindset {OS} (`mindset-os`) | "I am someone who ships" is a belief. "Ship three products by December" is a goal |
| becoming a different person, as a bounded project | Identity Shift {OS} (`identity-shift-os`) | a shift has an entry, evidence and an exit. A goal has a cost and a retirement condition |
| one hard call, right now | Decision {OS} (`decision-os`) | one call with options and reversibility is Decision. The standing allocation those calls happen inside is here |
| the daily behaviour that gets you there | Habit Tracker {OS} (`habit-tracker-os`) | a recurring contract plus evidence is Habit Tracker. What the contract is in service of is here |
| whether your body can carry the load | Health & Energy {OS} (`health-energy-os`) | it reports the ceiling and may veto a load. This unit allocates under that ceiling |
| doing the work | Execution {OS} (`execution-os`) | tasks, projects, sprints, delivery. This unit decides what deserves a project at all |
| your company's plan | Business Strategy {OS} (`business-strategy-os`) | positioning, market, business model. A business goal is held here only as your personal claim on it |
| which ventures to bet on | Strategy & Portfolio {OS} (`strategy-portfolio-os`) | a portfolio of company bets, not a life allocation |

## Capabilities

- Define one life-level goal with a domain, a horizon, a named cost and a
  retirement condition.
- Map every active goal onto `now`, `this year`, `three to five years` and
  `direction`, with sequencing between them.
- Compute an allocation of time, attention, money and energy across declared
  life domains, bounded by the capacity ceiling.
- Force and record a tradeoff when two goals claim the same capacity, ranked by
  the value order from Alignment {OS}.
- Produce the not-doing list: what was considered and refused, with reasons.
- Review planned against actual allocation at the close of a quarter and name
  the correction.
- Retire a goal as reached, released, superseded or failed, and reassign the
  freed capacity.
- Flag a goal that contradicts a standing belief held in Mindset {OS}.
- Hand a goal off as a project brief to Execution {OS} or a behaviour contract
  to Habit Tracker {OS}.

## Procedure

1. Load state: the existing goal set, horizon map, allocation ledger and
   not-doing list from Context & Memory {OS}. Name anything missing.
2. Load the ranking rule: the value set and priority order from Alignment {OS}.
   If absent, say so and fall back to explicit user preference, labelled.
3. Load the ceiling: capacity and any standing load veto from
   Health & Energy {OS}. If absent, ask the user for hours per week and money
   per month, and label the ceiling as user-stated.
4. Take the ask and route it to a mode: `GOAL_SET` for one new goal,
   `HORIZON_MAP` for timing, `TRADEOFF` for a contested claim,
   `ALLOCATION_REVIEW` at quarter close, `RETIRE` for a goal that is done,
   `STRATEGY` for an annual reset.
5. For each goal in scope, establish four fields before anything else: the
   statement, the domain, the horizon, and the cost in hours per week and money
   per month. A goal missing the cost is held as aspirational and gets no
   capacity.
6. Establish the retirement condition: the observable event that ends this
   goal, whether by success or by release. A goal with no end has no cost
   ceiling either.
7. Sum the allocation against the ceiling. On overflow, refuse the plan and ask
   which named claim gives up capacity. Do not scale everything down.
8. Record the tradeoff: what lost, what won, and the value or decision record
   that ranked them. Add the loser to the not-doing list with its reason.
9. Check each goal against the Mindset {OS} belief set. Report contradictions,
   do not resolve them here.
10. Write the canonical records through Context & Memory {OS}, after human
    approval for anything in the approval boundary.
11. Emit handoffs: a project brief to Execution {OS}, a behaviour contract to
    Habit Tracker {OS}, a framed call to Decision {OS}, a review packet to
    Review & Governance {OS}.
12. Close by stating what changed, what it cost, and what was refused to
    afford it.

## Handoffs

| Receives | What it gets | Shape |
|---|---|---|
| Execution {OS} (`execution-os`) | a goal that became work | project brief: goal id, outcome, the allocation it may consume (hours per week, money per month), the deadline horizon, the evidence that counts as progress |
| Habit Tracker {OS} (`habit-tracker-os`) | a goal whose path is recurring behaviour | behaviour contract: the behaviour, its cadence, the goal id it serves, the allocation it consumes. This unit does not define the evidence rules |
| Decision {OS} (`decision-os`) | a tradeoff that is a genuine hard call | framing packet: the two or more claims, what each would cost, the value order that failed to separate them, and what makes it irreversible |
| Review & Governance {OS} (`review-governance-os`) | the periodic allocation record | review packet: planned versus actual per domain, divergences over tolerance, retirements this cycle |
| Context & Memory {OS} (`context-memory-os`) | every canonical record | goal records, horizon map, allocation ledger, not-doing list, tradeoff records, retirement records |

What it receives back: progress evidence from Execution {OS} and
Habit Tracker {OS}, capacity from Health & Energy {OS}, decision records from
Decision {OS}, values from Alignment {OS}, beliefs from Mindset {OS}.
