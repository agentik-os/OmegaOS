# Client {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install client-os` | Installs this OS into your environment | Once, first |
| `agentik configure client-os` | Collects the minimum context it needs | After install |
| `agentik run client-os` | Starts the OS | Every session |
| `agentik doctor client-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update client-os` | Updates to the latest version | When a release lands |
| `agentik eval client-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/client-open` | Writes the expectation record | what was sold, who decides, the cadence | inclusions, exclusions, response times, escalation path, and the confirmation message to send |
| `/client-brief` | Maintains the one-page account brief | the ledger and recent history | what they bought, cadence, decider, red lines, exceptions granted, current health |
| `/client-update` | Drafts the cadence update | project position from Project {OS} | a status update in the client's language, with the next update date; refuses to omit a known slip |
| `/client-bad-news` | Drafts an early bad-news message | the slip or the failure, and the recovery option chosen | what happened, what is being done, what changes for them, and by when |
| `/client-boundary` | Answers an out-of-scope request | the request and the agreement | included, extra with a price, or refused with a reason and an alternative |
| `/client-exception` | Records a granted exception | the exception and the reason | a dated ledger entry, and the precedent risk if it is repeated |
| `/client-strain` | Diagnoses a struggling relationship | the signals: silence, tone, escalation, payment | the named cause, a specific repair action, an owner and a date |
| `/client-health` | Produces the health read | the account history and current signals | green, watch or at risk, with the evidence and the risk of loss |
| `/client-close` | Runs offboarding or non-renewal | the ending and its reason | what was delivered, what is handed over, what is owed, and what happens next |

### When to reach for which

- Before any work: `/client-open`. An engagement that starts without it will
  spend the rest of its life arguing about scope.
- Every cadence period: `/client-update`, on time, even when nothing changed.
- The day a slip is known: `/client-bad-news`. Not the day it lands.
- Any new request: `/client-boundary`, then `/client-exception` if it is granted.
- On the review cycle: `/client-health`, and `/client-strain` on anything that
  is not green.

## Send boundary

Every command in this OS produces a draft. The human reads it and sends it.
Client {OS} never sends, never commits a date, and never grants an exception on
its own.

## Command summary

| Command | Does |
|---|---|
| `/client-open` | expectations, confirmed in writing |
| `/client-brief` | the one page anyone can pick up |
| `/client-update` | the cadence update, in their language |
| `/client-bad-news` | early, with the recovery |
| `/client-boundary` | included, extra, or refused |
| `/client-exception` | records what was granted, and when |
| `/client-strain` | cause, repair, owner, date |
| `/client-health` | green, watch, at risk, with evidence |
| `/client-close` | a deliberate ending, not a fade |
