---
name: featureaudit
description: "Forensic feature completeness audit. 19 phases, scored /320 (normalized /100). Gestalt-Popper doctrine."
domain: features
phases: 19
max_score: 320
triggers: ["feature", "feature audit", "completeness", "what's missing", "feature gap"]
read_only: false
---

# /featureaudit — Feature Completeness Audit

**What it answers:** Is the product COMPLETE?

## Invocation

```bash
omega audit run featureaudit --dir <project-path>
# Or via agent skill:
/featureaudit --files=<globs> --scope="<description>"
```

## Phases (19)

1. PRD gap analysis
2. Competitive parity check
3. Feature depth scoring
4. Discoverability evaluation
5. Feature coherence analysis
6. Edge case completeness
7. API surface gap detection
8. Missing obvious capabilities
9. Configuration completeness
10. User role coverage
11. Notification completeness
12. Search/filter completeness
13. Export/import capabilities
14. Bulk operation support
15. Undo/redo coverage
16. Verdict synthesis
17. Implementation plan
18. Execution
19. Re-audit

## Scoring

- Max raw score: 320
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected for feature-build missions
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
