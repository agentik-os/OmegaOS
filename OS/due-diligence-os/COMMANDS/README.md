# Due Diligence {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install due-diligence-os` | Installs this OS into your environment | Once, first |
| `agentik configure due-diligence-os` | Collects the minimum context it needs | After install |
| `agentik run due-diligence-os` | Starts the OS | Every session |
| `agentik doctor due-diligence-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update due-diligence-os` | Updates to the latest version | When a release lands |
| `agentik eval due-diligence-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | When to use it | What it returns |
|---|---|---|---|
| `/diligence plan <target>` | Scopes the diligence by decision relevance and sets the time and cost budget | Before a single document is requested | The question list with each question's decision relevance, the dropped questions with reasons, and the budget |
| `/diligence request <target>` | Builds the information request list with owner, format and due date | Once the plan exists | A request list ready for a human to send, never sent by this OS |
| `/diligence chase <target>` | Reports what is overdue, what was refused and what is blocking a stream | Weekly during an active diligence | Overdue items by age, refusals with dates, and the streams they are blocking |
| `/diligence workstream <target> <stream>` | Opens or updates one stream: commercial, financial, legal, technical, people, references | When a stream can start on the material or sources available | The stream's working paper with questions answered, unanswered and refused |
| `/diligence evidence <target>` | Logs an answer with its source, date, confidence and seller or independent classification | Every time an answer arrives, before it is used anywhere | The stored evidence entry, and a warning if the only source is the seller |
| `/diligence finding <target>` | Registers a finding with a severity and an explicit consequence class | When evidence contradicts a claim or reveals a risk | The register entry with severity, evidence reference and consequence, and `diligence.finding.registered` |
| `/diligence redflag <target>` | Raises a stopping escalation and pauses the deal calendar | The moment something could end the deal | The escalation with evidence attached, a paused calendar, and `diligence.redflag.raised` |
| `/diligence gaps <target>` | Lists everything unverified, refused or resting only on a seller source | Before any commitment decision, and again before close | The unverified list with severities, and the assertions still labelled as assertions |
| `/diligence conditions <target>` | Derives the conditions that must be satisfied before completion | Once the findings register is stable | Conditions with owners and due dates, ready for Acquisition {OS} |
| `/diligence close <target>` | Produces the diligence report and closes the plan | When the questions are answered, the budget is spent, or a stop is called | The report including the list of what could not be verified, plus `diligence.completed` |

---

### `/diligence plan <target>`

The command that decides how the next three weeks are spent. Each question must
name the decision it could change. Questions that fail the test are dropped in
front of you, with the reason, rather than silently surviving into a request
list.

```bash
/diligence plan northwind
/diligence plan northwind --budget "3 weeks, 18k advisers"
/diligence plan northwind --from-thesis northwind-acquisition
```

**Returns:** the scoped question list, the dropped questions with reasons, the
budget, and the event `diligence.plan.set`. Run it even if a request list is
already out: it will tell you which of the answers you are waiting for actually
matter.

### `/diligence request <target>`

Builds the information request list: item, owner, format, due date, and the
stream it unblocks.

```bash
/diligence request northwind
/diligence request northwind --stream financial
```

**Returns:** a request list prepared for a human to send. **This OS has no
send.** Transmission to the counterparty or their advisers is an explicit human
act, by a person who has read the list.

### `/diligence evidence <target>`

The discipline that makes the rest of it worth anything. Every answer is logged
with a named source, a date, a confidence level and a classification of seller
sourced or independent.

```bash
/diligence evidence northwind \
  --question "top customer concentration" \
  --answer "61 percent of FY revenue in one account" \
  --source "system extract observed on screen, 2026-08-12" \
  --confidence directly-observed \
  --independent
```

**Returns:** the stored entry. If the only source is the seller, the entry is
recorded as an assertion with the speaker and date, and it appears in
`/diligence gaps` until something independent corroborates it. High confidence
on a single seller document is refused as a contradiction.

### `/diligence finding <target>`

A finding is not a note. It carries a severity and states what it does to the
deal: price, structure, condition or walk. It never carries a clause, a number
of basis points or an instrument, because that is Deal Structuring {OS}.

```bash
/diligence finding northwind \
  --severity high \
  --evidence EV-014 \
  --consequence condition \
  --statement "the top account has a 30 day termination right and no renewal"
```

**Returns:** the register entry and `diligence.finding.registered`. A finding
with no assignable consequence is held out of the register as an open
observation and escalated for a relevance decision.

### `/diligence redflag <target>`

The path that exists so a deal ending fact does not arrive as a paragraph on
page nine of a report published after exclusivity.

```bash
/diligence redflag northwind --evidence EV-022 \
  --statement "undisclosed litigation from a former distributor"
```

**Returns:** the escalation with evidence attached, and the deal calendar
paused. **Human approval gate:** only a person can clear or downgrade a red
flag, and only a person resumes the calendar. The decision to continue,
restructure or stop is recorded with its date.

### `/diligence gaps <target>`

Run this before any commitment decision. It lists what could not be verified,
what was refused, and what still rests only on management assertion.

```bash
/diligence gaps northwind
```

**Returns:** the unverified list with severities. Absence of evidence is
reported as absence. Nothing on this list has ever been scored as a pass.

### `/diligence close <target>`

Produces the report. The list of what could not be verified is a required
section and is not moved to an appendix.

```bash
/diligence close northwind
/diligence close northwind --stopped   # budget spent or a stop was called
```

**Returns:** the report, the conditions to completion with owners, and
`diligence.completed`. Items awaiting a named professional's written opinion
stay open and are listed as such. **Human approval gate:** the report is not
transmitted to anyone by this OS, and declaring diligence complete is a human
decision.
