# Meeting {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install meeting-os` | Installs this OS into your environment | Once, first |
| `agentik configure meeting-os` | Collects the minimum context it needs | After install |
| `agentik run meeting-os` | Starts the OS | Every session |
| `agentik doctor meeting-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update meeting-os` | Updates to the latest version | When a release lands |
| `agentik eval meeting-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/meeting-test` | Decides whether the meeting should exist | the proposed purpose, attendees, duration | hold, shrink, replace with async, or decline, with the reason and the person-hour cost |
| `/meeting-agenda` | Builds the agenda | the decisions to be made, the attendees | items with one decision, one decider and one time box each; items with no decision are removed |
| `/meeting-brief` | Assembles the pre-read | the facts, numbers and options | a circulated pre-read, plus any prior decision on the same topic |
| `/meeting-run` | Runs the room to decisions | the agenda, live | decisions taken, and parked items with their unblock condition |
| `/meeting-record` | Writes the record | what was decided | decision record with rationale and rejected alternatives, plus action items with one owner and one date |
| `/meeting-followup` | Closes the loop before the next occurrence | the open actions | each action closed, renegotiated with a new date, or reassigned; nothing silently carried |
| `/meeting-audit` | Judges a recurring meeting | its last several occurrences | keep, shrink, merge or kill, with the evidence, and the cost of keeping it |
| `/meeting-decline` | Writes the refusal | the request and the async alternative | a decline that names what will be delivered instead and by when |

### When to reach for which

- Before anything is scheduled: `/meeting-test`. It is the only command that
  can save the whole cost.
- Once justified: `/meeting-agenda`, then `/meeting-brief`, circulated ahead.
- After: `/meeting-record` immediately, while the room still agrees on what
  happened.
- Between occurrences: `/meeting-followup`. Not at the start of the next one.
- On the review date of any recurring meeting: `/meeting-audit`.

## Command summary

| Command | Does |
|---|---|
| `/meeting-test` | hold, shrink, async, or decline |
| `/meeting-agenda` | one decision and one decider per item |
| `/meeting-brief` | the pre-read, circulated before |
| `/meeting-run` | drives the room to decisions |
| `/meeting-record` | decision record and owned actions |
| `/meeting-followup` | closes actions before the next occurrence |
| `/meeting-audit` | keep, shrink, merge or kill, on evidence |
| `/meeting-decline` | a refusal with a real alternative |
