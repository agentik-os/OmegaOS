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

# History & config
omega log oracle-MyProject      # View JSONL session history
omega init                      # Initialize OmegaOS configuration
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
│   ├── omega-core/     # Core library
│   │   ├── session.rs      # rmux SDK integration, session roles
│   │   ├── dispatch.rs     # Oracle/worker dispatch
│   │   ├── scope.rs        # File-lock scope claims
│   │   ├── team.rs         # Multi-agent team spawning
│   │   ├── patrol.rs       # Session health watchdog
│   │   ├── session_log.rs  # JSONL session persistence
│   │   ├── gate.rs         # Rubric-based quality gates
│   │   ├── done.rs         # Done signal protocol
│   │   ├── progress.rs     # Progress tracking
│   │   └── config.rs       # Configuration management
│   ├── omega-tui/      # ratatui session manager with project grouping
│   └── omega-cli/      # `omega` binary with 18 subcommands
├── scripts/
│   ├── dispatch-to-oracle.sh   # Create oracle session
│   ├── dispatch-to-session.sh  # Create worker session
│   ├── worker-mark-done.sh     # Signal task completion
│   └── close-gate.sh           # 3-tier quality gate checks
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
- **[tmux-claude](https://github.com/agentik-os/tmux-claude)** — UX inspiration only (session grouping, tree hierarchy, progress bars). OmegaOS itself uses **rmux** (Rust), not tmux. The UX patterns were ported and reimplemented natively against the rmux SDK.
- **[OmegaSetup](https://github.com/agentik-os/OmegaSetup)** — Orchestration layer (dispatch, quality gates, done signals)
- **[earendil/coding-agent](https://github.com/earendil-works/pi)** — Session architecture patterns (JSONL persistence, RPC mode)

## Telegram Bot (Optional)

Remote dispatch via Telegram:

```bash
cd bot/
pip install -r requirements.txt
export OMEGA_BOT_TOKEN="your-bot-token-from-@BotFather"
export OMEGA_CHAT_ID="your-chat-id"
python main.py
```

Then in Telegram:
- `/dispatch MyProject Fix the login bug` → dispatches an oracle
- `/list` → shows active sessions
- `/status oracle-MyProject` → captures session output
- `/patrol` → runs health check

Done signals are automatically posted back to the chat as workers complete.

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
