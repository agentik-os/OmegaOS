# AISB — System Architecture

> Complete technical architecture of the AISB ecosystem.
> Single source of truth for how every component connects.
> Last updated: 2026-05-26 (Wave-4 intent-loop edition)

---

## Intent-Loop Pipeline (Wave-4, 2026-05-26)

Closed-loop autonomous execution: Intent → Decompose → Execute → Verify Runtime → Measure Delta → Loop if gap → Deliver + Proof.

3 new components close the loop between "what was asked" and "what was delivered":

| Component | Script | Stage | What it does |
|-----------|--------|-------|-------------|
| **Intent Parser** | `~/.aisb/lib/intent-parser.sh` | START (dispatch) | Transforms raw text → structured JSON: action, target, success_criteria, verify_commands |
| **Intent Verifier** | `~/.aisb/lib/intent-verifier.sh` | END (close-gate) | Compares intent JSON vs actual outcome: runs verify_commands (60%) + LLM delta assessment (40%) |
| **Deterministic Decomposition** | Oracle R-28 + `/plan-decompose` | PLAN | COMPLEX+ missions MUST use skill-based decomposition, not oracle improvisation |

**Pipeline flow:**
```
GARETH (text) → intent-parser.sh (JSON) → dispatch → worker (has full intent)
                                                        ↓
GARETH ← proof ← archive ← APPROVED ← intent-verifier ← mission-auditor ← worker .done.json
                              ↑ loop if delta < 75      ↓
                              └── REJECTED ← gap analysis → nudge worker → retry
```

**Toggle envs:** `INTENT_PARSE_ENABLED=0` (skip parsing), `INTENT_VERIFY_ENABLED=0` (skip delta check).

---

## Skill Orchestration Layer (Wave-3, 2026-05-16)

11 skills now wire the 4-level chain. Every junction (Telegram intent, oracle dispatch, worker dispatch, worker contract, oracle contract, stall recovery, done digest, Telegram format, close-gate audit, mission classification, plan decomposition) is a versioned skill instead of an ad-hoc f-string. See `SAFETY-MESH.md` §Skill Orchestration Layer for the wiring matrix and toggle envs. Patrol = 4 hits, AISB handlers/prompts = 21 hits, mission-auditor = 4 hits.

**KAIROS is OBSOLETE** as of W9 — `_nudge_oracle` is now a no-op. Primary stall recovery is `/resurrect` (4-tier cascade) triggered by `tracking-reactor.sh` (event-driven, inotify-based, 129ms latency) with `oracle-shadow.sh` as cold-path safety net.

---

---

## Overview

AISB (AI Super Brain) is a 4-level autonomous AI orchestration platform running on a dedicated VPS. Gareth communicates exclusively through Telegram. AISB routes all work to the appropriate layer, collects results, and reports back. Gareth never talks to agents directly.

```
+-----------------------------------------------------------+
|                    GARETH (Telegram)                       |
|         DM = general  |  Topic = project-specific         |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
|              LEVEL 1: AISB (The Brain)                    |
|  Bot Telegram (systemd) - Claude SDK - Router - Memory    |
|  NEVER codes in projects - Routes everything              |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
|           LEVEL 2: ORACLES (Project Managers)             |
|  ON-DEMAND tmux sessions - No limit on concurrent - Lazy mode   |
|  Analyze - Decompose - Create work sessions - Report      |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
|           LEVEL 3: WORK SESSIONS (Workers)                |
|  Ephemeral tmux sessions - {Project}-1, {Project}-2...    |
|  Execute code - Run slash commands - Deploy                |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
|           LEVEL 4: TEAMS & AGENTS (The Army)              |
|  281 agents - 130+ skills - /team /debugaudit              |
|  3-6 parallel agents per /team spawn                      |
+-----------------------------------------------------------+
```

---

## Infrastructure

### VPS

| Field | Value |
|-------|-------|
| **IP** | 72.61.197.216 |
| **SSH Port** | 42820 |
| **User** | hacker |
| **Shell** | ZSH |
| **OS** | Linux (systemd) |
| **Projects** | /home/hacker/VibeCoding/ |

### Telegram Bot

| Field | Value |
|-------|-------|
| **Entry point** | `~/VibeCoding/agentic/agentik-monitor/bot/main.py` (imports `aisb/` package) |
| **Structure** | 21 modules in `aisb/` package: config, state, formatting, auth, voice, sessions, clients, prompts, claude_runner, telegram_utils, tmux_dispatch, process_prompt, handlers, commands, oracle_commands, account, aisb_analysis, monitor, routines, intelligence, app |
| **Service** | `aisb-bot.service` (systemd, auto-restart) |
| **SDK** | Claude SDK (stream-json, 200 max turns, infinite timeout) |
| **Group** | `-1003587170167` (Forum with topics) |
| **Parse mode** | HTML (Design C blockquote cards) |
| **Features** | Voice (Whisper), documents, reactions, inline keyboards |

### Claude Code

| Field | Value |
|-------|-------|
| **Binary** | `~/.local/bin/claude` |
| **Mode** | `--dangerously-skip-permissions` (all sessions) |
| **Teammate mode** | tmux |
| **Global config** | `~/CLAUDE.md` |
| **Rules** | `~/.claude/rules/` (always loaded) |
| **Settings** | `~/.claude/settings.json` |

### External Services

| Service | Details |
|---------|---------|
| **Convex** | different-hound-874.eu-west-1 (Cloud Brain) |
| **Vercel** | Dashboard deploy (always `--token`, headless VPS) |
| **MCP Servers** | agentik-cloud-brain, composio-instagram |
| **OAuth** | `~/.aisb/lib/claude-oauth.sh` |

---

## Level 1: AISB Bot (The Brain)

### Role
- **Single gateway** between Gareth and 281 agents
- Routes Telegram messages to the right Oracle/agent
- NEVER writes code in projects -- router + coordinator ONLY
- Maintains persistent memory (MEMORY.md) and personality (SOUL.md)

### Message Flow

```
Telegram message received
    |
    +-- Reply to AISB report? -> Look up _report_message_map[msg_id]
    |     +-- Found project -> dispatch to that project's oracle
    |
    +-- DM? -> detect_multi_project (word boundary regex)
    |     +-- Multi-project (2+) -> parallel oracle dispatch
    |     +-- Single project -> auto-dispatch to oracle
    |     +-- No keyword -> AISB answers directly (CCSDK session)
    |
    +-- Direct oracle command? (/dent, /causio, /kommu, /agkt, etc.)
    |     +-- enhance_prompt -> dispatch to specific oracle tmux
    |
    +-- Topic message? -> Identify project from topic_id
            |
            +-- projects.json maps topic_id -> oracle session
            +-- enhance_prompt reformulates with project context
            +-- dispatch-to-session.sh creates oracle tmux (on-demand)
            |
            +-- Report pipeline:
                  1. Oracle writes /tmp/aisb-oracle-result-{project}.md
                  2. oracle_result_watcher (3s poll) detects file
                  3. Direct Telegram send via markdown_to_telegram_html
                  4. message_id tracked for reply routing
```

### Routing Table

| Gareth says... | AISB routes to... |
|----------------|-------------------|
| Technical task (fix, build, code) | ORACLE -> CTO -> dev-lead -> specialist |
| Code audit / quality check | ORACLE -> SERAPH (15-agent pipeline) |
| Research topic | ORACLE -> NIOBE (parallel research) |
| Plan / architecture | ORACLE -> KEYMAKER (DAG + milestones) |
| System health / status | ORACLE -> NEO + ZION |
| Marketing / content / blog | ORACLE -> CMO -> marketing-lead -> specialist |
| Strategy / pricing / analytics | ORACLE -> CPO -> strategy-lead -> specialist |
| Design / UI / branding | ORACLE -> CMO -> creative-lead -> specialist |
| Complex multi-department | ORACLE -> CEO -> coordinates C-Level |
| Simple question | AISB answers directly (no routing) |

### Files AISB Can Edit

Only these -- nothing else:
- `MEMORY.md` -- persistent user memory
- `SOUL.md` -- personality config
- `bot/main.py` + `bot/aisb/` package (22 modules) -- its own code
- `~/.aisb/` configs -- infrastructure configs

---

## Level 2: Oracles (Project Managers)

### Concept

Oracles are **on-demand, not persistent**. They are created when a user sends a message and destroyed when work is complete. **No limit on concurrent oracles.** Oracles are **managers, not coders**. They analyze tasks, decompose them into sub-tasks, create work sessions, monitor progress, and report results.

### Lazy Mode (since 2026-03-30)

- **Zero sessions at boot** -- no oracles start when the VPS/bot restarts
- Oracles are spawned **on-demand** when Gareth sends a message targeting a project
- Bot dispatches to oracle tmux via `~/.aisb/lib/dispatch-to-session.sh`
- **No limit** on concurrent oracle sessions
- When work completes, oracle closes work sessions + runs `/debugaudit` verification, then `/exit`
- **Session registry**: `/tmp/aisb-sessions.json` -- single source of truth for all active sessions, updated every 30s by monitor loop

### Available Oracles (18 projects, spawned on-demand, no limit)

> Auto-generated from `~/VibeCoding/agentic/agentik-monitor/bot/projects.json`. Sorted by topic ID.

| Oracle Session | Project | Topic ID | Path | Icon |
|---------------|---------|----------|------|------|
| oracle-DentistryGPT | DentistryGPT | 27 | ~/VibeCoding/clients/DentistryGPT | 🦷 |
| oracle-Causio | Causio | 28 | ~/VibeCoding/clients/Causio | ⚖️ |
| oracle-Loumna | Loumna | 29 | ~/VibeCoding/clients/loumna | 🌖 |
| oracle-L34D | L34D | 30 | ~/VibeCoding/work/nownownow | 📈 |
| oracle-Kommu | Kommu | 31 | ~/VibeCoding/work/kommu | 💬 |
| oracle-AgentikOS | AgentikOS | 32 | ~/VibeCoding/work/agentik-os-site | 🤖 |
| oracle-AgentikMonitor | AgentikMonitor | 293 | ~/VibeCoding/agentic/agentik-monitor/dashboard | 📊 |
| oracle-OneLife | OneLife | 303 | ~/VibeCoding/1-life | 🧬 |
| oracle-GlutenLibre | GlutenLibre | 3276 | ~/VibeCoding/clients/Gluten-Libre | 🌾 |
| oracle-CAIO-Academy | CAIO-Academy | 4968 | ~/VibeCoding/work/CAIO-Academy | 🎓 |
| oracle-CAIO | CAIO | 5071 | ~/VibeCoding/1-life/05-business | 👔 |
| oracle-GTA6 | GTA6 | 10924 | ~/VibeCoding/work/GTA6 | 📦 |
| oracle-OmegaVPS | OmegaVPS | 13036 | ~/ | Ω |
| oracle-OmegaGitHub | OmegaGitHub | 13075 | ~/VibeCoding/agentic/omega | Ω |

### Direct Oracle Commands

Telegram commands and aliases are derived from `projects.json` at bot startup. To see the live list, run `/help` in Telegram or inspect `BotCommands` registered by `bot/commands.py`.

### DM Auto-Dispatch

- `detect_multi_project` uses word boundary regex matching
- Single project keyword -> auto-dispatch to that oracle
- Multiple projects (2+) -> parallel dispatch to all matching oracles
- No project keyword -> AISB answers directly (CCSDK session)
- Project keywords: only exact names + aliases (no aggressive substrings)

### Reply-Based Project Routing (since 2026-03-30)

When AISB sends a report about a project, it tracks `message_id -> project_name` in `_report_message_map`. When Gareth replies to that report, AISB auto-routes the reply to the correct project's oracle. This enables concurrent multi-project conversations in DM.

### Oracle Rules (Absolute)

1. **NEVER** use the Agent tool or spawn internal sub-agents
2. **NEVER** launch /team, /debugaudit, or slash commands themselves
3. **NEVER** edit code in their own session
4. **ALWAYS** create separate tmux sessions ({Project}-N)
5. **ALWAYS** send prompts via load-buffer/paste-buffer (reliable for long text)
6. **ALWAYS** monitor with tmux capture-pane
7. **ALWAYS** write signal file `/tmp/aisb-oracle-result-{project}.md` when done
8. **ALWAYS** verify directory with `pwd` before any action (directory safety guard)
9. **ALWAYS** kill worker sessions when they finish (no orphan tmux sessions)
10. **ALWAYS** `/exit` after writing signal file (auto-cleanup)

### Oracle System Prompt

Generated per-project by `~/.aisb/lib/oracle-prompt.sh`:

```bash
~/.aisb/lib/oracle-prompt.sh {PROJECT_NAME} {PROJECT_PATH} {WORK_SESSION}
# Output: ~/.aisb/prompts/oracle-{PROJECT}.md
```

Injected at oracle boot via:
```bash
claude --dangerously-skip-permissions --append-system-prompt-file {prompt_file}
```

The system prompt contains:
- **RULE #1**: Dispatch for code changes, read directly for analysis
- `/team` dispatch rules (ALWAYS /team for implementation)
- `/debugaudit` mandatory verification loop
- Signal file template with exact format
- GOD MODE instructions (planner + phase execution)
- Session cleanup rules (kill workers, kill verify, /exit)
- Worker cleanup: `for s in $(tmux list-sessions | grep '^{Project}-'); do tmux kill-session -t "$s"; done`

When an oracle is idle and re-dispatched: exit + relaunch with fresh system prompt.

### Oracle Signal File (CRITICAL)

When an oracle finishes work, it MUST write a report to `/tmp/aisb-oracle-result-{project}.md`. This is how AISB knows the oracle is done and triggers the report pipeline to Gareth.

Format:
```markdown
# Oracle Report -- {Project}
PROJECT:{name}
STATUS:DONE/FAILED
BUILD:PASS/FAIL
## Resume
[2-5 lines of what was done]
## Not Done
[what remains]
## Next Steps
[suggestions]
```

Fallback: if the oracle forgets, the monitor loop detects the working->idle transition and writes the file automatically.

### Oracle 5-Step Workflow

```
ETAPE 1: ANALYSE   -- cat CLAUDE.md, decompose, define success criteria
ETAPE 2: DISPATCH  -- ~/.aisb/lib/dispatch-to-session.sh {Project}-N '/team [prompt]' {path}
ETAPE 3: MONITORING -- tmux capture-pane every 30s
ETAPE 4: CLOSE + VERIFY -- kill workers + dispatch /debugaudit verification
ETAPE 5: VERIFICATION GATE -- npm build, git commit+push, write result file, /exit
```

### The Full Chain

```
Gareth (Telegram)
  -> AISB Bot (/command or reply to report)
    -> enhance_prompt: AISB Brain reformulates + adds git/project context
    -> _build_oracle_dispatch_prompt: concise wrapper (project, path, signal)
    -> Oracle tmux (on-demand, no limit)
      -> dispatch-to-session.sh
        -> Work Session tmux + /team
          -> Claude Code agents
        <- .oracles/{session}.md report
      <- Oracle monitors, closes, /debugaudit verifies
      <- Oracle writes /tmp/aisb-oracle-result-{project}.md
    <- oracle_result_watcher (3s poll) detects signal file
    <- Direct Telegram send via markdown_to_telegram_html (no SDK)
    <- Tracks message_id -> project for reply routing
  <- AISB -> Gareth (Telegram DM)
    -> Gareth replies to report -> auto-routed to same oracle
```

### N+1 Intelligence Pipeline

```
Gareth: "fix le login"           (casual, 15 chars)
    |
enhance_prompt (AISB Brain):     reformulates + git log + branch + files
    |
_build_oracle_dispatch_prompt:   adds project path, session name, signal file
    |
Oracle receives:                 structured prompt with full context (~1000 chars)
    |
Oracle dispatches workers:       /team with success criteria
    |
Workers execute + report         .oracles/{session}.md
    |
Oracle verifies + writes         /tmp/aisb-oracle-result-{project}.md
    |
oracle_result_watcher:           3s poll detects file, sends via Telegram
    |
Gareth receives:                 "Login fixe. Token refresh avec retry 3x..."
```

---

## AISB Brain (Prompt Reformulation)

### enhance_prompt

Transforms casual Gareth messages into structured oracle prompts using Claude Code.

| Aspect | Detail |
|--------|--------|
| **Method** | `claude --print` via stdin pipe |
| **Auth** | `ANTHROPIC_API_KEY=""` forces OAuth (Claude Max, unlimited) |
| **Isolation** | Each reformulation = fresh process, zero context leaking between projects |
| **Input saved** | `.oracles/aisb-reformat-input.md` |
| **Output saved** | `.oracles/aisb-reformat-output.md` |
| **Timeout** | 60s |
| **Fallback** | If Claude fails, falls back to structured template |

### What it produces

Reformulates casual messages into professional oracle prompts containing:
- **Technical objective** -- clear description of what needs to happen
- **Files to touch** -- specific paths based on git log/branch analysis
- **Success criteria** -- measurable outcomes
- **Technical context** -- relevant git state, branch, recent changes

### Example

```
Input:  "fix le login"
Output: "## Objectif technique
         Corriger le flux d'authentification login.
         ## Fichiers concernes
         src/auth/login.ts, src/middleware.ts
         ## Criteres de succes
         - Login flow works end-to-end
         - npm run build = 0 errors
         ## Contexte
         Branch: main, last commit: fix token refresh"
```

---

## Report Pipeline

### Flow

```
Oracle writes /tmp/aisb-oracle-result-{project}.md
    |
oracle_result_watcher (3s poll interval) detects file
    |
Direct Telegram send via markdown_to_telegram_html (no SDK reformulation)
    |
message_id -> project tracked in _report_message_map
    |
Gareth replies to report -> auto-dispatch to that project's oracle
```

### Details

- **Watcher**: `oracle_result_watcher` polls every 3 seconds for new signal files
- **Send method**: Direct Telegram API via `markdown_to_telegram_html` -- NOT Claude SDK
- **Reply routing**: `_report_message_map[message_id] = project_name`
- **Monitor fallback**: If oracle forgets to write signal, monitor detects idle state and writes it automatically
- **Worker cleanup**: Workers killed by oracle after report, or by watcher after signal sent

---

## God Mode

### Entry Points

| Command | Behavior |
|---------|----------|
| `/godmode <project> <goal>` | AISB dispatches to project oracle in god mode |
| `/godmode <goal>` (free mode) | Presents project buttons, user picks |

### State Machine

```
WORK -> (STATUS:DONE detected) -> VERIFYING -> (/debugaudit ok) -> DONE
  ^                                   |
  +---- (errors found) ---------------+
```

### AISB God Mode

- State persisted to `~/.aisb/status/godmode-sessions.json`
- Progress log tracks previous iterations
- Pre-check: "Not Done" items -> CONTINUE (don't ask, just keep going)
- After STATUS:DONE -> ask /debugaudit once -> next report = GOAL_ACHIEVED
- `godmode_evaluate()` in `intelligence.py` manages transitions

### Oracle God Mode

When oracle prompt contains "GOD MODE":

1. **READ**: cat CLAUDE.md + Vision/*.md
2. **PLAN**: /planner (25 tasks/phase, description 80+ chars, 5 mandatory fields)
3. **EXECUTE** phase by phase:
   a) Dispatch each task via dispatch-to-session.sh
   b) Monitor workers
   c) KILL ALL worker sessions when done
   d) Verify /debugaudit, kill verify session
   e) npm run build, git commit
   f) Next phase
4. **FINISH**: Write signal file + /exit

### Fuzzy Project Matching

- Skips leading punctuation (handles `/godmode /Kommu fix auth`)
- Contains match (case-insensitive)
- Falls back to button selection if ambiguous

---

## Level 3: Work Sessions (Workers)

### Concept

Ephemeral tmux sessions created by Oracles. Each work session runs Claude Code and executes actual code changes.

### Naming Convention

| Pattern | Purpose | Example |
|---------|---------|---------|
| `{Project}-N` | Work session N for project | Kommu-1, Kommu-2 |
| `{Project}-{task}` | Named task session | DentistryGPT-fix-auth |
| `{Project}-verify` | /debugaudit verification session | Kommu-verify |
| `{Project}-fix` | Fix session after /debugaudit errors | Kommu-fix |
| `Home-{topic}` | Non-project work | Home-setup-cron |
| `AISB-{AGENT}` | Dedicated agent session | AISB-ORACLE, AISB-SERAPH |

### Available Slash Commands

| Command | Purpose | When |
|---------|---------|------|
| `/team [task]` | Senior parallel team (3-6 agents) | Default for complex tasks |
| `/codeaudit` | 23-phase forensic code audit | Deep code quality |
| `/flowaudit` | 20-phase user flow forensics | User journey audit |
| `/uiuxaudit` | Art Director design forensics | Design coherence |
| `/refontaudit` | Senior dashboard refonte (22 phases, /440) | Redesign to Linear/Vercel/Stripe level |
| `/debugaudit [scope]` | 18-phase runtime bug hunting | Before production (replaces /hunt, /maniac, /xoxo) |
| `/featureaudit` | 16-phase feature completeness | Feature gap analysis |
| `/perfaudit` | Performance forensics (18 phases) | Speed issues |
| `/secaudit` | Security forensics (OWASP) | Security review |
| `/uiuxaudit` | Design coherence audit | UI review |
| `/automationaudit` | 22-phase automation infrastructure | Cron, scripts, daemons, dispatch chains |
| `/logicaudit` | 20-phase systems logic architect | Optimization, architecture, performance |
| `/build` | Build + deploy pipeline | Ship to prod |
| `/planner` | DAG-based step planning | Before complex features |
| `/godmode [task]` | Full autonomy with heartbeat | Multi-hour missions |

---

## Level 4: Teams & Agents (The Army)

### 12 Core Agents (Matrix-Themed)

| Agent | Role | Model | Tier |
|-------|------|-------|------|
| **ORACLE** | Task classification & routing | Opus | Core |
| **MORPHEUS** | Execution & coordination | Opus | Core |
| **KEYMAKER** | Implementation planning | Sonnet | Core |
| **SERAPH** | Quality gates & verification | Sonnet | Core |
| **SMITH** | Self-improvement & learning | Sonnet | Specialist |
| **NIOBE** | Deep research & intelligence | Sonnet | Specialist |
| **ARCHITECT** | System design & infrastructure | Sonnet | Specialist |
| **MEROVINGIAN** | Knowledge consolidation | Haiku | Support |
| **NEO** | Session monitoring & health | Haiku | Support |
| **ZION** | Metrics dashboard & status | Haiku | Support |
| **LINK** | Communication relay | Haiku | Support |
| **CONSTRUCT** | Environment setup & tools | Haiku | Support |

### C-Level & Department Hierarchy

```
CEO -- Strategic coordination (all 281 agents)
+-- CTO -- Technical (117 agents: dev + QA + security)
|   +-- dev-lead (44 agents)
|   +-- qa-lead (35 agents)
|   +-- security-lead (29 agents)
+-- CMO -- Marketing + Creative (43 agents)
|   +-- marketing-lead (28 agents)
|   +-- creative-lead (15 agents)
+-- CPO -- Strategy & Analytics (32 agents)
    +-- strategy-lead (32 agents)
```

### Agent Files

| Level | Path | Count |
|-------|------|-------|
| AISB agents | `~/.claude/agents/AISB/` | 13 |
| C-Level | `~/.claude/agents/c-level/` | 4 |
| Leads | `~/.claude/agents/leads/` | 6 |
| Development | `~/.claude/agents/development/` | 44 |
| Quality | `~/.claude/agents/quality/` | 35 |
| Security | `~/.claude/agents/security/` | 29 |
| Marketing | `~/.claude/agents/marketing/` | 28 |
| Creative | `~/.claude/agents/creative/` | 15 |
| Strategy | `~/.claude/agents/strategy/` | 32 |
| Root specialists | `~/.claude/agents/*.md` | 44 |

---

## Session Architecture

### Tmux Session Types

| Type | Persistence | Creator | Purpose |
|------|------------|---------|---------|
| `Home` | Persistent | AISB master | Main AISB session (Telegram bot) |
| `oracle-{Project}` | On-demand (no limit) | dispatch-to-session.sh | Project management |
| `{Project}-N` | Ephemeral | Oracle | Code execution |
| `{Project}-verify` | Ephemeral | Oracle | /debugaudit verification |
| `{Project}-fix` | Ephemeral | Oracle | Post-debugaudit fixes |
| `AISB-{AGENT}` | Ephemeral | spawn-agent.sh | Dedicated agent tasks |
| `Home-{topic}` | Ephemeral | AISB | Non-project work |

### Session Registry

**`/tmp/aisb-sessions.json`** -- single source of truth for all active sessions.
- Updated every 30s by the monitor loop
- Tracks oracle + worker sessions with status, start time, project
- Bot reads this to enforce oracle session tracking
- "Back Online" message displays active tmux sessions + project emojis

### Session Lifecycle

```
Oracle: boot with system prompt
  -> dispatch /team workers
  -> monitor workers (capture-pane every 30s)
  -> kill worker sessions
  -> dispatch /debugaudit verification
  -> gate (build, deploy, push)
  -> write signal file
  -> /exit (self-cleanup)

Workers: ephemeral tmux
  -> killed by oracle when done
  -> OR killed by watcher after report sent

Auth: NO auto-reconnect on 401 (user decides via /next)

/stop: kills SDK, godmode, oracles, workers, chrome, node
/kill: full cleanup (tmux, processes, temp files, godmode, RAM cache)
```

### Critical Rules

| Rule | Why |
|------|-----|
| AISB master NEVER codes | All work goes through Oracles for visibility |
| NEVER create Home-{Project} | Bypasses Oracle, loses topic trace |
| ALWAYS post in Telegram topic | Gareth uses topics as project management |
| Multi-project = parallel Oracles | oracle-Kommu + oracle-L34D simultaneously |
| Prompts via load-buffer | Reliable for long text (send-keys truncates) |

---

## tmux-project Menu

Interactive project management menu with submenus:

| Submenu | Key | Actions |
|---------|-----|---------|
| **Oracle** | `o` | Open oracle, god mode, view output, kill |
| **Dev** | `d` | Open dev session, run build, deploy |
| **Init** | `i` | Initialize project (CLAUDE.md, git, dirs) |
| **Notif** | `n` | Notification settings |
| **Quit** | `q` | Exit menu |

Removed (deprecated): Background, Usage, Clean, Nuclear.

---

## Communication Protocol

### Telegram Formatting (Design C Blockquote Cards)

```html
<b>AISB  >  {Report Title}</b>
<code>{date}  -  {time} UTC  -  {context}</code>

<blockquote>{icon} {Name}  -  <code>{id}</code>
{details}  ->  <b>{RESULT}</b></blockquote>

{emoji}  <b>{summary stats}</b>  -  {extra}  -  {count}
```

### Notification Types

| Type | When | Format |
|------|------|--------|
| Lance | Task dispatched to oracle | 1 line: "{icon} {project} -- lance" |
| Termine | Oracle/work session done | Resume + next steps + buttons if decision needed |
| Decision | Input needed | Clear question + 2-3 InlineKeyboard buttons |

### Hub-and-Spoke Model

- Workers report to Oracle only
- Oracle aggregates and reports to AISB
- Cross-functional coordination via Oracle
- Emergency escalation directly to AISB
- Prevents n-squared communication complexity

---

## Planner v6.0

### Specification

| Field | Value |
|-------|-------|
| **Max tasks per phase** | 25 |
| **Phases** | Unlimited (up to 1500+ total tasks) |
| **Task fields** | 5 mandatory: description (80+ chars), files, criteria, dependencies, category |
| **Execution** | ABSOLUTE sequential (never skip) |
| **Input** | Vision/PRD files |
| **Integration** | Oracle dispatches each task via /team workers |

### Flow

```
/planner reads Vision/*.md
  -> Generates DAG with phases
  -> Each phase: max 25 tasks
  -> Each task: description 80+ chars, files, criteria
  -> Oracle executes phase by phase via /team dispatch
  -> Build + verify between phases
```

---

## SMITH Auto-Analyzer

### Purpose

Self-improvement system that detects recurring failures and proposes fixes.

| Field | Value |
|-------|-------|
| **Script** | `~/.aisb/lib/smith-analyze.py` |
| **Input** | `~/.aisb/smith/learnings.jsonl` (1000+ entries) |
| **Output** | `~/.aisb/smith/proposals.md` |
| **Patterns** | `~/.aisb/smith/patterns.json` |
| **Schedule** | Cron every 4h |
| **Testchain** | Step 11 runs SMITH analysis |

### Detection Patterns

- Oracle no-dispatch (oracle received task but didn't create workers)
- Missing signals (oracle finished but never wrote result file)
- Slow responses (time from dispatch to report > threshold)
- Recurring errors (same error category across multiple sessions)

### Flow

```
1. READ:    Parse all learnings from JSONL
2. PATTERN: Detect recurring failures, slow responses, missed dispatches
3. PROPOSE: Generate actionable improvements with severity + evidence
4. LOG:     Write proposals to proposals.md
5. APPLY:   Flag critical patterns for immediate action
```

---

## KAIROS (Tick Engine)

### Purpose

Background tick engine running system health and notifications.

| Field | Value |
|-------|-------|
| **Interval** | 60 seconds |
| **Health checks** | Disk, RAM, load average |
| **Oracle detection** | Result file monitoring |
| **Buddy mood** | Periodic mood updates |
| **Notifications** | Sent to GROUP (not DM) |

---

## /testchain

### Purpose

Full end-to-end test of the AISB -> Oracle -> /team -> Report -> Telegram chain.

### Quick Checks (22 tests, instant, no tokens)

| Test | Validates |
|------|-----------|
| Bot health | `.health` file freshness (<120s) |
| Prompt structure | `_build_oracle_dispatch_prompt` output format |
| Oracle system prompt | 7 sections: never_code, dispatch, team, debugaudit, signal, kill, godmode |
| tmux paste newlines | load-buffer preserves newlines |
| Dispatch script | `dispatch-to-session.sh` exists + executable + uses paste-buffer |
| Oracle no flatten | `_send_to_oracle` doesn't flatten newlines |
| Result file detection | `/tmp/aisb-oracle-result-*.md` glob works |
| Reply routing | `_report_message_map` + handler integration |
| Godmode state machine | work -> verifying -> done transitions |
| DM auto-dispatch | `detect_multi_project` matches all projects |
| Memory dirs | 4 dirs exist: user, project, local, agent-memory |
| All projects oracle prompt | Every project generates valid prompt (>500 chars) |

### Interactive E2E (10 steps, uses real tokens)

| Step | Action |
|------|--------|
| 1 | Create test project (FullTest in /tmp) |
| 2 | Generate oracle system prompt |
| 3 | Boot oracle with system prompt |
| 4 | AISB enriches + builds dispatch prompt |
| 5 | Send dispatch to oracle |
| 6 | Wait for oracle to dispatch worker |
| 7 | Wait for worker to create result file |
| 8 | Verify signal file written |
| 9 | Verify reply routing tracked |
| 10 | Cleanup (kill sessions, remove temp files) |

Results logged to SMITH learnings for pattern analysis.

---

## /newproject

### Command

```
/newproject <name> <clients|work> [emoji]
```

### What It Creates

| Item | Detail |
|------|--------|
| **Project directory** | `~/VibeCoding/{clients\|work}/{name}/` |
| **CLAUDE.md** | Project-specific instructions |
| **Vision/VISION.md** | Product vision document |
| **.planner/** | Planner task directory |
| **.oracles/** | Oracle report directory |
| **.gitignore** | Standard ignore patterns |
| **git init** | Initialize repository |
| **Telegram topic** | Created automatically in group |
| **projects.json** | Entry added with topic_id, path, icon |
| **Oracle prompt** | Generated via oracle-prompt.sh |
| **tmux aliases** | Added to shell config |

---

## Projects (18)

> Auto-generated from `~/VibeCoding/agentic/agentik-monitor/bot/projects.json`. Sorted by topic ID.

| Icon | Project | Path | Topic |
|------|---------|------|-------|
| 🦷 | DentistryGPT | ~/VibeCoding/clients/DentistryGPT | 27 |
| ⚖️ | Causio | ~/VibeCoding/clients/Causio | 28 |
| 🌖 | Loumna | ~/VibeCoding/clients/loumna | 29 |
| 📈 | L34D | ~/VibeCoding/work/nownownow | 30 |
| 💬 | Kommu | ~/VibeCoding/work/kommu | 31 |
| 🤖 | AgentikOS | ~/VibeCoding/work/agentik-os-site | 32 |
| 📊 | AgentikMonitor | ~/VibeCoding/agentic/agentik-monitor/dashboard | 293 |
| 🧬 | OneLife | ~/VibeCoding/1-life | 303 |
| 🌾 | GlutenLibre | ~/VibeCoding/clients/Gluten-Libre | 3276 |
| 🎓 | CAIO-Academy | ~/VibeCoding/work/CAIO-Academy | 4968 |
| 👔 | CAIO | ~/VibeCoding/1-life/05-business | 5071 |
| 📦 | GTA6 | ~/VibeCoding/work/GTA6 | 10924 |
| Ω | OmegaVPS | ~/ | 13036 |
| Ω | OmegaGitHub | ~/VibeCoding/agentic/omega | 13075 |

All project configuration is dynamic from `projects.json` -- BotCommands, keywords, shortcuts, oracle prompts generated from this single file.

---

## File System Map

### Core Documentation (Auto-Loaded)

| File | Purpose | Location |
|------|---------|----------|
| `CLAUDE.md` | Global system reference | `~/CLAUDE.md` |
| `SOUL.md` | AISB personality & identity | `~/VibeCoding/agentic/agentik-monitor/bot/SOUL.md` |
| `MEMORY.md` | Persistent user memory | `~/VibeCoding/agentic/agentik-monitor/bot/MEMORY.md` |
| `ARCHITECTURE.md` | This file -- system architecture | `~/.aisb/docs/ARCHITECTURE.md` |
| `ORCHESTRATION.md` | Complete orchestration reference | `~/.aisb/docs/ORCHESTRATION.md` |
| `CLOUD.md` | Cloud & infrastructure details | `~/.aisb/docs/CLOUD.md` |

### Infrastructure Scripts

| Script | Purpose | Location |
|--------|---------|----------|
| **dispatch-to-session.sh** | **THE dispatch method** -- creates tmux session + Claude boot + paste + Enter. Persists prompt to `brief-<session>.txt` (Layer 1 brief-replay). CPU throttle at top of file (Layer 2B). | `~/.aisb/lib/` |
| **oracle-prompt.sh** | Generates per-project oracle system prompt | `~/.aisb/lib/` |
| **smith-analyze.py** | SMITH auto-analyzer (patterns, proposals) | `~/.aisb/lib/` |
| **safe-npm-build.sh** | Global build mutex (Layer 2A) — `flock` on `/tmp/aisb-locks/global-next-build.lock`, 30-min timeout. Workers MUST use this instead of `npm run build`. | `~/.aisb/lib/` |
| **dispatch-queue-flusher.sh** | Re-dispatches throttled queue entries when load < 2× cores (cron `*/2`, Layer 2B′). | `~/.aisb/lib/` |
| **oracle-shadow.sh** | Shadow Manager (Layer 3) — 14 signals, Tier 1/2/3, oracle/worker asymmetry, kill-switch | `~/.aisb/lib/` |
| **oracle-observer.sh** | Shadow observer wiring (cron `*/3`) — runs M1–M6 then calls oracle-shadow.sh | `~/.aisb/lib/` |
| **mission-auditor.sh** | Phase-END quality gate (Layer 4) — classifies, runs 1–3 Quality Arsenal audits, ≥85/100 gate | `~/.aisb/lib/` |
| **close-gate.sh** | `ack-worker` hook invokes mission-auditor before writing `.acked.json` | `~/.aisb/lib/` |
| **worker-mark-done.sh** | Atomic `.done.json` writer + self-kill (workers ONLY) | `~/.aisb/lib/` |
| **worker-todo-init.sh / -update.sh** | File-based progress mirrors for patrol + oracle visibility | `~/.aisb/lib/` |
| dispatch-to-oracle.sh | Legacy send task to oracle (use dispatch-to-session.sh instead) | `~/.aisb/lib/` |
| spawn-agent.sh | Spawn AISB agents | `~/.aisb/team/` |
| heartbeat.sh | Agent health monitor (30s) | `~/.aisb/lib/` |
| session-resume.sh | Oracle context persistence | `~/.aisb/lib/` |
| `oracle-state-save.sh` | State snapshots | `~/.aisb/lib/` |
| `oracle-state-restore.sh` | Context injection | `~/.aisb/lib/` |
| skill-inject.sh | Runtime skill loading | `~/.aisb/lib/` |
| deploy-hook.sh | Post-deploy verification | `~/.aisb/lib/` |
| pre-restart-summary.sh | Pre-restart state capture | `~/.aisb/lib/` |
| claude-oauth.sh | OAuth reauth (no cache) | `~/.aisb/lib/` |

### Data Stores

| Store | Format | Location |
|-------|--------|----------|
| **Session registry** | JSON (30s update) | `/tmp/aisb-sessions.json` |
| **Godmode sessions** | JSON | `~/.aisb/status/godmode-sessions.json` |
| **Conversation memory** | JSON | `~/.aisb/status/conversation-memory.json` |
| **PKCE auth state** | JSON | `~/.aisb/status/` |
| Oracle prompts | Markdown | `~/.aisb/prompts/oracle-{project}.md` |
| Oracle reformat input | Markdown | `{project}/.oracles/aisb-reformat-input.md` |
| Oracle reformat output | Markdown | `{project}/.oracles/aisb-reformat-output.md` |
| Oracle result signals | Markdown | `/tmp/aisb-oracle-result-{project}.md` |
| Audit trail | JSONL append-only | `~/.aisb/audit/*.jsonl` |
| Task locks | File-based, 2h TTL | `~/.aisb/locks/*.lock` |
| Brief-replay (per dispatch) | Text | `~/.aisb/state/brief-<session>.txt` |
| Mission-audit verdicts | JSON | `~/.aisb/state/mission-audit-<session>.json` |
| Shadow oracle observations | JSONL append-only | `~/.aisb/logs/shadow-oracle-observations.jsonl` |
| Dispatch queue (CPU throttle) | JSONL | `~/.aisb/state/dispatch-queue.jsonl` |
| Heartbeats | 30s interval files | `~/.aisb/heartbeats/*.beat` |
| Session state | JSON snapshots | `~/.aisb/sessions/*.json` |
| Oracle state | JSON snapshots | `~/.aisb/state/*.json` |
| SMITH learnings | JSONL | `~/.aisb/smith/learnings.jsonl` |
| SMITH proposals | Markdown | `~/.aisb/smith/proposals.md` |
| SMITH patterns | JSON | `~/.aisb/smith/patterns.json` |
| Agent mailbox | JSONL | `~/.aisb/mailbox/*.jsonl` |
| Reply routing | In-memory | `_report_message_map` (rebuilt on new reports) |

### Project Structure

```
~/VibeCoding/
+-- work/                      Professional projects
|   +-- agentik-monitor/       AISB bot + dashboard
|   +-- agentik-os-site/       AgentikOS website
|   +-- kommu/                 Communication platform
|   +-- L34D/                  Lead analytics
|   +-- AI-GenX/               AI generation
|   +-- agkt/                  Paper trading + auto-trading
|   +-- ...
+-- clients/                   Client projects
|   +-- DentistryGPT/          Dental AI
|   +-- LawyerAI/              Causio (legal AI)
|   +-- loumna/                Music/creative
|   +-- LaSphere/              Crystal ball project
|   +-- ...
+-- 1-life/                    Personal/health (OneLife)
+-- routines/                  Automated scripts
```

### Git Configuration

| Category | Email | Projects |
|----------|-------|----------|
| AgentikOS | x@agentik-os.com | DevLensPro, L34D, Causio, Atma, AGKT, Kommu |
| DentistryGPT | cto.dentistrygpt@gmail.com | DentistryGPT |
| Gluten-Libre | tech.glutenlibre@gmail.com | Gluten-Libre |

---

## Persistence & Startup

### Lazy Mode (since 2026-03-30)

**Zero sessions at boot.** No oracle-boot cron. No earthbit cron. Oracles start on-demand only.

```
1. BOOT
   +-- systemd starts aisb-bot.service
       +-- Bot is online, zero oracle sessions running
       +-- "Back Online" message shows active tmux sessions + project emojis

2. ON-DEMAND ORACLE CREATION
   +-- Gareth sends message (DM keyword / /command / topic)
       +-- Bot creates oracle via dispatch-to-session.sh
       +-- No limit on concurrent oracles enforced

3. SESSION REGISTRY
   +-- /tmp/aisb-sessions.json updated every 30s by monitor loop
       +-- Tracks all active oracle + worker sessions

4. MONITORING
   +-- Bot monitor loop (30s interval)
       - Updates session registry
       - Sends DM + topic notifications on completion
       - Per-oracle heartbeat.sh still active
       - Bot->Convex heartbeat (30s)
```

### Persistence Components

| Component | Mechanism | Recovery |
|-----------|-----------|----------|
| Bot AISB | systemd (enabled, auto-restart) | Instant |
| Oracles | On-demand via dispatch-to-session.sh | Created when needed, no limit |
| Session registry | /tmp/aisb-sessions.json (30s update) | Rebuilt from tmux list-sessions |
| Conversation memory | ~/.aisb/status/conversation-memory.json | Persisted on every message |
| Reply routing map | In-memory (_report_message_map) | Rebuilt as new reports sent |
| Godmode sessions | ~/.aisb/status/godmode-sessions.json | Persisted on state change |
| PKCE auth state | ~/.aisb/status/ (not /tmp/) | Survives restart |
| Oracle context | session-resume.sh --resume | Context preserved |
| Oracle state | oracle-state-save/restore | Full state recovery |
| Agent health | heartbeat.sh (30s interval) | Auto-detect dead |
| Audit trail | JSONL append-only | Permanent record |
| Task locks | File-based, 2h TTL | Auto-expire |
| SMITH learnings | ~/.aisb/smith/learnings.jsonl | Permanent, analyzed every 4h |

### What Survives Restart

| Data | Persisted? | How |
|------|-----------|-----|
| Conversation context | Yes | conversation-memory.json |
| Oracle signal files | Yes | /tmp/*.md (until cleaned by /kill) |
| Project config | Yes | projects.json |
| SMITH learnings | Yes | learnings.jsonl |
| Godmode sessions | Yes | godmode-sessions.json |
| Reply routing map | No | Rebuilt as AISB sends new reports |
| Active tmux sessions | Yes | tmux survives bot restart |
| Claude Code sessions | Yes | --resume flag preserves context |

### Idle Oracle Detection (post-restart)

After bot restart, oracles may be idle without a report having been sent. The monitor handles this:
1. First scan: oracle detected as idle (no previous state)
2. Second scan (30s later): if still idle AND no result file exists -> captures output and writes signal file
3. result_watcher picks it up -> AISB reports to Gareth

---

## Automated Routines

| Routine | Schedule | Topic |
|---------|----------|-------|
| Morning Briefing | 7h daily | 1798 |
| Prod Health Check | Every 4h | 1798 |
| EOD Summary | 23h daily | 1798 |
| Git Status All | Every 6h | 1798 |
| Security Audit | Monday 10h | 1798 |
| SMITH Analysis | Every 4h | (cron) |
| KAIROS Tick | Every 60s | GROUP |
| Oracle supervisor | Every 60s | (cron) |
| Wake-on-worker-done | Every 2 min | (cron) |
| Oracle observer (M1–M6 + Shadow) | Every 3 min | (cron) |
| Dispatch queue flusher (CPU Guard) | Every 2 min | (cron) |

---

## Safety Mesh (2026-05-16)

The system enforces runtime discipline through **four independent failure surfaces**, each owning a distinct slice of the mission lifecycle. Full contracts live in `~/.aisb/docs/SAFETY-MESH.md`; this section is a pointer + quick-reference.

| Layer | Phase | Owner script | Failure surface |
|---|---|---|---|
| **1. Brief-Replay** | START | `~/.aisb/lib/dispatch-to-session.sh:621` (persists `brief-<session>.txt`) | Worker loses its prompt on rate-limit / API-error |
| **2. CPU Guard** | START + DURING | `safe-npm-build.sh` (mutex) + dispatch throttle + `dispatch-queue-flusher.sh` (cron `*/2`) + `oracle-shadow.sh` `CPU_OVERLOAD` signal | 2-core VPS saturation, `.next/` race corruption |
| **3. Shadow Manager** | DURING | `oracle-shadow.sh` (14 signals, Tier 1/2/3) + `oracle-observer.sh` (cron `*/3`) | Worker thrash, drift, idle, OOM, CPU overload |
| **4. Mission Auditor** | END | `mission-auditor.sh` (gates `close-gate.sh ack-worker` at score ≥ 85/100, retry × 2, Telegram escalation) | Worker self-marks `done_clean` with superficial verification |

### Oracle vs Worker asymmetry (Layer 3)

Workers and oracles are NOT treated symmetrically by the shadow. Prescriptive nudges that help a worker (single mission, narrow context) destroy an oracle (multi-mission manager). The asymmetry:

- **Worker** — default action is prescriptive `recovery_apply` nudge.
- **Oracle** — default action is `observe-only` (JSONL log + throttled FYI). THRASH disabled. STAGNATION floor raised to 360 min + idle confirmation. Emergency nudge uses brief-aware **question-mode**, never imperative.

Kill-switch: `~/.aisb/state/.shadow-nudge-disabled` (panic stop, both tiers).

Full asymmetry contract: `oracle-protocol.md` §Asymétrie ORACLE vs WORKER.
