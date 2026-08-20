# Workflow: status and slip

Produces a status report that states position rather than activity, and moves a
slip into the open on the day it becomes knowable.

## Trigger

The reporting cadence fires, or anyone learns that a milestone will not hold.
The second trigger takes priority and does not wait for the first.

## Inputs

- The current milestone plan and the critical path.
- Evidence of progress since the last report: completed acceptance tests,
  merged work, delivered artifacts, signed approvals.
- Blockers reported by owners.
- Any change records opened since the last report.

## Steps

1. **Collect evidence, not opinions.** Each milestone is advanced only by its
   own acceptance test. Confidence is not progress.
2. **Fix the position.** Milestone N of M, and whether it is on, ahead or
   behind, in days.
3. **Compute the slip on the critical path.** Slip off the critical path is
   noted; slip on it moves the end date and is stated as such.
4. **List the blockers with an owner and a date each.** A blocker with no owner
   is not a blocker, it is a complaint.
5. **Name the next decision due.** Every status report ends on a decision
   somebody has to make and by when, or explicitly on "no decision due".
6. **Say what is unknown.** A milestone with no evidence since the last cycle is
   reported as unknown position, never as a percentage.
7. **Route it.** Client {OS} translates it for the client. Meeting {OS} takes
   any decision that needs a room. Review & Governance {OS} receives it when
   the slip crosses the agreed threshold.
8. **Update the plan or mark it stale.** A plan that no longer matches reality
   and has not been replanned is marked stale in the record.

## Completion test

- The report states position and slip in days, not activity.
- Every claim of progress is backed by a passed acceptance test or a delivered
  artifact.
- Every blocker has an owner and a date.
- The next decision due is named, or the report says none is due.
- Milestones with no evidence are reported as unknown, not estimated.

## Failure paths

| Situation | Response |
|---|---|
| an owner reports "nearly done" with no artifact | record unknown position, and name the artifact that would settle it |
| the slip is large and the reporter wants to wait for better news | send it today; the value of a slip report falls to zero at the deadline |
| several milestones slip at once | stop reporting and run the recovery workflow instead |
| the plan has not been updated for two cycles | mark it stale, and refuse to report green against a stale plan |
