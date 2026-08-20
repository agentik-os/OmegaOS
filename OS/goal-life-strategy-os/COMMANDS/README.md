# Goal & Life Strategy {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install goal-life-strategy-os` | Installs this OS into your environment | Once, first |
| `agentik configure goal-life-strategy-os` | Collects the minimum context it needs | After install |
| `agentik run goal-life-strategy-os` | Starts the OS | Every session |
| `agentik doctor goal-life-strategy-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update goal-life-strategy-os` | Updates to the latest version | When a release lands |
| `agentik eval goal-life-strategy-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/life-strategy` | Opens the OS and routes to the right mode, or runs the full annual strategy pass | You do not know which of the commands below you need, or a year is turning | the current goal set, horizon map, allocation ledger and not-doing list, plus the mode it selected and why |
| `/goal-set` | Defines one life-level goal with domain, horizon, cost and retirement condition | You want something at life scale and it is not yet written down | one goal record, and its effect on the remaining capacity |
| `/horizon-map` | Places every active goal on `now`, `this year`, `three to five years`, `direction`, with sequencing | You have more than three goals, or their timing is vague | the horizon map, and any horizon that is overloaded against the ceiling |
| `/tradeoff` | Resolves two or more claims on the same time, attention, money or energy | A new commitment arrived, or two goals are both stalling | a tradeoff record naming what lost, what won, and the ranking rule used |
| `/allocation-review` | Compares planned against actual allocation per life domain | A quarter closed, or your week does not match your plan | planned versus actual per domain, every divergence over tolerance, one correction each |
| `/goal-retire` | Closes a goal as reached, released, superseded or failed | A goal is done, dead, or no longer yours | a retirement record and the reassignment of the freed capacity |

### `/life-strategy`

The entry point. With no argument it loads the current state and asks what
changed. It reads the goal set, horizon map and allocation ledger from
Context & Memory {OS}, the value order from Alignment {OS}, and the capacity
ceiling from Health & Energy {OS}, then names anything missing before it says
anything else.

Given an argument it routes: a new want goes to `/goal-set`, a conflict goes to
`/tradeoff`, a closed quarter goes to `/allocation-review`.

```
/life-strategy
/life-strategy "annual reset, I moved cities and left my job"
```

**Returns:** the current strategy in one screen, plus the selected mode and the
reason it was selected.

### `/goal-set`

Turns a want into a goal record. It will not finish without four fields: the
statement, the life domain, the horizon, and the cost in hours per week and
money per month. It then asks for the retirement condition, which is the
observable event that ends this goal by success or by release.

If the goal contradicts a standing belief in Mindset {OS}, it reports the
contradiction and holds the goal as blocked rather than resolving the belief
here.

```
/goal-set "run a business that pays me 8k a month without me in the room"
/goal-set "be strong at 50" --domain health --horizon 3-5y
```

**Returns:** the goal record, the capacity it claims, and what remains
unallocated. A goal that arrives without a cost is held as aspirational and
receives no capacity.

### `/horizon-map`

Sequences the goal set across four horizons: `now` (this quarter),
`this year`, `three to five years`, and `direction` (the aim with no date that
ranks the others). Sequencing matters more than sorting: it names which goal
must land before another one can start, so that two goals do not both claim the
same quarter.

```
/horizon-map
/horizon-map --horizon now
```

**Returns:** each goal on a horizon, the dependencies between them, and any
horizon whose summed cost exceeds the capacity ceiling.

### `/tradeoff`

The command that does the actual strategy work. Two claims arrive on the same
finite resource. It states both costs, applies the value order from
Alignment {OS} as the ranking rule, and writes down what lost.

When the call is irreversible, expensive, or the user cannot state a criterion,
it stops and hands the framing to Decision {OS} (`decision-os`) instead of
deciding. This OS records the outcome; it does not run the call.

```
/tradeoff "consulting contract at 4 days a week" vs "the product launch"
/tradeoff --resource money
```

**Returns:** a tradeoff record with what was taken from what, the ranking rule
cited by name, and the loser added to the not-doing list with its reason. When
Alignment {OS} has no value set, the record says the ranking was unranked user
preference.

### `/allocation-review`

The quarter-close command. Planned share of time, attention, money and energy
per domain, against what actually happened, read from Execution {OS} and
Habit Tracker {OS} where available and from the user otherwise.

Every divergence over the declared tolerance gets one named cause and one
correction, and the correction is a choice between two things only: change the
plan, or change the behaviour. Reporting a divergence with no correction is a
defect.

```
/allocation-review
/allocation-review --quarter Q3
```

**Returns:** the planned versus actual table, the divergences, the corrections,
and the goals now flagged unmeasured because no evidence exists for them.

### `/goal-retire`

Closes a goal. It requires a reason from four: reached, released, superseded,
failed. It then requires the reassignment: the freed hours and money go to a
named existing claim, or are explicitly banked as unallocated.

A goal that quietly leaves the set corrupts every future allocation, because
the capacity it held is never accounted for. This command exists so that never
happens silently.

```
/goal-retire "the certification" --reason superseded
/goal-retire --reason failed
```

**Returns:** the retirement record, what the goal cost over its life, and the
new unallocated capacity. Retiring a goal is inside the human approval
boundary: it asks first.

## Command summary

| Command | Does |
|---|---|
| `/life-strategy` | opens the OS, shows current strategy, routes to a mode |
| `/goal-set` | one life-level goal, with a cost and an end condition |
| `/horizon-map` | every goal placed and sequenced across four horizons |
| `/tradeoff` | resolves and records a contested claim on finite capacity |
| `/allocation-review` | planned versus actual per domain, with corrections |
| `/goal-retire` | closes a goal and reassigns the capacity it held |
