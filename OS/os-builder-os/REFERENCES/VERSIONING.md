# Versioning

Semantic versioning, applied to an OS rather than to a library. The question is
never "how big was the diff", it is "what breaks for someone who already
depends on this".

## The three moves

| Bump | When | Examples |
|---|---|---|
| `MAJOR` | a breaking change to workflow, schema, behaviour or contract | a mode is removed or renamed; an emitted event is renamed; a required manifest key changes shape; an artifact's structure changes so a downstream OS cannot parse it; an approval gate is removed; the boundary shrinks |
| `MINOR` | a compatible capability or asset addition | a new mode; a new command; a new emitted event; a new reference; a new eval suite; a new adapter; the boundary widens without moving anything already inside it |
| `PATCH` | a correction or a non breaking improvement | a fixed typo in a prompt; a clarified invariant; a repaired rubric dimension that changes no interface; a tightened test; a corrected citation |

Version `0.x.y` means the boundary is not yet stable and a `MINOR` may break.
The first release that another OS is allowed to depend on is `1.0.0`, and
reaching it requires passing the full release gate, not merely feeling finished.

## What counts as breaking, specifically

These are the changes that look small and are not:

- **Renaming an emitted event.** Every consumer's `consumes` array stops
  joining, and `OS/_tools/graph.py` reports an orphan consume on the *other*
  unit, not on yours. MAJOR.
- **Narrowing an output shape.** Removing a field a downstream OS reads is
  breaking even if the field was undocumented, because the dependency is real
  whether or not it was declared.
- **Removing or weakening an approval gate.** A consumer built its own safety
  posture on the assumption that this OS stops before acting. MAJOR, and it also
  requires a security re-review.
- **Changing a mode's completion test.** Downstream automation keyed on "done"
  now sees done at a different moment.
- **Moving a canonical state class to a projection**, or the reverse. Anyone
  caching your output is now caching the wrong thing.

These look breaking and are not:

- Adding an optional field to an artifact.
- Adding a new mode that nothing else is required to call.
- Rewriting prose in `README.md`, `SYSTEM.md` or a prompt, when the behaviour
  contract is unchanged.

## Obligations of a version bump

A version number is a promise, so every bump carries work:

1. `manifest.json` `version` is updated.
2. `CHANGELOG.md` gains an entry under the new version, with `Added`, `Changed`,
   `Fixed` and `Removed` as applicable. `verify.py` does not read this, the
   Runtime's `agentik update <slug>` does, so an unrecorded bump is invisible to
   every installed copy.
3. On `MAJOR`, the changelog entry names the migration: what a consumer must
   change, and what happens if they do not.
4. On `MAJOR` or `MINOR`, the release gate runs again in full. A patch may run
   the affected suites only, and the changelog says which.
5. On any change to `emits`, `consumes`, `consumes_from`, `emits_to` or
   `handoffs`, run `python3 OS/_tools/graph.py --strict` and confirm no orphan
   consume was introduced anywhere in the suite.

## The rule that stops silent drift

**A behaviour change with no version bump is a defect, not a shortcut.** The
Runtime cannot distinguish an OS that was improved from one that was quietly
altered, and the operator who trusted the previous behaviour has no signal. If
the change is too small to justify a bump, it is a `PATCH`, and `PATCH` is
cheap. There is no fourth option.
