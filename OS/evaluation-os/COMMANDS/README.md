# Evaluation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install evaluation-os` | Installs this OS into your environment | Once, first |
| `agentik configure evaluation-os` | Collects rubric locations, graders and baseline storage | After install |
| `agentik run evaluation-os` | Starts the OS | Every session |
| `agentik doctor evaluation-os` | Checks config, grader reachability, set integrity and dependencies | When something is off |
| `agentik update evaluation-os` | Updates to the latest version | When a release lands |
| `agentik eval evaluation-os` | Runs its own evaluation suite | Before trusting it |

## OS commands

The OS answers to `/eval`.

### `/eval rubric <task>`

Turn examples of good and bad output into criteria with observable anchors.

**When to use it:** before any score is produced, and whenever two people disagree
about whether an output is good.
**Returns:** the criteria, their anchors, their scale, and the two grader
agreement result. A criterion two graders apply differently is returned for
rewriting rather than averaged.

### `/eval set <task>`

Build the fixed, versioned evaluation set.

**When to use it:** once a rubric exists, and again only with explicit approval,
because changing the set invalidates every historical comparison.
**Returns:** the cases with their expected properties, the coverage map of
criteria to cases, and an explicit report of missing adversarial, edge and
refusal coverage.

### `/eval run <version>`

Score a version against the rubric and the set.

**When to use it:** after any change to a prompt, an agent, a retrieval index or
a pipeline.
**Returns:** score per case per criterion, each with its reason, plus the four
versions the score is only meaningful against: rubric, set, grader, and system
under test.

### `/eval calibrate`

Measure how well an automated grader agrees with human labels.

**When to use it:** before relying on a grader at scale, and periodically after.
**Returns:** agreement per criterion, the criteria where the grader is unreliable,
and the recommended weighting. Scores produced by an uncalibrated grader are
reported as such.

### `/eval compare <a> <b>`

Compare two versions with the uncertainty stated.

**When to use it:** whenever someone claims an improvement.
**Returns:** the difference per criterion, the estimated noise, and a verdict of
better, worse, or no detected difference. A difference inside the noise is never
reported as a win.

### `/eval regress [baseline]`

Detect regressions against the accepted baseline.

**When to use it:** on every change, and on a cadence for systems that change
underneath you.
**Returns:** what dropped, on which named cases, against which criteria, with the
baseline's date and version. It refuses to compare when the set has changed since
the baseline.

### `/eval report`

Report the trend and what it means.

**When to use it:** at review points, and when handing evidence to a release
decision.
**Returns:** movement per criterion over time, the outcome check where outcomes
exist, and the regressions routed to Quality & Evaluation {OS}. It contains no
ship recommendation, by design.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/eval rubric` | rubric | examples into criteria with anchors |
| `/eval set` | set | the fixed versioned cases, adversarial and refusal included |
| `/eval run` | run | score per case per criterion, with reasons |
| `/eval calibrate` | calibrate | grader agreement with human labels |
| `/eval compare` | compare | difference with its uncertainty |
| `/eval regress` | regress | what dropped, on which cases |
| `/eval report` | report | the trend, checked against outcomes |

No command in this OS issues a release verdict. That authority belongs to
Quality & Evaluation {OS}, and the separation is deliberate.
