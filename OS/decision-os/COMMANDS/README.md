# Decision {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install decision-os` | Installs this OS into your environment | Once, first |
| `agentik configure decision-os` | Collects the minimum context it needs | After install |
| `agentik run decision-os` | Starts the OS | Every session |
| `agentik doctor decision-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update decision-os` | Updates to the latest version | When a release lands |
| `agentik eval decision-os` | Runs its evaluation suite | Before trusting it |

## OS commands

Six commands, in the order a real call moves through them. `/decide` runs the
whole sequence; the other five let you enter or re-enter at one stage.

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/decide` | Runs the full sequence: frame, options, reversibility, evidence, choice, record | Any hard call you intend to close | A decision record with a review trigger |
| `/frame` | Reduces the situation to one decision question, a deadline, an objective, constraints and criteria | The call feels impossible and has never been written down | The frame, in one sentence, plus sourced criteria |
| `/options` | Generates at least three options including doing nothing and the unsaid one | The choice has collapsed into a binary | An option table with costs, downsides and second-order effects |
| `/reversibility` | Classes each option and prices the undo | Before committing to anything that may be one-way | Reversible, costly to reverse, or irreversible, per option, with undo cost |
| `/decision-record` | Writes the record of what was decided and why | At the moment of the call | The record: choice, rationale, discarded options, evidence, signal weight, review trigger |
| `/decision-review` | Grades an existing record against what happened | When the review trigger fires or the outcome lands | One of four verdicts, appended to the record |

### `/decide`

The main entry point. It runs `/frame`, `/options`, `/reversibility`, the
evidence check, then `/decision-record`, in that order, stopping wherever the
inputs run out rather than guessing past them. It pulls weighted values from
Alignment {OS}, the objective from Goal & Life Strategy {OS}, and any signal
from Intuitive {OS} with its calibration weight, and names any source that is
absent.

```
/decide renew the contract in March or let it end
```

It never proceeds through an irreversible option without asking, and it never
records a decision without a review trigger.

### `/frame`

The step most people skip and the one that resolves the most calls on its own.
It asks what the actual question is, when it truly must be answered, which
objective it serves, what the real constraints are, and which criteria it will
be scored against. Criteria come from Alignment {OS} where that unit is
installed; anything invented here is marked unsourced.

```
/frame I keep thinking about leaving but I cannot say what I would be leaving for
```

**Returns:** one sentence the user has to accept before anything else runs. If
they will not accept it, the frame is wrong and the session stays here.

### `/options`

Produces at least three genuine options. Doing nothing is always one of them,
with its cost stated, because an unnamed default wins by silence. It asks
directly for the option the user has been avoiding saying out loud, and it runs
a pre-mortem for each serious candidate: assume this failed, say why.

```
/options
```

**Returns:** an option table, each row with a cost, a downside and one
second-order effect. Fewer than three real options is reported as a false binary
and sends the session back to `/frame`.

### `/reversibility`

Classes every option as reversible, costly to reverse, or irreversible, and
prices what undoing it would take in time, money and relationships. This is what
sets the evidence bar: a reversible call is decided fast on thin evidence and
reviewed early, an irreversible one has to meet the stated threshold and get
explicit human approval.

```
/reversibility
```

**Returns:** the class and undo cost per option. An option whose undo cost
cannot be established is treated as irreversible until it can.

### `/decision-record`

Writes what was decided and, more importantly, what was believed at the time:
the choice, the rationale, the options discarded and why, the evidence relied
on, the intuition signal and the weight it was given (including when it was
overruled), the reversibility class, and the review trigger with its date. It
asks for approval before persisting, and it refuses to mark a call decided with
no review trigger.

```
/decision-record
```

**Returns:** the record, stored in Context & Memory {OS} and mirrored to Journal
{OS} as a dated entry.

### `/decision-review`

Grades an existing record against what actually happened. It reads the record
first and never edits it: a review appends. The verdict is one of four: held,
wrong for the reason predicted, wrong for a reason not predicted, or still open
with a new trigger. The third verdict is the valuable one, because it is the
only one that names a blind spot.

```
/decision-review march-contract
```

**Returns:** the verdict, the lesson, and where it goes next: Intuitive {OS}
reads the resolved outcome to score its calibration, Goal & Life Strategy {OS}
hears about a changed allocation.

## Command summary

| Command | Does |
|---|---|
| `/decide` | the full sequence, frame through record |
| `/frame` | one question, one deadline, sourced criteria |
| `/options` | three or more real options, doing nothing included |
| `/reversibility` | class per option and the price of undoing it |
| `/decision-record` | what was decided, and what was believed at the time |
| `/decision-review` | one of four verdicts, appended, never rewritten |
