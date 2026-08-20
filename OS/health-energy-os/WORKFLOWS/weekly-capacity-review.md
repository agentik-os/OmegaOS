# Health & Energy {OS}: Weekly capacity review

**Produces:** the updated capacity envelope for the coming week, emitted as `health.capacity.assessed`, plus a keep, change or stop decision on every active plan and experiment.
**Trigger:** the weekly cadence fires, or readiness has been below threshold for several consecutive days, or a plan reaches the review trigger written into it.
**Runs in:** `AUDIT`, handing to `PLAN` when the envelope changed, to `RECOVERY` when it shrank, and to `EXPERIMENT` when a question is still open.
**Takes:** the week's readiness assessments, trusted device trends, adherence from Habit Tracker {OS}, the load Execution {OS} actually ran, active experiments and their stopping rules, and the previous week's envelope.

## Steps

1. Run the safety gate. Any red flag surfaced during the week is handled first
   and routed, before any capacity claim is made.
2. Assemble the week: every readiness assessment, its limiting factor, and the
   days with no assessment. Count them and state the coverage.
3. Read the trends as trends: direction and context over the week, never a
   single day's score. Say explicitly what the device data cannot tell you.
4. Read adherence from Habit Tracker {OS} beside the plan. A plan that was not
   followed has not been tested, and it is reported as untested rather than as
   ineffective.
5. Compare what Execution {OS} actually ran against the envelope that was
   issued. A repeatedly exceeded envelope is a finding about the envelope or
   about the load, and it is named as one.
6. Identify the bottleneck for the coming week across sleep, movement, fuel,
   recovery and stress load. Label the evidence E1 through E5.
7. Check every active experiment against its stopping rule. Stop the ones that
   have hit it, and report the result including a null result.
8. Set the envelope for the coming week: the sustainable load, its constraints,
   its validity window and its confidence. If it shrank, say which specific
   commitments do not fit.
9. Emit `health.capacity.assessed` to Goal & Life Strategy {OS} and Strategy &
   Portfolio {OS}, `handoff.execution.capacity` to Execution {OS}, and any
   routine change as `handoff.habits.created` to Habit Tracker {OS}. Send
   agreed routines only, never raw medical detail.
10. Record one keep, change or stop decision per active plan, and stage the
    review canonically through Context & Memory {OS}.

## Completion test

The coming week's envelope is recorded with its constraints, validity window
and confidence; every active plan and experiment carries exactly one recorded
decision; the coverage of the week is stated as a count of assessed days; and
the envelope has been emitted to the units that schedule against it.

## Failure

Fewer than three readiness assessments in the week: report the coverage, carry
the previous envelope forward unchanged, and say that no revision is justified.
Adherence data unavailable from Habit Tracker {OS}: mark every plan untested
rather than judging it. Load data unavailable from Execution {OS}: state that
the envelope cannot be checked against reality this week. An experiment past
its stopping rule that was not stopped: stop it now and record why the rule was
missed. A red flag found in the week's records: escalate first, and produce no
capacity claim until it is resolved or professionally reviewed.
