# Workflow: Fast build

**Mode:** `FAST`
**Produces:** a registered unit whose five wave 1 files are authored and which
passes `python3 OS/_tools/verify.py <slug>` on the core tier, carrying an
explicit record of what it has not yet earned.

## Trigger

The viability tree returned BUILD, the capability is narrow, low risk and
already understood by the requester, and the operator wants it usable today
rather than complete this week.

## What fast is, and what it is not

Fast is a **shorter path to a usable unit**, not a lower standard for the parts
it builds. It reaches wave 1 and stops. It does not release.

```
DEFINE -> VALUE -> SKILL -> WORKFLOW -> ARTIFACTS -> CORE PACKAGE
       -> TEST -> SCORE -> REPAIR -> PACKAGE
```

Against the full pipeline it drops phase 3 (research), phase 10 (red team),
phase 13 (adapters and interfaces authored rather than scaffolded) and phase 14
(release). It keeps intake, capability definition, value, skill, operating
model, artifacts, build, test, score and repair.

## What it trades away, stated plainly

This is the section that keeps fast honest. Every item below is a real loss,
and the unit carries it as a known debt until a full pass repays it.

| Traded away | What that costs | When it bites |
|---|---|---|
| **Research (phase 3)** | the operating logic rests on the builder's own model of the domain, with no source plan, no limitations recorded and no separation of source derived from original synthesis | the first time a competent practitioner reads it and finds the received wisdom is wrong |
| **Red team (phase 10)** | nobody attacked the unit. Overclaiming, fabricated numbers, boundary violations and skipped approvals are unfound rather than absent | under executive pressure, which is exactly when the unit is used |
| **Wave 2 authoring (18 files)** | `SETUP.md`, `SYSTEM.md`, `README.md`, `CHANGELOG.md`, `MEMORY/policy.md`, `EVALS/README.md`, `PROMPTS/`, `REFERENCES/`, `TOOLS/`, `EXAMPLES/`, the four interfaces and the four adapters stay scaffolded | on any target that is not the one you built against, and the first time somebody else has to configure it |
| **Adapters (phase 13)** | no target's missing capabilities are recorded, so the unit silently behaves differently on Claude, Codex, ChatGPT and Gemini | when two people compare outputs and cannot reconcile them |
| **The release gate (phase 14)** | eighteen release conditions are unchecked, the changelog is unwritten and the version is not real | at install time, and at the next update |
| **Full tier verification** | `verify.py --full <slug>` fails by construction, on eighteen `AUTHORED still scaffold` lines | the moment anyone runs the suite-wide verification |

## What fast never waives

Three things are not negotiable at any speed, because waiving them produces a
unit that is wrong rather than a unit that is small.

1. **Evidence discipline.** No fabricated fact, source, number or citation.
   Unsupported claims are labelled unsupported. Absence of information is a
   reportable state, in fast exactly as in full.
2. **Security.** Sensitivity is classified and controls are declared. Nothing
   that touches money, legal rights, health, employment, compliance,
   production systems or regulated data may take this path at all: those go to
   `FULL_BUILD.md`, without exception and without an override flag.
3. **The quality floor on what was built.** The rubric still runs over the
   dimensions fast actually covered, and the same threshold applies to them: no
   dimension below 4, and no waiver on the five critical ones (evidence
   discipline, operating logic, artifact quality, security, testability). Fast
   reduces the surface graded, never the grade.

## Steps

1. **Intake and define.** Phase 0 and phase 1 of the full build, unchanged. The
   viability tree runs in full. Fast does not lower the bar for whether an OS
   should exist, only for how much of it is built today.
2. **Value.** Problem, user, promise, before, after, primary artifact,
   non-goals. Falsifiable, or go back.
3. **Skill.** The human competence: mental models, questions, signals,
   mistakes. Compressed but present, because this is what makes it an OS.
4. **Workflow.** The operating loop cut down to the modes actually needed now.
   Each mode still declares its entry condition, its artifact and its
   completion test.
5. **Artifacts.** The primary artifact and its consumer. Secondary artifacts
   are deferred by default.
6. **Register and scaffold.** Identical to phase 7b of the full build, with no
   shortcut. The slug goes in `OS/_tools/suite.py`, the count guard is updated,
   and the tree is materialised:

```bash
python3 OS/_tools/suite.py check
python3 OS/_tools/suite.py registry
python3 OS/_tools/scaffold.py build <slug>
```

7. **Author the core package.** The five files in `verify.py: CORE_FILES`:
   `OS.md` (all ten graded sections), `SKILL.md`, `manifest.json`,
   `COMMANDS/README.md`, `WORKFLOWS/README.md`. Remove the scaffold marker from
   each. Do not leave "to be authored" in `OS.md`, `SKILL.md` or
   `COMMANDS/README.md`: it is grepped for. Do not write a long dash anywhere.
8. **Declare the debt.** Set `"status": "draft"` in `manifest.json` and list the
   deferred phases in `CHANGELOG.md` under the unreleased version. A fast unit
   that presents itself as finished is worse than no unit, because it stops
   anyone from finishing it.
9. **Test.** The happy path, the missing input case, the out of scope handoff
   and one adversarial case. Four minimum, run live.
10. **Score and repair.** The rubric over the covered dimensions. Repair
    anything below 4, bounded at three rounds, then escalate.
11. **Package.** Stop here. Do not run `gen_os_products.py` or `gen_readme.py`
    claiming a release: regenerate them so the roster is accurate, and say in
    the report that the unit is wave 1 and unreleased.

## Completion test

```bash
python3 OS/_tools/suite.py check              # registry valid, exit 0
python3 OS/_tools/verify.py <slug>            # core tier, exit 0
python3 OS/_tools/verify.py --full <slug>     # expected to FAIL, read the list
```

The third command is part of the test, not a contradiction of it. Its failure
list is the debt register: every `AUTHORED still scaffold` line is one file a
later full pass must author. Copy that list into the report. A fast build whose
full-tier failure list was never read has hidden its own debt.

By inspection: `manifest.json` says `draft`, the changelog names the deferred
phases, and the report names them again so the operator sees the trade rather
than inferring it.

## Promotion to full

A fast unit is promoted, never rebuilt. Re-enter `FULL_BUILD.md` at phase 3
(research), carry the existing wave 1 files forward untouched unless research
contradicts them, and run phases 3 through 14 in order. `scaffold.py` will not
overwrite what you already authored, so the promotion is additive.

Promotion is mandatory before any other unit may declare this one in its
`requires`. A hard dependency on a unit that skipped research and red team
propagates both omissions into the dependent, silently.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the capability turns out to touch money, legal rights, health, employment, compliance, production systems or regulated data | abandon fast immediately and restart under `FULL_BUILD.md`. There is no flag that keeps you on this path. |
| the builder cannot write `OS.md` without inventing domain claims | stop. The absence of research has become load bearing, which is the signal that this was never a fast capability. |
| a mandatory dimension will not reach 4 in three rounds | escalate exactly as in phase 12 of the full build. Fast does not lower the threshold, so it does not lower the escalation either. |
| someone asks to release a fast unit | refuse and name the eighteen unchecked release conditions. Fast produces a usable unit, never a released one. |
