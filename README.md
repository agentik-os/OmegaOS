# OmegaOS

Agentic terminal operating system built on [rmux](https://github.com/agentik-os/rmux).

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
omega                           # Launch TUI session manager
omega new my-session            # Create a new rmux session
omega new worker-1 --cmd claude # Create session running Claude Code
omega list                      # List sessions (grouped by project)
omega dispatch MyProject "Fix the auth bug"  # Dispatch oracle mission
omega send my-session "hello"   # Send text to a session
omega capture my-session        # Capture pane output
omega status my-session         # Show last 30 lines
omega kill my-session           # Kill a session
omega done worker-1 done_clean "Implemented feature X"  # Signal completion
omega init                      # Initialize config
```

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

## Project Structure

```
OmegaOS/
├── crates/
│   ├── omega-core/     # rmux SDK integration, session model, dispatch, done signals
│   ├── omega-tui/      # ratatui session manager with project grouping
│   └── omega-cli/      # `omega` binary with subcommands
├── scripts/
│   ├── dispatch-to-oracle.sh   # Create oracle session for a project
│   ├── dispatch-to-session.sh  # Create worker session with prompt
│   ├── worker-mark-done.sh     # Signal task completion
│   └── close-gate.sh           # Quality gate checks
├── config/
│   └── default.toml            # Default configuration
└── install.sh                  # One-command installer
```

## Tech Stack

- **rmux** — Rust terminal multiplexer with typed SDK, daemon architecture, Playwright-style `wait_for_text`
- **ratatui** — Terminal UI framework for the session manager
- **clap** — CLI argument parsing
- **tokio** — Async runtime for rmux SDK operations

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

## Origins

OmegaOS integrates patterns from four projects:

- **[rmux](https://github.com/agentik-os/rmux)** — The terminal multiplexer engine (Rust, daemon-backed SDK)
- **[tmux-claude](https://github.com/agentik-os/tmux-claude)** — Session manager UX (grouping, tree hierarchy, progress)
- **[OmegaSetup](https://github.com/agentik-os/OmegaSetup)** — Orchestration layer (dispatch, quality gates, done signals)
- **[earendil/coding-agent](https://github.com/earendil-works/pi)** — Session architecture patterns (JSONL persistence, RPC mode)

## License

MIT OR Apache-2.0
