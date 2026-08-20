# Board {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install board-os` | Installs this OS into your environment | Once, first |
| `agentik configure board-os` | Collects the minimum context it needs | After install |
| `agentik run board-os` | Starts the OS | Every session |
| `agentik doctor board-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update board-os` | Updates to the latest version | When a release lands |
| `agentik eval board-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/board` | Opens the governance state: next meeting, pack status, open actions, overdue items | Any working session on governance | What is late, what is open, what needs a decision |
| `/board constitute` | Builds composition, committees, terms of reference and the authority matrix | When a board exists on paper but not in practice | The matrix with every decision class assigned, and the gaps it found |
| `/board authority` | Reads, tests or amends the delegated authority matrix | When anyone asks whether they can decide something alone | The class, who owns it, and whether the proposed decision is inside it |
| `/board calendar` | Lays out the annual board cycle with standing and annual items | At the start of the year, and after any structural change | Each meeting with its purpose and the once a year items placed |
| `/board pack` | Assembles and issues the board pack at the stated notice | Before every meeting, keyed to the notice deadline | The pack, the issue timestamp, and any paper that is missing |
| `/board agenda` | Builds the agenda ordered by decision, with open actions first | Once the pack is assembled | The agenda, plus the items moved to the written section |
| `/board meeting` | Captures decisions, dissent, conflicts and actions live | During the meeting | The decision record as it is built, with gaps flagged in the room |
| `/board minute` | Drafts minutes and resolutions for human approval | Immediately after the meeting | Drafts, clearly marked as drafts, routed for approval |
| `/board actions` | Maintains and chases the action register | Between meetings, weekly | Open actions by owner and date, and anything rolled over |
| `/board oversight` | Runs the oversight test with evidence | Between meetings, or when governance feels ceremonial | Whether risks are tracked, actions close, and management is challenged |
| `/board effectiveness` | Runs the board effectiveness review | Annually, or after a governance failure | Findings, each with an owner and a date |
| `/board conflicts` | Maintains the interests and conflicts register | On appointment, and before any related party item | Declared interests and which director must withdraw from which item |

### `/board authority`

```bash
/board authority --test "sign a 3 year lease over 200k"
/board authority --amend "capital expenditure above 50k moves to the board"
```

**Returns:** the decision class, who owns it, and whether the proposed decision
sits inside the matrix. A decision class that is not in the matrix returns as a
gap, is escalated to the board, and produces a proposed amendment. It is never
resolved by assuming the decision was fine because it worked.

### `/board pack`

```bash
/board pack --meeting 2026-09-18 --notice 7d
/board pack --status
```

**Returns:** the assembled pack, its issue timestamp against the notice
deadline, and any paper that has not arrived. If the deadline cannot be met, it
proposes deferring the affected items rather than issuing late, because a pack
read in the room produces ratification, not oversight. Issuing the pack is a
human approval step.

### `/board agenda`

```bash
/board agenda --meeting 2026-09-18
```

**Returns:** an agenda ordered by decision, open actions first, and a list of
papers moved to the written section because they ask for no decision. It states
the time allocated per decision, so the largest decision is not the one taken
last with the clock running.

### `/board minute`

```bash
/board minute --meeting 2026-09-18
```

**Returns:** draft minutes and any draft resolutions, marked as drafts on every
page. It never records a resolution as adopted, and it routes the approved
version to the company secretary who holds the statutory record and makes any
filing.

### `/board oversight`

```bash
/board oversight --period H1
```

**Returns:** the evidence based test. Are the risks on the register moving. Do
actions close or roll. Is there any record of management being challenged and
changing course. An action that has rolled three meetings is reported as an
oversight failure, not as a status update.

## Command summary

| Command | Does |
|---|---|
| `/board` | what is late, open, or needs deciding |
| `/board constitute` | composition, committees, terms of reference, authority matrix |
| `/board authority` | reads, tests or amends the authority matrix |
| `/board calendar` | the annual cycle with items placed |
| `/board pack` | assembles and issues at the notice period |
| `/board agenda` | ordered by decision, actions first |
| `/board meeting` | captures decisions, dissent, conflicts, actions |
| `/board minute` | drafts minutes and resolutions for approval |
| `/board actions` | the action register, chased |
| `/board oversight` | is this oversight or ceremony |
| `/board effectiveness` | the annual review, with owned findings |
| `/board conflicts` | interests and conflicts, before the item |
