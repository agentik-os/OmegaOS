# Intuitive {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install intuitive-os` | Installs this OS into your environment | Once, first |
| `agentik configure intuitive-os` | Collects the minimum context it needs | After install |
| `agentik run intuitive-os` | Starts the OS | Every session |
| `agentik doctor intuitive-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update intuitive-os` | Updates to the latest version | When a release lands |
| `agentik eval intuitive-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/intuition` | Opens the OS, lists due and overdue signals, and answers what your gut is worth right now on a named call | You are about to decide something and want the signal plus its track record | the signal, its disconfirmer, the domain tier and weight, or the label `uncalibrated` |
| `/signal-log` | Captures one signal as an immutable falsifiable prediction | You have a pull, a doubt or a certainty and the outcome is not yet known | the written signal record, with its resolution date and the confirmation that it is now immutable |
| `/signal-resolve` | Resolves a due signal against its pre-registered condition | The resolution date arrived, or the outcome landed early | the verdict (hit, miss, partial, unresolvable), the Brier score, and the domain tier after the update |
| `/calibration` | Recomputes and prints the calibration record per domain | You want to know whether your intuition is actually any good, and where | per domain: resolved count, misses, hit rate, mean Brier, skill against base rate, tier, weight, staleness |
| `/intuition-review` | Runs the monthly pass: overdue chase, tier changes, capture-quality defects | A month closed, or unresolvable signals are piling up | the calibration report, the closed overdue signals, and any domain flagged for a capture defect |

### `/intuition`

The entry point. With no argument it loads the log, lists every signal that is
due or overdue, and prints the current tier of each domain. With a call named
as an argument it runs `CONSULT`: it produces the signal, its disconfirmer, and
the weight the domain has earned.

It never recommends an option and never ranks options. That is Decision {OS}
(`decision-os`), and this command exists to hand that unit one input.

```
/intuition
/intuition "should I take the enterprise contract"
```

**Returns:** the signal packet, or the label `uncalibrated` with the resolved
count and how many more resolutions the domain needs to reach `provisional`.

### `/signal-log`

Captures a signal. It will not write the record without five things: the claim
rewritten into an observably true or false statement, the domain, a confidence
from 0 to 100, the base rate for outcomes like this one, and the disconfirmer.
It then sets the resolution condition and date.

It refuses an unfalsifiable claim. "I feel like this will not work out" names
no observable, so it offers two candidate rewrites and asks you to pick or edit
one. A signal offered after the outcome is already known is written marked
`retrospective`, and is never scored.

```
/signal-log "this hire will not last the probation period"
/signal-log "the deal closes" --confidence 75 --base-rate 40 --resolve-by 2026-11-30
```

**Returns:** the record, its capture timestamp, its resolution date, and a
statement that it is now immutable. Editing a captured signal is inside the
human approval boundary.

### `/signal-resolve`

Resolves a due signal. It reads the recorded claim, disconfirmer and resolution
condition back to you first, before you describe what happened. That ordering
is deliberate: judging the outcome against a claim you have just reinterpreted
is the failure mode this whole unit exists to prevent.

Four verdicts are legal: hit, miss, partial, unresolvable. `unresolvable`
requires a reason and costs the domain nothing except a count. A contested
outcome is held unresolved and routed to human approval.

```
/signal-resolve
/signal-resolve SIG-2026-041 --verdict miss
```

**Returns:** the verdict, the Brier score for that signal, and the domain tier
recomputed after it.

### `/calibration`

Prints the record. Per domain: resolved count, number of recorded misses, hit
rate, mean Brier score under the nine month recency half-life, skill against
the base rate reference, the tier, the weight, and the age of the newest
resolution.

The tiers and what they rest on:

| Tier | Condition | Weight |
|---|---|---|
| `uncalibrated` | fewer than 12 resolved signals, or fewer than 3 recorded misses | 0 |
| `provisional` | 12 to 29 resolved, positive skill against the base rate | 0.25 |
| `calibrated` | 30 or more resolved, skill 0.10 or better across the most recent 20 | 0.5 |
| `counter-indicative` | 20 or more resolved, skill below 0 | 0, plus an explicit flag |

A domain whose newest resolution is older than 12 months is marked `stale` and
its weight is halved. Past 24 months it reverts to `uncalibrated`. 0.5 is the
hard ceiling: a calibrated signal moves a decision, it never decides one.

```
/calibration
/calibration --domain hiring
```

**Returns:** the table above, per domain, with the count every tier rests on
printed beside it. A tier without its count is a defect.

### `/intuition-review`

The monthly pass. It chases every overdue signal, closes anything open for two
review cycles as `unresolvable` with reason `outcome never observed`, reports
every tier change with the evidence that caused it, and computes the
unresolvable rate per domain.

Any domain over 30 percent unresolvable is flagged with a capture defect and
blocked from promotion. The correction there is sharper capture, not more
signals, so the command runs a capture-quality pass over that domain's last ten
records instead of asking for new ones.

```
/intuition-review
/intuition-review --month 2026-07
```

**Returns:** the calibration report, the resolutions closed, the tier changes,
the capture-quality defects, and the packet sent to Journal {OS} for pattern
extraction.

## Command summary

| Command | Does |
|---|---|
| `/intuition` | opens the OS, lists due signals, answers what your gut is worth here |
| `/signal-log` | captures one falsifiable signal, with its disconfirmer and date |
| `/signal-resolve` | resolves a due signal against what was written, not what you now recall |
| `/calibration` | the per domain record: count, hit rate, Brier, skill, tier, weight |
| `/intuition-review` | monthly: overdue chase, tier changes, capture-quality defects |
