---
name: uiuxaudit
description: "Art Director-grade UI/UX forensic audit. 23 phases, scored /420 (normalized /100). Gestalt-Popper doctrine."
domain: design
phases: 23
max_score: 420
triggers: ["ux", "ui", "ui/ux", "design audit", "audit design", "audit visuel"]
read_only: false
---

# /uiuxaudit — UI/UX Design Audit

**What it answers:** Is the interface BEAUTIFUL?

## Invocation

```bash
omega audit run uiuxaudit --dir <project-path>
# Or via agent skill:
/uiuxaudit --url=<url> --scope="<description>"
```

## Phases (23)

1. Pixel-level design coherence
2. Cross-page consistency
3. Typography hierarchy
4. Color system integrity
5. Spacing rhythm analysis
6. Component anatomy
7. Interaction patterns
8. Motion design evaluation
9. Responsive fidelity
10. Accessibility compliance
11. Visual hierarchy scoring
12. Information density
13. Empty state design
14. Loading state design
15. Error state design
16. Icon consistency
17. Brand alignment
18. Whitespace balance
19. Contrast ratios
20. Touch target sizing
21. Verdict synthesis
22. Fix plan generation
23. Fix execution + re-audit

## Scoring

- Max raw score: 420
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves UI changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- ABORTS on non-UI projects (CLI, library, backend-only, headless)

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
