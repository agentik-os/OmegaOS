# OmegaOS — Architecture Documentation

> **Scope:** authoritative full-system reference (crates, orchestration,
> agent levels, CLI, channels). For the `~/.omega/` centralized runtime
> layout (credentials, models, settings), see [ARCHITECTURE-V3.md](ARCHITECTURE-V3.md).
>
> Complete technical reference for the OmegaOS system.
> Read this first to understand how everything fits together.

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Directory Layout](#2-directory-layout)
3. [Credentials & Multi-Provider](#3-credentials--multi-provider)
4. [4-Level Agent Hierarchy](#4-4-level-agent-hierarchy)
5. [Communication Channels](#5-communication-channels)
6. [Quality Gates](#6-quality-gates)
7. [Installation Flow](#7-installation-flow)
8. [Configuration Reference](#8-configuration-reference)

---

## 1. System Overview

OmegaOS is a 100% Rust agentic terminal operating system. It coordinates multiple
AI coding agents (Claude, Codex, Gemini, Pi, Hermes, GLM, OpenRouter) through a
unified orchestration layer, with a centralized config directory and a Telegram
bot for remote control.

**Key components:**

| Component | Role |
|-----------|------|
| `omega` CLI | Main binary — 40+ commands |
| TUI | 7-tab session manager (Sessions/Menu/Monitor/Projects/Settings/Agentic/Help) |
| rmux SDK | Terminal multiplexer (sessions, panes, send/capture) |
| AISB Master | Always-on Claude session with 13 Matrix agents |
| Telegram Bridge | Long-poll bot, relays messages to AISB |
| Quality Arsenal | 23 forensic audits (code, UX, perf, security, etc.) |

---

## 2. Directory Layout

```
~/.omega/                          MASTER — single source of truth
│
├── OMEGA.md                       Universal system prompt (all LLMs load this)
├── config.toml                    General settings
├── providers.toml                 Per-provider config
├── telegram.toml                  Telegram bridge config (gitignored)
├── projects.json                  Registry of all projects
│
├── credentials/                   ALL provider credentials live HERE
│   ├── claude.json                ← ~/.claude/.credentials.json (symlink)
│   ├── codex.json                 ← ~/.codex/auth.json (symlink)
│   ├── gemini.json                ← ~/.gemini/oauth_creds.json (symlink)
│   ├── glm.json
│   ├── openrouter.json
│   └── accounts/                  Saved account profiles
│       ├── claude-gareth.json
│       └── ...
│
├── rules/                         6 Laws + 20 Rules (.md, editable)
│   ├── L1-runtime-truth.md
│   ├── L2-researcher-not-sycophant.md
│   └── ... (all 15)
│
├── agents/                        Agent system prompts
│   ├── aisb-master.md             Master AISB brain
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                      13 Matrix agents
│       ├── oracle.md / morpheus.md / seraph.md
│       └── ... (13 total)
│
├── skills/                        OmegaOS-shipped skills (audits, design, planner…)
│   ├── pdfgen/                    PDF generator (Next.js + Playwright)
│   └── audits/                    Quality Arsenal audits
│
├── lib/                          Audit runtime (consolidated — NO ~/.aisb)
│   ├── audit-runner.sh           hybrid audit orchestrator
│   ├── audit-gather/             per-audit gatherers (.sh + -summarize.py)
│   └── safe-npm-build.sh         build mutex
├── bin/                          audit-notify.sh + helper binaries
├── repos/                        ← cloned GitHub repos (repos/omega-mc = dashboard)
├── tools/                        ← third-party tools an agent installs
├── prompts/                      runtime prompt scratch (oracle/worker dispatch)
│
├── docs/                          Reference docs (this file)
├── projects/<slug>/               Per-project overrides
│
├── state/                         Runtime (not in git)
│   ├── sessions/                  Active session metadata
│   ├── locks/                     Scope-claim file locks
│   ├── done/                      .done.json files
│   └── telegram-active-model.json Current provider+model per chat
│
├── logs/                          Session logs
└── audit/                         Audit results
```

### Placement convention — where new things go

One home, everything ordered. When an agent/LLM installs something, it lands here:

| Installing… | Goes to | How |
|---|---|---|
| OmegaOS skill (ships w/ product) | `~/.omega/skills/<name>/` | add to repo `skills/`; install.sh copies + makes `/<name>` |
| Global skill for every agent (SST) | `~/.claude/skills/<name>/` | `bunx skills add <repo> --skill <name> -g` |
| A GitHub repo (service/dep) | `~/.omega/repos/<name>/` | `git clone … ~/.omega/repos/<name>` |
| A third-party tool/binary | `~/.omega/tools/<name>/` + symlink in `~/.local/bin` | install there, link the entrypoint |
| An LLM provider CLI | provider default (npm-global / `~/.local/bin`) | `omega install <provider>` (on-demand) |
| Runtime scratch / state | `~/.omega/state` or `~/.omega/logs` | never the repo, never `~` root |

**No `~/.aisb` dual-home** — the audit runtime + state were consolidated into
`~/.omega/{lib,bin,state}`. Secrets live only in `~/.omega/credentials` /
`~/.omega/provisioning` (gitignored). Install parity (Law 0): if a fresh
`git clone && ./install.sh` wouldn't reproduce it, wire it into `install.sh`.

---

## 3. Credentials & Multi-Provider

### Principle

ALL provider credentials live in `~/.omega/credentials/`. The legacy paths
(`~/.claude/.credentials.json`, `~/.codex/auth.json`, etc.) are symlinks
pointing into `~/.omega/`. This way:

- LLM CLIs still find their creds at the expected paths
- Backups only need to cover `~/.omega/`
- Account switching = updating one file
- Migration is a one-time operation

### Supported Providers

| Provider | Type | Credential file | Default model |
|----------|------|-----------------|---------------|
| Claude | OAuth | `credentials/claude.json` | opus |
| Codex | API key | `credentials/codex.json` | gpt-5-codex |
| Gemini | OAuth | `credentials/gemini.json` | gemini-2.5-pro |
| GLM | API key | `credentials/glm.json` | glm-4.6 |
| OpenRouter | API key | `credentials/openrouter.json` | anthropic/claude-sonnet-4.6 |
| Pi | Config | `credentials/pi.json` | (uses OpenRouter) |
| Hermes | API key | `credentials/hermes.json` | (Nous Research) |

### Account Switching

Multiple accounts per provider live in `~/.omega/credentials/accounts/`:

```
omega accounts list                   Show all saved accounts
omega accounts add claude work        Save current Claude creds as "work"
omega accounts switch claude gareth   Switch to claude-gareth account
```

Behind the scenes: switching updates `~/.omega/credentials/claude.json`
to be a copy/symlink of the saved profile.

### Telegram Model Selection (per chat)

```
/model                       Show current + list available
/model claude                Switch to Claude (default model)
/model claude opus           Switch to Claude with opus
/model codex gpt-5           Switch to Codex with gpt-5
/model openrouter            Switch to OpenRouter (default model)
```

Active selection persisted to `~/.omega/state/telegram-active-model.json`.

---

## 4. 4-Level Agent Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│  Level 1 — Human Interface                                      │
│  TUI (7 tabs) · CLI (40+ cmds) · Telegram Bridge                │
│                      ↓ intent                                    │
├─────────────────────────────────────────────────────────────────┤
│  Level 2 — AISB Master (persistent, auto-restart, --continue)   │
│  13 Matrix Agents:                                               │
│    Oracle · Morpheus · Seraph · Keymaker · Smith · Niobe         │
│    Architect · Merovingian · Neo · Zion · Link · Construct       │
│    Pythia                                                        │
│                      ↓ dispatch                                  │
├─────────────────────────────────────────────────────────────────┤
│  Level 3 — Oracle (1 per project)                                │
│  Classify → Plan (Keymaker) → Dispatch workers → Gate (Seraph)  │
│                      ↓ decompose                                 │
├─────────────────────────────────────────────────────────────────┤
│  Level 4 — Workers (ephemeral, parallel, file-lock scoped)      │
│  Execute → Verify → done.json → Oracle acks → close             │
└─────────────────────────────────────────────────────────────────┘
```

### Session Roles

| Role | Icon | Pattern | Purpose |
|------|------|---------|---------|
| AISB Master | ★ | `aisb-master` | Always-on brain, 13 agents |
| Oracle | ◆ | `oracle-{Project}` | Strategic — classify, plan, dispatch |
| Worker | ● | `{Project}-worker-{task}` | Tactical — one task, scope-claimed |
| Home | ⌂ | `claude-1`, `codex-2` | Interactive user sessions |
| System | ⚙ | `omega-telegram-bridge` | Infrastructure daemons |

---

## 5. Communication Channels

### LLM ↔ OmegaOS

Each LLM CLI reads OmegaOS config:

| LLM | Mechanism |
|-----|-----------|
| Claude Code | `~/.claude/rules/omega-*.md` → symlinks to `~/.omega/rules/` |
| Gemini CLI | `~/.gemini/GEMINI.md` includes `@import ~/.omega/OMEGA.md` |
| Codex | `~/.codex/AGENTS.md` → symlink to `~/.omega/OMEGA.md` |
| Pi / Hermes / GLM | Launch with `--append-system-prompt-file ~/.omega/OMEGA.md` |

### User ↔ OmegaOS

| Channel | How |
|---------|-----|
| TUI | `omega` or `omega menu` — full session manager |
| CLI | `omega <cmd>` — 40+ commands |
| Telegram | `/help`, `/list`, `/model`, `/newproject`, `/account`, etc. |

### Inter-process

| Mechanism | Used for |
|-----------|----------|
| rmux SDK (Unix socket) | Session create/kill, send_text, capture_pane |
| JSONL files | done.json signals, inbox events |
| Telegram Bot API | Long-poll, sendMessage, sendDocument, sendChatAction |

---

## 6. Quality Gates

Every mission passes through the rules:

| Rule | Enforces | Implementation |
|------|----------|----------------|
| L1 | Runtime truth | `gate.rs` requires log/pane evidence |
| L2 | Researcher mindset | Worker prompt template |
| L3 | Decide & proceed | `oracle_lifecycle.rs` no idle stops |
| R-14 | Ship verification | `ship.rs` polls deploy URL until 200 |
| R-19 | Rubric upfront | `rubric.rs` writes criteria before dispatch |
| R-21 | Multi-grader (2/3) | `gate.rs` runs 3 lenses |
| R-22 | Regression check | `gate.rs` semantic diff vs previous |
| R-28 | Token budget | `mission.rs` tracks spend, hard cap |
| R-30 | Popper falsification | `gate.rs` runs ≥12 adversarial challenges |
| R-35 | Citation required | `verifier.rs` rejects uncited claims |
| TG-SEC | Telegram allow-list | `monitor.rs::is_authorized` |
| SCOPE-CLAIM | File locks | `scope.rs` rejects overlap at dispatch |

---

## 7. Installation Flow

### Fresh install (`./install.sh`)

```
Phase 1: Detect OS + arch
Phase 2: Install Rust (rustup) if missing
Phase 3: Build rmux from source
Phase 4: Build omega CLI
Phase 5: Setup ~/.omega/ + migrate existing credentials
         - Create credentials/, accounts/, state/, bin/
         - Move ~/.claude/.credentials.json → ~/.omega/credentials/claude.json
         - Create symlinks back to legacy paths
         - Same for codex, gemini, glm (if present)
         - Copy OMEGA.md, agents/, rules/, skills/
         - Run `omega rules export` and `omega sync`
Phase 6: Shell integration (PATH, completions, aliases)
```

### Post-install (`omega install <provider>`)

When a user installs a new LLM CLI:
1. Run the official installer (curl|sh pattern with TTY preserved)
2. `omega sync` — symlinks into LLM config dirs
3. If credentials get written by the installer to legacy path,
   migrate them to `~/.omega/credentials/` + create symlink

---

## 8. Configuration Reference

### `~/.omega/config.toml`

```toml
state_dir = "~/.omega/state"
logs_dir = "~/.omega/logs"
auto_spawn_master = true       # AISB Master auto-created on launch
auto_naming = true             # Sessions named claude-1, codex-2, ...
```

### `~/.omega/providers.toml`

```toml
default_provider = "claude"
default_model = "opus"

[claude]
type = "oauth"
cred_file = "credentials/claude.json"
models = ["opus", "sonnet", "haiku"]
default_model = "opus"

[codex]
type = "api_key"
cred_file = "credentials/codex.json"
models = ["gpt-5", "gpt-5-codex", "o3"]
default_model = "gpt-5-codex"

# ... (other providers)
```

### `~/.omega/telegram.toml` (gitignored)

```toml
bot_token = "..."
chat_id = ...
allow_user_ids = [...]
relay_session = "aisb-master"
enabled = true
```

---

## CLI Quick Reference

```
omega                    Launch TUI
omega menu               Same as omega
omega master             Attach AISB Master
omega list               List sessions
omega new <name>         Create session
omega kill <name>        Kill session
omega dispatch <P> <M>   Send mission to oracle
omega orchestrate <P> <M>  Full pipeline (classify → plan → dispatch → gate)
omega rules list         Show the 6 Laws + 20 Rules
omega rules export       Write to ~/.omega/rules/
omega sync               Symlink to all LLMs
omega accounts list      List provider accounts
omega accounts switch <provider> <name>
omega model show / set <provider> [model]
OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <CHAT> --user-id <UID>
omega telegram run       Start bridge
omega pdf --template=... --send
omega install <agent>    Install LLM CLI + auto-sync
omega projects           Auto-discover projects
```

---

## Verification

To verify the architecture is correctly set up:

```bash
ls ~/.omega/                              # Should show: credentials/ rules/ agents/ skills/ ...
ls ~/.omega/credentials/                  # Should show: claude.json codex.json ...
ls -la ~/.claude/.credentials.json        # Should be a symlink to ~/.omega/credentials/claude.json
omega rules list                          # Should show the 6 Laws + 20 Rules
cargo build --release                     # Should be 0 errors
```
