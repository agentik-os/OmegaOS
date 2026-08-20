# Health & Energy {OS}: Daily readiness

**Produces:** today's capacity assessment (level, limiting factor, constraints, validity window, confidence), and the same envelope delivered to the units that are about to schedule work against it.
**Trigger:** the morning cadence fires, the user asks what they can carry today, or Execution {OS} or Goal & Life Strategy {OS} requests a current envelope before committing a load.
**Runs in:** `CHECK_IN`, escalating to `RECOVERY` when readiness is below threshold and stopping at the safety gate when a red flag appears.
**Takes:** the user's own report of the night and the day ahead, trusted device data with source and timestamp, the standing baseline from the last `AUDIT`, adherence signals from Habit Tracker {OS}, and the load already committed for today.

## Steps

1. Run the safety gate before anything else. Any red flag stops this workflow,
   produces the alert and the routing to a qualified human professional, and
   ends the session. Nothing else is produced.
2. Load the standing baseline. If there is none, say so, run this pass at low
   confidence, and offer `AUDIT`.
3. Read the user's report: sleep duration and quality, energy, soreness,
   stress, appetite, mood, and the constraints of the day.
4. Read trusted device data for the same window. Every record must carry a
   source and a timestamp or it is not used.
5. Where the report and the device disagree, keep both. Name which one the
   recommendation depends on. Do not average them and do not silently prefer
   the number.
6. Identify the limiting factor across sleep, movement, fuel, recovery and
   stress load. One factor, named. If it cannot be identified, say what single
   observation tomorrow would identify it.
7. State the capacity level, today's ceiling, the constraints that follow from
   it, the validity window, and the confidence. Label each material claim E1
   through E5.
8. Compare the ceiling to the load already committed for today. If the load
   exceeds the ceiling, issue the constraint: `handoff.execution.capacity` to
   Execution {OS}, and the directive to shrink the active set to Habit Tracker
   {OS}. If readiness is below threshold, enter `RECOVERY`.
9. Stage the assessment canonically through Context & Memory {OS} and emit
   `readiness.assessed`.
10. Close with one concrete move for today, not a list.

## Completion test

A capacity level, a named limiting factor, a validity window and a confidence
are all recorded for today, every claim carries an evidence label, and either
the committed load fits under the ceiling or a constraint has been issued to
the units that own that load.

## Failure

No baseline: run at low confidence, say so in the first line, and offer
`AUDIT`. No device data: run on the subjective report alone and say which
claims that weakens, do not treat absence as a good reading. Report and device
in conflict: report both and name the dependency. Habit Tracker {OS} or
Execution {OS} unreachable when a constraint must be delivered: tell the user
the constraint has not landed, name the unit, and hold it pending. Any red
flag: stop, escalate, produce nothing else.
