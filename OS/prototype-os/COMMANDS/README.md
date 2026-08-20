# Prototype {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

Prototype is deliberately light on tooling. The expensive part of a prototype
is deciding what to test and being honest about the result, and no CLI does
either. The commands exist to keep the question, the threshold, the verdict and
the teardown recorded, so the artifact can be thrown away safely.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install prototype-os` | Installs this OS into your environment | Once, first |
| `agentik configure prototype-os` | Collects the minimum context it needs | After install |
| `agentik run prototype-os` | Starts the OS | Every session |
| `agentik doctor prototype-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update prototype-os` | Updates to the latest version | When a release lands |
| `agentik eval prototype-os` | Runs its evaluation suite | Before trusting it |

## Session commands

### `/prototype`

Open the OS without naming a question. It reads the open ASSUMPTION and UNKNOWN
records from Blueprint {OS} and the reversible decisions from Design {OS}, and
proposes the triage.

**Returns:** the ranked question list, and the one it recommends testing first.

### `triage`

Rank the open questions by cost of being wrong against cost of answering.

**When to use it:** more open questions than time, which is the normal case.
**Returns:** the ranking with both costs stated per question, the selected
question, and why the others wait.

### `question <text>`

Turn a vague worry into a falsifiable question with a pre-agreed threshold.

```text
question "will users understand the panel navigation"
```

**Returns:** the rewritten question, the threshold proposal, the method
recommendation, and the budget ceiling to confirm. Nothing is built until the
threshold is agreed.

### `spike <question-id>`

Build a throwaway implementation of the technically risky part only.

**When to use it:** feasibility, integration behaviour, or an unknown library
or platform limit.
**Returns:** the answer, the evidence (logs, timings, errors), and the parts of
the plan that were unblocked or newly blocked.

### `flow <question-id>`

Test comprehension or interaction with a clickable or paper artifact and a
fixed protocol.

**Returns:** the protocol used, the raw observations per participant, where
people hesitated or failed, and the verdict against the threshold.

### `fake <question-id>`

Test demand or willingness with a manual, concierge or smoke-test artifact
rather than a built product.

**When to use it:** before building anything, when the risk is that nobody
wants it.
**Returns:** observed behaviour, not stated intent, and the conversion or
completion measured against the threshold.

### `bench <question-id>`

Measure a performance, cost or model-quality assumption with a reproducible
harness.

**Returns:** the harness, the dataset, the measurements, and whether the
threshold was met. A single unrepeated run is reported as such.

### `verdict <prototype-id>`

Rule on the evidence.

**Returns:** `CONFIRMED`, `REFUTED` or `INCONCLUSIVE`, the evidence it rests
on, and the upstream record IDs it settles or reopens.

### `teardown <prototype-id>`

Delete the artifact, revoke what it was given, and record that it is gone.

**When to use it:** immediately after the verdict, without exception.
**Returns:** the teardown record: what existed, where, and confirmation of
removal.

### `ledger`

List every prototype: its question, its verdict, its expiry, and whether it has
been torn down.

**When to use it:** weekly, and before any release. An artifact past its expiry
is a finding.
**Returns:** the table, with anything still alive past expiry flagged.

## Command summary

| Command | Does |
|---|---|
| `/prototype` | open the OS, propose the triage |
| `triage` | rank open questions, select one |
| `question <text>` | make it falsifiable, set the threshold |
| `spike <id>` | throwaway implementation of the risky part |
| `flow <id>` | comprehension or interaction test with a protocol |
| `fake <id>` | demand test with observed behaviour |
| `bench <id>` | reproducible measurement against a threshold |
| `verdict <id>` | rule, and name what it settles upstream |
| `teardown <id>` | delete the artifact, record its removal |
| `ledger` | every prototype, verdict and expiry, live ones flagged |
