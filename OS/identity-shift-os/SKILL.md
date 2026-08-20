---
name: identity-shift-os
description: Deliberate identity change: who you must become before what you must do. Identity Shift {OS}, unit 02 of the AGENTIK {OS} suite (01 · PERSONAL). Use when the user asks about identity shift or invokes /identity-shift-os.
---

# Identity Shift {OS}

Deliberate identity change: who you must become before what you must do.

## When to use this

Reach for this OS when the user says something like:

- "I need to stop being a freelancer and start being a founder."
- "I am an engineer and I am about to run a team of eight."
- "I want to be someone who sells, and right now I avoid every sales call."
- "I am leaving employment in four months and I am not that person yet."
- "I have said I would change this for two years and nothing moved."

The signature is a named from, a named to, and a reason the change has to happen
inside a bounded period. If any of those three is missing, the request probably
belongs to another OS.

Near neighbours, and the one line that separates them:

| Confused with | Discriminator |
|---|---|
| Mindset {OS} (`mindset-os`) | it holds the identity you have, permanently. This OS runs one project that replaces it and then closes. |
| Alignment {OS} (`alignment-os`) | it owns values, what you chose to hold as important. This OS moves an identity, and refuses a target that fights a stated value. |
| Goal & Life Strategy {OS} (`goal-life-strategy-os`) | it owns the outcome and the deadline. This OS owns who has to exist for that outcome to be reachable. |
| Decision {OS} (`decision-os`) | it owns the call about whether to attempt the change. This OS runs the change once the call is made. |
| Habit Tracker {OS} (`habit-tracker-os`) | it owns the contract and the log. This OS reads that log as evidence of becoming. |
| Execution {OS} (`execution-os`) | it runs the work. This OS never runs project-scale work; it changes the person doing it. |

## Capabilities

- Decides whether a request is a real shift or belongs to a neighbouring OS, and
  routes it with the reason.
- Names the starting identity by reading it from Mindset {OS} rather than
  assuming it.
- Charters one shift: entry baseline, falsifiable exit test, evidence classes,
  review cadence, close-by date.
- Records dated evidence of becoming, classed confirming, disconfirming or
  ambiguous, with disconfirming evidence carried at equal weight.
- Runs a becoming review at the cadence, reporting the evidence balance and
  drift against the charter.
- Holds the shift when capacity, safety or life makes pushing it unsafe, and
  writes the resume condition.
- Closes the shift as achieved, expired or abandoned, and hands the resulting
  identity model to Mindset {OS}.
- Routes clinical, crisis and medical territory to a qualified human
  professional and stops.

## Procedure

1. Scope it. Ask for the current identity and the target identity in the user's
   own words, and for the event or deadline that makes now the moment. If there
   is no named from, no named to, or no bound, route the request to Mindset {OS},
   Alignment {OS}, Goal & Life Strategy {OS} or Decision {OS} and say why.
2. Read the current identity model and belief ledger from Mindset {OS}. Quote
   the starting identity into the charter verbatim. If Mindset holds no model,
   stop and send the user there first.
3. Check the target identity against the value set from Alignment {OS}. A
   conflict is escalated to the user as a conflict, not resolved here.
4. Write the exit test before anything else in the charter. It must be checkable
   by an observer who is not the user: an action taken, a role held, a request
   refused, a thing shipped. Then set the close-by date.
5. Record the entry baseline: what is true today, dated, in the same terms the
   exit test uses. This is what the close will be compared against.
6. Choose the one behaviour that carries the shift, and hand it to Habit Tracker
   {OS} as a contract tagged with the shift id. Include a floor version that
   survives a bad week.
7. Set the review cadence, weekly by default, and put the close-by date in it.
8. Log evidence as it happens: one dated entry per relevant action, classed
   confirming, disconfirming or ambiguous. Never log an intention.
9. At each review, compute the evidence balance since the last review, check for
   drift from the charter, and make exactly one adjustment. Continue, amend with
   a recorded reason, or move to close.
10. At the close-by date, or when the exit test is met, or when the user
    abandons it, write the closing record: the verdict, what the evidence showed,
    and the identity statements the shift produced, each a behaviour under a
    condition.
11. Hand those statements to Mindset {OS}, close every behaviour contract tagged
    with the shift id, archive the charter, and stop. Nothing about the person
    stays here.

## Handoffs

| Receiving OS | What it receives | Shape |
|---|---|---|
| Mindset {OS} (`mindset-os`) | the closing identity model | identity statements written as behaviours under conditions, each with the dated evidence that produced it, plus the verdict (achieved, expired, abandoned) |
| Habit Tracker {OS} (`habit-tracker-os`) | the behaviour contract that carries the shift | one trigger, one action, one evidence test, a floor version, tagged with the shift id so it closes with the shift |
| Goal & Life Strategy {OS} (`goal-life-strategy-os`) | an outcome the shift implies | the outcome, its horizon, and the identity statement it depends on |
| Decision {OS} (`decision-os`) | one blocking call | the framing, the charter it sits inside, and the reversibility of each option |
| Journal {OS} (`journal-os`) | the review record | the dated review, for reflective capture, not as a conclusion about the person |
| Context & Memory {OS} (`context-memory-os`) | the canonical charter and evidence ledger | shift id, status, close-by date, and every dated classed entry |

