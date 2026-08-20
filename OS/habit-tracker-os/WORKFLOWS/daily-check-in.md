# Habit Tracker {OS}: Daily check-in

**Produces:** one dated, provenance-labelled log record per habit that was due today, and the next action.
**Trigger:** the user reports on the day ("done", "half of it", "missed", "skipped, was travelling"), or the evening cadence they set at setup fires.
**Runs in:** `CHECK_IN`, entering `URGE`, `LAPSE` or `RECOVER` when the report calls for it.
**Takes:** today's due contracts from the ledger, the user's own statement, any trusted device import for the same day, the current season, and the capacity envelope from Health & Energy {OS}.

## Steps

1. Load today's due contracts and their statuses. A `PAUSED`, `RETIRED` or
   `ARCHIVED` contract is not due and is not asked about.
2. Read the user's statement and split it: what is explicitly stated, what a
   trusted device reported, what the model is inferring, what is unknown.
3. Write one record per due contract. Label each `explicit` or `observed`. An
   inference stays `inferred` and does not count as completion.
4. Ask at most one clarifying question, and only when the answer changes what
   gets recorded or what happens next. Otherwise record the explicit part and
   mark the rest unknown.
5. If the report is a temptation not yet acted on, leave `CHECK_IN` for `URGE`
   and run the urge protocol before any analysis.
6. If the report is a miss, leave `CHECK_IN` for `LAPSE`: record the antecedent
   (time, place, state, who was present) and protect the next occurrence. Do
   not score the day.
7. If the report signals overload across several contracts, or Health & Energy
   {OS} has reported capacity below the current load, propose `RECOVER` rather
   than a motivational fix.
8. Stage confirmed observations through Context & Memory {OS} and update the
   local ledger projection.
9. Close: state what was recorded, the one next action attached to a cue, and
   when the loop resumes.

## Completion test

Every contract due today has either a written record with a provenance label,
or an explicit "unknown" with the reason. No record labelled `inferred` or
`proposed` is counted as a completion.

## Failure

No due contracts: say so and offer `SETUP` or `TODAY`, do not invent a habit to
check in on. Ambiguous statement: record the explicit part, mark the remainder
unknown, and name the ambiguity in the close. Missing ledger or unreachable
Context & Memory {OS}: capture the report verbatim with its timestamp, tell the
user it is not yet persisted, and retry on the next session rather than
discarding it. Any safety signal: stop the check-in, surface the safety
boundary, and route to a qualified human professional.
