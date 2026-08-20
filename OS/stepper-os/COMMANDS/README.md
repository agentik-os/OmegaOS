# Stepper {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

The deterministic half is the `omega-stepper` CLI, which owns the graph, the
state and the verifier. The reasoning half is the agent, reached through
`/stepper-os` in Claude, the Codex prompt, or the OS master agent. The agent
drives the CLI; it never substitutes its own judgement for a check result.

The first `omega-stepper` run creates its virtualenv under
`~/.omega/os/stepper-os/.venv` and installs the engine. That is a runtime opt
in: the installer never pip-installs.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install stepper-os` | Installs this OS into your environment | Once, first |
| `agentik configure stepper-os` | Collects the minimum context it needs | After install |
| `agentik run stepper-os` | Starts the OS | Every session |
| `agentik doctor stepper-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update stepper-os` | Updates to the latest version | When a release lands |
| `agentik eval stepper-os` | Runs its evaluation suite | Before trusting it |

## Setting up the graph

### `omega-stepper init [--name <project>]`

Scaffold the manifest, the spec tree and a first step.

**When to use it:** once per project, after Blueprint and Design are frozen.
**Returns:** `stepper.yaml` and `stepper/{modules,epics,slices,steps}/`.
Declare both upstream sources under `sources:` before compiling.

### `omega-stepper validate`

Validate specs (schema, uniqueness, references) and the DAG, then audit that
each step's Blueprint and Design references resolve to real documents.

**When to use it:** after every edit to the spec tree, and always before
declaring `BUILD READY`.
**Returns:** errors that block, and warnings that matter: a UI-touching step
citing no design reference is warned about because it means someone will guess.

### `omega-stepper version`

Print the engine version.

**When to use it:** when a behaviour does not match this document.

## Driving execution

### `omega-stepper resume`

Restart-safety reconcile: interrupted RUNNING or VERIFYING attempts drop back
to FAILED so the planner re-offers them.

**When to use it:** first command of every session, without exception.
**Returns:** what was reconciled, and from which attempt.

### `omega-stepper status`

Raw and weighted progress, per-status counts.

**Returns:** a short card. This is the answer to "how much is really done",
and it is computed from the tracker, never from a narrative.

### `omega-stepper plan`

Ranked READY candidates plus the safe execution wave, respecting scope locks
and the work-in-progress limit.

**When to use it:** before claiming anything, and after every close.
**Returns:** the candidates in rank order and the wave that can run in
parallel without two writers on one file.

### `omega-stepper show <STEP-ID>`

Full spec and runtime state of one step.

**Returns:** the four blocks, both reference sets, dependencies, attempts,
review verdicts.

### `omega-stepper agent-brief <STEP-ID>`

Emit the self-contained markdown brief a coding agent executes.

**When to use it:** when dispatching a step to a worker or another agent.
**Returns:** a brief that stands alone: objective, constraints, definition of
done, do not touch, and the Blueprint and Design documents that bind it.

### `omega-stepper start <STEP-ID>`

Claim a READY step: transition to RUNNING and open an attempt.

**Returns:** the brief, or a refusal naming the blocking dependency. There is
no flag that skips the graph.

### `omega-stepper verify <STEP-ID>`

Run the deterministic checks without changing step state.

**When to use it:** mid implementation, to see where you are.
**Returns:** per-check pass or fail with the real command output.

### `omega-stepper done <STEP-ID>`

Verify, then close. DONE only if every check passes.

**Returns:** PASS and the next wave, or FAIL with the evidence a repair must
answer. No self-report closes a step.

### `omega-stepper fail <STEP-ID>`

Mark the current attempt failed, when the agent gave up or hit a hard error.

**Returns:** the attempt count against the ceiling.

### `omega-stepper block <STEP-ID>` and `omega-stepper unblock <STEP-ID>`

Block a step on an external decision or dependency, and lift the block.

**When to use it:** an upstream decision request is outstanding. Blocking keeps
the rest of the wave moving rather than stalling the plan.
**Returns:** the step's new state; unblock returns it to PENDING and the
planner re-offers it.

### `omega-stepper review <STEP-ID> <role> PASS|FAIL --by <name>`

Record a review verdict for a role gate.

**When to use it:** only for a review actually performed. A human gate goes to
the operator, never to an agent.

## Reporting and closing

### `omega-stepper report`

Markdown status report: weighted and raw progress, per-status table.

**Returns:** a document you can hand to someone who was not in the session.

### `omega-stepper events`

Tail of the append-only event log.

**When to use it:** reconstructing what happened, or auditing a claim.

### `omega-stepper release-check`

The release gate: PASS only when every step at the target priority is DONE.

**When to use it:** when the plan looks finished.
**Returns:** PASS or the list of steps that are not DONE. PASS is a statement
about the plan, not about quality, security or shippability.

## Command summary

| Command | Does |
|---|---|
| `omega-stepper init` | scaffold manifest and spec tree |
| `omega-stepper validate` | schema, references, DAG, design-reference audit |
| `omega-stepper resume` | reconcile interrupted attempts |
| `omega-stepper status` | raw and weighted progress |
| `omega-stepper plan` | ranked candidates plus the safe wave |
| `omega-stepper show <id>` | one step, spec plus runtime state |
| `omega-stepper agent-brief <id>` | the self-contained brief for a worker |
| `omega-stepper start <id>` | claim a READY step |
| `omega-stepper verify <id>` | run checks without changing state |
| `omega-stepper done <id>` | verify and close, never self-report |
| `omega-stepper fail <id>` | record a failed attempt |
| `omega-stepper block` / `unblock` | park a step on an external decision |
| `omega-stepper review <id> <role>` | record a role gate verdict |
| `omega-stepper report` | markdown status report |
| `omega-stepper events` | append-only event log tail |
| `omega-stepper release-check` | plan completion at the target priority |
