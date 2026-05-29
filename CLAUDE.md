# OmegaOS — Agentic Terminal Operating System

## What is this?

OmegaOS is an open-source agentic terminal operating system built on **rmux** (Rust terminal multiplexer).
It turns any VPS or local machine into a fully autonomous multi-agent development platform.

Anyone clones this repo, runs `./install.sh`, and gets:
- A daemon-backed terminal multiplexer (rmux) with typed SDK
- An AI agent orchestration layer (oracle/worker hierarchy)
- A session manager with live progress tracking (TUI menu)
- Quality gates, audit chains, and adversarial verification
- Telegram/webhook integration for human-in-the-loop

## Architecture — 4 Levels

```
Level 1 — Human Interface (Telegram / CLI / Web)
    ↓ intent
Level 2 — AISB Orchestrator (persistent daemon)
    ↓ dispatch
Level 3 — Oracle (1 per project, strategic)
    ↓ decompose + delegate
Level 4 — Workers (ephemeral, parallel, file-lock scoped)
    ↓ execute → verify → report
```

## Tech Stack

- **rmux** — Rust terminal multiplexer (daemon, SDK, hooks, PTY)
- **Claude Code CLI** — AI agent runtime (or any CLI agent)
- **Convex** (optional) — Real-time backend for state sync
- **Hermes** (optional) — Multi-agent coordinator via Gemini

## Key Patterns

### Inspired by tmux-claude UX (re-implemented in Rust against the rmux SDK — no tmux runtime dependency)
- Option+Z session manager menu (fzf-based, grouped, progress bars)
- Oracle/worker tree hierarchy display
- Team spawn + layout (N agents in split panes)
- Session protection, kill history, auto-discovery

### From Omega System (Orchestration)
- 13 AISB Matrix agents (Oracle, Morpheus, Seraph, Keymaker, etc.)
- Quality gates (rubric, grader consensus, Popper falsification)
- Done.json webhook → Telegram reports
- Multi-account Claude rotation for unlimited budget

### From earendil/coding-agent (Session Architecture)
- Service-Session-Runtime separation
- RPC mode for external orchestration
- JSONL session persistence with branching
- Extension hooks for customization

## Laws vs Rules

- **Laws (L1, L2, L3)** — inviolable, universal, top-priority. They bind every agent always and
  override every rule and task. Few, stable, never scoped-out. Rendered first everywhere
  (TUI Info tab, `omega rules list`, every prompt block) and visually distinct.
- **Rules (R-NN, named)** — operational, categorized (Universal / QualityGate / Orchestration /
  Reporting / Safety), scoped per agent level. Guidelines that implement the Laws in practice.

Source of truth: `crates/omega-core/src/rules.rs` (`RuleKind::{Law, Rule}`).

## Development Rules

- **LAW 0 — INSTALL PARITY (NON-NEGOTIABLE): every improvement to OmegaOS MUST
  keep `install.sh` complete. A feature is NOT done until a fresh
  `git clone … && ./install.sh` reproduces it.** Before declaring any change
  done:
  1. New asset (agent/command/config/template/cron/dir)? → add the copy/setup
     step to `install.sh` (binary changes ship automatically — `install.sh`
     builds from source).
  2. Run `./scripts/verify-install.sh` — it must pass (binary-from-source,
     agents, commands, configs, crons, **no secrets tracked**, git clean,
     remote in sync).
  3. `git add -A && commit && push` — GitHub always holds the latest, and the
     installer always installs the latest. NEVER leave an improvement that a
     fresh install wouldn't get.
  Secrets (tokens, creds) live in `~/.omega/` only — gitignored, NEVER in the repo.
- Law 1: Code lies. Only runtime tells the truth.
- Law 2: Researcher, not sycophant. Challenge flawed premises.
- Every feature must be verified with live runtime evidence before merge.
- Commits must pass build + lint + typecheck.
- No --force, no --no-verify, no secrets in code.
