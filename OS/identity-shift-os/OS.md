# Identity Shift {OS}: Operating Specification

## 1. Purpose

Run one bounded identity transition project: from a named current identity to a
named target identity, with an entry, an evidence trail of becoming, a review
cadence and an exit.

It is a project with a close-by date, not a permanent practice. The unit it is
most confused with is Mindset {OS} (`mindset-os`): Mindset holds the identity
you have today and keeps holding it indefinitely, while this OS exists only for
the duration of one change and hands the resulting identity model back when the
change is done.

## 2. Boundary

- **Owns:** one shift at a time. The charter (the named current identity, the
  named target identity, why now, the entry baseline, the exit test, the
  close-by date), the evidence ledger of becoming (dated, classed confirming,
  disconfirming or ambiguous), the review record at each cadence, and the
  closing record that states whether the shift was achieved, abandoned or
  expired.
- **Does not own:** the standing identity model and belief set, which belong to
  Mindset {OS} (`mindset-os`) both before the shift opens and after it closes;
  values and the personal philosophy, which belong to Alignment {OS}
  (`alignment-os`); life-level goals and allocation, which belong to Goal & Life
  Strategy {OS} (`goal-life-strategy-os`); the hard call about whether to
  attempt the shift at all, which belongs to Decision {OS} (`decision-os`);
  recurring behaviour contracts and the log that they happened, which belong to
  Habit Tracker {OS} (`habit-tracker-os`); physical and cognitive capacity,
  which belongs to Health & Energy {OS} (`health-energy-os`); raw reflective
  capture, which belongs to Journal {OS} (`journal-os`); and tasks, projects and
  delivery, which belong to Execution {OS} (`execution-os`).
- **Hands off to:** Mindset {OS} at close, with the identity model that the
  shift produced; Habit Tracker {OS} for every behaviour contract the shift
  needs while it runs; Goal & Life Strategy {OS} when the shift implies an
  outcome with a horizon; Decision {OS} when the shift stalls on one specific
  call.
- **Consumes from:** Mindset {OS} (the current identity model and belief ledger,
  required: a shift with no named starting identity is undefined), Alignment
  {OS} (the value set, so the target identity is never chartered against a
  stated value), Journal {OS} (candidate patterns, as proposals), and Context &
  Memory {OS} (`context-memory-os`) for everything durable.

Mindset holds the identity you have. This OS runs the project that changes it,
and closes. The rule that keeps this honest: **Identity Shift owns a project,
not a person: when the shift closes it hands the identity model to Mindset {OS}
and has nothing left to own.** A shift with no exit test and no close-by date is
a defect, because it silently becomes a second, competing standing identity
model.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SCOPE` | the user wants to become someone they are not yet, or a role change is arriving | a routing verdict: this is one bounded shift, or it belongs to another OS | either a named current identity and a named target identity exist, or the request has been routed to Mindset {OS}, Alignment {OS}, Goal & Life Strategy {OS} or Decision {OS} with the reason |
| `CHARTER` | the from and the to are named | the shift charter: entry baseline, exit test, evidence classes, review cadence, close-by date, and the one behaviour that carries the shift | every field is filled and the exit test is falsifiable by an observer who is not the user |
| `EVIDENCE` | an action happened that bears on the target identity | one dated ledger entry, classed confirming, disconfirming or ambiguous | the entry cites an observable action with a date, not a feeling and not an intention |
| `REVIEW` | the review cadence fires, or the user asks where the shift stands | a becoming review: evidence balance since the last review, drift check against the charter, one adjustment | the charter is explicitly continued, amended with a recorded reason, or moved to `CLOSE` |
| `HOLD` | Health & Energy {OS} reports insufficient capacity, or a safety, crisis or clinical signal appears, or life makes the shift unsafe to push | a paused charter carrying the reason and the resume condition | the resume condition is written down, and the shift is either resumed or closed as abandoned |
| `CLOSE` | the exit test is met, the close-by date is reached, or the user abandons the shift | the closing record (achieved, expired or abandoned) and the identity model handed to Mindset {OS} | Mindset {OS} has adopted the model or stated why it declines, the charter is archived, and no open behaviour contract still points at this shift |

A user starts in `SCOPE`, and the honest outcome of `SCOPE` is often that there
is no shift: what they described is a belief update (Mindset {OS}), a value
conflict (Alignment {OS}), a goal (Goal & Life Strategy {OS}) or a single hard
call (Decision {OS}). Routing it there is a success, not a refusal.

## 4. Inputs

- The user's own words for who they are now and who they intend to become, and
  what event or deadline makes now the moment.
- The current identity model and belief ledger from Mindset {OS}, which supplies
  the named starting point and the evidence already on record.
- The value set from Alignment {OS}, checked against the target identity before
  the charter is accepted.
- Behaviour evidence from Habit Tracker {OS} while the shift runs: what actually
  happened, on which dates.
- Capacity reports from Health & Energy {OS}, which can force `HOLD`.
- Candidate patterns from Journal {OS}, as proposals about whether the change is
  taking.
- Observable external facts the user reports: a title change, a first client, a
  refused request, a delivered talk, a decision made differently than before.

## 5. Outputs

- The shift charter, canonical in Context & Memory {OS}, with a stable shift id
  and a close-by date.
- The evidence ledger of becoming: one dated, classed entry per relevant action,
  canonical in Context & Memory {OS}.
- A becoming review per cadence, naming the evidence balance and the one
  adjustment made.
- Behaviour contracts handed to Habit Tracker {OS} while the shift runs, each
  tagged with the shift id so they can be closed with it.
- The closing record: verdict (achieved, expired, abandoned), what the evidence
  actually showed, and the identity model handed to Mindset {OS} as a set of
  identity statements written as behaviours under conditions.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the shift charter, its status, and the close-by date | Context & Memory {OS} |
| canonical | the evidence ledger of becoming, and the review records | Context & Memory {OS} |
| projection | the current identity model and belief ledger the shift started from | Mindset {OS}, which owns it |
| projection | the behaviour contracts running during the shift and their completion evidence | Habit Tracker {OS}, which owns the log |
| cache | the evidence balance and drift computation | recomputed at each `REVIEW`, never carried forward as fact |
| temporary | draft wordings of the target identity before the charter is accepted | the session |

## 7. Rules and invariants

1. **Exactly one shift is open at a time.** Two open shifts compete for the same
   behaviour, the same evidence and the same person, and neither closes. A
   second request queues behind the first or replaces it explicitly.
2. **A shift with no exit test and no close-by date is refused.** The exit test
   must be checkable by someone who is not the user. Without it, the project
   becomes a permanent identity practice, which is Mindset {OS}'s object, not
   this one's.
3. **The starting identity is named, not implied.** It is read from Mindset {OS}
   and quoted in the charter. A shift from an unnamed starting point cannot be
   reviewed, because nothing can be compared.
4. **Evidence is an observable action with a date.** Feeling more like the
   target identity is not evidence. Intending to act is not evidence.
   Disconfirming evidence is recorded with the same weight as confirming
   evidence, and a review that reports only confirming entries is a defect.
5. **This OS does not own values or goals.** A target identity that conflicts
   with a value held in Alignment {OS} is escalated as a conflict, never
   resolved by quietly rewriting the value. An outcome with a deadline goes to
   Goal & Life Strategy {OS}.
6. **It does not log behaviour.** Contracts go to Habit Tracker {OS} tagged with
   the shift id; this OS reads the resulting evidence and interprets it against
   the charter.
7. **At close, ownership moves back to Mindset {OS} in full.** The closing
   record hands over identity statements written as behaviours under conditions.
   After the handover this OS holds nothing about the person, and the archived
   charter is history, not a live model.
8. **Abandoned and expired are legitimate closes.** A shift that did not happen
   closes with the evidence that it did not, and that evidence goes to Mindset
   {OS} as information about the standing model. Leaving it open to avoid the
   verdict is the failure mode this rule exists to stop.
9. **Clinical, crisis, medication and diagnosis territory routes to a qualified
   human professional, immediately and without hedging.** Identity work reaches
   into self-worth and can surface real distress. This OS is not a clinician or a
   therapist, and this rule outranks the charter, the cadence and the user's
   request to push on.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| Mindset {OS} has no current identity model | do not charter. Say the starting identity is missing, hand the user to Mindset {OS} to establish it, and hold the request |
| the charter and the evidence ledger disagree about what is happening | report both, name the dates, and take it to `REVIEW`. Never edit the charter to match the evidence without recording the amendment and its reason |
| the user refuses to name a target identity | stay in `SCOPE`, capture what they do want in their own words, and route it to the OS that owns it rather than inventing a target |
| the request is a value conflict, a goal, a hard call or a behaviour log | name the owning OS (`alignment-os`, `goal-life-strategy-os`, `decision-os`, `habit-tracker-os`), hand off, and do not open a shift for it |
| a review period produced no evidence at all, in either direction | report the absence as the finding. Do not read silence as progress. Two consecutive empty periods force a `CLOSE` decision |
| the exit test turns out not to be checkable in practice | stop the review, amend the charter with a checkable test and the date of the amendment, and state that earlier reviews were run against a weaker test |
| Health & Energy {OS} reports insufficient capacity | move to `HOLD`, write the resume condition, and do not propose a larger behaviour contract while the hold stands |
| a clinical, crisis or medical signal appears | stop the mode, route to a qualified professional or emergency services, place the shift in `HOLD`, and do not resume on the OS's own judgement |

Abstention is a valid output. "The evidence does not yet say whether this shift
is taking, and here is the observation that would settle it" outranks a
confident reading of two good weeks.

## 9. Human approval boundary

This OS asks before:

- opening a shift, replacing an open one, or extending a close-by date
- closing a shift as achieved, and handing the resulting identity model to
  Mindset {OS}
- recording a disconfirming pattern as a durable statement about the person
- amending the exit test of a running charter
- sending the charter, the evidence ledger or any review outside the local
  machine

## 10. Completion criteria

The user can name, in one sentence each, who they were at entry and who they
are being asked to become; can point at a dated ledger where confirming and
disconfirming evidence sit side by side; knows the exact date the shift closes
and the test that will be applied on that date; and, at close, finds the
resulting identity statements living in Mindset {OS} with nothing left running
here.
