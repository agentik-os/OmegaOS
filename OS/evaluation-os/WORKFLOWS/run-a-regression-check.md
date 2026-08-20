# Workflow: Run a regression check

Find out whether a change helped, hurt, or did nothing measurable.

## Trigger

- A prompt, an agent brief, a retrieval index or a pipeline changed.
- Someone claims an improvement.
- A scheduled check on a system whose dependencies change underneath it.
- A user reported that output quality dropped.

## Steps

1. **Confirm the set has not changed since the baseline.** If it has, refuse the
   comparison, re-baseline first, and say clearly that the historical trend
   restarts here. Changing the set and the system together is how an improvement
   gets claimed that never happened.
2. **Record the four versions:** rubric, set, grader, and system under test. A
   score missing any of them cannot be compared to anything later.
3. **Run every case.** Not a sample, unless sampling is declared and its
   uncertainty is carried into the result.
4. **Score per criterion, with a reason per score.** A number with no reason
   cannot be argued with, and therefore cannot be improved from.
5. **Estimate the noise.** Re-run a subset unchanged and observe the spread. This
   is what makes the difference interpretable, and skipping it turns every small
   movement into a story.
6. **Compare against the baseline per criterion,** not only in aggregate. An
   aggregate that holds steady while one criterion collapses and another rises is
   the most common way a real regression hides.
7. **Report a difference inside the noise as no detected difference.** Never as a
   win, however welcome it would be.
8. **Attribute every regression to named cases** and the criteria they failed.
   "Quality dropped four points" is not actionable; "these four adversarial cases
   now fail criterion three" is.
9. **Check the refusal cases separately.** A system that gained on answering and
   lost on refusing has usually got worse, and the aggregate will say the
   opposite.
10. **Check against outcomes** where downstream outcome data exists. Scores
    rising while outcomes fall is a finding about the rubric, and it outranks the
    scores.
11. **Route regressions to Quality & Evaluation {OS}** with the evidence, and
    stop there. No ship recommendation is made in this workflow.
12. **Stage the run** to Context & Memory {OS} with its four versions.

## Completion test

- The evaluation set is identical to the one the baseline was produced on, or the
  comparison was refused.
- The four versions are recorded with the run.
- Every case was run, or the sampling and its uncertainty are declared.
- Every score carries a reason.
- Noise was estimated and differences are interpreted against it.
- Per criterion comparison exists, not only an aggregate.
- Every regression names its cases and criteria.
- Refusal cases were reported separately from answering cases.
- Regressions reached Quality & Evaluation {OS}, and this report contains no
  release recommendation.
- The run is staged to Context & Memory {OS}.

"It feels better" is not an output of this workflow, and neither is "ship it".
