---
name: automationaudit
description: "Forensic automation infrastructure audit. 22 phases, scored /330 (normalized /100). Gestalt-Popper doctrine."
domain: automation
phases: 22
max_score: 330
triggers: ["automation", "cron", "crontab", "scripts audit", "daemon health", "scheduled tasks"]
read_only: false
---

# /automationaudit — Automation Infrastructure Audit

**What it answers:** Is automation RELIABLE?

## Invocation

```bash
omega audit run automationaudit --dir <project-path>
# Or via agent skill:
/automationaudit --scope="<description>"
```

## Phases (22)

1. Cron job inventory
2. Shell script quality audit
3. Python script analysis
4. Daemon health check
5. Systemd timer review
6. CI/CD pipeline quality
7. Dispatch chain analysis
8. Orchestration logic review
9. Scheduling order verification
10. Dependency graph validation
11. Error recovery audit
12. Log rotation check
13. Dead automation detection
14. Race condition detection
15. Secret exposure scanning
16. Idempotency verification
17. Silent failure detection
18. Monitoring gap analysis
19. Verdict synthesis
20. Fix plan generation
21. Fix execution
22. Re-audit

## Scoring

- Max raw score: 330
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves scripts or scheduling
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
