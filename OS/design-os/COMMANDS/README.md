# Design {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

Design has two halves. The reasoning half is the agent, reached through
`/design-os` in Claude, the Codex prompt, or the OS master agent. The
deterministic half is the `omega-designer` CLI, which owns the two schema
validators that gate the handoff. A handoff is not ready until it validates,
whatever the pack says about itself.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install design-os` | Installs this OS into your environment | Once, first |
| `agentik configure design-os` | Collects the minimum context it needs | After install |
| `agentik run design-os` | Starts the OS | Every session |
| `agentik doctor design-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update design-os` | Updates to the latest version | When a release lands |
| `agentik eval design-os` | Runs its evaluation suite | Before trusting it |

## Session commands

### `/design-os` or `/design`

Open the compiler. It reads the pinned Blueprint handoff, detects whether a
design pack already exists and proposes a mode.

**When to use it:** immediately after `BLUEPRINT COMPLETE, STEPPER READY` on a
product with a UX surface.
**Returns:** the detected situation, the proposed mode, and the coverage map
from Blueprint requirements to user outcomes.

### `full`

Run every pass and emit the complete pack.

**When to use it:** the default for a new product surface.
**Returns:** the 15 part design pack and `design-handoff.json`, with readiness
either blocked and explained, or `STEPPER_READY`.

### `audit [--url <url>] [--path <dir>]`

Challenge an existing design, prototype or shipped UI against the gates.

**When to use it:** inherited product, redesign, or a design nobody has ever
stress tested.
**Returns:** gaps by gate, ranked findings, and a repair handoff that Stepper
{OS} can plan against.

### `flow <FLOW-###|name>`

Work selected journeys without recompiling the product.

**When to use it:** one journey is wrong and the rest is settled.
**Returns:** the before and after path, the edge states that were missing, and
the updated traceability for the touched requirements.

### `ai-app`

Prioritise the AI interaction surface: composer, context window, thinking,
tool and source rendering, branching, streaming, stop, retry, reconnect,
artifacts, memory transparency and write confirmation.

**When to use it:** the product contains a chat, an agent, generated artifacts
or model selection.
**Returns:** the `INT-###` contracts for every AI state, each with a named,
persistent rendering.

### `stax-fit`

Decide whether, where and how to use the STAX panel model, without designing
the whole product.

**Returns:** the fitness verdict, the rejected shells with reasons, and the
panel grammar if STAX wins.

### `revision <BLUEPRINT-DELTA|DDEC-###>`

Update only the impacted IDs and contracts after an upstream change.

**When to use it:** Blueprint {OS} cut a new version, or a design decision was
reversed.
**Returns:** the impacted contract list, the updated records, and confirmation
that no existing ID was renumbered.

## Deterministic commands

The `omega-designer` CLI, stdlib Python, no virtualenv.

### `omega-designer intake <blueprint-intake.json>`

Validate the Blueprint intake before designing anything.

**When to use it:** first, every time. Designing against a malformed or
unpinned intake wastes the whole pass.
**Returns:** the schema verdict and the missing fields by name.

### `omega-designer handoff <design-handoff.json>`

Validate the Design Handoff: flows, surfaces, states, evals, stepperSeeds,
readiness.

**When to use it:** before claiming any readiness, and again before freezing.
**Returns:** the schema verdict with the failing path named. Readiness stays
blocked until this passes.

### `omega-designer self-test`

Run the validator's own self-test.

**When to use it:** after install, or when a validation result looks wrong.

## Command summary

| Command | Does |
|---|---|
| `/design-os` | open the compiler, detect the mode |
| `full` | every pass, complete pack plus handoff |
| `audit` | challenge an existing design, emit a repair handoff |
| `flow <id>` | resolve selected journeys with full edge states |
| `ai-app` | compile the AI interaction surface |
| `stax-fit` | decide the navigation shell |
| `revision <delta>` | update only the impacted contracts |
| `omega-designer intake` | validate the Blueprint intake |
| `omega-designer handoff` | validate the Design Handoff for Stepper |
| `omega-designer self-test` | check the validators themselves |
