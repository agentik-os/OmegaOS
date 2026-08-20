# Automation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install automation-os` | Installs this OS into your environment | Once, first |
| `agentik configure automation-os` | Collects owners, approval gates and monitoring endpoints | After install |
| `agentik run automation-os` | Starts the OS | Every session |
| `agentik doctor automation-os` | Checks config, adapters, tool contracts and monitoring reachability | When something is off |
| `agentik update automation-os` | Updates to the latest version | When a release lands |
| `agentik eval automation-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The OS answers to `/automate`.

### `/automation-audit <process>`

Find and score the automation candidates in an approved simplified process.

**When to use it:** immediately after Operations {OS} approves a simplified map,
and never before it.
**Returns:** candidates ranked by value against risk, exceptions, maintenance and
change cost, with the arithmetic visible, or a refusal naming the missing map or
the missing baseline.

### `/automate <candidate>`

Design the blueprint for one candidate.

**When to use it:** once a candidate has cleared its score and its arithmetic.
**Returns:** inputs, steps, decision points, exception routes, approvals,
outputs and the named owner. Every observed exception appears with a path, and
every irreversible step appears with a gate.

### `/automate controls <blueprint>`

Design the controls that make the automation safe to run twice.

**When to use it:** before any runbook or deployment.
**Returns:** the idempotency key, retry policy with its ceiling, deduplication
rule, rate limits, alert thresholds, and the meaning of silence, which is
failure and never health.

### `/runbook <automation>`

Write the operating runbook, including the manual recovery path.

**When to use it:** before deployment, and again after any incident that
revealed a step the runbook did not cover.
**Returns:** how to run it, how to read its output, how to stop it, and how a
person who did not build it does the work by hand while it is down.

### `/agent-automation <workflow>`

Assess a workflow that mixes deterministic steps with a judgment step.

**When to use it:** when a blueprint contains a step that cannot be expressed as
rules.
**Returns:** the arbitration boundary, the judgment step handed to Agent {OS} with
its required falsifier, and the deterministic remainder kept here.

### `/automation-review [automation]`

Audit live automations against their blueprint, their controls and their
business outcome.

**When to use it:** on a cadence, and any time a number downstream looks wrong.
**Returns:** per automation: run evidence, exception rate, gate usage, unused
grants, and the reconciliation between the claimed effect and the actual
business outcome. A green run with a wrong effect is reported as a failure.

### `/automation-incident <automation>`

Contain a failure and recover from it.

**When to use it:** the moment a run fails, or produces an effect it should not
have.
**Returns:** containment first, then the idempotency check that decides whether a
replay is safe, then the recovery, then the cause and the control that was
missing. A blind retry is never offered.

### `/automate retire <automation>`

Retire an automation and restore the manual path.

**When to use it:** when the process changed, the owner is gone, or the value no
longer clears its maintenance.
**Returns:** the dependencies that would break, the manual capability to restore,
the credentials to revoke, and a refusal while anything still silently depends
on it.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/automation-audit` | score | find and score candidates in a simplified process |
| `/automate` | design | the blueprint, exceptions routed, irreversible steps gated |
| `/automate controls` | control | idempotency, retries, deduplication, alerting |
| `/runbook` | deploy | how to run it and how to do it by hand |
| `/agent-automation` | agent | split the judgment step out to Agent {OS} |
| `/automation-review` | audit | reconcile the claimed effect against the real outcome |
| `/automation-incident` | incident | contain, check idempotency, recover, name the cause |
| `/automate retire` | retire | close it down and restore the manual path |

Every one of these refuses to run against a process Operations {OS} has not
approved as simplified.
