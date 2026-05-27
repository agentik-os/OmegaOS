---
name: secaudit
description: "Forensic security audit. 25 phases, scored /400 (normalized /100). Gestalt-Popper doctrine."
domain: security
phases: 25
max_score: 400
triggers: ["sec", "security", "owasp", "vulnerab", "pentest-light", "is it secure"]
read_only: false
---

# /secaudit — Security Audit

**What it answers:** Is it SECURE?

## Invocation

```bash
omega audit run secaudit --dir <project-path>
# Or via agent skill:
/secaudit --files=<globs> --scope="<description>"
```

## Phases (25)

1. OWASP Top 10 verification
2. XSS testing (25+ payload patterns)
3. SQL/NoSQL injection testing
4. CORS misconfiguration detection
5. CSP headers audit
6. Authentication bypass testing
7. Session management review
8. JWT security analysis
9. IDOR detection
10. SSRF probing
11. Open redirect testing
12. File upload vulnerability check
13. Rate limiting verification
14. Brute force protection check
15. Secrets scanning (env, git history, JS bundles)
16. Dependency CVE audit
17. SSL/TLS configuration review
18. Security headers audit
19. API authentication verification
20. Input validation completeness
21. Verdict synthesis
22. Fix plan generation
23. Fix execution
24. Re-audit
25. Rate-limit safety gate

## Scoring

- Max raw score: 400
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves auth or security
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Consumes `/apiaudit` output for auth surface exploitation

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
