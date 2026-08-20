# Workflow: Kill review verdict

**Produces:** a verdict per bet (continue, narrow, pivot, pause or kill),
measured against the ORIGINAL thesis and thresholds, with the reusable learning
captured and the released resources re-assigned or explicitly banked.

## Trigger

A review trigger fired, a scenario signpost was hit
(`scenario.signpost.triggered`), a kill criterion was met, a validation verdict
landed (`validation.verdict.issued`, `validation.claim.killed`), or a project is
limping and nobody agreed in advance what would stop it.

Runs the kill review protocol.

## Steps

1. **Retrieve the original record.** The thesis as written when the bet was
   funded, the kill criteria as signed, the allocation committed, and the
   metrics that were supposed to move. Reviewing against a thesis reconstructed
   from memory is how a failing bet survives its own review.
2. **Assemble the actual evidence.** What the metrics show, what
   `execution.outcome.proven` reported, what Validation {OS} settled, and what
   the market did. Label each item E1 to E5. Absence of evidence is recorded as
   absence, never as a neutral result.
3. **Compare thesis to reality, clause by clause.** For each claim in the
   original thesis: confirmed, refuted, or still unknown. A thesis that has
   quietly been rewritten since funding is flagged, and the review is run against
   the original.
4. **Test the kill criteria.** Has any signed criterion been met? If yes, the
   default disposition is kill, and continuing requires an explicit, recorded
   override under the human approval boundary, with a new review date.
5. **Name the sunk cost out loud.** If the bet is being defended by what has
   already been spent on it, say so, and re-ask the decision as if starting
   today with the resources currently committed to it. Record that the reframing
   was applied and what it changed.
6. **Run the red team pass.** One independent argument for killing it and one
   for continuing, each at full strength. Never average the two: expose the
   governing tradeoff and let the operator choose.
7. **Choose the verdict:** continue, narrow, pivot, pause or kill. Narrow and
   pivot each require a NEW thesis and NEW kill criteria before they are allowed
   to keep their allocation; a pivot without a fresh thesis is a continue
   wearing a different name.
8. **Capture the learning.** What this bet taught that is reusable elsewhere,
   written so a future portfolio session can find it, plus the assumption that
   turned out wrong and where it originally came from.
9. **Release the resources.** State exactly what is freed in hours, people and
   capital, and either re-assign it to a named item or bank it explicitly.
   Resources that are freed without being re-assigned are silently reabsorbed by
   whatever is loudest.
10. **Update the not-doing list.** A killed bet joins the exclusions with the
    reason and the condition that would reopen it, so it is not re-proposed in
    two quarters by someone who never saw the reason.
11. **Route governance and approval.** A consequential pause or kill goes out as
    `strategy.change.requested`, and `portfolio.item.paused` or
    `portfolio.item.killed` is emitted only after `change.approved` returns.
    Overriding a met kill criterion requires human approval and is recorded with
    its author and stated reason.
12. **Emit `strategy.review.completed`** and write the verdict, the learning and
    the released allocation to canonical state through Context & Memory {OS}.
    The killed item is retained, never deleted.

## Completion test

- The verdict is measured against the original thesis and the signed kill
  criteria, both quoted in the record.
- Each clause of the original thesis is marked confirmed, refuted or unknown.
- If a kill criterion was met, either the bet is killed or an approved,
  attributed override with a new review date exists.
- Any sunk-cost argument raised in the review is named in the record along with
  what the today-start reframing changed.
- Narrow and pivot verdicts carry a new thesis and new kill criteria.
- The released resources are quantified and either re-assigned to a named item
  or explicitly banked.
- The learning is captured, and a killed bet appears on the not-doing list with
  its reopening condition.
- No consequential pause or kill event was emitted before its `change.approved`
  returned.
