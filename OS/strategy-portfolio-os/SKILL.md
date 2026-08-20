---
name: strategy-portfolio-os
description: Convert ambition, evidence and constraints into a coherent strategy, explicit choices, a ranked portfolio of bets and disciplined allocation of time, attention, people and capital. Omega Core function that selects goals, bets, projects and resource allocation before execution begins. Contains 12 specialist agents, 20 skills, 6 protocols and 7 schemas. Use for strategy formulation, bet ranking, portfolio prioritization, or resource allocation decisions across projects. Trigger words: strategy, portfolio, bets, resource allocation, prioritization, strategic choice; FR: strategie, portefeuille, paris strategiques, allocation de ressources, priorisation, choix strategique.
---

# Strategy & Portfolio {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/strategy` | design | Open strategic design |
| `/diagnosis` | diagnose | Define the critical challenge |
| `/portfolio` | portfolio | Review all projects and bets |
| `/prioritize` | portfolio | Rank competing initiatives |
| `/scenario` | scenario | Build future scenarios |
| `/strategic-decision` | decision | Structure a consequential choice |
| `/quarter-plan` | quarter | Create quarterly strategy |
| `/kill-review` | review | Decide continue/pivot/pause/kill |
| `/one-page-strategy` | design | Produce a concise strategy memo |
| `/not-doing` | portfolio | Define exclusions |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

## When to use this

Reach for Strategy & Portfolio when:

- You have more candidate projects than the quarter can carry and no defensible
  way to order them.
- Everything is "a priority" and the calendar shows nobody is actually working
  on the thing that was called most important.
- A concept came out of Brainstorm {OS} and somebody has to decide whether it
  gets money, people and calendar time against every other candidate.
- A market decision or a validation verdict arrived and the funding question is
  now open.
- A project is running badly and nobody agreed in advance what would make it
  stop.
- A quarter is starting and you need outcomes, owners, allocation, signals and
  an explicit list of what is not being done.
- A consequential choice needs a record: authority, alternatives, downside,
  reversibility, dissent and a review trigger.
- A decision hangs on an uncertainty that cannot be resolved before the deadline
  and you need scenarios with signposts instead of a forecast.

Near neighbours, and the line between them:

| Confused with | Difference |
| --- | --- |
| Validation {OS} | Validation settles one falsifiable claim against a threshold signed before the data. Strategy decides what to do about the result, against every competing candidate. A verdict never automatically funds or kills a bet. |
| Market Research {OS} | Market Research compiles the market and customer evidence body and issues one bounded market decision. It gathers; Strategy chooses, and may say the evidence is too thin to choose. |
| Research {OS} | Research answers a stated question with defensible outside sources. Strategy consumes memos, it never runs the study. |
| Brainstorm {OS} | Brainstorm invents, evolves and converges to one selected concept. Strategy decides whether that concept is funded at all. |
| Blueprint {OS} | Blueprint defines the selected product in full. Strategy only names the selected bet and what it must prove. |
| Execution {OS} | Execution performs personal work against committed outcomes. Strategy sets the outcomes and exclusions, then stays out of the week. |
| Decision {OS} | Decision handles one hard call under irreducible uncertainty and values. Strategy holds a portfolio of bets that must cohere and share one finite pool of resources. |
| Review & Governance {OS} | Governance approves consequential change and closes the learning loop. Strategy requests approval and obeys the answer, and never approves its own consequential change. |
| Business Strategy {OS} | That unit runs an existing business as an owned asset. This unit chooses which bets exist at all. |

## Capabilities

- Diagnose the critical challenge behind an ambition, and refuse to rank
  initiatives underneath an undiagnosed goal.
- Build the strategy kernel: diagnosis, guiding policy that rules options out,
  and a set of mutually reinforcing actions.
- Inventory the whole portfolio, including the maintenance and half-finished
  work nobody counts, and score each item on fit, evidence, upside, learning
  value, cost and downside.
- Rank competing initiatives and state the opportunity cost of each funded one:
  which candidate loses the hours, the people or the money.
- Allocate time, attention, people and capital against real capacity, and report
  an overcommitment as an overcommitment rather than publishing an optimistic
  plan.
- Attach kill criteria to every bet before it starts, in observable terms with a
  date or a threshold.
- Run reversibility analysis and propose the cheapest reversible experiment that
  buys the same information as an irreversible commitment.
- Build scenarios with signposts, no-regret moves and contingent options,
  without producing a single predicted future with false precision.
- Write a strategic decision memo with authority, deadline, alternatives,
  expected value, downside, dissent and a review trigger.
- Produce the quarterly strategic plan and the explicit not-doing list, and audit
  OKR quality against the doctrine that metrics represent progress, not activity.
- Run a kill review against the ORIGINAL thesis and thresholds, name the sunk
  cost when it is being used as an argument, and release the resources.
- Hand off cleanly: the product branch to Blueprint {OS}, the personal execute
  branch to Execution {OS}, consequential change to Review & Governance {OS}.

## Procedure

The operating loop is SITUATION, DIAGNOSIS, CHOICES, BETS, ALLOCATION,
EXECUTION HANDOFF, SIGNALS, REVIEW, then DOUBLE DOWN / ADAPT / KILL.

1. **Situation.** Establish intent and decision horizon. Retrieve the minimum
   authorized context: prior kernels, current portfolio, live constraints,
   standing exclusions. Never re-open a decision without saying which evidence
   changed.
2. **Separate the material.** Sort what you have into fact, statement,
   inference, assumption and unknown. Label every material claim E1 to E5. An
   E4 presented as an E1 is the error this step exists to prevent.
3. **Diagnose.** Name the critical challenge as a single obstacle a policy could
   act on, with the evidence behind it. If several problems compete, rank them
   by how much else each one unblocks and ask which is the challenge.
4. **Choose the guiding policy.** It must rule something out. Verify the
   exclusion is real by naming an attractive action the policy forbids.
5. **Design coherent actions.** Show how each action reinforces the others.
   Report any pair that pulls against another instead of quietly keeping both.
6. **Inventory and score the bets.** Every active and proposed item, including
   hidden maintenance. For each: thesis, strategic fit, evidence score, resource
   cost in time, capital and people, and kill criteria.
7. **Check capacity before ranking is published.** Hours that genuinely exist,
   people who genuinely exist, capital that can genuinely be committed. Consume
   `health.capacity.assessed` and `capital.reallocation.proposed` rather than
   assuming.
8. **Rank and allocate.** Fund, experiment, hold, pause or kill each item. State
   the opportunity cost of every funded bet. Publish the not-doing list with the
   condition that would reopen each exclusion.
9. **Request governance.** For any consequential funding, pause, kill or
   allocation change, emit `strategy.change.requested` and wait. Do not emit the
   portfolio or allocation event before `change.approved` returns.
10. **Hand off.** The selected product bet goes to Blueprint {OS}; the quarterly
    outcomes, owners and exclusions go to Execution {OS} as an execution packet.
    Neither handoff restates work the receiving OS owns.
11. **Instrument the signals.** For each bet, one leading signal, one lagging
    signal and a guardrail, each naming the decision it may affect, plus the
    review trigger that will bring the bet back here.
12. **Review.** When a trigger fires or a signpost is hit, compare actual
    evidence against the ORIGINAL thesis and thresholds. Name any sunk-cost
    argument out loud. Choose continue, narrow, pivot, pause or kill, capture
    the reusable learning, and re-assign or explicitly bank the released
    resources.
13. **Record.** Write objectives, bets, portfolio items, allocations, scenarios,
    decisions and metrics to canonical state through Context & Memory {OS}, each
    with owner, completion evidence, review trigger and handoff identifier. A
    kill is retained, never deleted; a decision is superseded, never overwritten.

## Handoffs

| To | Event | What it does with it |
| --- | --- | --- |
| Blueprint {OS} | `strategy.product_bet.approved` | defines the selected product in full, on a bet that is already funded |
| Execution {OS} | `strategy.execution_packet.created` | commits the quarterly outcomes, owners and exclusions into personal execution |
| Review & Governance {OS} | `strategy.change.requested` | approves or refuses a consequential funding, pause, kill or allocation change |
| Review & Governance {OS} | `strategy.refresh.requested` | closes the Review to Context to Strategy learning loop |
| Context & Memory {OS} | `strategy.diagnosis.created`, `strategy.kernel.approved`, `strategy.review.completed` | makes the kernel, the diagnosis and the review durable across sessions |
| Portfolio consumers | `portfolio.item.funded`, `portfolio.item.paused`, `portfolio.item.killed`, `allocation.changed` | emitted only after `change.approved` for the matching `strategy.change.requested` |
| Scenario watchers | `scenario.signpost.triggered` | reopens the affected decision with the contingent action already prepared |

Received from: Market Research {OS} (`market.validation.completed`,
`market.sizing.modeled`), Validation {OS} (`validation.verdict.issued`,
`validation.claim.killed`), Business Model {OS}
(`business_model.viability.assessed`, `business_model.unit_economics.modeled`),
Trend & Opportunity {OS} (`opportunity.named`, `trend.movement.confirmed`,
`opportunity.window.closed`), Research {OS} (`research.evidence.compiled`),
Health & Energy {OS} (`health.capacity.assessed`), Wealth {OS}
(`capital.reallocation.proposed`), Review & Governance {OS}
(`change.approved`), Execution {OS} (`execution.outcome.proven`), Context &
Memory {OS} (`memory.context.snapshot.created`), Mindset {OS}
(`mindset.identity_compilation.updated`).
