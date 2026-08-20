# Team & Delegation {OS}: Operating Specification

## 1. Purpose

Hand work off so it comes back right the first time.

Delegation fails in two directions. Under-briefed work comes back wrong and gets
redone, which teaches the delegator to stop delegating. Over-supervised work
comes back on time and teaches nobody anything, which means it must be
supervised again next time. This OS is built to avoid both: a brief precise
enough to be checked, an authority level stated explicitly, and check-ins sized
to the risk rather than to anxiety.

## 2. Boundary

- **Owns:** the decision of what to delegate and what must not be, the choice of
  who receives it, the delegation brief (outcome, constraints, definition of
  done, do-not-touch, authority level, deadline), the check-in schedule, the
  acceptance of returned work against the brief, the correction loop, the
  deliberate raising and lowering of authority, and taking work back when that
  is the right answer.
- **Does not own:**
  - **The procedure itself.** How the task is performed step by step belongs to
    Process & SOP {OS}. A good SOP makes a brief shorter.
  - **Whether the work should exist.** Operations {OS} decides that; delegating
    waste only moves it.
  - **The plan and the dates.** Project {OS} owns the milestones a work package
    serves.
  - **The user's own commitments.** Execution {OS} owns work that stays with
    you. The instant work is delegated it leaves Execution {OS} and arrives
    here, and the return date becomes a promise.
  - **Employment, compensation and performance management.** Those are human
    decisions with legal weight and belong to the operator, with Review &
    Governance {OS} for policy.
  - **Orchestrating software agents.** Agent {OS} and Orchestration {OS} own
    machine delegation. The brief discipline is similar; the accountability is
    not, and they are never merged here.
- **Hands off to:** Execution {OS} (the returned work, and any commitment that
  comes back to the user), Project {OS} (work package status, and any slip),
  Process & SOP {OS} (when the same brief has been written three times, it
  should be a procedure), Review & Governance {OS} (repeated failures that are
  systemic, and any authority change that crosses a policy boundary),
  Documentation {OS} (briefs worth reusing).
- **Consumes from:** Project {OS} (assignable work packages), Process & SOP {OS}
  (procedures that shorten the brief), Operations {OS} (which work is worth
  moving at all), Execution {OS} (the delegator's real capacity), Context &
  Memory {OS} (what this person has done before and how it went).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SELECT` | there is more work than capacity, or work that should not be yours | the delegate list and the keep list, each with a reason | every kept item has a reason that is not habit |
| `MATCH` | an item is delegable | the person, and the authority level appropriate to their evidence | the authority level is stated, not implied |
| `BRIEF` | a person and a task are matched | outcome, constraints, definition of done, do-not-touch, deadline, check-ins, escalation | the receiver can restate the outcome and the definition of done in their own words |
| `SUPPORT` | work is in progress | check-ins at the agreed points, unblocking, no surprise inspections | each check-in either confirms course or corrects it |
| `RECEIVE` | work is returned | acceptance against the brief, or a specific correction | accepted, or returned with one clear list of what is missing |
| `ADJUST` | evidence has accumulated | authority raised or lowered, with the evidence | the change and its reason are stated to the person |
| `RECALL` | the work must come back | a clean handover in reverse, without blame | the work is with a new owner and the person knows why |

## 4. Inputs

- **The work package,** with its outcome and its deadline.
- **The receiver:** their evidence on similar work, their current load, and what
  they are trying to learn.
- **The definition of done,** ideally borrowed from an existing SOP or quality
  bar rather than reinvented.
- **The constraints:** budget, tools, people they may involve, what they may
  decide alone.
- **The consequence of it going wrong,** which sets the check-in frequency and
  the authority level.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Delegate and keep lists | items, with the reason for each side | the delegator |
| Delegation brief | outcome, constraints, definition of done, do-not-touch, authority level, deadline, check-ins, escalation | the receiver |
| Check-in record | on course, corrected, or blocked, with the action taken | the delegator and the receiver |
| Acceptance record | accepted against the brief, or the specific gap | Project {OS}, Execution {OS} |
| Feedback note | what to do differently, tied to the brief, not to taste | the receiver |
| Authority change | raised or lowered, with the evidence and the reason | the receiver, Review & Governance {OS} when it crosses policy |
| Recall record | why the work came back, where it went, what is owed | Project {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | briefs, acceptance records, authority levels per person and per work type | delegation ledger |
| canonical | the history of what each person has delivered against a brief | delegation ledger, and Context & Memory {OS} |
| projection | milestone dates the work serves | Project {OS} |
| projection | the procedure being followed | Process & SOP {OS} |
| cache | current load per person | recomputed each cycle |
| temporary | a draft brief | the session |

Authority is tracked per person and per work type, not per person alone.
Somebody trusted to decide alone on client communications may not be trusted to
decide alone on infrastructure, and collapsing the two loses the information.

## 7. Rules and invariants

1. **Delegate the outcome, not the keystrokes.** A brief that specifies every
   action has not delegated anything; it has queued the delegator's own work
   through someone else's hands.
2. **The definition of done is written before the work starts.** It is what
   acceptance is judged against, and inventing it afterwards is how returned
   work is rejected on taste.
3. **State the authority level explicitly.** Decide alone, decide and tell me,
   propose and wait, or ask first. Ambiguous authority produces either paralysis
   or a surprise.
4. **Write the do-not-touch list.** What must not change, what must not be
   contacted, what must not be spent. This list prevents almost all of the
   expensive surprises.
5. **Check-ins are sized to consequence.** High consequence and low evidence
   means frequent, agreed check-ins. Low consequence and strong evidence means
   one at the end. Never a surprise inspection, which teaches concealment.
6. **Accept or correct against the brief only.** If the returned work meets the
   definition of done, it is accepted even if the delegator would have done it
   differently.
7. **A missing piece is named once, completely.** Returning work three times with
   one new objection each time destroys the relationship and the deadline.
8. **Correction changes the next result.** Feedback that names an observable
   behaviour and a specific alternative. Feedback about a person's qualities is
   not correction.
9. **Authority moves on evidence, in both directions,** and the reason is said
   out loud.
10. **Three identical briefs mean a missing procedure.** Send it to Process & SOP
    {OS} instead of writing a fourth.
11. **Never delegate accountability you retain.** If the delegator still answers
    for the outcome, the brief says so, and the check-ins reflect it.
12. **Recall without blame.** Work comes back for capacity, priority or fit.
    Say which, and do not let a recall become a silent verdict on a person.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the receiver cannot restate the outcome | the brief is not finished; rewrite it rather than starting the work |
| no definition of done can be written | do not delegate yet; the task is not understood well enough by anyone |
| work comes back wrong | check the brief first; a defect in the brief is not a defect in the person |
| the deadline is missed with no warning | that is a check-in design failure; add an earlier check-in and say why |
| the receiver keeps asking for decisions | the authority level is too low, or the do-not-touch list is too wide; raise it deliberately |
| the delegator keeps taking work back | record it; a pattern of recall is a delegation problem, not a people problem |
| the work is late and the client is waiting | the promise belongs to the delegator; route to Client {OS} immediately and do not wait for the receiver |
| repeated failures across several people on the same work | systemic; route to Operations {OS} or Process & SOP {OS}, not to feedback |

## 9. Human approval boundary

Team & Delegation {OS} asks before:

- assigning work to a named person, since consent is part of delegation
- changing someone's authority level, up or down
- recalling work from someone
- writing anything that records an individual's performance
- committing someone else's time to an external deadline
- escalating a person's repeated failure beyond the immediate working
  relationship

## 10. Completion criteria

The receiver can restate the outcome and the definition of done in their own
words, knows exactly what they may decide alone, knows when they will be
checked and by whom, and delivers work that is accepted against the brief
without a second round of new objections. The delegator's own capacity actually
went down, which is the only real proof that anything was delegated.
