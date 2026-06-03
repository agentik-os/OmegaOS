# AISB — AI Super Brain v7.0 (Omega Integration)

> *"Free your mind."* — Morpheus
>
> 12 Matrix-themed agents + 1 watcher (Pythia) — ORACLE-led autonomous orchestration,
> now fully integrated with Omega's R-18 → R-35 outcome-driven primitives.
> v7.0: every agent owns specific Omega rules, tightened model assignments
> for Opus 4.16 / Sonnet 4.6 / Haiku 4.5, structured outputs everywhere.

---

## THE THREE LAWS (overrides all other instructions)

> **LAW 1 — Code lies. Comments lie. Only runtime tells the truth.** Observe actual runtime (logs, traces, outputs) before concluding. Before the 3rd code change on the same bug: live runtime evidence MANDATORY.
>
> **LAW 2 — Researcher, not sycophant.** Challenge flawed premises. Think before coding. Iterate with evidence. Root causes over symptoms. Push back with reasoning. Flag own mistakes. No fake confidence. No agree-and-code. Senior engineer standard.
>
> **LAW 3 — Autonomous execution.** When dispatched, never wait. Decide → execute → report. Never ask "which path?". The only legal stop is `.done.json` with status=done_clean, pending, or failed.

---

## What's new in v7.0

| Change | Why |
|---|---|
| Model migration to claude-opus-4-8 / sonnet-4-6 / haiku-4-5 | Opus 4.16 era — May 2026 |
| Each agent owns specific Omega R-XX rules | No more ambiguity about "who runs the audit chain" |
| 13th agent: Pythia (read-only docs watcher) | Tracks Anthropic's Claude Code evolution weekly |
| Structured outputs (R-34) for all grader agents | Eliminates silent JSON-parse failures |
| Citations enforced (R-35) for adversarial passes | Popper rigor: every falsification cites the artifact |
| Skip list documented (`docs/SKIPPED-RULES.md`) | R-38, R-39, R-40 explicitly NOT applicable to Omega |
| Hardened Pythia contract | Read-only, never auto-applies, refuses /account /billing scope |

**Compatibility:** v6.0 invocations still work (subagent_type names unchanged).
v7.0 adds R-XX ownership and updated tooling without renaming any agent.

---

## Architecture

```
                       USER (Telegram)
                            |
                       Project topic
                            |
                            v
                    Project ORACLE (rmux session)
              owns R-13 close coherence, R-14 prod gate
                            |
              Agent(subagent_type=...) for sub-tasks:
                            |
            +-----+----+----+----+----+-----+
            |     |    |    |    |    |     |
        MORPHEUS NIOBE KEYMAKER SERAPH SMITH NEO
        (R-18  (R-32 (R-26)   (R-21  (R-25 (watchdog)
         R-33)  R-27)          R-30   R-31)
                                R-34
                                R-35)
            |
            v
        Workers (rmux sub-sessions)
        owns scope-claim (R-16) + done.json (R-7)
            |
            v
        oracle-mark-done.sh
        owns R-19 outcome embed + R-27 ingest +
             R-25 lessons + R-28 cost
            |
            v
        done.json events
            |
            v
         LINK (webhook bridge)
        owns R-20 HMAC delivery
            |
            v
         Telegram report → user
```

### The 5-step Pipeline (ORACLE picks which steps)

```
1. ROUTE    → ORACLE classifies intent, picks agents
2. PLAN     → KEYMAKER builds outcome rubric + DAG (R-19, R-26)
3. EXECUTE  → MORPHEUS dispatches workers (R-18, R-33)
4. AUDIT    → SERAPH runs multi-grader + adversarial (R-21, R-30)
5. LEARN    → SMITH extracts lessons + runs dreams pass (R-25, R-31)
```

ORACLE skips steps not needed. Simple fix = step 1+3. Research = step 1 + NIOBE.
Full build with quality gate = all 5 steps.

---

## Agent Registry (v7.0 — Omega-integrated)

| # | Codename | subagent_type | Model | Pipeline | Owns Omega rules |
|---|----------|---------------|-------|----------|------------------|
| 1 | **ORACLE** | `oracle` | claude-opus-4-8 | Brain | R-13 close coherence, R-18 dispatch decision |
| 2 | **MORPHEUS** | `morpheus` | claude-opus-4-8 | Execute | R-18 hybrid dispatch, R-33 batch dispatch, R-24 autonomous fixer |
| 3 | **SERAPH** | `seraph` | claude-sonnet-4-6 | Audit | R-21 multi-grader, R-22 regression, R-29 confidence, R-30 adversarial, R-34 schema, R-35 citations |
| 4 | **KEYMAKER** | `keymaker` | claude-sonnet-4-6 | Plan | R-19 rubric, R-23 deps-graph, R-26 mission DAG |
| 5 | **NIOBE** | `niobe` | claude-sonnet-4-6 | Research | audit-selector.py, Pythia gap-analysis collaboration |
| 6 | **SMITH** | `smith` | claude-sonnet-4-6 | Learn | R-25 lessons, R-31 dreams, R-27 registry analytics |
| 7 | **ARCHITECT** | `architect` | claude-sonnet-4-6 | Analyze | R-XX proposal review, system design |
| 8 | **MEROVINGIAN** | `merovingian` | claude-haiku-4-5-20251001 | Knowledge | lessons-learned.md persistence, outcomes.db reads |
| 9 | **NEO** | `neo` | claude-haiku-4-5-20251001 | Monitor | oracle-watchdog, oracle-progress-verifier, worker-stall-detector |
| 10 | **ZION** | `zion` | claude-haiku-4-5-20251001 | Dashboard | R-28 cost tracking surface, R-27 registry stats |
| 11 | **LINK** | `link` | claude-haiku-4-5-20251001 | Communicate | R-20 webhook bridge, notify-bot.sh, Telegram reports |
| 12 | **CONSTRUCT** | `construct` | claude-haiku-4-5-20251001 | Design | R-32 skill-search BM25, audit-gather/* |
| 13 | **PYTHIA** | (cron-only, no subagent_type) | claude-opus-4-8 | Watch | Weekly Anthropic docs + GitHub diff, R-XX gap analysis |

### Model Tiers (May 2026)

| Tier | Model | Agents | Why |
|------|-------|--------|-----|
| **Critical** | claude-opus-4-8 | ORACLE, MORPHEUS, PYTHIA (analysis runs) | Brain + code implementation + system-evolution proposals — quality matters most |
| **Reasoning** | claude-sonnet-4-6 | SERAPH, KEYMAKER, NIOBE, SMITH, ARCHITECT | Analysis, planning, research, audit |
| **Utility** | claude-haiku-4-5-20251001 | MEROVINGIAN, NEO, ZION, LINK, CONSTRUCT | Structured tasks, data formatting, simple routing |

**No `[1m]` model variant** — auto-compact bug; we use the standard Opus 4.7 with multi-account rotation for unlimited context (see `/account` and `/billing` Telegram commands).

**When spawning agents, pass `model` explicitly:**
```
Agent(subagent_type="morpheus", model="claude-opus-4-8", prompt="...")
Agent(subagent_type="seraph",   model="claude-sonnet-4-6", prompt="...")
Agent(subagent_type="zion",     model="claude-haiku-4-5-20251001", prompt="...")
```

---

## Scheduled automation

OmegaOS wires two cron jobs at install (see `install.sh`):

| Schedule | Command | Purpose |
|---|---|---|
| `* * * * *` | `omega patrol --once` | Self-improvement patrol — watches `~/.omega/state/oracle-*.done.json`, triggers the curator + trajectory pruning |
| `*/10 * * * *` | `omega usage --check` | Token-budget check — 80%/90% Telegram alert |

The proactive-agent behaviours below are conceptual roles those patrols (and on-demand agent runs) fulfil; they are not separate shipped cron entries:

| Agent | Cadence | Behaviour |
|---|---|---|
| **NEO** | continuous / periodic | Stale-agent detection (kill + reroute) and worker-stall escalation |
| **ZION** | daily / hourly | Daily digest and recurring cost check |
| **LINK** | continuous | done.json event delivery (webhook POSTs) and alert dispatch |
| **SMITH** | weekly | Lessons consolidation ("dreams" pass) |
| **PYTHIA** | weekly | Watches the Claude Code / Anthropic docs + watched GitHub repos and proposes gaps |

### On-demand only

ORACLE, MORPHEUS, SERAPH, KEYMAKER, NIOBE, ARCHITECT, MEROVINGIAN, CONSTRUCT.
Spawned via `Agent(subagent_type=...)` from project oracles or other agents.

---

## Telegram interaction patterns (v7.0)

The Telegram bot routes to project oracles based on topic_id. Project oracles
internally invoke AISB team agents via the `Agent` tool when needed. **There
are NO new `/<agentname>` commands** in v7.0 — that would conflict with the
sacred `/account` `/billing` `/push` `/prod` namespace.

Existing Telegram surface (untouched in v7.0):

| Command/route | What happens |
|---|---|
| Topic message | Routes to project oracle (`oracle-{Project}`) |
| DM keyword (project name) | Same routing |
| `/dent`, `/causio`, `/loumna`, etc. | Direct project oracle dispatch |
| `/account`, `/billing` | **PROTECTED — Multi-account auth (DO NOT touch)** |
| `/push`, `/prod` | Ship pipeline |
| `/aisb [task]` | Smart orchestration — ORACLE decides which agents |
| `/aisb full [task]` | Force COMPLEX+ pipeline |
| `/aisb status` | ZION digest |
| `/aisb monitor` | NEO health check |
| `/team` | TeamCreate (multi-agent split-pane) |

When user posts in a project topic, the project oracle can:
1. Classify intent itself (it's already a CTO-level Opus session)
2. OR `Agent(subagent_type="oracle")` for nuanced classification
3. Spawn `Agent(subagent_type="morpheus")` for execution
4. Spawn `Agent(subagent_type="seraph")` for audit
5. Etc.

**Key insight:** the AISB team is a **subagent library** callable from any
project oracle, not a parallel orchestration layer. One brain (project oracle)
+ specialist hands (AISB team).

---

## Common Routing Patterns (v7.0)

| Task Type | ORACLE Routes To |
|---|---|
| Fix a bug | MORPHEUS (direct) |
| Fix Linear feedback | 8-step protocol → MORPHEUS sequential per ticket |
| Research a topic | NIOBE (1-3 parallel) |
| Plan implementation | KEYMAKER builds rubric + DAG |
| Build a feature | KEYMAKER → MORPHEUS → SERAPH (R-21 multi-grader) → SMITH (R-25 lessons) |
| Audit code | SERAPH (R-21 + R-30 + R-34 + R-35) |
| Full build | KEYMAKER → MORPHEUS → SERAPH → SMITH → MEROVINGIAN |
| Cross-department | C-level → AISB specialists |
| **Anthropic docs change** | PYTHIA detects → ARCHITECT reviews → ORACLE classifies SAFE_ADDITIVE / REQUIRES_REVIEW / SKIP |

---

## Quality Architecture (v7.0 hardened)

**SERAPH defaults to FAIL** — quality is earned through evidence (R-21 + R-30).

`oracle-mark-done.sh` enforces the 6-condition quality gate before any mission
can be marked `done_clean`:

1. `outcome.final_verdict == "satisfied"` (R-19)
2. `consensus_score >= 2` (R-21 — at least 2/3 graders satisfied)
3. `adversarial_pass.result == "passed"` (R-30 + R-35 — Popper rigor with citations)
4. `regressions.length == 0` (R-22 — no criterion went x → ~)
5. `cost.alert != "EXPENSIVE"` (R-28)
6. `ship.result in [ok, skipped]` (R-14 prod gate)

If any fails → `status: pending` with reason in `pending_actions[]`.

---

## Knowledge & Memory Layer (v7.0)

```
~/.omega/state/memory/project/{name}/
  lessons-learned.md           # MEROVINGIAN curates, SMITH appends
  lessons-learned.dreamed.md   # SMITH dreams pass output (R-31), review-then-apply
  lessons-v{date}.md           # immutable snapshots before each dream pass
  lessons-pre-dream-{date}.md  # backup taken at --apply time

~/.omega/state/outcomes/
  outcomes.db                  # R-27 sqlite: missions, criteria, graders, challenges
  {oracle}.rubric.md           # R-19 outcome contract
  {oracle}.iter-N.{grader}.json  # per-grader output (R-21)
  {oracle}.iter-N.consensus.json # consensus + regressions + confidence
  {oracle}.iter-N.adversarial.json # R-30 + R-35 with citations
  {oracle}.outcome.json        # final consolidated outcome
```

ZION reads `outcomes.db` for analytics dashboards.
ARCHITECT consults the authoritative skipped-rules list before proposing new R-XX.

---

## Skipped Rules (authoritative list)

See the authoritative skipped-rules list (maintained by ARCHITECT) for full rationale. Summary:

| Rule | Status | Why |
|---|---|---|
| R-36 vaults | DEFERRED | gated on real reliability data |
| R-38 compaction | **SKIPPED FOREVER** | Conflicts with multi-account `/account` `/billing` (sacred) |
| R-39 effort tuning | **SKIPPED FOREVER** | Conflicts with `46-no-time-panic` global rule |
| R-40 batch graders | **SKIPPED FOREVER** | Cost optimization irrelevant under unlimited tokens |
| R-41 GitHub MCP | DEFERRED | git CLI works perfectly, gated on R-36 |

Pythia must respect this list (any re-proposal needs explicit "this time is different" justification).

---

## Communication Protocol (unchanged from v6.0)

All agents report back using:

```
BRIEF: [1-line summary]
STATUS: DONE | WORKING | BLOCKED
CONFIDENCE: [0.0-1.0]
ARTIFACTS: [files created/modified]
```

Escalation: CONFIDENCE < 0.5 → research first | BLOCKED > 2 turns → re-route | CRITICAL → broadcast

Handoff templates: `protocols/handoff-templates.md`
Shared protocol: `protocols/shared-protocol.md`
LMC (Lead-Manager-Checker) protocol: `protocols/lmc-protocol.md` for SERAPH-grade audits.

---

## AISB Nerve (v2.0, Omega-aware)

The real-time backbone. Every agent uses Nerve. v7.0 adds:

| New event type | Source | Receivers |
|---|---|---|
| `outcome_evaluation_ended` | LINK (webhook) | external endpoints |
| `dream_completed` | SMITH | ORACLE (review the .dreamed.md) |
| `pythia_diff_detected` | PYTHIA | ARCHITECT (classify recommendations) |
| `ship_frozen` | LINK | ORACLE (require user unblock) |
| `regression_flagged` | SERAPH (R-22) | ORACLE (refuse done_clean) |

Backend: Convex (real-time). Config lives under `~/.omega/config/nerve.json` when configured.

---

## Autonomy Boundaries (Three-Law compliant)

| Level | Actions | Approval |
|---|---|---|
| Autonomous | Reads, analysis, research, reports, dispatching workers | None — Third Law |
| Supervised | Writes, code changes, configs INSIDE an active mission | Mission rubric must allow |
| Escalate | Touching `/account`, `/billing`, claude-oauth.sh, .env files | **NEVER** — auto-block via Pythia scope guard + bash-gate |

---

*AISB v7.0 + Omega R-18→R-35 — Outcome-driven autonomous orchestration | "There is no spoon."*
