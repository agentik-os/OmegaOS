# Wealth {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install wealth-os` | Installs this OS into your environment | Once, first |
| `agentik configure wealth-os` | Collects the minimum context it needs | After install |
| `agentik run wealth-os` | Starts the OS | Every session |
| `agentik doctor wealth-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update wealth-os` | Updates to the latest version | When a release lands |
| `agentik eval wealth-os` | Runs its evaluation suite | Before trusting it |

`configure` asks for the reporting currency, the net worth change threshold that
should require your approval, and whether Money {OS} is installed. It does not
ask for your assets: that is `BASELINE`, and it is a conversation, not a form.

## OS commands

### `/wealth`

The position: net worth on its last dated basis, reserve cover in months, each
goal funded or short, and the top unmitigated risk.

```bash
/wealth
```

**When to use it:** the standing view, and the first thing to run in a session.
**Returns:** the four numbers that matter, each with the date and basis behind
it. Anything stale is marked stale rather than quietly reused.

### `/wealth-baseline`

Build the first dated balance sheet, or rebuild it from scratch after a
structural change.

```bash
/wealth-baseline
```

**When to use it:** first run, or after a divorce, a move, an inheritance, an
entity restructure or an exit.
**Returns:** assets, liabilities, net worth at a date, the basis for every line,
and the unvalued list kept separate at zero.

### `/wealth-update [--valuation <line>] [--from-close]`

Record a new dated point and attribute the movement.

```bash
/wealth-update --from-close            # after Money {OS} closes a month
/wealth-update --valuation "flat, appraisal 2026-08"
```

**When to use it:** on an event, not on a schedule: a closed month, a new
valuation, a position change, a debt repayment.
**Returns:** the new net worth point and the movement split into contribution,
valuation change and debt movement. A bare delta is never returned.

### `/wealth-reserve [--months <n>]`

Size the reserve, test where it is held, and write the refill rule.

```bash
/wealth-reserve
/wealth-reserve --months 9
```

**When to use it:** when no reserve policy exists, when the reserve has been
drawn down, or when income concentration changed.
**Returns:** target in months and in currency (computed from real outgoings in
closed months, not from an estimate), the liquidity verdict on each candidate
holding, the gap to target, and the refill rule. Publishes
`wealth.reserve_target.set` to Money {OS} on your approval.

### `/wealth-goal "<goal>" [--by <date>] [--target <amount>]`

Price a long-horizon goal against verified surplus.

```bash
/wealth-goal "deposit on a flat" --by 2029-06 --target 60000
/wealth-goal "stop needing client income" --by 2032-01
```

**When to use it:** the moment a goal is spoken, before it becomes an assumption.
**Returns:** required monthly contribution, funded or short by an amount per
month, the three levers if short (target, horizon, surplus), and what funding it
displaces. It picks no lever for you.

### `/wealth-risk`

The events that would break the plan, ranked by damage.

```bash
/wealth-risk
```

**When to use it:** annually, and after any change in income, health,
concentration or currency exposure.
**Returns:** each risk with the damage it would do in currency and months of
runway, its current mitigation, and the ones you have explicitly accepted.
Accepting a risk is recorded with a date, so it is a decision rather than a
silence.

### `/wealth-constraints`

Publish the boundaries Capital {OS} must allocate inside.

```bash
/wealth-constraints
/wealth-constraints --dry-run
```

**When to use it:** before any allocation decision, and whenever the reserve,
the goals or the risk picture moved.
**Returns:** reserve floor that must never be invested, dated liquidity needs
per goal, horizon, and tolerable loss expressed as what the household could
absorb without changing how it lives. Emits
`wealth.capital_constraints.published` on your approval.

### `/wealth-scenario "<what if>"`

Model a change as a range with its assumptions.

```bash
/wealth-scenario "client income stops for 9 months"
/wealth-scenario "rates up 2 points on the mortgage"
```

**When to use it:** before a commitment, and when a fear needs a number.
**Returns:** the effect on net worth, reserve cover and each goal, as a range,
with every assumption listed and what would falsify it. Never a single
compounding figure.

### `/wealth-pack [--for accountant|tax|insurance|legal]`

Assemble the adviser pack.

```bash
/wealth-pack --for tax
```

**When to use it:** before meeting a professional, or when they ask.
**Returns:** the organised numbers, plus every question this OS refused to
answer itself (tax treatment, cover adequacy, estate structure, suitability).
Sending it to anyone is a separate explicit approval, per recipient and per
document.

---

## Command summary

| Command | Does |
|---|---|
| `/wealth` | net worth, reserve cover, goal status, top risk |
| `/wealth-baseline` | build the first dated balance sheet |
| `/wealth-update` | a new dated point, with the movement attributed |
| `/wealth-reserve` | size, place and refill the reserve |
| `/wealth-goal "<goal>"` | price a long-horizon goal against verified surplus |
| `/wealth-risk` | what would break the plan, ranked by damage |
| `/wealth-constraints` | publish the boundaries Capital {OS} allocates inside |
| `/wealth-scenario "<what if>"` | model a change as a range with assumptions |
| `/wealth-pack` | organised numbers plus the professional's questions |

No command in this list buys, sells, funds or transfers anything. There is no
command to allocate: that is Capital {OS}, working inside the constraints this
OS publishes.
