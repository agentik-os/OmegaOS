# Acquisition {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install acquisition-os` | Installs this OS into your environment | Once, first |
| `agentik configure acquisition-os` | Collects the minimum context it needs | After install |
| `agentik run acquisition-os` | Starts the OS | Every session |
| `agentik doctor acquisition-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update acquisition-os` | Updates to the latest version | When a release lands |
| `agentik eval acquisition-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/acquire` | Opens the live deal: stage, blocking condition, today's owner and deliverable | Every working session during a live deal | What is blocking completion right now |
| `/acquire mandate` | Writes or revises the buy box and checks it against financing capacity | Before the first approach, and after every abandoned deal | The buy box with a stated range and reason per field |
| `/acquire search` | Runs the search campaign against the buy box | Weekly while searching | Targets with contact status and a dated next action each |
| `/acquire approach <target>` | Drafts the owner approach for a human to send | When a target is worth first contact | A draft message plus the contact record it will create |
| `/acquire qualify <target>` | Qualifies seller motivation and records the reason for sale | Immediately after the owner responds | The seller's own words, dated, plus qualified or not |
| `/acquire value <target>` | Produces the valuation range and the offer hypothesis | Once the seller has shared real information | A range, every assumption labelled evidenced or unverified |
| `/acquire offer <target>` | Prepares an indication of interest or a letter of intent as a draft | When the offer hypothesis holds and Capital {OS} has approved an amount | A draft marked for legal review and human signature |
| `/acquire exclusivity <target>` | Opens the exclusivity clock and publishes the dated close plan | On acceptance, once financing is evidenced | Workstream, owner, deliverable and date for every day |
| `/acquire close <target>` | Drives the closing checklist condition by condition | Daily during exclusivity | Conditions signed off, conditions open, and the current blocker |
| `/acquire escalate` | Raises a slipping date or a red flag with its options and their cost | The day something slips, not later | The options, their cost, and the decision required |
| `/acquire abandon <target>` | Executes a clean walk away and records why | When a kill criterion fires or a term fails | The withdrawal message draft, the recorded reason and the cost |
| `/acquire handover <target>` | Builds the day one transition pack | At completion | The pack for Portfolio Management {OS} and Board {OS} |

### `/acquire mandate`

```bash
/acquire mandate
/acquire mandate --revise --reason "two targets failed on owner dependency"
```

**Returns:** the buy box with size, sector, geography, owner situation and
financing capacity, each with a range and a reason. Revising it mid search is
allowed and is recorded as a version with its date and reason, so a target is
never retro fitted into a mandate it caused.

### `/acquire qualify <target>`

```bash
/acquire qualify "Meridian Fabrication"
```

**Returns:** the seller's stated reason for selling in their own words, their
timetable, who else is in the process, and a qualified or unqualified verdict.
An unqualified target stops here: valuation work on a business that is not
really for sale is the most common way a search year is lost.

### `/acquire offer <target>`

```bash
/acquire offer "Meridian Fabrication" --type ioi
/acquire offer "Meridian Fabrication" --type loi
```

**Returns:** a draft, clearly marked as a draft for legal review, never as an
executed document. It refuses to produce an offer without an approved
commitment amount from Capital {OS}, and it names the parts of a letter of
intent that are typically binding so a human reads those first.

### `/acquire exclusivity <target>`

```bash
/acquire exclusivity "Meridian Fabrication" --days 60
```

**Returns:** a day by day close plan with an owner and a deliverable per
workstream. It refuses to open the clock while financing is claimed but not
evidenced, and it names exactly which evidence is missing.

### `/acquire escalate`

```bash
/acquire escalate --reason "quality of earnings delayed by seller"
```

**Returns:** the slippage, what it costs in exclusivity days, the options
including the abandon option, and the decision a human must take. Escalation
happens on the day of the slip, because slippage absorbed quietly is discovered
when there is no time left to use it.

## Command summary

| Command | Does |
|---|---|
| `/acquire` | what is blocking the live deal right now |
| `/acquire mandate` | writes or versions the buy box |
| `/acquire search` | works the target list |
| `/acquire approach <target>` | drafts first contact for a human to send |
| `/acquire qualify <target>` | qualifies seller motivation before any valuation work |
| `/acquire value <target>` | valuation range and offer hypothesis |
| `/acquire offer <target>` | drafts an indication of interest or a letter of intent |
| `/acquire exclusivity <target>` | opens the clock and publishes the close plan |
| `/acquire close <target>` | drives the closing checklist |
| `/acquire escalate` | raises slippage or a red flag with its cost |
| `/acquire abandon <target>` | clean walk away, recorded |
| `/acquire handover <target>` | day one transition pack |
