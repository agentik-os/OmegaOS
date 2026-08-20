# Execution {OS}: Operating Specification

## 1. Purpose

Turn ambitions, obligations and ideas into a small number of time-bound
personal commitments, protect the hours those commitments need, and close each
one with evidence that it actually shipped.

Execution {OS} treats a day as a closed control loop, not a list:

```text
Capture -> Clarify -> Select -> Commit -> Focus -> Prove -> Review -> Adapt
```

The loop exists because the common failure is not laziness. It is an open day:
work started, nothing closed, no evidence, and no first physical action written
for tomorrow.

## 2. Boundary

- **Owns:** the personal commitment (one outcome, one owner, one deadline, one
  defined next action), the daily capacity budget, the protected focus block,
  the proof that a commitment was completed, the promise ledger held with other
  people, and the recovery path when a day, a week or a commitment fails.
- **Does not own:**
  - **The software build pipeline.** Specifying, planning, building and
    releasing software belongs to Blueprint {OS}, Stepper {OS} and Builder
    {OS}. Execution {OS} never writes a technical spec, a build plan or code.
    It only holds the commitment that says a build step will happen today.
  - **Multi-week scoped work with milestones and dependencies.** That is
    Project {OS}. Execution {OS} consumes a project's next actions; it does not
    plan the project.
  - **Consistency of repeated behaviour over months.** That is Habits {OS}.
  - **Identity, motivation and self-narrative.** That is Mindset {OS}.
  - **Delegated work.** The moment a commitment leaves your own hands it
    belongs to Team & Delegation {OS}, which returns a promise date.
- **Hands off to:** Project {OS} (a commitment that turns out to be a project),
  Team & Delegation {OS} (a commitment that should not be yours), Review &
  Governance {OS} (the weekly and monthly truth, and any change to how the
  system itself works), Documentation {OS} (proof artifacts worth keeping).
- **Consumes from:** Project {OS} (next actions from live projects), Meeting
  {OS} (action items whose owner is you), Client {OS} (promises made to
  clients), Habits {OS} (what must recur), Goal & Life Strategy {OS} (what the
  quarter is for), Context & Memory {OS} (your profile, capacity and history).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `BOOT` | the day starts | capacity, usable minutes, one must-win, the day's commitments | a must-win exists and every open commitment has a defined next action |
| `CAPTURE` | anything arrives from any source | an inbox entry, unjudged | the entry is stored with its source |
| `CLARIFY` | inbox has entries | each entry classified: commitment, project, delegation, reference, or dropped | the inbox is empty of unclassified entries |
| `COMMIT` | a clarified entry belongs to you, today or this week | a commitment with outcome, acceptance test, deadline and one physical next action | all four fields are present, none of them vague |
| `FOCUS` | a commitment is selected and time is available | a protected block of 25, 50 or 90 minutes on exactly one commitment | the block ends, or the commitment completes |
| `PROVE` | a commitment is claimed complete | evidence plus a stated acceptance test that the evidence satisfies | evidence exists and is inspectable by someone else |
| `RECOVER` | a commitment is blocked, missed, or wrong | a classification (blocked, deferred, cancelled, delegated) and a physical next action | the commitment is no longer silently open |
| `HALT` | the day ends | a day classification, a proof, and tomorrow's first physical action | tomorrow's first action is written |
| `RESET` | the week ends | the week's honest truth, next week's single win, and one system experiment | the experiment is named and testable |
| `AUDIT` | the month ends | one change to the system, not to the task list | the change is recorded and its test is stated |

`BOOT` and `HALT` are the two non-optional modes. A day that was booted and
never halted is an open day, and an open day is the failure state this OS
exists to prevent.

## 4. Inputs

- **Capacity.** GREEN, AMBER or RED, plus usable minutes. Self-reported, and
  taken at face value: the OS does not argue with a stated RED day, it resizes
  the day to fit it.
- **The must-win.** One outcome that makes the day count if nothing else lands.
- **Open commitments.** Carried from previous days, each with its next action.
- **Incoming obligations.** Meeting actions, client promises, project next
  actions, and anything captured during the day.
- **The operator profile.** Working hours, block length, recurring
  constraints, and the vocabulary the user actually uses. Read from Context &
  Memory {OS}; a local profile always overrides the shipped default.

## 5. Outputs

| Output | Shape | Where it lives |
|---|---|---|
| Daily command card | must-win, capacity, commitments, blocks | execution state ledger |
| Commitment record | outcome, acceptance test, deadline, next action, status | execution state ledger |
| Proof of completion | evidence reference plus the acceptance test it satisfies | ledger, and Documentation {OS} when it is worth keeping |
| Halt card | day classification, energy, focus, friction, proof, tomorrow's first action | execution state ledger |
| Promise ledger entry | who, what, by when, notice-by date, consequence if late | ledger, and Client {OS} when the promise is to a client |
| Weekly reset | truth, next week's win, one system experiment | ledger, and Review & Governance {OS} |
| Monthly audit | one system change plus its test | Review & Governance {OS} |

Day classifications are deliberately five, not two: `SHIPPED`, `VERIFIED`,
`PROGRESSED`, `TOUCHED`, `ABANDONED`. A binary done or not-done hides the
difference between a day that moved and a day that only felt busy.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | commitments, blocks, proofs, promises, day records | `~/.omega/os/execution-os/ledger/execution-state.json` |
| canonical | the operator profile | `~/.omega/os/execution-os/ledger/profile.md`, mirrored through Context & Memory {OS} |
| projection | project next actions, meeting actions, client promises | owned by Project, Meeting and Client {OS} |
| cache | today's selection and ordering | recomputed at every `BOOT` |
| temporary | the current focus block timer | the session |

The ledger is local files. Nothing about a user's execution history is trapped
in one AI vendor, and a proof recorded in the ledger stays readable after this
OS is uninstalled.

## 7. Rules and invariants

1. **Single thread.** One primary outcome per day, one commitment per focus
   block. A block that serves two commitments served neither.
2. **Defined next.** Every open commitment carries exactly one physical,
   startable next action. "Think about the pricing" is not one. "Write three
   price options in the pricing note" is.
3. **Closed day.** The day is not closed until tomorrow's first physical action
   is written. This is the single rule that most reduces next-morning drift.
4. **No completion without proof.** A commitment closes on evidence plus the
   acceptance test the evidence satisfies. Self-reported completion with no
   artifact is recorded as `TOUCHED`, never as shipped.
5. **Capacity is a budget, not a wish.** Committed minutes never exceed usable
   minutes. If they do, the OS asks which commitment leaves the day, and
   refuses to simply add.
6. **A missed day is data.** Recovery is a classification plus a next action,
   never a restart of the system and never a moral verdict.
7. **Domains stay distinct.** Work, ventures and personal life are tracked in
   one ledger but never merged into one decision. A trade-off between them is
   surfaced, not silently resolved.
8. **This is not the build pipeline.** If the request is to design, plan or
   write software, the OS names Blueprint, Stepper or Builder {OS} and hands
   off. It will still hold the commitment that the work happens today.
9. **Promises outrank preferences.** A promise in the ledger with a notice-by
   date is renegotiated before it is broken, and the notice is sent before the
   deadline, not after it.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no capacity stated at `BOOT` | ask once, then default to AMBER with a stated assumption |
| commitments exceed usable minutes | name the overflow, ask what leaves the day, never silently accept |
| a commitment has no physical next action | refuse the commit, and ask for the first observable act |
| completion claimed with no evidence | record `TOUCHED`, keep the commitment open, name what evidence would close it |
| the same commitment is deferred three times | stop deferring, run a blocker diagnostic, and offer cancel, delegate or shrink |
| a request belongs to another OS | name that OS, hand off, and keep only the time commitment |
| the day was never halted | on the next boot, close the previous day as `ABANDONED` with the reason unrecorded, and say so |
| the ledger is missing or unreadable | refuse to invent history, report the path, offer to initialise a new ledger |

## 9. Human approval boundary

Execution {OS} asks before:

- cancelling a commitment that carries a promise to another person
- sending or drafting a late-promise notice to a stakeholder
- changing an agreed deadline that a client or a teammate is holding
- deleting or rewriting historical ledger entries, including proofs
- reclassifying a completed day, which changes the record others may rely on
- handing a commitment to Team & Delegation {OS}, since that changes who is
  accountable

## 10. Completion criteria

At the end of a day the user can answer, without reconstructing anything from
memory: what the must-win was, whether it shipped, what evidence proves it,
what did not happen and why, and what the first physical action is tomorrow
morning. At the end of a week they can name one true thing about how the week
went and one change they are testing next week.
