# SIMPLICITY-COMPLETE — Simple but complete, never simplistic

**Category:** Universal
**Added:** 2026-05-28

## Rule

Simplicity does NOT mean incomplete or stripped-down. It means thinking with
simple parts that work in a simple way — while still covering every case the
problem demands.

A system is well-designed when:
- Each piece does ONE thing clearly
- Connections between pieces are obvious
- You can explain the whole flow in one paragraph
- A new developer (or LLM) can read any single file and understand its role

It is NOT well-designed when:
- Cleverness replaces clarity (multi-purpose abstractions to "save lines")
- A single feature requires reading 5+ files to trace
- Edge cases are buried inside the happy path
- "Robustness" means a maze of fallbacks no one fully understands

## Application

Before adding complexity, ask:
1. Can this be a 50-line function instead of a 5-class hierarchy?
2. Can two layers of indirection become one?
3. Is the fallback actually used, or is it dead defensive code?

When reviewing another agent's output, the bias should be toward:
- Smaller modules
- Fewer dependencies
- Direct data flow
- Explicit over implicit

## Origin

Observed during the OmegaOS Telegram bridge work: layers of polling,
extraction, stable-count, fallback fallbacks made the system harder to
debug than the problem warranted. Hermes (NousResearch) and OpenCode
solve similar problems with much smaller, more direct code paths.
Simplicity is the goal; completeness is the constraint, not vice versa.
