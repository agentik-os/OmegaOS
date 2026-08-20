# AI Logic {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install ai-logic-os` | Installs this OS into your environment | Once, first |
| `agentik configure ai-logic-os` | Collects the minimum context it needs | After install |
| `agentik run ai-logic-os` | Starts the OS | Every session |
| `agentik doctor ai-logic-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update ai-logic-os` | Updates to the latest version | When a release lands |
| `agentik eval ai-logic-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The OS answers to `/ai-logic` and its aliases `/ailogic` and `/ai-logic-os`. In
a terminal it is `omega-ailogic`.

### `/ai-logic arbitrate <step>`

Put one named step in exactly one bin: codify, augment, keep human, or delete.

**When to use it:** somebody is about to write a model call, and you want to know
whether a rule would do the job.
**Returns:** the bin, the reason in one line, the falsifier the step needs if it
stays a model call, and the cost comparison against the deterministic version.

### `/ai-logic map <process>`

Turn a process that lives in people's heads into numbered steps, each with an
owner and a duration, exceptions included.

**When to use it:** before any triage, and any time two people describe the same
process differently.
**Returns:** the numbered real process, the steps nobody owns, and the exceptions
that were being hidden behind the happy path.

### `/ai-logic baseline <process>`

Establish volume, time, error rate and cost as they are today, or specify the
measurement device that would produce them.

**When to use it:** whenever an improvement is being claimed and no numbers
exist. This is not optional; triage refuses to run without it.
**Returns:** the four numbers with the date and method, or a measurement device
specification and an explicit refusal to score.

### `/ai-logic triage`

Bin every step of a mapped and measured process, deletions announced first.

**When to use it:** after `map` and `baseline`, before anyone designs anything.
**Returns:** the four bin table with one line of justification per step, and the
total time recovered by the delete bin alone.

### `/ai-logic math <candidate>`

Annual gain against build plus maintenance, with every input visible.

**When to use it:** before approving any build, and again when someone claims an
automation has paid for itself.
**Returns:** the arithmetic, the verdict, and the specific number that would have
to change for the verdict to flip. Nothing that is not already in production is
counted.

### `/ai-logic verifier <output>`

Name the falsifier a consequential model output must carry.

**When to use it:** any time a model output leads to an action with a
consequence.
**Returns:** the concrete check (deterministic assertion, schema, citable source,
or a human rejection in seconds), or a refusal of the step when none exists.

### `/ai-logic challenge <system>`

Interrogate an existing agentic system against five questions, each finding
cited.

**When to use it:** a pipeline is expensive, unreliable or drifting, and nobody
can say where.
**Returns:** five answers in order (model doing a conditional's job,
unverified consequential output, ungated irreversible action, missing feedback
loop, absent primitive), each with a file and line, a rule, or a log entry, plus
the costliest gap first.

### `/ai-logic spec`

Specify the first approved move only, build ready.

**When to use it:** once a move has survived triage and arithmetic.
**Returns:** one move with its owner, inputs, done test, rollback and the
handoff target. Deliberately refuses to specify the second move.

## Command summary

| Command | Does |
|---|---|
| `/ai-logic arbitrate <step>` | one step, one bin, one reason |
| `/ai-logic map <process>` | the real process, numbered and owned |
| `/ai-logic baseline <process>` | the four numbers, or the device that measures them |
| `/ai-logic triage` | every step binned, deletions first |
| `/ai-logic math <candidate>` | gain against build plus maintenance |
| `/ai-logic verifier <output>` | the falsifier, or a refusal |
| `/ai-logic challenge <system>` | five cited findings against an agentic system |
| `/ai-logic spec` | the first move only, build ready |

Every command ends with what is not recommended, and why. That section is never
empty.
