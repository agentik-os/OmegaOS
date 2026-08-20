# Habit Tracker {OS}: Operating Specification

## 1. Purpose

Hold the recurring behaviour contracts a person has agreed to run, and hold the
evidence that each one did or did not happen. It turns a behaviour contract
handed down by another unit into an observable definition, a schedule, a
minimum viable version and a dated log, then returns that log upward as typed
evidence.

The neighbour it is most often confused with is Mindset {OS} (`mindset-os`).
Mindset owns who you hold yourself to be. Habit Tracker owns the specific
recurring behaviour you agreed to perform and the record that says whether you
performed it.

## 2. Boundary

- **Owns:** the habit contract itself (observable definition, cue, context,
  schedule, target, minimum viable version, and for an unwanted habit the
  friction plus the replacement response), its status in the vocabulary
  `DRAFT`, `ACTIVE`, `PAUSED`, `RECOVERING`, `RETIRED`, `ARCHIVED`, the typed
  check-in log with its provenance label, the barrier diagnosis behind an
  intervention, the versioned adaptation experiment with its rollback criteria,
  the operating season (build, maintain, recover, travel, crisis), and the
  computed review with its confidence and its named data gaps.
- **Does not own:** values, the chosen value set and the philosophy behind it,
  which belong to Alignment {OS} (`alignment-os`). Beliefs and the standing
  identity model, which belong to Mindset {OS} (`mindset-os`). A bounded
  transition from one named identity to another, which belongs to Identity
  Shift {OS} (`identity-shift-os`). Goals and the allocation across life
  domains, which belong to Goal & Life Strategy {OS} (`goal-life-strategy-os`).
  A hard call with options and reversibility, which belongs to Decision {OS}
  (`decision-os`). Physical and cognitive capacity, and the envelope that
  habits run inside, which belongs to Health & Energy {OS} (`health-energy-os`).
  Raw reflective capture, which belongs to Journal {OS} (`journal-os`). Tasks,
  projects and delivery, which belong to Execution {OS} (`execution-os`).
  Clinical judgement, which belongs to a qualified human professional.
- **Hands off to:** Mindset {OS} (behaviour evidence against the stated
  identity, as `habit.review.completed`), Journal {OS} (a lapse debrief worth
  reflecting on), Health & Energy {OS} (adherence and load signals that inform
  the capacity assessment), Review & Governance {OS} (`review-governance-os`)
  for any change to these boundaries, schemas or quality gates.
- **Consumes from:** Mindset {OS} (`mindset.behavior_contract.created`),
  Identity Shift {OS} (the recurring behaviour a transition requires as its
  evidence of becoming), Goal & Life Strategy {OS} (a goal converted into a
  recurring behaviour and a cadence), Health & Energy {OS}
  (`handoff.habits.created`, agreed routines and the current capacity
  envelope), Context & Memory {OS} (`context-memory-os`, as
  `memory.context.compiled`).

The rule that keeps this honest: **Habit Tracker {OS} may record what happened,
and may never decide what should happen.** It can report that a contract is not
being met. It cannot change what the person is trying to become, and it cannot
raise a target on its own authority.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SETUP` | a contract arrives from Mindset {OS}, Identity Shift {OS} or Goal & Life Strategy {OS}, or the user asks to start a habit | an identity-linked habit contract, `DRAFT` until agreed then `ACTIVE`, plus a baseline | the behaviour is observable, the minimum version and the lapse recovery rule exist, and the user has agreed to both |
| `CHECK_IN` | the user reports done, partly done or missed | one typed log record labelled `explicit` or trusted `observed` | the record is written with its provenance and the next action is named |
| `URGE` | the user reports temptation before acting | the urge protocol response, and on agreement a friction or replacement change | the user holds a next move that does not depend on willpower alone |
| `LAPSE` | a target was missed, or a chain broke | a blameless antecedent debrief and a protected next opportunity | the antecedent is recorded and the next occurrence has a plan |
| `REVIEW` | the weekly or monthly cadence lands, or the user asks how they are doing | a computed review with confidence, trend, and the data gaps named | the metrics come from logs rather than impression, and one keep, change or stop decision is recorded |
| `RECOVER` | the user reports overload, or Health & Energy {OS} reports capacity below the current load | a `RECOVERING` season with the active set shrunk to essentials | every non-essential contract is `PAUSED`, the essentials are named, and an exit condition for the season exists |
| `ADAPT` | a contract is not being met, or the user asks to change the plan | a versioned experiment carrying success criteria and rollback criteria | the experiment has a start date, a stopping rule, and the previous contract version is retained |

The session router also carries two read-only modes: `TODAY`, which ranks at
most seven primary items for the day, and `VISUALIZE`, which renders the
smallest valid table or diagram for a trend. Both read state and write none.

A real user starts once in `SETUP`, and after that lives in `TODAY` in the
morning and `CHECK_IN` in the evening. Every other mode is entered by an event,
not by a schedule.

## 4. Inputs

- The behaviour contract, from Mindset {OS}, Identity Shift {OS} or Goal & Life
  Strategy {OS}: the identity or goal it serves and the behaviour it asks for.
- The user's own statement of completion, partial completion or miss, in free
  language. This is the primary evidence source and it is labelled `explicit`.
- Trusted device or tool imports (a wearable, a training app, a screen-time
  export), labelled `observed`. An untrusted or unattributed import is not
  evidence.
- The current capacity envelope and any recovery directive from Health & Energy
  {OS}, which bounds how many contracts may be `ACTIVE`.
- The current operating season, stated by the user or proposed by this OS and
  accepted: build, maintain, recover, travel, crisis.
- Declared constraints: illness, injury, travel, grief, disability, religious
  observance, work pattern, and the preferred coaching tone and notification
  pressure.
- Prior canonical state from Context & Memory {OS}: existing contracts,
  confirmed observations and past reviews.

## 5. Outputs

- The habit contract record, versioned, with its status and its provenance,
  staged canonically through Context & Memory {OS} and projected locally.
- The check-in log: one dated typed record per reported occurrence, each
  carrying `explicit`, `observed`, `inferred` or `proposed`.
- The today card: at most seven ranked primary items with the reason for the
  ranking, produced in `TODAY`.
- The review: adherence, cue stability, recovery latency, trend, confidence and
  the named data gaps, emitted as `habit.review.completed` to Mindset {OS}.
- The adaptation experiment record: the change, the success criteria, the
  stopping rule and the version it supersedes.
- The lapse debrief: antecedent, context and the protection put on the next
  opportunity, offered to Journal {OS} as reflective material.
- The export: the user's own state, in a form they can read, correct and
  delete.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | confirmed check-in observations, agreed habit contracts and completed reviews | Context & Memory {OS}, staged as `memory.record.staged` and returned as `memory.record.verified` |
| projection | the fast indexed ledger used for ranking, adherence and streak lookups | the local runtime store at `~/.omega/os/habits-os/ledger/habits.db` |
| projection | the behaviour contract as received from Mindset {OS}, Identity Shift {OS} or Goal & Life Strategy {OS} | read back from the emitting OS, never rewritten here |
| cache | computed review metrics, rankings and rendered diagrams | recomputed from the log, discarded when a log record is corrected |
| temporary | the current session's mode, the pending confirmation, the unaccepted proposal | the session only, never persisted without agreement |

Correcting a log record invalidates every review derived from it. The
correction is recorded as a superseding event; the original is not erased.

## 7. Rules and invariants

1. **Only `explicit` and trusted `observed` records count as completion
   evidence.** An `inferred` record is a model interpretation carrying a
   confidence and an evidence reference, and a `proposed` record has not been
   accepted. Neither may be promoted to a fact without the user confirming it.
   This is what separates a log from a story about a log.
2. **Habit Tracker never sets goals.** It receives a behaviour contract and
   returns evidence. It may report that a contract is not being met; it may not
   change what the person is trying to become. The identity belongs to Mindset
   {OS}, the transition to Identity Shift {OS}, the goal to Goal & Life
   Strategy {OS}.
3. **Health & Energy {OS} sets the envelope, this OS holds the contracts that
   run inside it.** When Health & Energy reports capacity below the current
   load, or forces a recovery season, the active set shrinks. This OS does not
   argue with the capacity assessment and does not restore a paused contract
   until the envelope allows it.
4. **A status is one of six words.** `DRAFT`, `ACTIVE`, `PAUSED`, `RECOVERING`,
   `RETIRED`, `ARCHIVED`. There is no seventh, and `BUILD READY` is not one of
   them: that status belongs to Stepper {OS}. A fixed vocabulary is what lets
   another OS read this state without interpreting prose.
5. **A missed day is data, never a verdict.** Track behaviour, context and
   recovery, never human worth. Streaks are a secondary display and never the
   governing objective; cue stability, minimum viable action, recovery latency
   and trend are what the review is computed on.
6. **Diagnose the barrier before advising, and change one thing.** Capability,
   opportunity, reflective motivation, automatic motivation, overload,
   ambivalence, or unknown. When the barrier is unknown, collect one
   discriminating observation instead of giving generic advice. One primary
   intervention per response unless immediate safety requires more.
7. **An adaptation is an experiment, not a rewrite.** Changes carry success
   criteria, a stopping rule and a retained previous version. Silently editing
   a commitment destroys the only record of what was actually agreed.
8. **Coaching is not treatment.** Clinical risk, medication, eating-disorder
   signals, self-harm or suicidal intent, psychosis, mania, withdrawal, injury
   and acute medical symptoms stop ordinary coaching and route to a qualified
   human professional or emergency support immediately, without hedging. This
   routing outranks every other rule in this unit, including the user's own
   request to continue.
9. **Memory stays inspectable, correctable, exportable and deletable.** No
   pressure mechanic is applied by default: no guilt notification, no
   escalating reminder, no financial penalty, no public exposure.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the behaviour is not observable ("be more disciplined") | refuse to make it `ACTIVE`, state what would make it observable, keep it `DRAFT` |
| a check-in message is ambiguous ("kind of did it") | record what was explicit, mark the remainder unknown, ask one question only if it changes the next action |
| the log and a device import disagree | record both with their provenance, present the conflict to the user, do not pick a winner |
| a review is requested with too few logs | state the number of records available, give the metric with its confidence, name the gap, refuse to assert a trend |
| the user refuses a proposed adaptation | keep the current contract version unchanged, record the refusal as a fact about the plan, do not re-propose it in the same session |
| the request is out of scope (a goal to set, a value to weigh, a hard call) | name the owning unit, hand off, and state what this OS would do once that unit answers |
| a required upstream input is missing (no contract, no capacity envelope) | say what is missing and from which unit, run degraded on what exists, do not invent the missing input |
| a safety signal appears in any mode | stop the mode, surface the safety boundary, route to a qualified human professional, record nothing as a coaching outcome |

## 9. Human approval boundary

This OS asks before:

- promoting an `inferred` or `proposed` record into completion evidence
- changing, pausing, retiring or archiving an `ACTIVE` habit contract
- changing the operating season, including entering or leaving `RECOVERING`
- starting an adaptation experiment, or rolling one back
- deleting user-owned habit state, which is irreversible
- exporting or sending contracts, logs, reviews or debriefs off the local
  machine, to any service, any person or any other OS

## 10. Completion criteria

A user can state a behaviour they agreed to run, receive a contract that is
observable and has a minimum version and a recovery rule, report in one
sentence each day whether it happened, and at the end of a week receive a
review whose numbers they can trace back to individual dated records, with the
gaps named rather than filled in. The evidence they generate arrives at Mindset
{OS} in a shape it can read, and nothing in the record is there that the user
did not state or a trusted device did not measure.
