# Context & Memory {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install context-memory-os` | Installs this OS into your environment | Once, first, before any other OS that stores state |
| `agentik configure context-memory-os` | Collects permissions, retention and project scopes | After install |
| `agentik run context-memory-os` | Starts the OS | Every session |
| `agentik doctor context-memory-os` | Checks config, adapters, permissions and store integrity | When something is off |
| `agentik update context-memory-os` | Updates to the latest version | When a release lands |
| `agentik eval context-memory-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The ten commands are the OS's modes. `/memory` is the default entry point.

### `/memory [query]`

Search or inspect what is remembered, with provenance attached.

**When to use it:** whenever you want to know what the system believes, and on
whose word.
**Returns:** matching records with source, timestamp, confidence, consent and
tier. Records you are not permitted to see are reported as withheld, never
silently omitted.

### `/remember <statement>`

Propose a memory write.

**When to use it:** when something has been established that should survive this
session.
**Returns:** a staged record, its proposed tier, and what is still missing before
it can be verified. Nothing becomes canonical without provenance.

### `/ingest <file|event>`

Ingest a file or an event as a source.

**When to use it:** a document, transcript, export or event stream should become
retrievable.
**Returns:** the source hash, the extracted records staged for review, and the
result of the injection screen. A source that carries embedded instructions is
quarantined and nothing is ingested from it.

### `/context <purpose>`

Compile a purpose scoped context pack for an OS or a task.

**When to use it:** every time another OS starts work. This is the command the
rest of the suite depends on.
**Returns:** the minimum sufficient pack for that stated purpose, with provenance,
and an explicit list of what was left out and why.

### `/snapshot <project>`

Create an immutable, versioned canonical state of a project, goal or person.

**When to use it:** at a checkpoint worth being able to return to, and before any
change large enough that you would want the previous state back.
**Returns:** an addressable snapshot identifier and a diff against the previous
snapshot.

### `/decision-log <decision>`

Record a decision, its rationale, its alternatives and its expected outcome.

**When to use it:** at the moment of deciding, not afterwards. A rationale
reconstructed later is a story.
**Returns:** the decision record, and the review date at which its outcome will
be checked.

### `/contradiction`

Open, inspect or adjudicate conflicting records.

**When to use it:** when two records disagree, or when an OS reports a conflict.
**Returns:** both sides with their provenance, the adjudication and its reason,
and the superseded record, which is kept rather than deleted.

### `/memory-audit [scope]`

Audit provenance, permissions, age and access.

**When to use it:** periodically, and any time you want to know whether the store
still deserves trust.
**Returns:** per record: source present or absent, consent, age against retention,
who has read it. Records lacking provenance are listed individually, never
summarised into a count.

### `/forget <selector>`

Correct, archive or delete authorized memory.

**When to use it:** whenever the user wants something gone or wrong.
**Returns:** what will be deleted, what depends on it, and after confirmation, a
deletion receipt. Deletion is real.

### `/export-memory`

Produce a human readable export of everything remembered.

**When to use it:** for review, for portability, or on request.
**Returns:** the full export, structured by tier and project, with provenance.
Requires explicit approval because it collects everything in one place.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/memory` | retrieve | search or inspect memory |
| `/remember` | capture | propose a memory write |
| `/ingest` | capture | ingest a file or event as a source |
| `/context` | compile | compile a purpose scoped context pack |
| `/snapshot` | snapshot | create a versioned canonical state |
| `/decision-log` | capture | record a decision and its rationale |
| `/contradiction` | resolve | adjudicate conflicting records |
| `/memory-audit` | govern | audit provenance, permissions and age |
| `/forget` | forget | correct, archive or delete |
| `/export-memory` | govern | produce a readable export |

No command stores a credential, a token or a secret. That exclusion holds in
every mode, including `/ingest`.
