---
name: logicaudit
description: "Forensic systems logic audit. 20 phases, scored /360 (normalized /100). Gestalt-Popper doctrine."
domain: logic
phases: 20
max_score: 360
triggers: ["logic", "optimize logic", "system optimization", "architecture logic", "make it smarter"]
read_only: false
---

# /logicaudit — Systems Logic Audit

**What it answers:** Is the logic OPTIMAL?

## Invocation

```bash
omega audit run logicaudit --dir <project-path>
# Or via agent skill:
/logicaudit --files=<globs> --scope="<description>"
```

## Phases (20)

1. Redundant logic detection
2. Suboptimal algorithm identification
3. Wasted computation analysis
4. Architectural bottleneck mapping
5. Unnecessary complexity flagging
6. Missed abstraction opportunities
7. Pipeline inefficiency detection
8. Orchestration waste analysis
9. Data flow entropy measurement
10. Configuration drift detection
11. Dead path elimination
12. Over-engineering detection
13. Under-engineering detection
14. State machine defect analysis
15. Retry/fallback anti-pattern detection
16. Caching opportunity identification
17. Parallelization gap analysis
18. Verdict synthesis
19. Fix plan generation
20. Fix execution + re-audit

## Scoring

- Max raw score: 360
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected for optimization/refactor missions
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
