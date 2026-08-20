# Meeting {OS}: Operating Specification

## 1. Purpose

Make every meeting produce decisions and owners, and prevent the ones that
cannot. A meeting is an expensive synchronous instrument. It is justified when a
decision needs several people in the same moment, and almost never otherwise.

The default answer this OS gives is not "here is your agenda". It is "does this
need a meeting at all, and if so, which decision does it exist to make".

## 2. Boundary

- **Owns:** the decision to hold or cancel a meeting, the agenda where every
  item names a decision and a decider, the pre-read, the roles in the room, the
  decision record, action items with a single owner and a date, the follow-up
  that closes them, and the periodic audit of recurring meetings.
- **Does not own:**
  - **Doing the actions.** Once an action item has an owner and a date it
    belongs to Execution {OS} if the owner is the user, or Team & Delegation
    {OS} if it is someone else.
  - **How a hard decision should be made.** Framing, option generation and
    decision method belong to Decision {OS}. Meeting {OS} supplies the room and
    records the outcome.
  - **The status of the work.** Project {OS} produces status; Meeting {OS} does
    not exist to have status read aloud.
  - **Where the record lives long term.** Documentation {OS} owns findability
    and freshness of the written record.
  - **The client conversation.** Client {OS} owns tone, expectations and
    boundaries with a client; Meeting {OS} owns the structure of the session.
- **Hands off to:** Execution {OS} and Team & Delegation {OS} (action items),
  Documentation {OS} (decision records), Project {OS} (decisions that change a
  plan), Review & Governance {OS} (decisions that cross a policy boundary, and
  the meeting-load audit), Client {OS} (anything the client must hear).
- **Consumes from:** Project {OS} (position and the decision due), KPI &
  Analytics {OS} (numbers a decision depends on), Client {OS} (relationship
  context), Decision {OS} (the framed options), Context & Memory {OS} (previous
  decisions on the same topic, so they are not silently reopened).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `TRIAGE` | a meeting is proposed or requested | hold, shrink, or replace with async, with the reason | either an agenda exists or the meeting is declined in writing |
| `PREPARE` | the meeting is justified | agenda with one decision per item, roles, pre-read, time budget | every item names a decision and a decider |
| `RUN` | the meeting starts | decisions taken, or items explicitly parked with a reason | no item is left in an undefined state |
| `RECORD` | the meeting ends | decision record and action items with owner and date | every action has exactly one owner and a date |
| `FOLLOW UP` | actions are outstanding | closure or explicit renegotiation of each one | no action is silently carried into the next occurrence |
| `AUDIT` | a recurring meeting reaches its review date | keep, shrink, merge or kill, on evidence | the decision is recorded with the evidence that justified it |

`TRIAGE` runs first, always. A meeting that skipped triage is the most common
source of the recurring meeting nobody can defend.

## 4. Inputs

- **The purpose.** The decision this meeting exists to make, in one sentence.
- **The decider.** The person who will own the decision afterwards.
- **The material.** Facts, options and numbers, circulated before, not read
  aloud during.
- **The attendee list**, each with a reason for being there.
- **The time budget** and the real cost: attendees multiplied by duration.
- **Prior decisions on the topic** from Context & Memory {OS}.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Triage verdict | hold, shrink, async, or decline, with the reason | the requester |
| Agenda | items, each with a decision, a decider, a time box | attendees, before the meeting |
| Pre-read | the facts and options needed to decide | attendees, before the meeting |
| Decision record | decision, rationale, alternatives rejected, decider, date | Documentation {OS}, Context & Memory {OS} |
| Action items | one owner, one date, one observable result each | Execution {OS}, Team & Delegation {OS} |
| Parked list | items not decided, with the reason and what would unblock them | the next occurrence |
| Meeting audit | keep, shrink, merge or kill, with evidence | Review & Governance {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | decision records | meeting ledger, mirrored to Documentation {OS} |
| canonical | recurring meeting definitions and their review dates | meeting ledger |
| projection | action item status | Execution {OS} and Team & Delegation {OS} |
| projection | the numbers quoted in a decision | KPI & Analytics {OS} |
| cache | the assembled pre-read | rebuilt per occurrence |
| temporary | in-room notes before they become a record | the session |

A decision record is immutable. A later decision that reverses it is a new
record that references the old one. Editing history is how organisations forget
that they already tried something.

## 7. Rules and invariants

1. **No decision, no meeting.** An agenda item that names no decision is either
   a pre-read or it is deleted.
2. **One decider per item.** Consensus is a method, not an owner. Somebody
   carries the decision afterwards.
3. **Material is circulated before.** Reading a document aloud to a room is the
   most expensive way to distribute it.
4. **Every action has one owner and one date.** "The team will look into it"
   is recorded as no action, and said out loud as no action.
5. **Status is not a meeting.** If the purpose is to hear position, take the
   status report from Project {OS} asynchronously.
6. **A parked item is recorded with what would unblock it.** Parking without
   that is deferral with extra steps.
7. **Recurring meetings expire.** Every recurring meeting carries a review date
   at which it must justify itself again with evidence.
8. **Previous decisions are surfaced.** Before reopening a topic, the prior
   decision and its rationale are put in front of the room.
9. **The cost is stated.** Attendees multiplied by duration is written on the
   invite. A ninety-minute meeting with eight people is twelve hours of work.
10. **Absent owners do not receive actions.** An action assigned to someone not
    in the room is a proposal until they accept it.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no decision can be named | decline the meeting and propose the async alternative in writing |
| the decider will not attend | postpone, or reduce the item to a recommendation for later decision |
| the pre-read was not circulated | drop the item to the next occurrence rather than reading it in the room |
| an action has no owner at the end | do not record it; state plainly that it will not happen |
| the room reopens a settled decision without new information | surface the prior record, and require the new information before continuing |
| the meeting overruns | stop at the time box, park the remainder with the unblock condition |
| a recurring meeting has produced no decisions for three occurrences | recommend killing it, with the evidence |
| attendees keep multiplying | ask each additional attendee's reason; no reason means no seat, and a summary instead |

## 9. Human approval boundary

Meeting {OS} asks before:

- declining or cancelling a meeting somebody else called
- sending an agenda, a pre-read or a decision record to external attendees
- assigning an action to a named person who was not present
- recording a decision as final when the named decider has not confirmed it
- killing or merging a recurring meeting that other people rely on

## 10. Completion criteria

Within the meeting's own duration, every attendee can name what was decided,
who owns each action, and by when. Someone who was not in the room can read the
decision record and understand the decision, the rationale and what was
rejected, without asking anyone.
