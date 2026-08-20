# Social Intelligence {OS}: Interaction prep

**Produces:** a one-page preparation brief for one named upcoming conversation.
**Trigger:** the user runs `/prep`, or names a specific conversation that has
not happened yet and says they are unsure how to open it.
**Runs in:** `PREP`.
**Takes:** who, when, where, what the user wants out of it. Prior entries about
this person from Journal {OS}. The relevant values from Alignment {OS}. The
beliefs from Mindset {OS} that predict where this user misreads a room.

## Steps

1. Bound it: one conversation, one counterpart or one room, a date. If the user
   is describing a relationship rather than a conversation, hand off to Network
   {OS} and stop.
2. Screen the request. If the aim is to get the other person to act against
   their own interest, refuse now, name what was asked, offer the direct
   version, and stop. If the situation involves abuse, coercive control or
   violence, route to a qualified professional or emergency service and stop.
3. Make the user state the aim as an outcome, in one sentence. Not a feeling,
   not a topic: something that will be true afterwards. If they cannot, stop
   here and say the preparation is not ready. Do not draft a script around a
   vague aim.
4. Pull context: prior entries about this person, the values in play, the known
   blind spot. Name explicitly whichever of these was unavailable. Never fill
   the gap with an assumption about the counterpart.
5. Write the likely counter-aim: what the other person is probably trying to
   get out of the same conversation, stated as an outcome, marked as inference
   with a confidence and the evidence behind it.
6. Draft the opening line in the user's own register. Read it back and ask
   whether they can say it out loud. If not, redraft in their words.
7. Name the two or three things to listen for: the specific statements or
   behaviours that would confirm or kill the read of the counter-aim.
8. Set the walk-away line: what the user will not do, say or accept in this
   conversation, and what they do when it is reached.
9. Close the brief at one page. Anything longer will not be recalled in the room.

## Completion test

The user can state their aim in one sentence without reading it, knows the two
or three things to listen for, and can name the line they will not cross. The
brief fits on one page and the opening line is in words the user confirmed they
would actually say.

## Failure

- The aim cannot be stated as an outcome: stop, say the preparation is not
  ready, and offer to run `/read` on prior interactions instead.
- No prior context available: proceed with the brief but mark the counter-aim
  as an assumption with no evidence, and lower every confidence accordingly.
- The user cannot say the opening line: redraft twice, then drop the script and
  give them only the aim and the walk-away line. A script they cannot deliver is
  worse than none.
- The request turns out to be a hard call rather than a conversation: hand off
  to Decision {OS} and stop.
