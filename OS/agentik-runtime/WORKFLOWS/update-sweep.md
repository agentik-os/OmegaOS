# Workflow: update sweep

**Trigger.** A release landed, or a cadence the user chose.

**Produces.** Every installed unit current, with the changelog entries shown
before anything was applied.

## Steps

1. For each installed unit, read its current `version` and the available one.
2. Skip units already current. Report them as a count, not a list.
3. For each unit with an update, read the `CHANGELOG.md` entries between the two
   versions and show them.
4. Check compatibility and dependencies. An update that would break a declared
   dependency of another installed unit is held, not applied.
5. Apply the non-breaking updates.
6. Stop at any entry marked breaking. Ask, showing exactly what changes.
7. Run doctor across the updated units.

## Completion test

Every installed unit is either current, or held with a stated reason. The user
saw the changelog for every version applied. No breaking change was applied
without an explicit answer.

## Failure paths

| Situation | Response |
|---|---|
| no changelog between versions | do not apply, report the gap: an unexplained version bump is not trustworthy |
| a dependency conflict | hold both units, name the conflict |
| doctor regresses after update | report it against the specific unit, offer the previous version |
