# Workflow: Calibrate a grader

Find out whether the thing producing your scores agrees with the people whose
judgment the rubric was built from.

## Trigger

- An automated grader is about to be used at scale.
- Scores are being quoted in decisions and nobody has checked them.
- A grader was changed, or the model behind it was updated.
- Two graders disagree and it is not obvious which is right.

## Steps

1. **Select a calibration subset** that covers every criterion and every case
   family, including refusal cases. A subset drawn only from easy cases measures
   nothing worth knowing.
2. **Collect human labels first, blind to the automated score.** Showing the
   machine score before the human labels contaminates the reference, and the
   contamination is invisible afterwards.
3. **Use at least two human labellers** on an overlap, so that human disagreement
   is measured too. A grader cannot be more consistent than the standard it is
   being compared with, and knowing the human ceiling changes how the result is
   read.
4. **Run the automated grader on the same subset,** with the same rubric version.
5. **Measure agreement per criterion,** never only overall. Graders are typically
   strong on mechanical criteria and weak on judgment criteria, and an overall
   figure hides exactly the criteria you needed to know about.
6. **Identify the systematic direction of any disagreement.** A grader that is
   consistently generous is a different problem from one that is inconsistent,
   and the two need different responses.
7. **Decide per criterion:** trust the grader, weight it, or require human
   labels. Record the decision and the agreement figure that justified it.
8. **Attach the agreement figures to every subsequent score** produced by this
   grader. A score whose grader agreement is unknown is reported as such.
9. **Re-calibrate when anything changes:** the grader, the model behind it, the
   rubric, or the population of cases.
10. **Stage the calibration** to Context & Memory {OS} with its date and versions.

## Completion test

- The calibration subset covers every criterion and every case family, including
  refusal cases.
- Human labels were collected blind to the automated scores.
- Human to human agreement was measured on an overlap.
- Agreement is reported per criterion, not only overall.
- Any systematic direction of disagreement is named.
- Each criterion has a recorded decision: trust, weight, or require humans, with
  the figure that justified it.
- Subsequent scores carry the agreement figures.
- The calibration is staged to Context & Memory {OS} with its versions.

An uncalibrated grader is an opinion applied at scale, and it is more dangerous
than no grader because it comes with a number attached.
