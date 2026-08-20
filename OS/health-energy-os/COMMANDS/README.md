# Health & Energy {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install health-energy-os` | Installs this OS into your environment | Once, first |
| `agentik configure health-energy-os` | Collects the minimum context it needs | After install |
| `agentik run health-energy-os` | Starts the OS | Every session |
| `agentik doctor health-energy-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update health-energy-os` | Updates to the latest version | When a release lands |
| `agentik eval health-energy-os` | Runs its evaluation suite | Before trusting it |

## OS commands

Ten commands resolve onto the seven modes in `OS.md` section 3. They are
routing modes inside this OS, not separately installed skills. Routing priority
is fixed: the safety, legal and privacy boundary first, then an explicit
command, then user intent, then what evidence is available, then the cheapest
reversible action, then a handoff when another OS owns the next step.

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/health` | Opens the OS and routes your message to a mode | Default entry point when you do not know which command you want | The mode it entered and its first question |
| `/readiness` | Assesses what you can carry today | Every morning, and before committing to a heavy day | A capacity call with the limiting factor, the constraints, the validity window and the confidence |
| `/health-audit` | Builds the standing capacity baseline | The first session, and after anything that changes the body's situation | The baseline, its bottleneck and the evidence label on each claim |
| `/sleep` | Audits sleep and circadian constraints | Sleep is short, broken, mistimed, or is the suspected bottleneck | The sleep audit with the one change most likely to move it |
| `/training` | Builds or revises training | Starting, restarting, plateauing, or the plan no longer fits the capacity | The minimum effective plan, with its overload and recovery balance and a review trigger |
| `/nutrition` | Reviews fuel and adherence | Energy crashes, weight or performance drift, or a plan that is not being followed | The pattern review and one change, with adherence treated as part of the plan |
| `/recovery` | Responds to fatigue or overload | Readiness stays low, you are run down, ill or injured | A recovery prescription and a load reduction sent to Habit Tracker {OS} and Execution {OS} |
| `/travel-health` | Designs a travel and jet-lag protocol | A trip or a time-zone change is coming | The protocol per travel day: light, sleep, fuel, movement |
| `/health-experiment` | Creates an N-of-1 experiment | A question about your body that general evidence cannot answer | The experiment: one variable, the measure, the duration, the stopping rule, the rollback |
| `/wearable` | Interprets device trends conservatively | A score or trend needs reading without being over-read | The interpretation with its evidence label and its limits |

### `/health`

The default entry. Describe the situation in plain language and the router
picks the mode. The safety gate runs before the routing, so a red flag in your
message stops the session and produces the escalation instead of a plan.

```
/health
/health slept badly all week and training feels heavy
```

**Returns:** the mode selected, the context loaded, and one question at most.

### `/readiness`

The daily capacity call. It reads your report and any trusted device data, then
states what you can carry today, what limits it, and how long the assessment is
good for. It gives a ceiling, not an encouragement.

```
/readiness
/readiness 5h sleep, HRV down, meeting-heavy day
```

**Returns:** the capacity level, the limiting factor, today's constraints, the
validity window and the confidence. The same envelope goes to Execution {OS} as
`handoff.execution.capacity`.

### `/health-audit`

Builds the baseline the other commands depend on. Without it, `/readiness`
gives a low-confidence ceiling rather than a number. It separates the
bottleneck from the noise and labels each claim E1 through E5.

```
/health-audit
```

**Returns:** the standing capacity baseline, its bottleneck, and what would
resolve the largest remaining unknown.

### `/sleep`

Sleep is a foundational input, not an optional reward, so it gets its own
audit: duration, timing, continuity, light exposure, and the constraints you
cannot change. It proposes one change at a time.

```
/sleep
/sleep waking at 3am most nights
```

**Returns:** the audit and the single highest-leverage change, with its review
trigger.

### `/training`

Builds the smallest plan that produces the adaptation, then adds complexity
only after adherence holds. It will not prescribe a load above the current
capacity envelope, and it names both the overload and the recovery.

```
/training 3 sessions per week, no gym access
```

**Returns:** the plan, its progression, its recovery, and the trigger that
brings it back for revision.

### `/nutrition`

Reviews the pattern rather than counting for you: what is actually eaten, when,
and whether the plan is being followed. Adherence is treated as a property of
the plan, not a property of the person.

```
/nutrition energy crashes at 3pm
```

**Returns:** the pattern review and one change. Restriction protocols and
aggressive cuts require explicit approval and stop at the safety gate when a
disordered-eating signal is present.

### `/recovery`

The response to overload. It prescribes recovery and it issues the load
reduction: Habit Tracker {OS} receives the directive to shrink the active set,
Execution {OS} receives the workload constraint. It also sets the condition
that ends the recovery period.

```
/recovery
/recovery run down for ten days, second cold this month
```

**Returns:** the recovery prescription, the load reduction issued, and the exit
condition. Lifting a recovery directive later requires your explicit approval.

### `/travel-health`

Bounded to the trip. It plans light, sleep timing, fuel and movement across the
travel days and the adjustment days, and it tells Habit Tracker {OS} which
routines are suspended for the duration.

```
/travel-health Paris to Tokyo, 6 days, arriving evening
```

**Returns:** the day-by-day protocol and the routines suspended.

### `/health-experiment`

For questions general evidence cannot answer about your body. One variable, one
measure, a duration, a stopping rule and a rollback, all written down before
the experiment starts. An experiment without a stopping rule is not started.

```
/health-experiment caffeine cutoff at 12:00 for 3 weeks
```

**Returns:** the experiment record, and the date it will be reviewed or
stopped.

### `/wearable`

Reads a device trend conservatively: trend, context and function, never a
single daily score, and never a diagnosis. It says what the number cannot tell
you as clearly as what it can.

```
/wearable HRV dropped 20% this week
```

**Returns:** the interpretation, its evidence label, and its limits.

## Deterministic runtime

The pack ships a provider-neutral reference runtime, standard library only. It
proves the pack is self-describing and integrity-checkable. It is not a
production database, not an LLM adapter and not a security layer.

| Subcommand | Does |
|---|---|
| `python runtime/os_runtime.py info` | inspect the OS descriptor |
| `python runtime/os_runtime.py route /health` | resolve a command to its mode |
| `python runtime/os_runtime.py event note '{...}'` | record a structured event |
| `python runtime/os_runtime.py validate` | check package integrity against `MANIFEST.json` |

## Command summary

| Command | Does |
|---|---|
| `/health` | opens the OS, routes your message to a mode |
| `/readiness` | states what you can carry today, and what limits it |
| `/health-audit` | builds the standing capacity baseline |
| `/sleep` | audits sleep and circadian constraints |
| `/training` | builds or revises the training plan |
| `/nutrition` | reviews fuel and adherence |
| `/recovery` | prescribes recovery and issues the load reduction |
| `/travel-health` | designs the travel and jet-lag protocol |
| `/health-experiment` | creates an N-of-1 experiment with a stopping rule |
| `/wearable` | interprets a device trend conservatively |
| `os_runtime <sub>` | inspects, routes, records and validates the pack |
