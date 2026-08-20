# Mindset {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install mindset-os` | Installs this OS into your environment | Once, first |
| `agentik configure mindset-os` | Collects the minimum context it needs | After install |
| `agentik run mindset-os` | Starts the OS | Every session |
| `agentik doctor mindset-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update mindset-os` | Updates to the latest version | When a release lands |
| `agentik eval mindset-os` | Runs its evaluation suite | Before trusting it |

## OS commands

Two surfaces. The agent commands (`/mindset-os`, `/mindset`) run the coaching
loop. The `omega-mindset` binary owns the deterministic artifacts on disk: it
creates and validates files, and it never coaches.

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/mindset-os` | Opens the full coaching loop and routes the request to one mode | Any mindset, identity, philosophy, discipline or life-audit request | The mode it selected, and the artifact that mode promises |
| `/mindset` | The same OS, short form | Daily use, once you know the OS | Identical to `/mindset-os` |
| `omega-mindset new` | Creates the 19-file editable Markdown workspace | Once, at the start, and again only for a deliberate new cycle | The paths written, and the file to open first |
| `omega-mindset score` | Validates and summarizes one weekly scorecard JSON | Every time a week closes | State average, lowest and highest domain, promise kept-or-repaired rate |
| `omega-mindset challenge` | Creates the 180-day identity challenge workspace | When the user commits to a season, not a week | 180 daily, 26 weekly and 6 monthly follow-up files, plus `state.json` and `coaching/` |
| `omega-mindset coach` | Runs one AI coaching pass over the latest follow-ups, and can arm or disarm the daily loop | After follow-ups have been filled, or on a cadence you arm deliberately | A dated coaching file under `coaching/`, and a short card |

### `/mindset-os`

The coaching surface. It reads the request, picks one mode from OS.md section 3,
and produces that mode's artifact. It labels every claim E1 established, E2
promising, S spiritual, P personal or C clinical, and it stops and routes to a
qualified professional the moment a clinical or crisis signal appears.

It starts with safety, then baseline, then identity, then exactly one keystone
adjustment. Everything it decides not to do now goes onto the `NOT NOW` list
rather than into the plan.

```
/mindset-os I keep starting things and never finishing them
/mindset-os audit my life, I do not know where I stand
/mindset-os compile my personal philosophy
```

### `/mindset`

The same OS under a shorter name, for daily use. There is no behavioural
difference. Use `/mindset-os` in writing where the unit needs to be unambiguous.

### `omega-mindset new --name "<name>" --output <dir>`

Creates the editable workspace: 19 files covering the start guide, the identity
constitution, the 90-day plan, the daily card, the weekly scorecard JSON, the
identity and decision ledgers, the `NOT NOW` list, the reset protocols, the
personal philosophy, the written goals ledger, the simple disciplines, the Rohn
journal, the association and input audit, the seasons map, marketplace value,
financial philosophy, the lifestyle now-and-later design, and the 90-day Rohn
program.

The files are yours to edit. The OS reads them back; it does not own their
prose.

```bash
omega-mindset new --name "You" --output ~/mindset
```

**When to use it:** once, before the first coaching session, so the session has
somewhere durable to land.
**Returns:** the list of files written. It refuses to overwrite an existing
workspace unless you ask for replacement explicitly.

### `omega-mindset score <scorecard.json>`

Validates a weekly scorecard and prints a descriptive summary. Validation is
strict on purpose: every state score must be a number from 0 to 10, every count
must be a non-negative integer, and the boolean fields must be booleans. A file
that fails validation produces the failing field name, not a partial summary.

```bash
omega-mindset score ~/mindset/04_WEEKLY_SCORECARD.json
```

**When to use it:** every time a week closes, before any coaching pass on that
week.
**Returns:** week start, state average, the lowest and highest state domain, the
execution counts, the promise kept-or-repaired rate, the Rohn metrics, and an
explicit note that the summary is descriptive and is not a clinical score or a
measure of human worth.

### `omega-mindset challenge --output <dir> [--start YYYY-MM-DD] [--name "<name>"]`

Creates the six-month identity challenge workspace: 180 daily follow-ups, 26
weekly follow-ups, 6 monthly reviews, the identity constitution, the challenge
plan across six monthly seasons, `state.json` (start date, day index, cadence)
and an empty `coaching/` directory.

`--start` dates the daily files. Without it the files carry day numbers and no
calendar dates.

```bash
omega-mindset challenge --output ~/challenge --start 2026-08-11
```

**When to use it:** when the user commits to a season-long program rather than a
single cycle. This is a cadence and follow-up scaffold. If the user is running a
named transition from one identity to another, the charter for that transition
belongs to Identity Shift {OS} (`identity-shift-os`), which can drive this same
workspace.
**Returns:** the tree it wrote, and the first file to fill.

### `omega-mindset coach <workspace> [--arm|--disarm]`

Runs one coaching pass: it assembles the latest daily, weekly and monthly
follow-ups into a context, runs the Mindset master agent over them, and writes
a dated file under `coaching/`. The cadence is selected automatically: a closed
month gives a monthly pass, a closed week gives a weekly pass, otherwise a daily
pass.

Nothing runs autonomously until you arm it. `--arm` installs a daily 07:00 cron
entry for that workspace; `--disarm` removes it. Arming is an approval boundary,
not a default.

```bash
omega-mindset coach ~/challenge            # one pass, now
omega-mindset coach ~/challenge --arm      # daily 07:00, unattended
omega-mindset coach ~/challenge --disarm   # stop it
```

**When to use it:** after follow-ups have been filled. A pass over an empty week
has nothing to read and says so.
**Returns:** the path of the coaching file written, containing identity
evidence, a system diagnosis, one keystone adjustment and a protect-first check.
It refuses to run against a directory with no `state.json`, because that
directory is not a challenge workspace.

## Command summary

| Command | Does |
|---|---|
| `/mindset-os` | the coaching loop, routed to one mode |
| `/mindset` | the same OS, short form |
| `omega-mindset new` | the 19-file editable workspace |
| `omega-mindset score` | validate and summarize one closed week |
| `omega-mindset challenge` | the 180-day follow-up and cadence workspace |
| `omega-mindset coach` | one AI pass over the latest follow-ups, armable |
