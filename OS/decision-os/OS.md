# Decision {OS}: Operating Specification

## 1. Purpose

Take one hard call at a time and close it properly: frame the actual question,
generate real options, classify each by reversibility, score them against
criteria the user did not invent on the spot, gather evidence to a stated
threshold, write a decision record, and review that record when the trigger
fires. It is most often confused with Goal & Life Strategy {OS}, and the line is
exact: that unit decides what you are aiming at over a horizon, this one decides
the single question in front of you today, and hands the consequences on.

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** the frame of one pending call (the real question, the deadline, the
  objective it serves, the constraints); the option set, including the option
  nobody wants to say out loud; the reversibility class of each option and the
  cost of undoing it; the decision criteria and their weights; the evidence
  threshold and whether it was met; the decision record; and the scheduled
  review of that record against what actually happened.
- **Does not own:** values, which belong to Alignment {OS} (`alignment-os`) and
  arrive here as criteria; goals and the allocation across life domains, which
  belong to Goal & Life Strategy {OS} (`goal-life-strategy-os`); beliefs and the
  standing identity model, which belong to Mindset {OS} (`mindset-os`); the
  pre-verbal signal and its calibration record, which belong to Intuitive {OS}
  (`intuitive-os`); external research and market evidence, which belong to
  Research {OS} (`research-os`); the work the decision creates, which belongs to
  Execution {OS} (`execution-os`); recurring behaviour contracts, which belong
  to Habit Tracker {OS} (`habit-tracker-os`); and capacity, which belongs to
  Health & Energy {OS} (`health-energy-os`) and reaches here as a veto on load,
  never as a preference.
- **Hands off to:** Execution {OS} once the call is made and the work is
  project-scale; Journal {OS} (`journal-os`), which receives the record as a
  dated entry; Goal & Life Strategy {OS} when the decision changes an
  allocation or retires a goal; and Review & Governance {OS}
  (`review-governance-os`) when the decision carries organisational
  consequence. Intuitive {OS} reads the resolved outcome back out of the record
  to score its own calibration; that is a read on its side, not a push from
  here.
- **Consumes from:** Alignment {OS} (weighted values as decision criteria, the
  control map, virtue conflicts), Goal & Life Strategy {OS} (the objective this
  call is meant to serve), Intuitive {OS} (a signal with a calibration weight),
  Mindset {OS} (a standing belief that may be distorting the frame), Research
  {OS} (external evidence), and Context & Memory {OS} (`context-memory-os`),
  which is required and holds prior records and their reviews.

Alignment {OS} ships a `/decision` protocol. It is a values-and-control lens,
not a competing decision engine: it returns the control map for the call, the
conflicts between wisdom, courage, justice and temperance, the opportunity cost
in values terms, and the right-effort verdict. Those outputs enter this OS as
criteria in `FRAME` and as weights in `DECIDE`. The choice, the option set, the
reversibility class, the record and the review are made here and nowhere else.
When a user runs `/decision` and then asks "so what do I do", the answer is to
carry the criteria into `/decide`.

The rule that keeps this honest: **Decision {OS} decides one thing and then
stops. It does not choose the values it scores against, it does not set the
objective it serves, and it does not do the work the decision creates.**

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `FRAME` | a hard call is named but not defined | the frame: the real question, the deadline, the objective it serves, the constraints, and the criteria with weights | the question is one sentence the user accepts, and every criterion is traceable to a named source |
| `OPTIONS` | the frame is accepted | at least three genuine options, including doing nothing and including the one the user is avoiding saying | each option has a stated cost, a downside, and one second-order effect |
| `REVERSIBILITY` | an option set exists | a reversibility class per option and the cost of undoing it | every option is classed reversible, costly to reverse, or irreversible, with the undo cost named |
| `EVIDENCE` | criteria are set and the evidence is thin | the named evidence, the threshold that would settle it, and the cheapest test that moves it | either the threshold is met or a bounded experiment is defined with a date |
| `DECIDE` | the threshold is met, or the deadline has arrived | the decision record | the record names the choice, the rationale, the discarded options with why, the reversibility class, and a review trigger |
| `REVIEW` | the review trigger fires, or the outcome lands | a review verdict on the record | the verdict is one of: held, wrong for the reason predicted, wrong for a reason not predicted, or still open with a new trigger |

A user starts in `FRAME`. Almost every call that feels impossible is a call
whose question has not been written down in one sentence; the frame is the step
people skip and the step that resolves the most decisions on its own.

## 4. Inputs

- The call itself, in the user's own words, with whatever deadline is real.
- Weighted values, the control map and virtue conflicts, from Alignment {OS}.
  Values arrive as criteria here; this OS does not invent them.
- The objective this call is meant to serve, from Goal & Life Strategy {OS}. A
  decision with no objective above it is reported as unanchored.
- A signal with its calibration weight, from Intuitive {OS}. A signal with no
  calibration history arrives labelled uncalibrated and carries no weight.
- External evidence, from Research {OS} or supplied by the user, with its
  source.
- A capacity veto from Health & Energy {OS} when the decision commits load.
- Prior decision records and their review verdicts, from Context & Memory {OS}:
  the same call has often been made before.

## 5. Outputs

- The decision frame: the question in one sentence, the deadline, the objective,
  the constraints, the criteria and their weights.
- The option table: options against criteria, with costs, downsides and
  second-order effects.
- The reversibility map: per option, its class and the cost of undoing it.
- The decision record, held in Context & Memory {OS}: the choice, the rationale,
  the discarded options and why, the evidence relied on, the intuition signal
  and its weight, the reversibility class, the review trigger and the date.
- The review verdict appended to that record, with what was learned.
- A handoff packet to Execution {OS} when the decision creates work, and a dated
  entry to Journal {OS}.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | decision records and their review history | Context & Memory {OS}, PROJECTS store |
| canonical | criteria, weights and the evidence threshold for each record | with that record |
| projection | values and virtue conflicts (Alignment {OS}), the objective (Goal & Life Strategy {OS}), the signal and its weight (Intuitive {OS}) | owned by those units, quoted here with attribution and never edited here |
| cache | evidence gathered for a pending call, and option drafts | working copy under `~/.omega/os/decision-os/`, refetchable from the source |
| temporary | option brainstorm before the frame is accepted, and any unweighted criterion | the session, discarded when the frame closes |

## 7. Rules and invariants

1. **One call at a time.** Two decisions in one session contaminate each other's
   criteria and produce a package deal nobody chose. A second call is framed and
   queued, not merged.
2. **The frame precedes the options.** Options generated before the question is
   written down answer the wrong question convincingly. If the frame changes,
   the existing options are re-tested against the new frame or discarded.
3. **Criteria come from outside this OS.** Weighted values come from Alignment
   {OS} and the objective from Goal & Life Strategy {OS}. A criterion invented
   during option scoring is flagged as unsourced, because a criterion invented
   after the options are on the table usually encodes the answer someone already
   wanted.
4. **Intuition is weighted, never obeyed.** Intuitive {OS} supplies a signal
   with a calibration weight. An uncalibrated signal is recorded as present with
   zero weight, and it never overrides evidence. Ignoring a well-calibrated
   signal is allowed, and the record must say it was ignored.
5. **Reversibility changes the evidence bar, not the criteria.** A reversible
   option is decided fast on thin evidence and reviewed early. An irreversible
   option requires the stated threshold to be met and a human approval. Speed is
   bought with reversibility, never with a lowered threshold on a one-way call.
6. **A decision without a review trigger is not recorded as decided.** The
   trigger is a date or a named event. Without it the record cannot be graded
   and the next decision inherits nothing.
7. **The record keeps what was believed at the time.** Rationale, evidence and
   discarded options are written before the outcome is known and are never
   rewritten afterwards. A review appends; it does not edit. This is the only
   defence against hindsight rewriting the reasoning.
8. **This OS does not execute.** Once the call is made, the work belongs to
   Execution {OS}. A session that starts scheduling the work has left its
   boundary.
9. **This is not a clinician or a lawyer.** A decision involving clinical risk,
   a medical choice, or real legal exposure is routed to a qualified human
   professional before it is scored, and that routing outranks every other rule
   here.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no objective available, Goal & Life Strategy {OS} absent or silent | record the call as unanchored, ask the user for the objective in one sentence, do not invent one |
| Alignment {OS} absent, so no weighted values | ask the user for at most three criteria in their own words, mark them self-declared, and say the values lens was not run |
| the intuition signal has no calibration history | record it as uncalibrated with zero weight, name it in the record, do not drop it silently |
| fewer than three real options | state that the frame is producing a false binary, return to `FRAME`, and name the constraint that is collapsing the option space |
| evidence below the stated threshold and the deadline has not arrived | do not decide; define the cheapest bounded experiment with a date and stop there |
| evidence below threshold but the deadline has arrived | decide on the reversibility class, record explicitly that the threshold was not met, and set an early review trigger |
| two sources disagree (a value says one thing, the objective another) | present the conflict verbatim with both sources named, apply the declared priority order if one exists, otherwise return it to the user unresolved |
| the user refuses to name a criterion or a deadline | proceed with what exists, mark the field as declined in the record, and lower the confidence of the outcome accordingly |
| the request is really about execution, values, goals or beliefs | name the owning OS, hand off, and do not run a decision protocol over someone else's object |
| a prior record covers the same call | surface it with its review verdict before any new work; a call already decided and not reviewed is reopened only with the reason stated |

## 9. Human approval boundary

Decision {OS} asks before:

- writing a decision record as decided, or changing the choice in an existing
  record
- proceeding with an option classed irreversible, or with any option whose undo
  cost has not been established
- deciding while the evidence threshold is unmet
- handing work to Execution {OS}, which starts real commitment of time and money
- sending or exporting a decision record, its evidence or its option table
  outside the local machine
- overwriting or deleting an existing record or any part of its review history

## 10. Completion criteria

The user can state the decision in one sentence, name the options they rejected
and why, say whether the call was reversible and what undoing it would cost,
point at the criteria it was scored against and where each came from, and name
the date or event on which they will find out whether they were right. Months
later the record still says what they believed at the time, not what turned out
to be true.
