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
| `omega` CLI | Main binary; `omega --help` is the command source of truth |
| TUI | 7-tab manager: Sessions, Projects, OS, Menu, System, Help, Settings |
| rmux SDK | Terminal multiplexer (sessions, panes, send/capture) |
| Atlas Telegram service | Persistent orchestration process that classifies and routes messages |
| AISB conversation viewer | Optional read-only `aisb-master` rmux mirror; not an agent |
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
├── rules/                         the typed doctrine (.md, editable — `omega rules list` prints the current set)
│   ├── L0-ship-the-truth...md
│   ├── L1-runtime-is-the-only-truth.md
│   └── ... (the Laws + registry-owned named R-* rules; inspect with `omega rules list`)
│
├── agents/                        Agent system prompts
│   ├── aisb-master.md             orchestration prompt retained for service compatibility
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                      14 Matrix agents
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
/model codex                 Switch to Codex (OmegaOS default)
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
│  TUI (7 tabs) · CLI (`omega --help`) · Telegram Bridge          │
│                      ↓ intent                                    │
├─────────────────────────────────────────────────────────────────┤
│  Level 2 — Atlas Telegram orchestration service (persistent)    │
│  Matrix agent registry:                                          │
│    Oracle · Morpheus · Seraph · Keymaker · Smith · Niobe         │
│    Architect · Merovingian · Neo · Zion · Link · Construct       │
│    Pythia · Council                                              │
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
| AISB viewer | (hidden) | `aisb-master` | Optional read-only Telegram conversation mirror |
| Oracle | ◆ | `oracle-{Project}` | Strategic — classify, plan, dispatch |
| Worker | ● | `{Project}-worker-{task}` | Tactical — one task, scope-claimed |
| Home | ⌂ | `claude-1`, `codex-2` | Interactive user sessions |
| System | ⚙ | `omega-telegram-bridge` | Infrastructure daemons |

### Plan execution engine

For multi-step builds, the prose plan is replaced by a typed DAG:

1. **Planner** (`/omg-planner`) decomposes the work into single-worker-dispatch
   steps and emits `.planner/tracker.json` (typed steps, dependencies, a real
   `verify_command` per step).
2. **`omega plan-run`** drives the plan: it spawns a real worker per step in
   dependency order. Pre-run validation refuses skip-prone or fake-completing
   plans (trivial `verify_command`s like `true` are rejected).
3. **Gate** — structural can't-skip enforcement: a step only advances when its
   worker's `done.json` lands and the step's invariants hold. Sequencing is
   enforced by the engine, not by instructions.
4. **Guardian** — independent verification: the step's `verify_command` is run
   by the engine itself, so a worker's self-report alone never completes a step.

`omega plan-status` prints progress from `.planner/tracker.json` read-only.

### Worker isolation

Three mechanisms keep parallel workers from trampling each other:

- **Scope-claim file locks** — before writing, a worker claims its files with
  real advisory locks (`fs2`, `scope.rs`). Overlapping claims are rejected at
  dispatch (`omega scope`).
- **Worktree isolation** — `omega spawn-worker --worktree` gives each parallel
  worker its own git worktree, so simultaneous mutations never share a checkout.
- **Clean merge** — `omega-git-merge` folds worker worktrees back into the main
  branch when they finish.

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
| CLI | `omega <cmd>`; inspect the current surface with `omega --help` |
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
| R-PROD | Ship verification | `ship.rs` polls deploy URL until 200 |
| R-RUBRIC | Rubric upfront | `rubric.rs` writes criteria before dispatch |
| R-VERIFY | Multi-grader (2/3) + Popper falsification | `gate.rs` runs 3 adversarial lenses |
| R-BUDGET | Token budget | `mission.rs` tracks spend, hard cap |
| R-CITE | Citation required | `verifier.rs` rejects uncited claims |
| R-TGSEC | Telegram allow-list | `monitor.rs::is_authorized` |
| R-SCOPE | One writer per file | `scope.rs` rejects overlap at dispatch |

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
auto_spawn_master = false      # optional legacy read-only viewer
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
omega aisb-view          Open read-only AISB conversation viewer
omega master             Compatibility alias for omega aisb-view
omega aisb-chat          Interactive local chat through the Telegram service
omega list               List sessions
omega new <name>         Create session
omega kill <name>        Kill session
omega dispatch <P> <M>   Send mission to oracle
omega orchestrate <P> <M>  Full pipeline (classify → plan → dispatch → gate)
omega rules list         Show the current Laws + operational Rule registry
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
omega rules list                          # Should match the runtime Rule registry
cargo build --release                     # Should be 0 errors
```
