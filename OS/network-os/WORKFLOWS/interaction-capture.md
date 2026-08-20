# Interaction capture

Turn something that just happened into accurate memory and a closed set of
commitments, before either degrades.

## Trigger

A conversation, meeting, call or message exchange ends. Also fires when Meeting
{OS} publishes a meeting record that names a person tracked here.

## Steps

1. **User or Meeting {OS}** supplies the raw account: who, when, where, and what
   was said. Produces: the raw interaction.
2. **Network {OS}** separates the account into fact, user statement, inference,
   assumption and unknown, labelling each. Produces: a classified account.
3. **Network {OS}** extracts every commitment, in both directions, with the
   exact words used. Produces: a commitment list.
4. **Network {OS}** assigns an owner and a date to each commitment. A commitment
   with no date is presented back to the user as ambiguous rather than assigned
   an invented one. Produces: owned commitments.
5. **Network {OS}** checks each new fact against the existing person record. A
   contradiction is surfaced with both provenances, never silently overwritten.
   Produces: a merge proposal or a contradiction report.
6. **Human** confirms anything sensitive and any inference being promoted to a
   fact. Produces: approval, per item.
7. **Network {OS}** writes the interaction record and updates the person record
   in Context & Memory {OS}, each entry carrying source and timestamp.
   Produces: canonical memory.
8. **Network {OS}** hands the user's own commitments to Execution {OS} as tasks
   with an owner and a date, stripped of unnecessary personal detail. Produces:
   follow through commitments.
9. **Network {OS}** opens a follow up loop for each commitment the other party
   made, with the date at which its absence becomes worth mentioning.
   Produces: open loops.

## Completion test

Every commitment spoken in the interaction appears exactly once in the record,
with an owner and a date, and every commitment owned by the user exists as a
task in Execution {OS}. A commitment present in the raw account and absent from
the record is a failed capture, even if the record reads well.

## Failure and abort

- The raw account is too thin to classify: capture what exists, mark the record
  partial, and list the questions that would complete it. Do not invent context.
- A new fact contradicts a user-supplied fact: halt the write for that field,
  present both with timestamps, and change nothing until the user answers.
- A sensitive fact is present and approval is not given: record the interaction
  without it. The interaction still gets captured; the detail does not.
- The person cannot be resolved to a single record: halt, present both
  candidates, and refuse to merge. A wrong merge is not cheaply reversible.
