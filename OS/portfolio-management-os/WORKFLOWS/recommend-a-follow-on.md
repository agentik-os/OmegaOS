# Recommend a follow-on

Produces a follow-on or stand down recommendation, carrying the thesis
checkpoint result and the current mark, addressed to Capital {OS}, with no
amount attached.

## Trigger

A position is raising, an existing right (pro rata, preemption, a second
tranche) has a deadline, or triage has produced a stand down question that the
owner wants answered deliberately rather than by letting a right lapse.

## Inputs

- The position record, including the reserve already committed by Capital {OS}.
- The current mark, with its method, evidence and date.
- The reporting series since funding, normalised.
- The thesis, its kill criteria and the result of the checkpoint due for this
  stage, from Investment Thesis {OS}.
- The round terms on offer, and the deadline on the right.
- The support ledger for this position: what the owner has already spent on it.

## Steps

1. Fix the deadline first. A right that lapses while the recommendation is being
   perfected has been decided by inaction, which is the outcome this workflow
   exists to prevent.
2. Confirm the position is current on reporting. A position that has missed two
   consecutive periods cannot be recommended for follow-on until reporting is
   restored or the owner accepts, in writing, that the recommendation is being
   made without it.
3. Request the thesis checkpoint result from Investment Thesis {OS} for this
   stage. If it has not been run, stop and request it. Without it there is no
   recommendation, only a feeling about a founder.
4. State the checkpoint result plainly, including when it is unfavourable. A
   recommendation that omits a failed checkpoint is a misrepresentation of the
   evidence to whoever decides the amount.
5. Restate the current mark with its method and date, and say whether the round
   on offer confirms, contradicts or is silent on that mark. A priced round
   above the mark is a marking event and goes through the mark workflow, not
   into this recommendation as a claim.
6. Read the reporting series against the baseline captured at onboarding. Name
   the two or three metrics that actually moved, and say whether they moved the
   way the thesis predicted.
7. State the reserve already committed against this position by Capital {OS}, as
   a fact, not as a proposal. This OS never proposes how much of the reserve to
   use.
8. Read the support ledger. A position that has consumed disproportionate
   capacity is a data point in the recommendation, because capacity is part of
   the real cost of holding it.
9. Write the recommendation as follow on or stand down, with the reasoning, the
   checkpoint result, the current mark and the evidence. State explicitly that
   no amount is being recommended.
10. Name the consequence of standing down under the terms on offer: dilution,
    loss of pro rata, loss of an information right, or a pay to play provision.
    A stand down with an unstated consequence is not a decision, it is a guess.
11. Emit `portfolio.followon.recommended` and hand the pack to Capital {OS}.
    **Human approval gate:** Capital {OS} decides the amount, and the allocation
    is approved and signed there. Nothing in this workflow commits money,
    exercises a right or communicates an intention to the company.
12. Record the outcome returned by Capital {OS}, approved or declined, against
    the position, so the next recommendation is written knowing what happened
    to the last one.

## Completion test

The pack handed to Capital {OS} contains: the checkpoint result including any
failure, the current mark with method and date, the reserve already committed,
the metrics moved against baseline, the support consumed, the consequence of
standing down, and the deadline. It contains no amount. Search the document for
a currency figure proposed as a follow-on size: there is none.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| no checkpoint run | recommendation written on conviction and founder rapport | abstain, request the checkpoint from Investment Thesis {OS}, do not proceed |
| unfavourable checkpoint softened | the failed criterion appears as a "watch item" | restate the checkpoint result verbatim in the pack |
| an amount slips in | "we should put in another 150k" | strip the figure, the pack states evidence and a direction only |
| deadline discovered late | the right lapses during preparation | fix the deadline at step 1 and escalate to the owner as soon as it is at risk |
| round used as a mark inside the pack | the new round price quoted as the position's value | route it through the mark workflow with method and evidence, then cite the approved mark |
| stale reporting ignored | recommendation made on numbers two periods old | require restored reporting, or an explicit written acceptance from the owner |
| stand down without consequence stated | "we will pass on this round" | name the dilution, lost rights or pay to play effect before the decision is taken |
