# Methodological foundations

Builder {OS} borrows useful discipline from established evidence-synthesis and evaluation practices without pretending that every domain is clinical research.

## Transparent review protocols

PRISMA provides structured reporting expectations for systematic reviews, including explicit rationale, objectives, methods, selection and reporting. Builder adapts this transparency principle into `RESEARCH_PROTOCOL.yaml`, source inventories and release reports.

Reference: https://www.prisma-statement.org/prisma-2020-checklist

## Broad search and selection

The Cochrane Handbook emphasizes extensive, explicit searching and selection to reduce missed evidence and reporting bias. Builder adapts this through multi-source discovery, retained/rejected logs and corpus saturation.

Reference: https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-04

## Certainty and recommendation separation

GRADE separates certainty in evidence from the strength of a recommendation. Builder similarly separates source confidence, context and evidence from its own design choices and operational recommendations.

Reference: https://www.cochrane.org/learn/courses-and-resources/cochrane-methodology/grade-approach/grade-handbook

## Structured evaluations

Modern agent systems require structured tests that define inputs, expected behavior and grading logic. Builder requires deterministic validators, rubric-based graders, adversarial tests and regression suites.

Reference: https://developers.openai.com/api/docs/guides/evaluation-best-practices

## Long-horizon implementation

Codex supports dedicated environments, parallel work and reviewable diffs for longer tasks. Builder uses durable plans, milestone validation, isolated workers and explicit status artifacts for large OS builds.

References:

- https://developers.openai.com/codex/cloud
- https://developers.openai.com/codex/subagents
- https://developers.openai.com/blog/run-long-horizon-tasks-with-codex
