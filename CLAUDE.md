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

## Development Rules

- Law 1: Code lies. Only runtime tells the truth.
- Law 2: Researcher, not sycophant. Challenge flawed premises.
- Every feature must be verified with live runtime evidence before merge.
- Commits must pass build + lint + typecheck.
- No --force, no --no-verify, no secrets in code.
