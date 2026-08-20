# Social Intelligence {OS}: Weekly relational debrief

**Produces:** a debrief record for each significant interaction of the week,
each naming what the prior read got wrong, sent to Journal {OS} on approval.
**Trigger:** a fixed weekly slot the user chooses, or `/debrief --week`.
**Runs in:** `DEBRIEF`.
**Takes:** the reads and preparation briefs produced this week, the user's
account of what actually happened, and any entries already written about those
interactions in Journal {OS}.

## Steps

1. List the interactions this week that produced a read or a brief. Add any the
   user names that had neither. Cap the list at the five that mattered; a
   debrief of everything is a debrief of nothing.
2. For each one, get what actually happened, in sequence, in the user's words.
   Ask for the actual sentences where they are recoverable.
3. Put expected next to actual. For a prepared conversation, compare against the
   brief's aim and counter-aim. For an unprepared one, compare against whatever
   the user expected walking in.
4. Name what the prior read got wrong or left unverified. This step is the point
   of the workflow. A debrief that finds the read was entirely correct is
   treated as unverified, not as confirmation, and it says so.
5. Separate again: what was observed this time, what was stated, what is still
   inference. Do not let the outcome retro-fit the earlier inference into a fact.
6. Record one thing to check earlier next time: a specific observation, not a
   resolution to be more perceptive.
7. Flag anything that has stopped being an interaction. A pattern across
   several weeks with the same person is a Journal {OS} candidate pattern. A
   question about whether to keep the relationship is a Decision {OS} call. A
   standing fact about the person belongs to Network {OS}, and only with
   explicit approval.
8. Ask for approval per record before anything is sent or persisted. Nothing
   about a named third party is stored or transmitted without it.
9. Discard the superseded reads. They are not kept as evidence about anybody.

## Completion test

Every interaction on the list has a debrief record that names at least one
thing the prior read got wrong or left unverified, and one specific observation
to check earlier next time. Every record that was sent has an explicit approval
against it, and no superseded read remains in state.

## Failure

- The user cannot recall the sequence: record what is recoverable, mark the
  rest unknown, and do not reconstruct the conversation from the outcome.
- Outcome and read disagree with no clear cause: record the disagreement as
  open, name what evidence would settle it, and leave it open.
- The user declines approval on a record: keep it in the session only, persist
  nothing, and say plainly that it will not survive the session.
- An interaction turns out to involve abuse, coercive control or violence: stop
  the debrief, route to a qualified professional or emergency service, and do
  not produce a strategy for the next encounter.
