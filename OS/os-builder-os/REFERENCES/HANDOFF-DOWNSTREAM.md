# Downstream Handoff

What a finished OS must expose so another system can install it, run it, depend
on it and grade it, without reading its prose.

The deliverable is a **standalone OS package**. Standalone means a competent
operator who has never spoken to the builder can take the directory, install it,
and get the promised artifact.

## The exposed surface

Six things are machine-readable and must be correct, because six different
consumers read them and none of them read the README.

| Exposed | Where | Read by |
|---|---|---|
| identity and metadata | `manifest.json` keys `id`, `num`, `name`, `version`, `group`, `tagline` | the Runtime, `gen_readme.py`, `gen_os_products.py` |
| entrypoint | `manifest.json` `entrypoints`, and `OS.md` as the spec | the Runtime and every adapter |
| inputs and outputs | `OS.md` sections 4 and 5 | the operator, and any composing OS |
| the dependency graph | `manifest.json` `dependencies` | `graph.py`, `normalize.py`, the Runtime resolver |
| version and change history | `manifest.json` `version`, `CHANGELOG.md` | `agentik update <slug>` |
| quality status | the release scorecard and the gate verdict | the operator deciding whether to trust it |

## Consumption targets

A released OS must be usable in all five of these, or must state in
`ADAPTERS/<target>.md` exactly what it cannot do there and what it falls back
to. Silent degradation is the failure; declared degradation is acceptable.

1. **Claude Code**, via `ADAPTERS/claude.md` and the `SKILL.md` front matter.
2. **Codex**, via `ADAPTERS/codex.md` and an `AGENTS.md` placement.
3. **A generic LLM chat surface** (ChatGPT, Gemini and equivalents), via the
   corresponding adapter and a single pasteable system prompt.
4. **Repository installation**, as a directory under `OS/<slug>/` graded by
   `OS/_tools/verify.py`.
5. **A wider runtime**, via `manifest.json` alone: an orchestrator that never
   reads the markdown must still be able to install, resolve dependencies,
   invoke a command, and know what came back.

## Registration order

A finished package enters the suite in this order and no other. Every step
before registration is reversible; registration is the point where a defect
becomes everyone's problem.

```bash
# 1. grade the candidate where it stands, unregistered
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full

# 2. only on PASS, add the unit to the single source of truth
#    (edit the SUITE tuple in OS/_tools/suite.py, never _registry.json by hand)
python3 OS/_tools/suite.py check
python3 OS/_tools/suite.py registry

# 3. normalise the dependency schema and confirm the graph still joins
python3 OS/_tools/normalize.py --check
python3 OS/_tools/graph.py --strict

# 4. regenerate the derived surfaces
python3 OS/_tools/gen_readme.py
python3 OS/_tools/gen_os_products.py --check

# 5. grade it again, now as a registered unit
python3 OS/_tools/verify.py <slug> --full
```

Step 5 is not redundant. It is the first run where `num` is checked against the
registry rather than against the candidate's own claim, and where every handoff
slug resolves against the live suite.

## The handoff contract per consumer

**To the Runtime.** `manifest.json` must be complete and its `commands` array
non empty. `requires` names hard dependencies by slug; the Runtime refuses to
install an OS whose requirements are absent, so a missing entry produces a
runtime failure rather than a graceful one.

**To another OS.** Named artifact handoffs go in `dependencies.handoffs` as
`{to: slug, artifact: string}`. The artifact string is what the receiving OS
looks for, so it is written from the receiver's vocabulary, not the producer's.
Event coupling goes in `emits` and `consumes` as dotted event names, and the
whole graph must join: an event consumed by someone and emitted by nobody is an
orphan and it means the boundary is decorative.

**To the operator.** `README.md` says what it is and when to reach for it.
`SETUP.md` says the minimum to be useful now, not everything configurable.
`EXAMPLES/` shows one real run end to end. If the operator has to read `OS.md`
to get started, the handoff has failed even though every file is present.

**To an evaluator.** `EVALS/` carries the suites, the rubric and the gate, and
the released scorecard states what it actually scored, who scored it, and on
what date. A quality claim with no scorecard behind it is marketing.

## What is never handed downstream

- The operator's own data, credentials, or a licensed corpus. The package
  carries the pointer, never the payload.
- An `outputs/` directory. Outputs belong to whoever ran the OS.
- A quality claim the builder did not compute. "Production ready" without a
  scorecard is exactly the unsupported major claim the release gate blocks.
- A dependency on a slug that is not in `OS/_registry.json`.
