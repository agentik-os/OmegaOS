# Journal {OS}: Monthly pattern extract

**Produces:** a short list of candidate patterns for the month, each with its
supporting entries, its contradicting entries, n, and the OS that would own it.
**Trigger:** the first working day of a month, or `/pattern --since <date>` at
any time.
**Runs in:** `PATTERN`, then `PROPOSE` for whatever the user accepts.
**Takes:** every entry in the range. Optionally, Habit Tracker {OS} evidence,
Decision {OS} records, Health & Energy {OS} readiness and Intuitive {OS}
resolutions over the same range, used as corroboration only.

## Steps

1. Fix the range explicitly and state it. Do not widen it later without saying so.
2. Load the entries in the range. Report the count. If it is under ten, say so
   and lower every confidence claim accordingly.
3. Cluster by recurrence, not by sentiment. A cluster is a set of entries that
   name the same situation, the same person, the same constraint or the same
   avoidance. An emotional tone shared by unrelated entries is not a cluster.
4. For each cluster with at least two independent entries, write the candidate
   as one plain sentence with no interpretation of motive.
5. Run the falsifying search for each candidate before offering it: look for
   entries in the range that contradict it, and for periods inside the range
   where it does not hold. This step runs first, not after a challenge.
6. Present each candidate with: the sentence, supporting entry ids and dates,
   contradicting entry ids and dates, n, the date range, and what would change
   the conclusion. Anything with n equal to 1 is presented as an observation
   and is explicitly not a pattern.
7. Name the owning OS for each candidate: Mindset {OS} for a belief or an
   identity statement, Alignment {OS} for a values and action mismatch, Goal &
   Life Strategy {OS} for a goal that is not being pursued or a new aim that
   keeps recurring. Do not adopt any of them here.
8. Take the user's verdict per candidate: accept, reject or edit. Record every
   rejection against the candidate so the same one is not re-proposed unchanged
   next month.
9. Send each accepted candidate as a typed proposal to the named OS, with the
   entry ids attached. State what that OS would have to change if it accepted.
10. Close with what was proposed, to whom, and what was rejected.

## Completion test

Every candidate presented cites at least two entry ids and at least one
completed falsifying search, every accepted candidate has been delivered to a
named OS with its evidence, and every rejection is recorded. No statement about
the user is left standing without its entries.

## Failure

- Fewer than two entries in the range: report the count and stop. Do not
  produce candidates from one entry.
- A cluster where support and contradiction are balanced: present it as an open
  question with both sides, not as a pattern.
- The receiving OS is not installed: hold the proposal, name the missing unit,
  and tell the user what `agentik install` would fix it. Do not adopt the
  pattern locally as a substitute.
