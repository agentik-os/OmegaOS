# Intuitive {OS}: Operating Specification

## 1. Purpose

Treat pre-verbal signals as falsifiable predictions. Capture the signal before
the outcome is known, record what would make it wrong, resolve it against what
actually happened, and score it into a running calibration record with a hit
rate per domain. The output is a signal carrying a weight earned by that
record, or the honest label `uncalibrated`.

It is closest to Decision {OS} (`decision-os`) and is the opposite half of it.
Decision runs the call. This unit only ever supplies one input to that call,
and states how much that input has historically been worth in this specific
domain. It is not mysticism and it is not divination: a signal that cannot be
resolved against an outcome is logged as unresolvable and scores nothing.

## 2. Boundary

- **Owns:** the signal log (one record per captured signal, immutable after
  capture); the disconfirmer attached to each signal; the resolution record
  (hit, miss, partial, unresolvable) against the pre-registered condition; the
  calibration record per domain (resolved count, hit rate, Brier score, skill
  against the base rate, recency); the weight tier a domain has earned; and the
  staleness state of that tier.
- **Does not own:** the decision itself, which belongs to Decision {OS}
  (`decision-os`); values and what is worth wanting, which belong to
  Alignment {OS} (`alignment-os`); beliefs and the identity model, which belong
  to Mindset {OS} (`mindset-os`); goals and allocation, which belong to
  Goal & Life Strategy {OS} (`goal-life-strategy-os`); raw reflective capture
  and the patterns extracted from it, which belong to Journal {OS}
  (`journal-os`); physical states such as fatigue or stress that can produce a
  false signal, which belong to Health & Energy {OS} (`health-energy-os`);
  reading one live social situation, which belongs to
  Social Intelligence {OS} (`social-intelligence-os`); and any work at project
  scale, which belongs to Execution {OS} (`execution-os`).
- **Hands off to:** Decision {OS} (`decision-os`), which receives a signal with
  a calibration weight or the `uncalibrated` label; Journal {OS}
  (`journal-os`), which receives resolved signals as material for pattern
  extraction; Mindset {OS} (`mindset-os`), which receives a sustained
  counter-indicative domain as a candidate belief to examine.
- **Consumes from:** Journal {OS} (`journal-os`) for raw entries that contain
  an unlogged signal; Decision {OS} (`decision-os`) for the outcome of calls a
  signal was attached to, which is the cheapest source of resolutions;
  Context & Memory {OS} (`context-memory-os`) for the durable signal log and
  calibration record.

The rule that keeps this honest: **this OS never decides, and it never asserts
a signal as true. It reports a prediction, its disconfirmer, and the measured
track record of the domain it came from.** A signal presented without its
calibration state, or a domain given a weight it has not earned, is a defect,
not a shortcut.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `CAPTURE` | the user has a pull, a doubt or a certainty and the outcome is not yet known | one immutable signal record: claim, domain, confidence, base rate, disconfirmer, resolution condition, resolution date | the disconfirmer and the resolution date are both present and the record is written |
| `RESOLVE` | a signal's resolution date or resolution event has arrived | a resolution: hit, miss, partial or unresolvable, judged against the pre-registered condition only | the verdict is written and the signal's Brier score is computed |
| `CALIBRATE` | a resolution lands, or the record is inspected | the recomputed calibration record per domain: resolved count, hit rate, Brier score, skill against base rate, tier, staleness | every domain has a tier and every tier states the count it rests on |
| `CONSULT` | Decision {OS} asks for a signal on a live call, or the user asks what their gut says | a signal plus its domain weight, or the explicit label `uncalibrated` | the weight or the `uncalibrated` label is attached and the disconfirmer travels with it |
| `REVIEW` | a month closes, or the unresolvable rate crosses its threshold | the calibration report: per domain trend, overdue signals, capture-quality defects, tier changes | every overdue signal is resolved or reclassified, and every tier change is stated with its evidence |

A real user starts in `CAPTURE`, and gets nothing useful for the first two
months. The record has to exist before the weight can. `CONSULT` before any
resolutions exist returns `uncalibrated`, by design.

## 4. Inputs

- From the user, at capture: the claim in their own words, the life or work
  domain it belongs to, a confidence from 0 to 100, the base rate they think
  this outcome class has, what would make the signal wrong, and the date or
  event that will settle it.
- From the user, at resolution: what actually happened, in plain terms.
- From Decision {OS} (`decision-os`): the outcome of a decision a signal was
  attached to. This is the highest quality resolution source, because the
  outcome is recorded independently of the signal.
- From Journal {OS} (`journal-os`): entries containing an unlogged signal,
  offered as candidates for `CAPTURE`. A candidate lifted from a journal entry
  written after the outcome is retrospective and is never scored.
- From Health & Energy {OS} (`health-energy-os`) where connected: sleep debt,
  stress load or acute fatigue at the moment of capture, recorded on the signal
  as a state marker. It is not used to discount a signal automatically, only to
  make a state pattern visible in `REVIEW`.
- From Context & Memory {OS} (`context-memory-os`): the existing signal log,
  resolutions and calibration record.

## 5. Outputs

- **The signal log:** every captured signal with its full record, persisted
  through Context & Memory {OS}. Immutable after capture.
- **Resolution records:** verdict, the actual outcome, the Brier score, and the
  date it was resolved.
- **The calibration record:** per domain, the resolved count, hit rate, mean
  Brier score, skill against the base rate, the tier, and the age of the newest
  resolution.
- **The weighted signal packet** sent to Decision {OS}: the claim, the
  confidence, the disconfirmer, the domain, the weight, and the count the
  weight rests on.
- **The monthly calibration report:** produced by
  `WORKFLOWS/monthly-calibration-report.md`.
- **Capture-quality defects:** domains whose unresolvable rate is too high,
  reported as a capture problem rather than an intuition problem.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the signal log, resolution records, the calibration record per domain | Context & Memory {OS} (`context-memory-os`) |
| projection | decision outcomes used as resolutions | read from Decision {OS}, never edited here |
| projection | journal entries offered as capture candidates | read from Journal {OS}, never edited here |
| projection | physical state markers at capture time | read from Health & Energy {OS}, never edited here |
| cache | computed Brier scores, hit rates, skill values, tier assignments | recomputed on every resolution, discarded whenever a resolution is corrected |
| temporary | an unwritten signal being drafted inside a session | the session, discarded unless written to the log |

Local files under this unit are a working copy. A signal that exists only in
the working copy has not been captured.

## 7. Rules and invariants

1. **A signal is captured before the outcome is known, or it is not a signal.**
   Every record carries a capture timestamp and the resolution date, and the
   first must precede the second. A signal offered after the outcome is known
   is written to the log marked `retrospective`, is never scored, and never
   contributes to any weight. This single rule is what separates this OS from
   hindsight.
2. **No disconfirmer, no record.** The capture asks what would make this wrong,
   and refuses to write the signal without an answer that could actually occur.
   "I would be wrong if it does not work out" is refused: it names no
   observable. A claim that cannot be wrong cannot be scored and is not a
   prediction.
3. **Resolution is judged against the pre-registered condition only.** At
   `RESOLVE` the recorded disconfirmer and resolution condition are read out
   first, before what actually happened is discussed. The verdict is measured
   against that text, not against a reinterpretation of the signal. Rewriting
   the claim at resolution time is the failure mode this rule exists to stop.
4. **Unresolvable is a legal verdict and it costs the domain nothing but
   count.** A signal whose condition never became observable is logged
   `unresolvable` with its reason, excluded from Brier scoring, and counted
   separately. When a domain's unresolvable rate exceeds 30 percent of its
   signals, the domain is flagged with a capture defect: the signals there are
   not being stated falsifiably, and the correction is sharper capture, not
   more signals. A flagged domain cannot be promoted to a higher tier.
5. **A domain earns a weight; it is never assigned one.** Tiers are computed,
   with the confidence buckets, the base rate and the Brier score defined in
   section 3's `CALIBRATE` mode:
   - `uncalibrated`: fewer than 12 resolved signals in the domain, or fewer
     than 3 recorded misses. Weight 0. The record is too thin, or it contains
     only confirmations, which is not evidence of skill.
   - `provisional`: 12 to 29 resolved signals and a positive skill score
     against the base rate. Weight 0.25.
   - `calibrated`: 30 or more resolved signals and a skill score of 0.10 or
     better sustained across the most recent 20 resolutions. Weight 0.5, and
     0.5 is the hard ceiling for any domain.
   - `counter-indicative`: 20 or more resolved signals with a skill score below
     0. Weight 0, plus an explicit flag that this domain has historically run
     worse than the base rate. It is not automatically inverted: inverting is a
     second bet, and the sample is not large enough to justify it.
6. **The ceiling of 0.5 is deliberate.** A calibrated signal moves a decision;
   it never decides one. Decision {OS} weighs it against evidence, reversibility
   and cost, and a signal that could outweigh all three would make this unit an
   oracle, which is exactly what it is not.
7. **A stale record is discounted, then retired.** Each resolved signal is
   weighted by recency with a half-life of nine months, so a domain calibrated
   three years ago and unused since does not keep speaking. When the newest
   resolution in a domain is older than 12 months, the domain is marked `stale`
   and its weight is halved. Past 24 months it reverts to `uncalibrated` and
   must be re-earned. Calibration is a property of the person you are now, not
   of the person who logged those signals.
8. **This OS never decides.** Its output to Decision {OS} (`decision-os`) is a
   signal, a disconfirmer, a weight and the count that weight rests on. It does
   not recommend an option, it does not rank options, and it does not restate
   the decision. Where the domain is uncalibrated it says so plainly and the
   signal carries no weight at all.
9. **Signals about people are read as predictions about behaviour, not verdicts
   about character.** A signal of the form "this person is dishonest" is
   rewritten at capture into an observable prediction with a resolution date,
   or it is refused. The standing relationship judgement belongs to
   Social Intelligence {OS} (`social-intelligence-os`) and Network {OS}
   (`network-os`), and an unresolved suspicion is never persisted as a fact
   about a person.
10. **Clinical, crisis and safety territory is routed out immediately.** A
    signal about self-harm, harm from another person, an acute mental health
    crisis, or a medical symptom is not a prediction to be scored. Stop the
    calibration work and route to a qualified human professional or emergency
    services, directly and without hedging. This rule outranks every other rule
    in this file.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the user will not state a disconfirmer | refuse to write the signal. Offer two candidate disconfirmers derived from the claim and ask them to pick or edit one. An unfalsifiable claim is not stored as a signal |
| the user will not state a confidence number | write the signal with confidence recorded as absent, mark it unscorable, and exclude it from every calibration computation. Say that it counts for nothing at review time |
| the resolution date has passed and no outcome is known | move the signal to `overdue`, ask once at the next `REVIEW`, and if it is still unknown after two review cycles, close it `unresolvable` with reason `outcome never observed` |
| the recorded outcome and the user's account of it disagree | report both, do not average and do not pick. Hold the signal unresolved and route the contested resolution to human approval, which is where a contested outcome belongs |
| the signal as written does not match the claim the user now says they made | resolve against the written text, record the discrepancy, and note it as a capture-quality defect for `REVIEW`. The written record wins |
| the domain is uncalibrated and the user asks for a weight | return `uncalibrated` with the resolved count and the number of resolutions still needed to reach `provisional`. Do not produce a provisional weight from a thin record |
| an out-of-scope request: decide this for me, predict an event with no resolution condition, read a person's character, interpret a dream as fact | name the owning unit (Decision {OS}, Social Intelligence {OS}, Journal {OS}) or state plainly that the request has no resolvable form, and produce nothing in that shape |
| a domain's unresolvable rate exceeds 30 percent | flag the capture defect, block promotion for that domain, and run a capture-quality pass on its last ten signals instead of collecting more |
| the signal touches self-harm, harm from another, crisis or a medical symptom | stop the calibration work and route to a qualified human professional or emergency services, immediately |

## 9. Human approval boundary

This OS asks before:

- resolving a signal as hit or miss when the outcome is contested or the
  evidence is indirect
- promoting a domain out of `uncalibrated`, or moving any domain to a higher
  weight tier
- attaching a calibration weight to a signal passed to Decision {OS} for a
  decision that is irreversible or expensive
- editing or deleting a signal after its capture, and in particular after its
  resolution date has passed
- sending or exporting the signal log, resolutions or the calibration record
  outside the local machine

## 10. Completion criteria

The user can state, for each domain they log signals in, how many predictions
they have resolved, what fraction they got right, whether that beats the base
rate, and therefore how much their gut is currently worth there. On a live
call they receive either a weighted signal with the count behind it or the word
`uncalibrated`, and never a confident feeling dressed as knowledge. Signals
they can no longer resolve show up as unresolvable and as a capture problem to
fix, not as a mystery.
