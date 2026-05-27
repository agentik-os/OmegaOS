---
name: apiaudit
description: "Forensic API quality audit. 23 phases, scored /360 (normalized /100). Gestalt-Popper doctrine."
domain: api
phases: 23
max_score: 360
triggers: ["api audit", "audit api", "api contracts", "endpoint audit", "api quality"]
read_only: false
---

# /apiaudit — API Quality Audit

**What it answers:** Is the API SOLID?

## Invocation

```bash
omega audit run apiaudit --dir <project-path>
# Or via agent skill:
/apiaudit --files=<globs> --scope="<description>"
```

## Phases (23)

1. Endpoint inventory
2. REST/GraphQL contract compliance
3. Authentication verification (every endpoint)
4. Authorization (role-based access)
5. Input validation (every parameter)
6. Error response format check
7. Status code correctness
8. Pagination audit
9. Rate limiting verification
10. Versioning strategy review
11. Documentation accuracy
12. Response time benchmarking
13. N+1 detection
14. Idempotency verification
15. Webhook reliability testing
16. CORS configuration check
17. Content negotiation review
18. API deprecation handling
19. Verdict synthesis
20. Fix plan generation
21. Fix execution
22. Re-audit
23. Rate-limit safety gate

## Scoring

- Max raw score: 360
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves API changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Reads `/dataaudit` schema types when available
- Outputs auth surface consumed by `/secaudit`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
