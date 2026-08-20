# Growth {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install growth-os` | Installs this OS into your environment | Once, first |
| `agentik configure growth-os` | Collects the minimum context it needs | After install |
| `agentik run growth-os` | Starts the OS | Every session |
| `agentik doctor growth-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update growth-os` | Updates to the latest version | When a release lands |
| `agentik eval growth-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/growth` | Opens the growth view | none | live loops, running experiments, the top of the backlog |
| `/loop-map` | Maps the loops and separates them from funnels | performance data from the owning units | a loop map with cycle time, coefficient and cost per loop |
| `/loop-audit` | Audits one loop step by step for its weakest link | a loop | the weakest step with the evidence behind the ranking |
| `/hypothesis` | Turns an observation into a falsifiable hypothesis | an observation, a loop step | a hypothesis with mechanism and expected effect size |
| `/backlog` | Ranks the experiment backlog | hypotheses | a ranking by information per unit of cost and time |
| `/experiment-design` | Fixes threshold, sample, duration, stopping rule and guardrail | a selected hypothesis | a pre-registered design, immutable once running |
| `/experiment-run` | Registers the experiment as running and names its owning OS | an approved design | a running record and the change proposal for the owner |
| `/experiment-read` | Reads the result against the pre-registered threshold | a stopped experiment | a verdict, or an underpowered report |
| `/channel-economics` | Scores a channel on cost per retained cohort | a channel, a cohort window | acquisition cost against retained revenue |
| `/scale` | Proposes scaling a channel to its owning OS | a positive verdict | a scale proposal with its spend ceiling |
| `/kill` | Records the decision to stop a channel | a channel, evidence | a kill record |
| `/growth-review` | Reviews loops, experiments and channels on a cadence | none | what compounds, what leaks, what should stop |

---

## Structure

### `/growth`

The default view: which loops are live, which experiments are running, what is
at the top of the backlog, and which experiments are waiting on an approval.

```bash
/growth
```

**When to use it:** at the start of any growth session.
**Returns:** running experiments first, each with the date its stopping rule
fires. An experiment with no pre-registered threshold is flagged before
anything else is shown.

### `/loop-map`

Pulls performance data from Content {OS}, Sales {OS}, Revenue {OS} and Delivery
& Customer Success {OS} and traces each candidate loop's output back to its own
input.

```bash
/loop-map
```

**When to use it:** first, before any experiment, and again whenever the
business changes shape.
**Returns:** each loop with its cycle time, coefficient and cost. Anything that
does not close is listed separately, under the heading funnel, with the step at
which it fails to close.

### `/loop-audit <loop>`

Walks one loop step by step and ranks the steps by their effect on cycle time
and coefficient.

```bash
/loop-audit referral
```

**When to use it:** once a loop is worth improving.
**Returns:** the weakest step, the size of its effect, and the data that
supports the ranking.

---

## Experiments

### `/hypothesis <observation>`

Turns an observation into something that can be wrong.

```bash
/hypothesis "trial users who invite a teammate retain better"
```

**When to use it:** whenever a plausible-sounding claim appears in a review.
**Returns:** the loop step it touches, the mechanism it proposes, the expected
effect size, and the cheapest test that could falsify it.

### `/backlog`

Ranks hypotheses by information gained per unit of cost and time, which
deliberately favours experiments that can be killed cheaply.

```bash
/backlog
```

**Returns:** the ranked list with the ranking reason per entry.

### `/experiment-design <hypothesis>`

Fixes the target metric, the guardrail metric, the sample, the duration, the
success threshold and the stopping rule. Metric definitions are pulled from KPI
& Analytics {OS}, not restated.

```bash
/experiment-design h-014
```

**When to use it:** before anything is changed anywhere.
**Returns:** the pre-registered design. Once `/experiment-run` accepts it, the
threshold is immutable and any attempt to edit it is refused and logged.

### `/experiment-run <design>`

Registers the experiment as running and produces the change proposal for the
owning OS. It does not apply the change.

```bash
/experiment-run e-014
```

**When to use it:** after the design and, where required, the human approval.
**Returns:** the running record, the change proposal, and the name of the OS
that must apply it under its own approval gate.

### `/experiment-read <experiment>`

Reads a stopped experiment against the threshold fixed beforehand.

```bash
/experiment-read e-014
```

**When to use it:** when the stopping rule fires or the duration ends, not
before.
**Returns:** the verdict, both target and guardrail movements, and the sample
actually achieved. Below the pre-registered minimum it returns underpowered and
no verdict, naming the sample that would have been needed.

---

## Channels

### `/channel-economics <channel>`

Scores a channel on acquisition cost against the revenue its cohorts actually
retained, using Revenue {OS} and Delivery & Customer Success {OS} data.

```bash
/channel-economics paid-search --cohort 2026-Q1
```

**When to use it:** before scaling anything, and on every review.
**Returns:** cost per retained cohort, and an explicit provisional flag when
the retention window has not yet closed.

### `/scale <channel>`

Proposes scaling to the owning OS, with a spend ceiling attached.

```bash
/scale newsletter-sponsorships
```

**Returns:** a scale proposal. Any spend requires human approval before it is
handed on.

### `/kill <channel>`

Records the decision to stop a channel, with the economics behind it.

```bash
/kill cold-outbound --reason "cost per retained cohort above LTV for three quarters"
```

**When to use it:** as soon as the evidence is in. Killing is a normal outcome.
**Returns:** a kill record, durable enough to answer the next person who wants
to try the same channel again.

### `/growth-review`

Reviews loops, running experiments and channel economics on the operator's
cadence.

```bash
/growth-review
```

**Returns:** what compounds, what leaks, what should stop, and which backlog
entries the last quarter's verdicts made obsolete.

---

## Command summary

| Command | Does |
|---|---|
| `/growth` | loops, running experiments, top of backlog |
| `/loop-map` | maps loops, names funnels as funnels |
| `/loop-audit` | finds the weakest step in one loop |
| `/hypothesis` | turns an observation into something falsifiable |
| `/backlog` | ranks by information per unit of cost |
| `/experiment-design` | pre-registers threshold, sample, stopping rule, guardrail |
| `/experiment-run` | registers the run, hands the change to its owner |
| `/experiment-read` | reads against the fixed threshold, or reports underpowered |
| `/channel-economics` | cost per retained cohort |
| `/scale` | proposes scaling, with a ceiling |
| `/kill` | records why a channel stopped |
| `/growth-review` | the cadence review |
