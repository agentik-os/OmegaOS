# Investment Thesis {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install investment-thesis-os` | Installs this OS into your environment | Once, first |
| `agentik configure investment-thesis-os` | Collects the minimum context it needs | After install |
| `agentik run investment-thesis-os` | Starts the OS | Every session |
| `agentik doctor investment-thesis-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update investment-thesis-os` | Updates to the latest version | When a release lands |
| `agentik eval investment-thesis-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/thesis draft <name>` | Writes the thesis: what must become true, why now, why us, what you are paid for taking which risk | Before any commitment, as soon as an opportunity is worth writing about | A timestamped thesis v1, stored, with its retrospective flag set or clear |
| `/thesis falsify <name>` | Turns each statement into a falsifiable claim with its disproof condition, and strikes the ones that cannot be wrong | Immediately after drafting, always before money moves | The claim register, plus the list of struck statements with strike reasons |
| `/thesis kill-criteria <name>` | Sets the conditions under which you do not enter or you exit | While exit is still cheap, before the commitment | The kill criteria sheet with the date it was set and the exit cost assumed |
| `/thesis premortem <name>` | Assumes the loss and writes the most likely cause | After falsification, before the final commitment decision | Ranked loss causes, each mapped to a claim or flagged as an unmonitored gap |
| `/thesis checkpoint <name>` | Tests the claims against dated evidence and tests the kill criteria | At a scheduled date or a stated milestone | A claim by claim verdict, the kill criteria result, and a stored checkpoint record |
| `/thesis drift <name>` | Compares your current stated justification against the stored original text | When the story around a position has changed, or before a follow on | A quoted side by side, and a drift verdict of none, partial or full |
| `/thesis revise <name>` | Opens a new version on new evidence, recording the change and the reason | When evidence contradicts the written thesis | Version n+1, the diff, the stated reason, and the next checkpoint date |
| `/thesis retire <name>` | Closes the thesis as validated, invalidated or superseded, and separates wrong from unlucky | When the bet is closed, killed or replaced | The retirement verdict, the realised outcome, and the wrong versus unlucky call with its basis |
| `/thesis hitrate [--since <date>]` | Computes your hit rate across closed theses, excluding retrospective ones | Quarterly, or when you want to know how your judgement is doing | Counts by verdict, the wrong versus unlucky split, and the excluded retrospective count |
| `/thesis list [--status <s>]` | Lists theses with their status, next checkpoint and drift flag | At the start of a review session | One row per thesis, with overdue checkpoints at the top |

---

### `/thesis draft <name>`

The first command, and the only one that must run before money moves. It asks
for your reasoning in your own words first and structures it afterwards, so the
assumptions stay visible.

If a commitment has already been made, it says so and stamps the thesis
`retrospective`. A retrospective thesis is still worth writing, it is simply
never counted as a prediction.

```bash
/thesis draft northwind-acquisition
/thesis draft northwind-acquisition --retrospective   # you already committed
```

**Returns:** a stored, timestamped thesis v1 and the event `thesis.drafted`.

### `/thesis falsify <name>`

Rewrites each statement as a claim with an explicit disproof condition and a
date by which the disproof should be observable. Anything that survives every
possible world is struck and kept in the file with a reason.

```bash
/thesis falsify northwind-acquisition
/thesis falsify northwind-acquisition --claim 3   # rework one claim
```

**Returns:** the claim register, and a count of how much of the thesis rests on
unfalsifiable ground. If that share is high, the output says so before anything
else.

### `/thesis kill-criteria <name>`

Reads the intended commitment size and its exit cost from Capital {OS} and sets
the conditions for not entering or for exiting, against real numbers.

```bash
/thesis kill-criteria northwind-acquisition
```

**Returns:** the kill criteria sheet and the event `thesis.kill_criteria.set`.
If the commitment has already been made, the sheet records that the criteria
were set after entry, which lowers the weight they carry at checkpoint.

### `/thesis checkpoint <name>`

The command that makes the thesis worth having written. It gathers dated
evidence, marks each claim holding, weakening, broken or untestable, and tests
the kill criteria. The verdict is written before any narrative is attached.

```bash
/thesis checkpoint northwind-acquisition
/thesis checkpoint northwind-acquisition --at 2027-01-31
/thesis checkpoint northwind-acquisition --late   # the date passed unrun
```

**Returns:** the checkpoint record and `thesis.reviewed`. A claim with no
available evidence is recorded as untestable this cycle, never as holding. A
checkpoint run late writes the missed date into the history. If a kill
criterion is met, it emits `thesis.invalidated` and stops, and a human decides
what happens to the position.

### `/thesis drift <name>`

Quotes the stored original text and your current stated justification side by
side. The comparison is always against the file, never against recollection.

```bash
/thesis drift northwind-acquisition
```

**Returns:** a drift verdict. If the current reason for holding does not appear
in any stored version, that is full drift and it routes you to `/thesis revise`
before anything else proceeds.

### `/thesis revise <name>`

Creates version n+1. It never edits in place: the superseded text stays
readable, which is the only reason drift can be detected at all.

```bash
/thesis revise northwind-acquisition --reason "channel partner exited"
```

**Returns:** the new version, the diff, the reason, and a new checkpoint date.

### `/thesis retire <name>`

Closes the thesis and forces the judgement most processes skip: was the
argument faulty, or was the argument sound and the outcome went against it. The
call is offered with reasoning and confirmed by you.

```bash
/thesis retire northwind-acquisition --verdict invalidated
/thesis retire northwind-acquisition --verdict validated
/thesis retire northwind-acquisition --verdict superseded --by northwind-v2
```

**Returns:** the retirement record, the realised outcome, and the wrong versus
unlucky determination with its basis. Feeds `/thesis hitrate` and the pattern
library.

### `/thesis hitrate [--since <date>]`

Your record, computed from retirements rather than from memory. Retrospective
theses are excluded and the excluded count is shown, so the number cannot be
improved by writing theses after the fact.

```bash
/thesis hitrate
/thesis hitrate --since 2025-01-01
```

**Returns:** counts by verdict, the wrong versus unlucky split, the
retrospective exclusions, and the pattern entries that recur most.
