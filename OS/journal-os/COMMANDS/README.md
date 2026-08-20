# Journal {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install journal-os` | Installs this OS into your environment | Once, first |
| `agentik configure journal-os` | Collects the minimum context it needs | After install |
| `agentik run journal-os` | Starts the OS | Every session |
| `agentik doctor journal-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update journal-os` | Updates to the latest version | When a release lands |
| `agentik eval journal-os` | Runs its evaluation suite | Before trusting it |

`configure` asks three things and nothing more: where the journal store lives,
what may be persisted to Context & Memory {OS}, and whether entries may ever
leave the machine. The third answer defaults to no.

## OS commands

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/journal` | Opens the OS in `CAPTURE` mode | Every time you have something to write | An open capture surface, no questions asked |
| `/entry` | Stores one entry immediately from the text you pass | Mid-flow, when you do not want a session | The entry id and timestamp |
| `/revisit` | Retrieves past entries by date, range, tag, topic or person | When you want your own words back | The matching entries in date order, with dates and ids |
| `/pattern` | Computes candidate patterns over a range | Monthly, or when something feels repetitive | Candidates with supporting entries, contradicting entries, n and date range |
| `/journal-export` | Writes every entry to a portable file | Backup, migration, or leaving | The path to the file and the entry count |

### `/journal`

Opens the capture surface. It takes whatever you type and stores it. It does
not prompt, does not summarise, and does not reflect anything back beyond the
entry id. If you ask for a prompt it gives one, scoped to a theme you name.

```
/journal
/journal --prompt work
```

**When to use it:** any time. This is the command the OS exists for.
**Returns:** the stored entry id and timestamp. Nothing else unless you ask.

### `/entry`

One-shot capture. Stores the text you pass without opening a session, then
exits. Use it when you are inside another workflow and do not want to leave it.

```
/entry "third time this month I have said yes to a call I did not want"
/entry --tag berlin "the offer is real but the timing is wrong"
```

**When to use it:** mid-flow capture, from any other OS or from a script.
**Returns:** the entry id and timestamp.

### `/revisit`

Retrieval. Searches only the range you declare. Returns entries in date order,
your own words first, any commentary clearly separated from the text.

```
/revisit --since 2026-03-01 --until 2026-04-01
/revisit --tag berlin
/revisit --topic "co-founder"
```

**When to use it:** before making a call you have thought about before, when a
situation feels familiar, or when you want to check a memory against the record.
**Returns:** matching entries with dates and ids. If nothing matches it names
the range and terms searched and stops. It never widens the search silently.

### `/pattern`

Computes candidates across a range. Runs the falsifying search first, so every
candidate arrives with the entries that contradict it as well as the ones that
support it. Nothing below two independent entries is labelled a pattern.

```
/pattern --since 2026-06-01
/pattern --topic sleep --since 2026-01-01
```

**When to use it:** on a monthly cadence, or when you suspect you are repeating
yourself.
**Returns:** each candidate as one sentence, plus supporting entries,
contradicting entries, n, the date range, and what would change the conclusion.
Accepting a candidate sends it as a proposal to the OS that owns that object,
and Journal names which one before sending. It never sends without your accept.

### `/journal-export`

Writes every entry to markdown or plain text that opens without this software.

```
/journal-export --format markdown --out ~/journal-export.md
```

**When to use it:** backup, migration, or when you want to stop using this.
**Returns:** the file path and the number of entries written. This command
moves your data, so it always confirms before writing outside the journal store.

## Command summary

| Command | Does |
|---|---|
| `/journal` | opens capture, asks nothing |
| `/entry` | stores one entry and exits |
| `/revisit` | your past words back, by date, tag, topic or person |
| `/pattern` | candidate patterns with the evidence for and against |
| `/journal-export` | every entry, in a portable file |
