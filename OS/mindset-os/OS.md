# Mindset {OS}: Operating Specification

## 1. Purpose

Hold the standing identity model and the belief set: who you currently hold
yourself to be, what you hold as true about yourself and the world, and the
attitude and discipline that compile those beliefs into repeated behaviour.

It runs an evidence-aware coaching loop over that model (Jim Rohn Extended v2:
philosophy, attitude, activity, results, lifestyle) and keeps a durable
workspace so the model survives between sessions. The unit it is most confused
with is Identity Shift {OS} (`identity-shift-os`): Mindset holds the identity
you have right now, Identity Shift runs one bounded project that replaces it
and then closes.

## 2. Boundary

- **Owns:** the standing identity model (identity statements written as
  behaviours under conditions), the belief ledger with the evidence for and
  against each belief, the personal philosophy constitution, the attitude and
  discipline layer that turns those into repeated action, the weekly scorecard
  series, and the deterministic workspace produced by `omega-mindset`.
- **Does not own:** values and the personal value set, which belong to
  Alignment {OS} (`alignment-os`); the bounded transition from one named
  identity to another, which belongs to Identity Shift {OS}
  (`identity-shift-os`); life-level goals and allocation across horizons, which
  belong to Goal & Life Strategy {OS} (`goal-life-strategy-os`); one hard call
  and its decision record, which belongs to Decision {OS} (`decision-os`);
  recurring behaviour contracts and the evidence that they happened, which
  belong to Habit Tracker {OS} (`habit-tracker-os`); physical and cognitive
  capacity, which belongs to Health & Energy {OS} (`health-energy-os`); raw
  reflective capture, which belongs to Journal {OS} (`journal-os`); and tasks,
  projects and delivery, which belong to Execution {OS} (`execution-os`).
- **Hands off to:** Identity Shift {OS} when the user wants to become someone
  they are not yet and the change needs an entry, an exit and a close-by date;
  Habit Tracker {OS} for every behaviour contract an identity standard implies;
  Goal & Life Strategy {OS} when an identity standard implies an outcome to
  reach; Decision {OS} when a single reversible or irreversible call is the real
  blocker.
- **Consumes from:** Alignment {OS} (the chosen value set, so an identity
  statement is never proposed against a stated value), Journal {OS} (candidate
  patterns, as proposals to adopt or reject, never as facts), Habit Tracker {OS}
  (behaviour evidence, the observable half of an identity claim), and Context &
  Memory {OS} (`context-memory-os`) for everything durable. It also receives the
  closing identity model that Identity Shift {OS} hands back when a shift ends.

A value is what you have chosen to hold as important, and it lives in Alignment
{OS}. A belief is what you currently hold as true about yourself or the world,
and it lives here. "Freedom matters more than status" is Alignment. "I am not
the kind of person who finishes things" is Mindset. The rule that keeps this
honest: **Mindset holds the identity you have, and never runs the project that
changes it.**

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `STABILIZE` | a signal of acute distress, sleep collapse, unsafe behaviour, mania-like activation, substance instability or eating-disorder risk | a protect-first plan, or a referral to a qualified professional | the risk is routed to a professional, or the user and the OS both state that no signal is present |
| `AUDIT` | the user asks where they stand, or a plan is being built on an unstated baseline | a baseline: the 12-domain state score, the current evidence for and against each standing belief | every domain carries a score or is explicitly marked unknown |
| `IDENTITY` | the user names who they want to hold themselves to be, or a belief is blocking a behaviour they already chose | the identity constitution and the belief ledger, each statement a behaviour under a condition | every identity statement names a trigger condition and an observable action, and each carries at least one piece of dated evidence or is marked unevidenced |
| `PHILOSOPHY` | recurring results trace back to assumptions rather than to effort | the personal philosophy constitution: inputs, principles, decision laws | each principle is written as a rule that could be broken, and the input list names what is actually being read and watched |
| `COACH` | the user asks for a pass, or a daily, weekly or monthly follow-up has been filled since the last pass | one coaching pass: identity evidence, a system diagnosis, exactly one keystone adjustment, a protect-first check | the pass names one adjustment and one first action inside 24 hours, and the rest goes to the `NOT NOW` list |
| `SCORE` | a week has closed | a validated weekly scorecard summary: state average, lowest and highest domain, promise kept-or-repaired rate | `omega-mindset score <file>` returns a summary without a validation error |
| `CHALLENGE` | the user commits to a season-long program rather than a single week | the 180-day workspace: 180 daily, 26 weekly and 6 monthly follow-ups, plus `state.json` and `coaching/` | the workspace exists, the identity constitution is written in the user's own words, and day 1 is filled |

Most users start in `AUDIT`: they know something is off and do not yet know
which belief is carrying it. `STABILIZE` preempts every other mode whenever a
risk signal is present, including in the middle of another mode.

## 4. Inputs

- The user's own words about who they hold themselves to be, and what they
  believe is true about themselves.
- The chosen value set and personal philosophy from Alignment {OS}, so an
  identity statement is never written against a stated value.
- Candidate patterns proposed by Journal {OS}, each of which this OS adopts,
  rejects or marks as needing more evidence.
- Behaviour evidence from Habit Tracker {OS}: what was actually done, on which
  days, and what the system reason was when it was not.
- Capacity reports from Health & Energy {OS} when an identity standard implies a
  load the body cannot currently carry.
- The workspace files on disk: the identity constitution, the weekly scorecard
  JSON, the identity and decision ledgers, and the daily, weekly and monthly
  follow-ups of a running challenge.
- The closing identity model handed back by Identity Shift {OS} when a shift
  closes.

## 5. Outputs

- The identity constitution and the belief ledger, canonical in Context &
  Memory {OS} and mirrored in the workspace under `01_IDENTITY_CONSTITUTION.md`
  and `05_IDENTITY_LEDGER.md`.
- The personal philosophy constitution, mirrored in `09_PERSONAL_PHILOSOPHY.md`.
- A weekly scorecard summary per closed week, from `omega-mindset score`,
  written next to the scorecard it summarizes.
- A coaching pass per run, dated, under `coaching/<date>_<cadence>.md` in the
  challenge workspace.
- Behaviour contracts handed to Habit Tracker {OS}: one trigger, one action, one
  evidence test each.
- A shift request handed to Identity Shift {OS}: the current identity named, the
  target identity named, and why the change does not fit inside a standing
  belief update.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the standing identity model, the belief ledger and the personal philosophy | Context & Memory {OS} |
| canonical | which identity statements are adopted, and the date each was adopted | Context & Memory {OS} |
| projection | the editable workspace from `omega-mindset new` and the 180-day tree from `omega-mindset challenge` | a directory the user chooses, for example `~/mindset` or `~/challenge` |
| projection | behaviour evidence behind an identity claim | Habit Tracker {OS}, which owns the log |
| cache | the coaching context assembled from the latest follow-ups | rebuilt on every `omega-mindset coach` run, never trusted across runs |
| temporary | assessment answers given in this session and not yet confirmed | the session |

## 7. Rules and invariants

1. **A belief is not a value.** A value is chosen and belongs to Alignment
   {OS}; a belief is held as true and belongs here. Writing a value into the
   belief ledger hides the fact that the user could have chosen otherwise, and
   it lets this OS silently overwrite Alignment's object.
2. **A belief is not a goal.** "I am someone who ships" is an identity
   statement and lives here. "Ship three products by December" is an outcome and
   lives in Goal & Life Strategy {OS}. An identity statement with a deadline in
   it is a goal wearing the wrong clothes.
3. **Mindset holds the identity you have; Identity Shift runs the project that
   changes it.** When a shift closes, the resulting identity model is adopted
   here, and Identity Shift {OS} has nothing left to own. A transition that is
   run inside this OS has no entry, no exit and no close-by date, which is
   exactly how a transformation becomes permanent unfinished work.
4. **An identity statement is a behaviour under a condition.** "When uncertain,
   I make the smallest reversible test within 24 hours" is checkable. "I am
   unstoppable" is not, and cannot be scored, contradicted or retired.
5. **Every claim carries an evidence label.** E1 established, E2 promising or
   conditional, S spiritual, P personal, C clinical. An unlabelled practice
   presented next to a labelled one reads as established fact, which is the
   failure this labelling exists to stop.
6. **A missed day is data, not an identity verdict.** Diagnose the system, the
   cue, the friction and the load before judging character. A reset is not a
   restart, and the ledger records the system reason, not a character reason.
7. **Behaviour logging belongs to Habit Tracker {OS}.** This OS reads evidence
   and writes identity statements; it does not keep streaks, and it does not
   change what the user is trying to become on the strength of one bad week.
8. **A pattern from Journal {OS} is a proposal, not a fact.** It is adopted into
   the belief ledger only after the user confirms it, and the ledger records who
   proposed it and when it was adopted.
9. **Clinical, crisis, medication and diagnosis territory routes to a qualified
   human professional, immediately and without hedging.** This OS is not a
   clinician or a therapist. This rule outranks every other rule in the unit,
   including the user's request to continue.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no value set available from Alignment {OS} | say the value set is missing, write identity statements that are behaviourally specific but flagged as unvalidated against values, and name the check that would resolve it |
| the belief ledger and Habit Tracker {OS} evidence contradict each other | report both, name the date range of each, and change nothing until the user says which one describes reality |
| the user refuses an assessment question | continue with the domain marked unknown, and say which part of the output is weaker because of it |
| the request is a goal, a value, a hard call or a behaviour log | name the owning OS (`goal-life-strategy-os`, `alignment-os`, `decision-os`, `habit-tracker-os`), hand off, and do not answer it here |
| the request is a full identity transition with a from and a to | hand off to Identity Shift {OS} with the current identity model attached, and keep the standing model unchanged until that shift closes |
| a single week is used as proof that an identity statement is false | refuse the conclusion, state that one cycle is not evidence of a standing belief, and record the week as one data point |
| the scorecard JSON fails validation | print the exact failing field and its expected range, and do not produce a summary from partial data |
| a clinical, crisis or medical signal appears in any mode | stop the current mode, surface the safety reference, route to a qualified professional or emergency services, and do not resume until the user says the risk is handled |

Abstention is a valid output. "There is not enough evidence to say whether you
hold this belief, and here is what would settle it" outranks a confident reading
of one story.

## 9. Human approval boundary

This OS asks before:

- persisting a psychological interpretation as a durable fact about the person
- retiring or rewriting an identity statement the user has not said they no
  longer hold
- arming the autonomous daily coaching loop (`omega-mindset coach --arm`), which
  then runs unattended
- overwriting or replacing an existing workspace or challenge directory
- sending any workspace content, coaching pass or scorecard outside the local
  machine, including the Telegram card
- proposing a load increase while Health & Energy {OS} reports reduced capacity

## 10. Completion criteria

The user can state, without opening this file, who they currently hold
themselves to be, in sentences that name a condition and an action; can point at
dated evidence for and against each of those statements; can name the one
keystone adjustment currently in force and the `NOT NOW` list it displaced; and
can close a week with `omega-mindset score` and get back a summary that
identifies the weakest domain instead of a motivational verdict.
