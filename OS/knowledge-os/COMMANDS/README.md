# Knowledge {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install knowledge-os` | Installs this OS into your environment | Once, first |
| `agentik configure knowledge-os` | Collects corpus locations, permissions and review cadence | After install |
| `agentik run knowledge-os` | Starts the OS | Every session |
| `agentik doctor knowledge-os` | Checks config, index freshness, adapters and dependencies | When something is off |
| `agentik update knowledge-os` | Updates to the latest version | When a release lands |
| `agentik eval knowledge-os` | Runs its evaluation suite, including retrieval precision | Before trusting it |

## OS commands

The OS answers to `/knowledge`.

### `/knowledge ingest <source>`

Register, screen and chunk a source.

**When to use it:** whenever material should become retrievable.
**Returns:** the source identifier, its attributed author and date, the chunk
count, and the injection screen result. A source carrying embedded instructions
is quarantined and nothing is indexed from it.

### `/knowledge index`

Build or rebuild the index from registered sources.

**When to use it:** after ingesting, after changing chunking, and any time the
index and the sources may have diverged.
**Returns:** what was indexed, what changed, and the result of the known question
set against the new index. An index that cannot be rebuilt from sources is
reported as a defect.

### `/knowledge ask <question>`

Answer with citations, or abstain.

**When to use it:** any question whose answer should be defensible.
**Returns:** claims, each citing a source and a span, with stale and superseded
passages marked. When the corpus does not support an answer it returns an
abstention naming what was searched, what was absent, and what would resolve it.

### `/knowledge pack <purpose>`

Build a scoped retrieval pack for another OS.

**When to use it:** when an OS needs material for a stated purpose rather than a
free text answer.
**Returns:** the passages relevant to that purpose with provenance, plus what was
withheld and under which reason: permission, project isolation, irrelevance, or
staleness.

### `/knowledge trace <claim>`

Follow a claim back to its source span.

**When to use it:** when a claim is challenged, and before any claim is promoted
to a canonical fact.
**Returns:** claim, chunk, source, span, original document, and the date and
author of that source. A claim that cannot be traced is withdrawn.

### `/knowledge gaps`

Report the questions the corpus cannot answer.

**When to use it:** on a cadence, and before writing new material.
**Returns:** unanswerable questions ranked by how often they were asked, each
routed to Documentation {OS} or Librarian {OS} as missing material, or accepted
as out of scope with a reason.

### `/knowledge stale`

Report passages the world has moved past.

**When to use it:** on the review cadence, and after any event that invalidates a
class of material.
**Returns:** passages past their review date or superseded, with their owner and
the source that replaced them. Nothing is deleted; supersession is marked.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/knowledge ingest` | ingest | register, screen and chunk a source |
| `/knowledge index` | index | build or rebuild, then re-score retrieval |
| `/knowledge ask` | ask | cited answer, or an honest abstention |
| `/knowledge pack` | pack | scoped retrieval for another OS |
| `/knowledge trace` | trace | claim back to the exact span |
| `/knowledge gaps` | gaps | what the corpus cannot answer |
| `/knowledge stale` | stale | what has aged or been superseded |

No command declares a claim true. Promotion to canonical truth is a Context &
Memory {OS} write with its own confirmation.
