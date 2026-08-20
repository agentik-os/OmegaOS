# The Agentik OS Build Stepper: 120 OS, one method, 18 phases

> **Build program.** How every OS of the universe gets built, one by one, to the
> same standard. The taxonomy lives in [OS-UNIVERSE.md](OS-UNIVERSE.md), the
> anatomy of an integrated OS in [OS-SUITE.md](OS-SUITE.md); this file is the
> ORDER and the METHOD that turn the map into shipped units.
>
> | | |
> |---|---|
> | Status | Build program, v1 |
> | Adopted | 2026-08-18 |
> | Method | **The Forge 20-stage research-first pipeline** (`OS/os-builder-os/WORKFLOWS/ULTIMATE_BUILD.md`). The 18-phase chain below is retained as the governance view, mapped onto it in section 2.0. |
> | Universe | 120 distinct OS from the map, plus 2 registry units the map does not name (122 total) |
> | Registry SSOT | `OS/_tools/suite.py` (73 units today). Never edit a generated file by hand. |
> | Gate command | `python3 OS/_tools/verify.py <slug>` |
> | State below | Measured with `verify.py --summary` on 2026-08-18, not estimated |

---

## 1. Where we actually stand

This is the measured baseline, not a plan's opening assumption. Re-run
`python3 OS/_tools/verify.py --summary` before trusting any number here.

```
units passing CORE      : 73/73
contract files present  : 1679/1679 (100%)
CORE authored (5 files) :  365/365  (100%)
ALL authored (23 files) :  397/1679 (23%)
total failures          : 0
```

Read that honestly. **Every one of the 73 shipped OS passes the CORE tier and
exactly one is actually finished.** `os-builder-os` sits at 23/23. Every other
unit sits at 5/23 or 6/23: the five defining files are authored, the eighteen
surface files are scaffold. The suite is not 73 finished products, it is 73
correct skeletons and one product.

That splits the universe into four tracks, and the tracks are what make this
program finite:

| Track | What it is | Count | Phases it runs |
|-------|-----------|-------|----------------|
| **A** | Shipped at CORE tier, needs completing to full | **70** | Ratify, then 10 to 18 |
| **B** | Named in the map, not scaffolded at all | **49** | 01 to 18, all of them |
| **C** | Registry decisions to settle before any wave starts | **4 calls** | see section 7 |
| **D** | Complete, keep governed | **1** | 16 to 18 only |

The concrete debt of track A is **1282 files** (1679 minus 397). That is the
single largest number in this document and the reason waves exist.

---


## 2.0 One method, not two

**Update, 2026-08-19.** Builder {OS} v4.1.0 landed and it carries its own
canonical pipeline: 20 stages, research first. That pipeline is now **canon for
constructing an OS**, because it is the operator pack and because it fixes the
single largest hole in the 18-phase chain written here on 2026-08-18.

The hole is worth naming precisely, since it is the reason the two are not
equals. The 18-phase chain came from the CAIO transformation method, and it
opens at QUALIFY, moves through domain understanding and prioritisation, and
reaches ARCHITECTURE at phase 10. **Nowhere in it does anyone read anything.**
It has no corpus, no book analysis, no claim ledger, no contradiction map. It
assumes the domain knowledge is already in the room. For transforming a company
that is fair, the knowledge IS in the room. For building an OS from a domain
name it is false, and it is exactly how a model produces a plausible
architecture from a title while missing the foundational schools.

The Forge pipeline inserts stages 03 to 12 between framing and architecture:
research protocol, book discovery, corpus curation, deep analysis, evidence
expansion, claim normalisation, contradiction mapping, synthesis, ontology,
compiled logic. That is the knowledge acquisition half the CAIO chain never had.

What survives from this document, because the Forge pipeline does not cover it:
the measured state of the suite, the four tracks, the eleven waves, the
`verify.py` gates, the four-block step format, and the 120-OS ledger. The Forge
says HOW to build one OS. This document says WHICH ones, in WHAT ORDER, and
WHERE each one stands today.

### The mapping

| 18-phase governance view | Forge stage | Note |
|---|---|---|
| 01 QUALIFY | 00 CONTRACT | Same kill gate |
| 02 UNDERSTAND DOMAIN | 01 FRAME | |
| 03 CAPABILITY MATURITY | 01 FRAME | Box-local: what already ships here |
| 04 PROCESS DISCOVERY | 02 JOBS AND OUTCOMES | |
| 05 OPPORTUNITY MAP | 02 JOBS AND OUTCOMES | |
| 06 PRIORITIZE | 02 JOBS AND OUTCOMES | |
| 07 BUSINESS CASE (G1) | 00 CONTRACT | The GO or NO GO verdict |
| **no equivalent** | **03 RESEARCH PROTOCOL** | **added by v4.1.0** |
| **no equivalent** | **04 DISCOVER BOOKS** | **added, Librarian {OS}** |
| **no equivalent** | **05 CURATE CORPUS** | **added, Librarian {OS}** |
| **no equivalent** | **06 DEEP ANALYZE BOOKS** | **added, Librarian {OS}** |
| **no equivalent** | **07 EXPAND EVIDENCE** | **added, Research {OS}** |
| **no equivalent** | **08 NORMALIZE CLAIMS** | **added** |
| **no equivalent** | **09 MAP CONTRADICTIONS** | **added** |
| **no equivalent** | **10 SYNTHESIZE KNOWLEDGE** | **added** |
| **no equivalent** | **11 BUILD ONTOLOGY** | **added** |
| 08 OS STRATEGY | 12 COMPILE LOGIC | |
| 09 ROADMAP | 13 ARCHITECT SYSTEM | |
| 10 ARCHITECTURE (G2) | 13 ARCHITECT SYSTEM | |
| 11 POV (G3) | 15 DESIGN AND RUN EVALS | Thin slice first |
| 12 IMPLEMENT | 14 IMPLEMENT PACKAGE | |
| 13 EVALUATE (G4) | 15 EVALS plus 16 RED TEAM plus 17 REPAIR | |
| 14 DEPLOY (G5) | 18 DOCUMENT AND RELEASE | |
| 15 ADOPT | 18 DOCUMENT AND RELEASE | |
| 16 MEASURE VALUE | 19 CONTINUOUS UPDATE | |
| 17 GOVERN | 19 CONTINUOUS UPDATE | |
| 18 IMPROVE | 19 CONTINUOUS UPDATE | |

Ten stages have no equivalent in the older chain, and all ten are knowledge
acquisition. That asymmetry IS the upgrade.

### What this changes for the tracks

- **Track B (49 net new)**: runs the Forge pipeline end to end, all 20 stages.
- **Track A (70 to complete)**: already has an authored purpose and boundary, so
  it enters at RATIFY, then runs stages 12 to 19. It does NOT skip the corpus:
  a unit completed without one is a skeleton with surfaces, and the corpus is
  what the surfaces are supposed to contain.
- **Track D**: stage 19 only.

## 2. The unit of work, phase by phase

The chain is the operator's, applied to building an OS rather than transforming
a company. Each phase answers ONE question, produces ONE named artifact, and
passes ONE gate that a command can check.

Phase artifacts live in `OS/<slug>/BUILD/NN-<phase>.md`. **This is the one new
convention this program introduces**, and it is additive: `verify.py` checks that
the contract files are present and does not care about extra directories. The
reason it exists is resumability. A phase trail that lives only in a transcript
is gone at the first compaction, and then the next session re-derives phase 4
from scratch and gets a different answer.

| # | Phase | The question it answers | Artifact |
|---|-------|------------------------|----------|
| 01 | QUALIFY | Does this OS deserve to exist as its own unit? | `BUILD/01-qualify.md` |
| 02 | UNDERSTAND DOMAIN | Whose work is this, in what situation? | `BUILD/02-domain.md` |
| 03 | CAPABILITY MATURITY | What already exists on this box for it? | `BUILD/03-maturity.md` |
| 04 | PROCESS DISCOVERY | What steps does a real expert actually take? | `BUILD/04-process.md` |
| 05 | OPPORTUNITY MAP | Where does an OS add leverage over that? | `BUILD/05-opportunities.md` |
| 06 | PRIORITIZE | Which opportunities make v1? | `BUILD/06-priority.md` |
| 07 | BUSINESS CASE | Is it worth the build? | `BUILD/07-business-case.md` |
| 08 | OS STRATEGY | What is its primitive, its boundary, its refusals? | `BUILD/08-strategy.md` |
| 09 | ROADMAP | What is v1, v2, v3? | `BUILD/09-roadmap.md` |
| 10 | ARCHITECTURE | What exactly is the contract? | `OS.md`, `manifest.json`, `COMMANDS/README.md` |
| 11 | POV | Does it work once, on a real case? | `BUILD/11-pov.md` plus captured output |
| 12 | IMPLEMENT | Build it. | `SKILL.md`, `WORKFLOWS/`, `TOOLS/`, `PROMPTS/`, engine |
| 13 | EVALUATE | How do we know it is good? | `EVALS/` |
| 14 | DEPLOY | Does a fresh clone get it? | `install.sh` parity, registry, TUI |
| 15 | ADOPT | Will it actually get used? | `SETUP.md`, `README.md`, `INTERFACES/`, `ADAPTERS/` |
| 16 | MEASURE VALUE | Did it deliver? | `MEMORY/policy.md`, usage and outcome metrics |
| 17 | GOVERN | What are its limits and who owns them? | `SYSTEM.md`, rules entry |
| 18 | IMPROVE | What is the update loop? | `CHANGELOG.md` |

### 01 QUALIFY

- **Purpose.** Kill or keep, before a single file is written. An OS earns its
  own unit only if it owns a PRIMITIVE no other OS owns.
- **In.** The map entry: name, stack, one-line logic.
- **Out.** A verdict, KEEP or MERGE INTO `<slug>` or KILL, plus the primitive in
  one sentence.
- **Gate.** The primitive sentence does not fit any existing `OS/*/OS.md` Purpose.
- **Verify.** `grep -ril "<primitive keywords>" OS/*/OS.md` returns nothing that
  already covers it.
- **Anti-pattern.** Keeping an OS because it appears in the map. The map is the
  target, not a permission slip. Two of the calls in section 7 exist precisely
  because this phase was skipped once.

### 02 UNDERSTAND DOMAIN

- **Purpose.** Name the human whose work this replaces or amplifies, and the
  situation they are in when they reach for it.
- **In.** Phase 01 verdict.
- **Out.** The user, the trigger situation, the domain vocabulary, the expert
  being modelled, and the failure they suffer today.
- **Gate.** A concrete trigger sentence in the operator's own words exists.
- **Anti-pattern.** A persona. Write the situation, not a demographic.

### 03 CAPABILITY MATURITY

- **Purpose.** Baseline what the box ALREADY has, so the build extends instead
  of duplicating.
- **In.** The domain from 02.
- **Out.** An inventory: existing skills, rules, OS, data, CLIs that touch it,
  plus a maturity level from 0 (nothing) to 4 (already solved elsewhere).
- **Gate.** `omega-skills --rag "<the need in plain words>"` was actually run and
  its top matches are recorded with a keep or drop decision on each.
- **Anti-pattern.** Building an OS over a skill that already does the job.
  R-SKILL-ATLAS exists for this phase.

### 04 PROCESS DISCOVERY

- **Purpose.** Write the real process a competent human runs today, step by step,
  before deciding what to automate.
- **In.** 02 and 03.
- **Out.** A numbered flow with, per step, the input, the judgement, the output,
  and how long it takes.
- **Gate.** Every step has a named output. A step with no output is a belief.
- **Anti-pattern.** Documenting the process you WISH existed. Discover, then
  design, never both at once.

### 05 OPPORTUNITY MAP

- **Purpose.** Locate where an OS beats the human process: speed, memory,
  breadth, consistency, or judgement under load.
- **In.** The flow from 04.
- **Out.** One opportunity per step worth changing, each tagged automate,
  augment, or leave alone.
- **Gate.** At least one opportunity is tagged LEAVE ALONE. An OS that claims to
  improve every step has not looked.
- **Anti-pattern.** Confusing a feature with an opportunity.

### 06 PRIORITIZE

- **Purpose.** Cut v1 down to what earns its place.
- **In.** The opportunities from 05.
- **Out.** A RICE or ICE score per opportunity and an explicit v1 cut line.
- **Gate.** Something is BELOW the line and named. A prioritization that keeps
  everything is a list.
- **Anti-pattern.** Scoring after the scope was already decided.

### 07 BUSINESS CASE .. GATE 1

- **Purpose.** State the value, the cost and the payback before building.
- **In.** The v1 cut from 06.
- **Out.** Who pays (operator time, client fee, product revenue), what it saves
  or earns, what it costs to build and to run.
- **Gate 1 . DESERVES TO EXIST.** A written verdict GO or NO GO. NO GO closes
  the dossier with the reason and the OS is marked KILLED in the ledger, not
  quietly forgotten.
- **Anti-pattern.** A business case written to justify a decision already made.

### 08 OS STRATEGY

- **Purpose.** Position the unit inside the universe.
- **In.** Everything above.
- **Out.** The primitive, the boundary, what it REFUSES to do, which OS it hands
  off to, which capabilities it exposes and consumes.
- **Gate.** The refusals are non-empty and each names the OS that owns that work
  instead.
- **Anti-pattern.** An OS with no boundary. That is not an OS, that is a chatbot.
  This phase feeds the Purpose and Boundary sections of `OS.md` verbatim.

### 09 ROADMAP

- **Purpose.** Sequence v1, v2, v3 so v1 can ship alone.
- **Out.** Three scopes, each independently shippable, with the v1 done criteria
  copied from 06.
- **Gate.** v1 is useful with v2 and v3 never built. This is the fundamental rule
  of the universe applied inside a single unit.
- **Anti-pattern.** A v1 that is a foundation for v2 and useless by itself.

### 10 ARCHITECTURE .. GATE 2

- **Purpose.** Write the contract.
- **Out.** `OS.md` with its ten mandatory sections (Purpose, Boundary, Operating
  modes, Inputs, Outputs, State, Rules and invariants, Failure behaviour, Human
  approval boundary, Completion criteria), a valid `manifest.json` with commands
  and dependencies filled, and `COMMANDS/README.md`.
- **Gate 2 . CONTRACT COMPLETE.** `python3 OS/_tools/verify.py <slug>` passes
  STRUCTURE, MANIFEST, DEPS and SUBSTANCE. Every declared dependency resolves to
  a real slug.
- **Anti-pattern.** Writing `SKILL.md` first. The skill is the interface; the
  contract is the product. Interface before contract is how two OS end up owning
  the same primitive.

### 11 POV .. GATE 3

- **Purpose.** Prove the thing works ONCE, on a real case, before the full build.
- **Out.** One real case run end to end, with captured runtime output.
- **Gate 3 . PROVEN.** Captured output exists in `BUILD/11-pov.md`. Not a
  description of a run. The run.
- **Anti-pattern.** Skipping to IMPLEMENT because the design feels obvious. L1:
  runtime is the only truth, and a POV is the cheapest possible runtime.

### 12 IMPLEMENT

- **Purpose.** Build it, in one voice.
- **Out.** `SKILL.md`, `WORKFLOWS/`, `PROMPTS/`, `TOOLS/`, `REFERENCES/`,
  `EXAMPLES/`, plus `bin/` and an engine when the OS has runtime code.
- **Gate.** No file carries the scaffold marker; `verify.py` AUTHORED passes.
- **Anti-pattern.** Parallel sub-agents writing blocks of the same OS. One
  author, in sequence, holding the whole unit in mind. Split authorship
  fractures the voice and the anti-pattern ends up contradicting the purpose.

### 13 EVALUATE .. GATE 4

- **Purpose.** Decide what good means before shipping decides for you.
- **Out.** `EVALS/` with golden cases, graders, a failure taxonomy, and the
  score the OS must beat.
- **Gate 4 . QUALITY FLOOR.** The eval suite runs and the OS clears its own
  stated floor. A floor invented after seeing the score is not a floor.
- **Anti-pattern.** Grading on vibes. R-RUBRIC: the criteria are written before
  the work is judged.

### 14 DEPLOY .. GATE 5

- **Purpose.** Make a fresh clone reproduce it.
- **Out.** Registry entry via `suite.py` and its generated files, `install.sh`
  parity, the installed copy under `~/.omega/os/<slug>/`, the TUI OS tab entry.
- **Gate 5 . REPRODUCIBLE.** `./scripts/verify-install.sh` passes and the TUI
  shows the OS integrated, captured. Then committed and pushed (L0).
- **Anti-pattern.** Editing `os_products.rs` or `OS/README.md` by hand. Both are
  generated. Add the unit in `suite.py` and re-run the generators.

### 15 ADOPT

- **Purpose.** Make it reachable and actually reached.
- **Out.** `SETUP.md`, `README.md`, the four `INTERFACES/` and the four
  `ADAPTERS/`, the command wiring, and the operator's first real run.
- **Gate.** The operator ran it once on real work and the run is recorded.
- **Anti-pattern.** Counting a shipped OS as an adopted one. Fourteen phases of
  work die here more often than anywhere else.

### 16 MEASURE VALUE

- **Purpose.** Check the phase 07 promise against reality.
- **Out.** `MEMORY/policy.md` plus the usage and outcome numbers the business
  case predicted.
- **Gate.** The measured number sits next to the predicted one, including when
  it is worse.
- **Anti-pattern.** Measuring usage only. Usage is not value.

### 17 GOVERN

- **Purpose.** Set the limits and name the owner.
- **Out.** `SYSTEM.md`, the human approval boundary enforced, a rules entry when
  the OS can act irreversibly, a review cadence.
- **Gate.** The human approval boundary in `OS.md` is enforced somewhere real,
  not merely declared.
- **Anti-pattern.** Governance as a document nobody executes.

### 18 IMPROVE

- **Purpose.** Close the loop.
- **Out.** `CHANGELOG.md` and the next cycle's entry point, fed by 16.
- **Gate.** The next revision is entered at the earliest phase the evidence
  invalidates, not always at 12. Evidence that kills the business case sends the
  OS back to 07, and that is a legitimate outcome.
- **Anti-pattern.** An improvement loop that only ever adds features.

---

## 3. The five hard gates

Everything above compresses to five kill points. Passing a gate is a recorded
event, not a feeling.

| Gate | After phase | It asserts | Checked by |
|------|------------|-----------|-----------|
| **G1** | 07 Business case | This OS deserves to exist | Written GO or NO GO verdict |
| **G2** | 10 Architecture | The contract is complete | `verify.py <slug>` STRUCTURE, MANIFEST, DEPS, SUBSTANCE |
| **G3** | 11 POV | It works once, for real | Captured runtime output |
| **G4** | 13 Evaluate | It clears its own quality floor | The eval suite run |
| **G5** | 14 Deploy | A fresh clone reproduces it | `verify-install.sh` plus TUI capture, pushed |

A gate that fails sends the OS BACK to the phase that produced the flaw. It never
converts into a note promising to fix it later.

---

## 4. The step format

Every step handed to an agent or worked by hand carries four blocks. This is the
house format and it is not decoration: a step whose four blocks are not filled is
RED and blocking, and that is precisely what stops an agent being launched at
something vague and returning 900 unusable lines.

1. **Objective.** What this step produces, in one sentence.
2. **Constraints.** The rules that bind it, named (R-NODASH, R-SCOPE, the pack is
   never edited, and so on).
3. **Definition of done, mechanically verifiable.** A command and its expected
   output. `verify.py <slug>` passing. A file existing. A test going green. Never
   "the document reads well".
4. **Do not touch.** The files and areas this step must leave alone. This is the
   block everyone forgets and the one that prevents the sprawling diff.

---

## 5. Where each track enters the chain

The 18 phases are the full path. Only track B walks all of it.

- **Track B, 49 OS, from zero.** Phases 01 to 18, in order, gates G1 to G5.
  Roughly 23 files plus a `BUILD/` dossier per unit.
- **Track A, 70 OS, at CORE tier.** These already assert a purpose and a boundary
  in an authored `OS.md`, so phases 01 to 09 collapse into a single **RATIFY**
  step: read the existing `OS.md`, confirm the primitive and the boundary still
  hold against the map, correct them or send the unit to phase 01 if they do not.
  Then run 10 to 18 in full. The real work is phases 12, 13 and 15, because the
  eighteen unauthored files per unit are surfaces. **1282 files total.**
- **Track D, 1 OS.** `os-builder-os` is at 23/23. It runs 16 to 18 only, on the
  review cadence, and it is the reference implementation every other unit is
  read against.

A track A unit that fails RATIFY is not a track A unit. It drops to track B and
the ledger is corrected, which is the honest outcome and cheaper than completing
eighteen surfaces around a wrong primitive.

---

## 6. The build order

Eleven waves, dependency first. The rule is simple: **nothing is built before the
thing that builds it.** Wave 0 is the machine, waves 1 and 2 are the substrate
every OS runs on, wave 3 is the production line that constructs the rest, and the
verticals follow by leverage. CAIO is last, not because it matters least but
because it is the largest net-new block and it consumes the whole machine.

Within a wave, net new units are listed after the ones being completed, because a
completed unit teaches the pattern the new one copies.

### Wave 0 . The machine that builds the machine  (2 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Agentik Runtime {OS} | `agentik-runtime` | 00 | core 5/5, 5/23 files | A . complete to full |
| 2 | OS Builder / Forge {OS} | `os-builder-os` | 00 | 23/23 complete | **D . done, keep governed** |

### Wave 1 . Core substrate  (9 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | AI Logic {OS} | `ai-logic-os` | 00 | core 5/5, 6/23 files | A . complete to full |
| 2 | Agent {OS} | `agent-os` | 00 | core 5/5, 5/23 files | A . complete to full |
| 3 | Context & Memory {OS} | `context-memory-os` | 03 | core 5/5, 6/23 files | A . complete to full |
| 4 | Knowledge {OS} | `knowledge-os` | 00, 03 | core 5/5, 5/23 files | A . complete to full |
| 5 | Orchestration {OS} | `orchestration-os` | 00 | core 5/5, 5/23 files | A . complete to full |
| 6 | Context {OS} | . | 00 | not scaffolded | **B . build from zero** |
| 7 | Harness {OS} | . | 00 | not scaffolded | **B . build from zero** |
| 8 | Memory {OS} | . | 00 | not scaffolded | **B . build from zero** |
| 9 | Prompt {OS} | . | 00 | not scaffolded | **B . build from zero** |

### Wave 2 . Core quality and governance  (5 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Automation {OS} | `automation-os` | 00 | core 5/5, 5/23 files | A . complete to full |
| 2 | Documentation {OS} | `documentation-os` | 00, 05, 08 | core 5/5, 5/23 files | A . complete to full |
| 3 | Quality & Evaluation {OS} | `quality-evaluation-os` | 00, 05 | core 5/5, 5/23 files | A . complete to full |
| 4 | Review & Governance {OS} | `review-governance-os` | 00, 08 | core 5/5, 6/23 files | A . complete to full |
| 5 | Audit {OS} | . | 00 | not scaffolded | **B . build from zero** |

### Wave 3 . The Build stack (the production line)  (8 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Blueprint {OS} | `blueprint-os` | 05 | core 5/5, 6/23 files | A . complete to full |
| 2 | Builder {OS} | `builder-os` | 05 | core 5/5, 6/23 files | A . complete to full |
| 3 | Design {OS} | `design-os` | 05 | core 5/5, 5/23 files | A . complete to full |
| 4 | Prototype {OS} | `prototype-os` | 05 | core 5/5, 5/23 files | A . complete to full |
| 5 | Release {OS} | `release-os` | 05 | core 5/5, 5/23 files | A . complete to full |
| 6 | Research {OS} | `research-os` | 03, 04, 05 | core 5/5, 5/23 files | A . complete to full |
| 7 | Security {OS} | `security-os` | 05 | core 5/5, 5/23 files | A . complete to full |
| 8 | Stepper {OS} | `stepper-os` | 05 | core 5/5, 6/23 files | A . complete to full |

### Wave 4 . Discover and Strategy  (8 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Brainstorm {OS} | `brainstorm-os` | 04 | core 5/5, 6/23 files | A . complete to full |
| 2 | Business Model {OS} | `business-model-os` | 04 | core 5/5, 5/23 files | A . complete to full |
| 3 | Customer Discovery {OS} | `customer-discovery-os` | 04 | core 5/5, 5/23 files | A . complete to full |
| 4 | Decision {OS} | `decision-os` | 01, 04 | core 5/5, 5/23 files | A . complete to full |
| 5 | Market Research {OS} | `market-research-os` | 04 | core 5/5, 5/23 files | A . complete to full |
| 6 | Strategy & Portfolio {OS} | `strategy-portfolio-os` | 04 | core 5/5, 6/23 files | A . complete to full |
| 7 | Trend & Opportunity {OS} | `trend-opportunity-os` | 04 | core 5/5, 5/23 files | A . complete to full |
| 8 | Validation {OS} | `validation-os` | 04 | core 5/5, 5/23 files | A . complete to full |

### Wave 5 . Personal Evolution  (11 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Alignment {OS} | `alignment-os` | 01 | core 5/5, 6/23 files | A . complete to full |
| 2 | Execution {OS} | `execution-os` | 01, 08 | core 5/5, 6/23 files | A . complete to full |
| 3 | Goal & Life Strategy {OS} | `goal-life-strategy-os` | 01 | core 5/5, 5/23 files | A . complete to full |
| 4 | Habit Tracker {OS} | `habit-tracker-os` | 01 | core 5/5, 5/23 files | A . complete to full |
| 5 | Health & Energy {OS} | `health-energy-os` | 01 | core 5/5, 6/23 files | A . complete to full |
| 6 | Identity Shift {OS} | `identity-shift-os` | 01 | core 5/5, 5/23 files | A . complete to full |
| 7 | Intuitive {OS} | `intuitive-os` | 01 | core 5/5, 5/23 files | A . complete to full |
| 8 | Journal {OS} | `journal-os` | 01 | core 5/5, 5/23 files | A . complete to full |
| 9 | Mindset {OS} | `mindset-os` | 01 | core 5/5, 6/23 files | A . complete to full |
| 10 | Mentor {OS} | . | 01 | not scaffolded | **B . build from zero** |
| 11 | Routine {OS} | . | 01 | not scaffolded | **B . build from zero** |

### Wave 6 . Social and Learn  (9 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Librarian {OS} | `librarian-os` | 03 | core 5/5, 5/23 files | A . complete to full |
| 2 | Relationship & Network {OS} | `network-os` | 02, 07 | core 5/5, 5/23 files | A . complete to full |
| 3 | Social Intelligence {OS} | `social-intelligence-os` | 02 | core 5/5, 5/23 files | A . complete to full |
| 4 | Attraction / Seductive {OS} | . | 02 | not scaffolded | **B . build from zero** |
| 5 | Books {OS} | . | 03 | not scaffolded | **B . build from zero** |
| 6 | Communication {OS} | . | 02 | not scaffolded | **B . build from zero** |
| 7 | Conversation {OS} | . | 02 | not scaffolded | **B . build from zero** |
| 8 | Interest Media {OS} | . | 03, 06 | not scaffolded | **B . build from zero** |
| 9 | Relationship {OS} | . | 02 | not scaffolded | **B . build from zero** |

### Wave 7 . Content and Commercial  (13 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Affiliate {OS} | `affiliate-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 2 | Brand {OS} | `brand-os` | 06 | core 5/5, 5/23 files | A . complete to full |
| 3 | Content {OS} | `content-os` | 06 | core 5/5, 6/23 files | A . complete to full |
| 4 | Delivery & Customer Success {OS} | `delivery-cs-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 5 | Growth {OS} | `growth-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 6 | Offer {OS} | `offer-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 7 | Positioning {OS} | `positioning-os` | 06, 07 | core 5/5, 5/23 files | A . complete to full |
| 8 | Pricing {OS} | `pricing-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 9 | Revenue {OS} | `revenue-os` | 07, 09 | core 5/5, 6/23 files | A . complete to full |
| 10 | Sales {OS} | `sales-os` | 07 | core 5/5, 5/23 files | A . complete to full |
| 11 | Storyteller {OS} | `storyteller-os` | 06 | core 5/5, 5/23 files | A . complete to full |
| 12 | AGK-Market {OS} | . | 06 | not scaffolded | **B . build from zero** |
| 13 | Viral {OS} | . | 06 | not scaffolded | **B . build from zero** |

### Wave 8 . Operator and Business  (8 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Client {OS} | `client-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 2 | KPI & Analytics {OS} | `kpi-analytics-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 3 | Meeting {OS} | `meeting-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 4 | Operations & Automation {OS} | `operations-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 5 | Process & SOP {OS} | `process-sop-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 6 | Project {OS} | `project-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 7 | Team & Delegation {OS} | `team-delegation-os` | 08 | core 5/5, 5/23 files | A . complete to full |
| 8 | Operator {OS} | . | 08 | not scaffolded | **B . build from zero** |

### Wave 9 . Wealth and Capital  (15 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | Acquisition {OS} | `acquisition-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 2 | Board {OS} | `board-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 3 | Business Strategy {OS} | `business-strategy-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 4 | Capital {OS} | `capital-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 5 | Deal Flow {OS} | `deal-flow-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 6 | Deal Structuring {OS} | `deal-structuring-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 7 | Due Diligence {OS} | `due-diligence-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 8 | Exit & Liquidity {OS} | `exit-liquidity-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 9 | IP & Asset {OS} | `ip-asset-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 10 | Investment Thesis {OS} | `investment-thesis-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 11 | Money {OS} | `money-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 12 | Ownership {OS} | `ownership-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 13 | Portfolio Management {OS} | `portfolio-management-os` | 10 | core 5/5, 5/23 files | A . complete to full |
| 14 | Wealth {OS} | `wealth-os` | 09 | core 5/5, 5/23 files | A . complete to full |
| 15 | 0TO100M {OS} | . | 09 | not scaffolded | **B . build from zero** |

### Wave 10 . CAIO Professional Suite  (32 OS)

| # | OS | Slug | Stacks | Today | Track |
|---|----|------|--------|-------|-------|
| 1 | AI Adoption {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 2 | AI Architecture {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 3 | AI Change Management {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 4 | AI Evaluation {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 5 | AI Governance Board {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 6 | AI Implementation {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 7 | AI KPI & Value Realization {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 8 | AI Maturity {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 9 | AI Operations {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 10 | AI Opportunity Mapping {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 11 | AI Portfolio {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 12 | AI Portfolio Review {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 13 | AI Risk & Incident {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 14 | AI Strategy {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 15 | AI Training & Capability {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 16 | Business Case & ROI {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 17 | CAIO Case Study {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 18 | CAIO Client Delivery {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 19 | CAIO Offer {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 20 | CAIO Positioning {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 21 | CAIO Roadmap {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 22 | CAIO Role {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 23 | CAIO Sales {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 24 | Client Qualification {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 25 | Company Context {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 26 | Data & Knowledge {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 27 | Executive & Board Communication {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 28 | Organization Intelligence {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 29 | Process Discovery {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 30 | Production Readiness {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 31 | Proof of Value {OS} | . | 11 | not scaffolded | **B . build from zero** |
| 32 | Security & AI Governance {OS} | . | 11 | not scaffolded | **B . build from zero** |

<!-- distinct OS: 120 -->

---

## 7. Track C: four calls to settle before wave 1

These are map versus registry conflicts. Each one blocks a wave, each has a
recommended default, and none should be resolved silently by whoever gets there
first.

**C1. Context and Memory: one unit or two?** The map splits `Context {OS}`
(retrieve, filter, rank, compress, inject, verify) from `Memory {OS}` (observe,
qualify, store, retrieve, update, forget). The registry ships one merged
`context-memory-os`. *Recommended: split.* The two have genuinely different
primitives, and forgetting is not a compression strategy. Cost: one registry
migration in `suite.py` and a redistribution of one authored `OS.md`.

**C2. `evaluation-os` versus `Quality & Evaluation {OS}`.** Both exist in the
registry; the map names only the latter. *Recommended: merge into
`quality-evaluation-os`* and retire `evaluation-os`, unless phase 01 finds a
primitive that only the standalone owns.

**C3. `tool-integration-os` is absent from the map.** It ships and the map does
not name it. *Recommended: add it to the Core stack.* Tool and integration
binding is substrate; leaving it unnamed means the map is wrong, not the unit.

**C4. The 32 CAIO units.** They ship today as the `caio-*` SKILL suite, not as
registry OS. Promoting all 32 is the single largest item in this program.
*Recommended: qualify all 32 through phase 01 as one batch BEFORE scaffolding
any of them,* and expect the count to fall. Several look like phases of one
engagement rather than independent units, and the fundamental rule is that each
OS must work alone.

---

## 8. Running it

```bash
cd ~/Station/SideBusiness/OmegaOS

# state of the whole suite, or one unit verbose
python3 OS/_tools/verify.py --summary
python3 OS/_tools/verify.py <slug>

# add or reorder a unit: edit the SUITE tuple, then regenerate. Never edit
# _registry.json, os_products.rs or OS/README.md by hand.
$EDITOR OS/_tools/suite.py
python3 OS/_tools/suite.py

# scaffold the directory tree for a new unit
python3 OS/_tools/scaffold.py <slug>

# install parity before calling a wave done
./scripts/verify-install.sh
```

The existing prose processes in [OS-SUITE.md](OS-SUITE.md) stay authoritative for
the two special cases they cover: **add this OS** when an operator pack lands in
`~/Deposit/`, and **complete this OS** when a payload already exists. This
stepper is the general path; those two are the intake ramps onto it.

---

## Appendix A: the 23-file contract

The five CORE files that define an OS, and the eighteen surfaces that complete it.

**CORE, graded as tier 1.** `OS.md`, `SKILL.md`, `manifest.json`,
`COMMANDS/README.md`, `WORKFLOWS/README.md`.

**Surfaces, graded as tier 2.** `README.md`, `SYSTEM.md`, `SETUP.md`,
`CHANGELOG.md`, `MEMORY/policy.md`, `EVALS/README.md`, `PROMPTS/README.md`,
`REFERENCES/README.md`, `TOOLS/README.md`, `EXAMPLES/README.md`,
`INTERFACES/{chat,artifact,dashboard,generative-ui}.md`,
`ADAPTERS/{chatgpt,claude,gemini,codex}.md`.

## Appendix B: the ten mandatory sections of OS.md

Purpose, Boundary, Operating modes, Inputs, Outputs, State, Rules and invariants,
Failure behaviour, Human approval boundary, Completion criteria.

`verify.py` SUBSTANCE fails the unit if any section is missing or still says it
is to be authored.

## Appendix C: what verify.py checks

STRUCTURE (every contract file and directory present), AUTHORED (no scaffold
marker left), MANIFEST (valid JSON, required keys, commands and dependencies
filled), DEPS (every declared dependency resolves to a real slug), NODASH (no em
or en dash anywhere, R-NODASH), SUBSTANCE (the ten sections, nothing left to be
authored). Exit 0 only when every unit passes every check.
