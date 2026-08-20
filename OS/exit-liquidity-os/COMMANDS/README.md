# Exit & Liquidity {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

Read the table left to right: what you type, what happens, when you would reach
for it, and what you get back.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install exit-liquidity-os` | Installs this OS into your environment | Once, first |
| `agentik configure exit-liquidity-os` | Collects the minimum context it needs | After install |
| `agentik run exit-liquidity-os` | Starts the OS | Every session |
| `agentik doctor exit-liquidity-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update exit-liquidity-os` | Updates to the latest version | When a release lands |
| `agentik eval exit-liquidity-os` | Runs its evaluation suite | Before trusting it |

---

## Preparing

### `/exit-readiness`

Score how sellable the business is against what a buyer will actually test, and
return the gap list.

```
/exit-readiness
/exit-readiness --shape secondary
```

Every gap is classified as **paperwork** (produce a document), **value** (the
business is worth less than the owner thinks, and this belongs to Business
Strategy {OS}), or **structural** (an entity in the wrong place, an unassigned
IP right, an unsigned founder agreement, which belongs to Ownership {OS} or
IP & Asset {OS} and to counsel).

**When to use it:** first, and years earlier than feels necessary. Structural
gaps are the slow ones, and they are the ones that break timelines when a buyer
finds them instead of the owner.
**Returns:** a readiness score, the gap list with a classification, an owner and
a date per gap, and the three gaps that most change the outcome. Emits
`exit.readiness.scored`.

### `/exit-index`

Build the diligence readiness index: the standard buyer request list, and for
each line whether it exists, where it lives, who produces it and its state.

```
/exit-index
/exit-index --missing-only
```

A line that does not exist is marked absent with an owner and a date. It is
never left blank and never filled with a plausible reconstruction, because a
buyer's diligence finds the difference and finds it late.

**When to use it:** immediately after the readiness score, and again whenever a
buyer sends a request list of their own.
**Returns:** the index, the count of absent lines, and the absent lines that
block the earliest gate. Emits `exit.dataroom.indexed`.

### `/exit-value`

Produce an internal valuation range from the operator's own numbers.

```
/exit-value
/exit-value --sensitivity churn
```

**This is an internal working estimate, and every artifact it produces says so.**
It is not a formal valuation, an appraisal or a fairness opinion. If a number is
going to a counterparty, a lender or a court, it comes from a qualified valuer.

**When to use it:** once the index is materially complete and the financials
are stable enough that the range means something.
**Returns:** a low, a base and a high, the named assumption under each, the
sensitivities that move the range most, and the inputs too unstable to model.
Where inputs contradict each other, it abstains and names the contradiction
rather than averaging it.

---

## Positioning

### `/exit-acquirers`

Map who plausibly buys this, and why.

```
/exit-acquirers
/exit-acquirers --contact-state
```

**When to use it:** before any conversation, and before responding to an
unsolicited approach, so the approach can be judged against alternatives rather
than against nothing.
**Returns:** candidates with the stated reason to buy, what they have bought
before, and the current contact state per name. It contacts nobody.

### `/exit-structure`

Write down what the operator will and will not accept, before an offer exists.

```
/exit-structure
```

Covers cash at close versus deferred, earn-out shape, escrow, working capital
treatment, and the explicit walk-away.

**When to use it:** while no offer is live. A walk-away written during a live
process is written against the momentum of that process.
**Returns:** the structure preference, marked **provisional** until the tax
professional has reviewed it, plus the tax question pack. Tax treatment moves
net proceeds more than headline price does, and it must be settled before a
structure is agreed, not after. On approval, emits `exit.structure.proposed` to
Ownership {OS} as a proposal, never an instruction.

### `/exit-questions <adviser>`

Assemble a question pack for a named adviser.

```
/exit-questions lawyer
/exit-questions tax
/exit-questions accountant
```

**When to use it:** before every adviser conversation, so the billable hour is
spent on judgement rather than on reconstructing context.
**Returns:** the questions, the documents each one needs attached, and the
decision each question unblocks. It is a pack for a professional to answer, not
an answer.

---

## Running and closing

### `/exit-release <document> <recipient>`

Request approval to release one document to one outside party.

```
/exit-release financials-2024 buyer-northstar
```

Approval is **per document and per recipient**. Approval to send a document to
one buyer is not approval to send it to a second buyer, to a broker, or to an
adviser. The OS prepares the release and asks; a human sends it.

**When to use it:** every single time anything goes outside.
**Returns:** the approval request, and on approval an append only disclosure log
entry recording the document, the recipient, the date, the approving human, and
the confidentiality agreement relied on.

### `/exit-timeline`

Show the exit timeline, its gates, and which gate is currently blocking.

```
/exit-timeline
```

**When to use it:** weekly during preparation, and at every gate during a live
process.
**Returns:** the gates in order, what each requires, and the blocking item with
its owner.

### `/exit-obligations`

Track what survives the close.

```
/exit-obligations
/exit-obligations --due 90d
```

Earn-out milestones, escrow release dates, transition service commitments,
restrictive covenants and their expiry.

**When to use it:** from the day of close until the last obligation expires. A
closed transaction with an untracked earn-out is an unfinished transaction.
**Returns:** each obligation with its date, owner and release condition, and
what falls due in the window. Emits `exit.obligation.tracked`.

### `/exit-proceeds`

Hand the expected proceeds range to Wealth {OS}.

```
/exit-proceeds
```

**When to use it:** once the valuation range exists, and again whenever it
moves materially.
**Returns:** the range net of the deductions this OS can identify, with the
deductions it cannot identify named as open questions for the accountant and
tax professional. Emits `exit.proceeds.expected`, so reserves and long-horizon
goals are planned against a range and not a hope.

---

## What no command does

There is deliberately no command that sends an email to a buyer, opens a data
room to an outside party, drafts or redlines a letter of intent or a purchase
agreement, signs anything, or moves money. Those are human actions taken with a
legally accountable lawyer, accountant, tax professional or licensed
transaction adviser. This OS prepares the operator for the room; it does not
enter the room.

---

## Command summary

| Command | Does |
|---|---|
| `/exit-readiness` | scores sellability, returns the classified gap list |
| `/exit-index` | the diligence readiness index, absence recorded as absence |
| `/exit-value` | an internal valuation range with its assumptions, never a formal valuation |
| `/exit-acquirers` | who plausibly buys this, why, and contact state |
| `/exit-structure` | acceptable terms and the walk-away, written before any offer |
| `/exit-questions <adviser>` | a question pack for the lawyer, accountant or tax professional |
| `/exit-release <doc> <recipient>` | per document, per recipient approval, written to the disclosure log |
| `/exit-timeline` | the gates, and which one is blocking |
| `/exit-obligations` | what survives the close, with dates and release conditions |
| `/exit-proceeds` | the expected range, handed to Wealth {OS} |
