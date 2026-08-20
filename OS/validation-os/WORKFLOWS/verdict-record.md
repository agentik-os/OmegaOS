# Workflow: Verdict record

**Produces:** the verdict on a signed test, the kill note where it applies, and
the propagation of both to the OS units that depended on the claim.

## Trigger

A signed test has finished: the stopping rule fired, the sample completed, or
the run was abandoned.

## Steps

1. **Close the run log.** Record what was actually done, to whom, when, and
   every deviation from the spec. Deviations logged after the fact are treated
   as deviations discovered late, and that is noted too.
2. **Classify deviations.** A deviation that could have changed the result makes
   the run `INVALID`. A cosmetic one is recorded and the run continues to
   verdict.
3. **Measure against the signed threshold only.** Not against a threshold that
   now seems more reasonable. If the result is being compared to anything other
   than the signed number, the verdict is `INVALID`.
4. **Issue exactly one verdict:**
   - `CONFIRMED`: the result met or beat the signed threshold.
   - `KILLED`: the result fell below the kill line.
   - `INCONCLUSIVE`: the result landed in the stated noise band, or the sample
     never completed.
   - `INVALID`: the spec was not followed, or the threshold moved.
5. **Write the verdict record:** claim, spec reference, raw result, threshold,
   verdict, date, and who ran it.
6. **On `KILLED`, write the kill note.** What dies: the feature, the segment,
   the channel, the revenue line, the bet. What survives untouched. What the
   next riskiest open claim now is and what its test costs.
7. **On `INCONCLUSIVE`, name the next cheapest test** or state that the claim is
   not worth further budget and will be carried as an accepted risk, with the
   owner's acknowledgement.
8. **Propagate.** Emit `validation.verdict.issued` to Strategy & Portfolio {OS},
   Business Model {OS}, Blueprint {OS} and Market Research {OS}. On a kill also
   emit `validation.claim.killed`.
9. **Update the claim register.** The claim moves from open to settled, with a
   pointer to the verdict. It is not deleted.
10. **Report the verdict without softening it.** A kill is stated as a kill in
    the first line, with what it saved.

## Completion test

- The verdict names the signed threshold it was measured against.
- The run log lists every deviation, including ones discovered late.
- A killed claim has a kill note that names what dies in the plan.
- An inconclusive result has either a next test or an owner-acknowledged
  accepted risk. It is never carried as a soft pass.
- Downstream OS units have received the event, and the claim register shows the
  claim as settled with a pointer, not deleted.
