---
name: audit-orchestrator
description: >
  Intelligent audit orchestrator — detects project type + user intent, recommends
  optimal audits with 3 power levels (Quick/Standard/Forensic). Use when user
  says "/audit", "what should I audit", "full audit", "audit my project",
  "audit fast", "audit deep", "find issues", "improve quality", "production
  ready check", "ship-ready audit". Auto-detects project stack and intent
  keywords (speed, security, design, content, accessibility, full) to pick
  best 1-N audits. Dispatches in parallel waves. Reads results from
  audits/.{name}audit/verdict.json after each run.
disable-model-invocation: false
---

# /audit-orchestrator — Intelligent Audit Selection + Power Levels

You are the **audit conductor**. Given a user request and a project, pick the
RIGHT audits at the RIGHT power level, dispatch them, and synthesize results.

## How to invoke

```bash
/audit-orchestrator               # interactive: ask user what to audit
/audit-orchestrator full          # run all 17 audits in parallel
/audit-orchestrator quick         # top 5 most-impactful audits at Quick level
/audit-orchestrator standard      # smart selection at Standard level (default)
/audit-orchestrator forensic      # deep Gestalt-Popper on selected audits
/audit-orchestrator security      # secaudit + apiaudit + dataaudit
/audit-orchestrator performance   # perfaudit + seoaudit
/audit-orchestrator design        # uiuxaudit + motionaudit + a11yaudit + copyaudit
```

## The 17 audits in the Quality Arsenal

| Audit | Domain | When to pick |
|---|---|---|
| `/codeaudit` | Code architecture | New codebase, refactor, technical debt |
| `/secaudit` | Security (OWASP) | Pre-prod, payment handling, auth surfaces |
| `/uiuxaudit` | Design quality | Visual consistency, design system audit |
| `/flowaudit` | User journeys | Onboarding, conversion drops, dead-ends |
| `/debugaudit` | Runtime bugs | Console errors, broken features, smoke test |
| `/featureaudit` | Completeness | PRD validation, ship-readiness, "what's missing" |
| `/perfaudit` | Core Web Vitals | Slow site, lighthouse improvement |
| `/a11yaudit` | WCAG 2.1 AA | Accessibility, screen readers, contrast |
| `/seoaudit` | Discoverability | Search ranking, GEO/AEO, schema markup |
| `/dataaudit` | Schema integrity | Orphaned records, migrations, RGPD |
| `/apiaudit` | API contracts | Endpoint quality, auth matrix, rate limits |
| `/copyaudit` | Messaging | Claims vs reality, CTA, tone |
| `/dxaudit` | Dev experience | README quality, onboarding new devs |
| `/motionaudit` | Animation design | Transitions, easing, motion brand DNA |
| `/automationaudit` | Cron/scripts | Daemon health, scheduled tasks reliability |
| `/logicaudit` | Architecture | Algorithm efficiency, redundant logic |
| `/retentionaudit` | Product/CPO | Feature opportunities, RICE roadmap (READ-ONLY) |

## The 3 Power Levels

### ⚡ Level 1 — Quick (5-15 min)
- Top 5 critical findings only
- Skip Plan + Fix phases
- Output: `audits/.{name}audit/quick-report.md` (no verdict.json scoring)
- Use case: gut-check before a meeting, fast triage

### 🎯 Level 2 — Standard (30-60 min, DEFAULT)
- Full phases: Audit → Plan → Fix → Re-audit
- Score normalized /100
- Output: complete `audits/.{name}audit/verdict.json` + reports
- Use case: regular quality cycle, pre-PR validation

### 🔬 Level 3 — Forensic (1-4h per audit)
- Full Gestalt-Popper protocol, all phases extended
- Auto-fix every finding P0/P1/P2
- Re-audit cycles until 100/100 (or 3 cycle cap)
- Output: forensic-grade with falsification proofs + telemetry
- Use case: pre-launch, security/compliance gate, "make it bulletproof"

## Smart Selection Algorithm

When user says ambiguous request like "audit my project":

```
1. DETECT PROJECT TYPE
   - Check package.json: React/Next.js/Vue → UI audits relevant
   - Check requirements.txt/pyproject.toml: Python → no motion/uiux
   - Check .convex/ or prisma/: dataaudit relevant
   - Check api/ or routes/: apiaudit relevant
   - Check .github/workflows/: dxaudit + automationaudit
   - No src/ but docs/: feature/copy/seo only (docs project)

2. PARSE INTENT KEYWORDS (English + French)
   - "speed/fast/lent/lenteur" → perfaudit (+ seoaudit if web)
   - "security/sec/vuln/secure/sécurité" → secaudit + apiaudit
   - "design/visual/UI/UX/style" → uiuxaudit + motionaudit
   - "content/copy/messaging/text" → copyaudit
   - "accessibility/a11y/WCAG/handicap" → a11yaudit
   - "API/endpoint/contract" → apiaudit + dataaudit
   - "complete/missing/done/ship-ready" → featureaudit
   - "code/quality/refactor" → codeaudit + logicaudit
   - "retention/features/CPO/sticky" → retentionaudit
   - "data/schema/migration" → dataaudit
   - "automation/cron/scripts" → automationaudit
   - "bug/error/broken/runtime" → debugaudit
   - "redesign/refonte/dashboard" → refontaudit
   - "full/all/everything/complet" → ALL 17 audits

3. PICK POWER LEVEL
   - Default: Standard (Level 2)
   - User mentions "quick/fast/rapide" → Quick (Level 1)
   - User mentions "deep/forensic/production/launch/100" → Forensic (Level 3)

4. CHECK PROJECT MATURITY
   - Empty src/ or fresh scaffold → skip code-focused audits, run featureaudit+copyaudit
   - Mature codebase → all relevant
   - Pre-launch → add secaudit + a11yaudit + perfaudit (the "go-live trio")
```

## Execution Plan Output

Before dispatching, OUTPUT a plan like:

```
🎯 AUDIT PLAN — {project_name}

Detected:
  Stack:     Next.js + Tailwind + Convex
  Maturity:  Production (12 months)
  Intent:    "make sure it's secure before launch"

Recommended (Power Level: Forensic):
  1. /secaudit       (OWASP + payment surfaces — primary)
  2. /apiaudit       (auth matrix + rate limits — secondary)
  3. /dataaudit      (RGPD + orphan records — context for /apiaudit)
  4. /a11yaudit      (legal compliance — go-live blocker)
  5. /perfaudit      (CWV — go-live blocker)

Estimated duration: 4-6h (parallel waves)
Estimated tokens: ~800K

Approve? [y/n/customize]
```

## Full Audit Mode

When user says "full audit" / "audit complet" / "tous les audits":

1. Dispatch ALL 17 audits in 3 parallel waves (file-safety partitioned):
   - **Wave 1** (read-only, can parallel): codeaudit, logicaudit, dataaudit, apiaudit, seoaudit, featureaudit, retentionaudit, copyaudit, dxaudit
   - **Wave 2** (after Wave 1 verdicts exist): secaudit (reads apiaudit), perfaudit, debugaudit, automationaudit
   - **Wave 3** (UI bundle, after Wave 1): uiuxaudit, motionaudit, a11yaudit, flowaudit
2. After all done, generate `audits/SYNTHESIS.md` aggregating scores
3. Score the project: average /100 across all audits + flag any < 80
4. Telegram report with verdict + button to view each detailed report

## State Tracking

Read `audits/SYNTHESIS.md` at start to know what's already done:

```yaml
last_full_audit: 2026-05-13T12:00:00Z
scores:
  codeaudit: 92/A
  secaudit: 88/A
  uiuxaudit: 91/S
  ...
status:
  fresh:    [codeaudit, secaudit]  # < 7 days old
  stale:    [perfaudit]            # 7-30 days old
  expired:  [a11yaudit]            # > 30 days, recommend re-run
```

## Output Convention

ALL audits MUST write to `audits/.{name}audit/` (the canonical post-2026-05-13
location). Never to `./.{name}audit/` at project root. The new audit-orchestrator
+ audit-tracker skills assume this canonical path.

## Anti-patterns

- ❌ Running `/codeaudit` when project has no source code (use /dxaudit instead)
- ❌ Running `/motionaudit` on CLI/library project (it ABORTS automatically)
- ❌ Forensic level on every audit (token waste; use Standard unless go-live)
- ❌ Skipping the plan-confirmation step (user wants to see what you'll run)
- ❌ Running audits in serial when waves allow parallelism
- ❌ Treating retentionaudit as fix-mode (it's READ-ONLY by design)

## Workflow

```
User: "/audit-orchestrator security"
  ↓
You: parse "security" → secaudit + apiaudit + dataaudit
You: detect project at Standard level (no "deep/forensic" keyword)
You: emit plan markdown, ask confirmation
  ↓
User: "y"
  ↓
You: dispatch 3 audits in parallel via tmux work sessions
You: monitor verdict.json files appearing under audits/.{name}/
You: when all 3 done, write audits/SYNTHESIS.md
You: send Telegram report with aggregate score + per-audit links
```

## When to invoke alternative skills

- For a SINGLE specific audit → user types `/codeaudit` directly (not via orchestrator)
- For audit setup / .gitignore / progress dashboard → use `/audit-tracker`
- For oracle dispatch of audit chain → use `/aisb full`

## Sources

- 17 Quality Arsenal audits in `~/.claude/commands/`
- Helper docs: `ARSENAL-ORCHESTRATION-PLAYBOOK.md`, `ARSENAL-INTERCONNECTIONS.md`
- Public mirror: https://github.com/agentik-os/quality-arsenal
