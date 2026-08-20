# Journal {OS}: Daily entry

**Produces:** one stored entry for today, with its cross-OS context attached.
**Trigger:** the user opens `/journal`, sends text to `/entry`, or the day ends
with no entry and the user has asked for an end-of-day nudge.
**Runs in:** `CAPTURE`, then `PROMPTED` only if the user asks for a prompt.
**Takes:** free text from the user. Optionally, today's habit evidence from
Habit Tracker {OS}, today's readiness from Health & Energy {OS}, any decision
recorded today by Decision {OS}, any interaction debrief from Social
Intelligence {OS}.

## Steps

1. Open the capture surface. Print nothing but the prompt character. Do not ask
   how the day went, do not summarise yesterday, do not offer a theme unless
   the user asks for one.
2. Take the text exactly as typed. Store it with an id, a timestamp and the
   source. Echo back the id and the timestamp only.
3. Screen the stored text for crisis content: self-harm, suicidal ideation,
   abuse, acute medical or psychiatric emergency. If present, name a qualified
   professional or an emergency service, stop every other step, and keep the
   entry stored.
4. If the user asked for a prompt and gave a theme, offer exactly one question
   scoped to that theme. Take the answer as a second entry, not as an edit of
   the first.
5. Offer the cross-OS context that exists for today, as a list the user can
   accept or ignore: habits completed, readiness score, decision recorded,
   interaction debriefed. Attaching it links the entry to that record; it never
   copies the record into the entry text.
6. Offer tags only if the entry contains a name, a project or a topic already
   used in the store. Never invent a new taxonomy on the user's behalf.
7. If the entry contains a prediction about an outcome that has not happened
   yet, say so in one line and offer to send it to Intuitive {OS} as a signal.
   The user accepts or ignores.
8. If the entry contains a task, say so in one line and offer to send it to
   Execution {OS}. Journal does not track it.
9. Close. State the entry id and anything offered but not accepted.

## Completion test

An entry exists in the store with today's date, its text is byte-identical to
what the user typed, and its id was returned to the user. If crisis content was
present, the referral was given and no coaching followed it.

## Failure

- Store unreachable: refuse to report the entry as stored, hold the text in the
  session, and say plainly that it is not persisted and will be lost.
- Empty or unintelligible text: store it, mark it a fragment, do not guess.
- Cross-OS context unavailable: attach nothing, say which unit did not answer.
  Never fabricate a readiness score or a habit completion.
