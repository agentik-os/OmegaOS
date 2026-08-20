# OS Builder {OS}: Codex Adapter

The operating logic in `OS.md` is constant. This file records only how it is
implemented on Codex, and what Codex cannot do.

Codex is the **repair and validation** target. It is strongest at exactly the
phases where Claude is most likely to be talked out of rigour: reading what is
actually on disk before changing it, running the validators, and refusing to
report success over a red result. Use it for phases 8, 9, 12 and 14, and pair it
with Claude for the judged phases.

## Capabilities used

| Capability | Used for | Phase |
|---|---|---|
| `AGENTS.md` discovery, repository rooted | the OS is picked up automatically for work inside the repo | all |
| Filesystem read before write | the existing package is inspected before it is edited | 8, 12 |
| Shell execution | `validate_os.py`, `score_os.py`, `create_zip.py`, `graph.py` run and their exit codes are read | 8, 9, 11, 14 |
| Patch oriented editing | a repair is a diff against the candidate, which makes the repair diff view real rather than reconstructed | 12 |
| Plan before build | the phase plan is stated and tracked before files change | all |

## Installation

Codex reads `AGENTS.md` from the repository root and from directories on the
path to the file being worked on. This OS installs by placement, not by
registration:

```
<repo>/AGENTS.md                     repository wide instructions
<repo>/OS/os-builder-os/             the OS itself
```

The `AGENTS.md` entry for this OS names the entry point (`OS.md`), the
behaviour contract (`SYSTEM.md`), and the one hard rule below. It does not
restate the pipeline: duplicating the phase list into `AGENTS.md` creates a
second copy that will drift from `OS.md`, and the drift is discovered when the
two disagree about what phase 12 requires.

## Operating contract on Codex

1. **Inspect existing files before editing.** Read the candidate as it is, not
   as the plan assumes it is. Most repair failures on this target are edits made
   against a remembered version of a file.
2. **Plan before build.** State the phase, the files that will change, and the
   verification command, before touching anything.
3. **Preserve scope.** Change what the repair targets and nothing adjacent. A
   repair that also tidies neighbouring prose makes the repair diff unreadable,
   and the repair diff is the artifact that proves the score moved for the
   reason claimed.
4. **Write substantive content.** No placeholder, no "to be authored", no
   scaffold marker. `verify.py` fails on all three, and passing that check is
   the floor rather than the goal.
5. **Validate JSON and scripts before claiming a phase done.** Every
   `manifest.json` and every schema parses; every carried script compiles with
   `python3 -m py_compile`.
6. **Run the tests where they are executable.** A test skipped for convenience
   is recorded as skipped, never as passed.
7. **Maintain the changelog.** Every version bump gets its entry, per
   [`../REFERENCES/VERSIONING.md`](../REFERENCES/VERSIONING.md). Gate item 17 is
   mechanical and checks exactly this.
8. **Never report success when validation fails.** This is the hard rule, and it
   is why this adapter exists.

## The hard rule, and why it is on this adapter

> **A non zero exit code is a failure. It is never a warning, never a nit,
> never something to note in the summary and move past.**

Adversarial case A12 is the most consequential failure available to this OS,
because a package reported as validated and shipped broken ends the mission for
everyone downstream, and the failure is invisible at the point where it is
cheapest to fix.

Concretely, on this target:

```bash
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full ; echo "exit=$?"
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json  ; echo "exit=$?"
python3 OS/_tools/graph.py --strict                        ; echo "exit=$?"
```

The evidence a phase closed is the printed `exit=0`, pasted, not a sentence
saying the checks passed. A summary is not evidence. If the exit code is not in
the report, the phase is open.

## Unsupported capabilities

- **No parallel independent subagents.** Phase 10's red team runs sequentially
  here, which materially weakens it: one agent carrying its own earlier
  conclusions stops attacking after the third case it cannot break. The ledger
  records that the cases were not independent. Where both targets are available,
  run phase 10 on Claude and phase 12 on Codex.
- **Weaker at judged phases.** Phases 2, 4 and 11 are judgment, not
  transformation. Codex will produce a defensible scorecard and a mediocre value
  proposition. Score here, argue elsewhere.
- **Sandboxing varies by configuration.** Where the shell is restricted, the
  mechanical gate items are reported as unanswered with the reason. An
  unavailable check is a negative result, never a pass, and a sandbox denial is
  containment working rather than a bug to route around.
- **No rendered artifact surface.** The scorecard and gate verdict degrade to
  the plain tables the tools already print, per
  [`../INTERFACES/artifact.md`](../INTERFACES/artifact.md).

## What is false here that is true elsewhere

This adapter's discovery mechanism is `AGENTS.md` at the repository root, which
does not exist on Claude, ChatGPT or Gemini. Its evidence standard is a pasted
exit code, which the two chat targets cannot produce at all. Do not carry the
exit code standard into `chatgpt.md` or `gemini.md` and quietly satisfy it with
a claim.
