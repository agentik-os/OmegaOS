# Pricing {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install pricing-os` | Installs this OS into your environment | Once, first |
| `agentik configure pricing-os` | Collects the minimum context it needs | After install |
| `agentik run pricing-os` | Starts the OS | Every session |
| `agentik doctor pricing-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update pricing-os` | Updates to the latest version | When a release lands |
| `agentik eval pricing-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/pricing` | Opens the OS and reports the live price book and policy state | nothing | the price book version, the floor, and any unevidenced line |
| `/pricing-model` | Chooses the pricing model and names the ones rejected | an offer definition | the model, the fit reason, the rejected alternatives |
| `/pricing-package` | Designs the tier structure and what separates the tiers | a pricing model | tiers, each with a named buyer and a reason to move up |
| `/price-evidence` | Records the willingness to pay evidence behind a number | a price point | an evidence record with source and date |
| `/price-test` | Designs and reads a willingness to pay test | a price hypothesis | a test design, then an observed result |
| `/price-book` | Assembles and versions the price book | model, packaging, evidence | a versioned price book, one line per sellable thing |
| `/discount-policy` | Sets the permitted range, the approval ladder and the floor | unit economics, price book | a discount policy with a floor and a named owner |
| `/discount-check` | Adjudicates a specific discount request against the policy | a requested price | granted, escalated or refused, with the reason |
| `/price-change` | Plans a price move with its revenue impact | old price, target price | a change plan with effective date and impact estimate |
| `/grandfather` | Decides what happens to existing customers on a changed price | a change plan | a grandfathering decision, per customer segment |
| `/pricing-review` | Reconciles realised discounts against the stated policy | Revenue discount history | a variance report per deal and per seller |

---

### `/pricing`

```
/pricing
```

Opens Pricing {OS} and reports what is currently true: the live price book
version, the discount floor and its owner, and every price line whose evidence
reference is empty. Unevidenced lines are listed as unevidenced, never quietly
included in the count of healthy ones.

**When to reach for it:** at the start of any pricing session.
**Returns:** price book version, policy summary, unevidenced line count, and
any open price change.

### `/pricing-model`

```
/pricing-model offer-2
```

Chooses how the money is structured: per seat, per unit, per outcome,
retainer, usage or fixed. It asks how the buyer experiences value, then
records what was rejected.

**When to reach for it:** once per offer, before any number exists.
**Returns:** the chosen model with its fit reason, and the rejected models
each with the reason they were rejected. The rejection list matters: it is
what stops the same debate reopening every quarter.

### `/pricing-package`

```
/pricing-package offer-2 --tiers 3
```

Decides what sits in which tier. Packaging is a pricing decision, not a
marketing decision, so this command owns contents, not names.

**When to reach for it:** after the model, before the book.
**Returns:** the tier structure. Each tier carries a named buyer and a stated
reason to move up from the tier below. A tier with no distinct buyer is
reported as a tier that will not sell.

### `/price-evidence`

```
/price-evidence "team plan" --source "12 discovery calls, 2026 Q2"
```

Attaches evidence to a number: what comparable buyers paid, what they refused,
what they switched from and at what price.

**When to reach for it:** for every price point, before it enters the book.
**Returns:** an evidence record with source and date. It refuses to accept an
opinion as evidence and marks the line unevidenced instead.

### `/price-test`

```
/price-test "team plan" --hypothesis 240
```

Designs a willingness to pay test, and later reads its result.

**When to reach for it:** whenever a price point has no observation behind it.
**Returns:** on design, the test and what would falsify the hypothesis. On
read, the observed result and whether the hypothesis survived. A price with no
willingness to pay evidence is a guess wearing a decimal point, and this is
how the guess becomes a number.

### `/price-book`

```
/price-book --publish
```

Assembles the price book and stamps a version. Every line needs a price, a
unit, a currency, an effective date and an evidence reference.

**When to reach for it:** after the model, the packaging and the evidence.
**Returns:** the versioned price book, emitted to Sales {OS}, Revenue {OS} and
Growth {OS}. It rejects any incomplete line rather than inferring the missing
field, and it will not publish without human approval of the exact text.

### `/discount-policy`

```
/discount-policy --floor 0.85 --owner "founder"
```

Sets the permitted discount range, the approval ladder inside it, the floor
and the floor's owner.

**When to reach for it:** as soon as a price book is live, before the first
negotiation.
**Returns:** the policy, emitted to Sales {OS} and Revenue {OS}. The owner is
never the person negotiating the deal: a floor its own beneficiary can move is
not a floor.

### `/discount-check`

```
/discount-check deal-118 --requested 0.70
```

Adjudicates one request against the policy.

**When to reach for it:** every time a prospect asks for a number that is not
in the book.
**Returns:** granted, escalated to the floor owner, or refused, with the
reason and the arithmetic. The request is recorded either way, because a
refused request is still data about where the price is meeting resistance.

### `/price-change`

```
/price-change "team plan" --to 290 --effective 2026-11-01
```

Plans a move: old price, new price, effective date, and the modelled revenue
impact including expected churn.

**When to reach for it:** before anything is announced to anyone.
**Returns:** a change plan. It will not complete without a grandfathering
decision, and it excludes any customer whose signed contract term the change
would break, surfacing each one by name.

### `/grandfather`

```
/grandfather "team plan" --segment existing --decision hold-12-months
```

Decides what happens to customers already on the old price.

**When to reach for it:** as part of every price change, without exception.
**Returns:** the decision per customer segment, with notice period and the
date each segment moves. A price change without this creates two truths for
the same customer, resolved later by whoever happens to be on the call.

### `/pricing-review`

```
/pricing-review --period 2026-Q3
```

Pulls realised revenue and discount history from Revenue {OS} and reconciles
it against the stated policy.

**When to reach for it:** on a fixed cadence, and after any policy change.
**Returns:** a variance report per deal and per seller. Where behaviour and
policy disagree, it names the gap. It never rewrites the policy to match what
people actually did.

## Command summary

| Command | Does |
|---|---|
| `/pricing` | what is currently true about price and policy |
| `/pricing-model` | how the money is structured, and what was rejected |
| `/pricing-package` | what sits in which tier |
| `/price-evidence` | the observation behind a number |
| `/price-test` | turn a price hypothesis into an observation |
| `/price-book` | the versioned, quotable list |
| `/discount-policy` | the range, the ladder, the floor, the owner |
| `/discount-check` | adjudicate one request |
| `/price-change` | plan the move and model its impact |
| `/grandfather` | decide what happens to existing customers |
| `/pricing-review` | policy versus what actually happened |
