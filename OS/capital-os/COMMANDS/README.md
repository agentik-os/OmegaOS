# Capital {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install capital-os` | Installs this OS into your environment | Once, first |
| `agentik configure capital-os` | Collects the minimum context it needs | After install |
| `agentik run capital-os` | Starts the OS | Every session |
| `agentik doctor capital-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update capital-os` | Updates to the latest version | When a release lands |
| `agentik eval capital-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/capital policy` | Writes or revises the allocation policy: cheque bands, mix, concentration ceilings, reserve ratio | Before the first commitment, and whenever a ceiling is being tested | The policy document, versioned, with the reason for every amendment |
| `/capital budget <period>` | Sets deployable capital for a period and the pacing that spends it | At the start of a period, or when funded capital changes | Deployable amount, pacing plan, and the funding source each tranche reconciles to |
| `/capital screen <candidate>` | Tests a candidate against policy before any deep work | The moment a candidate arrives, before diligence spend | In band, out of band, or ceiling breach, naming the exact policy line |
| `/capital allocate <commitment>` | Sizes and approves or declines a named amount for a named commitment | When thesis and diligence are both in hand | A decision record with amount, reserve, policy lines tested and a signature block |
| `/capital reserve <position> <amount>` | Commits a follow-on reserve against a position | At allocation, and when a follow-on is recommended | The updated reserve ledger and the position it is held against |
| `/capital release <position>` | Releases a reserve that is no longer needed | On exit, impairment, or a decision to stand down | The released amount, the reason, and the new deployable balance |
| `/capital concentration` | Reports current concentration by position, sector, stage and vintage against every ceiling | Before any sizeable commitment, and at period close | A table of ceiling, current, headroom, and any line already breached |
| `/capital rebalance` | Turns drift between policy and reality into a corrective action or a written amendment | When concentration or mix has drifted past a band | The drift statement and the two legal options, corrective action or written amendment |
| `/capital review <period>` | Reviews pacing, concentration and realised outcomes against policy | At period close | Every policy line marked held, breached or amended, with evidence |
| `/capital decisions [--declined]` | Lists allocation decisions with the policy line that decided each | When you want to see whether the policy is right, not just whether it was followed | The decision log, approvals and declines, each with its governing policy line |

---

### `/capital policy`

Opens `POLICY` mode. Every ceiling must end with a number and a stated
consequence for breaching it. A ceiling with no consequence is a preference.

```bash
/capital policy                       # write or review the current policy
/capital policy --amend concentration # amend one line, with the reason recorded
```

The amendment path is deliberately slow. An amendment raised while a live
candidate is waiting is flagged as such in the policy history, because that is
the exact circumstance in which allocators talk themselves into a worse rule.

### `/capital budget <period>`

Deployable capital is cash that exists plus facilities that are drawable and
named. A pledge, a forecast distribution or an expected exit is not a funding
source. If you supply one, the budget is returned marked conditional and the
condition is stated.

```bash
/capital budget 2026-H2
/capital budget 2026-H2 --funded 400000 --facility "credit line, drawable, 100000"
```

### `/capital screen <candidate>`

Runs in minutes and is meant to be run before diligence spend, not after.

```bash
/capital screen "seed round, vertical SaaS, 250k ask"
```

**Returns** one of three verdicts and the line that produced it, for example:
out of band, cheque band is 50k to 150k and the ask is 250k. It does not
soften the verdict because the candidate looks good, and it does not need a
thesis to run, because its only job is to stop wasted work.

### `/capital allocate <commitment>`

The core command. It refuses to run without a thesis reference from Investment
Thesis {OS} and a diligence outcome from Due Diligence {OS}, and it refuses to
produce an allocation with no reserve line.

```bash
/capital allocate "Northwind seed" --amount 120000 --reserve 180000
/capital allocate "Northwind seed" --amount 120000 --reserve 0 \
  --reserve-reason "no follow-on right, single tranche instrument"
```

**Human approval gate.** The command produces a decision record and stops. It
does not approve itself. The record is unsigned until the allocator signs it,
and `capital.allocation.approved` is emitted only on signature. Nothing in this
command moves money, instructs a bank or transmits a subscription.

### `/capital reserve <position> <amount>`

Reserves are committed at the same moment as the initial cheque. Running this
command later, against a position that was allocated with no reserve line, is
treated as a new decision and requires its own signature.

```bash
/capital reserve northwind 180000
/capital reserve northwind 60000 --follow-on "series A pro rata"
```

### `/capital concentration`

The number, not the impression. Run it before the commitment, not after.

```bash
/capital concentration
/capital concentration --with "Northwind seed" --amount 120000 --reserve 180000
```

The `--with` form is the important one: it shows what the ceilings look like
after the commitment you are considering, reserve included. That is the test
that matters, because a commitment that fits today and breaches once its
reserve is drawn has already breached.

### `/capital review <period>`

Closes the period. Emits `capital.pacing.reported`.

```bash
/capital review 2026-H1
```

**Returns** pacing against plan, concentration against ceilings, and realised
outcomes against the assumptions the policy was built on. Where a policy line
was breached, the review names it, whether or not the breach was later
amended into legality.
