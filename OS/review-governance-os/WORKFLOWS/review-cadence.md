# Workflow: the review cadence

Produces a review record whose output is decisions with owners and dates. A
review that ends in a summary has not run.

## Trigger

The day, week, month or quarter closes. Each level has a different length and a
different job, and merging them destroys all four.

## Inputs

- Execution {OS}: day records, weekly resets, monthly system audits.
- Project {OS}: status, slips, change records, closeouts.
- KPI & Analytics {OS}: readings, movements and threshold breaches.
- Operations {OS}, Client {OS}, Team & Delegation {OS}, Documentation {OS}:
  their escalations for this period.
- The open decisions from the previous cycle at the same level.

## Steps

1. **Close the previous cycle first.** Every decision from last time is closed,
   in progress with a date, or explicitly dropped with a reason. Nothing carries
   silently.
2. **Choose the level and hold its length.**
   - Daily: minutes. What was intended, what happened, what is different
     tomorrow.
   - Weekly: short. What moved, what did not, what is decided this week.
   - Monthly: metrics against thresholds, and the systemic findings.
   - Quarterly: policy, decision rights, the risk register, and the portfolio of
     changes made this quarter.
3. **Read the evidence, not the memory.** What the OSes recorded during the
   period. Recollection at the review is an input, and it is the least reliable
   one.
4. **Compare against what was intended.** A review with no stated intention to
   compare against becomes a narration of activity.
5. **Explain the gaps in conditions,** not in character. What made the failure
   easy, what made the success repeatable.
6. **Look for the repeat.** Anything that has now appeared in three cycles is
   systemic and gets a change proposal rather than another mention.
7. **Take the decisions.** Each one with an owner and a date. A finding without a
   decision is a note.
8. **Route them.** To the owning OS for execution, to the change workflow if a
   boundary or policy is affected, to the risk register if it is a risk.
9. **Record it in the audit trail,** append-only, with the evidence cited.
10. **Set the next cycle's date** and, at the quarterly level, review whether the
    cadence itself is still the right shape.

## Completion test

- Every decision from the previous cycle at this level is closed, dated or
  dropped with a reason.
- The review stayed within the length that its level is supposed to take.
- Every finding cites the evidence it came from.
- Every finding either produced a decision with an owner and a date, or is
  explicitly recorded as noted and not acted on.
- Anything appearing for the third cycle became a change proposal.
- The record is in the append-only audit trail.

## Failure paths

| Situation | Response |
|---|---|
| the review has become a status meeting | say so, cut it back to decisions, and route status to Project {OS} and Meeting {OS} |
| the evidence is missing | run on what exists, and record the missing evidence as the first finding of the cycle |
| the same finding appears for the fourth time | stop discussing it; escalate to a change proposal with a named owner, and record why three cycles produced nothing |
| the review keeps expanding until nobody attends | shorten it by dropping items that never produce a decision, not by attending less |
