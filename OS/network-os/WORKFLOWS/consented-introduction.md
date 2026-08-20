# Consented introduction

A double opt-in introduction: consent from both sides before anything is sent,
never after.

## Trigger

The user identifies two people who would genuinely both gain from knowing each
other, or one person asks to be introduced to another. The request alone does
not authorise the introduction; it starts this workflow.

## Steps

1. **User** names both people and states the mutual benefit in one sentence.
   Produces: the introduction intent.
2. **Network {OS}** checks that the benefit is genuinely mutual, not a favour
   extracted from one side. If only one side gains, it says so and stops here.
   Produces: a mutuality verdict.
3. **Network {OS}** reads the consent state of both people from Context &
   Memory {OS}. Produces: consent status per person, with timestamps.
4. **Network {OS}** drafts two separate consent asks, each written for its
   recipient, each stating who the other person is, why now, and how to decline
   without cost. Produces: two unsent asks.
5. **Human** approves the exact wording of each ask. Nothing leaves the OS
   before this step. Produces: approval on the text.
6. **User** sends both asks and records each reply. Produces: two explicit
   answers, yes, no or silence.
7. **Network {OS}** records each answer as scoped, timestamped, revocable
   consent. Silence is recorded as absence of consent, never as assent.
   Produces: the consent record.
8. **Network {OS}** drafts the introduction message only if both answers are
   yes. Produces: an unsent introduction.
9. **Human** approves the exact introduction text. Produces: approval.
10. **User** sends it. **Network {OS}** creates a follow through commitment to
    check in once, and hands it to Execution {OS}. Produces: the introduction
    event and one open loop.
11. **Network {OS}** emits the consented introduction to Sales {OS} or
    Delivery & Customer Success {OS} only when the introduction is commercial
    in nature, carrying the consent state and no private note.

## Completion test

Both consent records exist, each with a timestamp and an explicit yes, and the
introduction message was sent after both timestamps. If any introduction event
carries a send time earlier than either consent timestamp, this workflow failed,
regardless of how the introduction turned out.

## Failure and abort

- One side declines: stop. Record the decline, tell the other side that the
  introduction is not happening, and give no reason that was not offered.
- One side does not answer: stop. Absence of a reply is not consent. The
  workflow stays open with a single reminder, then closes unresolved.
- The mutuality check fails at step 2: abort before anyone is asked. Offer the
  alternative, which is the user helping directly instead of spending someone
  else's attention.
- Consent is withdrawn after the asks but before the send: abort immediately and
  propagate the withdrawal to every consumer of the consent status.
