---
name: retentionaudit
description: "Forensic retention and opportunity audit. 20 phases, scored /400 (normalized /100). Gestalt-Popper doctrine. READ-ONLY."
domain: retention
phases: 20
max_score: 400
triggers: ["retention", "feature opportunities", "make it sticky", "what's missing for retention", "cpo mindset"]
read_only: true
---

# /retentionaudit — Retention & Opportunity Audit

**What it answers:** What FEATURES are missing? (READ-ONLY)

## Invocation

```bash
omega audit run retentionaudit --dir <project-path>
# Or via agent skill:
/retentionaudit --url=<url> --scope="<description>"
```

## Phases (20)

1. User-journey gap inventory
2. Drop-off forensics
3. Aha-moment latency analysis
4. Hook strength evaluation (Hooked — Eyal)
5. Personalization debt assessment
6. Onboarding completeness review
7. Empty-state design evaluation
8. Network effects analysis
9. Sales angle identification
10. Monetization hook discovery
11. Friction surface mapping
12. Reactivation flow evaluation
13. Community surface analysis
14. Discoverability assessment
15. Power-user delight features
16. Jobs-To-Be-Done mapping (Christensen)
17. Power of Moments analysis (Heath)
18. Fogg B=MAT adoption likelihood
19. RICE-prioritized roadmap
20. Verdict synthesis

## Scoring

- Max raw score: 400
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: manually invoked (not auto-selected)
- READ-ONLY: proposes ideas with RICE scoring, never edits code
- Hands off to `/planner` or `/implement` for execution
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
