---
name: trend-opportunity-os
description: Spot movement early and turn it into a named opportunity. Trend & Opportunity {OS}, unit 13 of the AGENTIK {OS} suite (02 · DISCOVER & DECIDE). Use when the user asks about trend & opportunity or invokes /trend-opportunity-os.
---

# Trend & Opportunity {OS}

Watch a domain over time, confirm movement with a direction and a rate, and name
one opportunity with a window that will close.

## When to use this

Reach for Trend & Opportunity when:

- You keep hearing that something is "taking off" and nobody in the room can say
  since when, how fast, or from what to what.
- A decision depends on timing rather than on size: whether to move now, in six
  months, or not at all.
- You want to be told, on a date, when a chance you were counting on has expired.
- You have a domain you care about and no systematic way of noticing change in
  it, so you only find out from a competitor's launch.
- Someone brought a deck full of screenshots and hockey sticks and you want to
  know which of them survive a date check and a source independence check.
- An opportunity has been sitting in a plan for a year and nobody has asked
  whether its window is still open.
- You need to know whether a behaviour actually changed, or whether only the
  coverage of that behaviour changed.

Near neighbours, and the line between them:

| Confused with | Difference |
|---|---|
| Market Research {OS} | Market Research measures a market at a point in time (size, segments, competition, pricing evidence) and issues a market decision. This OS watches over time and reports a direction and a rate. A rate is not a size, and this OS never answers "how big is it". |
| Brainstorm {OS} | This OS detects movement in the world and names an opportunity: who, what changed, why now, what window. Brainstorm invents and evolves concepts, with or without a trend behind them. An opportunity is a constraint and a deadline, not an idea. |
| Research {OS} | Research answers a stated question with defensible outside sources and delivers a memo. This OS keeps a standing watch and captures dated observations over months. Research is a question closed; a watch is a question left deliberately open. |
| Strategy & Portfolio {OS} | This OS says the window is open and closing on a date. Strategy decides whether that window gets money, people and calendar time against every other candidate. A confirmed movement never funds itself. |
| Customer Discovery {OS} | Discovery is the only unit in this group that talks to real humans, and it does so to learn what is true about them. This OS reads signals about behaviour at a distance, and consumes `discovery.insight.confirmed` when a real behaviour change has been observed in people. |

## Capabilities

- Turn a vague interest ("AI agents", "our category") into a watch with a named
  behaviour that would have to change, real sources, a cadence and a review date.
- Capture signals with both dates that matter: when the thing happened, and when
  you saw it.
- Label each source by interest: disinterested, participant, vendor selling into
  the trend, or funded by a participant.
- Detect source collapse, where five apparently independent reports cite one
  press release and count as one observation.
- Separate attention measures (coverage, search, conference agendas, posts) from
  behaviour measures (deployments, purchases, hires, migrations, filings,
  cancellations), and refuse to confirm a behavioural trend on attention alone.
- Compute a coarse rate over a stated window from dated observations, and say
  plainly when the observations cannot support one.
- Confirm movement: direction, rate, window, supporting signals, contradicting
  signals.
- Name an opportunity: who can act, what changed, why now rather than three
  years ago, what window, what closing condition, what expiry date.
- Review an opportunity on its date and either restate it with fresh evidence,
  narrow it, or retire it.
- Retire an opportunity with a record, and notify the units that were betting
  on it.
- Audit a claimed trend that arrived from outside: check dates, source
  independence, interest, and whether behaviour or only attention moved.

## Procedure

1. **Define the watch domain and the behaviour.** State the decision the watch
   serves, then write the behaviour that would have to change for the trend to
   be real, before collecting anything. "More interest in X" is rejected. "Teams
   with under 50 people replacing tool Y with X in production" is accepted,
   because you can go and look for it.
2. **Choose sources and cadence.** Pick at least two independent sources, and at
   least one that is not selling into the trend. Record each source's interest
   label and its access constraint (free, paid, rate limited, terms restricted).
   Set the sweep cadence and the next review date. Anything paid, restricted or
   automated on a schedule goes to the approval boundary now, not later.
3. **Capture dated signals.** For each observation record: what was observed,
   the observation date, the capture date, the source and its locator, the
   interest label, and which watch it belongs to. Never capture a signal you
   cannot date. Pull existing dated material from Librarian {OS} first, so the
   watch does not start from zero.
4. **Separate noise from movement.** Sweep the accumulated signals. Dismiss with
   a stated reason (one off, source collapse, attention only, restatement of an
   older signal, out of scope), or attach to a named candidate. Check
   independence before counting: five citations of one origin are one
   observation.
5. **Confirm movement, or refuse to.** A candidate becomes movement only with
   several dated observations, spread over time, across independent sources,
   with at least one disinterested source, showing a direction AND a rate over a
   stated window. If the rate cannot be computed, say so and keep the candidate
   open. Record contradicting signals inside the movement record rather than
   dropping them. Emit `trend.movement.confirmed`.
6. **Name the opportunity.** Four parts, all mandatory: **who** can act (a
   specific actor who can actually reach it), **what changed** (the confirmed
   movement, direction and rate), **why now** (what makes this moment different
   from a year ago and from a year from now), **what window** (the closing
   condition that would shut it, plus an expiry date if nothing else fires).
   Without an expiry date the opportunity is not named, it is admired. Emit
   `opportunity.named`.
7. **Review and retire.** On the review date, sweep again and restate the window
   with fresh dated evidence, narrow it, widen it, or close it. When the closing
   condition fires or the expiry passes, write the retirement record: what
   closed, when, why, what was learned, and who was betting on it. Emit
   `opportunity.window.closed`. An opportunity that quietly lives forever
   corrupts every downstream decision that reads it as current.

## Handoffs

| To | What it receives | What it does with it |
|---|---|---|
| Brainstorm {OS} | `opportunity.named` | treats the opportunity as the constraint and the deadline a concept must exploit, then generates and converges to one selected concept |
| Strategy & Portfolio {OS} | `trend.movement.confirmed`, `opportunity.named`, `opportunity.window.closed` | weighs the window against every other candidate bet, and reopens a kill review when a window closes under a funded bet |
| Market Research {OS} | `trend.movement.confirmed`, `opportunity.named` | sizes the market behind the movement and issues the market decision this OS deliberately does not make |
| Context & Memory {OS} | `trend.signal.captured`, and every canonical record | makes watches, signals, movements and opportunities durable and readable across sessions and OS units |

Received from: Librarian {OS} (`librarian.source.indexed`, dated material
already in your corpus), Research {OS} (`research.evidence.compiled`, the
mechanism behind a movement), Customer Discovery {OS}
(`discovery.insight.confirmed`, a behaviour change observed in real people
rather than in coverage).
