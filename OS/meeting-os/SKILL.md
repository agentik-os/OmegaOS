---
name: meeting-os
description: Meetings that produce decisions and owners, or do not happen. Meeting {OS}, unit 42 of the AGENTIK {OS} suite (05 · OPERATE). Use when the user asks about meeting or invokes /meeting-os.
---

# Meeting {OS}

Make a meeting produce decisions and owners, or prevent it from happening.

## When to use this

Use it before scheduling anything synchronous, when an agenda is being written,
when a meeting keeps happening and nobody can say what it produces, and when
last week's actions are still open at the start of this week's occurrence.

Typical openings: can we get everyone on a call, I need to run a kickoff, my
calendar is full of meetings, we discussed this last month and nothing moved.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| how to make a hard decision well | Decision {OS} |
| the position of the work | Project {OS} |
| doing the actions afterwards | Execution {OS}, Team & Delegation {OS} |
| keeping the written record findable and true | Documentation {OS} |
| the client conversation itself | Client {OS} |
| whether the whole meeting load is defensible | Review & Governance {OS}, with this OS supplying the audit |

## Capabilities

- Triage a proposed meeting into hold, shrink, replace with async, or decline,
  and write the decline so it does not read as obstruction.
- Build an agenda where every item names a decision, a decider and a time box.
- Assemble a pre-read containing the facts and options the decision needs.
- Compute and display the real cost of the meeting in person-hours.
- Run the room to decisions, and park what cannot be decided with the condition
  that would unblock it.
- Write a decision record with rationale and rejected alternatives.
- Produce action items that each have exactly one owner, one date and one
  observable result.
- Chase outstanding actions before the next occurrence rather than during it.
- Audit a recurring meeting on evidence and recommend keep, shrink, merge or
  kill.

## Procedure

1. Ask for the decision the meeting exists to make. If there is none, go to
   step 8.
2. Name the decider. If the decider cannot attend, propose a new time or reduce
   the item to a recommendation.
3. Check Context & Memory {OS} for a prior decision on the same topic. Surface
   it before the room reopens it.
4. Build the agenda: one decision per item, a time box per item, and the roles.
5. Assemble and circulate the pre-read. Nothing is read aloud in the room.
6. Run the meeting to the time boxes. Decide, or park with an unblock condition.
7. Record: decision, rationale, rejected alternatives, and action items with one
   owner and one date each.
8. If no decision exists, decline in writing and propose the asynchronous
   alternative that achieves the same result.
9. Route actions to Execution {OS} or Team & Delegation {OS}, and the decision
   record to Documentation {OS}.
10. Before the next occurrence, close or renegotiate every open action.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| Execution {OS} | actions owned by the user | one physical next action and a deadline |
| Team & Delegation {OS} | actions owned by someone else | outcome, definition of done, authority level |
| Documentation {OS} | decision records | the record, its topic, its owner and review date |
| Project {OS} | decisions that change a plan | the decision and its effect on scope or dates |
| Client {OS} | anything the client must hear | the facts, in the client's language |
| Review & Governance {OS} | boundary-crossing decisions and the meeting-load audit | the record and the change requested |
| Context & Memory {OS} | the decision and its rationale | confirmed, inspectable, referenced by any future reopening |
