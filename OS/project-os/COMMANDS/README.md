# Project {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install project-os` | Installs this OS into your environment | Once, first |
| `agentik configure project-os` | Collects the minimum context it needs | After install |
| `agentik run project-os` | Starts the OS | Every session |
| `agentik doctor project-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update project-os` | Updates to the latest version | When a release lands |
| `agentik eval project-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/project-scope` | Writes the scope statement | requester, outcome, constraints | done test, out-of-scope list, assumptions, first risks; refuses to continue without a done test |
| `/project-plan` | Builds the milestone plan | the agreed scope, real capacity | milestones with owner, date, acceptance test, dependencies, and the named critical path |
| `/project-status` | Reports position against plan | evidence of progress since the last report | milestone position, slip in days, blockers, and the next decision due |
| `/project-risk` | Maintains the risk register | new or changed risks | each risk with a trigger, a response, an owner and a review date |
| `/project-change` | Opens a change record | the requested change | cost in time and money, effect on the landing date, options, and the decision to be made |
| `/project-recover` | Presents the recovery option set | current slip and remaining capacity | cut scope, extend, add capacity, or stop, each with its consequence |
| `/project-land` | Runs acceptance and closeout | the met done test | acceptance record, what shipped, what was cut, actual versus planned, retro input |
| `/project-abort` | Stops the project deliberately | the reason to stop | a stop decision, what is salvaged, what is written off, and where the salvage goes |

### When to reach for which

- Before any work: `/project-scope`, then `/project-plan`. Never the reverse.
- On the reporting cadence: `/project-status`. On the day a slip is known, not
  on the milestone date.
- When anything new is asked for: `/project-change`, before it enters the plan.
- When the plan will not hold: `/project-recover`. It always includes stopping.
- At the end: `/project-land`, or `/project-abort`. A project never simply goes
  quiet.

## Command summary

| Command | Does |
|---|---|
| `/project-scope` | done test, out-of-scope, constraints |
| `/project-plan` | milestones, owners, dependencies, critical path |
| `/project-status` | position against plan, slip, next decision |
| `/project-risk` | risks with triggers and responses |
| `/project-change` | prices a change before it is accepted |
| `/project-recover` | the four options when the plan will not hold |
| `/project-land` | acceptance and closeout |
| `/project-abort` | a deliberate, recorded stop |
