# Run a thesis checkpoint

Produces a claim by claim verdict against dated evidence, a kill criteria
result, and a stored checkpoint record that survives whatever the user feels
about it.

## Trigger

A scheduled checkpoint date arrives (`thesis.checkpoint.due`), a stated
milestone is reached, a follow on is being considered, or a red flag arrives
from Due Diligence {OS} or an impairment from Portfolio Management {OS}.

## Inputs

- The stored thesis, latest version, quoted rather than summarised.
- The claim register with each claim's disproof condition and due date.
- The kill criteria sheet.
- Dated evidence: KPI reports and marks from Portfolio Management {OS}, plus
  any user supplied evidence carrying a source and a date.
- The checkpoint history, including any missed checkpoints.

## Steps

1. Open the stored thesis text. Work from the file, not from what anyone
   remembers the thesis said.
2. Record any checkpoint dates that passed without a run, with their dates,
   before doing anything else. A missed checkpoint is a finding.
3. For each claim, collect the evidence that bears on its disproof condition,
   with its source and date. Reject undated evidence.
4. Mark each claim holding, weakening, broken or untestable. A claim with no
   available evidence is untestable this cycle and is never marked holding.
5. Test each kill criterion against the evidence and state met, not met, or
   indeterminate, with the number or observation that decides it.
6. Write the verdict into the record before writing any narrative. The order
   matters: a narrative written first will bend the verdict.
7. Compare the user's current stated justification for holding against the
   stored text. If the reason no longer appears in any version, name it as
   drift and route to a revision decision.
8. If a kill criterion is met, emit `thesis.invalidated` and stop the
   checkpoint there. **Human approval gate:** exiting, writing down or holding
   anyway is a human decision made in Capital {OS} and Portfolio Management
   {OS}. This workflow presents the evidence and never closes a position,
   moves money or issues an instruction.
9. If claims are broken but no kill criterion is met, open a revision decision:
   revise the thesis with the change and its reason, or record explicitly that
   the user is holding a thesis they know is weakening and why.
10. Store the checkpoint record, emit `thesis.reviewed`, and set the next
    checkpoint date with its evidence source.

## Completion test

A checkpoint record exists for this date with every claim carrying one of the
four verdicts and its evidence reference, every kill criterion carrying an
explicit result, any missed prior checkpoints listed, and a next checkpoint
date on the calendar. The record exists whether or not the result was
favourable.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| Evidence unavailable | no dated source bears on a claim | mark untestable this cycle with the reason, never holding, and name the missing source |
| Undated or unsourced evidence | a number with no origin | reject it and treat the claim as untestable |
| Narrative first | a story explaining why the numbers are fine, written before the verdicts | discard the narrative, restart at the claim register |
| Silent kill criterion | criterion met and the checkpoint continues to a comfortable conclusion | stop at the criterion, emit `thesis.invalidated`, escalate to the human decision |
| Drift absorbed | the current reason for holding is new and nobody notices | quote both texts, name the drift, require a revision decision |
| Checkpoint skipped repeatedly | the same thesis is never checked | write each missed date into the history and raise it to Review & Governance {OS} as a cadence failure |
