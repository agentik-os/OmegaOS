# Habit Tracker {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install habit-tracker-os` | Installs this OS into your environment | Once, first |
| `agentik configure habit-tracker-os` | Collects the minimum context it needs | After install |
| `agentik run habit-tracker-os` | Starts the OS | Every session |
| `agentik doctor habit-tracker-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update habit-tracker-os` | Updates to the latest version | When a release lands |
| `agentik eval habit-tracker-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/habits` | Opens the OS and routes your message to the right mode | Default entry point when you do not know which command you want | The mode it entered and its first question |
| `/habit-setup` | Turns a behaviour contract into an observable, agreed habit record | Starting a habit, or receiving a contract from Mindset {OS}, Identity Shift {OS} or Goal & Life Strategy {OS} | A `DRAFT` contract with its definition, cue, schedule, minimum version and recovery rule |
| `/today` | Ranks at most seven primary items for today and explains the ranking | Every morning | The today card, read-only |
| `/checkin` | Records what happened as a typed, provenance-labelled log entry | Every evening, or right after the behaviour | The record written, and the next action |
| `/urge` | Runs the urge protocol before analysis | You are tempted and have not acted yet | A next move that does not depend on willpower alone |
| `/lapse` | Debriefs a miss without moral judgement and protects the next occurrence | You missed, or a chain broke | The antecedent recorded, and a plan for the next opportunity |
| `/habit-review` | Computes adherence, trend and confidence from the log | Weekly or monthly, or when you ask how you are doing | A review with metrics, one keep, change or stop decision, and the data gaps named |
| `/recover` | Enters or proposes a recovery season and shrinks the active set | You are overloaded, or Health & Energy {OS} reports capacity below the load | The `RECOVERING` season, the essentials kept, everything else `PAUSED` |
| `/adapt` | Creates a versioned experiment instead of rewriting a commitment | The plan does not fit, or a contract keeps failing | The experiment with success criteria, stopping rule and the version it supersedes |
| `/visualize` | Renders the smallest valid table or diagram for a trend | A number is not making the pattern obvious | A Mermaid diagram or a table, read-only |

### `/habits`

The default entry. Say what happened or what you want in plain language and the
session router picks the mode: setup, today, check-in, urge, lapse, review,
recover, adapt or visualize. Use it when you do not want to remember command
names.

```
/habits
/habits I skipped training again this week
```

**Returns:** the mode it selected, the state it loaded, and one question at
most.

### `/habit-setup`

Builds one habit contract. It will not mark a contract `ACTIVE` until the
behaviour is observable, a minimum viable version exists, and a lapse recovery
rule exists. For an unwanted habit it also requires a friction change and a
replacement response, because suppression alone is incomplete.

```
/habit-setup morning strength training, 3x per week
/habit-setup stop scrolling in bed
```

**Returns:** the contract in `DRAFT` with an ID, its definition and its
schedule. It becomes `ACTIVE` only when you agree.

### `/today`

Ranks the day. Never more than seven primary items, fewer when the season is
`RECOVERING` or the capacity envelope from Health & Energy {OS} is tight. Each
item carries the reason it is ranked where it is.

```
/today
```

**Returns:** the today card. Read-only: it writes nothing.

### `/checkin`

Converts one sentence into one typed record. "Done", "half of it", "missed, was
travelling" all work. What you state is labelled `explicit`. What a trusted
device imported is labelled `observed`. Anything the model concluded is
`inferred` and does not count as completion evidence until you confirm it.

```
/checkin done
/checkin did 15 minutes instead of 40
```

**Returns:** the record written, its provenance label, and the next action.

### `/urge`

For the moment before acting, not after. Latency matters more than analysis
here, so it responds with the protocol first and diagnoses afterwards.

```
/urge about to order takeaway again
```

**Returns:** a bounded next move, and on your agreement a friction or
replacement change to the contract.

### `/lapse`

For after the miss. It records the antecedent (what preceded it, where, with
whom, in what state) and protects the next opportunity. It does not score you
and it does not reset a streak as a punishment.

```
/lapse third miss this week
```

**Returns:** the debrief, and the protection applied to the next occurrence.
The debrief is offered to Journal {OS} as reflective material.

### `/habit-review`

Computes rather than guesses. Adherence, cue stability, recovery latency and
trend come from dated records. When there are too few records it says so and
gives the confidence rather than asserting a pattern.

```
/habit-review week
/habit-review month HAB-0003
```

**Returns:** the metrics, one keep, change or stop decision, the named data
gaps, and the `habit.review.completed` event that goes to Mindset {OS}.

### `/recover`

The seam with Health & Energy {OS}. It shrinks the active set to essentials and
sets the season to `RECOVERING`, with an exit condition. It is the correct
response to overload; adding a new habit is not.

```
/recover
/recover two weeks, keep sleep and walking only
```

**Returns:** the season, the essentials kept, the contracts moved to `PAUSED`,
and the condition that ends the season.

### `/adapt`

Changes are experiments, never silent rewrites. Every adaptation carries what
changes, what would count as success, when it stops, and which contract version
it supersedes. The previous version is retained so you can roll back.

```
/adapt drop the target from 5 to 3 per week for 3 weeks
```

**Returns:** the experiment record with its stopping rule, and the version it
replaces.

### `/visualize`

Picks the smallest representation that makes the trend legible: a table when a
table is enough, a Mermaid diagram when it is not. It refuses to render a chart
that implies more data than exists.

```
/visualize last 8 weeks
```

**Returns:** the diagram or table, read-only.

## Deterministic runtime

The coaching half runs in the agent. The durable half runs in `omega-habits`, a
standard-library Python CLI that owns the local indexed ledger at
`~/.omega/os/habits-os/ledger/habits.db` (override with `--db`). It ships with
the installed pack; run `omega-habits --help` for the current surface.

| Subcommand | Does |
|---|---|
| `omega-habits init` | create or update the user profile |
| `omega-habits add` | accept and version a habit contract |
| `omega-habits update` | revise a contract, keeping the previous version |
| `omega-habits list` | list contracts with their status |
| `omega-habits log` | append an `explicit` or `observed` evidence record |
| `omega-habits correct` | supersede a wrong log record, invalidating derived reviews |
| `omega-habits today` | rank today's primary habits, seven maximum |
| `omega-habits review` | compute an evidence-bounded review with confidence and data gaps |
| `omega-habits chart` | render a Mermaid progress diagram |
| `omega-habits context` | return compact context for the agent |
| `omega-habits export` | export user-owned state |
| `omega-habits season` | change the operating season |
| `omega-habits experiment` | create a bounded behaviour experiment |
| `omega-habits delete` | delete user-owned state |
| `omega-habits doctor` | validate database integrity |

The ledger is a projection built for fast lookups. Confirmed observations are
staged canonically through Context & Memory {OS}, which remains the source of
truth.

## Command summary

| Command | Does |
|---|---|
| `/habits` | opens the OS, routes your message to a mode |
| `/habit-setup` | builds an observable, agreed habit contract |
| `/today` | ranks at most seven items for today |
| `/checkin` | records what happened, with provenance |
| `/urge` | runs the urge protocol before acting |
| `/lapse` | debriefs a miss and protects the next occurrence |
| `/habit-review` | computes adherence, trend and confidence |
| `/recover` | enters a recovery season and shrinks the active set |
| `/adapt` | creates a versioned experiment with a rollback |
| `/visualize` | renders the smallest valid view of a trend |
| `omega-habits <sub>` | the deterministic ledger behind all of it |
