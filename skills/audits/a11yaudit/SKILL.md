---
name: a11yaudit
description: "Forensic accessibility audit. 21 phases, scored /320 (normalized /100). Gestalt-Popper doctrine."
domain: accessibility
phases: 21
max_score: 320
triggers: ["a11y", "accessibility", "wcag", "keyboard navigation", "screen reader"]
read_only: false
---

# /a11yaudit — Accessibility Audit

**What it answers:** Is it ACCESSIBLE?

## Invocation

```bash
omega audit run a11yaudit --dir <project-path>
# Or via agent skill:
/a11yaudit --url=<url> --scope="<description>"
```

## Phases (21)

1. WCAG 2.1 AA compliance check
2. Keyboard navigation (every interactive element)
3. Screen reader testing
4. ARIA labels/roles/states verification
5. Color contrast (4.5:1 AA, 3:1 large text)
6. Focus management audit
7. Skip navigation check
8. Form labels verification
9. Error announcements testing
10. Alt text completeness
11. Heading hierarchy validation
12. Landmark regions check
13. Touch targets (44px minimum)
14. Motion/animation preferences
15. Cognitive load assessment
16. Reading level evaluation
17. Verdict synthesis
18. Fix plan generation
19. Fix execution
20. Re-audit
21. Integration smoke gate

## Scoring

- Max raw score: 320
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves UI
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- ABORTS on non-UI projects (CLI, library, backend-only, headless)

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
