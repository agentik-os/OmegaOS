# Quality & Evaluation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

The commands below certify. None of them fix anything, and none of them ship
anything: a fix is a Stepper step for Builder {OS}, and shipping is Release
{OS}. That separation is what makes a verdict from this OS worth reading.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install quality-evaluation-os` | Installs this OS into your environment | Once, first |
| `agentik configure quality-evaluation-os` | Collects the minimum context it needs | After install |
| `agentik run quality-evaluation-os` | Starts the OS | Every session |
| `agentik doctor quality-evaluation-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update quality-evaluation-os` | Updates to the latest version | When a release lands |
| `agentik eval quality-evaluation-os` | Runs its evaluation suite | Before trusting it |

## Intake and planning

### `/quality`

Open the quality authority. It pins the build artifact and the contracts,
reports what is reachable, and proposes the certification scope.

**When to use it:** the moment Builder {OS} finalises.
**Returns:** the pinned inputs, what could not be reached, and the proposed
scope for you to confirm or narrow.

### `/traceability`

Build the bidirectional requirement-to-evidence matrix.

**When to use it:** before any test is planned. Testing before tracing means
testing what is easy.
**Returns:** every critical requirement with its planned evidence, plus two
gap lists: requirements with no evidence, and evidence attached to no
requirement.

### `/test-plan [--risk <model>]`

Produce the risk-based test and evaluation plan.

**When to use it:** after the matrix exists.
**Returns:** planned tests ordered by consequence and uncertainty, each with
the environment and data it needs, and an explicit list of what the plan does
not cover.

## Execution

### `/qa [--scope <area>]`

Run functional and exploratory QA against the plan.

**When to use it:** the main execution pass.
**Returns:** per-test results with real output, exploratory session notes, and
new defects. A surface that could not be reached is returned as blocked, never
as passing.

### `/regression [--since <version>]`

Run the regression suite against the previous certified build.

**When to use it:** every build after the first certification.
**Returns:** what broke that used to work, with the last known good version
named.

### `/contract-test`

Verify API, event and integration contracts against the Blueprint definitions.

**Returns:** per-contract conformance, including fields that exist but were
never specified, which are as much a finding as fields that are missing.

### `/performance [--profile <target>]`

Test against the non-functional requirements: latency, throughput, resource
use, degradation under load.

**Returns:** measurements against the stated thresholds, with the environment
recorded, and the spread rather than the best run.

### `/accessibility`

Test the built product against the accessibility contracts Design {OS} wrote.

**When to use it:** every release with a user interface.
**Returns:** per-contract conformance with the failing element, state and path
named. Design writes the contract; this command tests the build against it.

### `/data-migration`

Verify a data migration: row counts, invariants, reversibility, and behaviour
on partial failure.

**When to use it:** any release that changes stored data. Ask before running it
anywhere real.

### `/ai-eval [--dataset <path>]`

Design and run AI evaluations: task success, groundedness, hallucination rate,
refusal correctness, regression across model or prompt versions.

**When to use it:** any product with model-driven behaviour, and again whenever
the model or the prompt changes.
**Returns:** scores over the dataset, the dataset stored beside them, and the
model and prompt version the score belongs to. A single good answer is not a
result.

## Ruling

### `/defects [--triage]`

Show or triage the defect ledger.

**Returns:** each defect with severity, impact, reproduction, workaround and
owner. A defect missing any of those is incomplete and is flagged as such.

### `/verdict`

Issue the quality verdict.

**When to use it:** when the plan has run, or has been transparently narrowed.
**Returns:** `CONFORMS`, `CONFORMS WITH KNOWN DEFECTS` (each defect with its
acceptance authority) or `DOES NOT CONFORM`, always with the residual risk and
the uncovered surface named. This is what Security {OS} and Release {OS} read.

## Command summary

| Command | Does |
|---|---|
| `/quality` | pin the inputs, propose the certification scope |
| `/traceability` | requirement-to-evidence matrix, both gap lists |
| `/test-plan` | risk-ordered plan, with what it does not cover |
| `/qa` | functional and exploratory execution |
| `/regression` | what broke that used to work |
| `/contract-test` | API, event and integration conformance |
| `/performance` | non-functional requirements, measured |
| `/accessibility` | the build against the design accessibility contracts |
| `/data-migration` | migration correctness and reversibility |
| `/ai-eval` | scored AI evaluation over a stored dataset |
| `/defects` | the defect ledger and its triage |
| `/verdict` | the quality verdict, with residual risk and uncovered surface |
