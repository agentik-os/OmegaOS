# Business Strategy {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install business-strategy-os` | Installs this OS into your environment | Once, first |
| `agentik configure business-strategy-os` | Collects the minimum context it needs | After install |
| `agentik run business-strategy-os` | Starts the OS | Every session |
| `agentik doctor business-strategy-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update business-strategy-os` | Updates to the latest version | When a release lands |
| `agentik eval business-strategy-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/thesis`

```
/thesis
/thesis --revise "we moved from projects to retainers"
```

Writes or revises the asset thesis: what a rational third party would be buying,
and why that is not simply the owner. It interrogates each candidate answer and
rejects any that resolve to the owner's judgement, relationships or reputation.

**When to use it:** first, before anything else in this OS, and again whenever
the business changes materially: a new revenue model, a new customer class, a
departure, an acquisition.

**Returns:** the thesis, or the finding that there is not one yet, with the
specific dependencies that make the answer "the owner". Emits
`strategy.asset_thesis.published` when a thesis is accepted.

### `/dependence`

```
/dependence
/dependence --class sales
```

Scores owner dependence across the decision classes: sales, pricing, delivery,
hiring, vendor relationships, technical judgement, financial control, customer
relationships. Each class is assigned to a person, a document or a system.

**When to use it:** after the thesis, and on the review cadence. Also
immediately before any conversation about the operator stepping back.

**Returns:** the assessment, class by class, with the owner named where the owner
is the answer, and the specific decisions behind each score rather than a role
label. Emits `strategy.owner_dependence.scored`. The documentation gaps route to
Process & SOP {OS} and the roles route to Team & Delegation {OS}.

### `/drivers`

```
/drivers
/drivers --measure customer-concentration
/drivers --unverified
```

Builds and maintains the value driver table. Pulls what it can from KPI &
Analytics {OS} and Revenue {OS}, asks the operator for the rest, and labels every
entry verified or unverified with its source and measurement date.

**When to use it:** after dependence is scored, and whenever a metric behind a
driver is re-measured upstream.

**Returns:** the table, each driver with value, source, date and label.
`--unverified` returns only the drivers resting on the operator's own account,
and for each, the metric that would verify it. Emits
`strategy.value_driver.measured`.

### `/advantage`

```
/advantage
/advantage --test "our onboarding is faster"
```

Runs the copy test: would this still hold after a competent competitor copied
the visible surface, the product, the pricing, the site and the pitch. What
survives enters the durable advantage register. What does not is struck with the
reason recorded.

**When to use it:** after the drivers are measured, and any time the operator
catches themselves describing a feature as a moat.

**Returns:** the register, plus the struck claims with the reason each failed.
Struck claims are kept, because the same claim tends to come back.

### `/readiness`

```
/readiness
/readiness --standard "sellable to a strategic buyer in 24 months"
```

Computes the gap between the measured drivers and a readiness standard the
operator states. If no standard is on file, it asks and offers the common ones
with their differences. It does not pick one silently.

**When to use it:** once drivers are current, before any conversation with a
buyer, a bank or an adviser, and on the cadence.

**Returns:** the gap per driver, then, separately and always, the list of
unverified inputs and the same gap recomputed with them excluded. A single
headline number is never returned on its own. Emits
`strategy.readiness.flagged`, which Exit & Liquidity {OS} consumes.

### `/options`

```
/options
/options --driver key-person-risk
```

Generates the strategic option set. Each option names the value driver it moves,
what it costs, and whether it can be undone and at what price.

**When to use it:** when the gap is known and the question turns to what to do
about it.

**Returns:** the options, ranked by the size of the gap they close per unit of
cost, with reversibility shown separately because it is not a tiebreaker, it is
often the decision. Options accepted by the operator route to Execution {OS} as
dated work.

### `/value-range`

```
/value-range
/value-range --method multiple
```

Produces an internal working valuation range from the operator's own numbers,
with the method, the inputs and the assumptions exposed.

**When to use it:** for internal orientation only, to understand which drivers
move the number and by how much.

**Returns:** the range, the method, every input with its verified or unverified
label, and a plain statement that this is an internal working figure. It is not
an audited valuation, not an accountant's opinion of value, and not usable in a
filing, a loan application or a negotiation as an independent figure. For any of
those, the operator engages an accountant or a valuation professional.

### `/review`

```
/review
/review --since 2026-02-01
```

Runs the review cadence: re-measures what is stale, reports the delta since the
last review, and attaches the age of anything carried forward.

**When to use it:** on the cadence set at configuration, typically quarterly, and
after any event that would move a driver.

**Returns:** the delta per driver, what was re-measured, what was carried forward
and how old it is, and whether the readiness gap has moved. A carried forward
driver never enters a current claim without the operator saying so explicitly.

## Command summary

| Command | Does |
|---|---|
| `/thesis` | what a buyer would be buying, and whether that is just the owner |
| `/dependence` | which decisions live only in the owner's head |
| `/drivers` | the value driver table, each entry verified or unverified |
| `/advantage` | the copy test, and the register of what survives it |
| `/readiness` | the gap to a stated standard, with unverified inputs named |
| `/options` | what to do about the gap, with cost and reversibility |
| `/value-range` | an internal working range, never an audited valuation |
| `/review` | the delta on the cadence, with measurement ages attached |
