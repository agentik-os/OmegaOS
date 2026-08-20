# Operations {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install operations-os` | Installs this OS into your environment | Once, first |
| `agentik configure operations-os` | Collects the minimum context it needs | After install |
| `agentik run operations-os` | Starts the OS | Every session |
| `agentik doctor operations-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update operations-os` | Updates to the latest version | When a release lands |
| `agentik eval operations-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/ops-scope` | Sets the process boundary | the suspected problem | first trigger, last output, roles, what is out of scope, agreed with the people who run it |
| `/ops-interview` | Interviews the people who do the work | a role and its part of the process | how they actually do it, the workarounds, and where they wait |
| `/ops-observe` | Records a real run | a live run and consent to watch it | the observed sequence with timings, and the gaps against the interviews |
| `/ops-map` | Builds the current-state map | interviews and observation | steps, handoffs, waits, decisions, rework loops, systems touched |
| `/ops-measure` | Puts numbers on the map | volumes and timings | per-step frequency, touch time, wait time, error rate, rework rate, cost per run, with unknowns marked |
| `/ops-waste` | Separates waste from controls | the measured map | the waste list with evidence, and the control list routed for review |
| `/ops-simplify` | Runs the ladder in order | the measured map and the waste list | removed, merged, reordered, simplified or kept, each with a reason |
| `/ops-exceptions` | Enumerates the abnormal runs | recent runs | the exception list and its rate, or a statement that the rate is unknown |
| `/ops-target` | Designs the target operating model | the simplification decisions | the process as it should run, with its controls and exception paths, reachable from today |
| `/ops-readiness` | Issues the automation readiness verdict | the target model and the exception rate | ready, not ready, or ready for part, with reasons, plus the handoff packet for Automation {OS} |

### When to reach for which

- Always start at `/ops-scope`. A diagnosis without an agreed boundary produces
  an argument about what was measured.
- `/ops-observe` is the step people skip, and it is the step that changes the
  answer.
- `/ops-simplify` before `/ops-target`, and `/ops-target` before
  `/ops-readiness`. Out of order, you design a faster version of the waste.
- `/ops-readiness` is the last command in this OS. Anything after it belongs to
  Automation {OS}, Process & SOP {OS} or Team & Delegation {OS}.

## What this OS does not do

There is no build command here, no deploy command, no monitoring command and no
incident command. Automating a process, running it, watching it and recovering
it belong to Automation {OS}. Operations {OS} stops at the verdict and the
handoff packet, on purpose.

## Command summary

| Command | Does |
|---|---|
| `/ops-scope` | agrees the process boundary |
| `/ops-interview` | how the work is actually done |
| `/ops-observe` | a real run, timed |
| `/ops-map` | current state with waits and rework |
| `/ops-measure` | numbers per step, unknowns marked |
| `/ops-waste` | waste separated from controls |
| `/ops-simplify` | eliminate, simplify, then the rest |
| `/ops-exceptions` | the abnormal runs and their rate |
| `/ops-target` | the process as it should run |
| `/ops-readiness` | the verdict and the handoff packet |
