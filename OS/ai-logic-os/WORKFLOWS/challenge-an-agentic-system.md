# Workflow: Challenge an agentic system

Interrogate an existing agentic system (a pipeline, an agent, a skill, a coding
tool, a whole OS) against five questions, and return the costliest gap first.

## Trigger

- A system costs more than its owner expected and nobody can say which part.
- Output quality drifts and there is no loop that would have caught it.
- An incident happened that a gate should have prevented.
- A periodic review of a system that has been running unattended.

## Steps

1. **Read what already governs it.** Its rules, its laws, its configuration and
   its logs, before forming any opinion. A large share of apparent gaps are
   already governed, and proposing a duplicate control is a cost.
2. **Draw the actual execution path**, not the documented one. Where does
   control enter, what runs, what decides, what writes, where does it end.
3. **Question one: where does a model do the job of a conditional?** For each
   model call on the path, write the conditions that would replace it. Where you
   can write them, that is a finding: cite the file and line.
4. **Question two: where does a consequential output go unverified?** Follow
   each output to the action it causes. If the action has a consequence and no
   check stands between them, that is a finding. Name the missing verifier.
5. **Question three: where does an irreversible action lack a human gate?**
   Enumerate every send, publish, payment, deletion and signature. For each,
   locate the gate or the statistics that replaced it. A missing gate is a
   finding, cited.
6. **Question four: where is the feedback loop missing?** A system without a
   loop drifts silently. For each consequential decision, name the signal that
   would tell you it went wrong, and where that signal is read. An unread signal
   counts as absent.
7. **Question five: what primitive is absent and should exist?** Look for the
   thing being re-derived by hand every run, the gate that is hoped for rather
   than enforced, and the step that exists only to compensate for a defect
   elsewhere. These are the most expensive findings and the hardest to see.
8. **Drop every uncited finding.** Report how many were dropped. A finding
   without a file and line, a rule, or a log entry is not a finding.
9. **Rank by cost, not by ease.** The order is what the reader acts on.
10. **Specify only the first fix**, build ready, with a done test and a rollback,
    and route it to the OS that owns implementation.
11. **Write what you do not recommend.** Including any gap you found that is not
    worth closing, and why.

## Completion test

- All five questions are answered explicitly, including any answered with
  "cleared, and here is the rule that already covers it".
- Every surviving finding carries a file and line, a rule, or a log entry.
- The number of dropped uncited findings is reported.
- Findings are ranked by cost with the ranking inputs visible.
- Exactly one fix is specified, and it has a done test and a rollback.
- The not recommended section is non empty.

A challenge report that clears all five questions is a valid and useful result.
An audit that never returns "this holds" is an audit nobody reads.
