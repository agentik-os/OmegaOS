# Approve a commitment amount

Produces a signed allocation decision record: the initial amount, the follow-on
reserve, the policy lines it was tested against, and the human signature that
releases it for execution.

## Trigger

A named commitment has a thesis from Investment Thesis {OS}, a completed
diligence outcome from Due Diligence {OS}, and terms agreed or nearly agreed
from Deal Structuring {OS}, and somebody now has to decide the number.

## Inputs

- The candidate record and the screen verdict already produced by
  `/capital screen`.
- The thesis and its kill criteria, events `thesis.drafted` and
  `thesis.kill_criteria.set`.
- The diligence outcome, event `diligence.completed`, plus any
  `diligence.redflag.raised` since the screen.
- The agreed terms, event `structure.terms.agreed`, because the instrument
  changes what a given amount buys.
- The current allocation policy in force, and the current concentration table.
- The deployable balance remaining in this period's budget.

## Steps

1. Confirm the screen verdict is still current. If the candidate has changed
   size, stage, instrument or sector since the screen, rerun the screen before
   going further.
2. Confirm thesis and diligence are both present. If either is missing, abstain,
   name which one and which OS produces it, and stop. Conviction is not a
   substitute for either.
3. Read every red flag raised since the screen. A red flag raised after the
   screen reopens the decision: the earlier verdict is never carried forward
   over it.
4. Read the terms. State in one line what this amount actually buys under this
   instrument, because the same number is a different position under a
   convertible, an equity round and an earnout.
5. Propose the initial amount within the cheque band, with the reasoning tied to
   the thesis, not to the size of the ask.
6. Compute the follow-on reserve in the same step, using the reserve ratio in
   policy. Where the reserve is legitimately zero, write it as zero with its
   reason, for example no follow-on right under this instrument.
7. State the expected lock period and the resulting illiquid fraction of the
   pool after this commitment.
8. Run the concentration test with the reserve included, using
   `/capital concentration --with`. A commitment that fits today and breaches
   once its reserve is drawn has already breached.
9. If any ceiling is breached, take one of exactly two paths: decline the
   commitment and record the line that killed it, or route a written amendment
   through Review & Governance {OS} and wait for `change.approved` before
   sizing again. Do not record an undocumented exception.
10. Confirm the funding source: cash that exists, or a facility that is drawable
    and named. If the source is a forecast, mark the allocation conditional,
    name what must be confirmed, and do not present it as approved.
11. Label every forward-looking number in the record on the E1 to E5 scale.
    Return expectations, hold periods and follow-on probability are labelled,
    never stated flat.
12. **Human approval gate.** Present the complete decision record for
    signature: amount, reserve, lock, ceilings after, funding source, labelled
    assumptions. The record is unsigned and non-binding until the allocator
    signs. On signature emit `capital.allocation.approved` and
    `capital.reserve.committed`, or on a decline emit
    `capital.allocation.declined` with the governing policy line.
13. Hand the signed record to the allocator for execution and to Deal
    Structuring {OS} and Portfolio Management {OS} for their work. This OS does
    not initiate the wire, the subscription or the capital call.

## Completion test

A decision record exists with a human signature and a date, containing an
amount, a reserve line (including an explicit zero with its reason), the lock
period, the concentration table before and after, the funding source, and an
E label on every forward-looking claim. The concentration table after the
commitment shows no breached ceiling, or shows one with a matching
`change.approved` reference. No money has moved.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| reserve discovered later | position asks for follow-on money that was never held back | refuse to approve any allocation without a reserve line, zero or otherwise |
| approving against a forecast | funding source is an expected distribution or exit | mark conditional, name the confirmation required, withhold approval |
| exception instead of amendment | "we will go slightly over the ceiling this once" | decline, or route the written amendment, and record which one happened |
| ask-driven sizing | the amount equals the ask because the ask was that number | require the amount to be justified against the thesis, then compare to the ask |
| stale screen | candidate changed round size or instrument since the screen | rerun the screen before sizing |
| red flag ignored | diligence raised a flag after the screen and the memo does not mention it | reopen the decision, restate the verdict with the flag in it |
| single-point return promise | "this returns 4x" | replace with a labelled range and the assumptions it depends on |
