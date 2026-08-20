# Trend & Opportunity {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install trend-opportunity-os` | Installs this OS into your environment | Once, first |
| `agentik configure trend-opportunity-os` | Collects the minimum context it needs | After install |
| `agentik run trend-opportunity-os` | Starts the OS | Every session |
| `agentik doctor trend-opportunity-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update trend-opportunity-os` | Updates to the latest version | When a release lands |
| `agentik eval trend-opportunity-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/trend [<domain or claim>]`

The root command. With no argument it reports the state of every watch: what was
swept, when, what is still one observation, what is confirmed, and which
opportunities are approaching their expiry. Given a domain it opens a watch.
Given a claimed trend ("agents are replacing RPA") it runs the date, independence
and interest checks and tells you what you actually have.

**When to use it:** at the start of any session, and whenever somebody says
something is taking off.
**Returns:** either the watch board, a new watch entry awaiting sources, or a
verdict on the claim: signal, candidate, or confirmed movement, with the count of
dated observations behind it.

### `/watch <domain>`

Open or edit a watch. Forces the behaviour statement before any collection: what
would have to change in the world for this trend to be real, stated so you could
go and look for it.

```
/watch "small teams replacing tool Y in production"
/watch --list
/watch --edit <watch-id> --cadence monthly
```

**When to use it:** the moment a domain starts mattering to a decision, and long
before you want an answer about it.
**Returns:** the watch entry: domain, target behaviour, sources with their
interest labels and access constraints, cadence, next review date. It refuses to
open a watch whose behaviour statement is an attention measure.

### `/signals [<watch-id>]`

Capture and list dated observations. Every capture demands the observation date,
the capture date, the source and locator, and the interest label. It refuses a
signal it cannot date.

```
/signals <watch-id>                        # list what is captured
/signals <watch-id> --add                  # capture a new observation
/signals <watch-id> --since 2026-01-01
/signals --quarantined                     # signals missing a date or locator
```

**When to use it:** on every sweep, and any time you read something relevant.
**Returns:** the signal list with both dates per entry, the interest label, and
the independence grouping, so five reports citing one press release appear as
one observation and not five.

### `/movement <watch-id>`

Run the confirmation test on a watch's candidates: enough dated observations,
spread over time, independent sources, at least one disinterested source, a
direction AND a rate over a stated window.

**When to use it:** when signals have accumulated and somebody is about to call
it a trend.
**Returns:** for each candidate, either a confirmed movement record (direction,
rate, measurement window, supporting signals, contradicting signals) or a
refusal naming exactly what is missing: the number of observations, the
independent source, the disinterested source, or the rate. A refusal is a normal
outcome and is not softened. Confirmation emits `trend.movement.confirmed`.

### `/opportunity <movement-id>`

Name an opportunity from a confirmed movement. Demands all four parts: who can
act, what changed, why now, what window and expiry.

**When to use it:** once movement is confirmed and a specific actor could
actually do something about it.
**Returns:** the opportunity record with its closing condition, its expiry date
and its owner. It refuses to name an opportunity with no expiry date, and it
refuses one whose actor cannot reach it. Emits `opportunity.named`.

### `/window <opportunity-id>`

Inspect or restate the window: what closes it, how much time is left, what has
happened since it was named, and what would move the expiry.

```
/window <opportunity-id>
/window --closing-within 60d
```

**When to use it:** before committing anything to an opportunity, and whenever
someone plans work that lands after the expiry date.
**Returns:** the closing condition, the expiry date, days remaining, and the
dated evidence for or against the window still being open.

### `/review [<watch-id> | <opportunity-id>]`

Run the scheduled review: sweep the sources again, capture what is new, and
restate every open opportunity as still open, narrowed, widened or closed.

**When to use it:** on the cadence you set, and never later than an expiry date.
**Returns:** the review record: what was swept, what was found, the null results
stated with their dates, and the state change of every opportunity reviewed.

### `/retire <opportunity-id>`

Close an opportunity and write the record: what closed, when, why, what was
learned, and who was betting on it.

**When to use it:** when the closing condition fires, when the expiry passes,
when the movement reverses, or when the opportunity has been taken.
**Returns:** the retirement record and the list of downstream units notified.
Emits `opportunity.window.closed`. Retiring an opportunity another OS is already
betting on requires approval first.

### `/trend-audit <artifact or watch-id>`

Inspect a claimed trend, a watch, or an inherited deck. Checks dates, source
independence, source interest, attention versus behaviour, and whether any rate
was ever computed.

**When to use it:** when you inherit a trend report, when a watch has been quiet
for several cycles, and before anyone builds a plan on a slide.
**Returns:** a defect list per record: undated signals, collapsed sources,
interested-only bodies of evidence, attention presented as adoption, rates
asserted without a window, opportunities past their expiry that are still being
treated as open.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/trend` | entry point: watch board, new watch, or a check on a claimed trend | board, watch entry, or signal / candidate / confirmed verdict |
| `/watch` | open or edit a watch, behaviour statement first | watch entry with sources, interest labels and cadence |
| `/signals` | capture and list dated observations | signal list with both dates, interest labels and independence grouping |
| `/movement` | run the confirmation test on candidates | movement record with direction and rate, or a refusal naming what is missing |
| `/opportunity` | name an opportunity from confirmed movement | who, what changed, why now, window, expiry, owner |
| `/window` | inspect or restate a closing window | closing condition, expiry, days left, dated evidence |
| `/review` | scheduled sweep and restatement | review record, including null results with dates |
| `/retire` | close an opportunity with a record | retirement record and downstream notifications |
| `/trend-audit` | forensic check on a claimed trend or a drifting watch | defect list per record |
