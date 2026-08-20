# Workflow: the postmortem

Produces a blameless account of what happened and one change, with an owner and
a verification test, that would prevent a recurrence.

## Trigger

An incident, a material failure, a missed commitment with real consequences, or
a near miss that only luck prevented. Near misses are run too; they are the
cheapest evidence available.

## Inputs

- The timeline: what happened, in order, with times.
- The people involved, and their account of what they knew at each point.
- What the systems recorded: logs, tickets, messages, ledger entries.
- The controls that existed, and whether they fired.

## Steps

1. **Set the frame before anyone speaks.** This is about conditions, not
   character. Nobody's performance is being assessed here, and nothing recorded
   here is used that way.
2. **Build the timeline first,** from records rather than recollection. Order and
   timing dissolve most disagreements about cause on their own.
3. **Ask what each person knew at each moment,** not what they should have known.
   Hindsight makes every decision look obvious, and that is exactly the bias to
   exclude.
4. **Find the contributing conditions.** What made the mistake easy, what made
   it invisible, what made it slow to detect, and what made recovery hard. There
   is rarely one cause.
5. **Check the controls.** Did they exist, did they fire, were they bypassed,
   and if bypassed, was that reasonable at the time.
6. **Ask what would have caught this earlier**, and what the detection delay
   cost.
7. **Choose one change**, or a small number, that would prevent recurrence or
   would have caught it sooner. Prefer a change to a system over a change to
   people's care.
8. **Reject the non-changes.** "Be more careful", "communicate better" and "add
   a reminder" are not changes; they are hopes with a verb.
9. **Give the change an owner, a date and a verification test.**
10. **Route it.** Process defects to Operations {OS} and Process & SOP {OS},
    control changes through the change authorisation workflow, learning to
    Context & Memory {OS}, and the account itself to Documentation {OS}.
11. **Share the account** with everyone who could hit the same condition.

## Completion test

- The timeline is built from records and is agreed by the people involved.
- The account names contributing conditions, not individuals' qualities.
- Every control that existed is evaluated: fired, did not fire, or was bypassed.
- One change, or a small named set, has an owner, a date and a verification test.
- No item in the change list is a hope with a verb.
- The account has been shared with everyone exposed to the same condition.

## Failure paths

| Situation | Response |
|---|---|
| the discussion turns to who is at fault | return to the timeline and to the conditions; a blamed postmortem is the last honest one you will get |
| the cause is genuinely a single person's decision | describe the decision and what made it available; the change is to the conditions that permitted it |
| nobody wants to record it | record it anyway with the facts that are agreed, and note where accounts diverge |
| the only proposed change is more care | reject it and keep looking; if nothing systemic is found, say plainly that the risk is accepted, and record the acceptance |
