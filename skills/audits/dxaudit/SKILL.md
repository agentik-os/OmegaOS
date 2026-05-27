---
name: dxaudit
description: "Forensic developer experience audit. 21 phases, scored /320 (normalized /100). Gestalt-Popper doctrine."
domain: dx
phases: 21
max_score: 320
triggers: ["dx audit", "developer experience", "onboarding audit", "setup complexity"]
read_only: false
---

# /dxaudit — Developer Experience Audit

**What it answers:** Is the DX SMOOTH?

## Invocation

```bash
omega audit run dxaudit --dir <project-path>
# Or via agent skill:
/dxaudit --files=<globs> --scope="<description>"
```

## Phases (21)

1. README quality (can a new dev start in <10min)
2. Setup complexity (steps to run locally)
3. Error message quality (actionable vs cryptic)
4. TypeScript strictness
5. Code documentation (JSDoc on public APIs)
6. Testing infrastructure
7. CI/CD pipeline quality
8. PR template/process
9. Dependency management
10. Monorepo structure
11. Dev tooling (linting, formatting, pre-commit)
12. Environment parity (dev/staging/prod)
13. Debug tooling
14. Migration guides
15. Changelog maintenance
16. Contribution guide
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

- Oracle end-of-mission: auto-selected for infrastructure/tooling missions
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Primary audit for CLI/library projects (replaces /uiuxaudit + /flowaudit + /motionaudit which ABORT on non-UI)

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
