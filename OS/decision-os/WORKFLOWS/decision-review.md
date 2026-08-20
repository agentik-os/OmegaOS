# Decision {OS}: Decision review

**Produces:** a review verdict appended to an existing decision record: one of held, wrong for the reason predicted, wrong for a reason not predicted, or still open with a new trigger, plus the lesson and the handoffs it generates.
**Trigger:** the review trigger on a record fires (a date or a named event), the outcome lands earlier than expected, or the user runs `/decision-review <record>`.
**Runs in:** `REVIEW`.
**Takes:** the decision record as written at the time; what actually happened, from the user, Journal {OS} and Execution {OS}; the pre-mortem predictions stored with the record; and the intuition signal and weight recorded with it.

## Steps

1. Load the record and read the rationale back to the user before discussing the
   outcome. The order matters: outcome first contaminates the recall.
2. Record what actually happened, in facts and dates, separately from any
   interpretation of it.
3. Compare the outcome to each pre-mortem prediction stored with the record.
   Mark each prediction as hit, missed, or untested.
4. Decide whether the process was sound independently of whether the outcome was
   good. A good outcome from a bad process is graded as a bad process.
5. Issue exactly one verdict: held; wrong for the reason predicted; wrong for a
   reason not predicted; or still open, in which case set a new trigger and a
   new date.
6. When the verdict is wrong for a reason not predicted, name the blind spot
   explicitly. This is the only output of the whole workflow that improves the
   next decision.
7. Grade the intuition signal that was recorded: it pointed at the outcome, it
   pointed away from it, or it was silent. Leave the record where Intuitive {OS}
   can read the resolved outcome for its own calibration.
8. Append the verdict to the record. Never edit the original rationale, evidence
   or discarded options.
9. Emit the handoffs: a changed allocation or a retired goal to Goal & Life
   Strategy {OS}, an organisational consequence to Review & Governance {OS}, the
   verdict as a dated entry to Journal {OS}.
10. Ask before persisting, then write to Context & Memory {OS}.

## Completion test

The record now carries exactly one appended verdict from the four, with the
outcome stated in facts, every pre-mortem prediction marked hit, missed or
untested, and the original rationale byte for byte unchanged.

## Failure

If the record is missing, the review is refused: there is nothing to grade, and
reconstructing a rationale after the outcome is known produces hindsight, not a
review. If the outcome has not landed yet, the verdict is still open and a new
trigger is set; a premature verdict is not written. If the record has no
pre-mortem predictions, the review says so and grades only the process and the
outcome. If the user wants to change the original rationale, the request is
refused and the correction is recorded as part of the review instead. If
Execution {OS} or Journal {OS} cannot supply what happened, the review runs on
user recall and labels the outcome self-reported.
