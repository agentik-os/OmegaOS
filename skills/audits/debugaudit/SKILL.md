---
name: debugaudit
description: "Forensic runtime bug hunter. 23 phases, scored /360 (normalized /100). Gestalt-Popper doctrine."
domain: runtime
phases: 23
max_score: 360
triggers: ["debug", "runtime bug", "debugaudit", "find bugs", "what's broken", "hunt bugs"]
read_only: false
---

# /debugaudit — Runtime Bug Hunter

**What it answers:** What is BROKEN right now?

## Invocation

```bash
omega audit run debugaudit --dir <project-path>
# Or via agent skill:
/debugaudit --url=<url> --scope="<description>"
```

## Phases (23)

1. Console error inventory
2. Network failure detection
3. Visual regression scan
4. Security injection testing
5. Responsive breakage detection
6. Performance bottleneck profiling
7. Dead feature detection
8. Race condition hunting
9. State corruption analysis
10. Memory leak detection
11. Event handler audit
12. Form validation testing
13. API response verification
14. WebSocket stability
15. Third-party integration health
16. Cookie/storage integrity
17. Auth flow verification
18. Error boundary testing
19. Hydration mismatch detection
20. Redirect loop detection
21. Verdict synthesis
22. Fix plan generation
23. Fix execution + re-audit + integration gate

## Scoring

- Max raw score: 360
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected for bug-fix missions
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
