---
name: evaluation-os
description: Measure AI output quality with rubrics, not vibes. Evaluation {OS}, unit 69 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about evaluation or invokes /evaluation-os.
---

# Evaluation {OS}

Measure the quality of AI output against a written rubric, on a fixed set of
cases, so a change can be shown to have helped rather than assumed to have.

## When to use this

Use it when:

- Someone changed a prompt, an agent or a pipeline and says it is better.
- Output quality is drifting and nobody can say on which dimension.
- An automated grader is being used at scale and nobody has checked it against
  human judgment.
- Two approaches must be compared and the difference is small enough to argue
  about.
- An agent's brief needs the standard its output will be judged against.
- A retrieval system needs a fixed question set so index changes are measurable.

**Near neighbours, and why this is not them.** Quality & Evaluation {OS} in the
BUILD group certifies a built product before it ships: its unit is a release and
its output is a ship or do not ship verdict. This OS measures AI output
continuously, run after run: its unit is an output and its output is a score, a
difference and a regression. This OS has no authority over shipping, on purpose,
because a measurement system that can block becomes one people negotiate with.
Agent {OS} owns the brief that carries the rubric; AI Logic {OS} decides whether
the step should be a model call at all; Knowledge {OS} owns the retrieval this
OS may be scoring.

## Capabilities

- Turn examples of good and bad output into criteria with observable anchors.
- Test a rubric by having two graders apply it and comparing them.
- Build a fixed, versioned evaluation set including adversarial cases, edge
  cases and cases whose correct answer is refusal.
- Run an evaluation and produce a score per case per criterion, each with a
  reason.
- Calibrate an automated grader against human labels and report the agreement.
- Compare two versions and report the uncertainty alongside the difference.
- Detect regressions against a versioned baseline and name the failing cases.
- Track score trends per criterion over time.
- Check scores against downstream outcomes, and escalate the rubric itself when
  they diverge.

## Procedure

1. **Collect examples first.** Good output, bad output, and ideally a pair that
   differ only slightly. Without these, refuse to write a rubric.
2. **Extract criteria from the examples,** not from a general idea of quality.
   Ask what makes this one worse than that one, and write down the answer.
3. **Give each criterion an observable anchor** and a scale. If a criterion
   cannot be applied without knowing the author's intent, rewrite it.
4. **Test the rubric with two graders on the same cases.** Where they disagree,
   the criterion is at fault, not the graders. Rewrite it rather than averaging.
5. **Build the evaluation set:** real cases, edge cases, adversarial cases, and
   cases where the correct answer is refusal or abstention. Version it and freeze
   it.
6. **Check coverage:** every criterion must be exercised by at least one case,
   or it is decorative.
7. **Run.** Score per case per criterion, always with the reason. A score with
   no reason cannot be argued with, which means it cannot be improved from.
8. **Calibrate the grader** on a human labelled subset and report the agreement
   next to every score that relied on it.
9. **Compare against the baseline,** with the noise estimated. A difference
   inside the noise is reported as no detected difference.
10. **Attribute every regression** to specific cases and criteria.
11. **Check against outcomes** where they exist. Scores rising while outcomes
    fall is a finding about the rubric, and it outranks the scores.
12. **Report the trend,** and hand regressions to Quality & Evaluation {OS}
    without recommending a release decision.

## Handoffs

| Direction | Counterpart | Contract |
|---|---|---|
| in | Agent {OS} | completed runs to score, with their briefs |
| in | Orchestration {OS} | completed missions to score end to end |
| in | Knowledge {OS} | retrieval results against a fixed question set |
| in | Context & Memory {OS} | outcome records, so scores can be checked against reality |
| out | Agent {OS} | the rubric a brief must carry, and each agent's score trend |
| out | Quality & Evaluation {OS} | scores and named regressions feeding a release decision |
| out | AI Logic {OS} | evidence that a model step is not performing well enough to remain one |
| out | Context & Memory {OS} | rubrics, baselines and score history, staged as records |

This OS never says ship. It says what moved, by how much, on which cases, and
with how much confidence.
