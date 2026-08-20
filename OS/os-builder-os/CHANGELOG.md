# Changelog: OS Builder {OS}

## 4.1.0 . 2026-08-19

Adopted the operator pack `agentik-builder-os-v4.1.0-public.zip`, vendored
verbatim at `pack/v4.1.0/` (75 files, never edited). The Forge becomes
research-first.

- **New canonical pipeline**: 20 stages, 00 CONTRACT to 19 CONTINUOUS UPDATE,
  in `WORKFLOWS/ULTIMATE_BUILD.md`. Architecture can no longer precede synthesis.
- **Corpus 360**: seven discovery lenses (bestsellers, foundations, evidence-led,
  practitioner, specialist, current, critical). The corpus is a coverage
  portfolio, not a bestseller list.
- **Deep book analysis** replaces book summarising, against a fixed five-section
  standard and a JSON schema. Access honesty and analyst confidence are
  mandatory fields.
- **Knowledge synthesis protocol**: normalize concepts, normalize claims, cluster
  by mechanism, classify agreement (robust, promising, conditional, disputed,
  outdated). Disputed claims ship as disputed.
- **Librarian {OS} promoted** to owner of stages 04, 05 and 06 with a written
  contract in `OS/librarian-os/REFERENCES/BOOK-INTELLIGENCE-CONTRACT.md`.
- **11 new commands**: corpus discovery and curation, book deep, claim
  normalisation, contradiction mapping, synthesis, build status, resume, explain.
- **Traceability**: `/os-build-explain` traces any shipped rule back to the
  claims and the sources that produced it.
- **Public no-secrets policy**: a generated OS runs with no credential.
- Six operator-pack schemas and eleven templates now available to every build.
- Declared divergences from the pack recorded in `REFERENCES/PACK-v4.1.0.md`.


All notable changes to this OS are recorded here. The Runtime reads this file
for `agentik update os-builder-os`, so every version bump needs an entry.
Semantic versioning: MAJOR is a breaking change to a workflow, a schema or the
behaviour contract; MINOR is a compatible capability or asset addition; PATCH
is a correction or a non-breaking improvement.

## [0.1.0] (unreleased)

First authored version of unit 00, ported from the upstream OS Builder {OS}
payload (version 1.0.0, 2026-08-15) into the AGENTIK {OS} suite.

### Added
- `OS.md`: the ten section operating specification, with thirteen operating
  modes from `INTAKE` through `RELEASE` plus `AUDIT` and `REBUILD`, thirteen
  invariants, and a failure table that names a response per failure.
- `SKILL.md`: the capability architecture skill, its near-neighbour
  discriminators against Agentik Runtime {OS}, Blueprint {OS}, Stepper {OS},
  Builder {OS}, Quality & Evaluation {OS}, Research {OS}, Evaluation {OS} and
  Agent {OS}, and the sixteen step procedure.
- `SYSTEM.md`: the behaviour contract, the evidence states, the simplicity
  ladder, the stop-and-repair conditions and the release threshold.
- `manifest.json`: eighteen commands across the session and CLI surfaces, the
  dependency graph as events plus artifact handoffs, per-target notes, and
  seven actions gated on human approval.
- `README.md` and `SETUP.md`: the human entry point and the four required
  configuration inputs.

### Changed from the upstream payload
- **Built against this suite's real contract.** The upstream defined its own
  generic package standard: 9 root files and 19 directories, with no grader
  behind it beyond a six file presence check. This unit builds
  against the AGENTIK {OS} 23 file contract declared in `OS/_registry.json`,
  materialised by `OS/_tools/scaffold.py` and graded by `OS/_tools/verify.py`,
  authored in two waves: wave 1 is `CORE_FILES` (`OS.md`, `SKILL.md`,
  `manifest.json`, `COMMANDS/README.md`, `WORKFLOWS/README.md`), wave 2 is the
  remaining eighteen surfaces.
- **The grader is named and binding.** The upstream `validate_package.py`
  checked six files and four manifest keys. `OS/_tools/verify.py` runs six
  checks (STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH, SUBSTANCE) across all 23
  files at two tiers, and a red grader now blocks release explicitly.
- **The Runtime boundary is written down.** OS Builder BUILDS an OS; Agentik
  Runtime {OS} INSTALLS, RUNS and UPDATES it. Neither duplicates the other, and
  the packaging and ZIP stages of the upstream release phase are replaced by a
  handoff to the Runtime.
- **Position 0 is explained.** Unit 00 declares no hard dependency, because the
  first OS on a machine has to be buildable when nothing else is installed.
- **The approval boundary is concrete.** The upstream said high-impact actions
  need human approval. This unit names them: registering a slug in
  `OS/_tools/suite.py`, releasing, writing into an existing unit, waiving a
  gate, changing a neighbour's boundary, publishing off the machine, and
  building in a consequence-bearing domain.

### Preserved from the upstream payload
- The pipeline: IDEA, VALUE, RESEARCH, SKILL, WORKFLOW, ARTIFACTS, PACKAGE,
  TEST, RED TEAM, SCORE, REPAIR, RELEASE.
- The ten step definition of done, now mapped file by file onto the contract.
- The six evidence states and the three confidence levels.
- The simplicity ladder, from DO NOTHING to MULTI-AGENT.
- The sixteen dimension quality rubric and the release threshold: no mandatory
  dimension below 4, average at or above 4.3.
- The stop conditions, and DO NOT BUILD as a successful outcome.

### Known gaps
- The upstream JSON schemas (`os_spec`, `evidence`) are not yet carried into
  this unit; the same fields are enforced in prose by `OS.md` and `SYSTEM.md`.
- The unit is already registered in `OS/_tools/suite.py` and `_registry.json`
  at num 0, but `OS/README.md` still reports it absent. Regenerating the
  derived artifacts (`gen_readme.py`, `gen_os_products.py`) is a human action
  and is not part of this change.
- `verify.py --full` cannot pass until the wave 2 directory surfaces are
  authored. Wave 1 passes for the files in this change.
