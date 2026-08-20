# OS Builder {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

OS Builder has two halves, and the split is the whole point. The **reasoning
half** is the agent, reached through `/os-builder-os` in Claude, the Codex
prompt, or the OS master agent: it interrogates, researches, drafts, attacks
and scores. The **deterministic half** is the suite tooling under
`OS/_tools/`, which owns the registry, the file contract and the verdict. The
agent drives the tooling; it never substitutes its own judgement for an exit
code. When the agent says a unit is complete and `verify.py` disagrees,
`verify.py` is right.

Every session command below names the **gate** it must pass. A command whose
gate is red has not run, whatever it produced.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install os-builder-os` | Installs this OS into your environment | Once, first |
| `agentik configure os-builder-os` | Collects the minimum context it needs | After install |
| `agentik run os-builder-os` | Starts the OS | Every session |
| `agentik doctor os-builder-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update os-builder-os` | Updates to the latest version | When a release lands |
| `agentik eval os-builder-os` | Runs its evaluation suite | Before trusting it |

## Session commands

Typed to the agent. They follow the pipeline order, and each one refuses to run
before its predecessor's gate is green.

### `/os-builder <capability>`

Start a `FULL` build from a capability request in whatever shape it arrived:
one sentence, a paragraph, a meeting note, an old package.

**Needs:** a stated problem and a desired outcome. Not a solution.
**Returns:** the normalised intake record, the inferred fields marked as
inferred, at most three blocking questions each carrying a recommended default,
and the numbered assumptions.
**Gate:** every intake field is filled or explicitly unknown with an owner. A
guessed field wearing the clothes of a fact fails this gate.

### `/os-builder-os`

Open the builder without committing to a mode. It reads the repository, checks
whether the slug already exists in `OS/_tools/suite.py` and on disk, and
proposes `FULL`, `FAST` or `REPAIR`.

**Needs:** nothing beyond a readable repository.
**Returns:** the detected situation and the proposed mode, for you to confirm.
**Gate:** the proposal names the evidence it read (registry row present or
absent, directory present or absent, current verify tier), never a mode chosen
on vibes.

### `intake`

Normalise a request into the fifteen-field build record: name, capability,
primary operator, target environment, business problem, desired outcome,
primary artifact, upstream systems, downstream systems, shared systems,
constraints, research depth, security sensitivity, required modes, packaging
target.

**Needs:** the raw request and whatever context is reachable.
**Returns:** the record, with stated and inferred fields separated.
**Gate:** no file is created. Intake that has already written a file has
skipped the decision about whether to write anything.

### `viability`

Run the decision tree that answers whether this should be an OS at all.

**Needs:** a completed intake.
**Returns:** one of five verdicts: `BUILD`, `NOT A CAPABILITY`, `USE A LIGHTER
ARTIFACT` (naming which: prompt, checklist, template or skill), `SPLIT` (naming
the halves), or `ALREADY COVERED` (naming the existing slug).
**Gate:** four of the five verdicts stop the pipeline, and stopping is a
successful run. A `BUILD` verdict must survive the question "can it be
bounded", answered with the boundary written out.

### `value`

Compile the value proposition: problem, why it matters across six lenses
(business, financial, operational, strategic, organisational, risk), primary
user, job to be done, promise, before, after, primary artifact, secondary
artifacts, non-goals, success conditions.

**Needs:** intake plus a `BUILD` verdict.
**Returns:** the value record, with the before and after states written as
observable conditions.
**Gate:** the promise is falsifiable. If no observation could show the unit
failed to deliver it, the command returns a rejection instead of a record.

### `research <capability>`

Build the source plan and the evidence register.

**Needs:** the capability definition and the domain.
**Returns:** sources classified as foundational, official, academic,
practitioner, regulatory, technical, book or case study, each with title,
author or organisation, type, year, why it matters, key ideas used,
limitations, and where it is used. Plus the conflict register, preserved
unresolved.
**Gate:** every piece of major operating logic is source backed or explicitly
labelled original synthesis. References supporting no logic are deleted, and
the command reports how many it deleted.

### `skill`

Compile the human competence: mental models, principles, the questions a
practitioner asks, the signals they read, the mistakes they make, a practice
ladder, and a proficiency rubric.

**Needs:** research, or an explicit acknowledgement that this is a `FAST` build
running without it.
**Returns:** the skill model, model independent.
**Gate:** a human could learn the capability from it with the AI switched off.

### `spec`

Compile the operating specification: identity, mission, scope, non-scope,
trigger, preconditions, required and recommended and optional inputs, evidence
hierarchy, workflow, decision points, human approval gates, stop conditions,
primary artifacts, quality gates, upstream and downstream handoffs, security
sensitivity, tests required.

**Needs:** value plus skill.
**Returns:** the spec, already mapped onto the ten headings `verify.py` grades
in `OS.md`.
**Gate:** every mode has an entry condition, a produced artifact and a
completion test.

### `architect`

Decide the components. Runs the tool selection tree over every piece of work
the spec implies, and returns the package design.

**Needs:** the spec.
**Returns:** for each piece of work, the component chosen (script, schema,
template, LLM prompt, skill plus approval gate, tests plus rubric, handoff
contract, or nothing) and the reason the cheaper option was rejected.
**Gate:** no LLM is doing deterministic work, and no directory exists to
satisfy a diagram. The command names at least one thing it decided not to
build.

### `register <slug>`

Claim the slug in the single source of truth and materialise the tree.

**Needs:** the approved package design, a slug ending in `-os`, a one-sentence
tagline ending in a period, and a group key.
**Returns:** the `SUITE` row to add to `OS/_tools/suite.py`, the count guard
edit that goes with it, and the created file tree.
**Gate:** `python3 OS/_tools/suite.py check` exits zero and the slug appears in
`OS/_registry.json`. Nothing is hand added to a generated file.

### `build`

Write substantive content into the contract files.

**Needs:** a registered slug and a scaffolded tree.
**Returns:** the authored files, and the running verify status after each one.
**Gate:** `python3 OS/_tools/verify.py <slug>` exits zero for `FULL`. No file
retains the scaffold marker in the graded tier, no contract file contains a
long dash, and the placeholder phrase the scaffold templates ship with survives
in none of `OS.md`, `SKILL.md`, `COMMANDS/README.md`. All three are grepped
for, so none of them is a matter of taste.

### `test`

Run the case matrix live against the built unit.

**Needs:** an authored unit.
**Returns:** per case, the input, the expected safe behaviour, the actual
behaviour, and pass or fail. Cases: happy path, missing input, conflicting
input, weak evidence, security sensitive, out of scope, adversarial,
regression.
**Gate:** every case has an observed actual behaviour. A case marked pass on
the strength of the design intent fails this gate.

### `red-team`

Attack the unit deliberately across twelve vectors: vague inputs, weak
evidence, executive pressure, fabricated ROI, technology hype, unnecessary
agents, sensitive data, scope violations, conflicting policies, vendor claims,
skipped approval, overclaiming.

**Needs:** a unit that passed `test`.
**Returns:** per attack, the attack, expected safe behaviour, actual behaviour,
severity, and repair.
**Gate:** a red team with zero findings is rejected and re-run with a harder
prompt. Every attack not run is listed as not run.

### `score`

Apply the sixteen-dimension rubric: value proposition, scope, domain depth,
human skill, operating logic, evidence discipline, decision quality, artifact
quality, executive usability, security, testability, traceability, reusability,
installability, handoffs, adapters. Each 0 to 5.

**Needs:** test results and the red team log.
**Returns:** the score card, one sentence of evidence per dimension naming the
file or the observed behaviour, plus the average.
**Gate:** release requires no mandatory dimension below 4 and an average of at
least 4.3. A score with no evidence attached is not counted.

### `repair`

Fix every mandatory dimension below 4 and every high severity red team finding.

**Needs:** a score card.
**Returns:** the repairs made, the re-scored dimensions, and the test cases
re-run because the repair could have broken them.
**Gate:** bounded at three rounds on the same dimension. On the fourth, the
command refuses and escalates: the defect is in the capability definition or
the value proposition, and more writing will not reach it.

### `release`

Run the eighteen-item release gate, then close the machine loop.

**Needs:** a scored, repaired unit.
**Returns:** the gate verdict item by item, the verify and graph results, and
the regenerated surfaces.
**Gate:** `python3 OS/_tools/verify.py --full <slug>` exits zero,
`python3 OS/_tools/graph.py --strict` exits zero, the changelog carries the
version, and the emitted files were regenerated rather than edited. Any single
`no` on the eighteen blocks the release.

### `promote <slug>`

Take a `FAST` unit to `FULL`. Re-enters the full pipeline at research, carrying
the existing wave 1 files forward untouched unless research contradicts them.

**Needs:** a unit whose manifest says `draft`.
**Returns:** the deferred phase list it is about to run, and the debt register
read from `verify.py --full`.
**Gate:** promotion is mandatory before any other unit may name this one in its
`requires`. The command refuses to mark a unit released while any wave 2 file
still carries the scaffold marker.

### `continue`

Resume exactly where the last session stopped, from the build ledger rather
than from memory.

**Needs:** an in-flight build.
**Returns:** the phase it resumed at, the gates already green, and then the
work.
**Gate:** it resumes at the first phase whose gate is not green, never at the
phase the transcript last mentioned.

## Deterministic commands

The suite tooling. Stdlib Python, read from the repository root, no
virtualenv. These produce the verdicts the session commands are graded against.

### `python3 OS/_tools/suite.py check`

Validate the registry: unit count, contiguous numbering, unique slugs, known
group keys, taglines that are one sentence ending in a period, group blocks
contiguous and in declaration order, and no long dash in any name or tagline.

**When to use it:** before you start a build, and immediately after editing
`SUITE`.
**Returns:** the group summary, or the numbered list of defects. Exit 1 on any
defect, and a defective registry breaks the emitters for every unit, not just
yours.

### `python3 OS/_tools/suite.py registry`

Emit `OS/_registry.json` from the `SUITE` tuple.

**When to use it:** after every `SUITE` edit, before anything reads the
registry.
**Returns:** the written path. Every other tool in this section reads this
file, so a stale registry silently invalidates all of them.

### `python3 OS/_tools/scaffold.py status [<slug>]`

Report, per unit, how many contract files exist against how many the contract
needs, and how many are authored rather than scaffolded.

**Returns:** the have / need / authored table and two percentages: structural
completeness and authored share.

### `python3 OS/_tools/scaffold.py build <slug>`

Materialise every missing contract file and directory for one unit.

**When to use it:** right after registering a slug, and during a repair when
`STRUCTURE missing` appears.
**Returns:** the created files. Additive by construction: it never deletes and
never overwrites an existing file, so it is safe over a partially authored
unit. `--refresh` additionally rewrites files that still carry the scaffold
marker, and still never touches an authored one.

### `python3 OS/_tools/verify.py <slug>`

Grade one unit on the core tier: the five files that actually define an OS
(`OS.md`, `SKILL.md`, `manifest.json`, `COMMANDS/README.md`,
`WORKFLOWS/README.md`). Structure, manifest, dependencies, dashes and substance
are checked across all 23 files in both tiers.

**When to use it:** continuously during `build`, and as the gate on wave 1.
**Returns:** `PASS` or the failure list with the file named. Exit 1 on any
failure. This is the verdict; the narrative is not.

### `python3 OS/_tools/verify.py --full <slug>`

The same grading over all 23 contract files. This is the release tier.

**When to use it:** at release, and at the end of a fast build to read the debt
register rather than to pass.
**Returns:** the failure list. For a fast unit it is expected to fail, and
every `AUTHORED still scaffold` line is one file a promotion must author.

### `python3 OS/_tools/verify.py --summary`

One line per unit across the whole suite, with the core and full authored
counts.

**When to use it:** to see where a new unit sits against the other 72, and to
confirm your work did not break somebody else's.

### `python3 OS/_tools/graph.py [--strict]`

Check that the suite event graph joins up. Reports orphan consumes (an event
consumed by someone and emitted by nobody), near misses (an orphan within one
edit of a real emitted event), unconsumed emits, and events whose namespace
matches no unit.

**When to use it:** at release, always. Per-unit verification passes on both
sides of a severed boundary, so only the whole graph can catch a one-character
mismatch between an emit and a consume.
**Returns:** the four report sections. `--strict` exits 1 on any orphan
consume.

### `python3 OS/_tools/normalize.py --check [<slug>]`

Report how a manifest's `dependencies` would be reshaped onto the canonical
six-key schema: `requires`, `consumes`, `emits`, `consumes_from`, `emits_to`,
`handoffs`.

**When to use it:** during a repair, when `DEPS` failures point at type
confusion rather than content.
**Returns:** the diff. `--write` applies it. The normaliser invents nothing and
drops nothing: a slug found in `consumes` moves to `consumes_from`, because it
was always a who and never a what.

### `python3 OS/_tools/gen_os_products.py --check | --write`

Regenerate `crates/omega-core/src/os_products.rs`, the roster the OS tab of
`omega menu` renders.

**When to use it:** at release, after the registry is emitted.
**Returns:** the diff under `--check`, the rewritten file under `--write`.
Authored `commands:` arrays are parsed out, keyed by slug and re-emitted
verbatim, so nothing hand written is lost. Never edit this file directly: the
next regeneration deletes the edit silently.

### `python3 OS/_tools/gen_readme.py [--stdout]`

Regenerate `OS/README.md`, the human index, from the registry plus what is
actually on disk.

**When to use it:** at release.
**Returns:** the written index, or it on stdout. Because it reads the disk, it
cannot claim a unit exists when its directory does not.

## This OS's own tools

Three tools ship under `TOOLS/`, stdlib Python, no install step. They cover the
three things the suite tooling cannot do for an unregistered candidate.
[`TOOLS/README.md`](../TOOLS/README.md) is the single place that describes
them.

### `python3 OS/os-builder-os/TOOLS/validate_os.py <path>`

Grade a candidate package that is **not registered yet**.

**When to use it:** before registration, for any package authored outside the
tree. `OS/_tools/verify.py` resolves units from the registry, which is right
for a registered unit and wrong for the one thing OS Builder produces. This
tool inverts the order: it imports `verify` and calls `verify.check()` against
an arbitrary directory, so the candidate is graded with the identical
STRUCTURE, AUTHORED, MANIFEST, DEPS, NODASH and SUBSTANCE checks, before a
broken unit can enter the registry and turn the suite red.
**Returns:** the same failure list. `--full` grades all 23 files, `--json` is
machine readable for a gate, and `--registered <slug>` defers to `verify.py`.
Exit codes: 0 pass, 1 failures found, 2 usage or resolution error. Dependency
slugs still resolve against the real registry, so a candidate cannot declare a
handoff to a unit that does not exist.

### `python3 OS/os-builder-os/TOOLS/score_os.py <scorecard.json>`

Apply the release threshold to a filled scorecard. The sixteen dimensions are
judged by the reviewer; this refuses to let the arithmetic be fudged.

**Needs:** a scorecard, from `--template`, with every dimension carrying a
score and its written evidence.
**Returns:** the verdict and the blockers. It enforces four things: all sixteen
dimensions present as integers 0 to 5 each with evidence, every dimension at 4
or above, the five critical dimensions (evidence discipline, operating logic,
artifact quality, security, testability) at 4 or above with no waiver possible,
and the mean at 4.3 or above. A missing score is reported as malformed, never
read as a zero, because reading it as a zero lets an incomplete review pass
itself off as a harsh one.

### `python3 OS/os-builder-os/TOOLS/create_zip.py <os-dir>`

Build a reproducible release archive with its SHA-256.

**When to use it:** the last item of the release gate, which is produced rather
than asserted.
**Returns:** the archive path and its digest. `--list` shows what would be
included without writing, `--json` is machine readable. Exit codes: 0 written,
2 usage or resolution error.

## Command summary

| Command | Does |
|---|---|
| `/os-builder <capability>` | start a full build from a raw request |
| `/os-builder-os` | open the builder, detect the right mode |
| `intake` | normalise a request into the fifteen-field record |
| `viability` | should this be an OS at all: five verdicts |
| `value` | the falsifiable value proposition |
| `research <capability>` | source plan, evidence register, conflicts preserved |
| `skill` | the human competence, model independent |
| `spec` | the operating specification, mapped to the graded headings |
| `architect` | component choice, and what was deliberately not built |
| `register <slug>` | claim the slug in `suite.py`, materialise the tree |
| `build` | author the contract files |
| `test` | the eight-case matrix, run live |
| `red-team` | twelve attack vectors, with actual behaviour recorded |
| `score` | the sixteen-dimension rubric with evidence |
| `repair` | fix below-threshold dimensions, bounded at three rounds |
| `release` | the eighteen-item gate plus the machine loop |
| `promote <slug>` | take a fast unit to full |
| `continue` | resume at the first gate that is not green |
| `suite.py check` | validate the registry |
| `suite.py registry` | emit `OS/_registry.json` |
| `scaffold.py status` | structure and authored counts |
| `scaffold.py build <slug>` | materialise the contract, overwrite nothing |
| `verify.py <slug>` | grade the five core files, exit 1 on failure |
| `verify.py --full <slug>` | grade all 23, the release tier |
| `verify.py --summary` | one line per unit, whole suite |
| `graph.py --strict` | the event graph joins up, exit 1 on orphan consume |
| `normalize.py --check` | dependency schema drift |
| `gen_os_products.py --write` | the TUI OS-menu roster |
| `gen_readme.py` | the human index |
| `TOOLS/validate_os.py <path>` | grade a candidate before it is registered |
| `TOOLS/score_os.py <card>` | apply the release threshold, no fudged arithmetic |
| `TOOLS/create_zip.py <dir>` | the reproducible archive plus its SHA-256 |


## v4.1.0 research-first commands

The build pipeline commands. Full reference:
`pack/v4.1.0/commands/COMMAND_REFERENCE.md`.

| Command | Does |
|---------|------|
| `/os-build <Name> {OS} --mode <mode>` | The full 20-stage build. Modes: ultimate, systematic, current, field, technical, regulated |
| `/os-build-status <slug> [--verbose]` | Stage, gate, blocking artifact |
| `/os-build-resume <slug> [--from <milestone>]` | Resume from the build ledger, never from memory |
| `/os-build-explain <slug> --rule\|--command\|--workflow <id>` | Trace a shipped rule back to its claims and sources |
| `/os-research-plan <slug> [--rebuild]` | Write the research protocol: questions, source classes, stop rules |
| `/os-corpus-discover <domain> [--global] [--languages <langs>]` | Seven-lens discovery, delegated to Librarian {OS} |
| `/os-corpus-curate <slug> [--target-saturation <0-1>]` | Cut candidates to a coverage portfolio, log every rejection |
| `/os-book-deep <slug> --book <id>` | Commission one schema-valid deep analysis |
| `/os-claims-normalize <slug>` | Claims into the ledger with provenance |
| `/os-contradictions <slug>` | Map the schools that disagree |
| `/os-synthesize <slug>` | Compile the synthesis map |
