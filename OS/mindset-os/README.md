# Mindset OS

AgentikOS operative system — personal group — **integrated** (Jim Rohn Extended v2).

An evidence-aware identity, wellbeing, performance and wealth coach built on
Jim Rohn's philosophy-attitude-activity-results-lifestyle framework. It builds
a coherent personal operating system — identity, purpose, written goals,
self-education, discipline, mental and emotional fitness, health/energy,
habits, wealth behavior, relationships, seasons, routines and reviews — not a
motivational list. Payload source: `Mindset_OS_Jim_Rohn_Extended_v2.zip`
(Deposit, 2026-08-10).

## Layout

| Path | What |
|---|---|
| `pack/` | The pack verbatim: SKILL.md, 20 references (doctrine, Jim Rohn approach + 90-day program, identity/purpose, philosophy compiler, goals, habits, mental-emotional, health-energy, wealth-relations, spirituality-manifestation, practices, cadences, measurement, assessment, question bank, output contracts, operating model, evidence map, **safety**), assets (icon), 2 scripts, agents/openai.yaml |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-mindset` | The OmegaOS CLI — `new` (workspace) + `score` (weekly scorecard); stdlib Python, no venv |
| `commands/codex-mindset-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/mindset-os.md`) |

The Claude command is the `mindset-os` skill (the pack skill folder verbatim
at `skills/mindset-os/`), installed as `/mindset-os` + `/omg-mindset-os`.

## Run it

```bash
omega-mindset new --name "You" --output ~/mindset   # 19-file editable workspace
omega-mindset score ~/mindset/04_WEEKLY_SCORECARD.json
```

The coaching runs in an agent: `/mindset-os` in Claude, the Codex prompt, or
the OS master agent (TUI OS tab -> Enter, Telegram bot via `T`). It runs the
STABILIZE -> OBSERVE -> CLARIFY -> DESIGN IDENTITY -> CHOOSE STRATEGY loop and
labels every claim E1/E2/S/P/C.

## Safety (non-negotiable)

Protect life, health, sleep, mental stability, integrity and relationships
before optimization. Wealth is an OUTCOME, never promised. No clinical,
crisis, medication or diagnosis advice — `references/safety.md` routes to a
qualified professional. Coach WITH the operator, never create dependency.
