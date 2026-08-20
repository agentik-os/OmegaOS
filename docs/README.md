# docs/ — index

Start with the repo-root [GUIDE.md](../GUIDE.md) (the operator manual) and [README.md](../README.md).

## ⭐ Start here — the feature map

- [FEATURES.md](FEATURES.md) — **the complete feature catalog**: every subsystem (oracles/workers,
  Telegram, rules & laws, skills + atlas + RAG + power-ups, design + Open Design, product system,
  marketing, audits, council, self-improvement) with the exact command/trigger and where it lives.
  The "nothing hidden" map.

## Current reference

- [GETTING-STARTED.md](GETTING-STARTED.md) — post-install setup (login, Telegram, keys, projects, doctor); printed by `omega guide`.
- [ARCHITECTURE.md](ARCHITECTURE.md) — full-system reference: crates, 4-level orchestration, plan engine, worker isolation, channels, gates.
- [ARCHITECTURE-V3.md](ARCHITECTURE-V3.md) — the `~/.omega/` centralized runtime layout (credentials, providers, state).
- [MAP.md](MAP.md) — where everything lives: source repo vs installed binary vs `~/.omega/` runtime.
- [OS-UNIVERSE.md](OS-UNIVERSE.md) — **doctrine**: the Master OS Map v1 — the 12 stacks of the Agentik OS universe, the logic each OS runs, and the rule that every OS works alone. The build standard for an integrated OS is [OS-SUITE.md](OS-SUITE.md); the shipped registry is `OS/_tools/suite.py`.
- [OS-BUILD-STEPPER.md](OS-BUILD-STEPPER.md) — **build program**: the 18-phase method applied per OS, the 5 hard gates, the 4-block step format, and the 11-wave build order over all 120 OS with each one's measured state today.
- [INSTALL-AND-CREDENTIALS.md](INSTALL-AND-CREDENTIALS.md) — install flow + the credentials/OAuth system.
- [THEMES.md](THEMES.md) — the TUI palette gallery and contrast contract.
- [RESET-RECOVERY.md](RESET-RECOVERY.md) — backing up and rebuilding a box (`omega backup` / restore).
- [VERIFICATION-GATE.md](VERIFICATION-GATE.md) — the build-verification gate checklist.
- [skill-atlas.md](skill-atlas.md) — the skill discovery system: `omega-skills`, the semantic RAG, the served catalog.
- [third-party-skills.md](third-party-skills.md) — the pinned MIT skill collections wired at install.
- [open-design.md](open-design.md) — the self-hosted design engine (`omega-design`): local-CLI on your subscription, redesign a project, tailnet view.
- [marketing-machine-ssot.md](marketing-machine-ssot.md) — the marketing suite + publishing funnel.

## Historical / planning (kept for context, not maintained)

- [plans/](plans/) — [CONCEPT.md](plans/CONCEPT.md) (pre-0.1 concept note), [IMPLEMENTATION-PLAN.md](plans/IMPLEMENTATION-PLAN.md) (the Rust-rewrite plan), [GAP-ANALYSIS.md](plans/GAP-ANALYSIS.md), [VAULTS-PROMPT-ANALYSIS.md](plans/VAULTS-PROMPT-ANALYSIS.md), plus dated design specs.
- [MENU-AUDIT.md](MENU-AUDIT.md) — historical TUI menu audit notes.
- [HERMES-INTEGRATION.md](HERMES-INTEGRATION.md), [RECOMMENDED-STACK.md](RECOMMENDED-STACK.md), [CLAUDE-CODE-INTEGRATION.md](CLAUDE-CODE-INTEGRATION.md) — integration notes of varying age; check against the live binary (`omega --help`).
- [specs/](specs/), [reference/](reference/) — design specs and ported-source reference material.
