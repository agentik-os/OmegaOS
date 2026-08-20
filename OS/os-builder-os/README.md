# OS Builder {OS}

> Build an operative system itself: intake, spec, research, build, red team,
> score, release.

**Suite position:** `00` in **00 · RUNTIME** (Build and run the entire Agentik
ecosystem). First unit in the suite, because it is the unit that produces the
others.

## What this OS is for

Every other unit in the suite is a capability. This one is the factory. It
takes a vague capability request and returns a complete, bounded, teachable,
testable, evidence-based, installable OS, or a defensible refusal to build one.

```
IDEA -> VALUE -> RESEARCH -> SKILL -> WORKFLOW -> ARTIFACTS -> PACKAGE
     -> TEST -> RED TEAM -> SCORE -> REPAIR -> RELEASE
```

An OS is not a giant prompt and not a decorative folder tree. It is an
operating environment for one repeatable professional capability. The test is
behavioural: a competent operator who has never seen it must be able to
**INSTALL** it, **UNDERSTAND** it, **LEARN THE SKILL** behind it, **LOAD
CONTEXT**, **EXECUTE**, **COLLECT EVIDENCE**, **DECIDE**, **PRODUCE
ARTIFACTS**, **REVIEW** and **HAND OFF**. Each of those ten steps is carried by
a named file of the contract. A step with no file behind it is what an
unfinished OS looks like.

Use it when the object of the work is an OS: building a new one, rebuilding an
old one, auditing one against the contract, or deciding that the right answer
is a checklist and not an OS at all.

## OS Builder and the Runtime, which is which

The two units of group 00 are complements and never overlap:

| | OS Builder {OS} | Agentik Runtime {OS} |
|---|---|---|
| Verb | BUILDS an OS | INSTALLS, RUNS and UPDATES it |
| Output | a package of 23 files plus a manifest | a working unit on a machine |
| Writes | inside the slug it is building | nothing, ever |
| Ends at | the release record, awaiting a human | the moment the user gets a result |

OS Builder never installs anything and never starts a session for the unit it
just built. The Runtime never authors a line of any unit. Neither should learn
the other's job.

## The contract it builds against

The suite has one file contract, 23 graded files per unit, declared in
`OS/_registry.json`, materialised by `OS/_tools/scaffold.py` and graded by
`OS/_tools/verify.py`. It is authored in two waves:

- **Wave 1, CORE (5 files):** `OS.md`, `SKILL.md`, `manifest.json`,
  `COMMANDS/README.md`, `WORKFLOWS/README.md`. With these authored the unit is
  usable and the Runtime can resolve it.
- **Wave 2 (18 files):** the surfaces, from `README.md` and `SETUP.md` through
  `MEMORY/policy.md`, `EVALS/`, the four `INTERFACES/` files and the four
  `ADAPTERS/` files. They make the unit teachable, portable and reviewable.

```bash
python3 OS/_tools/scaffold.py build <slug>   # materialise the 23 files, overwrites nothing
python3 OS/_tools/verify.py <slug>           # grade wave 1 (CORE)
python3 OS/_tools/verify.py --full <slug>    # grade all 23
python3 OS/_tools/verify.py --summary        # one line per unit, whole suite
```

The grader runs six checks: STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH and
SUBSTANCE. A red grader blocks release and is never argued with.

## Start here

| You want to | Read |
|---|---|
| Understand what it does | this file |
| Understand how it operates | [`OS.md`](OS.md) |
| See the AI behaviour contract | [`SYSTEM.md`](SYSTEM.md) |
| See what it can do | [`SKILL.md`](SKILL.md) |
| Configure it for yourself | [`SETUP.md`](SETUP.md) |
| See every command | [`COMMANDS/`](COMMANDS/) |
| See the build pipeline as a process | [`WORKFLOWS/`](WORKFLOWS/) |
| See it in use | [`EXAMPLES/`](EXAMPLES/) |

## Install and run

```bash
agentik install os-builder-os          # install this OS
agentik configure os-builder-os        # answer the minimum setup questions
agentik run os-builder-os              # start using it
```

Inside OmegaOS it is also reachable from `omega menu`, **OS** tab, entry
`00. OS Builder {OS}`.

## What a run gives you

- The package: `OS/<slug>/`, 23 files, passing the grader at both tiers.
- The OS spec: scope, non-scope, trigger, evidence rules, gates, stop
  conditions, handoffs, all agreed before a file was written.
- The source register: each source with why it matters, what was used from it,
  its limitations and where it is used.
- The test and red-team ledger, with the real result of every case and a repair
  for every attack.
- The score sheet: 16 dimensions, each with the evidence behind its number.
- The release record: version, open risks, named unknowns, and the human
  approval recorded against them.

## What it will not do alone

Releasing an OS into the suite is hard to reverse. OS Builder proposes and a
human disposes for: registering a slug or a number, releasing a unit, writing
into an existing unit's directory, waiving a gate, changing a neighbour's
declared boundary, publishing off this machine, and building in a domain that
carries real-world consequence. The full list is `OS.md` section 9.

## Structure

```
os-builder-os/
├── README.md      this file, the human entry point
├── OS.md          the complete operating specification
├── SYSTEM.md      AI and system instructions
├── SKILL.md       capabilities and procedures
├── SETUP.md       initial configuration
├── manifest.json  machine-readable metadata
├── CHANGELOG.md   what changed between versions
├── WORKFLOWS/     the full build and the fast build
├── COMMANDS/      every command, explained
├── PROMPTS/       intake, architect, research, build, red team, review
├── REFERENCES/    the standards this OS builds against
├── MEMORY/        what may be remembered, updated, forgotten
├── TOOLS/         scaffold.py, verify.py and what they cost
├── EVALS/         tests that prove it behaves correctly
├── EXAMPLES/      a worked build, end to end
├── INTERFACES/    chat, artifact, dashboard, generative UI
└── ADAPTERS/      ChatGPT, Claude, Gemini, Codex
```

## Provenance

Ported from the upstream OS Builder {OS} payload (version 1.0.0, 2026-08-15),
which carried its own generic package standard. This unit builds against the
AGENTIK {OS} 23 file contract instead, and its grader is
`OS/_tools/verify.py`. The upstream pipeline, evidence states, simplicity
ladder, 16 dimension rubric and release threshold are preserved as written.
