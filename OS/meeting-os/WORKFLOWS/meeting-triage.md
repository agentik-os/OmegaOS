# Workflow: meeting triage

Produces a verdict on whether a proposed meeting should exist at all, and the
asynchronous alternative when it should not.

## Trigger

A meeting is proposed, an invite arrives, or a recurring occurrence is about to
be scheduled again.

## Inputs

- The stated purpose, in the requester's own words.
- The proposed attendee list and duration.
- Whether a decision is needed, and who would own it.
- Prior decisions on the same topic from Context & Memory {OS}.

## Steps

1. **Ask for the decision.** One sentence: what will be decided that cannot be
   decided any other way.
2. **Compute the cost.** Attendees multiplied by duration, in person-hours,
   stated plainly. This single number changes more meeting behaviour than any
   argument.
3. **Test for the async alternative.** If the purpose is to inform, to collect
   input, to review a document, or to hear status, the alternative is a written
   artifact plus a comment window. Say which artifact.
4. **Test for the decider.** If the person who will own the decision cannot
   attend, the meeting cannot make it. Move it or reduce it.
5. **Test for prior art.** If this topic was decided before, surface the prior
   decision. Reopening requires new information, and the new information is
   named before the room is booked.
6. **Choose the verdict.**
   - Hold: a decision, a decider present, and material that will be circulated.
   - Shrink: the decision is real but half the attendees are spectators. Cut the
     list and send them the record.
   - Async: no synchronous decision is needed. Name the artifact and the date.
   - Decline: no decision exists, and nothing will be produced.
7. **Write the verdict to the requester.** A decline names what they will get
   instead and by when. A decline with no alternative reads as obstruction and
   will be overruled.
8. **If held, hand to the decision-meeting workflow.**

## Completion test

- The decision the meeting exists to make is written down, or the meeting is
  declined.
- The person-hour cost is stated.
- The decider is confirmed present, or the meeting is moved.
- A declined meeting carries a named alternative artifact with a date.
- Any prior decision on the topic has been surfaced to the requester.

## Failure paths

| Situation | Response |
|---|---|
| the requester outranks the user and insists | hold the meeting, but still write the agenda with a decision per item, and record what it produced for the next audit |
| the purpose is genuinely relational, such as a first client call | that is a valid purpose; record it as relational, hand the session shape to Client {OS}, and do not force a decision item |
| the purpose is "alignment" | ask what would be different afterwards; if nothing observable, decline with a written summary instead |
| the meeting is recurring and nobody remembers why | run the recurring audit workflow before scheduling the next one |
