# Money {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install money-os` | Installs this OS into your environment | Once, first |
| `agentik configure money-os` | Collects the minimum context it needs | After install |
| `agentik run money-os` | Starts the OS | Every session |
| `agentik doctor money-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update money-os` | Updates to the latest version | When a release lands |
| `agentik eval money-os` | Runs its evaluation suite | Before trusting it |

`configure` asks for the account list and the currency of each account, and
nothing else. Categories, obligations and rules are learned from use.

## OS commands

### `/money`

The current position: this month so far, marked provisional, against the last
closed month and the obligations still to leave.

```bash
/money
```

**When to use it:** any time you want the short answer without opening anything.
**Returns:** in, out and left for the current month, the next dated obligations,
and the runway figure with the burn it used.

### `/money-intake <path|--paste>`

Stage a statement, a bank export, a card export, a receipt or a photograph into
records. Nothing is counted yet: every extracted field carries its source and a
confidence, and low confidence stays staged.

```bash
/money-intake ~/Documents/statements/2026-07-current-account.csv
/money-intake ~/Photos/receipt-2026-07-14.jpg
/money-intake --paste
```

**When to use it:** whenever a document arrives, not once a quarter in a panic.
**Returns:** how many lines staged, how many classified automatically, how many
need you, and any suspected duplicate of an earlier import.

### `/money-classify [--queue | --rule]`

Work the staged queue. The OS suggests a category and the reasoning behind it;
you confirm, correct or defer. A correction can be saved as a rule for future
imports, and a rule never rewrites a line you already categorised.

```bash
/money-classify --queue
/money-classify --rule "SNCF -> travel"
```

**When to use it:** after intake, before a close.
**Returns:** the queue worked down, the rules created, and what is still
unclassified with its amount.

### `/money-close [<month>]`

Close a month. Reconciles every account against its statement balance, then
produces the month as fact.

```bash
/money-close            # the month that just ended
/money-close 2026-07
```

**When to use it:** once a month, when intake for that month is drained.
**Returns:** in, out, left, surplus or deficit, by category, the unclassified
residue named separately, and whether the month funded the reserve target
Wealth {OS} asked for. If an account does not reconcile the close is refused and
the gap is reported in currency and direction.

### `/money-runway`

How many months the current balances cover at the current burn.

```bash
/money-runway
/money-runway --burn last-3-closed
```

**When to use it:** before a commitment, after an income change, and whenever
the answer feels like a guess.
**Returns:** months of cover, plus the two inputs it used: which balances were
counted and which burn was used. It never returns the number alone.

### `/money-forecast [<months>]`

Expected in and out for the coming months, from the obligation register and the
closed-month pattern.

```bash
/money-forecast 6
```

**When to use it:** planning a period, not judging a past one.
**Returns:** month by month expected in, out and left, each with the assumption
list under it. Everything here is labelled a projection.

### `/money-obligations`

The recurring obligation register and the next 90 days of committed outflow.

```bash
/money-obligations
/money-obligations --cancellable
```

**When to use it:** when outflow feels higher than it should, and before any new
subscription or instalment.
**Returns:** each obligation with amount, cadence, next date, source account and
whether it is cancellable, plus anything whose amount has changed since the
register was written.

### `/money-decide "<decision>"`

An affordability read for a named decision. It does not tell you yes or no.

```bash
/money-decide "new laptop, 2400"
/money-decide "move flat, rent +350 a month"
```

**When to use it:** for any spend above the threshold you set at configure time.
**Returns:** the effect on this month, on the runway in months, and on the
reserve contribution, with the tradeoff stated in currency. The decision stays
yours.

### `/money-pack [--period <range>]`

Assemble the accountant pack: the records organised by period and category, plus
every question this OS refused to answer itself.

```bash
/money-pack --period 2026-01..2026-06
```

**When to use it:** before a meeting with your accountant, or when they ask.
**Returns:** the organised export path and the question list. Sending it to
anyone, including your own accountant, is a separate explicit approval, per
recipient and per document.

---

## Command summary

| Command | Does |
|---|---|
| `/money` | the current position, provisional |
| `/money-intake <path>` | stage a document into records with sources |
| `/money-classify` | work the staged queue, learn rules from corrections |
| `/money-close [<month>]` | reconcile and close the month as fact |
| `/money-runway` | months of cover, with the burn and balances shown |
| `/money-forecast [<months>]` | expected in and out, assumptions listed |
| `/money-obligations` | the recurring register and the next 90 days |
| `/money-decide "<decision>"` | effect on month, runway and reserve |
| `/money-pack` | organised records plus the accountant's questions |

No command in this list moves money. Every one of them ends at an instruction
the operator executes in their own bank.
