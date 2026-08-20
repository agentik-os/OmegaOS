# Market Research {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install market-research-os` | Installs this OS into your environment | Once, first |
| `agentik configure market-research-os` | Collects the minimum context it needs | After install |
| `agentik run market-research-os` | Starts the OS | Every session |
| `agentik doctor market-research-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update market-research-os` | Updates to the latest version | When a release lands |
| `agentik eval market-research-os` | Runs its evaluation suite | Before trusting it |

## OS commands

This OS has one root command and no sub-commands, deliberately. The nine
invocation modes are inferred from what you bring, not typed as flags, because a
user who already knows they want `DILIGENCE` at `INVESTMENT_GRADE` depth is not
the user this OS is for.

### `/market-research <idea, question, artifact or opportunity>`

The root command. Give it an idea, a market question, a concept from a
brainstorm, an existing study, or two opportunities to compare. It states the
mode and depth profile it inferred, then runs.

```
/market-research a scheduling tool for independent physiotherapists in France
/market-research audit this study before the board reads it
/market-research compare these two opportunities on the same evidence standard
/market-research what changed in this market since the March version
```

**When to use it:** whenever a market decision is about to be made on an
assertion nobody has measured.
**Returns:** the stated mode and depth profile with its exclusions, the framed
decision, and then the pack the mode promises, ending in exactly one bounded
decision (`GO`, `PIVOT`, `HOLD`, `NO-GO`, `INSUFFICIENT EVIDENCE`) and one
completion status (`MARKET RESEARCH IN PROGRESS`, `MARKET RESEARCH BLOCKED`,
`MARKET RESEARCH COMPLETE, DECISION READY`).

### `/market-research-os <same input>`

The fully qualified alias. Identical behaviour. Use it when several OS units are
loaded in one session and you want the routing to be unambiguous.

### `/omg-market-research` and `/omg-market-research-os`

The same two commands under the OmegaOS `omg-` namespace, for sessions where the
suite is installed alongside other skill libraries and the short names collide.
No behavioural difference.

### How the mode is inferred

You never type the mode. It is read off what you brought and what already
exists, and it is always stated back to you before any work starts. If the
inference is wrong, say so in one line and it is restated.

| What you bring | Mode inferred |
|---|---|
| an idea with no prior material | `NEW` |
| prior chats, files, studies or decisions | `RECOVER`, first, before anything else runs |
| a request for a fast read before spending | `RAPID_SCAN`, at `SIGNAL` depth, never called validation |
| a pending launch or funding decision | `FULL_VALIDATION` |
| investment, acquisition, board or enterprise stakes | `DILIGENCE`, at `INVESTMENT_GRADE` depth |
| one bounded question: a segment, competitor, feature, price or channel | `DEEP_DIVE` |
| an approved collection plan and elapsed time | `MONITOR` |
| an existing study, deck or claim to inspect | `AUDIT` |
| two versions or two opportunities to compare | `DELTA` |

The depth profile is chosen the same way: the lowest of `SIGNAL`, `VALIDATION`
or `INVESTMENT_GRADE` that can support the decision, with the exclusions named
out loud rather than left implicit.

## Workspace CLI

The durable, machine readable half of the OS is a stdlib Python CLI. It owns the
research state file: records, stable IDs, gate results and the continuation
pointer. There is no installed shell wrapper, run it with Python from the pack.

```
python3 scripts/market_research_os.py <subcommand> <workspace> [options]
```

It is deterministic on purpose. It complements expert judgement and never
replaces it: it can tell you a hypothesis has no evidence attached, it cannot
tell you the evidence is any good.

### `init <ws> --project-id <id> --project-name <name> --decision <text> [--mode <m>] [--depth <d>]`

Create the versioned state file for a new research workspace.

**When to use it:** once, at the start of a study that will outlive one session.
**Returns:** the created workspace path, the initial version, and the recorded
decision frame, mode and depth profile.

### `validate <ws> [--strict]`

Run the schema checks and the quality gates over the workspace: record shape,
required fields, orphan records, hypotheses with no evidence, decisions with no
kill criteria, evidence with no provenance.

**When to use it:** before any handoff, before freezing a version, and after any
bulk edit.
**Returns:** per-check pass or fail with the offending record ID named. Exits
non-zero on a critical defect, so it can gate a pipeline. `--strict` promotes
warnings to failures.

### `status <ws>`

Report where the study is: mode, depth, version, which artifacts exist, which
gates have run, and what the checkpoint says comes next.

**When to use it:** at the start of every session, and after any interruption.
**Returns:** the progress view, the current and next artifact pointers, and the
open gate list.

### `score <ws>`

Diagnostics over the hypothesis register and the gates: confidence distribution,
evidence strength per hypothesis, contradictions, and which hypotheses the
decision actually depends on.

**When to use it:** before writing the recommendation, and in any critic pass.
**Returns:** the hypothesis scorecard and the gate scorecard, with the weakest
load-bearing hypotheses listed first.

### `checkpoint <ws> --current <artifact> --next <artifact>`

Write a restart-safe continuation pointer into the state file.

**When to use it:** at every natural break in a long run, and always before a
context boundary. A study that cannot be resumed is a study that gets restarted.
**Returns:** the recorded pointer and the timestamp.

### `allocate <ws> <prefix>`

Allocate the next stable ID for a record class (`SRC`, `HYP`, `EST`, `EXP`,
`SEG`, `CMP`, `SIG`, `RSK`, `GATE` and the rest of the register).

**When to use it:** every time a normative record is created. Never hand-write an
ID, and never reuse one.
**Returns:** the allocated ID, monotonically, for example `HYP-014`.

### `export <ws> --output <path>`

Write the state or the status view out of the workspace for a handoff, a review
or an archive.

**When to use it:** when producing the Blueprint Input Manifest, handing the pack
to another OS, or filing a frozen version.
**Returns:** the written path and what it contains.

### `demo <ws>`

Generate a small, valid, fully populated workspace to read.

**When to use it:** the first time you touch the CLI, or when you want to see
what a record with complete provenance is supposed to look like.
**Returns:** a workspace that passes `validate` and can be inspected with
`status` and `score`.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/market-research` | root entry point: infers mode and depth, runs the study | the pack for that mode, one bounded decision, one completion status |
| `/market-research-os` | fully qualified alias of the root command | identical |
| `/omg-market-research` | the root command in the OmegaOS namespace | identical |
| `/omg-market-research-os` | fully qualified alias in the OmegaOS namespace | identical |
| `market_research_os.py init` | create the versioned workspace | workspace path, version, decision frame |
| `market_research_os.py validate` | schema checks plus quality gates | per-check pass or fail, non-zero exit on a critical defect |
| `market_research_os.py status` | where the study stands | progress, artifact pointers, open gates |
| `market_research_os.py score` | hypothesis and gate diagnostics | scorecards, weakest load-bearing hypotheses first |
| `market_research_os.py checkpoint` | restart-safe continuation pointer | recorded pointer and timestamp |
| `market_research_os.py allocate` | next stable ID for a record class | a monotonic, never reused ID |
| `market_research_os.py export` | state or status view out of the workspace | the written path and its contents |
| `market_research_os.py demo` | a valid minimal workspace to read | a workspace that passes `validate` |
