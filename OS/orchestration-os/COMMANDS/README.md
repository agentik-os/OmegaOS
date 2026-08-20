# Orchestration {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install orchestration-os` | Installs this OS into your environment | Once, first |
| `agentik configure orchestration-os` | Collects budget ceilings, ledger location and escalation channels | After install |
| `agentik run orchestration-os` | Starts the OS | Every session |
| `agentik doctor orchestration-os` | Checks config, ledger readability, worker reachability and dependencies | When something is off |
| `agentik update orchestration-os` | Updates to the latest version | When a release lands |
| `agentik eval orchestration-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The OS answers to `/orchestrate`.

### `/orchestrate shape <mission>`

Draw the topology and justify every edge.

**When to use it:** before any dispatch, on any mission with more than two steps.
**Returns:** nodes, edges, and for each edge the data that actually moves along
it. Edges where nothing moves are deleted and the chain widens. Any barrier is
returned with which of the three legal reasons justifies it, or removed.

### `/orchestrate plan <mission>`

Write the durable ledger.

**When to use it:** immediately after shaping, before the first dispatch.
**Returns:** one entry per ask in the requester's own order, persisted to a file,
with owner and state. Prose plans are refused: the file is the mission state.

### `/orchestrate dispatch`

Start the ready set concurrently.

**When to use it:** when the ledger has entries whose dependencies are satisfied.
**Returns:** the steps started, their owners, their budgets, their scope claims,
and the verification command each will be judged by. Overlapping file scopes are
serialised or isolated before anything starts.

### `/orchestrate watch`

Classify running steps and act on the classification.

**When to use it:** for the duration of any mission running unattended.
**Returns:** per step: working (no action), stalled (nudged), blocked (escalated,
never nudged), or asking (escalated to a human, never answered on their behalf),
each with the evidence for the classification.

### `/orchestrate verify <task>`

Run the verification the brief named and record the result.

**When to use it:** every time a step claims completion, before its ledger entry
moves.
**Returns:** the command, its output, and pass or fail, recorded separately from
the delegate's own claim so the two can visibly disagree.

### `/orchestrate synthesise`

Assemble every child output into one result.

**When to use it:** after a fan out, always. A fan out with no synthesis is
unfinished work.
**Returns:** one coherent result, with every child output either represented or
explicitly discarded with a reason, and null results from failed nodes accounted
for rather than skipped.

### `/orchestrate close`

Signal the mission's end and shut it down cleanly.

**When to use it:** when every ledger entry is done and verified, or when the
mission must end without that being true.
**Returns:** clean, pending with exactly what remains, or failed with the
evidence. It refuses clean while any worker it started is still running, releases
every scope claim, and is safe to run more than once.

### `/orchestrate resume <mission>`

Continue an interrupted mission from its persisted ledger.

**When to use it:** after any interruption, restart or compaction.
**Returns:** the ledger as it stands and the first entry that is not done. It
resumes from the file, never from a recollection of what was happening.

### `/orchestrate postmortem <mission>`

Turn a failure or an overrun into a change.

**When to use it:** after any mission that failed, overran its budget, or dropped
an ask.
**Returns:** the cause, and the specific change to the topology or the ledger
discipline. A resolution to be more careful is not an accepted output.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/orchestrate shape` | shape | the topology, every edge justified |
| `/orchestrate plan` | plan | the persisted ledger, one entry per ask |
| `/orchestrate dispatch` | dispatch | start the ready set, claims held |
| `/orchestrate watch` | watch | four state classification per running step |
| `/orchestrate verify` | verify | run the done test yourself |
| `/orchestrate synthesise` | synthesise | one result from every child output |
| `/orchestrate close` | close | clean, pending or failed, and nothing left running |
| `/orchestrate resume` | plan | continue from the file, not from memory |
| `/orchestrate postmortem` | postmortem | the shape change the failure earned |
