# Deal Flow {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install deal-flow-os` | Installs this OS into your environment | Once, first |
| `agentik configure deal-flow-os` | Collects the minimum context it needs | After install |
| `agentik run deal-flow-os` | Starts the OS | Every session |
| `agentik doctor deal-flow-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update deal-flow-os` | Updates to the latest version | When a release lands |
| `agentik eval deal-flow-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/dealflow` | Opens the funnel: live items, stalled items, what needs a decision today | Start of a working session | The funnel state, ordered by what is blocking |
| `/dealflow screen write` | Writes or revises the screen, and checks it against Capital {OS} policy | Before the pipeline fills, and whenever a criterion changes | A new versioned screen, with what changed and why |
| `/dealflow intake` | Logs one opportunity with its source, date and raw claim | Every time something arrives, from any direction | The record id, plus any duplicate it matched |
| `/dealflow screen <id>` | Applies the screen inside its time budget | Once per intake record | Qualify, pass or abstain, with the reason and the criteria that decided it |
| `/dealflow track <id>` | Sets or advances stage, next action, owner and date | Whenever the state of an opportunity really changes | The new stage and the next action with its owner |
| `/dealflow pass <id>` | Drafts the pass message and records the reason | On any no, at any stage | A draft for a human to send, plus the logged reason |
| `/dealflow handoff <id>` | Builds the packet for the receiving OS and transfers ownership | When an opportunity qualifies for real work | The packet, the receiving OS, and the recorded transfer |
| `/dealflow source` | Works the channels: what is quiet, who is owed a touch, what to do next | Weekly, or when the funnel thins | Each channel with a dated next action |
| `/dealflow sweep` | Ages every live item and marks the stalled ones | On a cadence, never on demand only | The stalled list, and what each one needs |
| `/dealflow report` | Produces the funnel report for the period | At period close, or before a strategy review | Volume, conversion by stage and by source, time in stage, ranked pass reasons |
| `/dealflow sources report` | Reports the source ledger: qualified per channel and time cost | When deciding where the next hour goes | Channels ranked by qualified output against time spent |

### `/dealflow screen write`

The screen is a policy artifact. Writing one mid deal is allowed, but it is
recorded as a new version with the date and the reason, so nobody can later
claim the deal fitted the screen it caused.

```bash
/dealflow screen write
/dealflow screen write --from-pass-reasons   # start from the ranked reasons you actually passed on
```

**Returns:** the new version, a diff against the previous one, and any conflict
with the current allocation policy from Capital {OS}.

### `/dealflow intake`

Logs one opportunity. It always asks for the source, and it never accepts
"unknown" silently: an unattributed record is created but flagged, and appears
in the funnel report as unattributed.

```bash
/dealflow intake --source "referrer: J. Okonkwo" --raw ./inbound/teaser.pdf
```

**Returns:** the record id, the attributed source, and any existing record it
looks like a duplicate of.

### `/dealflow screen <id>`

```bash
/dealflow screen OPP-118
/dealflow screen OPP-118 --budget 20m
```

**Returns:** qualify, pass or abstain. Abstain names the criterion the screen
does not cover and asks whether the screen needs a new version. Exceeding the
time budget returns the fact that it was exceeded, which is the signal that the
remaining question belongs to Due Diligence {OS}.

### `/dealflow handoff <id>`

```bash
/dealflow handoff OPP-118 --to investment-thesis-os
/dealflow handoff OPP-118 --to acquisition-os
```

**Returns:** the packet the receiving OS expects, and an explicit record that
Deal Flow no longer owns this opportunity. From that point Deal Flow reports it
but does not drive it.

### `/dealflow report`

```bash
/dealflow report --period Q3
```

**Returns:** volume in, conversion by stage, conversion by source, median and
worst time in stage, and the ranked pass reasons. Stalled items are shown as
stalled and are excluded from the live count.

## Command summary

| Command | Does |
|---|---|
| `/dealflow` | opens the funnel and what needs deciding |
| `/dealflow screen write` | writes or versions the screen |
| `/dealflow intake` | logs one opportunity with its source |
| `/dealflow screen <id>` | qualify, pass or abstain, time boxed |
| `/dealflow track <id>` | stage, next action, owner, date |
| `/dealflow pass <id>` | drafts the pass, records the reason |
| `/dealflow handoff <id>` | packet plus recorded transfer of ownership |
| `/dealflow source` | works the channels |
| `/dealflow sweep` | ages items, marks stalled |
| `/dealflow report` | the funnel report |
| `/dealflow sources report` | which channel is worth the next hour |
