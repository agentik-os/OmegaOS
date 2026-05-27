# OmegaOS

**100% Rust agentic terminal OS — built on [rmux](https://github.com/agentik-os/rmux), NOT tmux.**

Sessions, panes, multiplexing, the daemon, the SDK, the orchestrator, the TUI — all native Rust.
tmux is **not** a dependency and is **never invoked**. If you see a reference to "tmux" in this
repo it's either historical credit for UX inspiration or a legacy env-var fallback.

Turns any machine into a multi-agent development platform. Create oracle/worker hierarchies, dispatch missions, track progress, and gate quality — all from the terminal.

## Quick Start

```bash
# Install (builds rmux + omega from source)
curl -sSL https://raw.githubusercontent.com/agentik-os/OmegaOS/main/install.sh | bash

# Or clone and build locally
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

## Usage

```bash
# Session management
omega                           # Launch TUI session manager (alias: omega menu)
omega new my-session            # Create a new rmux session
omega new worker-1 --cmd claude # Create session running Claude Code
omega new w-auth --files "src/auth.rs,src/db.rs"  # Create with scope-claim
omega list                      # List sessions (grouped by project, with scopes)
omega attach my-session         # Attach to a session
omega kill my-session           # Kill a session (auto-releases scope)
omega status my-session         # Show last 30 lines of pane
omega send my-session "hello"   # Send text + Enter to a session
omega capture my-session        # Capture full pane content

# Orchestration
omega dispatch MyProject "Fix the auth bug"  # Dispatch oracle mission
omega spawn-worker auth "Fix login flow" --files "src/auth.rs"  # Spawn scoped worker
omega team MyProject builder:"Build the UI" tester:"Write tests"  # Multi-agent team
omega done worker-1 done_clean "Implemented feature X"  # Signal completion

# Quality & monitoring
omega gate oracle-MyProject --mission "Build auth"  # Create quality rubric
omega gate oracle-MyProject                          # Check rubric criteria
omega scope worker-B src/auth.rs src/api.rs          # Check scope conflicts
omega patrol --once                                  # Run health check once
omega patrol --interval 60                           # Run patrol daemon

# Configuration & rules
omega rules list                # Show all 15 operational rules
omega rules export              # Export rules to ~/.omega/rules/*.md
omega sync                      # Sync config to all LLMs (Claude, Gemini, Codex)
omega config set claude.model opus  # Set provider config
omega config show               # Show all provider settings

# Telegram bridge (talk to AISB Master from your phone)
omega telegram setup <TOKEN> <CHAT_ID> --user-id <UID>
omega telegram run              # Start the bridge (auto-starts on setup)

# PDF reports
omega pdf --template=whitepaper --demo --out=report.pdf
omega pdf --template=audit --data=audit.json --send  # Generate + Telegram

# Agents & tools
omega install hermes            # Install an agent CLI (+ auto-sync)
omega install pi                # Each install auto-wires to ~/.omega/
omega agents                    # List all supported agents + availability
omega projects                  # Auto-discover projects

# History & config
omega log oracle-MyProject      # View JSONL session history
omega init                      # Initialize OmegaOS configuration
```

## Centralized Config (`~/.omega/`)

All LLM agents share a single source of truth:

```
~/.omega/
├── OMEGA.md            # Universal system prompt (loaded by every LLM)
├── rules/              # 15 operational rules as editable .md files
├── agents/             # AISB Master + 13 Matrix agent prompts
├── skills/pdfgen/      # PDF report generator (4 templates)
├── config.toml         # General settings
├── providers.toml      # Per-LLM settings (model, API key, etc.)
└── telegram.toml       # Telegram bridge config (gitignored)
```

When you install an LLM (via `omega install` or the TUI Settings), OmegaOS
automatically runs `omega sync` to create symlinks:

| LLM | Integration |
|-----|-------------|
| Claude | `~/.claude/rules/omega-*.md` → symlinks to `~/.omega/rules/` |
| Gemini | `~/.gemini/GEMINI.md` → imports `~/.omega/OMEGA.md` |
| Codex | `~/.codex/AGENTS.md` → symlink to `~/.omega/OMEGA.md` |
| Pi/Hermes/GLM | System prompt injected via `--append-system-prompt-file` |

## Architecture

```
Level 1 — Human Interface (CLI / TUI / Telegram)
    ↓ intent
Level 2 — AISB Orchestrator (optional, persistent daemon)
    ↓ dispatch
Level 3 — Oracle (1 per project, strategic planning)
    ↓ decompose + delegate
Level 4 — Workers (ephemeral, parallel, file-lock scoped)
    ↓ execute → verify → done.json signal
```

### Session Roles

| Role | Icon | Pattern | Purpose |
|------|------|---------|---------|
| Oracle | ◆ | `oracle-{Project}` | Strategic — analyzes mission, spawns workers |
| Worker | ● | `{Project}-worker-{task}` | Tactical — executes one task, signals done |
| Home | ⌂ | `Home`, `c-*` | Interactive human sessions |
| System | ⚙ | `AISB-*`, `earthbit-*` | Infrastructure daemons |

### Quality Gate Chain

```
Worker → done.json → Oracle acks → oracle.done.json → AISB reports → session close
```

Each level must acknowledge the level below before a session can be safely closed.

## TUI Session Manager

Launch with `omega` or `omega menu`:

```
┌─ OmegaOS ──────────────────────────────────────┐
│  Sessions  │  Menu  │  Help                     │
├─────────────────────────────────────────────────┤
│ ─── Home ───                                    │
│   ⌂ Home                                       │
│                                                 │
│ ─── Causio ───                                  │
│   ◆ oracle-Causio          [████░░░░] 50%      │
│   ├ ● Causio-worker-auth   [██████░░] 75%      │
│   └ ● Causio-worker-ui     [████████] 100%     │
│                                                 │
│ ─── System ───                                  │
│   ⚙ AISB-monitor                               │
├─────────────────────────────────────────────────┤
│ Ω  Press ? for help │ Tab to switch │ q to quit │
└─────────────────────────────────────────────────┘
```

**Keys:** `↑↓`/`jk` navigate, `Enter` attach, `x` kill, `.` protect, `r` refresh, `Tab` switch views

## Orchestration Scripts

```bash
# Dispatch an oracle for a project
scripts/dispatch-to-oracle.sh MyProject "Build the landing page"

# Dispatch a worker session
scripts/dispatch-to-session.sh MyProject-worker-auth "Fix the login flow" ~/projects/myproject

# Mark worker as done (called by the worker agent)
scripts/worker-mark-done.sh done_clean "Implemented OAuth flow"

# Check if a session can be safely closed
scripts/close-gate.sh check-worker MyProject-worker-auth
scripts/close-gate.sh ack-worker MyProject-worker-auth oracle-MyProject
```

## Configuration

```toml
# ~/.omega/config.toml

state_dir = "~/.omega/state"
logs_dir = "~/.omega/logs"
agent_command = "claude"       # or "codex", or path to any CLI agent
default_model = "opus"

[[projects]]
name = "MyProject"
path = "/home/user/projects/myproject"
category = "Work"
```

## Scope-Claim File Locking

Prevents two workers from editing the same files concurrently:

```bash
omega new worker-auth --files "src/auth.rs,src/session.rs"
omega new worker-db --files "src/auth.rs"  # ERROR: scope conflict with worker-auth
omega done worker-auth done_clean "Done"   # Auto-releases scope
omega new worker-db --files "src/auth.rs"  # Now succeeds
```

## Team Spawning

Create N agents in split panes within a single rmux session:

```bash
omega team MyProject \
  architect:"Design the API schema" \
  builder:"Implement the endpoints" \
  tester:"Write integration tests"
```

## Patrol Daemon

Background watchdog for session health:

```bash
omega patrol                # Run as daemon (60s interval)
omega patrol --once         # Single health check
omega patrol --interval 30  # Custom interval
```

Detects: orphaned sessions, done workers awaiting acknowledgment, stale scope claims.

## Project Structure

```
OmegaOS/
├── crates/
│   ├── omega-core/          # Core library (Rust)
│   │   ├── session.rs           # rmux SDK integration, session roles
│   │   ├── agents.rs            # Agent registry (Claude/Codex/Gemini/Pi/Hermes/GLM)
│   │   ├── aisb.rs              # AISB Master (auto-spawn, --continue)
│   │   ├── aisb_agents.rs       # 13 Matrix agents (typed, with prompts)
│   │   ├── rules.rs             # 15 operational rules registry
│   │   ├── monitor.rs           # Billing, Telegram config, bot status
│   │   ├── providers.rs         # Per-LLM config (model, API key)
│   │   ├── dispatch.rs          # Oracle/worker dispatch
│   │   ├── scope.rs             # File-lock scope claims
│   │   ├── team.rs              # Multi-agent team spawning
│   │   └── gate.rs              # Rubric-based quality gates
│   ├── omega-tui/           # TUI session manager (ratatui)
│   │   ├── ui.rs                # 6 tabs: Sessions/Menu/Monitor/Settings/Info/Help
│   │   ├── input.rs             # Keyboard + mouse + paste handling
│   │   └── app.rs               # App state, fields, navigation
│   └── omega-cli/           # `omega` binary (25+ subcommands)
│       ├── main.rs              # CLI entry point + all command handlers
│       └── telegram_bridge.rs   # Rust Telegram bot (long-poll, relay, typing)
├── agents/                  # Agent system prompts
│   ├── aisb-master.md           # Master AISB brain
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                    # 13 Matrix agents
├── rules/                   # 15 operational rules (.md files)
├── tools/pdfgen/            # PDF report generator (Next.js + Playwright)
├── config/default.toml      # Default configuration
├── OMEGA.md                 # Universal agent instructions
└── install.sh               # One-command installer (6 phases)
```

## Supported Agents

OmegaOS orchestrates multiple AI coding agents. Install any of them from the TUI (Settings → Install) or CLI:

| Agent | Provider | Install | Description |
|-------|----------|---------|-------------|
| [Claude Code](https://claude.ai/code) | Anthropic | `omega install claude` | Primary agent — Opus/Sonnet models, tool use, multi-file editing |
| [Codex](https://github.com/openai/codex) | OpenAI | `omega install codex` | GPT-4 powered coding agent |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | Google | `omega install gemini` | Gemini 2.5 Pro/Flash models |
| [Pi](https://github.com/earendil-works/pi) | earendil-works | `omega install pi` | Lightweight agent, OpenRouter multi-model |
| [Hermes](https://hermes-agent.nousresearch.com/) | Nous Research | `omega install hermes` | Multi-agent coordinator |
| [GLM](https://www.z.ai/) | Z.AI / Zhipu | `omega install glm` | GLM-4 models |
| Shell | — | built-in | Plain terminal (no AI) |

Every install automatically runs `omega sync` — the new agent reads from `~/.omega/` (same rules, same prompts, same skills).

## Integrated Tools

| Tool | Description |
|------|-------------|
| **PDF Generator** | 4 templates (whitepaper, audit, marketing, doc) with Playwright rendering. `omega pdf --template=whitepaper --send` generates + sends to Telegram. |
| **Telegram Bridge** | Persistent bot that relays messages to AISB Master. Typing indicator, auto-restart on crash, HTML formatted responses. |
| **AISB Master** | Always-on AI brain (13 Matrix agents). Auto-spawns on launch, resumes conversation with `--continue`. |

## Tech Stack

| Component | Technology | Role |
|-----------|-----------|------|
| [rmux](https://github.com/agentik-os/rmux) | Rust | Terminal multiplexer — daemon, SDK, PTY, sessions, panes |
| [ratatui](https://github.com/ratatui/ratatui) | Rust | TUI framework — 6-tab session manager |
| [tokio](https://tokio.rs) | Rust | Async runtime for rmux SDK operations |
| [clap](https://github.com/clap-rs/clap) | Rust | CLI argument parsing + completions |
| [reqwest](https://github.com/seanmonstar/reqwest) | Rust | Telegram Bot API + PDF delivery |
| [Next.js](https://nextjs.org) + [Playwright](https://playwright.dev) | TypeScript | PDF report rendering engine |

## Done Signal Protocol

Workers signal completion by writing `~/.omega/state/worker-{session}.done.json`:

```json
{
  "session": "MyProject-worker-auth",
  "status": "done_clean",
  "summary": "Implemented OAuth2 flow with PKCE",
  "commit": "a1b2c3d4",
  "finished_at": "2026-05-26T14:23:45Z"
}
```

Status values: `done_clean` (verified complete), `pending` (more work needed), `failed` (blocked).

## Origins & Credits

OmegaOS integrates patterns from four projects, all reimplemented in Rust:

| Project | What we took | Link |
|---------|-------------|------|
| **rmux** | Terminal multiplexer engine (daemon, SDK, PTY) — our runtime | [agentik-os/rmux](https://github.com/agentik-os/rmux) |
| **tmux-claude** | UX patterns only (session grouping, tree hierarchy, progress bars) — reimplemented against rmux SDK, no tmux dependency | [agentik-os/tmux-claude](https://github.com/agentik-os/tmux-claude) |
| **OmegaSetup** | Orchestration layer (dispatch, quality gates, done signals, AISB agents) | [agentik-os/OmegaSetup](https://github.com/agentik-os/OmegaSetup) |
| **Pi (coding-agent)** | Session architecture patterns (JSONL persistence, RPC mode) | [earendil-works/pi](https://github.com/earendil-works/pi) |

## Telegram Bridge (Optional)

Talk to AISB Master from your phone — pure Rust, no Python dependency.

```bash
# 1. Create a bot via @BotFather on Telegram
# 2. Get your user ID via @userinfobot
# 3. Setup:
omega telegram setup <BOT_TOKEN> <CHAT_ID> --user-id <YOUR_USER_ID>
# Bridge auto-starts after setup

# Manual control:
omega telegram run       # Start bridge
omega telegram status    # Check config
omega telegram disable   # Pause without deleting config
omega telegram enable    # Resume
```

**Features:**
- Messages relayed to AISB Master, responses sent back (HTML formatted)
- Typing indicator while agent thinks
- Auto-restart: if AISB Master crashes, bridge revives it with `--continue` (same conversation)
- Healthcheck every 60s — Master always alive
- Security: `allow_user_ids` filter — only you can control the bot
- Commands: `/help`, `/list`, `/status [session]`, `/billing`, `/relay <session> <text>`

## Shell Completions

```bash
omega completions bash > /etc/bash_completion.d/omega
omega completions zsh > ~/.zsh/completions/_omega
omega completions fish > ~/.config/fish/completions/omega.fish
```

Auto-installed by `install.sh` based on your shell.

## Agent Templates

System prompts for oracle/worker/team-lead live in `agents/` and are installed to `~/.omega/agents/`. Customize them per project to inject domain knowledge.

## License

Licensed under either of MIT or Apache-2.0 at your option.
