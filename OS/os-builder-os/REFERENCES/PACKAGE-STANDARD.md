# Package Standard

The shape of a finished OS. This is the target OS Builder builds toward and the
thing `TOOLS/validate_os.py` grades a candidate against before it is allowed
into the registry.

## The contract is 7 files and 10 directories

The AGENTIK {OS} suite contract is defined in `OS/_registry.json` under
`contract` and is enforced by `OS/_tools/verify.py`. It is not negotiable and it
is not a suggestion, because the Runtime, the TUI roster generator and the human
index all read it.

```
<slug>/
  README.md          human entry point: what it is, when to reach for it
  OS.md              the operating specification, 10 required sections
  SYSTEM.md          the AI behaviour contract
  SKILL.md           capabilities and procedures, with YAML front matter
  SETUP.md           the minimum configuration to be useful now
  manifest.json      machine-readable metadata, 13 required keys
  CHANGELOG.md       what changed between versions
  WORKFLOWS/         repeatable processes, one file per process
  COMMANDS/          every command, explained; undocumented means nonexistent
  PROMPTS/           reusable prompt units with input contract and output shape
  REFERENCES/        knowledge the OS needs, each with source and trust class
  MEMORY/policy.md   what may be remembered, updated, forgotten
  TOOLS/             external capabilities, permission, and fallback
  EVALS/             the tests that prove it behaves correctly
  EXAMPLES/          worked examples, opening move to finished artifact
  INTERFACES/        chat.md artifact.md dashboard.md generative-ui.md
  ADAPTERS/          chatgpt.md claude.md gemini.md codex.md
```

Twenty three files are graded. Five of them are the **CORE** tier (wave 1) and
define whether the OS exists at all: `OS.md`, `SKILL.md`, `manifest.json`,
`COMMANDS/README.md`, `WORKFLOWS/README.md`. The other eighteen are the surface
tier (wave 2). `verify.py` grades the two tiers separately on purpose, so
finished wave 1 work is not reported as failing for not yet being wave 2.

## The ten required sections of OS.md

`verify.py` matches these as level 2 headings, numbered or not. A missing
section is a `SUBSTANCE` failure and blocks at CORE tier:

`Purpose` · `Boundary` · `Operating modes` · `Inputs` · `Outputs` · `State` ·
`Rules and invariants` · `Failure behaviour` · `Human approval boundary` ·
`Completion criteria`

The literal string `to be authored` anywhere in `OS.md`, `SKILL.md` or
`COMMANDS/README.md` is a `SUBSTANCE` failure. So is the scaffold marker, the
HTML comment that `OS/_tools/scaffold.py` writes into every placeholder it
generates (its `MARK` constant): a graded file still carrying it counts as **not
authored**, whatever else is in it. Removing the marker without writing real
content simply moves the failure from `AUTHORED` to the reader.

## manifest.json

Thirteen keys are required: `schema_version`, `id`, `num`, `name`, `version`,
`group`, `tagline`, `commands`, `dependencies`, `targets`, `entrypoints`,
`requires_human_approval_for`. `id` must equal the directory name and `num` must
equal the registry entry.

`commands` may not be empty. `dependencies` may not be entirely empty across all
six of its keys. The canonical dependency schema, produced by
`OS/_tools/normalize.py`, is:

| Key | Element type | Meaning |
|---|---|---|
| `requires` | slug | hard dependency, must be installed |
| `consumes` | dotted event name | events this OS listens for |
| `emits` | dotted event name | events this OS publishes |
| `consumes_from` | slug | OSes it takes input from |
| `emits_to` | slug | OSes its output reaches |
| `handoffs` | `{to: slug, artifact: string}` | named artifact handoff, no event |

An event name is `namespace.thing.verb`, lowercase, at least two dots separated
parts. Putting a slug into `emits` is the specific mistake `verify.py` is
watching for, because it silently severs a boundary that looks connected.

An emitted event nobody consumes is informational. A **consumed event nobody
emits** is an orphan and it means the boundary does not join. Check the whole
graph with `python3 OS/_tools/graph.py --strict` before release: per unit
verification cannot see this class of defect, only the graph can.

## Optional components, and where each one lives

The upstream OS Builder canon named a richer tree than the suite contract:
`prompts`, `schemas`, `templates`, `assets`, `frameworks`, `scripts`, `modes`,
`learning`, `checklists`, `decision-trees`, `traceability`, `handoffs`,
`evaluation`, `outputs`, `tests`. None of that content is lost. Each maps onto a
contract directory as a subdirectory or a named file:

| Upstream component | Lives at | Include when |
|---|---|---|
| `prompts/` | `PROMPTS/` | always, one file per prompt unit |
| `schemas/` | `TOOLS/schemas/` | the OS exchanges structured data |
| `templates/` | `PROMPTS/templates/` | the OS produces a repeatable document |
| `frameworks/` | `REFERENCES/` | the OS teaches a named method |
| `modes/` | `OS.md` section 3, one row per mode | always |
| `learning/` | `SKILL.md`, practice ladder and proficiency rubric | always |
| `checklists/` | `WORKFLOWS/checklists/` | a phase has a preflight |
| `decision-trees/` | `WORKFLOWS/` or `REFERENCES/` | a real branch exists |
| `traceability/` | `REFERENCES/TRACEABILITY.md` | the OS makes claims |
| `handoffs/` | `REFERENCES/HANDOFF-*.md` plus manifest `handoffs` | always |
| `evaluation/` | `EVALS/` | always |
| `tests/` | `EVALS/TEST-PLAN.md` | always |
| `scripts/` | `TOOLS/` | deterministic work exists |
| `assets/` | `REFERENCES/assets/` | a real asset exists |
| `outputs/` | not shipped | never: outputs are the user's, not the package's |

## The two rules that keep a package honest

**Do not create empty folders.** A directory that exists to look complete is a
defect. The contract's ten directories each carry at least their required file
with real content; anything beyond that is included only when it adds value.

**Prefer fewer complete assets over many hollow files.** Package inflation is
one of the ten adversarial test cases in [`../EVALS/TEST-PLAN.md`](../EVALS/TEST-PLAN.md)
precisely because it is the most tempting failure: a large tree reads as
thorough at a glance and collapses on the first read of any single file.

## Validating a candidate

A candidate is graded **before** it is registered, never after. Registering
first means a broken unit enters `OS/_tools/suite.py`, the generators run over
it, and the whole suite goes red before anyone looks at it.

```bash
python3 OS/os-builder-os/TOOLS/validate_os.py <path/to/candidate>          # CORE
python3 OS/os-builder-os/TOOLS/validate_os.py <path/to/candidate> --full   # all 23
python3 OS/os-builder-os/TOOLS/validate_os.py <path/to/candidate> --json   # for a gate
```

Only when `--full` passes does the unit go into `OS/_tools/suite.py`, followed
by `python3 OS/_tools/suite.py registry`, then the generators. See
[`../TOOLS/README.md`](../TOOLS/README.md) for the full ordering.
