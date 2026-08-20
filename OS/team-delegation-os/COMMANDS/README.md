# Team & Delegation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install team-delegation-os` | Installs this OS into your environment | Once, first |
| `agentik configure team-delegation-os` | Collects the minimum context it needs | After install |
| `agentik run team-delegation-os` | Starts the OS | Every session |
| `agentik doctor team-delegation-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update team-delegation-os` | Updates to the latest version | When a release lands |
| `agentik eval team-delegation-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/delegate-pick` | Splits the load | the current work list | the delegate list and the keep list, each item with a reason, and a challenge to every reason that is habit |
| `/delegate-match` | Chooses the person and the authority level | the task, the candidates, their evidence | the person, the authority level, and the reason both were chosen |
| `/delegate-brief` | Writes the brief | the task, the person, the authority level | outcome, constraints, definition of done, do-not-touch, deadline, check-ins, escalation path |
| `/delegate-confirm` | Tests that the brief landed | the receiver's restatement | confirmation, or the specific part of the brief that did not survive the handoff |
| `/delegate-check` | Runs a scheduled check-in | progress since the last point | on course, corrected, or blocked, with the action and who owns it |
| `/delegate-receive` | Judges returned work | the work and the brief | accepted, or one complete list of what is missing against the definition of done |
| `/delegate-feedback` | Writes the correction | the gap between the work and the brief | an observable behaviour, a specific alternative, and what will be different next time |
| `/delegate-level` | Changes authority | the evidence since the last change | raised or lowered, with the evidence and the message to the person |
| `/delegate-recall` | Takes the work back | the reason: capacity, priority or fit | a recall record, the new owner, and what the person is told |

### When to reach for which

- `/delegate-pick` when the load is the problem. It usually finds work that
  should be deleted rather than delegated, which goes to Operations {OS}.
- `/delegate-brief` then `/delegate-confirm`, always as a pair. The confirm step
  is where most failed delegations are caught, before they cost anything.
- `/delegate-receive` before `/delegate-feedback`. Judge the work first, then
  correct, and never mix a rejection with a new requirement.
- `/delegate-level` on evidence, on a cadence, in both directions.

## The three-brief rule

If `/delegate-brief` writes essentially the same brief for the third time, the
OS says so and routes it to Process & SOP {OS}. A fourth identical brief is a
procedure that nobody has written down.

## Command summary

| Command | Does |
|---|---|
| `/delegate-pick` | delegate list and keep list, with reasons |
| `/delegate-match` | the person and the authority level |
| `/delegate-brief` | the six required parts of a brief |
| `/delegate-confirm` | proves the brief actually landed |
| `/delegate-check` | an agreed check-in, never a surprise |
| `/delegate-receive` | accept, or name everything missing once |
| `/delegate-feedback` | correction that changes the next result |
| `/delegate-level` | authority moves on evidence |
| `/delegate-recall` | takes work back without a verdict |
