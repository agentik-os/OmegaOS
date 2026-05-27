---
name: codeaudit
description: "CIA-grade code architecture audit. 23 phases, scored /420 (normalized /100). Gestalt-Popper doctrine."
domain: code
phases: 23
max_score: 420
triggers: ["code", "code audit", "audit code", "code quality", "code review everything"]
read_only: false
---

# /codeaudit — Code Architecture Audit

**What it answers:** Is the code SOLID?

## Invocation

```bash
omega audit run codeaudit --dir <project-path>
# Or via agent skill:
/codeaudit --files=<globs> --scope="<description>"
```

## Phases (23)

1. Phantom detection
2. Dependency dissection
3. Contract interrogation
4. Data flow tracing
5. State mutation analysis
6. Concurrency autopsy
7. Blast radius mapping
8. Time bomb hunting
9. Supply chain forensics
10. Error propagation tracing
11. Behavioral fingerprinting
12. Configuration drift detection
13. Feature verification
14. Entropy analysis
15. Git criminal profiling
16. Runtime vivisection
17. Observability audit
18. Test coverage analysis
19. API contract verification
20. Resilience testing
21. Verdict synthesis
22. Fix plan generation
23. Fix execution + re-audit

## Scoring

- Max raw score: 420
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves code changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
