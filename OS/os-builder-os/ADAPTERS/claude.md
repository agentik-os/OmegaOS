# OS Builder {OS}: Claude Adapter

The operating logic in `OS.md` is constant. This file records only how it is
implemented on Claude, and what Claude cannot do.

Claude is the **reference target** for this OS. It is the only one of the four
where every phase of the pipeline runs end to end without a human moving files
by hand, because it is the only one with a filesystem, a shell and parallel
subagents in the same session. Every other adapter is measured against what
happens here.

## Capabilities used

| Capability | Used for | Phase |
|---|---|---|
| Skills, with `SKILL.md` YAML front matter | the OS is discoverable by name and auto invoked on a matching request | all |
| Filesystem read and write | the candidate package is produced on disk, not pasted | 7, 8, 12, 13 |
| Bash | `validate_os.py`, `score_os.py`, `create_zip.py`, `verify.py`, `graph.py` run for real, and their exit codes are the evidence | 8, 9, 11, 14 |
| Parallel subagents | the red team runs several independent attackers at once, each blind to the others' findings | 10 |
| Subagent isolation, worktrees where available | a repair cycle edits a candidate without touching the live suite | 12 |
| Web fetch and search | source capture in research, where the operator permits it | 3 |
| Long context | the whole candidate package is held while it is graded, so a review is not a series of file summaries | 9, 11 |

## Installation

The OS is a directory. Claude reaches it by one of three placements, and they
are not equivalent:

```bash
# 1. inside the suite, the canonical home, graded by OS/_tools/verify.py
OS/os-builder-os/

# 2. as a user skill, available in every session on this machine
~/.claude/skills/os-builder-os/        # symlink or copy of the directory

# 3. as an OmegaOS skill, installed by install.sh and synced by omega sync
~/.omega/skills/os-builder-os/
```

Placement 1 is the source of truth. Placements 2 and 3 are how a session finds
it. A change made in 2 or 3 and not in 1 is lost on the next sync, which is the
one installation mistake that costs real work.

`SKILL.md` carries the YAML front matter Claude reads for discovery:

```yaml
---
name: os-builder-os
description: <the tagline>. OS Builder {OS}, unit 00 of the AGENTIK {OS} suite
  (00 · RUNTIME). Use when the user asks to build an OS or invokes /os-builder-os.
---
```

## Operating contract on Claude

1. **Read `SYSTEM.md` and `SKILL.md` before anything else.** Not `README.md`:
   the README is written for a human deciding whether to use this OS, and it is
   the wrong entry point for the agent about to run it.
2. **Run the FULL build unless Fast Build is explicitly selected**, and record
   which one is running in the ledger. Fast Build never waives evidence,
   security or quality gates; it waives depth, not floors.
3. **Maintain the build ledger** described in
   [`../INTERFACES/chat.md`](../INTERFACES/chat.md): phase, status, unresolved
   risks, next gate. Keep it in the harness task list as well as in the
   transcript, because the transcript does not survive a compaction and the task
   list does.
4. **One phase active at a time.** Phase transitions are recorded when they
   happen, not batched at the end.
5. **Never report completion before the release gate.** Claude can run every
   mechanical gate item itself, so there is no legitimate reason here to report
   done on an unverified package. A completion claim whose evidence is a summary
   rather than an exit code is adversarial case A12.
6. **Verify a subagent's claim before accepting it.** A red team subagent
   reporting "no escapes found" is an input. Re-run the case that matters, or
   the fan out has bought coverage in appearance only.

## Red team fan out, phase 10

The one phase where Claude's parallelism changes the result rather than only the
speed. The twelve adversarial cases are dispatched as independent subagents,
each given one case and no knowledge of the others' findings, each returning a
verdict against the case's fail signature.

Independence is the point. A single agent walking all twelve cases in sequence
carries its own earlier conclusions forward and stops attacking after the third
case it cannot break. Twelve blind attackers do not.

The coordinator then re-runs, itself, every case that reported an escape. A
finding accepted on a delegate's word has not been verified, and the red team
exists precisely to catch the things a first pass believes.

## Commands

The eight build commands are `/os-intake`, `/os-architect`, `/os-build`,
`/os-test`, `/os-review`, `/os-repair`, `/os-package` and `/os-status`. They are
documented in [`../COMMANDS/README.md`](../COMMANDS/README.md), which is their
single source of truth; this adapter only records that Claude exposes them as
slash commands and that `/os-status` prints the build ledger.

## Unsupported capabilities

Stated rather than worked around, per the adapter contract:

- **No persistent state between sessions of its own.** Claude's memory of a
  build is the ledger on disk plus whatever Context & Memory {OS} holds. A build
  resumed in a fresh session reads the ledger from the file; it does not
  remember. This is why the ledger is written down rather than narrated.
- **Context is finite and a full suite audit exceeds it.** Grading seventy three
  units in one context is not possible; the fan out is per unit, and the
  coordinator holds only the verdicts.
- **Network access may be restricted.** Where research cannot reach a source,
  the evidence state is `UNKNOWN` with what was attempted, never an inferred
  citation. A blocked fetch is an abort for that item, never a pass.
- **Subagent availability varies by harness version.** Where parallel subagents
  are unavailable, phase 10 runs sequentially and the ledger records that the
  cases were not independent, because that materially weakens the red team and
  the reader is entitled to know.

## What is false here that is true elsewhere

Claude runs the validators itself, so gate items 15 through 18 are answered by
exit codes rather than by inspection. On ChatGPT and Gemini they cannot be
answered at all without the operator running the commands, and those adapters
say so. Do not carry this adapter's assumption that a gate result is available
into a surface where it is not.
