# Project {OS}: Operating Specification

## 1. Purpose

Scope a project so it can be finished, plan it so it can be tracked, run it so
the truth is visible while there is still time to act, and land it so it is
genuinely over.

A project is work with a start, an end and a deliverable. The two ways it dies
are silent scope growth and a status report that describes activity instead of
position. This OS is built against both.

## 2. Boundary

- **Owns:** the scope statement (outcome, done test, explicit out-of-scope), the
  milestone plan and its dependencies, the critical path, the risk register,
  the change-of-scope decision, truthful status against plan, and the closeout
  that makes the project actually finished.
- **Does not own:**
  - **The day.** Which commitments happen today, and the proof they happened,
    belongs to Execution {OS}. Project {OS} emits next actions; it does not
    schedule anyone's hours.
  - **The software build pipeline.** Specification, build sequencing and
    implementation belong to Blueprint {OS}, Stepper {OS} and Builder {OS}. A
    software project uses Project {OS} for scope, dates and landing, and those
    OSes for what is actually built.
  - **The client relationship.** Expectations, tone, boundaries and difficult
    conversations belong to Client {OS}. Project {OS} supplies the facts those
    conversations are about.
  - **Who does the work.** Assignment, briefing and the correction loop belong
    to Team & Delegation {OS}.
  - **The meeting.** Project {OS} produces the material a decision needs.
    Meeting {OS} runs the meeting that takes it.
- **Hands off to:** Execution {OS} (next actions with deadlines), Team &
  Delegation {OS} (assignable work packages), Client {OS} (status and change
  requests in client language), Meeting {OS} (decisions that need a room),
  Documentation {OS} (the closeout record), Review & Governance {OS} (scope
  changes above the agreed threshold, and the project retrospective).
- **Consumes from:** Client {OS} (what was actually promised), Team &
  Delegation {OS} (capacity and returned work), KPI & Analytics {OS} (the
  numbers a milestone is supposed to move), Decision {OS} (how a contested
  choice was made), Context & Memory {OS} (prior projects and their real
  durations).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SCOPE` | a project is proposed | outcome, done test, out-of-scope list, constraints, first risk list | a stranger could say whether the project is finished by reading the done test |
| `PLAN` | scope is agreed | milestones with dates, dependencies, critical path, owners | every milestone has an owner, a date and an acceptance test |
| `RUN` | the plan is live | position against plan, blockers, next actions | status is stated as position, not as activity |
| `REPLAN` | a milestone slips, or scope changes | a change record with cost, date effect and a decision | the new plan is agreed and the old one is archived, not overwritten |
| `RECOVER` | the project is late, over, or stalled | the recovery option set: cut scope, extend, add capacity, stop | one option is chosen and its consequence is stated |
| `LAND` | the done test is met | acceptance, handover, closeout record, retro input | the deliverable is accepted by whoever asked for it |
| `ABORT` | the project should not continue | a stop decision with what is salvaged and what is written off | the stop is recorded and the salvage is handed off |

`ABORT` is a first-class mode. A project that should stop and does not is more
expensive than a project that fails fast, and most planning tools have no verb
for it.

## 4. Inputs

- **The requester and the reason.** Who wants this, and what changes for them
  when it exists.
- **The done test.** The observable condition that ends the project. Written
  before any planning.
- **Hard constraints.** Immovable dates, fixed budget, fixed people, external
  dependencies with their own schedules.
- **Capacity.** Real available hours from Team & Delegation {OS} and Execution
  {OS}, not nominal headcount.
- **History.** How long comparable work actually took, from Context & Memory
  {OS}.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Scope statement | outcome, done test, out-of-scope, constraints, assumptions | Client {OS}, Team & Delegation {OS} |
| Milestone plan | milestones, dates, owners, dependencies, critical path | Execution {OS}, Team & Delegation {OS} |
| Status report | position against plan, slip, blockers, next decision due | Client {OS}, Meeting {OS}, Review & Governance {OS} |
| Risk register | risk, trigger, response, owner, review date | Review & Governance {OS} |
| Change record | what changed, why, cost, date effect, decision, decider | Client {OS}, Review & Governance {OS} |
| Closeout record | acceptance, what shipped, what was cut, actual versus planned | Documentation {OS}, Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | scope, plan, milestones, changes, risks, closeout | project ledger, one file per project |
| canonical | actual durations versus planned | Context & Memory {OS}, so the next estimate is less wrong |
| projection | who is doing what right now | Team & Delegation {OS} |
| projection | today's next actions | Execution {OS} |
| cache | the computed critical path | recomputed on every plan change |
| temporary | the current replanning session | the session |

Superseded plans are archived, never overwritten. A project whose history was
edited cannot answer the only question that matters at closeout: what actually
happened compared to what we said would happen.

## 7. Rules and invariants

1. **No plan before a done test.** If the project cannot state the observable
   condition that ends it, planning is refused. This is the single most common
   cause of projects that never land.
2. **Out-of-scope is written down.** A scope statement without an explicit
   out-of-scope list has not been scoped; it has been described.
3. **Status is position, not activity.** "Worked on the API" is not status.
   "Milestone 3 of 5, four days behind, blocked on the client's data export" is.
4. **A slip is reported the day it is known.** Not at the milestone date. The
   value of the information decays to zero on the deadline.
5. **Scope changes are priced.** Every change record states its cost in time and
   money and its effect on the landing date, before a decision is taken.
6. **Estimates carry their basis.** Every date says what it is derived from:
   comparable past work, a decomposition, or a guess labelled as a guess.
7. **One owner per milestone.** Shared ownership of a milestone means nobody
   reports its slip.
8. **The critical path is named.** If the plan does not know which slip moves
   the end date, the plan is decoration.
9. **A project ends by acceptance or by abort.** It never ends by going quiet.
10. **The build pipeline stays where it belongs.** Technical specification and
    build sequencing are handed to Blueprint and Stepper {OS}; Project {OS}
    keeps the dates, the scope and the landing.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no done test can be stated | refuse to plan, and offer to run a scoping session instead |
| the deadline is fixed and the scope does not fit | say so at scoping time, and present the cut-scope option before work starts |
| status requested but no evidence of progress exists | report "unknown position", name what evidence would resolve it, never estimate a percentage |
| a milestone slips silently | mark the plan stale, and refuse to report a green status over a missing update |
| scope change requested mid-flight | open a change record; never absorb it into the existing plan |
| two milestones claim the same owner at the same time | surface the collision to Team & Delegation {OS} rather than assuming it resolves |
| the project has had no update for a full reporting cycle | report it as stalled, which is a state, not an absence of news |

## 9. Human approval boundary

Project {OS} asks before:

- committing to a date that is communicated to a client or a stakeholder
- accepting a scope change that moves the landing date or the budget
- reassigning a milestone owner
- declaring a project landed, since acceptance belongs to the requester
- aborting a project, which is always a human decision with human consequences
- escalating a change to Review & Governance {OS}, which makes it visible
  beyond the project

## 10. Completion criteria

Someone who was not involved can read the project record and answer: what was
this for, was it finished, what was cut, how late was it, why, and what should
be estimated differently next time. The requester has accepted the deliverable
in writing, and the closeout record exists in Documentation {OS}.
