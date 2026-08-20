# KPI & Analytics {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install kpi-analytics-os` | Installs this OS into your environment | Once, first |
| `agentik configure kpi-analytics-os` | Collects the minimum context it needs | After install |
| `agentik run kpi-analytics-os` | Starts the OS | Every session |
| `agentik doctor kpi-analytics-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update kpi-analytics-os` | Updates to the latest version | When a release lands |
| `agentik eval kpi-analytics-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/kpi-select` | Chooses the small set | the decisions somebody makes repeatedly | the metrics that would change a decision, and the list of what to stop tracking |
| `/kpi-define` | Makes a metric computable | a selected metric | formula, authoritative source, filters, period, cadence, owner, and the explicit counting rules |
| `/kpi-threshold` | Attaches the decision | a defined metric | the threshold, the decision it triggers, and the owner of that decision, all agreed before the number is seen |
| `/kpi-instrument` | States what must be captured | a definition the data cannot yet satisfy | the fields and events required, handed to Automation {OS}, and the metric marked not yet computable |
| `/kpi-read` | Runs the reading cycle | the current period | value, movement, comparison to history, and whether the movement exceeds normal variation |
| `/kpi-decide` | Handles a breach | a crossed threshold | the pre-agreed decision presented to its owner, taken or deferred with a reason |
| `/kpi-audit` | Judges the metric set | the last several cycles | for each metric, the decisions it changed; kept, redefined or retired, with vanity metrics named |
| `/kpi-reconcile` | Resolves disagreeing numbers | two sources reporting the same quantity | both values, the authoritative source, the counting rule that differs, and who owns the fix |

### When to reach for which

- `/kpi-select` before anything else, and again whenever the list has grown past
  one page.
- `/kpi-define` before a metric is ever reported. An undefined metric produces a
  meeting about the number instead of a decision from it.
- `/kpi-threshold` before the first reading, never after.
- `/kpi-read` on the cadence the number can actually move, not on the cadence
  somebody wants to look.
- `/kpi-audit` on the review cycle. It is the command that keeps the set small.

## What this OS refuses to do

It does not narrate noise, does not interpolate a missing reading, does not
produce a number a human typed in, and does not build the pipeline that collects
the data. The last of those belongs to Automation {OS}.

## Command summary

| Command | Does |
|---|---|
| `/kpi-select` | the few numbers that change decisions |
| `/kpi-define` | a definition a stranger can compute |
| `/kpi-threshold` | the decision, agreed before the number |
| `/kpi-instrument` | what must be captured, and by whom |
| `/kpi-read` | values, movement, and noise separated |
| `/kpi-decide` | the pre-agreed decision on a breach |
| `/kpi-audit` | keeps the set small and honest |
| `/kpi-reconcile` | settles two disagreeing sources |
