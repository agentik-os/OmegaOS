# ADR — AGK Lab loop on three backends

Status: accepted (2026-08-24)
Informed by: Gareth Desktop master prompt (Discord lab content) + live Mac
evidence (Grok 2026-08-24). No Discord connector.

## Decision

Every Omega mission runs the Lab loop as plan steps, not as a docs-only essay:

Understand → Explain → Design → Build → Debug → Test → Evaluate → Secure →
Deploy → Observe → Improve.

The same orchestration API (`omega new`, `omega send`, `omega capture`,
`omega dispatch`, `omega spawn-worker`, `omega status --json`) targets three
backends:

| Backend | Role |
|---|---|
| **Codex** | Mac/VPS writer and default oracle. `omega new --agent codex` must keep Codex alive. |
| **Hermes** | Home only (`omega new --agent hermes`). Never `dispatch --agent hermes`, never a worker. Optional: type `codex` inside a Hermes/shell pane. |
| **Cloud** | This Cursor Cloud Agent. Writer for OmegaOS itself. Not `omega dispatch`. Omega oracles stay on Mac/VPS Codex/Claude. |

Grok Bot (when present) is an external orchestrator. It is not a fourth Omega
backend and not a substitute for `omega send`.

## Launch contract

The pane **is** the agent. Agent exit = session death. Never `; exec bash`
after the agent.

Codex unattended pair (0.149+):

```
codex --sandbox workspace-write --ask-for-approval never
```

Invalid: `--approve-for-me` together with `--sandbox` (CLI or
`~/.codex/config.toml` `sandbox_mode`).

Hermes Home: `hermes chat --yolo` (and `HERMES_YOLO_MODE=1`). Never `-q` for a
pane launch.

## Follow-up

One mission owner. Composer not ready → JSON error
`followup_pane_not_ready`. Do not spawn `oracle-*-2`.

## Skills budget

`omega sync` must not dump the full skill catalog into `~/.agents/skills`.
Codex SessionStart allowlists `agentic-engineering-lab` only.

## Out of scope

Publishing, second writer, CLIENT tenants, rewriting the provider catalog
again, Discord connector.
