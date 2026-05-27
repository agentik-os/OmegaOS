---
name: motionaudit
description: "Forensic motion design audit. 23 phases, scored /360 (normalized /100). Gestalt-Popper doctrine."
domain: motion
phases: 23
max_score: 360
triggers: ["motion audit", "animation audit", "why does it feel lifeless", "easing audit"]
read_only: false
---

# /motionaudit — Motion Design Audit

**What it answers:** Is the motion PURPOSEFUL?

## Invocation

```bash
omega audit run motionaudit --dir <project-path>
# Or via agent skill:
/motionaudit --url=<url> --scope="<description>"
```

## Phases (23)

1. CSS transition inventory
2. JS animation audit
3. WebGL effects review
4. Scroll-driven choreography
5. P5.js/canvas analysis
6. Page transition evaluation
7. Micro-interaction catalog
8. Loading sequence design
9. Easing system consistency
10. Duration consistency
11. Choreography composition
12. Reduced-motion compliance
13. Mobile motion performance
14. Brand motion DNA
15. Performance budget check
16. Animation frame rate profiling
17. GPU compositing analysis
18. Interaction feedback timing
19. Verdict synthesis
20. Fix plan generation
21. Fix execution
22. Re-audit
23. Integration smoke gate

## Scoring

- Max raw score: 360
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves animation/motion
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- ABORTS on non-UI projects (CLI, library, backend-only, headless)

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
