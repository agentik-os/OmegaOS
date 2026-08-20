---
name: agentik-runtime
description: Install, configure, run, compose, update and evaluate every Agentik OS. Agentik Runtime {OS}, unit 00 of the AGENTIK {OS} suite (00 · RUNTIME). Use when the user asks about agentik runtime or invokes /agentik-runtime.
---

# Agentik Runtime {OS}

Install, configure, run, compose, update and evaluate every Agentik OS, on
whichever AI environment the user happens to be in.

## When to use this

Reach for the Runtime when the question is about the SUITE rather than about a
subject:

- "I want to accomplish X, which systems do I need" (compose a stack)
- "install / configure / run / update / remove an OS"
- "is this OS working" (doctor) or "is it any good" (eval)
- "what exists" (list the 72 units across 9 groups)
- "will this work on ChatGPT / Gemini / Codex" (adapter capability)

Do NOT use it for the subject matter itself. A question about pricing belongs
to Pricing {OS}, not here. The Runtime installs and starts that OS, then gets
out of the way.

**Confused with:** Orchestration {OS} (71) composes many AGENTS inside one
mission; the Runtime composes many OS UNITS across a user's working life. AI
Logic {OS} (64) arbitrates code versus model judgment inside a system; the
Runtime never reasons about a system's internals at all.

## Capabilities

- Compose an objective stated in plain words into an ordered stack of OS units, each with the reason it is there.
- Install one OS or a named stack, resolving declared dependencies first.
- Collect the minimum context an OS needs to be useful now, not everything it could ever use.
- Start a configured OS and hand control to it.
- Report per-surface health, naming any capability the current environment cannot support and the fallback taken.
- Update against a changelog, refusing to apply a breaking change without asking.
- Run an OS's evaluation suites and report per-suite pass or fail.
- Enforce each OS's declared permission boundary and escalate anything beyond it.

## Procedure

1. If the user named an OS, skip to step 4. Otherwise ask the single question:
   what are you trying to accomplish.
2. Resolve that objective against the registry and propose an ordered stack,
   stating why each unit is in it and what it hands to the next.
3. On acceptance, take the first unit only. A stack is installed one unit at a
   time so the user gets a result before the second install.
4. Read the unit's `manifest.json`. Resolve `requires`; offer to install any
   missing dependency rather than proceeding silently.
5. Check the unit's `targets` against the current environment. State any
   unsupported capability and the fallback. Never degrade silently.
6. Install, then configure only the inputs marked required.
7. Run it, and hand over. The OS opens with its own first question.
8. On any later invocation, resolve to the mode the user needs: doctor, eval,
   update, remove, or compose again.

## Handoffs

- To the selected OS, once installed and configured. That OS owns everything
  from there.
- To Context & Memory {OS} (65) for durable user configuration, which is
  canonical there and only projected here.
- From every OS, a declared `requires` list and permission boundary, which the
  Runtime enforces but never authors.
