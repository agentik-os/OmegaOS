# Ownership {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install ownership-os` | Installs this OS into your environment | Once, first |
| `agentik configure ownership-os` | Collects the minimum context it needs | After install |
| `agentik run ownership-os` | Starts the OS | Every session |
| `agentik doctor ownership-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update ownership-os` | Updates to the latest version | When a release lands |
| `agentik eval ownership-os` | Runs its evaluation suite | Before trusting it |

---

## OS commands

### `/entity-map`

Build or show the entity map: every company, partnership, trust and personal
holding, with the edges between them labelled by position and percentage.

```bash
/entity-map                      # the whole map
/entity-map --entity "Holdco Ltd"  # one entity and its immediate edges
/entity-map --unknowns           # only the facts still missing
```

**When to use it:** first, before anything else, and again whenever an entity is
created or dissolved.
**Returns:** the tree, with entity type, jurisdiction and registration number on
each node, and an explicit unknown wherever a fact is missing. The first run
normally returns several unknowns, and that is the correct output.

### `/position`

Record, amend or show a position: holder, entity, instrument, quantity, class
and the denominator its percentage is computed against.

```bash
/position add --entity "Opco SAS" --holder me --instrument ordinary --qty 4000
/position show --entity "Opco SAS"
/position amend --id POS-014 --qty 3600 --source "SPA 2026-03-11 cl.4.2"
```

**When to use it:** on any issuance, grant, transfer, dilution or cancellation,
after it has happened and there is a document.
**Returns:** the position record with its verification state, and the recomputed
cap table for that entity showing every holder against the same denominator. An
amendment requires approval before it mutates the register.

### `/terms`

Extract and show the terms register for an entity: voting, liquidation
preference, drag, tag, anti-dilution, pre-emption, transfer restriction and
information rights.

```bash
/terms --entity "Opco SAS"
/terms extract --doc ~/docs/sha-2026.pdf --entity "Opco SAS"
/terms --entity "Opco SAS" --class drag
```

**When to use it:** when an agreement lands, and before any transaction that a
clause could block.
**Returns:** one row per term class with the clause reference and the operative
text, or `absent` with the document that was searched. It never returns an
interpretation of a clause presented as the clause.

### `/vesting`

Project a grant to a date: vested, unvested, behind the cliff, and what
accelerates on a trigger.

```bash
/vesting --grant G-003
/vesting --grant G-003 --at 2027-01-01
/vesting --scenario change-of-control
```

**When to use it:** before negotiating a departure, an exercise, or a change of
control.
**Returns:** vested and unvested quantities at the date, the cliff status, the
exercise price and expiry, and the acceleration clause reference where one
applies. Recomputed every time, never read from a stored figure.

### `/reconcile`

Diff the working register against the source documents and the statutory
register.

```bash
/reconcile --entity "Opco SAS"
/reconcile --all
```

**When to use it:** after a round closes, before a due diligence, and on a
quarterly cadence.
**Returns:** a line-level report: matched, differing (with both values and both
sources), and unverified. Where the statutory register differs, the statutory
register is treated as correct and the difference is raised. Nothing is changed
without approval.

### `/consents`

For a proposed transaction, produce the consent map: who must sign, who must
waive, and which clause creates each requirement.

```bash
/consents --transaction "sale of 100% of Opco SAS"
/consents --transaction "issue 500 shares to new investor"
```

**When to use it:** before an exit process opens, and before any issuance or
transfer.
**Returns:** the list of signatories and waivers with their clause references,
and any consent whose holder is unknown or unreachable flagged separately.
Handed to Exit & Liquidity {OS} once approved.

### `/obligations`

Show and maintain the ownership obligations calendar per entity and
jurisdiction.

```bash
/obligations                    # everything due, soonest first
/obligations --entity "Holdco Ltd"
/obligations --unowned          # obligations with no responsible human
```

**When to use it:** monthly, and whenever an entity is registered in a new
jurisdiction.
**Returns:** dated obligations with jurisdiction, lead time and responsible
human. Entries with no named human are listed separately, because an unowned
filing is a missed filing. Approved entries are pushed to Execution {OS} as
tasks.

### `/counsel-pack`

Assemble the question pack for a lawyer, corporate secretary, accountant or tax
adviser: the register extract, the relevant documents and the specific written
question.

```bash
/counsel-pack --question "does cl. 7.3 pre-emption apply to an intra-group transfer"
/counsel-pack --flag F-002
```

**When to use it:** every time a question needs legal or tax judgement, which is
every time the answer would otherwise be advice.
**Returns:** a single document containing the question, the entity and position
context, the clause text, and the professional it is addressed to. Sharing it
with anyone outside the user requires explicit approval.

---

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/entity-map` | build or show the entity map | the tree, with explicit unknowns |
| `/position` | record, amend or show a position | the position and the recomputed cap table |
| `/terms` | extract and show the terms register | clause references and operative text, or `absent` |
| `/vesting` | project a grant to a date | vested, unvested, cliff and acceleration |
| `/reconcile` | diff the register against sources | matched, differing, unverified, line by line |
| `/consents` | consent map for a proposed transaction | signatories, waivers and the clause behind each |
| `/obligations` | the ownership obligations calendar | dated obligations with an owner, and the unowned ones |
| `/counsel-pack` | prepare a question for a professional | the question, the context and the documents |
