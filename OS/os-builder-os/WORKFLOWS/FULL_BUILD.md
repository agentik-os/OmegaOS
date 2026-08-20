# Workflow: Full build

**Mode:** `FULL`
**Produces:** one registered AGENTIK {OS} unit whose 23 contract files are
authored, whose rubric scores clear the release threshold, and for which
`python3 OS/_tools/verify.py --full <slug>` exits zero.

## Trigger

A capability request arrives (from the operator, from an academy capability,
from a workflow somebody runs by hand, or from an existing package to be
rebuilt), the viability tree returns BUILD, and no slug exists for it yet.

## Preconditions

- The requester can state the problem and the desired outcome in their own
  words. Not the solution, the outcome.
- The repository is present and `python3 OS/_tools/suite.py check` exits zero
  before you start. Building on an already invalid registry hides your own
  breakage inside somebody else's.
- The capability is not already covered by a unit in the suite. Check
  `OS/README.md` and the `SUITE` tuple before proposing a new one. Duplication
  is the most expensive defect this OS can ship, because it is invisible until
  two units disagree.

## The pipeline

```
IDEA -> VALUE -> RESEARCH -> SKILL -> WORKFLOW -> ARTIFACTS -> PACKAGE
     -> TEST -> RED TEAM -> SCORE -> REPAIR -> RELEASE
```

Fifteen phases carry it. Every phase boundary is a gate: a stated condition
that must hold before the next phase may start. A gate that is skipped is
recorded as skipped, never as passed.

---

## Phase 0. Intake

Normalise the request into one record. Fields, all of them, blank where
genuinely unknown rather than guessed:

name, capability, primary operator, target environment, business problem,
desired outcome, primary artifact, upstream systems, downstream systems,
shared systems, constraints, research depth, security sensitivity, required
modes, packaging target.

Separate what was stated from what you inferred. Label every inferred field as
inferred. Raise at most three blocking questions, and only where a wrong answer
means throwing work away rather than adjusting it. Everything else becomes a
numbered assumption with a reversal trigger.

Do not create a single file in this phase.

**Gate 0.** Every field is either filled or explicitly marked unknown with an
owner. No field is filled with a guess wearing the clothes of a fact.

---

## Phase 1. Capability definition

Define the job to be done, the scope, the non-scope, the upstream, the
downstream, and the primary artifact. Then run the viability tree
(`PROMPTS/00-intake.md`):

```
Repeatable professional capability?
  no  -> do not build an OS
  yes -> recurring decisions, workflow, or artifacts?
    no  -> a prompt, checklist, template or skill is the right size
    yes -> can it be bounded?
      no  -> split the capability and re-enter with each half
      yes -> does reusable operating infrastructure add value?
        no  -> a lighter artifact wins
        yes -> BUILD
```

The four non-BUILD leaves are real outputs. Returning "a checklist is the right
size for this" is a successful run of this OS, not a failure of it.

**Gate 1.** The capability can be explained to a competent stranger without
mentioning folders, prompts, files or models. If the explanation needs the
package to make sense, the capability is not yet a capability.

---

## Phase 2. Value proposition

Compile: problem, why it matters (business, financial, operational, strategic,
organisational, risk), primary user, job to be done, promise, before state,
after state, primary artifact, secondary artifacts, non-goals, success
conditions.

Before and after are the load bearing pair. "Fragmented opinions about
readiness" to "an evidence backed maturity model with named gaps" is a value
proposition. "Better decisions" is not.

**Gate 2.** The value is specific and falsifiable. Someone could run the unit
and demonstrate it did not deliver. If no observation could disconfirm the
promise, the promise is decoration.

---

## Phase 3. Research

Build the source plan. Cover, as relevant to the domain: foundational,
official, academic, practitioner, regulatory, technical, book, and case study
sources. For each source capture title, author or organisation, type, year, why
it matters, key ideas used, limitations, and where it is used in the unit.

Classify every claim the unit will make as source derived or original
synthesis. Both are legitimate. Silently presenting synthesis as established
practice is not.

Preserve conflicting sources as conflicts. Never average two disagreeing
authorities into a third position nobody holds.

The eight-field entry shape and the classification rules are fixed in
[`REFERENCES/REFERENCE-POLICY.md`](../REFERENCES/REFERENCE-POLICY.md). Use it
rather than inventing a register format per build.

**Gate 3.** Every piece of major operating logic is either source backed with a
citation or explicitly labelled original synthesis. References that support no
logic are deleted, not kept for weight.

---

## Phase 4. Human skill

Define the capability as a human competence, independent of any model: mental
models, principles, the questions a practitioner asks, the signals they read,
the mistakes they make, a practice ladder from novice to fluent, and a
proficiency rubric.

This phase is what separates an OS from a prompt. A prompt encodes an output.
An OS encodes a competence, of which the automated part is one expression.

**Gate 4.** A human could learn the capability from this section with the AI
switched off, and would be measurably better at it afterwards.

---

## Phase 5. Operating model

Compile the specification: identity, mission, scope, non-scope, trigger,
preconditions, required inputs, recommended inputs, optional inputs, evidence
hierarchy, workflow, decision points, human approval gates, stop conditions,
primary artifacts, quality gates, upstream handoffs, downstream handoffs,
security sensitivity, tests required.

The canonical operating loop, from which every unit's modes are cut:

```
TRIGGER -> INTAKE -> VALIDATE -> DISCOVER -> ANALYZE -> CHALLENGE -> DECIDE
        -> SYNTHESIZE -> REVIEW -> ARTIFACT -> QUALITY GATE -> HANDOFF
```

Declare, explicitly: which actions require a human decision, what conditions
stop the run, what the evidence states are (observed, reported, inferred,
assumed, unknown, conflicting), and how confidence is expressed.

Map the modes onto the ten sections `verify.py` requires in `OS.md`, because
that file is graded on exactly these headings: Purpose, Boundary, Operating
modes, Inputs, Outputs, State, Rules and invariants, Failure behaviour, Human
approval boundary, Completion criteria.

**Gate 5.** Every mode has an entry condition, a produced artifact and a
completion test. A mode that cannot say when it is done is a conversation, not
a mode.

---

## Phase 6. Artifact architecture

Define the primary artifact, the secondary artifacts, the schemas that carry
them between systems, the executive view (what a decision maker reads in two
minutes), the registers (assumptions, conflicts, unknowns, decisions), and the
handoff object that the next unit consumes.

Trace every recommendation the unit can make back through finding, evidence and
source, per [`REFERENCES/TRACEABILITY.md`](../REFERENCES/TRACEABILITY.md). Keep
assumptions in their own register, never mixed into findings.

**Gate 6.** The primary artifact has a named owner, a place to live and a
consumer. An artifact nobody consumes is a phase that can be deleted.

---

## Phase 7. Package design and registration

Two halves: choose the components, then claim the slug.

### 7a. Tool selection

Choose only what earns its place.

| The work is | The component is |
|---|---|
| a deterministic calculation | a script or calculator, never a model |
| a strict interchange between systems | a schema |
| a reusable document | a template |
| adaptive questioning or synthesis | an LLM prompt |
| human judgement | a skill section plus an approval gate |
| repeated quality assurance | tests plus a rubric |
| movement across system boundaries | a handoff contract |
| a one-off explanation | nothing, do not overbuild |

Do not create an empty directory to satisfy a diagram. The contract's 23 files
are mandatory; anything beyond them must justify itself.

### 7b. Register the slug in the single source of truth

`OS/_tools/suite.py` holds the `SUITE` tuple. It is the single source of truth
for the whole suite, and `OS/_registry.json`, `OS/README.md` and
`crates/omega-core/src/os_products.rs` are all emitted from it. Never hand add a
unit to a generated file: the next regeneration deletes it and the loss is
silent.

1. Add one row to `SUITE`, appended inside its group block so that group blocks
   stay contiguous:
   `(num, slug, name, group, tagline, maps_from)`.
   The tagline is one sentence ending in a period, with no long dash. The slug
   ends in `-os` and is the directory name.
2. Groups must stay contiguous and in declaration order, and numbers must stay
   a contiguous range starting at zero. Appending to the last group (`systems`)
   is the only insertion that renumbers nothing.
3. `validate()` hard codes the unit count. Adding a unit means updating the
   count guard in the same edit (`len(SUITE) != 73` and `list(range(73))` near
   lines 157 to 160), or `suite.py check` refuses the registry.
4. Emit the registry, then materialise the tree:

```bash
python3 OS/_tools/suite.py check              # must exit 0 before anything else
python3 OS/_tools/suite.py registry           # writes OS/_registry.json
python3 OS/_tools/scaffold.py build <slug>    # creates the 23 files, overwrites nothing
```

`scaffold.py` is additive by construction: it never deletes and never
overwrites an existing file. Running it against a partially authored unit fills
the gaps and leaves authored content alone.

### 7c. The ordering question, answered

`scaffold.py build <slug>` resolves the unit from `OS/_registry.json`, so an
in-tree build has to register before it can scaffold. That is the order above,
and it is correct for a unit authored inside `OS/`.

It is the wrong order for a package authored somewhere else first (an external
rebuild, a candidate assembled outside the tree). Registering an unvalidated
package means a broken unit enters the registry, the generators run over it,
and the suite goes red before anyone has looked at it. For that case, grade the
candidate in place before it goes near the registry:

```bash
python3 OS/os-builder-os/TOOLS/validate_os.py <path>          # core tier
python3 OS/os-builder-os/TOOLS/validate_os.py <path> --full   # all 23 files
```

It imports `verify` and calls `verify.check()` against an arbitrary directory,
so a candidate is graded with the identical STRUCTURE, AUTHORED, MANIFEST,
DEPS, NODASH and SUBSTANCE checks the suite itself uses. Dependency slugs still
resolve against the real registry, so a candidate cannot declare a handoff to
an OS that does not exist. See [`TOOLS/README.md`](../TOOLS/README.md).

**Gate 7.** `suite.py check` exits zero, `OS/_registry.json` contains the new
slug, and `scaffold.py status <slug>` reports the structure complete. The unit
exists as structure, and nothing yet claims it is authored.

---

## Phase 8. Build

Write substantive content into all 23 contract files. No placeholders.

The 23, exactly as `verify.py: CONTRACT_FILES` enumerates them:

| Root | Directories |
|---|---|
| `README.md` | `COMMANDS/README.md` |
| `OS.md` | `MEMORY/policy.md` |
| `SYSTEM.md` | `EVALS/README.md` |
| `SKILL.md` | `WORKFLOWS/README.md` |
| `SETUP.md` | `PROMPTS/README.md` |
| `manifest.json` | `REFERENCES/README.md` |
| `CHANGELOG.md` | `TOOLS/README.md` |
| | `EXAMPLES/README.md` |
| | `INTERFACES/{chat,artifact,dashboard,generative-ui}.md` |
| | `ADAPTERS/{chatgpt,claude,gemini,codex}.md` |

Five of them are wave 1 (`verify.py: CORE_FILES`): `OS.md`, `SKILL.md`,
`manifest.json`, `COMMANDS/README.md`, `WORKFLOWS/README.md`. A unit with those
five authored is usable. The other eighteen are wave 2 surfaces.

Four mechanical constraints, each of which is checked and therefore not
negotiable:

- **Remove every scaffold marker.** A file still containing the marker comment
  that `scaffold.py` writes counts as not authored. In the core tier that is a
  failure at any grading tier; in wave 2 it fails under `--full`.
- **Never write a long dash.** `verify.py` fails on U+2014 and U+2013 anywhere
  in any contract file (R-NODASH). Use a comma, a period, a colon or
  parentheses. The single exemption is the protocol literal listed in
  `PROTOCOL_LITERALS`, which is machine matched and not copy.
- **Never leave "to be authored"** in `OS.md`, `SKILL.md` or
  `COMMANDS/README.md`. It is grepped for by `SUBSTANCE` and it is the exact
  phrase the scaffold templates ship with.
- **Fill the manifest properly.** `commands` must be non-empty. `dependencies`
  must have at least one of `requires`, `consumes`, `emits`, `consumes_from`,
  `emits_to`, `handoffs` populated. `id` must equal the slug and `num` must
  equal the registry number, or the check fails on the mismatch.

The dependency schema is typed, and mixing the types is the error worth
catching. `requires`, `consumes_from` and `emits_to` hold **slugs**.
`consumes` and `emits` hold **dotted event names** matching
`^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$`, in the shape `namespace.thing.verb`.
Putting a slug in `emits` is a reported failure, not a style difference.
`handoffs` holds objects that must carry an `artifact` key, and whose `to`, if
present, must resolve to a real slug.

Build checklist, ticked in order: SYSTEM, SKILL, README, manifest, workflows,
artifacts, prompts, schemas where useful, scripts only where justified,
references, tests, examples, handoffs, adapters, security review, quality
review.

**Gate 8.** `python3 OS/_tools/verify.py <slug>` exits zero on the core tier,
and `python3 OS/_tools/verify.py --full <slug>` names only failures you have
deliberately deferred to wave 2. Nothing is claimed done that the tool did not
confirm. `TOOLS/validate_os.py <path> --json` gives the same verdict in a shape
a gate can read, for a build driven by a script rather than by a person.

---

## Phase 9. Test

Run the case matrix against the unit, as a live session, not as a thought
experiment.

| Case | What it proves |
|---|---|
| Happy path | the unit produces its primary artifact from good input |
| Missing input | it asks or abstains, and does not invent the gap |
| Conflicting input | it preserves the conflict and escalates, and does not average |
| Weak evidence | it lowers confidence visibly, and does not launder a guess |
| Security sensitive | it applies the declared controls and refuses to collect what it does not need |
| Out of scope | it names the correct unit and hands off, and does not improvise that unit's job |
| Adversarial | it holds the boundary under pressure |
| Regression | previously fixed defects have not returned |

Ten standing cases apply to every build regardless of domain: a vague request
(scope before build), the prompt-only temptation (justify the OS), adjacent
duplication (reuse, do not duplicate), fabricated ROI (refuse the guarantee),
missing research (block the release), package inflation (reject hollow files),
a high risk capability (add approval and security controls), conflicting
sources (preserve the conflict), an unnecessary agent (choose the simpler
mechanism), and the release gate itself (block when mandatory quality fails).
They are written out in [`EVALS/TEST-PLAN.md`](../EVALS/TEST-PLAN.md).

**Gate 9.** Every case ran, every result is recorded with the actual behaviour
observed, and no case is marked pass on the strength of the design intent.

---

## Phase 10. Red team

Attack the unit deliberately. Twelve vectors: vague inputs, weak evidence,
executive pressure, fabricated ROI, technology hype, unnecessary agents,
sensitive data, scope violations, conflicting policies, vendor claims, skipped
approval, and the temptation to overclaim.

For each attack record: the attack, the expected safe behaviour, the actual
behaviour, the severity, and the repair.

The specific failures to hunt: overclaiming, invented ROI numbers, AI used
where a script would be correct and cheaper, boundary violations into an
adjacent unit, conclusions unsupported by the evidence register, and an
approval gate quietly skipped under time pressure.

**Gate 10.** Every attack has a recorded actual behaviour. An attack that was
not run is listed as not run. A red team with no findings is a red team that
was not adversarial, and it is rejected rather than celebrated.

---

## Phase 11. Score

Apply the rubric. Sixteen dimensions, each scored 0 to 5:

value proposition, scope, domain depth, human skill, operating logic, evidence
discipline, decision quality, artifact quality, executive usability, security,
testability, traceability, reusability, installability, handoffs, adapters.

| Score | Meaning |
|---|---|
| 5 | exceptional |
| 4 | strong, professional release quality |
| 3 | usable but incomplete |
| 2 | weak |
| 1 | superficial |
| 0 | absent |

The dimension definitions and the anchors for each level live in
[`EVALS/OS-QUALITY-RUBRIC.md`](../EVALS/OS-QUALITY-RUBRIC.md). Every score
carries one sentence of evidence naming the file or the observed behaviour that
justifies it. A score with no evidence is not a score.

The judging is yours; the arithmetic is not. Fill the scorecard and let the
tool apply the threshold, so it cannot be rounded in the retelling:

```bash
python3 OS/os-builder-os/TOOLS/score_os.py --template > scorecard.json
python3 OS/os-builder-os/TOOLS/score_os.py scorecard.json
```

A missing score is reported as malformed, not read as a zero, because treating
an unfilled field as a zero lets an incomplete review pass itself off as a
harsh one.

**Gate 11.** No dimension is below 4, the mean is at least 4.3, and the five
critical dimensions clear 4 with no waiver available: evidence discipline,
operating logic, artifact quality, security, testability. A waiver may lift the
threshold for a non-critical dimension only, and it is recorded. Below any of
these, the unit goes to phase 12 and does not proceed.

---

## Phase 12. Repair

Repair every mandatory dimension scoring below 4, and every red team finding at
high severity or above. Re-score only the dimensions you touched, and re-run
the test cases the repair could have broken.

Repair is bounded. After three repair rounds on the same dimension without it
reaching 4, stop and escalate: the defect is in the capability definition
(phase 1) or the value proposition (phase 2), not in the writing, and grinding
the text will never fix it.

**Gate 12.** Re-scored dimensions clear the threshold, or the unit is returned
to phase 1 with the structural reason named.

---

## Phase 13. Adapters and interfaces

Author the four adapters (`ADAPTERS/claude.md`, `codex.md`, `chatgpt.md`,
`gemini.md`) and the four interfaces (`INTERFACES/chat.md`, `artifact.md`,
`dashboard.md`, `generative-ui.md`).

The operating logic in `OS.md` stays constant across every target. An adapter
records only how it is implemented on that target and, more importantly, what
that target cannot do. State the absence explicitly and name the fallback.
Silently working around a missing capability is how a unit acquires two
different behaviours under one name.

Every interface states what it shows, what the user can do from it, what it
must never do, and what it degrades to when the environment cannot render it.

**Gate 13.** Each adapter names at least one capability the target lacks, or
states positively that it lacks none, and each interface has a declared
degradation path.

---

## Phase 14. Release

Run the gate in [`EVALS/RELEASE-GATE.md`](../EVALS/RELEASE-GATE.md). Every item
must be yes:

bounded capability, specific value proposition, primary artifact defined, human
skill defined, evidence states defined, decision gates defined, stop conditions
defined, handoffs defined, appropriate security controls, tests beyond the
happy path, substantive files, realistic examples, purposeful references, no
unsupported major claims, quality threshold passed, package validated,
changelog updated, reproducible package.

Then close the machine loop, in this order:

```bash
python3 OS/_tools/verify.py --full <slug>     # 23 files, must exit 0
python3 OS/_tools/graph.py                    # the event graph must join up
python3 OS/_tools/normalize.py --check        # dependency schema drift
python3 OS/_tools/gen_os_products.py --write  # the TUI OS-menu roster
python3 OS/_tools/gen_readme.py               # the human index
```

`graph.py` catches what no per-unit check can: an event this unit consumes that
nobody emits, or a near miss where one character separates your consume from
somebody else's emit. Per-unit verification passes on both sides of a severed
boundary, which is exactly why the whole graph is checked separately.

`gen_os_products.py` preserves authored `commands:` arrays in the Rust file: it
parses them out, keys them by slug and re-emits them verbatim, so regenerating
loses nothing hand written. Run it with `--check` first if you want to read the
diff before it lands.

Version the release semantically, per
[`REFERENCES/VERSIONING.md`](../REFERENCES/VERSIONING.md): MAJOR for a breaking
workflow, schema or behaviour change, MINOR for a compatible capability or
asset addition, PATCH for a correction. Write the `CHANGELOG.md` entry in the
same commit, because the Runtime reads that file for `agentik update <slug>`.

The last gate item, the reproducible package, is produced rather than asserted:

```bash
python3 OS/os-builder-os/TOOLS/create_zip.py OS/<slug> --list   # what goes in
python3 OS/os-builder-os/TOOLS/create_zip.py OS/<slug>          # the archive plus its SHA-256
```

If the archive is deliberately not shipped, drop that gate item explicitly and
say so. Never let it pass silently on the strength of the other seventeen.

**Gate 14.** `verify.py --full <slug>` exits zero, `graph.py` reports no orphan
consume for this unit, the generated files are regenerated rather than edited,
and the changelog carries the version.

---

## Completion test

```bash
python3 OS/_tools/suite.py check                 # registry valid
python3 OS/_tools/verify.py --full <slug>        # 23/23 authored, exit 0
python3 OS/_tools/graph.py --strict              # no orphan consume, exit 0
```

And, by inspection: the rubric card exists with sixteen scored dimensions and
their evidence, the red team log exists with an actual behaviour per attack, no
mandatory dimension is below 4, and `OS/README.md` lists the unit because it
was regenerated rather than edited.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the viability tree returns anything but BUILD | stop, deliver the recommended lighter artifact, and record why an OS was refused. This is a completed run. |
| research finds no evidence base for the core logic | do not release. Ship the unit labelled `status: draft` with the unsupported claims marked, or return to phase 1 and narrow the capability to what is supportable. |
| the capability cannot be bounded | split it and re-enter phase 0 once per half. Never build one unit that owns two capabilities: the boundary is what makes the suite composable. |
| an adjacent unit already covers most of this | stop and extend that unit instead. Record the near-duplicate in the intake so the next requester does not re-propose it. |
| `suite.py check` refuses the registry after your edit | fix the registry before touching anything else. A broken registry breaks the emitters for all 73 units, not just yours. |
| `verify.py` fails on a file another writer owns | report it, do not fix it across the boundary. One writer per file. |
| the output limit forces a split mid build | mark the unit INCOMPLETE, list the contract files finished and remaining by name, and resume at the exact next file. Never restart the phase. |
| a repair round hits the phase 12 ceiling | escalate to a human with the failing dimension, the three attempts and the structural hypothesis. Do not release at 3. |
