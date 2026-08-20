# Workflow: Contradiction report

**Produces:** a record of where credible sources disagree, what each side rests
on, why they disagree, and what evidence would settle it.

## Trigger

Two or more sources that both pass vetting give incompatible answers to the same
sub-question. Or someone asks for a single number where the sources plainly do
not agree on one.

## Steps

1. **Confirm it is a real contradiction.** Check first that the sources are
   answering the same question. Most apparent contradictions dissolve here:
   different population, different unit, different geography, different
   definition of the same word.
2. **State each position separately,** in the source's own terms, with its
   number or claim intact. Do not pre-harmonise the wording, because the wording
   is often where the disagreement lives.
3. **Attach the evidence to each side.** Source, locator, date of evidence,
   method, population, sample. A side with no method behind it is not a side, it
   is an assertion, and it is labelled as one.
4. **Check independence on both sides.** Three sources agreeing may be one
   source repeated three times. Count independent origins per side, not
   citations, and mark any circular support.
5. **Characterise the disagreement.** Name which of these it is: different
   population, different time window, different definition, different method,
   different data source, or different interest. Where more than one applies,
   name them all. This is the part that makes a contradiction useful rather than
   merely uncomfortable.
6. **Test the reconciliation hypothesis.** Ask whether both can be true at once
   under a stated condition (a segment, a period, a definition). If they can, the
   claim is not contested: it is conditional, and the condition is now part of
   the claim.
7. **Do not average.** A midpoint between two incompatible measurements is a
   number no source supports and no reader can check. If a single figure is
   demanded, give the range with each endpoint's source and method.
8. **Class the claim `contested`** and record it in the contradiction record with
   both sides intact.
9. **Name what would settle it.** The specific evidence, dataset, access or
   instrument that would resolve the disagreement, and roughly what obtaining it
   would cost.
10. **Route it.** If a decision rests on the contested claim, emit
    `research.claim.contested` and hand it to Validation {OS}, along with the
    evidence both sides rest on so a fair test can be designed. Research states
    the disagreement; it does not adjudicate it.
11. **Report the contradiction in the memo,** in the answer section rather than
    an appendix. A contradiction hidden at the back is a contradiction the reader
    will hit later, at a worse moment.

## Completion test

- Each contradiction entry names the sub-question both sides are answering, and
  the sides are genuinely answering the same one.
- Both positions appear in their own terms, with source, locator, date, method
  and population.
- Independent origins are counted per side, and circular support is marked.
- The disagreement is characterised with at least one named cause.
- The reconciliation hypothesis was tested and its result recorded, including
  when both can be true under a stated condition.
- No averaged or split-the-difference figure appears anywhere.
- Each contested claim states what evidence would settle it and its rough cost.
- Every contested claim a decision rests on was emitted as
  `research.claim.contested` to Validation {OS}.
- No side was dropped for being inconvenient, and the report says which side, if
  any, the memo's answer leans on and why.
