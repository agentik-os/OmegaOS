# Release {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

Every command that touches production is an approval boundary. None of them run
on an assumed yes, and none of them run without both the quality verdict and
the security clearance on file.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install release-os` | Installs this OS into your environment | Once, first |
| `agentik configure release-os` | Collects the minimum context it needs | After install |
| `agentik run release-os` | Starts the OS | Every session |
| `agentik doctor release-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update release-os` | Updates to the latest version | When a release lands |
| `agentik eval release-os` | Runs its evaluation suite | Before trusting it |

## Deciding

### `/release`

Open the release authority. It reads the build artifact, the quality verdict,
the security clearance and the Blueprint release definition, and reports what
is missing.

**When to use it:** when a release is being considered.
**Returns:** the candidate inputs, what is absent, and the proposed boundary.

### `/boundary`

Fix what is in the release, what is out, and what it is called.

**When to use it:** before assembling anything. An unbounded release cannot be
rolled back meaningfully, because nobody can say what would come out.
**Returns:** the in list, the out list with reasons, and the version.

### `/release-candidate`

Assemble the candidate and its evidence pack.

**Returns:** the artifact fingerprint, the quality verdict, the security
clearance, the Blueprint release definition, the rollback plan and the
observability contract, with anything missing named rather than skipped.

### `/release-gate`

Run the gate and record the decision.

**When to use it:** once the candidate is assembled and the abort criteria are
set.
**Returns:** go or no-go, the decider, the evidence read, the accepted risks
with an acceptance authority each, and the abort criteria. A bypassed gate
requires a Review & Governance {OS} exception first, and the command says so
rather than proceeding.

## Shipping

### `/rollout [--strategy canary|progressive|staged|full]`

Plan and run the rollout.

**When to use it:** after a go decision, never before.
**Returns:** the stages, the population per stage, the signals watched, the
abort thresholds, and the current position. Smallest blast radius first, always.

### `/deploy [--env <environment>]`

Execute the deployment for the current stage.

**When to use it:** inside a rollout, with approval. This is an approval
boundary every time, including for a stage that looks routine.
**Returns:** what was deployed, where, at what version, and the immediate
health signals.

### `/verify-production`

Exercise the real golden path in production and read the real signals.

**When to use it:** immediately after every stage.
**Returns:** the paths exercised with their real responses, the signals against
their thresholds, and a pass or fail. A successful deploy is not a working
feature, and this command is the difference.

### `/observability`

Define or check the observability contract the release ships with.

**When to use it:** before the gate. A change that cannot be observed cannot be
verified or safely rolled out, which is a no-go reason.
**Returns:** what is measured, what alerts, at what threshold, who is paged,
and which changed paths have no telemetry at all.

## Undoing

### `/rollback-plan`

Write or review the way back, before deploying.

**When to use it:** part of the candidate, never written during an incident.
**Returns:** the steps, the time it takes, who can execute it, what it
restores, and explicitly what it does not restore once data has moved.

### `/rollback [--to <version>]`

Execute the rollback.

**When to use it:** an abort criterion fired, or production verification
failed. Rolling back is the default response, and staying broken while
investigating is the choice that needs an owner.
**Returns:** the restored version, the state of the data, and what remains
inconsistent. A rollback involving a data migration is an approval boundary.

### `/incident`

Open and run the incident path.

**When to use it:** production is degraded, whatever the cause.
**Returns:** the incident record, built as it happens: timeline, impact,
containment actions, decisions and who made them. The timeline is never
rewritten afterwards.

### `/handoff`

Hand the release over.

**When to use it:** once production verification passes and the release is
stable.
**Returns:** runbooks and the observability contract to Operations & Automation
{OS}, the customer-facing change to Delivery & Customer Success {OS},
postmortems and exceptions to Review & Governance {OS}.

## Command summary

| Command | Does |
|---|---|
| `/release` | open the authority, report the candidate inputs |
| `/boundary` | what is in, what is out, what it is called |
| `/release-candidate` | assemble the candidate and its evidence pack |
| `/release-gate` | the go or no-go decision, recorded with its owner |
| `/rollout` | stages, signals and abort thresholds |
| `/deploy` | execute one stage, with approval |
| `/verify-production` | the real golden path against the real signals |
| `/observability` | what is measured, what alerts, who is paged |
| `/rollback-plan` | the way back, written before the deploy |
| `/rollback` | execute the way back |
| `/incident` | contain, record, decide, route |
| `/handoff` | to Operations, Delivery and Governance |
