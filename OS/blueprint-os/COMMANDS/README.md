# Blueprint {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

Blueprint has two halves. The deterministic half is the `omega-blueprint` CLI,
which owns the state file and the validator. The reasoning half is the agent,
reached through `/blueprint-os` in Claude, the Codex prompt, or the OS master
agent. The CLI never reasons and the agent never claims progress the CLI has
not validated.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install blueprint-os` | Installs this OS into your environment | Once, first |
| `agentik configure blueprint-os` | Collects the minimum context it needs | After install |
| `agentik run blueprint-os` | Starts the OS | Every session |
| `agentik doctor blueprint-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update blueprint-os` | Updates to the latest version | When a release lands |
| `agentik eval blueprint-os` | Runs its evaluation suite | Before trusting it |

## Session commands

Typed to the agent, in any mode.

### `/blueprint <idea>`

Start a `NEW` compile from an idea plus whatever context is reachable.

**When to use it:** at the start of a product, before anyone opens an editor.
**Returns:** an initialised state file, the first compiled sections, and the
open question set (at most three).

### `/blueprint-os`

Open the compiler without committing to a mode. It reads the project, detects
whether a pack already exists, and proposes `NEW`, `RECOVER`, `EXTEND`,
`REVISE`, `AUDIT` or `DELTA`.

**Returns:** the detected situation and the proposed mode, for you to confirm.

### `recover`

Rebuild canonical truth from prior sources: old specs, tickets, code, notes.

**When to use it:** the project exists, its definition does not.
**Returns:** recovered records, each labelled and traced to where it was found,
plus the conflicts recovery surfaced.

### `extend <module>`

Add a capability or module to a frozen pack.

**When to use it:** new scope on a defined product.
**Returns:** new IDs, no renumbering of existing ones, and the impact set:
which existing records the addition touches.

### `revise <id>`

Supersede a record and propagate.

**Returns:** the superseding record with history preserved, and every dependent
record updated or flagged for a decision.

### `audit`

Evaluate the pack without changing it.

**Returns:** gaps, orphans, conflicts, and the G01 to G20 verdicts with the
failing assertion named.

### `delta <version-a> <version-b>`

Semantic diff between two frozen versions.

**Returns:** what changed, by record, classified by blast radius, and which
downstream artifacts must be re-read.

### `continue`

Resume exactly at the continuation pointer stored by the last checkpoint.

**When to use it:** after a compaction, a restart or a new session.
**Returns:** the state it resumed from, then the work.

## Deterministic commands

The `omega-blueprint` CLI, stdlib Python, no virtualenv.

### `omega-blueprint init <state.json>`

Initialise a canonical state file.

```bash
omega-blueprint init blueprint/state.json \
  --project-id my-app --project-name "My App" \
  --namespace my.app --request "Compile the blueprint"
```

**Returns:** the created state file and its initial revision.

### `omega-blueprint validate <state.json>`

Validate structure and semantic invariants: ID integrity, classification,
traceability, gate preconditions.

**When to use it:** before any claim of progress, and always before freezing.
**Returns:** the issue list by severity. **Exit 1 on any critical or high
issue**, which outranks every narrative claim.

### `omega-blueprint status <state.json>`

Concise status: revision, section coverage, gate verdicts, open unknowns and
conflicts.

**Returns:** a short card, phone readable.

### `omega-blueprint checkpoint <state.json>`

Advance the revision and save the continuation pointer.

```bash
omega-blueprint checkpoint blueprint/state.json \
  --current "compiled sections 1 to 12" --next "domain invariants"
```

**When to use it:** before any context compaction, long pass or handover.
**Returns:** the new revision and the stored pointer.

### `omega-blueprint demo`

Print a valid minimal state, to read rather than to keep.

**When to use it:** learning the shape of the contract, or checking the CLI
works at all.

## Command summary

| Command | Does |
|---|---|
| `/blueprint <idea>` | start a new compile |
| `/blueprint-os` | open the compiler, detect the right mode |
| `recover` | rebuild canonical truth from prior sources |
| `extend <module>` | add scope, preserve IDs, report impact |
| `revise <id>` | supersede a record and propagate |
| `audit` | gaps, orphans, conflicts, gate verdicts |
| `delta <a> <b>` | semantic diff plus blast radius |
| `continue` | resume at the continuation pointer |
| `omega-blueprint init` | create the canonical state file |
| `omega-blueprint validate` | validate, exit 1 on critical or high |
| `omega-blueprint status` | concise status card |
| `omega-blueprint checkpoint` | advance revision, store continuation |
| `omega-blueprint demo` | a valid minimal state to read |
