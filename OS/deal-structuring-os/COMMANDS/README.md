# Deal Structuring {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install deal-structuring-os` | Installs this OS into your environment | Once, first |
| `agentik configure deal-structuring-os` | Collects the minimum context it needs | After install |
| `agentik run deal-structuring-os` | Starts the OS | Every session |
| `agentik doctor deal-structuring-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update deal-structuring-os` | Updates to the latest version | When a release lands |
| `agentik eval deal-structuring-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/structure` | Opens the current structure: chosen instrument, open terms, unpriced items | Whenever a deal has a shape and open questions | The open terms, each with its cash value or a note that it has none yet |
| `/structure instrument` | Chooses the instrument and records why the alternatives were rejected | Before any modelling | The decision record, with a stated reason per rejected instrument |
| `/structure captable` | Loads and validates the complete cap table | Before the first model, and after any new instrument | The cap table, plus every instrument it believes is missing |
| `/structure model` | Models the waterfall at low, middle and high exits | After the instrument is chosen | Cash per party at three exit values, with unverified inputs labelled |
| `/structure price <term>` | Prices one term in cash at three exit values | Whenever a term is proposed, by either side | What the term is worth, to whom, and what it costs the other side |
| `/structure protect` | Sizes downside protection against named risks | Once the downside is understood | The protection register, each entry with its named risk and its cost |
| `/structure incentive` | Designs vesting, pool, and management incentives | When people are staying in the business | The incentive model, with pool timing and its dilution stated |
| `/structure earnout` | Models an earnout including the gamed case | Any time a payment depends on a future metric | The honest case, the gamed case, and the dispute surface |
| `/structure termsheet` | Assembles the term sheet as a draft for legal review | Once terms are agreed in substance | A draft marked for legal review, with typically binding parts flagged |
| `/structure prep` | Ranks terms by cash value and builds the trade set | Before a negotiation session | Ranked terms, the trade set, and the walk away terms |
| `/structure reconcile` | Compares executed documents line by line against agreed terms | When documents return from the lawyers | Every difference, with its cash effect, before completion |
| `/structure tax-questions` | States the tax questions an adviser must answer | Before choosing between structures with different tax treatment | The precise questions and the alternatives being compared |

### `/structure captable`

```bash
/structure captable --load ./captable.csv
/structure captable --validate
```

**Returns:** the loaded cap table and a list of what it believes is missing:
old convertibles, warrants, unissued options, side letters. It refuses to model
a waterfall while it believes the table is incomplete, because a partial
waterfall is confidently wrong rather than usefully uncertain.

### `/structure model`

```bash
/structure model --exits 4m,9m,22m
```

**Returns:** what every party receives in cash at each exit value, the
conversion decisions each holder would rationally make, and a labelled list of
inputs that are still unverified. The middle case is presented first, because
that is the one that will happen.

### `/structure price <term>`

```bash
/structure price "1x non-participating preference"
/structure price "seller note, 24 months, 6 percent"
```

**Returns:** the cash difference the term makes at each exit value, who gains
it, and what it costs the other side. A term whose value genuinely cannot be
quantified is reported as unquantifiable and ranked qualitatively, never given
an invented number.

### `/structure earnout`

```bash
/structure earnout --metric "gross profit" --period 24m --cap 900k
```

**Returns:** the honest case, the case where the party controlling the metric
optimises against it, and the specific decisions that would create a dispute.
If the earnout only works when both sides behave well, it says so plainly.

### `/structure reconcile`

```bash
/structure reconcile --agreed ./termsheet-v4.md --executed ./spa-final.pdf
```

**Returns:** a line by line comparison and every difference with its cash
effect. Differences are raised before completion. A difference found after
completion has stopped being a negotiation and become a dispute.

## Command summary

| Command | Does |
|---|---|
| `/structure` | the current shape and every open term |
| `/structure instrument` | chooses the instrument, records the rejections |
| `/structure captable` | loads and validates the full cap table |
| `/structure model` | the waterfall at low, middle and high exits |
| `/structure price <term>` | one term, in cash, at three exit values |
| `/structure protect` | protections sized to named risks |
| `/structure incentive` | vesting, pool, pool timing, management incentives |
| `/structure earnout` | the honest case and the gamed case |
| `/structure termsheet` | a draft for legal review |
| `/structure prep` | terms ranked by cash value, trade set, walk away terms |
| `/structure reconcile` | executed documents against agreed terms |
| `/structure tax-questions` | the questions for the adviser, not the answers |
