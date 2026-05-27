---
name: dataaudit
description: "Forensic data integrity audit. 21 phases, scored /320 (normalized /100). Gestalt-Popper doctrine."
domain: data
phases: 21
max_score: 320
triggers: ["data integrity", "schema", "data audit", "database audit", "orphaned records"]
read_only: false
---

# /dataaudit — Data Integrity Audit

**What it answers:** Is the data INTACT?

## Invocation

```bash
omega audit run dataaudit --dir <project-path>
# Or via agent skill:
/dataaudit --files=<globs> --scope="<description>"
```

## Phases (21)

1. Schema validation
2. Migration status check
3. Orphaned record detection
4. Referential integrity verification
5. Data consistency analysis
6. Type safety (runtime vs schema)
7. Null handling audit
8. Duplicate detection
9. Cascade behavior review
10. Backup verification
11. Query performance profiling
12. Index coverage analysis
13. Data lifecycle (TTL, archival)
14. PII detection
15. Seed data separation
16. Transaction integrity
17. Verdict synthesis
18. Fix plan generation
19. Fix execution
20. Re-audit
21. DB backup safety gate

## Scoring

- Max raw score: 320
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves database changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Outputs schema types consumed by `/apiaudit`
- DESTRUCTIVE audit: verifies backup exists before any write operation

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
