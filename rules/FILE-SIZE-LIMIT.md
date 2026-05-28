# FILE-SIZE-LIMIT — Avoid files >1500 lines, refactor proactively

**Category:** Code Quality
**Added:** 2026-05-28

## Rule

Files over 1500 lines should be refactored unless there is a documented
reason to keep them whole. This is NOT a hard ban — it is a strong signal
that the file is doing too many things.

When a file approaches 1500 lines, do an analysis:
1. List the distinct responsibilities in the file
2. Identify natural seams (groups of functions that share state vs not)
3. Split into modules along those seams
4. Each new module should have a single clear purpose statable in one sentence

## Why

- LLM context windows: a 3000-line file consumes most of the window when
  read, leaving little room for the actual task
- Cognitive load: humans (and agents) lose track of context past ~500 lines
- Merge conflicts: large files are conflict magnets
- Test isolation: hard to unit-test pieces of a giant file

## When NOT to split

- Generated code (e.g., a 2000-line protobuf stub) — keep as-is
- Single-purpose data files (a JSON-like Rust constants module)
- When splitting would create artificial cross-module dependencies that
  hurt clarity more than the file size hurts it

## Application to OmegaOS

Current state (2026-05-28): the largest file is
`crates/omega-cli/src/telegram_bridge.rs` at 2300+ lines. It needs a
refactor pass — natural seams: handlers/commands, callbacks, account
helpers, OAuth flow, polling loop, formatting helpers. Each could become
its own module under `crates/omega-cli/src/telegram/`.

## Application to general Claude Code rules

When dispatched, agents should:
- Check the file size of any file they're about to modify
- If >1500 lines, propose a refactor BEFORE adding new code
- Never grow a file past 2000 lines without explicit user approval

## Origin

Repeated experience: large bridges/handlers ended up with tangled state,
hard-to-trace bugs (the "before captured after send" bug took 5 iterations
to find inside a 1500-line file). Smaller files with explicit interfaces
catch these bugs at the type-checker level.
