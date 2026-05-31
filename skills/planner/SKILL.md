---
name: planner
description: >
  OmegaOS-native implementation planner. Reads Vision/PRD/CLAUDE.md, decomposes the work
  into a DAG of single-worker-dispatch steps, and emits a typed `.planner/tracker.json`
  that the Rust engine (`omega plan-run`) executes with structural can't-skip enforcement
  (Gate) and independent verify-command proof (Guardian). Replaces the prose-only VPS
  planner: sequential execution is enforced by the engine, not by instructions.
  Use when user says "/planner", "plan this", "make a plan", "build the plan", "decompose",
  "planifie", "fais le plan".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep"]
domain: orchestration
read_only: false
triggers: ["planner", "plan this", "make a plan", "build the plan", "decompose", "planifie"]
---

# /planner — OmegaOS-native Implementation Planner

You produce a **typed plan** that the Rust engine drives. Your job is NOT to execute the
plan step-by-step yourself — you generate `.planner/tracker.json`, then hand it to the
engine. The engine guarantees no step is skipped and no step is "done" without its
`verify_command` passing. Your only job is to make the plan **good**.

## Commands

```bash
/planner [task]          # Generate .planner/tracker.json from Vision/PRD (then offer to run)
/planner run             # Execute the existing plan via the engine  → omega plan-run
/planner status          # Progress dashboard                         → omega plan-status
```

The engine binary is `omega` (built from source by `install.sh`, at `~/.local/bin/omega`).
If `omega` is missing, fall back to `~/.omega/skills/planner/fallback/plan.ts` (Bun).

---

## IRON RULES

### 1. Build from Vision/PRD — ALWAYS
Before writing any plan, read the real context:
```bash
cat CLAUDE.md 2>/dev/null
cat Vision/VISION.md 2>/dev/null || cat Vision/*.md 2>/dev/null
find . -iname "PRD*" -o -iname "prd*" | head -5
git log --oneline -10 2>/dev/null
```
Every step must trace to a Vision/PRD requirement. No inventing features.

### 2. Every step is ONE worker-dispatch unit
If a step cannot be claimed and finished by ONE worker in ONE session, split it. If two
steps must share a file/commit/verification, merge them. Disjoint `files_to_touch` across
steps in the same wave (the engine runs file-disjoint ready steps in parallel).

### 3. Every step carries the 7 fields the engine requires
`step_id`, `title`, `description` (80+ chars, precise), `files_to_touch` (≥1 exact path,
never empty, never `["src/"]`), `done_criteria` (testable), `verify_command` (a single
shell line that exits 0 iff the step is truly done — the Guardian RE-RUNS this; "true" is
never acceptable for real work), `depends_on` (exact step_ids; the DAG must be acyclic).

### 4. The engine enforces sequencing — you only declare the DAG
Do NOT write "do steps in order" instructions. Encode order in `depends_on`. The engine's
`ready_steps` selector will only ever hand a worker a step whose deps are all `done`. You
cannot make it skip, and you don't need to.

### 5. Audits are a terminal `wave`, not a final phase
Quality-Arsenal audits belong in the plan as steps with `"wave": "audit"`, `depends_on` =
all implementation steps, one audit per step (`verify_command` checks the audit verdict).
The engine holds the audit wave until all implementation steps are `done`.

---

## Output: `.planner/tracker.json` (EXACT schema — the engine parses this)

Write valid JSON matching the Rust `PlanTracker` type exactly. Fields `wave`, `attempt`,
`started_at`, `completed_at` accept defaults (`null`/`0`) but include them for clarity.
`status` is always `"pending"` at generation time.

```json
{
  "project": "ProjectName",
  "total_phases": 2,
  "active_phase": 1,
  "planner_version": "7.0",
  "generated_at": "2026-05-31T00:00:00Z",
  "phases": [
    { "id": 1, "name": "Foundation", "goal": "Schema + auth — everything builds on this",
      "step_ids": ["STEP-001", "STEP-002"] },
    { "id": 2, "name": "Audit", "goal": "Forensic quality gate", "step_ids": ["STEP-003"] }
  ],
  "steps": [
    {
      "step_id": "STEP-001", "phase": 1,
      "title": "Create Convex bookings schema with indexes",
      "description": "Define convex/schema.ts with a bookings table (userId, slotId, status, createdAt) plus search indexes on userId and slotId. Validate with Convex validator types so `npx convex dev` accepts it.",
      "files_to_touch": ["convex/schema.ts"],
      "done_criteria": "npx convex dev --once starts with no schema error and bookings appears in _generated/api",
      "verify_command": "npx convex dev --once 2>&1 | grep -qv 'schema error'",
      "depends_on": [],
      "wave": "foundation", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    },
    {
      "step_id": "STEP-002", "phase": 1,
      "title": "Add booking create mutation",
      "description": "Implement convex/bookings.ts create() mutation taking {slotId,userId}, inserting a bookings row with status='pending'. Reject double-booking of the same slot.",
      "files_to_touch": ["convex/bookings.ts"],
      "done_criteria": "npm run build passes AND the mutation is callable",
      "verify_command": "npm run build",
      "depends_on": ["STEP-001"],
      "wave": "w1", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    },
    {
      "step_id": "STEP-003", "phase": 2,
      "title": "/codeaudit on the booking MVP",
      "description": "Run the forensic code audit scoped to convex/** to verify the booking implementation is solid before ship.",
      "files_to_touch": ["audits/.codeaudit/verdict.json"],
      "done_criteria": "codeaudit normalized score >= 85/100",
      "verify_command": "test -f audits/.codeaudit/verdict.json",
      "depends_on": ["STEP-001", "STEP-002"],
      "wave": "audit", "attempt": 0,
      "status": "pending", "started_at": null, "completed_at": null
    }
  ]
}
```

Valid `wave` values: `foundation | w1 | w2 | w3 | audit | deploy` (or omit / `null`).

## Validation before handing to the engine
After writing `tracker.json`, sanity-check it yourself:
```bash
# valid JSON?
python3 -c "import json;json.load(open('.planner/tracker.json'))" && echo "JSON ok"
# engine accepts it + shows the DAG (this also proves no cycle / valid schema):
omega plan-status . 2>&1 || echo "engine could not load the plan — fix the schema/DAG"
```
`omega plan-status` printing the steps with `ready N | blocked M` confirms the engine
parsed the typed plan and computed the DAG. If it errors, fix the JSON to match the schema
above — do not weaken the plan to satisfy a parse error.

## Execute
```bash
omega plan-run .     # the engine drives: ready-set -> spawn worker -> Guardian verify -> advance
omega plan-status .  # watch progress
```

If `omega` is not on PATH, run the Bun fallback:
```bash
bun ~/.omega/skills/planner/fallback/plan.ts status .
bun ~/.omega/skills/planner/fallback/plan.ts run .
```

## Anti-patterns (rejected at validation)
| Failure | Fix |
|---|---|
| `verify_command: "true"` for real work | A real falsifiable command (build/test/curl) |
| `files_to_touch: []` or `["src/"]` | Exact file paths, ≥1 |
| "do steps in order" prose | Encode order in `depends_on` — the engine enforces it |
| Audits as a separate Phase 7 | Audits as `wave: "audit"` steps in the plan |
| One giant step ("build the feature") | Split into one-worker-dispatch units |
