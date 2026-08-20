# Workflow: Repair an existing OS

**Mode:** `REPAIR`
**Produces:** the same slug, repaired in place, passing the grading tier it
claims, with a changelog entry naming what changed and why.

## Trigger

One of four situations, all of which mean a unit exists and is wrong rather
than absent:

1. `python3 OS/_tools/verify.py <slug>` reports failures.
2. The unit was scaffolded and never authored, so its files still carry the
   scaffold marker comment and its `OS.md` still says "to be authored".
3. The unit drifted: the contract grew, the dependency schema was normalised,
   or the registry renamed something underneath it.
4. An external package (an older `{Name}_OS/` tree, an academy capability, a
   hand written workflow) is being rebuilt as a suite unit.

## Preconditions

- The slug is already in the `SUITE` tuple of `OS/_tools/suite.py`. If it is
  not, this is a build, not a repair: go to `FULL_BUILD.md` phase 7b.
- You own the files you are about to touch. Verification failures on files
  owned by another writer are reported, never fixed across the boundary.
- The current failure list has been captured before any edit, so the repair can
  be measured rather than asserted.

## Steps

1. **Capture the baseline.** Both tiers, before touching anything:

```bash
python3 OS/_tools/verify.py <slug>            # core tier
python3 OS/_tools/verify.py --full <slug>     # all 23 contract files
```

   Save both failure lists. This is the repair's definition of done, written
   before the work, which is the only order in which it means anything.

2. **Classify every failure by kind.** The checker emits five, and each has a
   different repair. Do not treat them as one pile.

| Kind | What it means | Repair |
|---|---|---|
| `STRUCTURE missing <rel>` | a contract file is absent | `python3 OS/_tools/scaffold.py build <slug>` creates it without overwriting anything, then author it |
| `AUTHORED still scaffold: <rel>` | the file exists but is a template | author it and delete the marker line |
| `NODASH long dash in <rel>` | U+2014 or U+2013 in a contract file | replace with a comma, a period, a colon or parentheses. Never with a hyphen where a hyphen changes the reading |
| `MANIFEST ...` / `DEPS ...` | the manifest is malformed, empty or type confused | see step 4 |
| `SUBSTANCE ...` | a required `OS.md` heading is missing, or "to be authored" survives | see step 5 |

3. **Restore structure first.** `scaffold.py` is additive by construction: it
   never deletes and never overwrites an existing file, so it is safe to run
   over a partially authored unit. Run it before authoring, so that you author
   into a complete tree rather than discovering a missing file at the end.

4. **Repair the manifest against its real schema.** `commands` must be
   non-empty. At least one of `requires`, `consumes`, `emits`, `consumes_from`,
   `emits_to`, `handoffs` must be populated. `id` must equal the slug and `num`
   must equal the registry number. The types are distinct and mixing them is
   the failure worth catching: `requires`, `consumes_from` and `emits_to` hold
   **slugs**; `consumes` and `emits` hold **dotted event names**
   (`namespace.thing.verb`); `handoffs` holds objects carrying an `artifact`
   key. If the drift is schema shape rather than content, let the normaliser do
   it and read the diff first:

```bash
python3 OS/_tools/normalize.py --check <slug>   # report what would change
python3 OS/_tools/normalize.py --write <slug>   # apply
```

   The normaliser invents nothing and drops nothing. It moves a slug found in
   `consumes` to `consumes_from`, because it was always a who and never a what.

5. **Repair substance.** `OS.md` is graded on ten headings, matched as level
   two markdown headings with an optional leading number: Purpose, Boundary,
   Operating modes, Inputs, Outputs, State, Rules and invariants, Failure
   behaviour, Human approval boundary, Completion criteria. A missing heading
   is a missing section, not a formatting slip: write the section, then the
   heading. Then remove every occurrence of "to be authored" from `OS.md`,
   `SKILL.md` and `COMMANDS/README.md`, and make sure you removed it by
   answering the question it was standing in for.

6. **Re-verify after each class of repair, not once at the end.** A repair that
   introduces a new failure while fixing an old one is common, and finding it
   at the end costs a full re-read.

7. **Re-score what the repair touched.** Run the rubric dimensions affected by
   the change. A structural repair usually moves installability and
   testability; a substance repair usually moves operating logic and evidence
   discipline. Do not re-score untouched dimensions and do not carry a stale
   score forward as if it were fresh.

8. **Re-run the test cases the repair could have broken.** At minimum: the
   happy path, the out of scope handoff, and any case whose behaviour the
   changed file describes.

9. **Record the repair.** A `CHANGELOG.md` entry under a PATCH version for a
   correction, MINOR for a compatible addition, MAJOR for a breaking workflow,
   schema or behaviour change. Name what failed, not just what changed: the
   next repairer reads this entry to find out whether the defect is structural.

10. **Regenerate the emitted surfaces** if the repair touched the registry, the
    manifest or the unit's existence:

```bash
python3 OS/_tools/gen_os_products.py --check    # read the diff first
python3 OS/_tools/gen_os_products.py --write    # the TUI OS-menu roster
python3 OS/_tools/gen_readme.py                 # the human index
```

## Completion test

```bash
python3 OS/_tools/verify.py <slug>            # core tier, exit 0
python3 OS/_tools/verify.py --full <slug>     # full tier, exit 0 if the unit claims release
python3 OS/_tools/graph.py --strict           # no orphan consume introduced
```

And, by comparison: every line of the captured baseline failure list is either
gone or explicitly deferred with a reason. A repair that fixed nine of eleven
failures and reported success is the failure mode this test exists to catch.

## Rebuilding an external package

When the input is an older `{Name}_OS/` tree rather than a suite unit, the
repair is a translation, and the mapping is fixed:

| External | Suite |
|---|---|
| `SYSTEM.md`, `SKILL.md`, `README.md`, `manifest.json`, `CHANGELOG.md` | the same five root files, reshaped to the contract |
| `MISSION.md`, `modes/` | fold into `OS.md` sections 1, 2 and 3 |
| `workflows/` | `WORKFLOWS/`, one file per workflow, each with a completion test |
| `prompts/` | `PROMPTS/`, one file per unit, each with an input contract and an output shape |
| `references/`, `frameworks/`, `learning/` | `REFERENCES/` |
| `evaluation/`, `tests/`, `QUALITY.md` | `EVALS/` |
| `examples/`, `outputs/` | `EXAMPLES/` |
| `schemas/`, `templates/`, `scripts/`, `assets/` | `TOOLS/`, or inline into the prompt that consumes them |
| `checklists/`, `decision-trees/`, `traceability/` | the workflow phase that runs them |
| `handoffs/` | `manifest.json: dependencies`, as typed slugs and events |
| `adapters/claude/`, `adapters/codex/`, `adapters/generic/` | `ADAPTERS/claude.md`, `codex.md`, and `generic` splits into `chatgpt.md` plus `gemini.md` |
| `SECURITY.md`, `VERSIONING.md`, `PACKAGE_STANDARD.md` | `OS.md` sections 7 and 9, plus `CHANGELOG.md` |

Two directories in the external standard have no suite equivalent and are
deliberately dropped rather than translated: empty folders kept for symmetry,
and any folder whose content decorates the package without supporting a
decision. If a translation produces a file with nothing in it, that is the
answer, not a gap to fill.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the failure is in a file another writer owns | report it with the exact verify line, do not edit. One writer per file, always |
| the slug is not in `SUITE` | stop, this is a build. Go to `FULL_BUILD.md` phase 7b and register it first |
| repairing the manifest would change the unit's meaning | stop and escalate. A manifest is a declaration, and quietly rewriting a declaration to pass a check is fabrication with extra steps |
| the unit fails because the contract itself changed | repair the unit, then check whether the other 72 units fail the same way. A contract change that broke one unit usually broke many, and fixing them one at a time as they are noticed is how drift becomes permanent |
| the baseline could not be captured because `_registry.json` is invalid | fix the registry first with `suite.py check`, then start over. Every check reads it |
