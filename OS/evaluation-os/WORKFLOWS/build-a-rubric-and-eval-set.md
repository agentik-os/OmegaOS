# Workflow: Build a rubric and an evaluation set

Turn "I know it when I see it" into criteria two people apply the same way, and
a fixed set of cases that exercises all of them.

## Trigger

- Quality is being argued about and there is no written standard.
- Agent {OS} needs the rubric a brief must carry.
- A system is about to be changed and there is nothing to compare against.
- An existing rubric produces scores that nobody trusts.

## Steps

1. **Collect examples before writing anything.** Several good outputs, several
   bad ones, and if possible a pair that differ only slightly. Without examples,
   stop here; a rubric written from a general idea of quality measures the idea,
   not the work.
2. **Ask what makes the bad one bad,** case by case, and write the answers down
   verbatim. The rubric is assembled from these answers.
3. **Convert each answer into a criterion with an observable anchor.** Not
   "clear" but "states the constraint before the recommendation". Not "accurate"
   but "every factual claim carries a source". If applying the criterion requires
   knowing what the author intended, it is not yet a criterion.
4. **Give each criterion a scale** with described levels, so a middle score means
   something specific rather than meaning uncertainty.
5. **Test with two graders on the same five cases.** Where they disagree, the
   criterion is at fault. Rewrite it. Do not average the disagreement, because
   averaging preserves the ambiguity and hides it behind a number.
6. **Assemble the evaluation set** from four families: real cases from actual
   use, edge cases, adversarial cases designed to break the system, and cases
   whose correct answer is refusal or abstention.
7. **Include the refusal cases deliberately.** A system evaluated only on
   questions it should answer is being trained, by its own metric, to answer
   everything.
8. **Map coverage.** Every criterion must be exercised by at least one case. A
   criterion no case tests is decorative and is either dropped or given a case.
9. **Version and freeze both artifacts.** Record the date, the author and the
   reason for this version.
10. **Establish the first baseline** by running the current system and accepting
    the result as the reference point, explicitly, with approval.
11. **Stage the rubric, the set and the baseline** to Context & Memory {OS}.

## Completion test

- The rubric was derived from real examples, and the examples are kept with it.
- Every criterion has an observable anchor and a described scale.
- Two graders applied the rubric to the same cases and their disagreements were
  resolved by rewriting criteria, not by averaging.
- The set contains real, edge, adversarial and refusal cases, and each family is
  identifiable.
- Every criterion is exercised by at least one case.
- The rubric and the set are versioned, frozen and dated.
- A baseline exists and its acceptance is recorded with an approver.
- All three are staged to Context & Memory {OS}.

If any criterion still requires reading the author's mind, this workflow is not
finished, no matter how good the set looks.
