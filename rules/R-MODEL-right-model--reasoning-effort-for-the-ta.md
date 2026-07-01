# R-MODEL — Right model & reasoning-effort for the task

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-07-02

## Rule

Match the Claude model tier AND reasoning effort to the task's cognitive load — never habit, never inertia. Tiers: Opus 4.8 (claude-opus-4-8) = hardest reasoning — oracle/orchestration brains, adversarial verify/judge stages, architecture, security analysis, final synthesis. Sonnet 5 (claude-sonnet-5) = the balanced pick when a standard build/edit sub-agent is explicitly tiered. Haiku 4.5 (claude-haiku-4-5) = cheap high-volume mechanical fan-out — file-by-file transforms, grep/extract/classify, label/format passes, structured extraction. Fable 5 (claude-fable-5) = creative/expressive drafting — naming, copy hooks, narrative. In a Workflow, DEFAULT to omitting per-agent model/effort (inherit the session model — almost always correct); override only when highly confident a different tier fits. Reasoning effort: omitted = inherit the session/dispatch effort; when you set it, low for mechanical stages, medium as the balanced baseline, high/xhigh/max for the hardest verify/judge/design. The map guides the tier you CHOOSE at dispatch/spawn/Workflow time — never re-tier a running session mid-mission. Start at the map's tier for the load; the cheapest tier that hits the quality bar is the correct call (it keeps missions inside the R-BUDGET cap — the bar itself is L5's: cost-matching is never an excuse for a 'lightweight' pass of a real task), and escalate the moment a cheaper tier demonstrably fails on runtime evidence (L1), never on vibes. Use live model ids — never a retired id; deliberately pinned older-but-live models (R-COUNCIL's seats, the AISB matrix table) are doctrine that OVERRIDES this map — re-tier them by editing their own doc, never silently. The claude-api skill is the SSOT for ids/pricing/limits/caching — on any divergence from the ids above, the skill wins; consult it, never guess. Complements R-ORCH (which primitive) and R-COUNCIL (which owns council composition).

## Origin

rules.rs had R-STACK (languages), R-ORCH (which primitive), R-COUNCIL (multi-model for high-stakes), R-BUDGET (the cap) — but nothing matched the Claude model tier + reasoning effort to the cognitive load of a task, so agents defaulted to the session model by habit: max effort on mechanical fan-out and, worse, cheap passes on judge stages. R-MODEL makes quality-matched AND cost-matched model choice an injected, scoped doctrine.
