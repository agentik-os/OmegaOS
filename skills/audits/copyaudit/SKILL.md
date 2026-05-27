---
name: copyaudit
description: "Forensic copy and messaging audit. 19 phases, scored /280 (normalized /100). Gestalt-Popper doctrine."
domain: copy
phases: 19
max_score: 280
triggers: ["copy audit", "messaging audit", "check the copy", "review the text", "tone check"]
read_only: false
---

# /copyaudit — Copy & Messaging Audit

**What it answers:** Is the copy CLEAR?

## Invocation

```bash
omega audit run copyaudit --dir <project-path>
# Or via agent skill:
/copyaudit --url=<url> --scope="<description>"
```

## Phases (19)

1. Headline clarity (5-second test)
2. Value proposition accuracy
3. CTA effectiveness
4. Claim verification (promises vs reality)
5. Tone consistency
6. Technical accuracy
7. Grammar/spelling check
8. Reading level (Flesch-Kincaid)
9. SEO keyword integration
10. Social proof accuracy
11. Legal compliance
12. Microcopy (buttons, errors, empty states)
13. Accessibility of copy
14. Brand voice adherence
15. Verdict synthesis
16. Fix plan generation
17. Fix execution
18. Re-audit
19. i18n wrapping detection

## Scoring

- Max raw score: 280
- Normalized: /100
- Threshold for PASS: 70/100

## Integration

- Oracle end-of-mission: auto-selected when mission involves copy changes
- Quality gate: score feeds into GateResult.audit_pass
- Results: written to `~/.omega/state/<mission-id>.audit-results.json`
- Owns hardcoded string scanning; `/a11yaudit` owns rendered-locale verification

## Protocol

The full forensic protocol is loaded as a Claude Code skill.
When installed via `omega sync`, the skill is symlinked into `~/.claude/commands/`.
