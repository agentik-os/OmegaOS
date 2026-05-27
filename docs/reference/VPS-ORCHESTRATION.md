# AISB — Orchestration Reference

> How every task flows from Gareth's Telegram message to execution and back.
> This is the operational manual. Architecture = what. Orchestration = how.
> Last updated: 2026-05-16 (Wave-3 skill-wired edition)

---

## Skill-Wired Junctions (Wave-3)

Each junction in the chain below is now backed by an invocable skill, replacing legacy f-strings / regex with versioned protocols. Best-effort with silent fallback — zero regression for obvious cases.

```
GARETH ──Telegram──▶ AISB ──tmux──▶ ORACLE ──tmux──▶ WORKER ──/team──▶ AGENTS
   intent: /classify-intent     ▲        ▲       ▲              ▲
   dispatch: /dispatch-oracle   │        │       │              │
                       /dispatch-worker  │       │              │
                            /omega-protocol      │              │
                                       /worker-protocol         │
                                       /plan-decompose          │
                                       /resurrect  (stall recovery)
                                       /audit-mission  (close-gate)
GARETH ◀──Telegram── AISB ◀──tmux── ORACLE ◀──tmux── WORKER ◀──/synthesize-report
                              /format-telegram-report
```

11 skills shipped on 2026-05-16 + 7 weakness fixes (W5 memory wired, W6 brief atomic write, W7 mission-auditor classify-intent, W8 event-driven tracking-reactor, W9 KAIROS retired, W10 /resurrect validated, W12 omega-overview.md). Full wiring matrix: `SAFETY-MESH.md` §Skill Orchestration Layer.

---

## The Golden Rule

**Gareth talks to AISB. AISB talks to Oracles. Oracles talk to Workers. Results flow back up.**

```
GARETH ──Telegram──▶ AISB ──tmux──▶ ORACLE ──tmux──▶ WORKER ──/team──▶ AGENTS
                                                                         │
GARETH ◀──Telegram── AISB ◀──tmux── ORACLE ◀──tmux── WORKER ◀──report───┘
```

Nobody skips a level. AISB never codes. Oracles never code. Workers never report to Gareth directly.

---

## Complete Request Flow

### Step-by-Step: "Fix the dashboard auth bug in Kommu"

```
1. GARETH types in Kommu topic (topic 31) on Telegram

2. AISB BOT receives message
   ├── Identifies: topic 31 → Kommu (via projects.json)
   ├── enhance_prompt: Claude SDK reformulates message + adds git context
   ├── _build_oracle_dispatch_prompt: wraps with project path + signal file
   └── Dispatches enriched prompt to oracle-Kommu tmux session

3. ORACLE-KOMMU receives task
   ├── Reads project CLAUDE.md for context
   ├── Decomposes: "auth bug" → check auth flow, find bug, fix, test
   ├── Creates tmux session: Kommu-1
   └── Sends detailed prompt via load-buffer/paste-buffer:
       "Context: Kommu auth system uses Clerk + Convex.
        Task: Find and fix the dashboard auth bug.
        Steps: 1) Check console errors 2) Trace auth flow 3) Fix 4) Test
        Success: Auth works, no console errors, /debugaudit passes
        Output: Report what changed + test results"

4. KOMMU-1 WORK SESSION executes
   ├── Reads codebase, finds the bug
   ├── Fixes the code
   ├── Runs /debugaudit auth → passes
   └── Reports results back to oracle-Kommu

5. ORACLE-KOMMU collects results
   ├── Verifies build, deploy, git push
   ├── Runs /debugaudit verification loop (fix until 0 errors)
   └── Writes /tmp/aisb-oracle-result-Kommu.md (MANDATORY signal)

6. AISB REPORT PIPELINE (automatic)
   ├── oracle_result_watcher (3s poll) detects signal file
   ├── aisb_speak_to_gareth: Claude SDK analyzes oracle output
   ├── Crafts conversational DM with summary + next steps
   ├── Tracks message_id → "Kommu" for reply routing
   └── Sends to Gareth in DM:
       "Auth bug fixé. Le token ne se rafraîchissait pas au
        changement de route. Ajouté refresh dans le middleware.
        Build: ✅  Deploy: ✅  Git: pushed.
        Tu veux que je continue sur autre chose pour Kommu?"

7. GARETH replies to the report message
   ├── AISB detects reply_to_message → looks up _report_message_map
   ├── Finds "Kommu" → routes to oracle-Kommu
   └── Cycle continues (enhance → dispatch → oracle → report → DM)
```

---

## Dispatch Mechanisms

### 1. Telegram Topic → Oracle (Primary Path)

This is the normal flow. Gareth writes in a Telegram topic.

```bash
# Bot identifies project from topic_id (projects.json)
# Dispatches via dispatch-to-session.sh (THE dispatch method):
~/.aisb/lib/dispatch-to-session.sh oracle-Kommu "Fix the dashboard auth bug" /home/hacker/VibeCoding/work/kommu
```

**dispatch-to-session.sh** does:
1. Creates the tmux session if it doesn't exist (on-demand)
2. Starts Claude Code if not running
3. Flattens the prompt (removes newlines that break Claude input)
4. Pastes via `tmux load-buffer` → `tmux paste-buffer` (reliable for long text)
5. **Sends Enter** to submit the prompt

**No limit on concurrent oracles** — bot spawns as many as needed.

### 1b. Direct Oracle Commands (Guaranteed Dispatch)

Telegram commands that always dispatch to a specific oracle:
`/dent`, `/causio`, `/loumna`, `/l34d`, `/kommu`, `/agentikos`, `/monitor`, `/onelife`, `/aigenx`

These use `dispatch-to-session.sh` under the hood.

### 2. Telegram DM → AISB Direct or Oracle (Keyword Routing)

Gareth sends a DM. AISB checks for project keywords.

```
DM "What's the status of all projects?"
  → No project keyword → AISB answers directly

DM "Fix the auth bug in Kommu"
  → Keyword "Kommu" detected → dispatch to oracle-Kommu (on-demand)

DM "Setup a new cron for backups"
  → No project keyword → AISB creates Home-setup-cron session
  → Executes task
  → Reports in DM
```

**DM keyword routing**: Mentioning ANY project name in a DM auto-routes to its oracle. Even a single project mention triggers dispatch.

### 3. Multi-Project Dispatch (Parallel)

When a task spans multiple projects, AISB dispatches to multiple Oracles simultaneously.

```
"Update the auth system in Kommu, DentistryGPT, and L34D"
  → dispatch-to-session.sh oracle-Kommu "Update auth system: [details]" /path/to/kommu
  → dispatch-to-session.sh oracle-DentistryGPT "Update auth system: [details]" /path/to/dent
  → dispatch-to-session.sh oracle-L34D "Update auth system: [details]" /path/to/l34d
  (All 3 run in parallel, no limit on concurrent = exactly at limit)
```

### 4. AISB Agent Spawn (Infrastructure Tasks)

For tasks that don't belong to a project:

```bash
~/.aisb/team/spawn-agent.sh ORACLE /home/hacker/VibeCoding "Classify this task"
~/.aisb/team/spawn-agent.sh KEYMAKER /path "Plan the migration"
~/.aisb/team/spawn-agent.sh NIOBE /path "Research competitor pricing"
```

**spawn-agent.sh** process:
1. Creates tmux session: `AISB-{AGENT_NAME}`
2. Starts Claude Code: `claude --dangerously-skip-permissions`
3. Waits 5s for Claude to boot
4. Builds context-enriched prompt (agent role + team context + task)
5. Sends via load-buffer/paste-buffer
6. Spawns background watcher (auto-cleanup on completion)
7. Sends mailbox notification when done

---

## Oracle Operations

### How an Oracle Creates Work Sessions

**ALWAYS use dispatch-to-session.sh** — never raw tmux commands:

```bash
# dispatch-to-session.sh handles EVERYTHING:
# 1. CPU throttle guard (Safety Mesh Layer 2B):
#    if 1-min load > 2.5× cores → queue to ~/.aisb/state/dispatch-queue.jsonl
#                                  + exit 0 with DEFERRED (flusher picks up at load < 2× cores).
#    Bypass: DISPATCH_FORCE=1
# 2. Creates tmux session if not exists
# 3. Starts Claude Code if not running
# 4. Flattens prompt (removes newlines)
# 5. Persists prompt to ~/.aisb/state/brief-<session>.txt (Safety Mesh Layer 1 — brief-replay)
# 6. Pastes via load-buffer + paste-buffer
# 7. Sends Enter to submit

~/.aisb/lib/dispatch-to-session.sh Kommu-1 '/team Fix the dashboard auth bug. Read CLAUDE.md first. Success: npm build = 0 errors.' /home/hacker/VibeCoding/work/kommu
```

The brief file enables Layer 3 (Shadow Manager) to replay the original instructions verbatim if the worker is hit by `RATE_LIMIT_STALL` or `API_ERROR_TRANSIENT`. The CPU throttle prevents 2-core VPS saturation when many dispatches arrive simultaneously.

### Oracle 5-Step Workflow

```
ETAPE 1: ANALYSE — cat CLAUDE.md, decompose, define success criteria
ETAPE 2: DISPATCH — ~/.aisb/lib/dispatch-to-session.sh {Project}-N '/team [prompt]' {path}
ETAPE 3: MONITORING — tmux capture-pane every 30s
ETAPE 4: CLOSE + VERIFY — tmux kill-session + dispatch /debugaudit verification
ETAPE 5: VERIFICATION GATE — npm build, convex deploy, git push, write result file
```

### Session Complete → Close + Verify (Auto)

When a work session finishes:
1. Worker writes `~/.aisb/state/worker-<session>.done.json` (status=`done_clean | pending | failed`)
2. Oracle invokes `~/.aisb/lib/close-gate.sh ack-worker <session> <oracle>`
3. **Safety Mesh Layer 4 — Mission Auditor** intercepts before ack:
   - Classifies mission type heuristically (`bug-fix | feature | ui | api | ship-post-deploy | refactor | docs | config | generic`)
   - Selects 1–3 Quality Arsenal audits via rules table
   - Runs them under global `flock /tmp/aisb-locks/mission-audit.lock` (ONE at a time VPS-wide)
   - Verdict: min score ≥ 85/100 → **APPROVED** (ack proceeds). < 85 → **REJECTED** (worker nudged, retry × 2, then Telegram escalation)
   - Bypass: `CLOSEGATE_SKIP_AUDIT=1` (emergencies, audits-themselves)
4. On APPROVED: ack writes `oracle-worker-<session>.acked.json`. Oracle may proceed to next worker or close.
5. **CLOSE** the work session: `worker-mark-done.sh` self-terminates the tmux session (5s grace).
6. **DM notification**: Monitor loop sends completion notification in DM (not just topic).

Full Mission Auditor contract: `oracle-protocol.md` §Mission Auditor; pipeline overview: `SAFETY-MESH.md` §Layer 4.

### How an Oracle Monitors Work Sessions

```bash
# Capture last 50 lines of output
tmux capture-pane -t "Kommu-1" -p -S -50

# Check if Claude is still working (look for these indicators):
# - "Thinking..." / "Running..." / "Writing..." = still working
# - "❯" or prompt visible = task complete or idle
# - Error messages = needs intervention
```

### Shadow Manager (DURING — Safety Mesh Layer 3)

Live signal monitor running every 3 min via `~/.aisb/lib/oracle-observer.sh` (cron). For each target (worker OR oracle), `~/.aisb/lib/oracle-shadow.sh` evaluates **14 signals** at Tier 1 (heuristic, 0 token):

`THRASH`, `ERROR_BURST`, `SILENT_DRIFT`, `SCOPE_CREEP`, `BUILD_REGRESSION`, `PROGRESS_STAGNATION`, `PANE_STUCK_PATTERN`, `WORKER_HEALTH`, `TODO_STALL`, `RATE_LIMIT_STALL`, `API_ERROR_TRANSIENT`, `PANE_PROMPT_IDLE`, `OOM_HINT`, `CPU_OVERLOAD`.

| Tier | Cost | Behavior |
|---|---|---|
| Tier 1 | 0 token | Heuristic detection from `tmux capture-pane` + state files |
| Tier 2 | Haiku via Max (opt-in `SHADOW_LLM=haiku`) | Disambiguates ambiguous Tier-1 hits |
| Tier 3 | Telegram | Escalation when retry × 2 exhausted |

**Oracle vs Worker asymmetry (2026-05-16 design-flaw fix):**

| Aspect | WORKER | ORACLE |
|---|---|---|
| Default | Prescriptive nudge (`recovery_apply`) | **Observe-only** (JSONL + throttled FYI) |
| THRASH | Active | Disabled |
| STAGNATION floor | ~9 min | 360 min + idle ≥600s confirmation |
| Brief-aware emergency | n/a | Question-mode ("tu y es toujours ?") |

Kill-switch: `~/.aisb/state/.shadow-nudge-disabled` (panic stop). Logs: `~/.aisb/logs/shadow.log` (workers), `shadow-oracle-observations.jsonl` (oracles).

Full asymmetry contract: `oracle-protocol.md` §Asymétrie ORACLE vs WORKER.

### Oracle Prompt Template

```
## Context
Project: {Project}
Stack: {stack details from CLAUDE.md}
Current state: {relevant context}

## Task
{Clear description of what needs to be done}

## Steps
1. {Specific step 1}
2. {Specific step 2}
3. {Specific step 3}

## Success Criteria
- {Criterion 1}
- {Criterion 2}
- Run /debugaudit {scope} and pass

## Output Format
Report:
- What was changed (files + summary)
- Test results
- Any issues found
- Next steps if any
```

### Oracle Report Template (Telegram)

```html
<b>⚡ AISB  ›  {Project} Task Complete</b>
<code>{date}  ·  {time} UTC  ·  oracle-{Project}</code>

<blockquote>{icon} {Task description}
{What was done}  →  <b>✅ DONE</b></blockquote>

<blockquote>📋 Changes
{file1}: {change}
{file2}: {change}</blockquote>

✅  <b>{summary}</b>  ·  {test results}  ·  {deploy status}
```

---

## Tmux Operations Reference

### Reliable Message Delivery

**ALWAYS use load-buffer for long prompts** (send-keys truncates after ~500 chars):

```bash
# Write prompt to temp file
cat > /tmp/prompt.txt << 'EOF'
Your detailed prompt here...
Can be as long as needed.
EOF

# Load into tmux buffer
tmux load-buffer /tmp/prompt.txt

# Paste into target session
tmux paste-buffer -t "session-name"

# Press Enter to submit
tmux send-keys -t "session-name" Enter

# Clean up
rm /tmp/prompt.txt
```

### Session Management

```bash
# List all sessions
tmux list-sessions

# Check if session exists
tmux has-session -t "oracle-Kommu" 2>/dev/null && echo "alive"

# Create new session in specific directory
tmux new-session -d -s "Kommu-1" -c "/path/to/project"

# Capture output (last N lines)
tmux capture-pane -t "session-name" -p -S -50

# Kill session
tmux kill-session -t "session-name"

# Send short command
tmux send-keys -t "session-name" "command" Enter
```

### Session Naming Rules

| Pattern | Creator | Persistence |
|---------|---------|-------------|
| `Home` | AISB master | Always running |
| `oracle-{Project}` | dispatch-to-session.sh (bot) | On-demand, no limit on concurrent |
| `{Project}-N` | Oracle (via dispatch-to-session.sh) | Ephemeral (killed after task) |
| `{Project}-{task}` | Oracle (via dispatch-to-session.sh) | Ephemeral (killed after task) |
| `AISB-{AGENT}` | spawn-agent.sh | Ephemeral (auto-cleanup) |
| `Home-{topic}` | AISB | Ephemeral (killed after task) |

---

## Slash Commands (Work Session Arsenal)

### Execution Commands

| Command | Agents | Purpose |
|---------|--------|---------|
| `/team [task]` | 3-6 senior | Default parallel team for complex work |
| `/godmode [task]` | Full autonomy | Multi-hour missions with heartbeat |
| `/build` | Pipeline | Build + deploy to production |
| `/new [feature]` | Team | New feature implementation |

### Quality Commands

| Command | Agents | Purpose |
|---------|--------|---------|
| `/codeaudit` | Forensic | 23-phase forensic code audit |
| `/flowaudit` | Forensic | 20-phase user flow forensics |
| `/uiuxaudit` | Art Director | Design forensics & coherence |
| `/refontaudit` | Senior lead | 22-phase dashboard refonte — Linear/Vercel/Stripe-grade ground-up redesign (/440) |
| `/debugaudit [scope]` | Deep verify | 18-phase runtime bug hunting (replaces /hunt, /maniac, /xoxo) |
| `/featureaudit` | Forensic | 16-phase feature completeness |
| `/automationaudit` | Forensic | 22-phase automation infrastructure (cron, scripts, daemons) |
| `/logicaudit` | Architect | 20-phase systems logic optimization (Einstein-grade) |
| `/debugaudit [scope]` | Quick | Post-fix runtime verification |
| `/e2e [scope]` | Integration | End-to-end tests |

### Planning Commands

| Command | Purpose |
|---------|---------|
| `/planner` | DAG-based step planning |
| `/refontaudit` | Senior dashboard refonte (22 phases, Linear/Vercel-grade) — replaces /deepux |
| `/aisb [task]` | ORACLE-led smart orchestration |

### Design Commands

| Command | Purpose |
|---------|---------|
| `/taste-skill` | Anti-generic UI architect |
| `/taste-skill` | Premium aesthetic, anti-generic UI |
| `/minimalist-skill` | Editorial design (Notion/Linear) |
| `/minimalist-skill` | Editorial design, bento grids |
| `/redesign-skill` | Audit & upgrade existing UIs |

---

## Health Monitoring

### Session Registry (Primary Monitor)

```bash
# /tmp/aisb-sessions.json — updated every 30s by bot monitor loop
# Tracks all active oracle + worker sessions
# Bot reads this to enforce no limit on concurrent oracle limit
cat /tmp/aisb-sessions.json
```

### Heartbeat System

```bash
# Bot→Convex heartbeat (30s interval)
# Per-oracle heartbeat.sh still active for each running oracle

~/.aisb/lib/heartbeat.sh beat oracle-Kommu

# Timeout: 2 minutes (no beat = dead)
# Files: ~/.aisb/heartbeats/{session}.beat
# Check all: heartbeat.sh status
```

### DM Notifications

Monitor loop sends completion notifications in DM (not just topic). Gareth gets notified wherever he is.

### Manual Commands

```bash
# Check session registry
cat /tmp/aisb-sessions.json | jq .

# Check all heartbeats
~/.aisb/lib/heartbeat.sh status

# List saved sessions
~/.aisb/lib/session-resume.sh list

# Manually dispatch to oracle
~/.aisb/lib/dispatch-to-session.sh oracle-Kommu "Resume: check your last task and continue" /home/hacker/VibeCoding/work/kommu

# Force re-inject skills
~/.aisb/lib/skill-inject.sh inject-set /path/to/project core

# View audit trail
~/.aisb/bin/aisb-audit Kommu

# Check locks
~/.aisb/bin/aisb-locks --list
```

---

## Inter-Agent Communication

### Mailbox System

```bash
# Send message between agents
~/.aisb/mailbox/send.sh SERAPH MORPHEUS "Build is done, please verify"

# Read inbox
~/.aisb/mailbox/read.sh MORPHEUS

# Storage: ~/.aisb/mailbox/{AGENT}.jsonl
```

### Hub-and-Spoke Model

```
Workers ─────► Oracle (PM) ─────► AISB ─────► Gareth
Workers ◄───── Oracle (PM) ◄───── AISB ◄───── Gareth
```

Rules:
- Workers only talk to their Oracle
- Oracles only talk to AISB
- Cross-project coordination goes through AISB
- Emergency escalation can bypass one level
- No direct worker-to-worker or oracle-to-oracle communication

---

## Audit & Logging

### Audit Trail

```bash
# Every significant action is logged
# Format: JSONL append-only
# Location: ~/.aisb/audit/YYYY-MM-DD.jsonl

# Fields per entry:
{
  "timestamp": "ISO-8601",
  "actor": "AISB|oracle-Kommu|Kommu-1",
  "action": "dispatch|complete|error|deploy",
  "project": "Kommu",
  "details": "...",
  "duration_ms": 12345
}
```

### Task Locks

```bash
# Prevent two agents working on same task
# Auto-expire: 2h TTL
# Auto-steal: if lock holder is dead (no heartbeat)
# Location: ~/.aisb/locks/{task-hash}.lock
```

### SMITH Learning Engine

```bash
# Auto-improvement system
# Logs patterns and learnings from completed tasks
# Location: ~/.aisb/smith/learnings.jsonl

# Commands:
smith learn "Pattern description" "category"
smith evolve    # Full evolution cycle
smith suggest   # View suggestions
smith patterns  # View learned patterns
```

---

## Decision Framework

### AISB Auto-Execute (No Confirmation)

- Read files, check status, run diagnostics
- Answer questions about infrastructure
- Spawn research agents (NIOBE)
- Quick code fixes on non-production files
- Run builds and tests

### AISB Notify After Doing

- Running builds or tests
- Checking service health
- Spawning agents for tasks

### AISB Ask First (Always)

- Deploy to production
- Modify configuration files
- Restart services
- Destructive commands (delete, reset)
- Spending money or credits

---

## Quick Reference: The Complete Flow

```
Gareth (Telegram)
  → AISB Bot (DM keyword / /command / topic)
    → Oracle tmux (on-demand, no limit)
      → dispatch-to-session.sh
          ├─ Safety Mesh L2: CPU throttle (queue if load > 2.5×cores)
          └─ Safety Mesh L1: persist brief-<session>.txt
        → Work Session tmux + /team
          → Claude Code agents
          ↳ Safety Mesh L3: Shadow observer (cron */3) watches signals
        ← worker writes .done.json
      → close-gate.sh ack-worker
          └─ Safety Mesh L4: Mission Auditor (≥85/100) → APPROVED → ack
      ← Oracle monitors, closes
    ← Monitor loop → DM + Topic notification
  ← AISB → Gareth (Telegram)
```

### Detailed Step-by-Step

```
GARETH writes (DM with keyword / /dent / topic message)
    │
    ▼
AISB BOT receives
    ├── Identify project (keyword / direct command / topic_id → projects.json)
    ├── Check no limit on concurrent oracles
    ├── Post in topic (visibility)
    └── dispatch-to-session.sh oracle-{Project} "{task}" {path}
            │
            ▼
        ORACLE-{Project} (tmux, on-demand)
            ├── ETAPE 1: ANALYSE — Read CLAUDE.md, decompose
            ├── ETAPE 2: DISPATCH — dispatch-to-session.sh {Project}-N '/team [prompt]' {path}
            ├── ETAPE 3: MONITORING — tmux capture-pane every 30s
            │           │
            │           ▼
            │       {Project}-N WORKER (tmux)
            │           ├── Claude Code executes
            │           ├── /team → 3-6 parallel agents
            │           ├── Code changes + tests
            │           └── Report results
            │
            ├── ETAPE 4: CLOSE + VERIFY — kill session + /debugaudit
            └── ETAPE 5: VERIFICATION GATE — build, deploy, push, result file
                        │
                        ▼
                AISB formats + sends to Gareth
                └── DM notification + Design C blockquote in topic
```

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| Oracle not responding | Dead tmux session | Re-dispatch via `dispatch-to-session.sh` (on-demand recreation) |
| Work session stuck | Claude idle/crashed | Oracle should detect + respawn |
| No Telegram report | Dispatch missed topic post | Check audit trail, re-dispatch |
| Duplicate work | Missing task lock | Check `aisb-locks --list` |
| Lost context after restart | session-resume failed | Check `~/.aisb/sessions/` for saved state |
| Slow response | Too many concurrent sessions | Check system resources, no oracle limit |
