# Decision {OS}: Decision record

**Produces:** one decision record: the question, the choice, the rationale, the discarded options and why, the evidence relied on, the intuition signal and the weight it was given, the reversibility class, and the review trigger with its date.
**Trigger:** the user runs `/decide` or `/decision-record`, or a framed call reaches its deadline or meets its evidence threshold.
**Runs in:** `FRAME`, `OPTIONS`, `REVERSIBILITY`, `EVIDENCE`, then `DECIDE`.
**Takes:** the call in the user's own words with its real deadline; weighted values and the control map from Alignment {OS}; the objective from Goal & Life Strategy {OS}; a signal with its calibration weight from Intuitive {OS}; external evidence from Research {OS} or the user; a capacity veto from Health & Energy {OS} when load is committed; and any prior record on the same call from Context & Memory {OS}.

## Steps

1. Search Context & Memory {OS} for a prior record on this call. If one exists,
   show it with its review verdict before anything new is generated.
2. Write the decision question as one sentence and get explicit agreement on it.
   Fix the deadline, and state whether the deadline is real or self-imposed.
3. Name the objective this call serves. If Goal & Life Strategy {OS} has none,
   record the call as unanchored and ask the user for the objective in one
   sentence.
4. Pull the criteria and their weights from Alignment {OS}. Mark any criterion
   that originates in this session as unsourced.
5. Sort the situation into what the user chooses, influences, cannot control,
   and does not yet know. Delete constraints that turn out not to be real.
6. Generate at least three options, including doing nothing with its cost
   stated, and ask directly for the option being avoided.
7. Class each option by reversibility and price the undo. An option whose undo
   cost cannot be established is treated as irreversible.
8. Score the options against the weighted criteria, then run a pre-mortem on
   each serious option: assume it failed, say why.
9. Record the intuition signal with its calibration weight, or as uncalibrated
   with zero weight. Note explicitly if it points against the leading option.
10. Test the evidence against the threshold. If it is short and the deadline
    allows, define one bounded experiment with a date and stop the workflow
    there.
11. Ask for approval, then decide. Approval is mandatory for any irreversible
    option and for deciding under an unmet threshold.
12. Write the record with a review trigger, hand the work to Execution {OS} and
    the record to Journal {OS}.

## Completion test

The record exists in Context & Memory {OS} and contains all of: the question in
one sentence, the chosen option, at least two discarded options each with a
reason, the reversibility class with its undo cost, the evidence and whether the
threshold was met, the intuition signal with its weight, and a review trigger
with a date. A record missing the review trigger is not a record.

## Failure

If the objective is missing, the record is written and flagged unanchored. If
Alignment {OS} is unavailable, the user supplies at most three criteria, marked
self-declared, and the record says the values lens was not run. If fewer than
three real options appear, the workflow returns to `FRAME` and names the
constraint collapsing the option space. If the evidence threshold is unmet and
the deadline has not arrived, no decision is written: only an experiment with a
date. If the threshold is unmet and the deadline has arrived, the call is made
on the reversibility class, the unmet threshold is recorded verbatim, and the
review trigger is set early. If the user declines a field, it is marked declined
rather than filled in.
