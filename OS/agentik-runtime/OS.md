# Agentik Runtime {OS}: Operating Specification

## 1. Purpose

Install, configure, run, compose, update and evaluate every Agentik OS, on
whichever AI environment the user happens to be in.

The Runtime is the operating layer of the suite. It is the only unit that knows
about all the others, and the only one a user has to install by hand.

## 2. Boundary

- **Owns:** the lifecycle of an OS on a machine (install, configure, run,
  compose, update, evaluate, doctor), the registry of what exists, dependency
  resolution between OS units, the permission model, and the adapter layer that
  maps one operating logic onto different AI environments.
- **Does not own:** the content of any OS. It never decides what Blueprint {OS}
  should ask or how Revenue {OS} models a receivable. It runs them.
- **Hands off to:** the selected OS, once installed and configured.
- **Consumes from:** `OS/_registry.json` (what exists), each unit's
  `manifest.json` (what it needs), and Context & Memory {OS} (what the user
  has already established).

The rule that keeps this honest: **the Runtime may read every OS, and may write
none of them.**

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `INSTALL` | user names an OS, or a stack | the unit present locally | files on disk and `doctor` is green |
| `CONFIGURE` | an installed OS lacks required context | a populated config | every required input has a value |
| `RUN` | a configured OS is selected | an active session | the user ends it |
| `COMPOSE` | user states an objective, not an OS | an ordered stack | the user accepts the stack |
| `UPDATE` | a newer version exists | the updated unit | version bumped, changelog shown |
| `EVALUATE` | user asks whether an OS is sound | a pass or fail report | every suite has run |
| `DOCTOR` | something is not working | a diagnosis and next step | each surface reported present or absent |

`COMPOSE` is the mode a new user actually starts in. They do not know which OS
they need; they know what they are trying to accomplish.

## 4. Inputs

- The suite registry: the 72 units, their groups, numbers and slugs.
- Each unit's `manifest.json`: version, dependencies, targets, entrypoints,
  and its human approval boundary.
- The active environment: which AI product this is running on, and which
  capabilities it exposes.
- The user's objective, in their own words.

## 5. Outputs

- Installed and configured OS units under `~/.omega/os/<slug>/`.
- A composed stack: an ordered list of OS units with the reason for each.
- A doctor report: per-surface present or absent, never a summary badge.
- An evaluation report: per-suite pass or fail with the failing assertion named.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | which OS are installed, and at what version | `~/.omega/os/<slug>/manifest.json` |
| canonical | user configuration per OS | Context & Memory {OS} |
| projection | the registry | generated from `OS/_tools/suite.py` |
| cache | resolved dependency graphs | recomputed, never trusted across versions |
| temporary | the current session's selection | the session |

## 7. Rules and invariants

1. **The manifest is the contract.** The Runtime operates on `manifest.json`,
   not on folder shape. An OS may organise its own files however it likes, as
   long as its manifest declares where the parts are. This is what allows 72
   units of different vintages to be driven by one layer.
2. **Absence is reported, never inferred away.** If an environment lacks a
   capability an OS needs, `doctor` says so and names the fallback. It never
   silently degrades.
3. **Static presence is not runtime verification.** A directory existing is not
   an OS working. `install` reports files; only `eval` reports behaviour.
4. **Dependencies resolve before run.** An OS whose declared requirements are
   not installed does not start; the Runtime offers to install them.
5. **Permissions are explicit.** An OS gets access to what its manifest
   declares and nothing else. Escalation is a question to the user.
6. **The user owns their systems.** Everything is local files. Nothing about a
   configured OS is trapped in one AI vendor.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| unknown OS name | list the closest matches from the registry, do not guess |
| missing dependency | name it, offer to install, do not proceed silently |
| unsupported capability on this target | state it, name the fallback, continue degraded and say so |
| invalid manifest | refuse to run that OS, report the parse error |
| conflicting config | ask, never pick |

## 9. Human approval boundary

The Runtime asks before:

- deleting or overwriting an installed OS or its configuration
- sending anything to an external service
- granting an OS access to a data class its manifest did not previously declare
- running an update that a changelog marks as breaking

## 10. Completion criteria

A user can say what they are trying to accomplish, receive a stack, install it,
configure only what is needed to start, run it, and get a real result, without
ever reading this file.
