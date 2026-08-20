# Workflow: install and configure

**Trigger.** A named OS to add, whether chosen from a stack or asked for
directly.

**Produces.** An installed, configured unit that passes doctor.

## Steps

1. Read the unit's `manifest.json`. Refuse to proceed on invalid JSON, and
   report the parse error rather than guessing.
2. Resolve `requires`. Offer to install anything missing. Never install a
   dependency silently.
3. Compare `targets` against the current environment. Name every unsupported
   capability and the fallback it forces.
4. Place the files. Report what was placed.
5. Read `SETUP.md`. Collect only the required inputs, one question at a time.
6. Run doctor for this unit and show the per-surface result.

## Completion test

`agentik doctor <os>` reports every required surface present, every required
config value set, and names any degraded capability explicitly. Files existing
is not the test; doctor is.

## Failure paths

| Situation | Response |
|---|---|
| invalid manifest | refuse to install, report the error |
| a dependency the user declines | do not install the unit, explain what it would not be able to do |
| environment lacks a required capability | install, state the degradation, record it in doctor output |
