---
name: flowaudit
description: "Forensic user flow audit. 25 phases, scored /400 (normalized /100). Gestalt-Popper doctrine."
domain: flows
phases: 25
max_score: 400
triggers: ["flow", "user flow", "flow audit", "parcours", "workflow audit"]
read_only: false
---

# /flowaudit — User Flow Audit

**What it answers:** Does the experience WORK?

## Invocation

```bash
omega audit run flowaudit --dir <project-path>
# Or via agent skill:
/flowaudit --url=<url> --scope="<description>"
```

## Phases (25)

1. Flow mapping
2. State verification
3. Dead-end detection
4. Permission gap analysis
5. Data integrity through flows
6. Onboarding completeness
7. Cross-session continuity
8. Error recovery paths
9. Accessibility of journeys
10. Flow performance
11. Edge case inventory
12. Multi-user flow conflicts
13. Offline/degraded mode
14. Progressive disclosure
15. Navigation consistency
16. Form flow validation
17. Redirect chain analysis
18. Deep link integrity
19. Back button behavior
20. Timeout handling
21. Concurrent flow testing
22. Verdict synthesis
23. Fix plan generation
24. Fix execution
25. Re-audit + integration gate

## Scoring

- Max raw score: 400
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves user-facing changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
