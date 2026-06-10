# docs/ — index

Start with the repo-root [GUIDE.md](../GUIDE.md) (the operator manual) and [README.md](../README.md).

## Current reference

- [GETTING-STARTED.md](GETTING-STARTED.md) — post-install setup (login, Telegram, keys, projects, doctor); printed by `omega guide`.
- [ARCHITECTURE.md](ARCHITECTURE.md) — full-system reference: crates, 4-level orchestration, plan engine, worker isolation, channels, gates.
- [ARCHITECTURE-V3.md](ARCHITECTURE-V3.md) — the `~/.omega/` centralized runtime layout (credentials, providers, state).
- [MAP.md](MAP.md) — where everything lives: source repo vs installed binary vs `~/.omega/` runtime.
- [INSTALL-AND-CREDENTIALS.md](INSTALL-AND-CREDENTIALS.md) — install flow + the credentials/OAuth system.
- [THEMES.md](THEMES.md) — the TUI palette gallery and contrast contract.
- [RESET-RECOVERY.md](RESET-RECOVERY.md) — backing up and rebuilding a box (`omega backup` / restore).
- [VERIFICATION-GATE.md](VERIFICATION-GATE.md) — the build-verification gate checklist.

## Historical / planning (kept for context, not maintained)

- [plans/](plans/) — [CONCEPT.md](plans/CONCEPT.md) (pre-0.1 concept note), [IMPLEMENTATION-PLAN.md](plans/IMPLEMENTATION-PLAN.md) (the Rust-rewrite plan), [GAP-ANALYSIS.md](plans/GAP-ANALYSIS.md), [VAULTS-PROMPT-ANALYSIS.md](plans/VAULTS-PROMPT-ANALYSIS.md), plus dated design specs.
- [MENU-AUDIT.md](MENU-AUDIT.md) — historical TUI menu audit notes.
- [HERMES-INTEGRATION.md](HERMES-INTEGRATION.md), [RECOMMENDED-STACK.md](RECOMMENDED-STACK.md), [CLAUDE-CODE-INTEGRATION.md](CLAUDE-CODE-INTEGRATION.md) — integration notes of varying age; check against the live binary (`omega --help`).
- [specs/](specs/), [reference/](reference/) — design specs and ported-source reference material.
