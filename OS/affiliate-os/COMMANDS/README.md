# Affiliate {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install affiliate-os` | Installs this OS into your environment | Once, first |
| `agentik configure affiliate-os` | Collects the minimum context it needs | After install |
| `agentik run affiliate-os` | Starts the OS | Every session |
| `agentik doctor affiliate-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update affiliate-os` | Updates to the latest version | When a release lands |
| `agentik eval affiliate-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/affiliate` | Opens the partner portfolio | none | every live partner, its stop condition and its trust cost |
| `/partner-vet` | Vets a candidate product before terms are read | product, evidence of use | a vetting record with an accept or reject verdict |
| `/partner-terms` | Records programme terms and the exit clause | programme details | a terms record |
| `/trust-cost` | Scores what a recommendation would cost in audience trust | product, audience | a trust cost with the harm case named |
| `/disclosure` | Drafts the disclosure and states where it must live | jurisdiction, surfaces | disclosure text and a publication requirement |
| `/promotion` | Builds the promotion plan and its stop condition | partner, surfaces, dates | a promotion plan pending approval |
| `/promo-copy` | Drafts promotional copy traced to real use | promotion plan | draft copy, unapproved |
| `/attribution` | Reconciles own tracking against the partner report | period, both reports | a reconciliation with any gap named |
| `/affiliate-revenue` | Emits the period's revenue event to Revenue {OS} | reconciled period | an affiliate revenue event |
| `/partner-exit` | Withdraws from a partnership | partner, reason | a withdrawal record and an audience notice draft |
| `/affiliate-review` | Reviews the portfolio against trust cost and return | none | keep, pause or exit per partner |

---

## Portfolio

### `/affiliate`

Opens the partner portfolio: every live partnership with its terms, its stop
condition, its disclosure state and its trust cost to date.

```bash
/affiliate
```

**When to use it:** at the start of any affiliate session, and before agreeing
to anything new.
**Returns:** one line per partner. A partner whose disclosure is not live, or
whose stop condition is unwritten, is flagged before anything else is shown.

---

## Selection

### `/partner-vet <product>`

Vets a candidate. Asks for evidence of use first and refuses to continue
without it. The commission rate is deliberately not read at this stage.

```bash
/partner-vet "Acme Analytics"
```

**When to use it:** the moment a programme is mentioned, before any reply is
sent to the partner.
**Returns:** a vetting record: use evidence, the harm case, the coherence check
against the positioning claim, and an accept or reject verdict.

### `/trust-cost <product>`

Scores the audience-trust cost of the recommendation on its own, so the number
cannot be rationalised backwards from projected income.

```bash
/trust-cost "Acme Analytics"
```

**When to use it:** inside `/partner-vet`, or standalone when an existing
partner's product changes.
**Returns:** who would be harmed, how badly, how visibly, and what recovering
that trust would take.

### `/partner-terms <partner>`

Records the programme terms: commission, attribution window, payout schedule,
claim restrictions, how terms may change, and the exit clause.

```bash
/partner-terms "Acme Analytics"
```

**When to use it:** after an accept verdict, before any asset is built.
**Returns:** a terms record. A term the partner can change unilaterally is
marked as such, because that is what makes a mid-promotion change detectable.

---

## Promotion

### `/disclosure <jurisdiction>`

Drafts the disclosure text and states where it must appear for this audience.

```bash
/disclosure eu
/disclosure us --surfaces newsletter,youtube
```

**When to use it:** before the promotion plan is approved. Nothing publishes
until this is live.
**Returns:** the disclosure text, the required placement per surface, and an
explicit unresolved flag if the jurisdiction's rules are ambiguous.

### `/promotion <partner>`

Builds the promotion plan: surfaces, assets, sequence, and the stop condition,
written before launch.

```bash
/promotion "Acme Analytics"
```

**When to use it:** once terms and disclosure exist.
**Returns:** a plan pending human approval, with the stop condition stated as a
threshold and not as a judgement call.

### `/promo-copy <asset>`

Drafts promotional copy. Every claim traces to recorded use evidence; anything
that cannot be traced is omitted and reported as omitted.

```bash
/promo-copy newsletter-issue-1
```

**When to use it:** after the plan is approved.
**Returns:** draft copy, explicitly unapproved, plus the list of claims that
were dropped for lack of evidence. A human approves the exact text before it
reaches Content {OS}.

---

## Money and exit

### `/attribution <period>`

Reconciles own tracking against the partner's report for a payout period.

```bash
/attribution 2026-07
```

**When to use it:** every payout period, without exception.
**Returns:** both numbers, the gap, and the gap as a named finding with an
amount. It never adopts the partner's number as truth.

### `/affiliate-revenue <period>`

Emits the reconciled period as an affiliate revenue event to Revenue {OS}.

```bash
/affiliate-revenue 2026-07
```

**When to use it:** after `/attribution` for the same period.
**Returns:** the revenue event, and a refusal if the period is not reconciled.

### `/partner-exit <partner>`

Withdraws from a partnership, whether the stop condition fired or the operator
chose to leave.

```bash
/partner-exit "Acme Analytics" --reason "product degraded"
```

**When to use it:** the moment a stop condition fires, or on decision.
**Returns:** a withdrawal record, the list of assets to take down, and a draft
notice to the audience explaining what changed. The notice needs approval.

### `/affiliate-review`

Reviews the whole portfolio against trust cost and return.

```bash
/affiliate-review
```

**When to use it:** on a cadence the operator sets, and after any incident.
**Returns:** keep, pause or exit per partner, each with the evidence behind it.

---

## Command summary

| Command | Does |
|---|---|
| `/affiliate` | opens the partner portfolio |
| `/partner-vet` | vets a candidate on use evidence, before the rate is read |
| `/trust-cost` | scores what the recommendation costs in audience trust |
| `/partner-terms` | records terms, change rights and the exit clause |
| `/disclosure` | drafts the disclosure and gates the promotion on it |
| `/promotion` | builds the plan and its stop condition |
| `/promo-copy` | drafts copy traced to real use, unapproved |
| `/attribution` | reconciles own and partner numbers, names the gap |
| `/affiliate-revenue` | emits the revenue event to Revenue {OS} |
| `/partner-exit` | withdraws and tells the audience |
| `/affiliate-review` | keep, pause or exit, per partner |
