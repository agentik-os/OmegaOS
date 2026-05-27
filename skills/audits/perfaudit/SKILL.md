---
name: perfaudit
description: "Forensic performance audit. 23 phases, scored /360 (normalized /100). Gestalt-Popper doctrine."
domain: performance
phases: 23
max_score: 360
triggers: ["perf", "performance", "core web vitals", "why is it slow", "speed audit"]
read_only: false
---

# /perfaudit — Performance Audit

**What it answers:** Is it FAST enough?

## Invocation

```bash
omega audit run perfaudit --dir <project-path>
# Or via agent skill:
/perfaudit --url=<url> --scope="<description>"
```

## Phases (23)

1. Core Web Vitals measurement
2. Bundle size analysis
3. Render performance profiling
4. JavaScript execution audit
5. Image optimization check
6. Font loading strategy
7. Caching strategy evaluation
8. CDN configuration review
9. SSR/SSG analysis
10. Lazy loading audit
11. Code splitting evaluation
12. API response time benchmarking
13. N+1 query detection
14. Database query performance
15. Memory leak detection
16. Connection pooling review
17. Resource hints audit
18. Third-party script impact
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

- Oracle end-of-mission: auto-selected when mission involves performance
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Outputs CWV data consumed by `/seoaudit`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
