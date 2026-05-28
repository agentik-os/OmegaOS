---
name: audit-tracker
description: >
  Audit setup + tracking dashboard. Use when user says "/audit-tracker", "audit
  status", "audit dashboard", "audit history", "list audits", "where am I with
  audits", "setup audits", "init audits". Ensures audits/ folder exists, .gitignore
  configured, tracks all audits run with scores + freshness, recommends re-runs
  when stale (>30 days). Reads audits/.{name}audit/verdict.json across all
  audit subdirs to build dashboard.
disable-model-invocation: false
---

# /audit-tracker — Setup + Progress Dashboard

You are the **audit accountant**. Init audit infrastructure for a project and
report status of all past + ongoing audits.

## Modes

```bash
/audit-tracker init           # setup audits/ + .gitignore + initial SYNTHESIS.md
/audit-tracker                # dashboard: status of all audits
/audit-tracker stale          # only audits older than 30 days
/audit-tracker scores         # only the scores table (compact)
/audit-tracker latest         # most recent audit + summary
```

## Mode 1 — `/audit-tracker init`

Bootstrap audits infrastructure in the current project:

1. Create `audits/` directory if missing
2. Append to `.gitignore` (idempotent — only if not already present):
   ```gitignore
   # Audit outputs (Quality Arsenal)
   /audits/.*audit*/
   !/audits/.*audit*/verdict.json
   !/audits/.*audit*/REPORT.md
   !/audits/.*audit*/CHECKLIST.md
   !/audits/SYNTHESIS.md
   ```
   This ignores the bulky audit artifacts but preserves the headline outputs
   (verdict.json, REPORT.md, SYNTHESIS.md).
3. Write `audits/SYNTHESIS.md` skeleton:
   ```markdown
   # Audit Synthesis — {project_name}

   Last update: 2026-05-13
   Status: 🟡 No audits run yet

   ## Recommended starting audits

   - `/audit-orchestrator quick` — gut-check (15 min)
   - `/audit-orchestrator standard` — regular quality cycle (60 min)
   - `/audit-orchestrator full` — complete arsenal (4h)

   ## Past runs

   _none yet_
   ```
4. Output to user: "✅ Audits initialized. Run /audit-orchestrator to start."

## Mode 2 — `/audit-tracker` (dashboard)

Scan `audits/` for all `.{name}audit*/verdict.json` files. Build a markdown table:

```
🎯 AUDIT DASHBOARD — {project_name}

┌──────────────────────┬──────┬──────┬───────────┬────────────────┐
│ Audit                │ Score │ Grade │ Age      │ Status         │
├──────────────────────┼──────┼──────┼───────────┼────────────────┤
│ codeaudit (v2)       │  92  │  A   │  2 days   │ ✅ Fresh       │
│ secaudit             │  88  │  A   │  5 days   │ ✅ Fresh       │
│ uiuxaudit (v3)       │  91  │  S   │  3 days   │ ✅ Fresh       │
│ a11yaudit (v2)       │  88  │  A   │ 14 days   │ ⚠️ Aging       │
│ perfaudit            │  79  │  B   │ 35 days   │ 🔴 Stale       │
│ apiaudit             │  67  │  C   │ 12 days   │ 🟡 Re-audit    │
└──────────────────────┴──────┴──────┴───────────┴────────────────┘

Overall health: 84/100 (Grade A-)
Recommended: re-run /perfaudit (stale 35d), push /apiaudit to >85 (re-audit)
```

Status thresholds:
- **Fresh** ≤ 7 days
- **Aging** 8-30 days
- **Stale** > 30 days (recommend re-run)
- **Re-audit** score < 85 (recommend fix cycle)

## Mode 3 — `/audit-tracker stale`

Filter dashboard to only show audits > 30 days old.

## Mode 4 — `/audit-tracker scores`

Compact one-liner per audit:
```
codeaudit: 92/A · secaudit: 88/A · uiuxaudit: 91/S · ...
```

## Mode 5 — `/audit-tracker latest`

Show the single most recent audit + its findings summary + verdict link.

## Implementation hints

To parse a verdict.json:
```bash
jq -r '.score, .grade, .timestamp' audits/.{name}audit/verdict.json
```

If the audit has v2/v3/v4 variants (e.g., `.codeaudit-v3/`), prefer the
HIGHEST version (most recent re-audit cycle).

Detect project name from:
1. `package.json` "name" field
2. Else basename of cwd

Detect audit freshness:
- File mtime of `verdict.json` → compare to `now()`
- Days = int((now - mtime) / 86400)

## Anti-patterns

- ❌ Listing audits in random order (sort by mtime desc OR by score asc)
- ❌ Missing the "Recommended actions" footer
- ❌ Including audits that have no verdict.json (incomplete runs)
- ❌ Modifying audit outputs (read-only)
- ❌ Running an audit directly (delegate to `/audit-orchestrator`)

## Output format

Always end with **3 actionable recommendations** like:
```
📋 Next actions:
1. Re-run /perfaudit (last run 35d ago, scores drift)
2. Push /apiaudit from C → A via 2 fix cycles
3. Run /retentionaudit (never run, would unlock new feature ideas)
```

## Sources

- Reads: `audits/SYNTHESIS.md`, `audits/.{name}audit*/verdict.json`
- Writes: `audits/SYNTHESIS.md` (updates), `.gitignore` (init mode)
- Related: `/audit-orchestrator` to actually RUN audits
- Public mirror: https://github.com/agentik-os/quality-arsenal
