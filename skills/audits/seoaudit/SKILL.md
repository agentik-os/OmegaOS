---
name: seoaudit
description: "Forensic SEO audit. 25 phases, scored /400 (normalized /100). Gestalt-Popper doctrine."
domain: seo
phases: 25
max_score: 400
triggers: ["seo", "seo audit", "crawlability", "search optimization", "organic traffic"]
read_only: false
---

# /seoaudit — SEO Audit

**What it answers:** Is it DISCOVERABLE?

## Invocation

```bash
omega audit run seoaudit --dir <project-path>
# Or via agent skill:
/seoaudit --url=<url> --scope="<description>"
```

## Phases (25)

1. Crawlability (robots.txt, meta robots, canonical)
2. Indexability (sitemap, internal links)
3. Core Web Vitals (LCP, FID, CLS via /perfaudit)
4. Schema.org markup validation
5. Meta tags audit
6. Heading hierarchy check
7. Image SEO review
8. URL structure analysis
9. Mobile-friendliness test
10. Page speed evaluation
11. Content quality (E-E-A-T)
12. Internal linking analysis
13. External links review
14. Hreflang verification
15. Pagination audit
16. Redirect chain detection
17. 404/broken link detection
18. JavaScript rendering check
19. GEO/AEO (AI search optimization)
20. Competitor SERP analysis
21. Verdict synthesis
22. Fix plan generation
23. Fix execution
24. Re-audit
25. Integration smoke gate

## Scoring

- Max raw score: 400
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves content or pages
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Reads `/perfaudit` CWV data when available

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
