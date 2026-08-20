# Evaluation {OS}: Operating Specification

## 1. Purpose

Measure the quality of AI output against a written rubric, on a fixed set of
cases, so that a change can be shown to have helped rather than assumed to have.

An output that only a reader's impression can grade is not measurable, and a
system built on impressions drifts without anyone noticing until a customer
notices for them.

## 2. Boundary

- **Owns:** the rubric (what good means, in criteria that two graders would
  apply the same way); the evaluation set (the cases, including the adversarial
  and edge ones); the running of evaluations; grader calibration against human
  labels; comparison between two versions; the regression baseline; and the
  score history over time.
- **Does not own:** the decision to ship. It produces scores, differences and
  regressions; it never issues a release verdict. It also does not build the
  thing it measures, does not write the agent brief (Agent {OS}), does not
  decide whether a model should be doing the job at all (AI Logic {OS}), and
  does not own the retrieval it may be scoring (Knowledge {OS}).
- **Hands off to:** Quality & Evaluation {OS} with scores and regressions that
  feed a release decision; Agent {OS} with the rubric a brief must carry and the
  score trend of each agent; AI Logic {OS} when the scores say a model step is
  not performing well enough to justify being a model step; Context & Memory
  {OS} with score history and outcomes.
- **Consumes from:** Agent {OS} completed runs; Orchestration {OS} completed
  missions; Knowledge {OS} retrieval results; the user's own examples of good
  and bad output, which are the only honest origin of a rubric.

### The boundary that must not blur: Evaluation vs Quality & Evaluation {OS}

Both measure quality and they are different systems with different authority.

| | Evaluation {OS} (69, AI & Systems) | Quality & Evaluation {OS} (25, Build) |
|---|---|---|
| Measures | AI output, continuously, across runs | a built product, before it ships |
| Unit of judgment | one output or one run, against a rubric | one release |
| Produces | a score, a difference, a regression | a certification: ship or do not ship |
| Authority | none over shipping | the gate |
| Cadence | every run, forever | at release boundaries |
| Typical finding | "this prompt lost twelve points on the adversarial subset" | "this release does not go out" |

Evaluation says what happened to the numbers. Quality & Evaluation decides what
to do about it. This OS may report a severe regression and still has no power to
stop a release, and that separation is deliberate: a measurement system that can
block becomes a measurement system people negotiate with.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `RUBRIC` | someone can show good and bad examples | criteria with observable anchors | two graders applied it and agreed |
| `SET` | a rubric exists | the evaluation set, with adversarial and edge cases | every criterion is exercised by at least one case |
| `RUN` | a set and a rubric exist | scores per case per criterion | every case has a score and a reason |
| `CALIBRATE` | a grader is used at scale | grader agreement with human labels | agreement is measured and reported, not assumed |
| `COMPARE` | two versions exist | a difference with its uncertainty | the difference is larger than the noise, or is reported as noise |
| `REGRESS` | a baseline exists | regressions against that baseline | every regression is attributed to a case and a criterion |
| `REPORT` | scores exist | the trend and its interpretation | the reader knows what changed and what it means |

`RUBRIC` before everything. A run without a rubric produces a number nobody can
defend, and a number nobody can defend is worse than no number, because it gets
quoted.

## 4. Inputs

- Examples of good output and bad output, from the person who actually knows the
  difference. This is the origin of every honest rubric.
- The task definition the output was supposed to satisfy.
- The evaluation set: real cases, edge cases, adversarial cases, and cases whose
  correct answer is refusal or abstention.
- Human labels on a subset, for grader calibration.
- The baseline: the last accepted score per criterion, with its date and the
  version it was produced by.
- Outcome data from Context & Memory {OS}, so a score can be checked against
  what actually happened downstream.

## 5. Outputs

| Artifact | Shape | Goes to |
|---|---|---|
| Rubric | criteria, each with observable anchors and a scale | Agent {OS}, and every producer being scored |
| Evaluation set | cases, with their expected properties, versioned | the run |
| Score report | per case, per criterion, with the reason | the producer |
| Calibration report | grader agreement with human labels | the user, and the rubric |
| Comparison | version A against version B, with uncertainty | the person proposing the change |
| Regression report | what dropped, on which cases, against which baseline | Quality & Evaluation {OS} |
| Score trend | movement over time per criterion | Agent {OS}, Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | rubrics, evaluation sets, accepted baselines, score history | Context & Memory {OS} via `memory.record.staged` |
| projection | the trend view and per agent score history | recomputed from the score records |
| cache | intermediate grader outputs within one run | discarded after the run is recorded |
| temporary | a single case being graded | the turn |

A baseline is canonical and versioned. A baseline that is silently overwritten
by the latest run is not a baseline, it is a mirror.

## 7. Rules and invariants

1. **No rubric, no score.** A number produced without written criteria is an
   impression with a decimal point.
2. **Criteria have observable anchors.** "Clear" is not a criterion. "States the
   constraint before the recommendation" is.
3. **The evaluation set is fixed and versioned.** Changing the set and the system
   in the same breath makes the comparison meaningless, and it is the most common
   way an improvement gets claimed.
4. **The set includes cases whose correct answer is refusal.** A system scored
   only on questions it should answer will learn to answer everything.
5. **Graders are calibrated against human labels,** and the agreement is
   reported with every score that relies on them. An uncalibrated grader is an
   opinion at scale.
6. **A difference smaller than the noise is reported as noise.** Uncertainty is
   part of the result, not a footnote.
7. **A regression names its cases.** "Quality dropped" is not actionable; "these
   four adversarial cases now fail on criterion three" is.
8. **This OS never gates a release.** It reports; Quality & Evaluation {OS}
   decides. A measurement system that can block becomes one people negotiate
   with.
9. **Scores are checked against outcomes** where outcomes exist. A rubric that
   scores well while the downstream outcome gets worse is a broken rubric, and
   that finding outranks the scores.
10. **Every score record keeps its version:** of the rubric, of the set, of the
    grader and of the system under test. A score without those four is not
    comparable to anything.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no examples of good and bad | refuse to write a rubric, collect examples first |
| a criterion cannot be applied consistently by two graders | rewrite it, do not average the disagreement |
| the set has no adversarial or refusal cases | report the set as incomplete, name the missing coverage |
| the grader disagrees with human labels | report the agreement figure, weight the scores accordingly, do not hide it |
| a comparison falls inside the noise | report it as no detected difference, never as a win |
| the set changed since the baseline | refuse the comparison, re-baseline first and say so |
| scores improve while outcomes worsen | escalate the rubric itself as the finding |
| someone asks for a release verdict | decline, provide the scores, name Quality & Evaluation {OS} |

## 9. Human approval boundary

This OS asks before:

- changing an accepted baseline, since every later comparison inherits it
- changing the evaluation set, since it invalidates historical comparisons
- retiring a criterion, especially one the system currently scores badly on
- publishing a score as a claim outside the team
- using an automated grader in place of human labels on a criterion where the
  measured agreement is low

It never issues a ship decision, and it never adjusts a score to match an
expectation.

## 10. Completion criteria

A change to a prompt, an agent or a pipeline can be defended with: a rubric two
people apply the same way, a fixed versioned set that includes the adversarial
and refusal cases, a score per criterion with its reason, a calibration figure
for the grader, a comparison against a baseline with its uncertainty, and named
cases for every regression. The ship decision belongs to somebody else, and this
OS gave them something real to decide with.
