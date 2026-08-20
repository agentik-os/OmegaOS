# Execution {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install execution-os` | Installs this OS into your environment | Once, first |
| `agentik configure execution-os` | Collects the minimum context it needs | After install |
| `agentik run execution-os` | Starts the OS | Every session |
| `agentik doctor execution-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update execution-os` | Updates to the latest version | When a release lands |
| `agentik eval execution-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/boot` | Opens the day: capacity, usable minutes, one must-win, the day's commitments | capacity GREEN, AMBER or RED, usable minutes, must-win | the daily command card, and the overflow if the day does not fit |
| `/capture` | Records anything incoming without judging it | one line, plus its source | an inbox entry |
| `/clarify` | Classifies the inbox: commitment, project, delegation, reference, dropped | the inbox | an empty inbox and a set of routed items |
| `/commit` | Turns an intention into a commitment | outcome, deadline | a commitment with an acceptance test and one physical next action, or a refusal naming what is missing |
| `/focus` | Protects a block on exactly one commitment | commitment, 25, 50 or 90 minutes | an open block, and a warning if another block is already open |
| `/prove` | Closes a commitment on evidence | commitment, evidence, acceptance test | a completion record, or `TOUCHED` if the evidence is absent |
| `/recover` | Handles a commitment that failed | commitment, reason | a classification (blocked, deferred, cancelled, delegated) plus a physical next action |
| `/promise` | Records a promise made to another person | who, what, by when, consequence | a promise ledger entry with a notice-by date |
| `/capacity` | Shows or sets the capacity budget | optional new capacity | committed minutes against usable minutes, and the overflow |
| `/halt` | Closes the day | classification, energy, focus, friction, proof, tomorrow's first action | the halt card; refuses to close without tomorrow's first action |
| `/reset` | The weekly reset | the week's ledger | the honest truth, next week's single win, one system experiment |
| `/audit` | The monthly system audit | the month's ledger | one change to the system, plus the test that will tell you if it worked |

### When to reach for which

- Every morning: `/boot`. Every evening: `/halt`. Those two are the system.
- During the day: `/capture` for anything arriving, `/focus` to work,
  `/prove` when something is genuinely finished.
- When something goes wrong: `/recover`, never silence. A commitment left
  silently open is the state this OS is built to prevent.
- Weekly: `/reset`. Monthly: `/audit`. Both send their output to Review &
  Governance {OS}.

## The deterministic engine

The state file is owned by the `omega-execution` CLI, which runs without a
model and without a network. The slash commands above are the coaching layer
over the same state.

```bash
omega-execution init --owner "You"
omega-execution boot --capacity GREEN --usable-minutes 240 --must-win "Ship X"
omega-execution focus <commitment> --minutes 50
omega-execution complete <commitment> --kind ship --evidence "..." --acceptance "..."
omega-execution halt --classification SHIPPED --tomorrow "..."
omega-execution reset ...      # weekly
omega-execution audit ...      # monthly
```

State defaults to `~/.omega/os/execution-os/ledger/execution-state.json`.

## Command summary

| Command | Does |
|---|---|
| `/boot` | opens the day against a capacity budget |
| `/capture` | records an incoming item, unjudged |
| `/clarify` | empties the inbox by classifying it |
| `/commit` | creates a commitment with a defined next action |
| `/focus` | protects one block for one commitment |
| `/prove` | closes a commitment on evidence |
| `/recover` | classifies a failure and names the next action |
| `/promise` | records a promise with a notice-by date |
| `/capacity` | shows committed minutes against usable minutes |
| `/halt` | closes the day with a proof and tomorrow's first action |
| `/reset` | the weekly truth and one system experiment |
| `/audit` | the monthly system change |
