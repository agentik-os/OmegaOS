# Retire a thesis and update the hit rate

Produces a retirement verdict, a wrong versus unlucky determination with its
basis, an updated hit rate, and a pattern library entry written to change the
next draft.

## Trigger

A position is exited or written off, a kill criterion has been acted on, a bet
was never entered after the thesis was written, or a thesis has been superseded
by a later one on the same opportunity.

## Inputs

- The full version history of the thesis, including struck claims.
- Every checkpoint record, including missed checkpoints.
- The realised outcome from Capital {OS} and Portfolio Management {OS}.
- The pre-mortem, so the actual cause can be compared with the predicted one.
- The existing hit rate and pattern library from Context & Memory {OS}.

## Steps

1. State the verdict: validated, invalidated or superseded. A thesis whose
   claims held but whose position lost money is still recorded against what the
   claims did, not against the profit and loss alone.
2. Record the realised outcome as a fact from Capital {OS} and Portfolio
   Management {OS}, not as a recollection.
3. Walk the claim register one last time and mark the final state of each
   claim, including the ones that were never testable.
4. Compare the realised cause with the pre-mortem. Record whether the actual
   cause was ranked, mentioned, or absent from the pre-mortem entirely.
5. Determine wrong versus unlucky: was the reasoning faulty given what was
   knowable at the time, or was the reasoning sound and the world went the
   other way. Write the basis for the call, not just the label.
6. **Human approval gate:** present the wrong versus unlucky call with its
   reasoning and have the user confirm it. This OS never records a loss as
   unlucky on its own, because a system that lets every loss be unlucky teaches
   nothing.
7. Record whether the checkpoint discipline held: how many checkpoints were
   scheduled, how many ran, and how many ran late.
8. Write the pattern entry in a form that changes behaviour: the condition that
   was present, what it predicted, and the specific question the next `DRAFT`
   should ask because of it.
9. Recompute the hit rate, excluding retrospective theses and showing the
   excluded count.
10. Store the retirement, close the checkpoint calendar for this thesis, and
    hand the pattern entry to Context & Memory {OS} so the next draft loads it.

## Completion test

A retirement record exists carrying the verdict, the realised outcome with its
source, the final state of every claim, the pre-mortem comparison, a user
confirmed wrong versus unlucky call with its written basis, and the checkpoint
discipline count. The hit rate has been recomputed and shows the retrospective
exclusions. At least one pattern entry has been written or an existing one has
had its count incremented.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| Every loss is unlucky | no retirement is ever recorded as wrong | show the ratio across all retirements and require the user to defend it |
| Outcome scored instead of reasoning | a winning bet with broken claims recorded as validated | record the claim verdicts separately from the outcome and report both |
| No pre-mortem to compare | the pre-mortem was skipped at draft | record the absence as its own pattern entry, since skipping the pre-mortem is itself a repeating behaviour |
| Retrospective thesis counted | a thesis written after the cheque enters the hit rate | exclude it mechanically and display the excluded count next to the rate |
| Pattern entry is a platitude | "do better diligence next time" | rewrite as a specific question the next draft must answer, or discard it |
| Thesis superseded without closure | a v2 written and the v1 left open | close v1 as superseded and name the successor, so the calendar does not carry a dead thesis |
