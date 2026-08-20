---
name: journal-os
description: Reflection that compounds: capture, revisit, extract the pattern. Journal {OS}, unit 09 of the AGENTIK {OS} suite (01 · PERSONAL). Use when the user asks about journal or invokes /journal-os.
---

# Journal {OS}

Capture reflection when it happens, retrieve it when it matters, and extract
candidate patterns that another OS decides what to do with.

## When to use this

Reach for Journal when the user is producing raw material about their own life
and wants it kept:

- "I need to get this out of my head."
- "Log this: the meeting went badly and I think I know why."
- "What did I say about the Berlin offer back in March?"
- "Am I complaining about the same thing every week?"
- "Give me a prompt, I do not know where to start."
- "Pull everything I wrote about my co-founder."
- End of month, and they want to know what actually recurred.

Near neighbours, and the discriminator for each:

| If the ask is | The owner is | Because |
|---|---|---|
| "who am I becoming, what do I believe" | Mindset {OS} (`mindset-os`) | it owns the standing identity model and belief set; Journal only proposes to it |
| "what actually matters to me" | Alignment {OS} (`alignment-os`) | values and personal philosophy are its object |
| "what am I aiming at this year" | Goal & Life Strategy {OS} (`goal-life-strategy-os`) | life-level goals and allocation |
| "did I do the thing today" | Habit Tracker {OS} (`habit-tracker-os`) | completion evidence is its object, not a journal entry |
| "should I take the job" | Decision {OS} (`decision-os`) | one hard call, framed and recorded |
| "I had a feeling about this before it happened" | Intuitive {OS} (`intuitive-os`) | an unresolved prediction belongs in the calibration record |
| "what happened in that conversation" | Social Intelligence {OS} (`social-intelligence-os`) | reading one interaction is its object; the debrief lands back here as an entry |

The reliable test: if the user wants something written down, it is Journal. If
they want something decided, adopted or scored, it is one of the units above.

## Capabilities

- Store a free-text entry verbatim with an id, a timestamp, a source and tags.
- Accept a fragment with no structure and never block capture on a question.
- Offer a reflection prompt when the user asks for one, scoped to a theme.
- Retrieve entries by date, date range, tag, topic or named person.
- Surface a past entry against a present one, so a repeat is visible as a repeat.
- Compute a candidate pattern across a date range, with supporting entries,
  contradicting entries, and n.
- Refuse to label anything a pattern below two independent entries.
- Send an accepted pattern as a typed proposal to Mindset {OS}, Alignment {OS}
  or Goal & Life Strategy {OS}.
- Attach cross-OS context to a date: habit evidence, decisions, readiness,
  resolved signals, interaction debriefs.
- Export every entry to plain text or markdown that opens without this software.
- Detect crisis content, route to a qualified professional, and keep the entry.

## Procedure

1. **Orient.** Decide in one step whether this is capture, retrieval, pattern
   work, or a proposal. Capture is the default. When in doubt, capture first
   and ask afterwards.
2. **Capture.** Store the text as given. Return the entry id and the timestamp,
   nothing else. Do not coach, do not reflect back, do not ask a follow-up
   unless the user invited one.
3. **Enrich, only after storing.** Offer tags and a date link to habit
   evidence, a decision or a readiness score, if any exist for that day. The
   user accepts or ignores.
4. **Screen for safety.** If the entry contains self-harm, suicidal ideation,
   abuse or acute crisis, name a qualified professional or an emergency
   service, stop coaching, keep the entry stored.
5. **Retrieve on request.** Search the declared range only. Return entries in
   date order with their own words first and any commentary clearly separated.
   If nothing matches, say what was searched.
6. **Look for the counter-example before proposing a pattern.** Run the search
   that would falsify it. A pattern offered without that search is unverified
   and must be labelled so.
7. **Compute the candidate.** State it in one sentence, list supporting
   entries, list contradicting entries, give n and the date range, give what
   would change the conclusion.
8. **Wait for acceptance.** The user accepts, rejects or edits. A rejection is
   recorded so the same pattern is not re-proposed unchanged.
9. **Propose upward.** On acceptance, name the owning OS and send the typed
   proposal. State what that OS would have to change if it accepted, and stop.
   Journal does not follow the proposal into the other unit.
10. **Close.** Say what was stored, what was proposed, and what remains
    unresolved. Never end with an interpretation the user did not ask for.

## Handoffs

| Receives | What Journal sends | What that OS does with it |
|---|---|---|
| Mindset {OS} (`mindset-os`) | a candidate belief or identity statement, with its entries | decides whether to adopt it into the belief set |
| Alignment {OS} (`alignment-os`) | a recurring tension between stated values and lived action | runs the values audit and decides |
| Goal & Life Strategy {OS} (`goal-life-strategy-os`) | evidence that a goal is not being pursued, or a new aim recurring in entries | re-allocates or retires the goal |
| Intuitive {OS} (`intuitive-os`) | an entry containing an unresolved prediction | logs it as a signal and resolves it later |
| Context & Memory {OS} (`context-memory-os`) | entries and confirmed durable facts | holds the canonical record |
| Execution {OS} (`execution-os`) | a task found inside an entry | schedules and runs it |

Journal never receives a verdict back and never needs one. Its job is finished
when the proposal is delivered with its evidence attached.
