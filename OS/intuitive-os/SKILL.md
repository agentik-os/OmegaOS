---
name: intuitive-os
description: Train and calibrate intuition as a usable, falsifiable signal. Intuitive {OS}, unit 07 of the AGENTIK {OS} suite (01 · PERSONAL). Use when the user asks about intuitive or invokes /intuitive-os.
---

# Intuitive {OS}

Pre-verbal signals captured as falsifiable predictions, resolved against
outcomes, and scored into a calibration record with a hit rate per domain.

## When to use this

In the user's own words:

- "Something is off about this deal and I cannot say what."
- "I have a strong feeling about this hire. Should I trust it?"
- "Write this down: I think this launch flops, and here is why I would be
  wrong."
- "Is my gut actually any good at this, or do I just remember the times it was
  right?"
- "Last month I said the partnership would break. It broke. Log it."
- "Decision {OS} is asking what my instinct says on this one."
- "How many of my calls this year actually came true?"

Near neighbours, and the one line that separates them:

| If the ask is really about | It belongs to | Discriminator |
|---|---|---|
| making the call | Decision {OS} (`decision-os`) | it decides. This unit only supplies one weighted input to it and never ranks the options |
| writing down what you felt, without a prediction | Journal {OS} (`journal-os`) | a journal entry has no resolution date. A signal has one, and a disconfirmer |
| whether you believe something about yourself | Mindset {OS} (`mindset-os`) | "I never see these coming" is a belief. "This deal closes by March 1" is a signal |
| what you should want | Alignment {OS} (`alignment-os`) | values are chosen, not predicted, and they are never scored against outcomes |
| whether you were tired or stressed when the feeling arrived | Health & Energy {OS} (`health-energy-os`) | it owns the physical state. This unit records the state marker on the signal and looks for a pattern at review |
| reading the person in front of you, right now | Social Intelligence {OS} (`social-intelligence-os`) | it reads one live interaction and chooses an action. A signal about a person is rewritten here into an observable prediction with a date |
| the standing relationship ledger | Network {OS} (`network-os`) | long-term relationship state. This unit never persists an unresolved suspicion as a fact about a person |

Do not use it for anything with no resolution condition. A question that cannot
be settled by an observable outcome has no shape this unit can score, and
saying so is the correct answer.

## Capabilities

- Capture a signal as an immutable record: claim, domain, confidence 0 to 100,
  base rate, disconfirmer, resolution condition, resolution date.
- Refuse an unfalsifiable claim and offer a falsifiable rewrite of it.
- Resolve a due signal against its pre-registered condition only: hit, miss,
  partial, unresolvable.
- Score each resolution with a Brier score and compare it to the base rate
  reference forecast.
- Compute per domain: resolved count, hit rate, mean Brier, skill against the
  base rate, and the recency-weighted trend.
- Assign a tier per domain (`uncalibrated`, `provisional`, `calibrated`,
  `counter-indicative`) and the weight it earns, capped at 0.5.
- Discount a stale domain by recency half-life, and revert it to
  `uncalibrated` after 24 months without a resolution.
- Emit a weighted signal packet to Decision {OS}, or the `uncalibrated` label.
- Flag capture-quality defects when a domain's unresolvable rate exceeds 30
  percent.
- Detect and mark a retrospective signal so it never scores.

## Procedure

The loop is capture, wait, resolve, score, weight. It only produces value after
the second step has been repeated enough times.

1. Load the signal log and calibration record from Context & Memory {OS}. List
   any signal that is due or overdue before doing anything else.
2. Route the request: a new feeling goes to `CAPTURE`, a due signal goes to
   `RESOLVE`, a live call goes to `CONSULT`, a month boundary goes to `REVIEW`.
3. In `CAPTURE`, take the claim in the user's words and rewrite it into a
   statement that will be observably true or false. Read the rewrite back and
   get agreement before writing.
4. Ask for the domain, the confidence from 0 to 100, and the base rate: how
   often outcomes like this one happen anyway. The base rate is the reference
   forecast the score is measured against, so a signal without one is scored
   against the domain's historical resolution rate and marked as such.
5. Ask what would make this wrong. Refuse anything that names no observable
   event. Offer two candidate disconfirmers derived from the claim if the user
   cannot produce one.
6. Set the resolution condition and the resolution date. Write the record with
   its capture timestamp and any state marker from Health & Energy {OS}. The
   record is immutable from this point.
7. In `RESOLVE`, read the recorded claim, disconfirmer and resolution condition
   out loud first, before discussing what happened. Then take the outcome and
   judge it against that text only.
8. Write the verdict: hit, miss, partial, or unresolvable with a reason.
   Compute the Brier score. A contested outcome is held unresolved and routed
   to human approval.
9. In `CALIBRATE`, recompute the domain: resolved count, misses, hit rate, mean
   Brier with the nine month recency half-life, skill against the base rate
   reference, then the tier and the weight. State the count every tier rests
   on.
10. In `CONSULT`, produce the signal, its disconfirmer, its domain and the
    domain weight. If the domain is `uncalibrated`, say so and pass no weight.
    Never rank the options and never recommend one.
11. In `REVIEW`, chase overdue signals, close what has been open for two
    cycles as unresolvable, report tier changes with their evidence, and flag
    any domain over the 30 percent unresolvable threshold as a capture defect.
12. Persist through Context & Memory {OS}, after human approval where the
    boundary requires it.

## Handoffs

| Receives | What it gets | Shape |
|---|---|---|
| Decision {OS} (`decision-os`) | one weighted signal per live call | signal packet: claim, confidence, disconfirmer, domain, tier, weight (0, 0.25 or 0.5), and the resolved count the weight rests on. `uncalibrated` domains send the label and no weight |
| Journal {OS} (`journal-os`) | resolved signals as pattern material | resolution records: the claim, the verdict, the state marker at capture, the gap between confidence and outcome |
| Mindset {OS} (`mindset-os`) | a sustained counter-indicative domain | a candidate belief to examine: "in this domain, your certainty has run below the base rate across N resolutions". It is a proposal, not a finding, and Mindset {OS} decides what to do with it |
| Context & Memory {OS} (`context-memory-os`) | the canonical record | the signal log, resolution records, and the calibration record per domain |

What it receives back: decision outcomes from Decision {OS} as high quality
resolutions, unlogged signal candidates from Journal {OS}, and state markers
from Health & Energy {OS} at capture time.
