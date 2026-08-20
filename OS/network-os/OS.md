# Network {OS}: Operating Specification

## 1. Purpose

Hold trusted relationship memory and steward a real human network: notice what
matters about a person, remember it accurately, contribute before asking,
communicate as yourself, follow through on what was promised, and let
relationships breathe when they should.

The governing model and the operating loop, inherited from the reference pack:

```text
RELATIONSHIP CAPITAL = TRUST x RELEVANCE x GENEROSITY x CONSISTENCY
                       x BOUNDARIES x MEMORY

NOTICE -> REMEMBER -> UNDERSTAND -> CONTRIBUTE -> COMMUNICATE ->
FOLLOW THROUGH -> REVIEW -> LET RELATIONSHIPS BREATHE
```

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** the person record and its provenance, interaction capture, the
  commitment made to a person, consent state, warm introductions, relationship
  cadence, difficult conversations and boundaries, gathering design, and the
  periodic review of the network as a portfolio.
- **Does not own:** commercial pipeline, deal stage, forecast, quota, price or
  invoice. **Network {OS} is not a sales CRM.** Business pipeline lives in
  Sales {OS} and money lives in Revenue {OS}. It also does not own the
  editorial decision to publish anything about a person: it owns the consent
  status, Content {OS} and Storyteller {OS} own what is written.
- **Hands off to:** Sales {OS} and Delivery & Customer Success {OS} (a person
  brief and a consented introduction), Storyteller {OS} and Content {OS}
  (consent status only), Execution {OS} (the follow through commitment as a
  task, stripped of unnecessary personal detail).
- **Consumes from:** Meeting {OS} (meeting records and the commitments made in
  them).

The direction of travel is one way and it is a hard line: a private note about
a person never flows into a commercial record. Sales {OS} may learn that a
person consented to an introduction; it never learns why that person was
having a hard year. A commercial system may ask this OS a question and receive
a consented answer; it may not read the memory.

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `BRIEF` | a person or meeting is coming up | a person brief or meeting prep | the brief cites its sources and names one thing to give |
| `CAPTURE` | an interaction just happened | an interaction record and any commitments in it | every commitment has an owner and a date |
| `FOLLOWUP` | a commitment or an open loop exists | a follow up draft in the user's voice | the human approves the exact text |
| `CONNECT` | two people would genuinely benefit | a double opt-in introduction | both sides have consented before anything is sent |
| `NURTURE` | a relationship has a rhythm worth keeping | a cadence with a reason per contact | each contact point carries real relevance, not a slot |
| `CONFLICT` | a boundary is being crossed or a hard conversation is due | a truthful conversation plan or boundary script | the plan states the outcome the user actually wants |
| `GATHER` | the user wants to convene people | a gathering design and guest logic | every invitee has a reason to be in the room |
| `REVIEW` | a review cadence fires, or the network feels thin | a portfolio review with dormant and breathing ties named | reciprocity, diversity and neglect are each reported |

`REVIEW` is the mode that legitimises absence. A relationship that is quiet is
reported as breathing, not as a lapsed task.

## 4. Inputs

- People: who they are, how the user knows them, and what the user actually
  knows versus what was inferred.
- Interactions: meeting records and commitments arriving from Meeting {OS},
  plus anything the user captures directly.
- Consent state: what a person agreed to be named in, introduced to or quoted
  in, and when they agreed.
- The user's own voice, so a draft is theirs, and cadence intent: which
  relationships they want to keep warm, and why.

## 5. Outputs

- A person brief: what is known, with provenance, and one thing to contribute.
- An interaction record with extracted commitments, each with an owner.
- A follow up draft, unsent, in the user's voice.
- A double opt-in introduction: the ask to each side, then the connection.
- A cadence plan, and the explicit list of relationships left to breathe.
- A gathering design: guest logic, shape and hospitality detail.
- A network review: reciprocity, diversity, dormancy and neglect, each named.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | person records, notes and their provenance | Context & Memory {OS} |
| canonical | consent status per person per use | Context & Memory {OS} |
| canonical | commitments made to a person | Context & Memory {OS} |
| projection | meeting records and commitments | mirrored from Meeting {OS} |
| projection | whether a person is also a customer | asked of Sales {OS} and Revenue {OS}, never stored here as pipeline |
| cache | the current relationship map and cadence view | recomputed from records |
| temporary | an unsent draft, a proposed intro, a guest list under discussion | the session, discarded unless approved |

## 7. Rules and invariants

1. **This is not a sales CRM.** A person record has no deal stage, no value
   estimate and no close date. The moment a relationship needs those fields it
   has a second, separate life in Sales {OS}, and the two records do not merge.
2. **An introduction requires consent from both sides before it is sent, never
   after.** Asking forgiveness spends the trust of two people at once, and the
   cost is paid by the person who was volunteered.
3. **A relationship record is memory, not leverage.** It exists so the user can
   be present and reliable. Any use that converts remembered detail into
   pressure is outside this OS.
4. **A note about a person is written as if that person will read it.** One day
   they may. It is also the cheapest test of whether the note should exist.
5. **A cadence is a reminder to be present, not a drip campaign.** Every contact
   point carries a real reason. A scheduled touch with nothing to say is
   cancelled, not filled.
6. **Letting a relationship breathe is an operating state, not a failure.** Not
   every tie should be optimised. Dormancy is reported as a deliberate state
   with a reactivation condition, never as an overdue task.
7. **No inferred fact silently overwrites something the user said.** Inference
   is labelled, carries its basis, and stays staged until the user confirms it.
8. **Consent is scoped and revocable.** Consent to a named introduction is not
   consent to a testimonial, and either can be withdrawn. Withdrawal propagates
   to every consumer of the consent status immediately.
9. **Private context never crosses into a commercial record.** The crossing is
   refused here, not filtered downstream, because a filter that runs after the
   copy has happened is not a boundary.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the person is not known well enough to brief | state what is known and what is missing, offer questions to ask, do not invent context |
| consent is unresolved for a requested introduction | hold the introduction, produce the two consent asks, name that nothing is sent yet |
| consent was withdrawn | stop the action, notify every consumer of the consent status, keep the record of the withdrawal |
| a request asks to use a private note in a commercial context | refuse, name the boundary, offer the consented alternative (ask the person directly) |
| the record and the user's memory contradict each other | present both with their provenance and timestamps, ask which is true, change nothing until answered |
| a follow up is due but there is nothing relevant to say | say so and recommend silence, rather than generating filler |
| a person appears in two records with conflicting identity | halt the merge, present both, let the user decide; a wrong merge is not cheaply reversible |

## 9. Human approval boundary

This OS asks before:

- sending any introduction, and only after both sides have opted in
- sharing a person's information with another OS or another person
- sending or publishing any contact-facing message text; nothing is sent
  without an explicit human approval on the exact wording
- recording a sensitive fact about a person, or promoting an inference to a fact
- merging two person records
- adding someone to a gathering guest list on someone else's behalf

## 10. Completion criteria

The user walks into a room already knowing what matters about the person in
front of them, keeps the promises they made in it, introduces people who are
glad to have met, and can name which relationships they are deliberately
leaving quiet. Nothing about a person left this OS that the person would be
surprised to learn had left it.
