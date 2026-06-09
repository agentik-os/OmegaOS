# OmegaOS — Where Everything Lives

> Quick reference: source code vs runtime data vs installed binary.

## 3 Locations You Need to Know

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. SOURCE CODE (the repo)                                       │
│    ~/Station/SideBusiness/OmegaOS/                              │
│    github.com/agentik-os/OmegaOS                                │
│    → Edit code here, push to GitHub                             │
├─────────────────────────────────────────────────────────────────┤
│ 2. INSTALLED BINARY                                             │
│    ~/.local/bin/omega                                           │
│    → The compiled Rust executable (8.9 MB)                      │
│    → Created by `cargo build --release` + `cp target/release/`  │
├─────────────────────────────────────────────────────────────────┤
│ 3. RUNTIME DATA (the master config)                             │
│    ~/.omega/                                                    │
│    → Credentials, rules, agents, skills, state, logs            │
│    → Created by install.sh, edited by `omega` commands          │
│    → THIS is what all LLMs read from                            │
└─────────────────────────────────────────────────────────────────┘
```

## Source Code Layout (`~/Station/SideBusiness/OmegaOS/`)

```
OmegaOS/
├── crates/                            ALL RUST CODE
│   ├── omega-core/src/                Core library (32 modules)
│   │   ├── account.rs                 Account management
│   │   ├── agents.rs                  LLM agent registry
│   │   ├── aisb.rs                    AISB Master spawning
│   │   ├── aisb_agents.rs             14 Matrix agents
│   │   ├── audit.rs                   23 Quality Arsenal audits
│   │   ├── bootstrap.rs               Project bootstrap pipeline
│   │   ├── credentials.rs             Multi-provider credential store
│   │   ├── dispatch.rs                Worker dispatch
│   │   ├── done.rs                    Done signal protocol
│   │   ├── formatting.rs              HTML/Markdown formatting
│   │   ├── gate.rs                    Quality gates (R-19/R-21/R-30)
│   │   ├── inbox.rs                   JSONL event queue
│   │   ├── intent.rs                  Intent parser
│   │   ├── mission.rs                 Mission tracking
│   │   ├── monitor.rs                 Billing/Telegram config
│   │   ├── oauth.rs                   OAuth flow
│   │   ├── oracle_lifecycle.rs        Oracle state machine
│   │   ├── orchestration.rs           4-level orchestration
│   │   ├── patrol.rs                  Stall detection
│   │   ├── planner.rs                 DAG-enforced planner
│   │   ├── project_manager.rs         Project CRUD
│   │   ├── providers.rs               Provider catalog
│   │   ├── router.rs                  Smart routing
│   │   ├── rubric.rs                  Success criteria
│   │   ├── rules.rs                   6 Laws + 20 Rules
│   │   ├── scope.rs                   File-lock scope claims
│   │   ├── session.rs                 rmux SDK integration
│   │   ├── ship.rs                    12-step ship pipeline
│   │   ├── skill_registry.rs          Skill discovery
│   │   ├── team.rs                    Multi-agent teams
│   │   ├── verifier.rs                Intent verification
│   │   └── ... (more)
│   │
│   ├── omega-tui/src/                 Terminal UI (ratatui)
│   │   ├── app.rs                     App state (7 tabs)
│   │   ├── input.rs                   Keyboard + mouse handling
│   │   ├── theme.rs                   Theme engine (15 palettes, Settings → Theme)
│   │   └── ui.rs                      Tab rendering
│   │
│   └── omega-cli/src/                 CLI binary
│       ├── main.rs                    40+ commands
│       └── telegram_bridge.rs         Telegram bot (Rust, was Python)
│
├── agents/                            Agent system prompts (markdown)
│   ├── aisb-master.md
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                          14 Matrix agents
│
├── rules/                             6 Laws + 20 Rules (.md)
│   ├── L1-runtime-truth.md
│   ├── R30-popper-falsification.md
│   └── ... (all of them)
│
├── skills/                            Bundled skills
│   ├── pdfgen/                        PDF generator (Next.js — see below)
│   └── audits/                        23 audit skills
│
├── docs/                              Documentation
│   ├── ARCHITECTURE.md                ← READ THIS for the full system
│   ├── ARCHITECTURE-V3.md             ← Credential architecture spec
│   ├── IMPLEMENTATION-PLAN.md
│   ├── VERIFICATION-GATE.md
│   └── reference/                     Reference materials
│       └── oauth/                     Python source for OAuth reference
│
├── tools/pdfgen/                      PDF generator (TypeScript + Next.js + Playwright)
│   ├── components/templates/          Whitepaper / Audit / Marketing / Doc
│   ├── bin/pdfgen.ts                  CLI entrypoint
│   └── package.json
│
├── config/default.toml                Default OmegaOS config (template)
├── OMEGA.md                           Universal agent prompt (deployed to ~/.omega/)
├── README.md                          Project intro
├── Cargo.toml                         Rust workspace
└── install.sh                         Install script (6 phases)
```

## Runtime Data Layout (`~/.omega/`)

```
~/.omega/                              MASTER — what all LLMs read from
├── OMEGA.md                           Copied from repo at install time
├── config.toml                        User config (editable)
├── providers.toml                     Per-provider settings
├── telegram.toml                      Telegram bot config (gitignored, secret)
├── projects.json                      Project registry
│
├── credentials/                       ALL provider credentials
│   ├── claude.json                    ← ~/.claude/.credentials.json points here
│   ├── codex.json                     ← ~/.codex/auth.json points here
│   ├── gemini.json                    ← ~/.gemini/oauth_creds.json points here
│   └── accounts/                      Saved account profiles
│
├── rules/                             6 Laws + 20 Rules (.md, synced from repo on install)
├── agents/                            19 agent prompts
├── skills/
│   ├── pdfgen/                        PDF generator
│   └── audits/                        23 audits
│
├── state/                             Runtime state (sessions, locks, done.json)
├── logs/                              Session logs
└── audit/                             Audit results
```

## Languages — 99% Rust, Justified Exceptions

| Component | Language | Why |
|-----------|----------|-----|
| **Core library** (omega-core) | **Rust** | 100% — type safety, performance |
| **TUI** (omega-tui) | **Rust** | 100% — ratatui native |
| **CLI binary** (omega-cli) | **Rust** | 100% — clap + tokio |
| **Telegram bot** (telegram_bridge.rs) | **Rust** | 100% — reqwest + long-poll (was Python) |
| **Install script** (install.sh) | Bash | Bootstrap before Rust is compiled |
| **PDF generator** (tools/pdfgen/) | TypeScript + Next.js | Playwright requires Node.js; rendering needs full DOM/CSS |
| **OAuth helper** | Bash | One-line wrapper around `claude /login` for fallback |
| Reference docs | Python (read-only) | Just documentation of the VPS Python implementation we ported |

**No Python is executed by OmegaOS.** The Python files in `docs/reference/oauth/`
exist only as reference material — they document how the VPS Python system worked
so the Rust port (`oauth.rs`, `account.rs`) could match the behavior.

## Install Flow

```
$ ./install.sh
  Phase 1: Detect OS + arch
  Phase 2: Install Rust (rustup) if missing
  Phase 3: Build rmux from github.com/agentik-os/rmux
           → ~/.local/bin/rmux
  Phase 4: Build omega CLI
           cargo build --release
           → ~/.local/bin/omega
  Phase 5: Setup ~/.omega/
           - Create credentials/, state/, logs/, accounts/
           - migrate_creds claude → moves ~/.claude/.credentials.json
             into ~/.omega/credentials/claude.json + symlink back
           - Same for codex, gemini
           - Copy OMEGA.md, agents/, rules/, skills/pdfgen/, skills/audits/
           - Run `omega rules export` and `omega sync`
  Phase 6: Shell integration
           - Add ~/.local/bin to PATH
           - Install shell completions (bash/zsh/fish)
           - Add alias: om="omega menu"
```

## Daily Usage

| Action | Command |
|--------|---------|
| Launch TUI | `omega` or `om` |
| Talk to AISB Master from Telegram | Just send a message (plain text) |
| List sessions | `/sessions` on Telegram or `omega list` |
| Switch model | `/model claude opus` |
| Manage accounts | `/account` (button menu) |
| New project | `/projects` → [+ New project] |
| Generate PDF | `omega pdf --template=audit --send` |
| Update code | edit, `cargo build --release`, `cp target/release/omega ~/.local/bin/` |

## Verification

```bash
# Source is in the right place
ls ~/Station/SideBusiness/OmegaOS/crates/

# Binary is installed
which omega && omega --version

# Runtime data exists
ls ~/.omega/credentials/

# Symlinks are correct
ls -la ~/.claude/.credentials.json   # → ~/.omega/credentials/claude.json

# Build still works
cd ~/Station/SideBusiness/OmegaOS && cargo build --release
```
