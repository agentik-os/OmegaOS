# Builder {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

The deterministic half is the `omega-builder` CLI (stdlib Python, no
virtualenv), which owns Builder state, the evidence ledger and the BG gates.
The reasoning half is the agent, reached through `/build` or `/builder-os` in
Claude, the Codex prompt, or the OS master agent. Builder drives Stepper for
everything about the plan: it never keeps a second one.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install builder-os` | Installs this OS into your environment | Once, first |
| `agentik configure builder-os` | Collects the minimum context it needs | After install |
| `agentik run builder-os` | Starts the OS | Every session |
| `agentik doctor builder-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update builder-os` | Updates to the latest version | When a release lands |
| `agentik eval builder-os` | Runs its evaluation suite | Before trusting it |

## Session commands

### `/build` or `/builder-os`

Open the implementation runtime. It verifies fingerprints, inspects the
repository, reconciles interrupted attempts and reports from the tracker.

**Returns:** the session preflight: state, fingerprints, working tree, open
attempts, and the ready set from Stepper.

### `preflight`

Check readiness without claiming anything.

**When to use it:** at the start of every session, and before claiming a step
after a long gap.
**Returns:** fingerprint match or mismatch, working tree status, unreconciled
attempts, and whether the claimed contract still holds against the repository.

### `step <STEP-ID>`

Run one full step transaction: claim, hydrate, preflight, micro-plan,
implement, verify, repair, review, integrate, evidence, done.

**When to use it:** the everyday unit of work.
**Returns:** the closed step with its evidence, or the blocker that stopped it.

### `repair <STEP-ID>`

Correct a failing step against the printed evidence, under the bounded ceiling.

**Returns:** the passing check, or the escalation record with every attempt
attached.

### `resume`

Reconcile after an interruption and continue from real state.

**When to use it:** first thing after any crash, compaction or handover.
**Returns:** what was reconciled and what is now claimable.

### `report`

Evidence-backed status of the build.

**Returns:** steps done and open, gate results, and the checks that produced
them. Never a narrative claim of progress.

## Deterministic commands

The `omega-builder` CLI. State lives in a JSON file you name.

### `omega-builder init <state.json>`

Initialise canonical Builder state for a project.

**Returns:** the created state file with the project identity and the pinned
upstream fingerprints.

### `omega-builder validate <state.json>`

Validate structure and semantic invariants.

**When to use it:** before trusting any status, and after any manual edit.
**Returns:** the issue list. A failing validate outranks any narrative.

### `omega-builder status <state.json>`

Evidence-backed status.

**Returns:** per-step state, attempts, recorded checks and gate results, from
the ledger rather than from memory.

### `omega-builder sync-step <state.json> <STEP-ID>`

Import or refresh one Stepper step mirror.

**When to use it:** before claiming, and after any change to the Stepper graph.
**Returns:** the mirrored contract and its current Stepper status.

### `omega-builder claim <state.json> <STEP-ID>`

Claim a READY Stepper step.

**Returns:** the open attempt, or a refusal when the step is not READY.

### `omega-builder transition <state.json> <STEP-ID> <state>`

Apply a valid Builder attempt transition.

**Returns:** the new attempt state, or a refusal when the transition is not
legal from the current one.

### `omega-builder record-check <state.json> <STEP-ID> ...`

Record deterministic check evidence: the command and its real output.

**When to use it:** every time a check runs. This is the ledger the whole OS
rests on.
**Returns:** the stored evidence entry.

### `omega-builder mark-step <state.json> <STEP-ID> <status>`

Mirror a Stepper status after an external verifier decision.

**When to use it:** after Stepper's verifier has ruled. This mirrors a verdict;
it never creates one.

### `omega-builder gate <state.json>`

Evaluate the build gates BG01 to BG20.

**Returns:** per-gate PASS, FAIL or UNEVALUATED with the evidence. Unevaluated
is reported as itself, never rounded to pass.

### `omega-builder checkpoint <state.json>`

Create a recovery checkpoint.

**When to use it:** before a long step, a risky integration or any expected
context loss.

### `omega-builder set-release <state.json>`

Record the frozen candidate and the Stepper release result.

### `omega-builder release-check <state.json>`

Evaluate terminal release readiness from Builder's side.

**Returns:** whether Stepper release PASS and BG01 to BG20 PASS both hold. This
is engineering readiness, not the release decision, which belongs to Release
{OS}.

### `omega-builder finalize <state.json>`

Create the final handoff and terminal status when all gates pass.

**When to use it:** once, at the end. It is an approval boundary.
**Returns:** the frozen final engineering and operations handoff.

### `omega-builder demo`

In-memory end-to-end semantic self-test.

**When to use it:** after install, or when the CLI's behaviour is in doubt.

## Command summary

| Command | Does |
|---|---|
| `/build` | open the runtime, preflight the session |
| `preflight` | readiness without claiming |
| `step <id>` | one full step transaction |
| `repair <id>` | bounded correction against evidence |
| `resume` | reconcile and continue from real state |
| `report` | evidence-backed build status |
| `omega-builder init` | initialise canonical state |
| `omega-builder validate` | structure and semantic invariants |
| `omega-builder status` | evidence-backed status |
| `omega-builder sync-step` | mirror one Stepper step |
| `omega-builder claim` | claim a READY step |
| `omega-builder transition` | apply a legal attempt transition |
| `omega-builder record-check` | store real command evidence |
| `omega-builder mark-step` | mirror an external verifier verdict |
| `omega-builder gate` | evaluate BG01 to BG20 |
| `omega-builder checkpoint` | recovery checkpoint |
| `omega-builder set-release` | record candidate and Stepper release result |
| `omega-builder release-check` | engineering readiness verdict |
| `omega-builder finalize` | freeze the final handoff |
| `omega-builder demo` | in-memory self-test |
