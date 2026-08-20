# Close diligence with conditions

Produces the diligence report, including a required list of what could not be
verified, and the conditions that must be satisfied before completion, each
with an owner and a due date.

## Trigger

The plan's decision relevant questions are answered, the diligence budget is
spent, a stop has been called, or a completion date is approaching and the
decision maker needs the position as it stands.

## Inputs

- The diligence plan with its question list and dropped questions.
- The evidence log in full, with sources, dates, confidences and
  classifications.
- The findings register with severities and consequence classes.
- Every red flag and the dated human decision on each.
- The status of each item awaiting a named professional's written opinion.
- The deal calendar and the completion date, from Acquisition {OS}.

## Steps

1. Mark every question on the plan answered, unanswered or refused. There is no
   fourth state and no blank.
2. Build the list of what could not be verified: unanswered questions, refused
   requests, and anything resting only on a seller source. Assign each a
   severity. This list is a required section of the report and does not move to
   an appendix.
3. Confirm every finding in the register carries a severity, an evidence
   reference and a consequence class. Anything without a consequence is held
   out of the register as an open observation and named in the report as such.
4. Confirm every red flag has a dated human decision attached. An open red flag
   blocks the close: report it as open and stop rather than closing over it.
5. List the items still awaiting a professional's written opinion, name the
   profession for each, and mark them open. The report never substitutes this
   OS's analysis for a quality of earnings, a legal opinion or an audit.
6. Derive the conditions to completion from the findings whose consequence
   class is `condition`, and give each an owner and a due date. Conditions are
   stated as facts to be satisfied, not as clauses to be drafted.
7. Write the report: what was verified and by which source, what was not, what
   each finding does to the decision, and what is still being taken on trust.
   The report states the position; it never states that the position supports
   proceeding.
8. Hand the findings register to Deal Structuring {OS} with the consequence
   classes intact and no proposed terms attached, and the conditions to
   Acquisition {OS} for the completion sequence.
9. Hand the verified basis and the gaps list to Capital {OS} so that any
   approval is made with the unverified items visible.
10. **Human approval gate:** declaring diligence complete is a human decision,
    and the report is transmitted to advisers, the counterparty or anyone else
    only by a person who has read it. This OS has no send and never marks a
    condition satisfied.
11. Emit `diligence.completed` and store the report, the register and the
    conditions in Context & Memory {OS}.

## Completion test

Every plan question has one of three states, the list of what could not be
verified exists in the report body with severities, no red flag is open, every
finding carries a severity and a consequence class, and every condition to
completion has a named owner and a due date. A human has explicitly declared
diligence complete and has sent the report themselves.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| Gaps buried | what could not be verified appears in an appendix or as "no issues identified" | move it into the report body with severities; absence of evidence is reported as absence |
| Closing over an open red flag | the report is written while an escalation has no decision | refuse to close, report the flag as open, escalate again |
| Budget spent, close reported as complete | the plan is unfinished and the report reads as finished | close as stopped, name the unanswered decision relevant questions, and state that the decision would be taken without them |
| Conditions written as clauses | the report proposes drafting language | strip it back to the fact that must be satisfied and hand it to Deal Structuring {OS} |
| A professional opinion assumed to have arrived | an item marked closed with no attached document | reopen it and name the profession that must answer |
| The report reads as a recommendation | a conclusion that the deal should proceed | rewrite as position and consequence only; the decision belongs to a human in Capital {OS} or Acquisition {OS} |
| Report sent by the system | a draft transmitted automatically to advisers | blocked by design; transmission is a human act after a human has read it |
