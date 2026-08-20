# Librarian {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install librarian-os` | Installs this OS into your environment | Once, first |
| `agentik configure librarian-os` | Collects the minimum context it needs | After install |
| `agentik run librarian-os` | Starts the OS | Every session |
| `agentik doctor librarian-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update librarian-os` | Updates to the latest version | When a release lands |
| `agentik eval librarian-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/librarian [<question or source>]`

The root command. With a question, it runs `ASK` against the corpus. With a
source, it runs `INGEST`. With nothing, it reports the state of the corpus:
sources, extracts, open defects.

**When to use it:** the default way in.
**Returns:** an answer with citations, an ingest confirmation, or the corpus
state.

### `/ingest <source> [--purpose <why>]`

Create the canonical source record and index the material.

**When to use it:** the moment a source enters your world, not the moment you
finish it.
**Returns:** the source record (title, author, edition, date, format, copy
location, licence note) and whether the text could be indexed.

### `/extract <source> [--grain claim|model|method|number|quote]`

Pull extracts out of a source at the requested grain, each with a locator.

**When to use it:** while reading, and again when you need the source for a
specific job.
**Returns:** typed extracts with locators, and a list of the sections that were
skipped so you know what was not covered.

### `/ask "<question>" [--shelf <name>]`

Answer strictly from indexed material.

**When to use it:** before any outside search.
**Returns:** the answer with a citation on every claim, plus an explicit list of
what the corpus does not cover. If nothing relevant is indexed, it says so
rather than improvising.

### `/synthesize "<subject>" [--sources <a,b,c>]`

Compare what several sources say about one subject.

**When to use it:** when you have read three books on a subject and they do not
agree.
**Returns:** the positions, attributed, the axes of disagreement, and what none
of the sources addresses.

### `/shelf <name> [--rule "<inclusion rule>"]`

Create or show a named subset of the corpus.

**When to use it:** when a project needs its own reading set.
**Returns:** the shelf with its inclusion rule, its members, and what was
deliberately excluded.

### `/reread <source>`

Record a new reading of a source you have read before.

**When to use it:** when a source lands differently than it did the first time.
**Returns:** the delta: new extracts, superseded extracts, and every synthesis
that used the superseded ones.

### `/cite <extract-id> [--style <style>]`

Produce a citation for an extract, at the permitted quotation length.

**When to use it:** when writing anything that quotes a source.
**Returns:** the formatted citation with locator, and the licence limit that
applies.

### `/corpus-audit`

Inspect the corpus for defects.

**When to use it:** monthly, and before trusting the corpus for anything
important.
**Returns:** extracts with no locator (quarantined), sources with no extracts,
duplicate source records, superseded editions still being cited, and index
staleness.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/librarian` | entry point: ask, ingest, or report corpus state | answer, source record, or state |
| `/ingest` | create the canonical source record and index it | the record plus index status |
| `/extract` | pull typed extracts with locators | extracts, and what was skipped |
| `/ask` | answer strictly from the corpus | cited answer plus the stated gaps |
| `/synthesize` | compare several sources on one subject | attributed positions and their disagreements |
| `/shelf` | name a subset with an inclusion rule | shelf members and exclusions |
| `/reread` | record a later reading as a delta | new and superseded extracts |
| `/cite` | citation at the permitted quote length | citation plus licence limit |
| `/corpus-audit` | find corpus defects | defect list per record |
