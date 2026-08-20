# Alignment {OS}: Weekly values audit

**Produces:** a per-value verdict for the week (matched, drifted, or unmeasured), each with the evidence behind it, plus one governing principle for the coming week and any drift alerts emitted to neighbouring units.
**Trigger:** the user runs `/weekly`, or the week closes with a declared value set present and no audit recorded for that week.
**Runs in:** `VALUES_AUDIT`.
**Takes:** the declared value set and priority order (Context & Memory {OS}), the seven daily pass records for the week, habit evidence from Habit Tracker {OS}, entries and candidate patterns from Journal {OS}, and any decision outcomes from Decision {OS} in the period.

## Steps

1. Load the declared value set. Fix the audited window explicitly by dates.
2. Collect the evidence for the window: pass records, habit evidence, journal
   entries, decision outcomes. List what is present and what is missing before
   judging anything.
3. Run the weekly council across the ten domains: wins, failures, energy,
   habits, relationships, work, money, meaning, learning, and the stop, start,
   continue pass.
4. For each declared value, in priority order, cite the specific behaviour from
   the window that supports it and the specific behaviour that contradicts it.
5. Issue one verdict per value: matched, drifted, or unmeasured. Unmeasured is
   the required verdict when there is no observed behaviour either way.
6. For every drifted value, state whether the gap is a capacity problem, an
   allocation problem, or a value that is no longer actually held. Do not pick
   between the three when the evidence does not separate them; report the split.
7. Where a candidate pattern from Journal {OS} bears on a value, present it as a
   proposal with its confidence and ask the user to adopt or reject it.
8. Write one governing principle for the coming week, in one sentence.
9. Emit the handoffs: a contradiction report to Mindset {OS} when a value
   collides with a standing belief, a drift alert to Goal & Life Strategy {OS}
   when the allocation contradicts a value. Ask before either is sent as an
   adopted finding.
10. Ask before persisting the audit, then write it to Context & Memory {OS}.

## Completion test

Every declared value carries exactly one verdict with a cited behaviour or the
label "unmeasured this period", and the audit ends with exactly one governing
principle for the coming week.

## Failure

If no value set exists, the audit is refused with the reason stated. If the
window has no evidence at all, every value is marked unmeasured and the audit
reports that it graded nothing; it never infers compliance from intention. If
Journal {OS} or Habit Tracker {OS} is absent, the audit runs on user recall,
labels those verdicts as self-reported, and names the missing source. If two
values conflict and the declared priority order does not resolve it, the
conflict is reported unresolved and handed back to the user.
